//! The local store (`<state-dir>/sessions.db`, SQLite): per main transcript the redacted lifted
//! commands, and the rows of `sessionspec.view` that `sync` projects from what the generated
//! component publishes. Readers replay the stored commands instead of parsing transcripts again;
//! `serve` answers the views from the rows. The store lives outside the repository: it holds the
//! harness's session titles, which a local UI may show and which never leave the machine.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::{Instant, UNIX_EPOCH};

use rayon::prelude::*;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use sessionspec_server::{json, wire};
use sessionspec_types::cli as c;
use sessionspec_types::primitives::{Binary64, Timestamp};
use sessionspec_types::session as s;
use sessionspec_types::view as v;

use crate::claude::{Actor, Cmd, Lifted, Step};

/// Raised when what the store holds changes meaning; a store of another version is emptied and
/// filled again by the next sync.
pub const SCHEMA_VERSION: i64 = 4;

/// The zstd level of the `steps` and `events` blobs.
const LEVEL: i32 = 3;

fn pack(text: &str) -> Vec<u8> {
    zstd::encode_all(text.as_bytes(), LEVEL).unwrap_or_default()
}

fn unpack(blob: &[u8]) -> io::Result<String> {
    let bytes = zstd::decode_all(blob)?;
    String::from_utf8(bytes).map_err(io::Error::other)
}

