//! The local store (`<state-dir>/sessions.db`, SQLite): per main transcript the redacted lifted
//! commands, and the rows of `agenticspec.view` that `sync` projects from what the generated
//! component publishes. Readers replay the stored commands instead of parsing transcripts again;
//! `serve` answers the views from the rows. The store lives outside the repository: it holds the
//! harness's session titles, which a local UI may show and which never leave the machine.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::{Instant, UNIX_EPOCH};

use agenticspec_server::{json, wire};
use agenticspec_types::cli as c;
use agenticspec_types::primitives::{Binary64, Timestamp};
use agenticspec_types::session as s;
use agenticspec_types::view as v;
use rayon::prelude::*;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

use crate::claude::{Actor, Cmd, Lifted, Step};

/// Raised when what the derived tables hold changes meaning. A store of another version drops its
/// derived tables (sources, sessions, agents, events, the hourly sums, the search index, checks)
/// and fills them again: the next sync refills transcript sessions, and `open` projects every wire
/// session again from its `wire_steps`, which no sync can refill and which are never dropped.
pub const SCHEMA_VERSION: i64 = 5;

/// Raised when the encoding of a `wire_steps` row changes. A store whose wire rows this build
/// cannot read is refused by [`Store::open`], and nothing in it is dropped.
pub const WIRE_FORMAT: i64 = 1;

/// The tables a sync or a wire re-projection fills again; `open` drops them on a schema change.
/// `event_chunks` is the events table of schema 4.
const DERIVED: &[&str] = &[
    "sources",
    "sessions",
    "agents",
    "events",
    "hourly",
    "session_names",
    "checks",
    "event_chunks",
];

/// The most rows one page of a session's events holds.
pub const EVENT_PAGE: i64 = 500;

/// The zstd level of the `steps` blob.
const LEVEL: i32 = 3;

fn pack(text: &str) -> Vec<u8> {
    zstd::encode_all(text.as_bytes(), LEVEL).unwrap_or_default()
}

fn unpack(blob: &[u8]) -> io::Result<String> {
    let bytes = zstd::decode_all(blob)?;
    String::from_utf8(bytes).map_err(io::Error::other)
}

/// The checks every synced session is judged by (`agenticspec.view.SessionChecks`).
pub const CHECKS: &str = include_str!("../../../queries/session.sq");

