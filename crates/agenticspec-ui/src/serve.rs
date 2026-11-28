//! `agenticspec ui serve`: the default control plane of one architecture on fixtures, with no
//! runtime. It answers the runtime interface (`ui/runtime.openapi.yaml`) under the operator
//! ingress, `{url}/ingress/operator/scopes/{architecture}[/{part}[/{instance}]]`, and serves the
//! operator app `agenticspec ui build` writes for the same architecture at `/`, its `config.json`
//! pointing at those routes:
//!
//! - R1 answers each view of the plane (see [`crate::plane`]) with the rows of the panels'
//!   fixture file (`ui/views/fixtures/<view>.json`) at every scope of the architecture;
//! - R2 plays the channel scripts (`ui/views/fixtures/scripts/`): a level's channel plays every
//!   script, one after the other, a block's channel its own, each event the operator ingress
//!   carries once, with its position as the cursor; a reconnect with `Last-Event-ID` gets the
//!   events after it;
//! - R4 records each command (`GET /fixture/commands` lists them) and answers the outcome
//!   `ui/views/source/outcomes.json` scripts for it, `{"row": null}` when none does;
//! - R5 allows every command of the plane at every scope.
//!
//! Those files are built into the binary as one compact file, [`BUNDLE_FILE`] (written by
//! `agenticspec ui panels` with [`write_bundle`], held by `ui panels --check`), so a released
//! binary serves with no checkout ([`Fixtures::embedded`]); `--fixtures <dir>` reads a consumer's
//! own directory instead ([`Fixtures::from_dir`]).
//!
//! It listens on 127.0.0.1 only and answers only a request whose `Host` is `127.0.0.1` or
//! `localhost`. Every request of the interface carries the token, as `Authorization: Bearer` or
//! the session cookie, which the server sets when it serves the app's page.
//!
//! [`Faults`] makes it break one obligation at a time, so a test proves that
//! `agenticspec ui conform --architecture` notices.

use std::collections::BTreeMap;
use std::io::{self, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::interface::{self, Ordered};
use crate::plane::Plane;
use crate::source::{json_files, read_json};
use crate::wire::{self, Request};

/// The base path of the operator ingress; the scopes are below `{ROOT}/scopes/`.
pub const ROOT: &str = "/ingress/operator";
/// The port when none is named, the default `root` of `ui/runtime.openapi.yaml`.
pub const PORT: u16 = 7430;
/// The panels' fixtures below the repository root.
pub const FIXTURES: &str = "ui/views/fixtures";
/// The scripted outcomes of the commands below the repository root.
pub const OUTCOMES_FILE: &str = "ui/views/source/outcomes.json";
/// Where the record of the commands is read.
pub const RECORD: &str = "/fixture/commands";
/// The grant R5 names for every command it allows.
pub const GRANT: &str = "fixture-allows-every-action";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    command: String,
    #[serde(default)]
    input: Option<Map<String, Value>>,
    #[serde(default)]
    accepted: Option<Value>,
    #[serde(default)]
    refused: Option<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutcomesFile {
    format: String,
    outcomes: Vec<Outcome>,
}

#[derive(Deserialize)]
struct ViewFile {
    view: String,
    rows: Vec<Ordered>,
}

#[derive(Deserialize)]
struct ScriptEntry {
    event: String,
    payload: Value,
}

#[derive(Deserialize)]
struct ScriptFile {
    channel: String,
    events: Vec<ScriptEntry>,
}

/// The fixtures built into the binary, below the repository root: every view fixture, every
/// channel script and the outcomes in one compact file (`agenticspec ui panels` writes it).
pub const BUNDLE_FILE: &str = "ui/views/serve.fixtures.json";
/// The format of [`BUNDLE_FILE`].
pub const BUNDLE_FORMAT: &str = "agenticspec-serve-fixtures/1";
/// The bytes of [`BUNDLE_FILE`] this binary was built with.
pub const BUNDLE: &str = include_str!("../../../ui/views/serve.fixtures.json");
/// The outcomes file in a directory named with `ui serve --fixtures`.
pub const DIR_OUTCOMES: &str = "outcomes.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleFile {
    format: String,
    #[allow(dead_code)]
    sources_sha256: String,
    views: Vec<ViewFile>,
    scripts: Vec<ScriptFile>,
    outcomes: OutcomesFile,
}