/// The checks every synced session is judged by (`sessionspec.view.SessionChecks`).
pub const CHECKS: &str = include_str!("../../../queries/session.sq");

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS sources (
  path TEXT PRIMARY KEY, session_id TEXT NOT NULL, fingerprint TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS sessions (
  session_id TEXT PRIMARY KEY, path TEXT NOT NULL, title TEXT, agents INTEGER NOT NULL,
  files TEXT NOT NULL, steps BLOB NOT NULL, events_count INTEGER NOT NULL,
  last_ms INTEGER NOT NULL, row TEXT NOT NULL, kpis TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS event_chunks (
  session_id TEXT NOT NULL, chunk INTEGER NOT NULL, events BLOB NOT NULL,
  PRIMARY KEY (session_id, chunk));
CREATE TABLE IF NOT EXISTS agents (
  session_id TEXT NOT NULL, agent_id TEXT NOT NULL, started_ms INTEGER NOT NULL,
  row TEXT NOT NULL, PRIMARY KEY (session_id, agent_id));
CREATE TABLE IF NOT EXISTS checks (
  session_id TEXT NOT NULL, name TEXT NOT NULL, about TEXT NOT NULL, severity TEXT NOT NULL,
  verdict TEXT NOT NULL, rows INTEGER NOT NULL, possible INTEGER NOT NULL,
  PRIMARY KEY (session_id, name));
CREATE TABLE IF NOT EXISTS wire_steps (
  session_id TEXT NOT NULL, seq INTEGER NOT NULL, harness TEXT NOT NULL, step TEXT NOT NULL,
  PRIMARY KEY (session_id, seq));
CREATE INDEX IF NOT EXISTS sessions_last ON sessions (last_ms);
";

/// `$XDG_DATA_HOME/sessionspec`, else `~/.local/share/sessionspec`.
pub fn default_dir() -> PathBuf {
    match std::env::var("XDG_DATA_HOME") {
        Ok(d) if !d.is_empty() => PathBuf::from(d).join("sessionspec"),
        _ => PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".local/share/sessionspec"),
    }
}

/// The store in `state_dir`, or in [`default_dir`].
pub fn path_in(state_dir: Option<&Path>) -> PathBuf {
    state_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(default_dir)
        .join("sessions.db")
}

/// A stored session as `load` reads it: id, title, agents, files, packed steps.
type StoredSession = (String, Option<String>, i64, String, Vec<u8>);

pub struct Store {
    path: PathBuf,
}

/// What one sync wrote: the result the command line prints and the sessions it changed.
pub struct Synced {
    pub result: c::SyncResult,
    pub changed: Vec<String>,
}

fn sql(e: rusqlite::Error) -> String {
    e.to_string()
}

impl Store {
    /// Opens the store at `path`, creating it and its directory; a store of another schema
    /// version is emptied.
    pub fn open(path: &Path) -> Result<Store, String> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let store = Store {
            path: path.to_path_buf(),
        };
        let conn = store.conn().map_err(sql)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .map_err(sql)?;
        let version: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key = 'schema'", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(sql)?;
        if version.as_deref() != Some(&SCHEMA_VERSION.to_string()) {
            conn.execute_batch(
                "DROP TABLE IF EXISTS sources; DROP TABLE IF EXISTS sessions;
                 DROP TABLE IF EXISTS agents; DROP TABLE IF EXISTS events;
                 DROP TABLE IF EXISTS checks;
                 DROP TABLE IF EXISTS wire_steps; DROP TABLE IF EXISTS event_chunks;",
            )
            .map_err(sql)?;
            conn.execute_batch(SCHEMA).map_err(sql)?;
            conn.execute_batch("VACUUM;").map_err(sql)?;
            conn.execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema', ?1)",
                params![SCHEMA_VERSION.to_string()],
            )
            .map_err(sql)?;
        }
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn conn(&self) -> rusqlite::Result<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.busy_timeout(std::time::Duration::from_secs(10))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(conn)
    }

    fn read(&self) -> rusqlite::Result<Connection> {
        let conn = Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(std::time::Duration::from_secs(10))?;
        Ok(conn)
    }

    /// Lifts the main transcripts under `root` with a file modified at or after `from_ms` whose
    /// files changed since the last sync, and writes each in one transaction.
    pub fn sync(&self, root: &Path, from_ms: i64) -> Result<Synced, String> {
        let started = Instant::now();
        let mains = crate::kpi::sessions_in(root, from_ms)
            .map_err(|e| format!("{}: {e}", root.display()))?;
        let conn = self.conn().map_err(sql)?;
        let known: HashMap<String, String> = {
            let mut st = conn
                .prepare("SELECT path, fingerprint FROM sources")
                .map_err(sql)?;
            st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(sql)?
                .collect::<rusqlite::Result<_>>()
                .map_err(sql)?
        };
        let todo: Vec<(PathBuf, String)> = mains
            .iter()
            .map(|(p, _)| (p.clone(), fingerprint(p)))
            .filter(|(p, f)| known.get(&p.display().to_string()) != Some(f))
            .collect();
        let program = crate::sq::compile(CHECKS)?;
        let records: Vec<Result<Record, String>> = todo
            .par_iter()
            .map(|(p, f)| record(p, f, &program))
            .collect();
        let mut conn = conn;
        let (mut lifted, mut failed) = (0i64, 0i64);
        let mut changed = Vec::new();
        for r in records {
            match r {
                Ok(rec) => {
                    write(&mut conn, &rec).map_err(sql)?;
                    changed.push(rec.session_id.clone());
                    lifted += 1;
                }
                Err(_) => failed += 1,
            }
        }
        let sessions: i64 = conn
            .query_row("SELECT count(*) FROM sessions", [], |r| r.get(0))
            .map_err(sql)?;
        Ok(Synced {
            result: c::SyncResult {
                db: self.path.display().to_string(),
                scanned: mains.len() as i64,
                lifted,
                unchanged: mains.len() as i64 - todo.len() as i64,
                failed,
                sessions,
                ms: started.elapsed().as_millis() as i64,
            },
            changed,
        })
    }

    /// The lifted commands of a main transcript as the last sync stored them.
    pub fn load(&self, main: &Path) -> io::Result<Lifted> {
        let conn = self.read().map_err(io::Error::other)?;
        let row: Option<StoredSession> = conn
            .query_row(
                "SELECT session_id, title, agents, files, steps FROM sessions WHERE path = ?1",
                params![main.display().to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(io::Error::other)?;
        let Some((session_id, title, agents, files, steps)) = row else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not in the store", main.display()),
            ));
        };
        // A wire session keeps its steps as `wire_steps` rows.
        let steps = if main.to_string_lossy().starts_with("wire:") {
            self.wire_steps(&session_id).map_err(io::Error::other)?
        } else {
            decode_steps(&unpack(&steps)?).map_err(io::Error::other)?
        };
        Ok(Lifted {
            session_id,
            title,
            agents: agents as usize,
            files: files.lines().map(PathBuf::from).collect(),
            steps,
        })
    }

    /// Every session row, newest `last_at` first, with its rank.
    pub fn sessions(&self) -> rusqlite::Result<Vec<v::SessionUpdated>> {
        let conn = self.read()?;
        let mut st = conn.prepare("SELECT row FROM sessions ORDER BY last_ms DESC, session_id")?;
        let rows: Vec<String> = st
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let mut row = decode(t, wire::decode_event_sessionspec_view_session_updated)?;
                row.rank = i as i64 + 1;
                Some(row)
            })
            .collect())
    }

    pub fn session_kpis(&self, session_id: &str) -> rusqlite::Result<Option<c::KpiReport>> {
        let conn = self.read()?;
        let t: Option<String> = conn
            .query_row(
                "SELECT kpis FROM sessions WHERE session_id = ?1",
                params![session_id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(t.and_then(|t| decode(&t, wire::decode_sessionspec_cli_kpi_report)))
    }

    pub fn agents(&self, session_id: &str) -> rusqlite::Result<Vec<v::AgentUpdated>> {
        let conn = self.read()?;
        let mut st = conn.prepare(
            "SELECT row FROM agents WHERE session_id = ?1 ORDER BY started_ms, agent_id",
        )?;
        let rows: Vec<String> = st
            .query_map(params![session_id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows
            .iter()
            .filter_map(|t| decode(t, wire::decode_event_sessionspec_view_agent_updated))
            .collect())
    }

    /// The events of a session from `from_seq` on, of one kind and agent when given.
    pub fn events(
        &self,
        session_id: &str,
        from_seq: i64,
        kind: Option<v::EventKind>,
        agent: Option<&str>,
    ) -> rusqlite::Result<Vec<v::EventAppended>> {
        let conn = self.read()?;
        let text = self.event_text(&conn, session_id)?;
        let kind = kind.map(|k| format!("{k:?}"));
        Ok(text
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(4, '\t');
                let seq: i64 = parts.next()?.parse().ok()?;
                let k = parts.next()?;
                let a = parts.next()?;
                if seq < from_seq
                    || kind.as_deref().is_some_and(|x| x != k)
                    || agent.is_some_and(|x| x != a)
                {
                    return None;
                }
                decode(
                    parts.next()?,
                    wire::decode_event_sessionspec_view_event_appended,
                )
            })
            .collect())
    }

    /// How many events each session holds, by session.
    pub fn event_counts(&self) -> rusqlite::Result<HashMap<String, i64>> {
        let conn = self.read()?;
        let mut st = conn.prepare("SELECT session_id, events_count FROM sessions")?;
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect()
    }

    pub fn checks(&self, session_id: &str) -> rusqlite::Result<Vec<v::SessionChecks>> {
        let conn = self.read()?;
        let mut st = conn.prepare(
            "SELECT name, about, severity, verdict, rows, possible FROM checks
             WHERE session_id = ?1 ORDER BY name",
        )?;
        st.query_map(params![session_id], |r| {
            let name: String = r.get(0)?;
            let verdict: String = r.get(3)?;
            Ok(v::SessionChecks {
                check_id: v::CheckId(format!("{session_id}.{name}")),
                session_id: s::SessionId(session_id.to_owned()),
                check: name,
                about: r.get(1)?,
                severity: r.get(2)?,
                verdict: match verdict.as_str() {
                    "Gap" => s::Verdict::Gap,
                    "Unk" => s::Verdict::Unk,
                    _ => s::Verdict::Ok,
                },
                rows: r.get(4)?,
                possible: r.get(5)?,
            })
        })?
        .collect()
    }

    /// Appends the steps a gateway lifted from its calls (`crate::wire::lift_call`) to the stream
    /// of the session named by `session_hash`, and extends its projection by them: the new event
    /// rows, the session's and agents' rows and KPIs as they now stand. The session's source key
    /// is `wire:<session hash>`.
    ///
    /// `open` keeps each open session's projection, so an append costs what its own steps cost.
    /// A session `open` does not hold, or whose new steps start before its first instant, is
    /// projected again from every stored step. The check verdicts run over the whole session;
    /// they are brought up to date by [`Store::flush_checks`], and by an append once they lag
    /// more than 30 seconds.
    pub fn append(
        &self,
        session_hash: &str,
        harness: s::Harness,
        steps: &[Step],
        program: &crate::sq::Program,
        open: &mut Projections,
    ) -> Result<(), String> {
        let mut conn = self.conn().map_err(sql)?;
        let tx = conn.transaction().map_err(sql)?;
        let next: i64 = tx
            .query_row(
                "SELECT coalesce(max(seq) + 1, 0) FROM wire_steps WHERE session_id = ?1",
                params![session_hash],
                |r| r.get(0),
            )
            .map_err(sql)?;
        {
            let mut st = tx
                .prepare(
                    "INSERT INTO wire_steps (session_id, seq, harness, step)
                     VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(sql)?;
            for (i, step) in steps.iter().enumerate() {
                st.execute(params![
                    session_hash,
                    next + i as i64,
                    format!("{harness:?}"),
                    encode_steps(std::slice::from_ref(step)).trim_end()
                ])
                .map_err(sql)?;
            }
        }
        tx.commit().map_err(sql)?;

        open.tick += 1;
        let tick = open.tick;
        if let Some((t, p)) = open.open.get_mut(session_hash) {
            *t = tick;
            let started = steps
                .iter()
                .filter(|st| matches!(st.cmd, Cmd::StartAgent(_)))
                .count();
            if p.extend(steps.to_vec()).is_ok() {
                p.lifted.agents += started;
                let lagging = p.dirty_since.is_some_and(|d| d.elapsed() >= CHECKS_LAG);
                let d = p.delta(program, lagging);
                return write_delta(&mut conn, &d).map_err(sql);
            }
            open.open.remove(session_hash);
        }

        // Not open, or the fold cannot continue: the whole stream from the store.
        let all = self.wire_steps(session_hash)?;
        let lifted = Lifted {
            session_id: session_hash.to_owned(),
            title: None,
            agents: all
                .iter()
                .filter(|st| matches!(st.cmd, Cmd::StartAgent(_)))
                .count(),
            files: Vec::new(),
            steps: all,
        };
        let mut p = Projection::new(
            lifted,
            Origin {
                path: format!("wire:{session_hash}"),
                fingerprint: String::new(),
                project: String::new(),
                project_dir: String::new(),
                harness,
            },
        );
        let rec = p.full_record(program);
        write(&mut conn, &rec).map_err(sql)?;
        if open.open.len() >= open.capacity
            && let Some(oldest) = open
                .open
                .iter()
                .min_by_key(|(_, (t, _))| *t)
                .map(|(k, _)| k.clone())
            && let Some((_, mut gone)) = open.open.remove(&oldest)
            && gone.checks_dirty
        {
            let checks = gone.checks(program);
            write_checks(&mut conn, &gone.lifted.session_id, &checks).map_err(sql)?;
        }
        open.open.insert(session_hash.to_owned(), (tick, p));
        Ok(())
    }

    /// Brings the check verdicts of every open session whose steps moved on up to date. The
    /// appender calls it when its queue is empty. Returns how many sessions it judged.
    pub fn flush_checks(
        &self,
        program: &crate::sq::Program,
        open: &mut Projections,
    ) -> Result<usize, String> {
        let mut conn = self.conn().map_err(sql)?;
        let mut n = 0;
        for (_, p) in open.open.values_mut() {
            if p.checks_dirty {
                let checks = p.checks(program);
                write_checks(&mut conn, &p.lifted.session_id, &checks).map_err(sql)?;
                n += 1;
            }
        }
        Ok(n)
    }

    /// Everything the store holds for a session, as stored text in a fixed order: the session
    /// row, the KPIs, the agent rows, the event lines and the verdicts. Two stores that hold the
    /// same projection answer the same text.
    pub fn projection_text(&self, session_id: &str) -> Result<String, String> {
        let conn = self.read().map_err(sql)?;
        let mut out = String::new();
        let (agents, count, last, row, kpis): (i64, i64, i64, String, String) = conn
            .query_row(
                "SELECT agents, events_count, last_ms, row, kpis FROM sessions
                 WHERE session_id = ?1",
                params![session_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .map_err(sql)?;
        let _ = writeln!(
            out,
            "agents {agents} events {count} last {last}\n{row}\n{kpis}"
        );
        let mut st = conn
            .prepare("SELECT agent_id, started_ms, row FROM agents WHERE session_id = ?1 ORDER BY agent_id")
            .map_err(sql)?;
        let rows: Vec<(String, i64, String)> = st
            .query_map(params![session_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .map_err(sql)?
            .collect::<rusqlite::Result<_>>()
            .map_err(sql)?;
        for (id, started, row) in rows {
            let _ = writeln!(out, "{id} {started} {row}");
        }
        out.push_str(&self.event_text(&conn, session_id).map_err(sql)?);
        for ch in self.checks(session_id).map_err(sql)? {
            let _ = writeln!(
                out,
                "{} {} {:?} {} {}",
                ch.check, ch.severity, ch.verdict, ch.rows, ch.possible
            );
        }
        Ok(out)
    }

    /// The event lines of a session, its chunks in order.
    fn event_text(&self, conn: &Connection, session_id: &str) -> rusqlite::Result<String> {
        let mut st =
            conn.prepare("SELECT events FROM event_chunks WHERE session_id = ?1 ORDER BY chunk")?;
        let blobs: Vec<Vec<u8>> = st
            .query_map(params![session_id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let mut text = String::new();
        for b in blobs {
            text.push_str(
                &unpack(&b).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            );
        }
        Ok(text)
    }

    /// The steps the wire appended for a session, in order.
    pub fn wire_steps(&self, session_hash: &str) -> Result<Vec<Step>, String> {
        let conn = self.conn().map_err(sql)?;
        let mut st = conn
            .prepare("SELECT step FROM wire_steps WHERE session_id = ?1 ORDER BY seq")
            .map_err(sql)?;
        let lines: Vec<String> = st
            .query_map(params![session_hash], |r| r.get(0))
            .map_err(sql)?
            .collect::<rusqlite::Result<_>>()
            .map_err(sql)?;
        decode_steps(&lines.join("\n"))
    }

    /// The paths the store holds, for readers that select by window themselves.
    pub fn paths(&self) -> rusqlite::Result<Vec<PathBuf>> {
        let conn = self.read()?;
        let mut st = conn.prepare("SELECT path FROM sessions ORDER BY path")?;
        st.query_map([], |r| r.get::<_, String>(0).map(PathBuf::from))?
            .collect()
    }
}

fn decode<T>(text: &str, f: fn(&json::Value, &str) -> Result<T, json::DecodeError>) -> Option<T> {
    let value = json::parse(text).ok()?;
    f(&value, "$").ok()
}

fn mtime_ns(p: &Path) -> u128 {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos())
}

/// The size and modification time of a main transcript and of each of its sub-agent files.
pub fn fingerprint(main: &Path) -> String {
    let mut out = String::new();
    let size = fs::metadata(main).map(|m| m.len()).unwrap_or(0);
    let _ = write!(out, "{size}:{}", mtime_ns(main));
    let mut subs: Vec<PathBuf> = crate::claude::sub_agent_files(main)
        .into_iter()
        .map(|s| s.path)
        .collect();
    subs.sort();
    for p in subs {
        let size = fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let _ = write!(out, ";{name}:{size}:{}", mtime_ns(&p));
    }
    out
}

/// One main transcript as the store holds it.
struct Record {
    path: String,
    fingerprint: String,
    session_id: String,
    title: Option<String>,
    agents_count: i64,
    files: String,
    steps: Vec<u8>,
    events: Vec<u8>,
    events_count: i64,
    last_ms: i64,
    row: String,
    kpis: String,
    agents: Vec<(String, i64, String)>,
    checks: Vec<crate::sq::SessionCheck>,
}

/// The working directory the first records of a transcript name, relative to the home
/// directory; the projects directory name when none does.
fn project_of(main: &Path) -> String {
    if let Ok(f) = fs::File::open(main) {
        for line in io::BufReader::new(f)
            .lines()
            .take(200)
            .map_while(Result::ok)
        {
            if !line.contains("\"cwd\"") {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line)
                && let Some(cwd) = val.get("cwd").and_then(serde_json::Value::as_str)
            {
                return home_relative(cwd);
            }
        }
    }
    main.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_owned()
}

fn home_relative(cwd: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && cwd.starts_with(&home) => {
            format!("~{}", &cwd[home.len()..])
        }
        _ => cwd.to_owned(),
    }
}

fn usd(micro: i64) -> Binary64 {
    Binary64::new(micro as f64 / 1_000_000.0).unwrap_or_else(|_| Binary64::new(0.0).expect("zero"))
}

fn share(x: f64) -> Binary64 {
    Binary64::new(if x.is_finite() { x } else { 0.0 })
        .unwrap_or_else(|_| Binary64::new(0.0).expect("zero"))
}

fn ts(ms: i64) -> Timestamp {
    Timestamp(crate::time::rfc3339(ms))
}

fn role(name: &str) -> s::AgentRole {
    match name {
        "Main" => s::AgentRole::Main,
        "SideCall" => s::AgentRole::SideCall,
        _ => s::AgentRole::SubAgent,
    }
}

fn record(main: &Path, fingerprint: &str, program: &crate::sq::Program) -> Result<Record, String> {
    let lifted = crate::claude::lift(main).map_err(|e| format!("{}: {e}", main.display()))?;
    Ok(record_of(
        lifted,
        Origin {
            path: main.display().to_string(),
            fingerprint: fingerprint.to_owned(),
            project: project_of(main),
            project_dir: project_dir(main),
            harness: s::Harness::ClaudeCode,
        },
        program,
    ))
}

/// Where a stored session came from: its source key (a transcript path, or `wire:<session hash>`),
/// the key's fingerprint, the project it names and its harness.
#[derive(Clone)]
struct Origin {
    path: String,
    fingerprint: String,
    project: String,
    project_dir: String,
    harness: s::Harness,
}

fn record_of(lifted: Lifted, origin: Origin, program: &crate::sq::Program) -> Record {
    let mut p = Projection::new(lifted, origin);
    p.full_record(program)
}

/// Per agent: the running figures of its rows.
#[derive(Debug, Clone, Default)]
struct Track {
    model: Option<String>,
    turns: i64,
    context: i64,
    context_max: i64,
    output: i64,
    cost: i64,
    tools: i64,
    busts: i64,
}

/// One session's projection as a fold over its steps: the replay, the view events, the running
/// figures of the agent and session rows, and the KPI fold. [`record_of`] runs it over a whole
/// session; the appender keeps one per open session and runs it over each call's steps, so the
/// rows it writes are the rows a projection of the whole session writes.
pub struct Projection {
    origin: Origin,
    lifted: Lifted,
    replayed: crate::replay::Replayed,
    replayer: crate::replay::Replayer,
    view: crate::view::Projector,
    facts: Option<crate::kpi::Facts>,
    /// Published events seen before the first instant of the session is known.
    pending: Vec<sessionspec_system::SystemEvent>,
    first: Option<i64>,
    last: Option<i64>,
    tracks: BTreeMap<String, Track>,
    session_cost: i64,
    refused_calls: i64,
    tool_names: Vec<String>,
    models: Vec<String>,
    /// Event lines projected since the last write: `seq \t kind \t agent \t EventAppended`.
    fresh: String,
    events_count: i64,
    /// The check verdicts are older than the steps.
    checks_dirty: bool,
    dirty_since: Option<Instant>,
}

/// A batch moved the session's first instant: the fold cannot continue, project it again.
pub struct Rebuild;

/// What a projection writes after a batch.
struct Delta {
    session_id: String,
    path: String,
    fingerprint: String,
    agents_count: i64,
    lines: String,
    events_count: i64,
    last_ms: i64,
    row: String,
    kpis: String,
    agents: Vec<(String, i64, String)>,
    checks: Option<Vec<crate::sq::SessionCheck>>,
}

impl Projection {
    fn new(mut lifted: Lifted, origin: Origin) -> Projection {
        let steps = std::mem::take(&mut lifted.steps);
        let mut p = Projection {
            origin,
            lifted,
            replayed: crate::replay::Replayed {
                events: Vec::new(),
                log: Vec::new(),
                refused: 0,
            },
            replayer: crate::replay::Replayer::new(),
            view: crate::view::Projector::default(),
            facts: None,
            pending: Vec::new(),
            first: None,
            last: None,
            tracks: BTreeMap::new(),
            session_cost: 0,
            refused_calls: 0,
            tool_names: Vec::new(),
            models: Vec::new(),
            fresh: String::new(),
            events_count: 0,
            checks_dirty: true,
            dirty_since: None,
        };
        // A whole session in one batch sets the first instant from all of it: no rebuild.
        let _ = p.extend(steps);
        p
    }

    /// Folds `steps` in. `Err(Rebuild)` when one of their view events is earlier than the first
    /// instant the fold counted from; the projection is then stale and must be dropped.
    fn extend(&mut self, steps: Vec<Step>) -> Result<(), Rebuild> {
        let mut published = Vec::new();
        for st in &steps {
            let logged = self.replayer.push(st, &mut published);
            self.replayed.log.push(logged);
        }
        self.replayed.refused = self.replayer.refused;
        self.lifted.steps.extend(steps);
        let mut viewed = Vec::new();
        for e in &published {
            self.view.push(e, &mut viewed);
        }
        let batch_first = viewed.iter().map(|e| e.at).filter(|&t| t > 0).min();
        if let (Some(f), Some(b)) = (self.first, batch_first)
            && b < f
        {
            return Err(Rebuild);
        }
        if self.first.is_none() {
            self.first = batch_first;
        }
        if let Some(m) = viewed.iter().map(|e| e.at).max() {
            self.last = Some(self.last.map_or(m, |l| l.max(m)));
        }
        match (&mut self.facts, self.first) {
            (Some(f), _) => {
                for e in &published {
                    f.push(e);
                }
                f.drop_marks();
            }
            (None, Some(from)) => {
                let mut f = crate::kpi::Facts::new(from, i64::MAX);
                for e in self.pending.drain(..).chain(published.iter().cloned()) {
                    f.push(&e);
                }
                f.drop_marks();
                self.facts = Some(f);
            }
            (None, None) => self.pending.extend(published.iter().cloned()),
        }
        let sid = self.lifted.session_id.clone();
        for e in &viewed {
            self.row_of(&sid, e);
        }
        self.events_count += viewed.len() as i64;
        self.replayed.events.extend(published);
        self.checks_dirty = true;
        if self.dirty_since.is_none() {
            self.dirty_since = Some(Instant::now());
        }
        Ok(())
    }

    /// Adds one view event to the agent's and the session's running figures and writes its row.
    fn row_of(&mut self, session_id: &str, e: &v::Event) {
        let t = self.tracks.entry(e.agent.clone()).or_default();
        match e.kind {
            v::EventKind::ModelTurn => {
                t.model = Some(e.label.clone());
                t.turns += 1;
                t.context = e.context_tokens.unwrap_or(0);
                t.context_max = t.context_max.max(t.context);
                t.output += e.output_tokens.unwrap_or(0);
                t.cost += e.cost_microdollars.unwrap_or(0);
                self.session_cost += e.cost_microdollars.unwrap_or(0);
                if !self.models.contains(&e.label) {
                    self.models.push(e.label.clone());
                }
            }
            v::EventKind::ToolCall => {
                t.tools += 1;
                if !self.tool_names.contains(&e.label) {
                    self.tool_names.push(e.label.clone());
                }
            }
            v::EventKind::CacheBust => t.busts += 1,
            v::EventKind::Decision if e.ok == Some(false) => self.refused_calls += 1,
            _ => {}
        }
        let row = v::EventAppended {
            event_id: v::EventId(format!("{session_id}.{}", e.seq)),
            session_id: s::SessionId(session_id.to_owned()),
            seq: e.seq,
            at: ts(e.at),
            agent: s::AgentId(e.agent.clone()),
            kind: e.kind,
            label: e.label.clone(),
            detail: e.detail.clone(),
            call: e.call.clone(),
            ok: e.ok,
            context_tokens: e.context_tokens,
            output_tokens: e.output_tokens,
            cache_read_tokens: e.cache_read_tokens,
            cache_write_tokens: e.cache_write_tokens,
            cost_microdollars: e.cost_microdollars,
            bytes: e.bytes,
            agent_cost_usd: usd(t.cost),
            session_cost_usd: usd(self.session_cost),
        };
        let _ = write!(self.fresh, "{}\t{:?}\t{}\t", e.seq, e.kind, e.agent);
        wire::encode_event_sessionspec_view_event_appended(&row, &mut self.fresh);
        self.fresh.push('\n');
    }

    /// The rows as they stand, and the event lines since the last call. Check verdicts when
    /// `checks` is set.
    fn delta(&mut self, program: &crate::sq::Program, checks: bool) -> Delta {
        let first = self.first.unwrap_or(0);
        let last = self.last.unwrap_or(first);
        let refused = self.replayed.refused;
        let kpis = match &self.facts {
            Some(f) => f.acc(refused, last),
            None => crate::kpi::session_facts(&self.pending, refused, first, last),
        }
        .report(first, last);
        let sid = self.lifted.session_id.clone();
        let mut agents = Vec::new();
        for a in &self.view.agents {
            let t = self.tracks.get(&a.id).cloned().unwrap_or_default();
            let row = v::AgentUpdated {
                agent_id: s::AgentId(a.id.clone()),
                session_id: s::SessionId(sid.clone()),
                parent: a.parent.clone().map(s::AgentId),
                role: role(&a.role),
                model: t.model.clone(),
                started_at: ts(a.started_at),
                turns: t.turns,
                context_tokens: t.context,
                context_max: t.context_max,
                output_tokens: t.output,
                cost_usd: usd(t.cost),
                tool_calls: t.tools,
                busts: t.busts,
            };
            let mut out = String::new();
            wire::encode_event_sessionspec_view_agent_updated(&row, &mut out);
            agents.push((a.id.clone(), a.started_at, out));
        }

        let project = self.origin.project.clone();
        let uncached: i64 = kpis.cache.iter().map(|x| x.uncached_tokens).sum();
        let read: i64 = kpis.cache.iter().map(|x| x.read_tokens).sum();
        let written: i64 = kpis.cache.iter().map(|x| x.write_tokens).sum();
        let input = read + written + uncached;
        let wall = ((last - first) / 1000).max(1);
        let waiting = kpis.autonomy.waiting_on_human_seconds;
        let mut names = format!(
            "{} {}",
            project,
            self.lifted.title.clone().unwrap_or_default()
        );
        for m in &self.models {
            names.push(' ');
            names.push_str(m);
        }
        for t in &self.tool_names {
            names.push(' ');
            names.push_str(t);
        }
        let row = v::SessionUpdated {
            session_id: s::SessionId(sid.clone()),
            project,
            title: self.lifted.title.clone(),
            names: names.to_lowercase(),
            rank: 0,
            harness: self.origin.harness,
            first_at: ts(first),
            last_at: ts(last),
            agents: self.view.agents.len() as i64,
            cost_usd: usd(self.session_cost),
            output_tokens: kpis.models.iter().map(|m| m.output_tokens).sum(),
            context_max: self
                .tracks
                .values()
                .map(|t| t.context_max)
                .max()
                .unwrap_or(0),
            cache_hit_share: share(if input > 0 {
                read as f64 / input as f64
            } else {
                0.0
            }),
            busts: kpis.busts.iter().map(|b| b.count).sum(),
            human_prompts: kpis
                .autonomy
                .prompts
                .iter()
                .filter(|p| p.author == s::PromptAuthor::Human)
                .map(|p| p.count)
                .sum(),
            tool_calls: self.tracks.values().map(|t| t.tools).sum(),
            refused_tool_calls: self.refused_calls,
            exposures: kpis.exposures.iter().map(|x| x.count).sum(),
            working_seconds: kpis.autonomy.working_seconds,
            waiting_on_human_seconds: waiting,
            waiting_share: share((waiting as f64 / wall as f64).min(1.0)),
        };
        let mut row_text = String::new();
        wire::encode_event_sessionspec_view_session_updated(&row, &mut row_text);
        let mut kpis_text = String::new();
        wire::encode_sessionspec_cli_kpi_report(&kpis, &mut kpis_text);
        let checks = checks.then(|| self.checks(program));
        Delta {
            session_id: sid,
            path: self.origin.path.clone(),
            fingerprint: if self.origin.path.starts_with("wire:") {
                self.lifted.steps.len().to_string()
            } else {
                self.origin.fingerprint.clone()
            },
            agents_count: self.lifted.agents as i64,
            lines: std::mem::take(&mut self.fresh),
            events_count: self.events_count,
            last_ms: last,
            row: row_text,
            kpis: kpis_text,
            agents,
            checks,
        }
    }

    /// The check verdicts over the whole session.
    fn checks(&mut self, program: &crate::sq::Program) -> Vec<crate::sq::SessionCheck> {
        let fb = crate::sq::facts::build(&self.lifted, &self.replayed, &self.origin.project_dir);
        let answers = crate::sq::eval::run(program, &fb);
        self.checks_dirty = false;
        self.dirty_since = None;
        crate::sq::session_checks(program, &answers)
    }

    /// The whole session as one record: every row, every event line, the verdicts.
    fn full_record(&mut self, program: &crate::sq::Program) -> Record {
        let d = self.delta(program, true);
        let wire_source = self.origin.path.starts_with("wire:");
        Record {
            path: d.path,
            fingerprint: d.fingerprint,
            session_id: d.session_id,
            title: self.lifted.title.clone(),
            agents_count: d.agents_count,
            files: self
                .lifted
                .files
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            // A wire session's steps are its `wire_steps` rows; `load` reads them there.
            steps: if wire_source {
                Vec::new()
            } else {
                pack(&encode_steps(&self.lifted.steps))
            },
            events: pack(&d.lines),
            events_count: d.events_count,
            last_ms: d.last_ms,
            row: d.row,
            kpis: d.kpis,
            agents: d.agents,
            checks: d.checks.unwrap_or_default(),
        }
    }
}

/// The projections the appender keeps for open sessions, the least recently appended dropped
/// first.
pub struct Projections {
    capacity: usize,
    tick: u64,
    open: HashMap<String, (u64, Projection)>,
}

impl Projections {
    pub fn new(capacity: usize) -> Projections {
        Projections {
            capacity: capacity.max(1),
            tick: 0,
            open: HashMap::new(),
        }
    }

    /// How many sessions are open.
    pub fn len(&self) -> usize {
        self.open.len()
    }

    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
    }
}

/// How long a session's verdicts may lag its steps while calls keep arriving.
const CHECKS_LAG: std::time::Duration = std::time::Duration::from_secs(30);

fn project_dir(main: &Path) -> String {
    main.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_owned()
}

fn write(conn: &mut Connection, r: &Record) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    let old: Option<String> = tx
        .query_row(
            "SELECT session_id FROM sources WHERE path = ?1",
            params![r.path],
            |row| row.get(0),
        )
        .optional()?;
    for id in old.iter().chain(std::iter::once(&r.session_id)) {
        tx.execute("DELETE FROM sessions WHERE session_id = ?1", params![id])?;
        tx.execute("DELETE FROM agents WHERE session_id = ?1", params![id])?;
        tx.execute("DELETE FROM checks WHERE session_id = ?1", params![id])?;
        tx.execute(
            "DELETE FROM event_chunks WHERE session_id = ?1",
            params![id],
        )?;
    }
    tx.execute(
        "INSERT OR REPLACE INTO sources (path, session_id, fingerprint) VALUES (?1, ?2, ?3)",
        params![r.path, r.session_id, r.fingerprint],
    )?;
    tx.execute(
        "INSERT INTO sessions (session_id, path, title, agents, files, steps, events_count,
           last_ms, row, kpis)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            r.session_id,
            r.path,
            r.title,
            r.agents_count,
            r.files,
            r.steps,
            r.events_count,
            r.last_ms,
            r.row,
            r.kpis
        ],
    )?;
    tx.execute(
        "INSERT INTO event_chunks (session_id, chunk, events) VALUES (?1, 0, ?2)",
        params![r.session_id, r.events],
    )?;
    put_agents(&tx, &r.session_id, &r.agents)?;
    put_checks(&tx, &r.session_id, &r.checks)?;
    tx.commit()
}

/// Writes what one append changed: the session row, a chunk of new event lines, every agent row,
/// and the verdicts when they were judged again.
fn write_delta(conn: &mut Connection, d: &Delta) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE sessions SET agents = ?2, events_count = ?3, last_ms = ?4, row = ?5, kpis = ?6
         WHERE session_id = ?1",
        params![
            d.session_id,
            d.agents_count,
            d.events_count,
            d.last_ms,
            d.row,
            d.kpis
        ],
    )?;
    tx.execute(
        "UPDATE sources SET fingerprint = ?2 WHERE path = ?1",
        params![d.path, d.fingerprint],
    )?;
    if !d.lines.is_empty() {
        tx.execute(
            "INSERT INTO event_chunks (session_id, chunk, events)
             VALUES (?1, (SELECT coalesce(max(chunk) + 1, 0) FROM event_chunks WHERE session_id = ?1), ?2)",
            params![d.session_id, pack(&d.lines)],
        )?;
    }
    put_agents(&tx, &d.session_id, &d.agents)?;
    if let Some(checks) = &d.checks {
        tx.execute(
            "DELETE FROM checks WHERE session_id = ?1",
            params![d.session_id],
        )?;
        put_checks(&tx, &d.session_id, checks)?;
    }
    tx.commit()
}