// Every view the UI reads is a table here, kept by `sync` and the appender, so `serve` answers a
// request with one indexed query and holds no row beyond its answer. `change` numbers each write
// (`meta.change`); a live channel reads the rows written after the last number it sent.
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS sources (
  path TEXT PRIMARY KEY, session_id TEXT NOT NULL, fingerprint TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS sessions (
  sk INTEGER PRIMARY KEY, session_id TEXT NOT NULL UNIQUE, path TEXT NOT NULL, title TEXT,
  agents INTEGER NOT NULL, files TEXT NOT NULL, steps BLOB NOT NULL,
  events_count INTEGER NOT NULL, last_ms INTEGER NOT NULL, names TEXT NOT NULL,
  row TEXT NOT NULL, kpis TEXT NOT NULL, change INTEGER NOT NULL,
  cost_micro INTEGER NOT NULL, working_s INTEGER NOT NULL, waiting_share REAL NOT NULL,
  cache_share REAL NOT NULL, tool_calls INTEGER NOT NULL, refused INTEGER NOT NULL);
CREATE INDEX IF NOT EXISTS sessions_last ON sessions (last_ms DESC, session_id);
CREATE INDEX IF NOT EXISTS sessions_change ON sessions (change);
CREATE VIRTUAL TABLE IF NOT EXISTS session_names USING fts5 (names, tokenize = 'trigram');
CREATE TABLE IF NOT EXISTS events (
  sk INTEGER NOT NULL, seq INTEGER NOT NULL, ord INTEGER NOT NULL, at_ms INTEGER NOT NULL,
  agent TEXT NOT NULL, kind TEXT NOT NULL, notable INTEGER NOT NULL, label TEXT NOT NULL,
  detail TEXT, call TEXT, ok INTEGER, context_tokens INTEGER, output_tokens INTEGER,
  cache_read_tokens INTEGER, cache_write_tokens INTEGER, cost_micro INTEGER, bytes INTEGER,
  agent_cost_micro INTEGER NOT NULL, session_cost_micro INTEGER NOT NULL,
  PRIMARY KEY (sk, seq)) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS events_ord ON events (sk, ord);
CREATE INDEX IF NOT EXISTS events_agent ON events (sk, agent, ord);
CREATE INDEX IF NOT EXISTS events_kind ON events (sk, kind, ord);
CREATE INDEX IF NOT EXISTS events_notable ON events (sk, ord) WHERE notable;
CREATE TABLE IF NOT EXISTS hourly (
  hour INTEGER PRIMARY KEY, sessions INTEGER NOT NULL, cost_micro INTEGER NOT NULL,
  working_s INTEGER NOT NULL, waiting_share REAL NOT NULL, cache_share REAL NOT NULL,
  tool_calls INTEGER NOT NULL, refused INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS agents (
  session_id TEXT NOT NULL, agent_id TEXT NOT NULL, started_ms INTEGER NOT NULL,
  row TEXT NOT NULL, change INTEGER NOT NULL, PRIMARY KEY (session_id, agent_id));
CREATE INDEX IF NOT EXISTS agents_change ON agents (change);
CREATE TABLE IF NOT EXISTS checks (
  session_id TEXT NOT NULL, name TEXT NOT NULL, about TEXT NOT NULL, severity TEXT NOT NULL,
  verdict TEXT NOT NULL, rows INTEGER NOT NULL, possible INTEGER NOT NULL,
  PRIMARY KEY (session_id, name));
CREATE TABLE IF NOT EXISTS wire_steps (
  session_id TEXT NOT NULL, seq INTEGER NOT NULL, harness TEXT NOT NULL, step TEXT NOT NULL,
  PRIMARY KEY (session_id, seq));
";

/// `$XDG_DATA_HOME/agenticspec`, else `~/.local/share/agenticspec`.
pub fn default_dir() -> PathBuf {
    data_home().join("agenticspec")
}

/// The directories of the tool's earlier names beside [`default_dir`], the newest first:
/// `agentspec`, then `sessionspec`.
pub fn legacy_dirs() -> [PathBuf; 2] {
    let home = data_home();
    [home.join("agentspec"), home.join("sessionspec")]
}

fn data_home() -> PathBuf {
    match std::env::var("XDG_DATA_HOME") {
        Ok(d) if !d.is_empty() => PathBuf::from(d),
        _ => PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"),
    }
}

/// Moves `legacy` to `current` when only `legacy` exists. Returns whether it moved.
pub fn adopt_legacy_dir(legacy: &Path, current: &Path) -> std::io::Result<bool> {
    if current.exists() || !legacy.is_dir() {
        return Ok(false);
    }
    if let Some(parent) = current.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(legacy, current)?;
    Ok(true)
}

/// The store in `state_dir`, or in [`default_dir`]. Without `state_dir`, the newest directory
/// of an earlier name is moved to [`default_dir`] first, when that does not exist yet.
pub fn path_in(state_dir: Option<&Path>) -> PathBuf {
    let dir = match state_dir {
        Some(d) => d.to_path_buf(),
        None => {
            let dir = default_dir();
            for legacy in legacy_dirs() {
                if adopt_legacy_dir(&legacy, &dir).unwrap_or(false) || dir.exists() {
                    break;
                }
            }
            dir
        }
    };
    dir.join("sessions.db")
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
    /// Opens the store at `path`, creating it and its directory.
    ///
    /// A store of another schema version drops its derived tables and fills them again; its
    /// `wire_steps` are kept and every wire session is projected again from them. A store whose
    /// `wire_steps` are in a wire format this build cannot read is refused with an error naming
    /// both versions, and nothing in it changes.
    pub fn open(path: &Path) -> Result<Store, String> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let store = Store {
            path: path.to_path_buf(),
        };
        let mut conn = store.conn().map_err(sql)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .map_err(sql)?;
        let meta = |key: &str| -> Result<Option<String>, String> {
            conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
                r.get(0)
            })
            .optional()
            .map_err(sql)
        };
        let version = meta("schema")?;
        // A store written before the wire format was recorded holds format 1.
        let wire_format = meta("wire_format")?;
        let wire_rows: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'wire_steps'",
                [],
                |r| r.get(0),
            )
            .map_err(sql)?;
        let wire_rows = if wire_rows > 0 {
            conn.query_row("SELECT count(*) FROM wire_steps", [], |r| {
                r.get::<_, i64>(0)
            })
            .map_err(sql)?
        } else {
            0
        };
        let stored_format = wire_format.as_deref().unwrap_or("1");
        if wire_rows > 0 && stored_format != WIRE_FORMAT.to_string() {
            return Err(format!(
                "{}: its {wire_rows} wire rows are in wire format {stored_format} (schema {}); \
                 this build reads wire format {WIRE_FORMAT} (schema {SCHEMA_VERSION}); \
                 nothing was changed",
                path.display(),
                version.as_deref().unwrap_or("none"),
            ));
        }
        if version.as_deref() == Some(&SCHEMA_VERSION.to_string()) {
            if wire_format.is_none() {
                set_meta(&conn, "wire_format", &WIRE_FORMAT.to_string())?;
            }
            return Ok(store);
        }
        let drop: String = DERIVED
            .iter()
            .map(|t| format!("DROP TABLE IF EXISTS {t};"))
            .collect();
        conn.execute_batch(&drop).map_err(sql)?;
        conn.execute_batch(SCHEMA).map_err(sql)?;
        conn.execute_batch("VACUUM;").map_err(sql)?;
        set_meta(&conn, "schema", &SCHEMA_VERSION.to_string())?;
        set_meta(&conn, "wire_format", &WIRE_FORMAT.to_string())?;
        if wire_rows > 0 {
            store.reproject_wire(&mut conn)?;
        }
        Ok(store)
    }

    /// Projects every wire session again from its `wire_steps` into the derived tables.
    fn reproject_wire(&self, conn: &mut Connection) -> Result<(), String> {
        let sessions: Vec<(String, String)> = {
            let mut st = conn
                .prepare(
                    "SELECT session_id, min(harness) FROM wire_steps GROUP BY session_id
                     ORDER BY session_id",
                )
                .map_err(sql)?;
            st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(sql)?
                .collect::<rusqlite::Result<_>>()
                .map_err(sql)?
        };
        let program = crate::sq::compile(CHECKS)?;
        for (hash, harness) in sessions {
            let harness = match harness.as_str() {
                "ClaudeCode" => s::Harness::ClaudeCode,
                "Codex" => s::Harness::Codex,
                _ => s::Harness::Other,
            };
            self.project_wire(conn, &hash, harness, &program)?;
        }
        Ok(())
    }

    /// Projects one wire session from every stored step and writes its rows.
    fn project_wire(
        &self,
        conn: &mut Connection,
        session_hash: &str,
        harness: s::Harness,
        program: &crate::sq::Program,
    ) -> Result<Projection, String> {
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
        write(conn, &rec).map_err(sql)?;
        Ok(p)
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
        let mut conn = conn;
        let (mut lifted, mut failed) = (0i64, 0i64);
        let mut changed = Vec::new();
        // Lifts run in chunks, each written and dropped before the next starts. A lift holds
        // several times its transcript in memory, so a chunk holds at most `LIFT_BUDGET` bytes of
        // transcript (a larger one alone) and one transcript per thread.
        let mut sized: Vec<(u64, &(PathBuf, String))> =
            todo.iter().map(|t| (transcript_bytes(&t.0), t)).collect();
        sized.sort_by_key(|a| std::cmp::Reverse(a.0));
        let threads = rayon::current_num_threads().max(1);
        let mut chunks: Vec<Vec<&(PathBuf, String)>> = Vec::new();
        let mut held = 0u64;
        for (bytes, t) in sized {
            match chunks.last_mut() {
                Some(c) if c.len() < threads && held + bytes <= LIFT_BUDGET => {
                    c.push(t);
                    held += bytes;
                }
                _ => {
                    chunks.push(vec![t]);
                    held = bytes;
                }
            }
        }
        for part in chunks {
            let records: Vec<Result<Record, String>> = part
                .par_iter()
                .map(|(p, f)| record(p, f, &program))
                .collect();
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
        }
        if lifted > 0 {
            release_freed_memory();
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
        self.session_page(&Page::default())
    }

    /// One page of the session list (`agenticspec.view.SessionList`), in one query: the rank is
    /// the place among every session by `last_at`; `search` matches the trigram index of `names`
    /// (a substring, any case), or `names` itself for fewer than three characters.
    pub fn session_page(&self, page: &Page) -> rusqlite::Result<Vec<v::SessionUpdated>> {
        let conn = self.read()?;
        let search = page
            .search
            .as_deref()
            .map(str::to_lowercase)
            .filter(|q| !q.is_empty());
        let fts = search.as_deref().is_some_and(|q| q.chars().count() >= 3);
        let matcher = match (&search, fts) {
            (None, _) => "1",
            (Some(_), true) => "sk IN (SELECT rowid FROM session_names WHERE names MATCH ?2)",
            (Some(_), false) => "instr(names, ?2) > 0",
        };
        let text = format!(
            "SELECT row, rank FROM (
               SELECT sk, row, last_ms, names,
                      row_number() OVER (ORDER BY last_ms DESC, session_id) AS rank
               FROM sessions)
             WHERE last_ms >= ?1 AND {matcher} AND rank > ?3 AND rank <= ?4
             ORDER BY rank"
        );
        let mut st = conn.prepare(&text)?;
        // A trigram query is a phrase, so a quote in the search is doubled.
        let q = match (&search, fts) {
            (Some(q), true) => format!("\"{}\"", q.replace('"', "\"\"")),
            (Some(q), false) => q.clone(),
            (None, _) => String::new(),
        };
        let rows = st.query_map(
            params![
                page.since_ms.unwrap_or(i64::MIN),
                q,
                page.offset.unwrap_or(0),
                page.limit.unwrap_or(i64::MAX)
            ],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
        )?;
        let mut out = Vec::new();
        for row in rows {
            let (text, rank) = row?;
            if let Some(mut s) = decode(&text, wire::decode_event_agenticspec_view_session_updated)
            {
                s.rank = rank;
                out.push(s);
            }
        }
        Ok(out)
    }

    /// One session row by its id, with its rank.
    pub fn session(&self, session_id: &str) -> rusqlite::Result<Option<v::SessionUpdated>> {
        let conn = self.read()?;
        let row: Option<(String, i64)> = conn
            .query_row(
                "SELECT row, (SELECT count(*) FROM sessions o
                              WHERE o.last_ms > s.last_ms
                                 OR (o.last_ms = s.last_ms AND o.session_id <= s.session_id))
                 FROM sessions s WHERE session_id = ?1",
                params![session_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        Ok(row.and_then(|(text, rank)| {
            let mut s = decode(&text, wire::decode_event_agenticspec_view_session_updated)?;
            s.rank = rank;
            Some(s)
        }))
    }

    /// The session rows written after change `after`, with their rank, and the last change.
    pub fn sessions_changed(
        &self,
        after: i64,
    ) -> rusqlite::Result<(Vec<v::SessionUpdated>, Vec<v::AgentUpdated>, i64)> {
        let conn = self.read()?;
        let last = last_change(&conn)?;
        let mut st = conn.prepare(
            "SELECT row, rank FROM (
               SELECT row, change, row_number() OVER (ORDER BY last_ms DESC, session_id) AS rank
               FROM sessions)
             WHERE change > ?1 AND change <= ?2 ORDER BY change",
        )?;
        let mut sessions = Vec::new();
        for row in st.query_map(params![after, last], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })? {
            let (text, rank) = row?;
            if let Some(mut s) = decode(&text, wire::decode_event_agenticspec_view_session_updated)
            {
                s.rank = rank;
                sessions.push(s);
            }
        }
        let mut st = conn.prepare(
            "SELECT row FROM agents WHERE change > ?1 AND change <= ?2 ORDER BY change, agent_id",
        )?;
        let mut agents = Vec::new();
        for row in st.query_map(params![after, last], |r| r.get::<_, String>(0))? {
            if let Some(a) = decode(&row?, wire::decode_event_agenticspec_view_agent_updated) {
                agents.push(a);
            }
        }
        Ok((sessions, agents, last))
    }

    /// The number of the last write.
    pub fn last_change(&self) -> rusqlite::Result<i64> {
        last_change(&self.read()?)
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
        Ok(t.and_then(|t| decode(&t, wire::decode_agenticspec_cli_kpi_report)))
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
            .filter_map(|t| decode(t, wire::decode_event_agenticspec_view_agent_updated))
            .collect())
    }

    /// The events of a session from `from_seq` on, of one kind and agent when given, by `seq`.
    pub fn events(
        &self,
        session_id: &str,
        from_seq: i64,
        kind: Option<v::EventKind>,
        agent: Option<&str>,
    ) -> rusqlite::Result<Vec<v::EventAppended>> {
        let conn = self.read()?;
        let mut st = conn.prepare(&format!(
            "SELECT {EVENT_COLUMNS} FROM events
             WHERE sk = (SELECT sk FROM sessions WHERE session_id = ?1) AND seq >= ?2
               AND (?3 IS NULL OR kind = ?3) AND (?4 IS NULL OR agent = ?4)
             ORDER BY seq"
        ))?;
        st.query_map(
            params![session_id, from_seq, kind.map(kind_name), agent],
            |r| event_of(session_id, r),
        )?
        .collect()
    }

    /// One page of a session's events (`agenticspec.view.SessionEvents`), newest `ord` first, at
    /// most [`EVENT_PAGE`] rows, in one indexed query.
    pub fn event_page(
        &self,
        session_id: &str,
        filter: &EventFilter,
    ) -> rusqlite::Result<Vec<v::EventAppended>> {
        let conn = self.read()?;
        let mut st = conn.prepare(&format!(
            "SELECT {EVENT_COLUMNS} FROM events
             WHERE sk = (SELECT sk FROM sessions WHERE session_id = ?1)
               AND (?2 IS NULL OR agent = ?2) AND (?3 IS NULL OR kind = ?3)
               AND (?4 IS NULL OR notable = ?4) AND ord < ?5
             ORDER BY ord DESC LIMIT ?6"
        ))?;
        st.query_map(
            params![
                session_id,
                filter.agent,
                filter.kind.map(kind_name),
                filter.notable,
                filter.before.unwrap_or(i64::MAX),
                EVENT_PAGE
            ],
            |r| event_of(session_id, r),
        )?
        .collect()
    }

    /// How many events of each kind a session holds, of one agent when given.
    pub fn event_kind_counts(
        &self,
        session_id: &str,
        agent: Option<&str>,
    ) -> rusqlite::Result<Vec<(v::EventKind, i64)>> {
        let conn = self.read()?;
        let mut st = conn.prepare(
            "SELECT kind, count(*) FROM events
             WHERE sk = (SELECT sk FROM sessions WHERE session_id = ?1)
               AND (?2 IS NULL OR agent = ?2)
             GROUP BY kind ORDER BY kind",
        )?;
        let rows = st.query_map(params![session_id, agent], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (k, n) = row?;
            if let Some(k) = kind_of(&k) {
                out.push((k, n));
            }
        }
        Ok(out)
    }

    /// The timeline slots of a session (`agenticspec.view.SessionTimeline`): its events grouped
    /// by agent and by `slot_seconds` from the epoch, oldest first, in one grouped query.
    pub fn timeline(
        &self,
        session_id: &str,
        slot_seconds: i64,
    ) -> rusqlite::Result<Vec<v::SessionTimeline>> {
        let conn = self.read()?;
        let width = slot_seconds.max(1) * 1000;
        let mut st = conn.prepare(
            "SELECT agent, at_ms / ?2 AS slot, min(at_ms), max(at_ms), count(*),
               sum(kind = 'tool_call' AND detail = 'Read'),
               sum(kind = 'tool_call' AND detail = 'Search'),
               sum(kind = 'tool_call' AND detail = 'Edit'),
               sum(kind = 'tool_call' AND detail = 'Execute'),
               sum(kind = 'tool_call' AND detail = 'Network'),
               sum(kind = 'tool_call' AND detail = 'Spawn'),
               sum(kind = 'tool_call' AND detail = 'Message'),
               sum(kind = 'tool_call' AND detail = 'Plan'),
               sum(kind = 'tool_call' AND coalesce(detail, '') NOT IN
                   ('Read', 'Search', 'Edit', 'Execute', 'Network', 'Spawn', 'Message', 'Plan')),
               sum(kind = 'prompt' AND label = 'Human'),
               sum(kind = 'prompt' AND label IN ('Delegate', 'PeerSession')),
               sum(kind = 'decision' AND ok = 0),
               sum(kind = 'cache_bust'),
               sum(kind = 'compaction'),
               max(CASE WHEN kind = 'model_turn' THEN context_tokens END),
               max(agent_cost_micro), max(session_cost_micro)
             FROM events
             WHERE sk = (SELECT sk FROM sessions WHERE session_id = ?1) AND kind != 'agent_started'
             GROUP BY agent, slot ORDER BY min(at_ms), agent",
        )?;
        st.query_map(params![session_id, width], |r| {
            let agent: String = r.get(0)?;
            let slot: i64 = r.get(1)?;
            Ok(v::SessionTimeline {
                slot_id: v::SlotId(format!("{session_id}.{agent}.{}", slot * width)),
                session_id: s::SessionId(session_id.to_owned()),
                agent: s::AgentId(agent),
                slot_seconds: slot_seconds.max(1),
                start: ts(r.get(2)?),
                end: ts(r.get(3)?),
                events: r.get(4)?,
                read_calls: r.get(5)?,
                search_calls: r.get(6)?,
                edit_calls: r.get(7)?,
                execute_calls: r.get(8)?,
                network_calls: r.get(9)?,
                spawn_calls: r.get(10)?,
                message_calls: r.get(11)?,
                plan_calls: r.get(12)?,
                other_calls: r.get(13)?,
                human_prompts: r.get(14)?,
                relayed_prompts: r.get(15)?,
                refusals: r.get(16)?,
                busts: r.get(17)?,
                compactions: r.get(18)?,
                context_max: r.get(19)?,
                agent_cost_usd: usd(r.get(20)?),
                session_cost_usd: usd(r.get(21)?),
            })
        })?
        .collect()
    }

    /// The overview's figures (`agenticspec.view.OverviewFigures`): the hourly sums from
    /// `since_ms` on, added per bucket of `bucket_seconds`, in one grouped query.
    pub fn figures(
        &self,
        since_ms: i64,
        bucket_seconds: i64,
    ) -> rusqlite::Result<Vec<v::OverviewFigures>> {
        let conn = self.read()?;
        let width = bucket_seconds.max(1) * 1000;
        let mut st = conn.prepare(
            "SELECT (hour * 3600000 - ?1) / ?2 AS b, sum(sessions), sum(cost_micro),
               sum(working_s), sum(waiting_share), sum(cache_share), sum(tool_calls), sum(refused)
             FROM hourly WHERE hour * 3600000 >= ?1 GROUP BY b ORDER BY b",
        )?;
        st.query_map(params![since_ms, width], |r| {
            let start = since_ms + r.get::<_, i64>(0)? * width;
            Ok(v::OverviewFigures {
                bucket_id: v::BucketId(start.to_string()),
                start: ts(start),
                bucket_seconds: bucket_seconds.max(1),
                sessions: r.get(1)?,
                cost_usd: usd(r.get(2)?),
                working_seconds: r.get(3)?,
                waiting_share: share(r.get(4)?),
                cache_hit_share: share(r.get(5)?),
                tool_calls: r.get(6)?,
                refused_tool_calls: r.get(7)?,
            })
        })?
        .collect()
    }

    /// The last `seq` of each session written after change `after`.
    pub fn last_seqs(&self, after: i64) -> rusqlite::Result<HashMap<String, i64>> {
        let conn = self.read()?;
        let mut st = conn.prepare(
            "SELECT s.session_id, (SELECT max(seq) FROM events e WHERE e.sk = s.sk)
             FROM sessions s WHERE s.change > ?1",
        )?;
        let rows = st.query_map(params![after], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?))
        })?;
        let mut out = HashMap::new();
        for row in rows {
            let (id, seq) = row?;
            out.insert(id, seq.unwrap_or(-1));
        }
        Ok(out)
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
        let p = self.project_wire(&mut conn, session_hash, harness, program)?;
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
        for e in self.events(session_id, 0, None, None).map_err(sql)? {
            let _ = write!(out, "{}\t{:?}\t{}\t", e.seq, e.kind, e.agent.0);
            wire::encode_event_agenticspec_view_event_appended(&e, &mut out);
            out.push('\n');
        }
        for ch in self.checks(session_id).map_err(sql)? {
            let _ = writeln!(
                out,
                "{} {} {:?} {} {}",
                ch.check, ch.severity, ch.verdict, ch.rows, ch.possible
            );
        }
        Ok(out)
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