/// What the server answers from: the panels' rows by view, the channel scripts in the order of
/// their files, and the scripted outcomes.
#[derive(Debug, Clone, PartialEq)]
pub struct Fixtures {
    views: BTreeMap<String, Vec<Ordered>>,
    scripts: Vec<Script>,
    outcomes: Vec<Outcome>,
}

/// The JSON files of the view fixtures and of the channel scripts below the repository root,
/// each sorted, and the outcomes file: what [`bundle`] reads.
fn sources(root: &Path) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let named = |p: std::path::PathBuf| {
        let rel = p
            .strip_prefix(root)
            .map_or_else(|_| p.display().to_string(), |r| r.display().to_string())
            .replace('\\', "/");
        (rel, p)
    };
    let mut out: Vec<_> = json_files(&root.join(FIXTURES))?
        .into_iter()
        .map(named)
        .collect();
    out.extend(
        json_files(&root.join(FIXTURES).join("scripts"))?
            .into_iter()
            .map(named),
    );
    out.push((OUTCOMES_FILE.to_string(), root.join(OUTCOMES_FILE)));
    Ok(out)
}

/// `text` without the whitespace between JSON tokens; strings are kept byte for byte.
fn minify(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let (mut in_string, mut escaped) = (false, false);
    for c in text.chars() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if !c.is_ascii_whitespace() {
            out.push(c);
        }
    }
    out
}

/// The text of [`BUNDLE_FILE`] from the fixtures below the repository root: each view fixture
/// and each channel script as its file holds it, in the order of the files, and the outcomes,
/// without whitespace between tokens, with the sha256 of the lines `<sha256>  <path>` of every
/// file read.
///
/// # Errors
///
/// A file cannot be read, or the bundle does not read back as the fixtures of the files.
pub fn bundle(root: &Path) -> Result<String, String> {
    let files = sources(root)?;
    let mut digest = String::new();
    let (mut views, mut scripts, mut outcomes) = (Vec::new(), Vec::new(), String::new());
    for (rel, path) in &files {
        let bytes = std::fs::read(path).map_err(|e| format!("{rel}: {e}"))?;
        digest.push_str(&format!("{}  {rel}\n", crate::operator::sha256(&bytes)));
        let text = minify(&String::from_utf8_lossy(&bytes));
        if rel == OUTCOMES_FILE {
            outcomes = text;
        } else if rel.starts_with(&format!("{FIXTURES}/scripts/")) {
            scripts.push(text);
        } else {
            views.push(text);
        }
    }
    let text = format!(
        "{{\"format\":\"{BUNDLE_FORMAT}\",\"sources_sha256\":\"{}\",\"views\":[{}],\"scripts\":[{}],\"outcomes\":{outcomes}}}\n",
        crate::operator::sha256(digest.as_bytes()),
        views.join(","),
        scripts.join(","),
    );
    if Fixtures::parse_bundle(&text)? != Fixtures::load(root)? {
        return Err(format!(
            "{BUNDLE_FILE}: does not read back as the fixtures it is written from"
        ));
    }
    Ok(text)
}