fn write_checks(
    conn: &mut Connection,
    session_id: &str,
    checks: &[crate::sq::SessionCheck],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM checks WHERE session_id = ?1",
        params![session_id],
    )?;
    put_checks(&tx, session_id, checks)?;
    tx.commit()
}

fn put_agents(
    tx: &rusqlite::Transaction<'_>,
    session_id: &str,
    agents: &[(String, i64, String)],
) -> rusqlite::Result<()> {
    let mut st = tx.prepare(
        "INSERT OR REPLACE INTO agents (session_id, agent_id, started_ms, row)
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    for (id, started, row) in agents {
        st.execute(params![session_id, id, started, row])?;
    }
    Ok(())
}

fn put_checks(
    tx: &rusqlite::Transaction<'_>,
    session_id: &str,
    checks: &[crate::sq::SessionCheck],
) -> rusqlite::Result<()> {
    let mut st = tx.prepare(
        "INSERT OR REPLACE INTO checks (session_id, name, about, severity, verdict, rows, possible)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for ch in checks {
        st.execute(params![
            session_id,
            ch.check,
            ch.about,
            ch.severity,
            format!("{:?}", ch.verdict),
            ch.rows,
            ch.possible
        ])?;
    }
    Ok(())
}

fn actor_name(a: Actor) -> &'static str {
    match a {
        Actor::HarnessRuntime => "HarnessRuntime",
        Actor::Human => "Human",
        Actor::Delegate => "Delegate",
        Actor::PeerSession => "PeerSession",
        Actor::AgentLoop => "AgentLoop",
        Actor::PermissionAuthority => "PermissionAuthority",
        Actor::ToolProvider => "ToolProvider",
        Actor::Redactor => "Redactor",
    }
}