/// What one page of the session list asks for (`agenticspec.view.SessionList`).
#[derive(Debug, Clone, Default)]
pub struct Page {
    pub since_ms: Option<i64>,
    pub search: Option<String>,
    /// The last rank of the page.
    pub limit: Option<i64>,
    /// The rank the page starts after.
    pub offset: Option<i64>,
}

/// What one page of a session's events asks for (`agenticspec.view.SessionEvents`).
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub agent: Option<String>,
    pub kind: Option<v::EventKind>,
    pub notable: Option<bool>,
    /// The page holds events with an `ord` below this.
    pub before: Option<i64>,
}

const EVENT_COLUMNS: &str = "seq, ord, at_ms, agent, kind, notable, label, detail, call, ok,
  context_tokens, output_tokens, cache_read_tokens, cache_write_tokens, cost_micro, bytes,
  agent_cost_micro, session_cost_micro";

fn event_of(session_id: &str, r: &rusqlite::Row<'_>) -> rusqlite::Result<v::EventAppended> {
    let seq: i64 = r.get(0)?;
    let kind: String = r.get(4)?;
    Ok(v::EventAppended {
        event_id: v::EventId(format!("{session_id}.{seq}")),
        session_id: s::SessionId(session_id.to_owned()),
        seq,
        ord: r.get(1)?,
        at: ts(r.get(2)?),
        agent: s::AgentId(r.get(3)?),
        kind: kind_of(&kind).unwrap_or(v::EventKind::TurnEnd),
        notable: r.get(5)?,
        label: r.get(6)?,
        detail: r.get(7)?,
        call: r.get(8)?,
        ok: r.get(9)?,
        context_tokens: r.get(10)?,
        output_tokens: r.get(11)?,
        cache_read_tokens: r.get(12)?,
        cache_write_tokens: r.get(13)?,
        cost_microdollars: r.get(14)?,
        bytes: r.get(15)?,
        agent_cost_usd: usd(r.get(16)?),
        session_cost_usd: usd(r.get(17)?),
    })
}