/// Writes [`BUNDLE_FILE`] below the repository root, or with `check` compares it with a fresh
/// bundle.
///
/// # Errors
///
/// A source cannot be read, the file cannot be written, or with `check` it differs (its sources
/// changed since it was written, or it was edited).
pub fn write_bundle(root: &Path, check: bool) -> Result<(), String> {
    let fresh = bundle(root)?;
    let path = root.join(BUNDLE_FILE);
    if !check {
        return std::fs::write(&path, &fresh).map_err(|e| format!("{BUNDLE_FILE}: {e}"));
    }
    let held = std::fs::read_to_string(&path).map_err(|e| format!("{BUNDLE_FILE}: {e}"))?;
    if held == fresh {
        return Ok(());
    }
    let digest = |t: &str| {
        serde_json::from_str::<Value>(t)
            .ok()
            .and_then(|v| v["sources_sha256"].as_str().map(String::from))
    };
    let why = if digest(&held) == digest(&fresh) {
        "differs from a fresh bundle of the same sources"
    } else {
        "was written from other sources than the current fixtures and outcomes"
    };
    Err(format!(
        "{BUNDLE_FILE}: {why}. Generated files are never edited: run task ui:panels"
    ))
}

impl Fixtures {
    /// Reads the fixtures below the repository root `root`: the view fixtures and channel
    /// scripts of [`FIXTURES`] and the outcomes of [`OUTCOMES_FILE`].
    ///
    /// # Errors
    ///
    /// A file cannot be read, is not the JSON it should be, or an outcome is not exactly one of
    /// `accepted` and `refused` (a refusal naming its `error`).
    pub fn load(root: &Path) -> Result<Fixtures, String> {
        let dir = root.join(FIXTURES);
        let views = read_views(&dir)?;
        let scripts = read_scripts(&dir.join("scripts"))?;
        let file: OutcomesFile = read_json(&root.join(OUTCOMES_FILE))?;
        Fixtures::assemble(views, scripts, file, OUTCOMES_FILE)
    }

    /// The fixtures built into this binary ([`BUNDLE`]).
    ///
    /// # Panics
    ///
    /// The built-in bundle does not read; a test holds it against the files it is written from.
    pub fn embedded() -> Fixtures {
        Fixtures::parse_bundle(BUNDLE).expect("the fixtures built into the binary")
    }

    /// The fixtures of a directory a consumer names with `ui serve --fixtures`: one
    /// `<view>.json` per view (`{view, rows}`), the channel scripts in `scripts/`
    /// (`{channel, events}`, optional) and the outcomes in `outcomes.json`
    /// (`agenticspec-command-outcomes/1`, optional).
    ///
    /// # Errors
    ///
    /// The directory holds no view fixture, or a file cannot be read or is not the JSON it
    /// should be.
    pub fn from_dir(dir: &Path) -> Result<Fixtures, String> {
        let views = read_views(dir)?;
        if views.is_empty() {
            return Err(format!(
                "{}: holds no view fixture (<view>.json)",
                dir.display()
            ));
        }
        let scripts_dir = dir.join("scripts");
        let scripts = if scripts_dir.is_dir() {
            read_scripts(&scripts_dir)?
        } else {
            Vec::new()
        };
        let outcomes_path = dir.join(DIR_OUTCOMES);
        let file = if outcomes_path.exists() {
            read_json(&outcomes_path)?
        } else {
            OutcomesFile {
                format: "agenticspec-command-outcomes/1".to_string(),
                outcomes: Vec::new(),
            }
        };
        Fixtures::assemble(views, scripts, file, &outcomes_path.display().to_string())
    }

    fn parse_bundle(text: &str) -> Result<Fixtures, String> {
        let b: BundleFile =
            serde_json::from_str(text).map_err(|e| format!("{BUNDLE_FILE}: {e}"))?;
        if b.format != BUNDLE_FORMAT {
            return Err(format!(
                "{BUNDLE_FILE}: format {} is not {BUNDLE_FORMAT}",
                b.format
            ));
        }
        let views = b.views.into_iter().map(|v| (v.view, v.rows)).collect();
        let scripts = b.scripts.into_iter().map(script).collect();
        Fixtures::assemble(views, scripts, b.outcomes, BUNDLE_FILE)
    }