fn source_name(s: s::Source) -> &'static str {
    match s {
        s::Source::Disk => "Disk",
        s::Source::Wire => "Wire",
    }
}

fn source_of(name: &str) -> Option<s::Source> {
    match name {
        "Disk" => Some(s::Source::Disk),
        "Wire" => Some(s::Source::Wire),
        _ => None,
    }
}

fn actor_of(name: &str) -> Option<Actor> {
    Some(match name {
        "HarnessRuntime" => Actor::HarnessRuntime,
        "Human" => Actor::Human,
        "Delegate" => Actor::Delegate,
        "PeerSession" => Actor::PeerSession,
        "AgentLoop" => Actor::AgentLoop,
        "PermissionAuthority" => Actor::PermissionAuthority,
        "ToolProvider" => Actor::ToolProvider,
        "Redactor" => Actor::Redactor,
        _ => return None,
    })
}

/// The command's wire JSON, as the generated encoder writes it.
pub fn command_text(cmd: &Cmd) -> String {
    let mut out = String::new();
    match cmd {
        Cmd::OpenSession(i) => wire::encode_command_sessionspec_session_open_session(i, &mut out),
        Cmd::StartAgent(i) => wire::encode_command_sessionspec_session_start_agent(i, &mut out),
        Cmd::SubmitPrompt(i) => wire::encode_command_sessionspec_session_submit_prompt(i, &mut out),
        Cmd::EndTurn(i) => wire::encode_command_sessionspec_session_end_turn(i, &mut out),
        Cmd::RecordModelTurn(i) => {
            wire::encode_command_sessionspec_session_record_model_turn(i, &mut out)
        }
        Cmd::CompactContext(i) => {
            wire::encode_command_sessionspec_session_compact_context(i, &mut out)
        }
        Cmd::RequestToolCall(i) => {
            wire::encode_command_sessionspec_session_request_tool_call(i, &mut out)
        }
        Cmd::DecideToolCall(i) => {
            wire::encode_command_sessionspec_session_decide_tool_call(i, &mut out)
        }
        Cmd::CompleteToolCall(i) => {
            wire::encode_command_sessionspec_session_complete_tool_call(i, &mut out)
        }
        Cmd::RecordExposure(i) => {
            wire::encode_command_sessionspec_session_record_exposure(i, &mut out)
        }
    }
    out
}