const KINDS: [(v::EventKind, &str); 10] = [
    (v::EventKind::AgentStarted, "agent_started"),
    (v::EventKind::Prompt, "prompt"),
    (v::EventKind::ModelTurn, "model_turn"),
    (v::EventKind::ToolCall, "tool_call"),
    (v::EventKind::Decision, "decision"),
    (v::EventKind::ToolResult, "tool_result"),
    (v::EventKind::TurnEnd, "turn_end"),
    (v::EventKind::Compaction, "compaction"),
    (v::EventKind::Exposure, "exposure"),
    (v::EventKind::CacheBust, "cache_bust"),
];

/// The wire name of an event kind, as the `events.kind` column holds it.
pub fn kind_name(k: v::EventKind) -> &'static str {
    KINDS
        .iter()
        .find(|(x, _)| *x == k)
        .map_or("turn_end", |(_, n)| n)
}

fn kind_of(name: &str) -> Option<v::EventKind> {
    KINDS.iter().find(|(_, n)| *n == name).map(|(k, _)| *k)
}

/// The events the log shows first: prompts, model turns, tool calls, refusals, cache busts and
/// compactions.
fn notable(kind: v::EventKind, ok: Option<bool>) -> bool {
    match kind {
        v::EventKind::Prompt
        | v::EventKind::AgentStarted
        | v::EventKind::CacheBust
        | v::EventKind::Compaction => true,
        v::EventKind::Decision | v::EventKind::ToolResult => ok == Some(false),
        _ => false,
    }
}