    fn assemble(
        views: BTreeMap<String, Vec<Ordered>>,
        scripts: Vec<Script>,
        file: OutcomesFile,
        shown: &str,
    ) -> Result<Fixtures, String> {
        if file.format != "agenticspec-command-outcomes/1" {
            return Err(format!(
                "{shown}: format {} is not agenticspec-command-outcomes/1",
                file.format
            ));
        }
        for (i, o) in file.outcomes.iter().enumerate() {
            let refusal_named = o
                .refused
                .as_ref()
                .is_none_or(|r| r["error"].as_str().is_some());
            if o.accepted.is_some() == o.refused.is_some() || !refusal_named {
                return Err(format!(
                    "{shown}: outcomes/{i} ({}) holds exactly one of accepted and refused, and a refusal names its error",
                    o.command
                ));
            }
        }
        Ok(Fixtures {
            views,
            scripts,
            outcomes: file.outcomes,
        })
    }
}

/// A channel script: its channel and its events, each an event name and its payload.
type Script = (String, Vec<(String, Value)>);

fn script(s: ScriptFile) -> Script {
    let events = s.events.into_iter().map(|e| (e.event, e.payload)).collect();
    (s.channel, events)
}

/// The view fixtures of `dir` by view: every `*.json` but [`DIR_OUTCOMES`].
fn read_views(dir: &Path) -> Result<BTreeMap<String, Vec<Ordered>>, String> {
    let mut views = BTreeMap::new();
    for p in json_files(dir)? {
        if p.file_name().is_some_and(|n| n == DIR_OUTCOMES) {
            continue;
        }
        let v: ViewFile = read_json(&p)?;
        views.insert(v.view, v.rows);
    }
    Ok(views)
}

/// The channel scripts of `dir`, in the order of their files.
fn read_scripts(dir: &Path) -> Result<Vec<Script>, String> {
    json_files(dir)?
        .iter()
        .map(|p| read_json::<ScriptFile>(p).map(script))
        .collect()
}

/// One obligation the server breaks on purpose.
#[derive(Debug, Clone, Default)]
pub struct Faults {
    /// R1 of this view adds to each row a field the view does not declare.
    pub undeclared_field: Option<String>,
    /// A reconnect gets the event of its cursor again.
    pub resend_on_resume: bool,
    /// R5 withholds this command, and R4 still carries it out.
    pub withhold: Option<String>,
}

/// How the server runs.
#[derive(Debug, Clone)]
pub struct Options {
    /// The port on 127.0.0.1; 0 lets the system pick one.
    pub port: u16,
    /// The token every request of the interface carries.
    pub token: String,
    pub faults: Faults,
}

/// A token nobody can guess, for a server started without one.
pub fn random_token() -> String {
    use std::hash::{BuildHasher, Hasher};
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    (0..2u8)
        .map(|i| {
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u8(i);
            h.write_u128(now);
            h.write_u32(std::process::id());
            format!("{:016x}", h.finish())
        })
        .collect()
}