fn command_of(name: &str, value: &json::Value) -> Result<Cmd, String> {
    let e = |e: json::DecodeError| format!("{name}: {e:?}");
    Ok(match name {
        "OpenSession" => Cmd::OpenSession(
            wire::decode_command_sessionspec_session_open_session(value, "$").map_err(e)?,
        ),
        "StartAgent" => Cmd::StartAgent(
            wire::decode_command_sessionspec_session_start_agent(value, "$").map_err(e)?,
        ),
        "SubmitPrompt" => Cmd::SubmitPrompt(
            wire::decode_command_sessionspec_session_submit_prompt(value, "$").map_err(e)?,
        ),
        "EndTurn" => {
            Cmd::EndTurn(wire::decode_command_sessionspec_session_end_turn(value, "$").map_err(e)?)
        }
        "RecordModelTurn" => Cmd::RecordModelTurn(
            wire::decode_command_sessionspec_session_record_model_turn(value, "$").map_err(e)?,
        ),
        "CompactContext" => Cmd::CompactContext(
            wire::decode_command_sessionspec_session_compact_context(value, "$").map_err(e)?,
        ),
        "RequestToolCall" => Cmd::RequestToolCall(
            wire::decode_command_sessionspec_session_request_tool_call(value, "$").map_err(e)?,
        ),
        "DecideToolCall" => Cmd::DecideToolCall(
            wire::decode_command_sessionspec_session_decide_tool_call(value, "$").map_err(e)?,
        ),
        "CompleteToolCall" => Cmd::CompleteToolCall(
            wire::decode_command_sessionspec_session_complete_tool_call(value, "$").map_err(e)?,
        ),
        "RecordExposure" => Cmd::RecordExposure(
            wire::decode_command_sessionspec_session_record_exposure(value, "$").map_err(e)?,
        ),
        other => return Err(format!("unknown command {other}")),
    })
}