fn last_change(conn: &Connection) -> rusqlite::Result<i64> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key = 'change'", [], |r| {
            r.get::<_, String>(0)
        })
        .optional()?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0))
}

/// Takes the next change number in `tx`.
fn next_change(tx: &rusqlite::Transaction<'_>) -> rusqlite::Result<i64> {
    let n = last_change(tx)? + 1;
    tx.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('change', ?1)",
        params![n.to_string()],
    )?;
    Ok(n)
}

fn mtime_ns(p: &Path) -> u128 {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos())
}

/// The transcript bytes one chunk of a sync lifts at once.
pub const LIFT_BUDGET: u64 = 192 << 20;

/// The bytes of a main transcript and of its sub-agent files.
fn transcript_bytes(main: &Path) -> u64 {
    let size = |p: &Path| fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    size(main)
        + crate::claude::sub_agent_files(main)
            .iter()
            .map(|s| size(&s.path))
            .sum::<u64>()
}

/// Gives the pages the allocator holds free back to the system. A lift allocates hundreds of
/// megabytes for a large transcript on each worker thread; glibc keeps them in the thread's arena
/// after they are freed, so a server that re-lifts a live session every few seconds grew to
/// gigabytes of resident memory that nothing used.
pub fn release_freed_memory() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        unsafe extern "C" {
            fn malloc_trim(pad: usize) -> i32;
        }
        // SAFETY: `malloc_trim` takes no pointer and only returns free pages of glibc's arenas.
        unsafe {
            malloc_trim(0);
        }
    }
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
    events: Vec<EventOut>,
    events_count: i64,
    last_ms: i64,
    names: String,
    figures: Figures,
    row: String,
    kpis: String,
    agents: Vec<(String, i64, String)>,
    checks: Vec<crate::sq::SessionCheck>,
}