/// A running server. Dropping it stops it.
pub struct Served {
    url: String,
    recorded: Arc<Mutex<Vec<Value>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Served {
    /// Starts the server on 127.0.0.1 with the files of the operator app `app` (what
    /// [`crate::operator::build_with`] writes for the plane's architecture, `api_base` [`ROOT`]).
    ///
    /// # Errors
    ///
    /// The port cannot be bound.
    pub fn start(
        plane: Plane,
        app: Vec<(String, Vec<u8>)>,
        fixtures: Fixtures,
        options: Options,
    ) -> io::Result<Served> {
        let listener = TcpListener::bind(("127.0.0.1", options.port))?;
        let url = format!("http://{}", listener.local_addr()?);
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let shared = Arc::new(Shared {
            plane,
            app,
            fixtures,
            token: options.token,
            faults: options.faults,
            recorded: Arc::clone(&recorded),
            stop: Arc::clone(&stop),
        });
        let thread = std::thread::spawn(move || {
            for conn in listener.incoming() {
                if shared.stop.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(conn) = conn else { continue };
                let shared = Arc::clone(&shared);
                std::thread::spawn(move || shared.connection(conn));
            }
        });
        Ok(Served {
            url,
            recorded,
            stop,
            thread: Some(thread),
        })
    }

    /// `http://127.0.0.1:<port>`, where the operator app is served.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The root of the runtime interface, `{url}/ingress/operator`.
    pub fn interface_root(&self) -> String {
        format!("{}{ROOT}", self.url)
    }

    /// The commands received so far, oldest first: `{scope, command, input, status}`.
    pub fn recorded(&self) -> Vec<Value> {
        self.recorded.lock().map(|r| r.clone()).unwrap_or_default()
    }

    /// Serves until the process ends.
    pub fn wait(mut self) {
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(addr) = self.url.strip_prefix("http://") {
            let _ = TcpStream::connect(addr);
        }
    }
}

struct Shared {
    plane: Plane,
    app: Vec<(String, Vec<u8>)>,
    fixtures: Fixtures,
    token: String,
    faults: Faults,
    recorded: Arc<Mutex<Vec<Value>>>,
    stop: Arc<AtomicBool>,
}

fn error(status: u16, name: &str, message: impl Into<String>) -> (u16, String) {
    (
        status,
        json!({"error": name, "message": message.into()}).to_string(),
    )
}

fn content_type(name: &str) -> &'static str {
    match name.rsplit_once('.').map(|(_, x)| x) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

fn write_bytes(
    out: &mut impl Write,
    status: u16,
    extra: &[(&str, &str)],
    ctype: &str,
    body: &[u8],
) -> io::Result<()> {
    let mut head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n",
        wire::reason(status),
        body.len()
    );
    for (k, v) in extra {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str("\r\n");
    out.write_all(head.as_bytes())?;
    out.write_all(body)?;
    out.flush()
}

/// The scope of a request, its part by path.
struct Scope {
    part: Option<String>,
    instance: Option<u32>,
}

impl Shared {
    fn scope_text(&self, s: &Scope) -> String {
        interface::scope_path(&self.plane.architecture, s.part.as_deref(), s.instance)
    }

    fn connection(&self, stream: TcpStream) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
        let Ok(write) = stream.try_clone() else {
            return;
        };
        let mut reader = BufReader::new(stream);
        let mut out = write;
        let request = match wire::read_request(&mut reader) {
            Ok(r) => r,
            Err(_) => {
                let (s, b) = error(400, "InvalidRequest", "not an HTTP request");
                let _ = wire::write_json(&mut out, s, &[], &b);
                return;
            }
        };
        let _ = self.answer(&request, &mut out);
    }

    /// Whether the request names this host: `127.0.0.1` or `localhost`, any port.
    fn local_host(r: &Request) -> bool {
        r.header("host")
            .map(|h| h.rsplit_once(':').map_or(h, |(name, _)| name))
            .is_some_and(|name| name == "127.0.0.1" || name.eq_ignore_ascii_case("localhost"))
    }