/// One line per step: `at_ms \t order_ms \t actor \t command \t wire JSON`.
pub fn encode_steps(steps: &[Step]) -> String {
    let mut out = String::new();
    for st in steps {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}",
            st.at_ms,
            st.order_ms,
            actor_name(st.actor),
            source_name(st.source),
            crate::replay::command_name(&st.cmd),
            command_text(&st.cmd)
        );
    }
    out
}

pub fn decode_steps(text: &str) -> Result<Vec<Step>, String> {
    text.lines()
        .map(|line| {
            let mut parts = line.splitn(6, '\t');
            let mut next = || {
                parts
                    .next()
                    .ok_or_else(|| format!("short step line: {line:.60}"))
            };
            let at_ms: i64 = next()?.parse().map_err(|e| format!("{e}"))?;
            let order_ms: i64 = next()?.parse().map_err(|e| format!("{e}"))?;
            let actor = actor_of(next()?).ok_or("unknown actor")?;
            let source = source_of(next()?).ok_or("unknown source")?;
            let name = next()?;
            let value = json::parse(next()?).map_err(|e| format!("{e:?}"))?;
            Ok(Step {
                at_ms,
                order_ms,
                actor,
                source,
                cmd: command_of(name, &value)?,
            })
        })
        .collect()
}