/// One event row as projected, with the numbers its columns hold as integers.
struct EventOut {
    at_ms: i64,
    agent_cost: i64,
    session_cost: i64,
    row: v::EventAppended,
}

/// The figures of a session row the overview sums per hour.
#[derive(Debug, Clone, Copy, Default)]
struct Figures {
    cost_micro: i64,
    working_s: i64,
    waiting_share: f64,
    cache_share: f64,
    tool_calls: i64,
    refused: i64,
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
    pending: Vec<agenticspec_system::SystemEvent>,
    first: Option<i64>,
    last: Option<i64>,
    tracks: BTreeMap<String, Track>,
    session_cost: i64,
    refused_calls: i64,
    tool_names: Vec<String>,
    models: Vec<String>,
    /// Event rows projected since the last write.
    fresh: Vec<EventOut>,
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
    events: Vec<EventOut>,
    events_count: i64,
    last_ms: i64,
    names: String,
    figures: Figures,
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
            fresh: Vec::new(),
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
            ord: 0,
            at: ts(e.at),
            agent: s::AgentId(e.agent.clone()),
            kind: e.kind,
            notable: notable(e.kind, e.ok),
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
        self.fresh.push(EventOut {
            at_ms: e.at,
            agent_cost: t.cost,
            session_cost: self.session_cost,
            row,
        });
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
            wire::encode_event_agenticspec_view_agent_updated(&row, &mut out);
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
        wire::encode_event_agenticspec_view_session_updated(&row, &mut row_text);
        let mut kpis_text = String::new();
        wire::encode_agenticspec_cli_kpi_report(&kpis, &mut kpis_text);
        let checks = checks.then(|| self.checks(program));
        let figures = Figures {
            cost_micro: self.session_cost,
            working_s: row.working_seconds,
            waiting_share: row.waiting_share.get(),
            cache_share: row.cache_hit_share.get(),
            tool_calls: row.tool_calls,
            refused: row.refused_tool_calls,
        };
        // New events take the places after the ones written before, in the order they came.
        let mut events = std::mem::take(&mut self.fresh);
        let base = self.events_count - events.len() as i64;
        for (i, e) in events.iter_mut().enumerate() {
            e.row.ord = base + i as i64;
        }
        Delta {
            session_id: sid,
            path: self.origin.path.clone(),
            fingerprint: if self.origin.path.starts_with("wire:") {
                self.lifted.steps.len().to_string()
            } else {
                self.origin.fingerprint.clone()
            },
            agents_count: self.lifted.agents as i64,
            events,
            events_count: self.events_count,
            last_ms: last,
            names: row.names.clone(),
            figures,
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
        let mut d = self.delta(program, true);
        // A whole session is placed by time.
        d.events.sort_by_key(|e| (e.at_ms, e.row.seq));
        for (i, e) in d.events.iter_mut().enumerate() {
            e.row.ord = i as i64;
        }
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
            events: d.events,
            events_count: d.events_count,
            last_ms: d.last_ms,
            names: d.names,
            figures: d.figures,
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

fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map(|_| ())
    .map_err(sql)
}

fn write(conn: &mut Connection, r: &Record) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    let change = next_change(&tx)?;
    let old: Option<String> = tx
        .query_row(
            "SELECT session_id FROM sources WHERE path = ?1",
            params![r.path],
            |row| row.get(0),
        )
        .optional()?;
    for id in old.iter().chain(std::iter::once(&r.session_id)) {
        let sk: Option<i64> = tx
            .query_row(
                "SELECT sk FROM sessions WHERE session_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(sk) = sk {
            tx.execute("DELETE FROM events WHERE sk = ?1", params![sk])?;
            tx.execute("DELETE FROM session_names WHERE rowid = ?1", params![sk])?;
        }
        tx.execute("DELETE FROM sessions WHERE session_id = ?1", params![id])?;
        tx.execute("DELETE FROM checks WHERE session_id = ?1", params![id])?;
        if id != &r.session_id {
            tx.execute("DELETE FROM agents WHERE session_id = ?1", params![id])?;
        }
    }
    tx.execute(
        "INSERT OR REPLACE INTO sources (path, session_id, fingerprint) VALUES (?1, ?2, ?3)",
        params![r.path, r.session_id, r.fingerprint],
    )?;
    let f = r.figures;
    tx.execute(
        "INSERT INTO sessions (session_id, path, title, agents, files, steps, events_count,
           last_ms, names, row, kpis, change, cost_micro, working_s, waiting_share, cache_share,
           tool_calls, refused)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            r.session_id,
            r.path,
            r.title,
            r.agents_count,
            r.files,
            r.steps,
            r.events_count,
            r.last_ms,
            r.names,
            r.row,
            r.kpis,
            change,
            f.cost_micro,
            f.working_s,
            f.waiting_share,
            f.cache_share,
            f.tool_calls,
            f.refused
        ],
    )?;
    let sk = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO session_names (rowid, names) VALUES (?1, ?2)",
        params![sk, r.names],
    )?;
    put_events(&tx, sk, &r.events)?;
    put_agents(&tx, &r.session_id, &r.agents, change, true)?;
    put_checks(&tx, &r.session_id, &r.checks)?;
    put_hourly(&tx)?;
    tx.commit()
}