    fn authenticated(&self, r: &Request) -> bool {
        let bearer = r
            .header("authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim);
        let name = interface::session_cookie();
        let cookie = r.header("cookie").and_then(|c| {
            c.split(';').find_map(|kv| {
                let (k, v) = kv.trim().split_once('=')?;
                (k == name).then_some(v)
            })
        });
        bearer.or(cookie) == Some(self.token.as_str())
    }

    fn answer(&self, r: &Request, out: &mut TcpStream) -> io::Result<()> {
        if !Self::local_host(r) {
            let (s, b) = error(
                403,
                "ForeignHost",
                "the server answers a request to 127.0.0.1 or localhost only",
            );
            return wire::write_json(out, s, &[], &b);
        }
        let interface = r
            .path
            .strip_prefix(ROOT)
            .filter(|rest| rest.starts_with('/'));
        if interface.is_some() || r.path == RECORD {
            if !self.authenticated(r) {
                let (s, b) = error(
                    401,
                    "Unauthenticated",
                    "send Authorization: Bearer <token> or the session cookie",
                );
                return wire::write_json(out, s, &[("WWW-Authenticate", "Bearer")], &b);
            }
            if r.path == RECORD {
                let list = self.recorded.lock().map(|l| l.clone()).unwrap_or_default();
                return wire::write_json(out, 200, &[], &Value::Array(list).to_string());
            }
            return self.interface(r, interface.unwrap_or(""), out);
        }
        if r.method != "GET" {
            let (s, b) = error(405, "MethodNotAllowed", format!("{} {}", r.method, r.path));
            return wire::write_json(out, s, &[], &b);
        }
        let name = match r.path.trim_start_matches('/') {
            "" => "index.html",
            n => n,
        };
        let Some((_, bytes)) = self.app.iter().find(|(n, _)| n == name) else {
            let (s, b) = error(404, "NotFound", format!("{} is no file of the app", r.path));
            return wire::write_json(out, s, &[], &b);
        };
        let cookie = format!(
            "{}={}; Path=/; HttpOnly; SameSite=Strict",
            interface::session_cookie(),
            self.token
        );
        let extra: Vec<(&str, &str)> = if name == "index.html" {
            vec![("Set-Cookie", cookie.as_str())]
        } else {
            Vec::new()
        };
        write_bytes(out, 200, &extra, content_type(name), bytes)
    }

    /// A request below the operator ingress, `/scopes/{scope}/api/...`.
    fn interface(&self, r: &Request, path: &str, out: &mut TcpStream) -> io::Result<()> {
        let (scope, rest) = match self.scope_of(path) {
            Ok(x) => x,
            Err((s, b)) => return wire::write_json(out, s, &[], &b),
        };
        let mut words = rest.splitn(2, '/');
        let (kind, name) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
        let (status, body) = match (r.method.as_str(), kind) {
            ("GET", "views") if !name.is_empty() => self.read(name),
            ("GET", "channels") if !name.is_empty() => return self.stream(r, name, out),
            ("POST", "commands") if !name.is_empty() => self.command(r, &scope, name),
            ("GET", "actions") if name.is_empty() => self.actions(),
            (_, "views" | "channels" | "commands" | "actions") => {
                error(405, "MethodNotAllowed", format!("{} {}", r.method, r.path))
            }
            _ => error(
                404,
                "NoSuchPath",
                format!("{} is no path of the interface", r.path),
            ),
        };
        wire::write_json(out, status, &[], &body)
    }

    fn scope_of<'a>(&self, path: &'a str) -> Result<(Scope, &'a str), (u16, String)> {
        let prefix = interface::scope_prefix();
        let no_scope = || {
            error(
                404,
                "NoSuchScope",
                format!(
                    "{ROOT}{path}: the interface answers below {ROOT}{prefix}{}",
                    self.plane.architecture
                ),
            )
        };
        let below = path.strip_prefix(&prefix).ok_or_else(no_scope)?;
        let (scope, rest) = below.split_once("/api/").ok_or_else(no_scope)?;
        let segs: Vec<&str> = scope.split('/').collect();
        if segs.len() > 3 || segs[0] != self.plane.architecture {
            return Err(no_scope());
        }
        let part = match segs.get(1) {
            None => None,
            Some(p) => Some(self.plane.part(p).ok_or_else(|| {
                error(
                    404,
                    "NoSuchScope",
                    format!("no part {p} in {}", self.plane.architecture),
                )
            })?),
        };
        let instance = match (segs.get(2), part) {
            (None, _) => None,
            (Some(i), Some(p)) => {
                let n = i
                    .parse::<u32>()
                    .ok()
                    .filter(|n| *n >= 1 && u64::from(*n) <= p.count);
                Some(n.ok_or_else(|| {
                    error(
                        404,
                        "NoSuchScope",
                        format!("part {} runs {} copies, not a copy {i}", p.path, p.count),
                    )
                })?)
            }
            (Some(_), None) => return Err(no_scope()),
        };
        Ok((
            Scope {
                part: part.map(|p| p.path.clone()),
                instance,
            },
            rest,
        ))
    }