/// Writes what one append changed: the session row, the new event rows, every agent row that
/// changed, and the verdicts when they were judged again.
fn write_delta(conn: &mut Connection, d: &Delta) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    let change = next_change(&tx)?;
    let f = d.figures;
    tx.execute(
        "UPDATE sessions SET agents = ?2, events_count = ?3, last_ms = ?4, row = ?5, kpis = ?6,
           names = ?7, change = ?8, cost_micro = ?9, working_s = ?10, waiting_share = ?11,
           cache_share = ?12, tool_calls = ?13, refused = ?14
         WHERE session_id = ?1",
        params![
            d.session_id,
            d.agents_count,
            d.events_count,
            d.last_ms,
            d.row,
            d.kpis,
            d.names,
            change,
            f.cost_micro,
            f.working_s,
            f.waiting_share,
            f.cache_share,
            f.tool_calls,
            f.refused
        ],
    )?;
    tx.execute(
        "UPDATE sources SET fingerprint = ?2 WHERE path = ?1",
        params![d.path, d.fingerprint],
    )?;
    let sk: i64 = tx.query_row(
        "SELECT sk FROM sessions WHERE session_id = ?1",
        params![d.session_id],
        |row| row.get(0),
    )?;
    tx.execute(
        "UPDATE session_names SET names = ?2 WHERE rowid = ?1",
        params![sk, d.names],
    )?;
    put_events(&tx, sk, &d.events)?;
    put_agents(&tx, &d.session_id, &d.agents, change, false)?;
    if let Some(checks) = &d.checks {
        tx.execute(
            "DELETE FROM checks WHERE session_id = ?1",
            params![d.session_id],
        )?;
        put_checks(&tx, &d.session_id, checks)?;
    }
    put_hourly(&tx)?;
    tx.commit()
}

fn put_events(
    tx: &rusqlite::Transaction<'_>,
    sk: i64,
    events: &[EventOut],
) -> rusqlite::Result<()> {
    let mut st = tx.prepare(
        "INSERT OR REPLACE INTO events (sk, seq, ord, at_ms, agent, kind, notable, label, detail,
           call, ok, context_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
           cost_micro, bytes, agent_cost_micro, session_cost_micro)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18,
           ?19)",
    )?;
    for e in events {
        let r = &e.row;
        st.execute(params![
            sk,
            r.seq,
            r.ord,
            e.at_ms,
            r.agent.0,
            kind_name(r.kind),
            r.notable,
            r.label,
            r.detail,
            r.call,
            r.ok,
            r.context_tokens,
            r.output_tokens,
            r.cache_read_tokens,
            r.cache_write_tokens,
            r.cost_microdollars,
            r.bytes,
            e.agent_cost,
            e.session_cost
        ])?;
    }
    Ok(())
}

/// The sums of the session figures per hour of `last_ms`, written again from the session rows.
fn put_hourly(tx: &rusqlite::Transaction<'_>) -> rusqlite::Result<()> {
    tx.execute_batch(
        "DELETE FROM hourly;
         INSERT INTO hourly (hour, sessions, cost_micro, working_s, waiting_share, cache_share,
           tool_calls, refused)
         SELECT last_ms / 3600000, count(*), sum(cost_micro), sum(working_s), sum(waiting_share),
           sum(cache_share), sum(tool_calls), sum(refused)
         FROM sessions GROUP BY last_ms / 3600000;",
    )
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
    change: i64,
    whole: bool,
) -> rusqlite::Result<()> {
    // A row keeps its change number while its text stays the same, so a live channel sends
    // only the agents that changed.
    let mut st = tx.prepare(
        "INSERT INTO agents (session_id, agent_id, started_ms, row, change)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (session_id, agent_id) DO UPDATE
           SET started_ms = excluded.started_ms, row = excluded.row, change = excluded.change
           WHERE agents.row != excluded.row",
    )?;
    for (id, started, row) in agents {
        st.execute(params![session_id, id, started, row, change])?;
    }
    if whole {
        // The agents of a whole session are all of them: drop the ones it no longer has.
        let keep: Vec<&str> = agents.iter().map(|(id, _, _)| id.as_str()).collect();
        let mut have = tx.prepare("SELECT agent_id FROM agents WHERE session_id = ?1")?;
        let gone: Vec<String> = have
            .query_map(params![session_id], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?
            .into_iter()
            .filter(|a| !keep.contains(&a.as_str()))
            .collect();
        for a in gone {
            tx.execute(
                "DELETE FROM agents WHERE session_id = ?1 AND agent_id = ?2",
                params![session_id, a],
            )?;
        }
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
        Cmd::OpenSession(i) => wire::encode_command_agenticspec_session_open_session(i, &mut out),
        Cmd::StartAgent(i) => wire::encode_command_agenticspec_session_start_agent(i, &mut out),
        Cmd::SubmitPrompt(i) => wire::encode_command_agenticspec_session_submit_prompt(i, &mut out),
        Cmd::EndTurn(i) => wire::encode_command_agenticspec_session_end_turn(i, &mut out),
        Cmd::RecordModelTurn(i) => {
            wire::encode_command_agenticspec_session_record_model_turn(i, &mut out)
        }
        Cmd::CompactContext(i) => {
            wire::encode_command_agenticspec_session_compact_context(i, &mut out)
        }
        Cmd::RequestToolCall(i) => {
            wire::encode_command_agenticspec_session_request_tool_call(i, &mut out)
        }
        Cmd::DecideToolCall(i) => {
            wire::encode_command_agenticspec_session_decide_tool_call(i, &mut out)
        }
        Cmd::CompleteToolCall(i) => {
            wire::encode_command_agenticspec_session_complete_tool_call(i, &mut out)
        }
        Cmd::RecordExposure(i) => {
            wire::encode_command_agenticspec_session_record_exposure(i, &mut out)
        }
    }
    out
}

fn command_of(name: &str, value: &json::Value) -> Result<Cmd, String> {
    let e = |e: json::DecodeError| format!("{name}: {e:?}");
    Ok(match name {
        "OpenSession" => Cmd::OpenSession(
            wire::decode_command_agenticspec_session_open_session(value, "$").map_err(e)?,
        ),
        "StartAgent" => Cmd::StartAgent(
            wire::decode_command_agenticspec_session_start_agent(value, "$").map_err(e)?,
        ),
        "SubmitPrompt" => Cmd::SubmitPrompt(
            wire::decode_command_agenticspec_session_submit_prompt(value, "$").map_err(e)?,
        ),
        "EndTurn" => {
            Cmd::EndTurn(wire::decode_command_agenticspec_session_end_turn(value, "$").map_err(e)?)
        }
        "RecordModelTurn" => Cmd::RecordModelTurn(
            wire::decode_command_agenticspec_session_record_model_turn(value, "$").map_err(e)?,
        ),
        "CompactContext" => Cmd::CompactContext(
            wire::decode_command_agenticspec_session_compact_context(value, "$").map_err(e)?,
        ),
        "RequestToolCall" => Cmd::RequestToolCall(
            wire::decode_command_agenticspec_session_request_tool_call(value, "$").map_err(e)?,
        ),
        "DecideToolCall" => Cmd::DecideToolCall(
            wire::decode_command_agenticspec_session_decide_tool_call(value, "$").map_err(e)?,
        ),
        "CompleteToolCall" => Cmd::CompleteToolCall(
            wire::decode_command_agenticspec_session_complete_tool_call(value, "$").map_err(e)?,
        ),
        "RecordExposure" => Cmd::RecordExposure(
            wire::decode_command_agenticspec_session_record_exposure(value, "$").map_err(e)?,
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