    fn read(&self, view: &str) -> (u16, String) {
        let rows = self
            .fixtures
            .views
            .get(view)
            .filter(|_| self.plane.views.contains(view));
        let Some(rows) = rows else {
            return error(
                404,
                "NoSuchView",
                format!("{view} is no view of the control plane"),
            );
        };
        let mut rows = rows.clone();
        if self.faults.undeclared_field.as_deref() == Some(view) {
            for r in &mut rows {
                r.0.push((
                    "undeclared".into(),
                    json!("a field the view does not declare"),
                ));
            }
        }
        let text = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into());
        (200, format!("{{\"rows\":{text}}}"))
    }

    /// The events of a channel, each the operator ingress carries, in order.
    fn events(&self, channel: &str) -> Option<Vec<&(String, Value)>> {
        let level = self.plane.channels.iter().any(|c| c == channel);
        let scripts: Vec<_> = self
            .fixtures
            .scripts
            .iter()
            .filter(|(name, _)| level || name == channel)
            .collect();
        if scripts.is_empty() {
            return None;
        }
        let feeds = &self.plane.feeds;
        Some(
            scripts
                .into_iter()
                .flat_map(|(_, events)| events.iter())
                .filter(|(event, _)| feeds.is_empty() || feeds.contains(event))
                .collect(),
        )
    }

    fn stream(&self, r: &Request, channel: &str, out: &mut TcpStream) -> io::Result<()> {
        let Some(events) = self.events(channel) else {
            let (s, b) = error(
                404,
                "NoSuchChannel",
                format!("no channel {channel} in the control plane"),
            );
            return wire::write_json(out, s, &[], &b);
        };
        let after = r
            .header(&interface::cursor_header())
            .and_then(|v| v.trim().parse::<usize>().ok());
        out.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        )?;
        for (i, (event, payload)) in events.into_iter().enumerate() {
            let cursor = i + 1;
            let skip = match after {
                Some(c) if self.faults.resend_on_resume => cursor < c,
                Some(c) => cursor <= c,
                None => false,
            };
            if skip {
                continue;
            }
            let id = cursor.to_string();
            out.write_all(wire::event_text(event, Some(&id), &payload.to_string()).as_bytes())?;
        }
        out.flush()?;
        while !self.stop.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(200));
            out.write_all(b": keep-alive\n\n")?;
            out.flush()?;
        }
        Ok(())
    }

    fn command(&self, r: &Request, scope: &Scope, command: &str) -> (u16, String) {
        let input: Map<String, Value> = match serde_json::from_slice(&r.body) {
            Ok(Value::Object(m)) => m,
            _ => return error(400, "InvalidRequest", "a command's input is a JSON object"),
        };
        if !self.plane.commands.contains(command) {
            return error(
                404,
                "NoSuchCommand",
                format!("{command} is no command of the control plane"),
            );
        }
        let found = self.fixtures.outcomes.iter().find(|o| {
            o.command == command
                && o.input
                    .as_ref()
                    .is_none_or(|want| want.iter().all(|(k, v)| input.get(k) == Some(v)))
        });
        let (status, row) = match found {
            Some(Outcome {
                refused: Some(row), ..
            }) => (422, row.clone()),
            Some(Outcome {
                accepted: Some(row),
                ..
            }) => (200, row.clone()),
            _ => (200, Value::Null),
        };
        if let Ok(mut list) = self.recorded.lock() {
            list.push(json!({
                "scope": self.scope_text(scope),
                "command": command,
                "input": input,
                "status": status,
            }));
        }
        (status, json!({ "row": row }).to_string())
    }

    fn actions(&self) -> (u16, String) {
        let list: Vec<Value> = self
            .plane
            .commands
            .iter()
            .map(|c| {
                if self.faults.withhold.as_deref() == Some(c.as_str()) {
                    json!({"command": c, "allowed": false})
                } else {
                    json!({"command": c, "allowed": true, "grant_id": GRANT})
                }
            })
            .collect();
        (200, Value::Array(list).to_string())
    }
}
