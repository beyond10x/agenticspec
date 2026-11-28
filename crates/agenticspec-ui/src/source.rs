//! The fixture source: an adapter that answers R1-R5 of the runtime interface
//! (`ui/runtime.openapi.yaml`) at a scope, from the panels' fixtures (`ui/views/fixtures/`, a file
//! per view and a script per channel) and the fixture source file (`ui/views/source/source.json`:
//! the principals by token, the placements, the architecture's parts, the grants, the action class
//! of each command and the answers of the commands). It listens on 127.0.0.1 only, on a port the
//! system picks, and stands in for a runtime in tests and in `agenticspec ui conform`.
//!
//! [`Faults`] makes it break one obligation at a time, so a test proves the conformance suite
//! notices.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::interface::{self, Ordered};
use crate::wire::{self, Request};

/// The fixture source file below the repository root.
pub const SOURCE_FILE: &str = "ui/views/source/source.json";
/// The conformance plan of the fixture source below the repository root.
pub const PLAN_FILE: &str = "ui/views/source/plan.json";
/// The panels' fixtures below the repository root.
pub const FIXTURES: &str = "ui/views/fixtures";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Principal {
    token: String,
    principal_id: String,
}

/// A grant as R5 judges it: the fields of `agentic.authority.Grant` the evaluator reads.
#[derive(Debug, Clone, Deserialize)]
pub struct Grant {
    pub grant_id: String,
    #[serde(default)]
    pub grantee_kind: Option<String>,
    pub grantee_id: String,
    pub action_class: String,
    pub scope: String,
    pub effect: String,
    pub state: String,
}

/// What the evaluator decides for a command at a scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// An `Allow` or `ApprovalRequired` grant covers it.
    Allowed(String),
    /// A `Deny` grant covers it.
    Forbidden(String),
    /// No grant covers the command's action class there.
    NotCovered(String),
}

/// The authority evaluator R5 and R4 share: `Deny` first, then the first grant that allows.
pub fn evaluate(
    grants: &[Grant],
    classes: &BTreeMap<String, String>,
    principal: &str,
    command: &str,
    scope: &str,
) -> Verdict {
    let class = classes.get(command).cloned().unwrap_or_default();
    let mine = grants.iter().filter(|g| {
        g.grantee_id == principal
            && g.state == "Active"
            && !class.is_empty()
            && g.action_class == class
            && interface::covers(&g.scope, scope)
    });
    let mut allow = None;
    for g in mine {
        match g.effect.as_str() {
            "Deny" => return Verdict::Forbidden(g.grant_id.clone()),
            "Allow" | "ApprovalRequired" if allow.is_none() => allow = Some(g.grant_id.clone()),
            _ => {}
        }
    }
    allow.map_or(Verdict::NotCovered(class), Verdict::Allowed)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandFixture {
    command: String,
    input: Map<String, Value>,
    #[serde(default)]
    accepted: Option<Value>,
    #[serde(default)]
    refused: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFile {
    format: String,
    architecture_id: String,
    principals: Vec<Principal>,
    #[serde(default)]
    views: BTreeMap<String, Vec<Ordered>>,
    placements: Vec<Ordered>,
    action_classes: BTreeMap<String, String>,
    grants: Vec<Grant>,
    #[serde(default)]
    commands: Vec<CommandFixture>,
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

/// Everything the fixture source answers from.
#[derive(Clone)]
pub struct Data {
    views: BTreeMap<String, Vec<Ordered>>,
    scripts: BTreeMap<String, Vec<(String, Value)>>,
    file: SourceFile,
}

pub(crate) fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub(crate) fn json_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    Ok(files)
}

impl Data {
    /// Reads the fixtures and the source file below the repository root `root`.
    pub fn load(root: &Path) -> Result<Data, String> {
        let file: SourceFile = read_json(&root.join(SOURCE_FILE))?;
        if file.format != "agenticspec-fixture-source/1" {
            return Err(format!(
                "{SOURCE_FILE}: format {} is not agenticspec-fixture-source/1",
                file.format
            ));
        }
        let mut views = BTreeMap::new();
        for p in json_files(&root.join(FIXTURES))? {
            let v: ViewFile = read_json(&p)?;
            views.insert(v.view, v.rows);
        }
        let mut scripts = BTreeMap::new();
        for p in json_files(&root.join(FIXTURES).join("scripts"))? {
            let s: ScriptFile = read_json(&p)?;
            scripts.insert(
                s.channel,
                s.events.into_iter().map(|e| (e.event, e.payload)).collect(),
            );
        }
        for (view, rows) in &file.views {
            views.insert(view.clone(), rows.clone());
        }
        Ok(Data {
            views,
            scripts,
            file,
        })
    }

    /// The architecture the source holds.
    pub fn architecture(&self) -> &str {
        &self.file.architecture_id
    }
}

/// One obligation the source breaks on purpose.
#[derive(Debug, Clone, Default)]
pub struct Faults {
    /// R1 of this view adds one row that is out of scope.
    pub leak_row: Option<String>,
    /// R1 of this view leaves out one row that is in scope.
    pub drop_row: Option<String>,
    /// A reconnect gets the event of its cursor again.
    pub resend_on_resume: bool,
    /// R5 allows every command, whatever the grants.
    pub allow_everything: bool,
    /// A request without credentials is answered as the first principal.
    pub open: bool,
}

/// A scope of a request: `{architecture}[/{part}[/{instance}]]`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Scope {
    architecture: String,
    part: Option<String>,
    instance: Option<u32>,
}

impl Scope {
    fn path(&self) -> String {
        interface::scope_path(&self.architecture, self.part.as_deref(), self.instance)
    }
}

/// A running fixture source. Dropping it stops it.
pub struct FixtureSource {
    url: String,
    dispatched: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
}

impl FixtureSource {
    /// Starts the source on 127.0.0.1 and a free port.
    pub fn start(data: Data, faults: Faults) -> io::Result<FixtureSource> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let url = format!("http://{}", listener.local_addr()?);
        let dispatched = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let shared = Arc::new(Shared {
            data,
            faults,
            dispatched: Arc::clone(&dispatched),
            stop: Arc::clone(&stop),
        });
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if shared.stop.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(conn) = conn else { continue };
                let shared = Arc::clone(&shared);
                std::thread::spawn(move || shared.connection(conn));
            }
        });
        Ok(FixtureSource {
            url,
            dispatched,
            stop,
        })
    }

    /// The root of the source, `http://127.0.0.1:<port>`; scopes are below `/scopes/`.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// How many requests passed authentication and were dispatched.
    pub fn dispatched(&self) -> usize {
        self.dispatched.load(Ordering::SeqCst)
    }
}

impl Drop for FixtureSource {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(addr) = self.url.strip_prefix("http://") {
            let _ = TcpStream::connect(addr);
        }
    }
}

struct Shared {
    data: Data,
    faults: Faults,
    dispatched: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
}

fn error(status: u16, name: &str, message: impl Into<String>) -> (u16, String) {
    (
        status,
        json!({"error": name, "message": message.into()}).to_string(),
    )
}

impl Shared {
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

    /// The principal of the request's credentials: a bearer token or the session cookie.
    fn principal(&self, r: &Request) -> Option<String> {
        let token = r
            .header("authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .map(String::from)
            .or_else(|| {
                let name = interface::session_cookie();
                r.header("cookie").and_then(|c| {
                    c.split(';').find_map(|kv| {
                        let (k, v) = kv.trim().split_once('=')?;
                        (k == name).then(|| v.to_string())
                    })
                })
            });
        let found = token.and_then(|t| {
            self.data
                .file
                .principals
                .iter()
                .find(|p| p.token == t)
                .map(|p| p.principal_id.clone())
        });
        if found.is_none() && self.faults.open {
            return self
                .data
                .file
                .principals
                .first()
                .map(|p| p.principal_id.clone());
        }
        found
    }

    fn answer(&self, r: &Request, out: &mut TcpStream) -> io::Result<()> {
        let Some(principal) = self.principal(r) else {
            let (s, b) = error(
                401,
                "Unauthenticated",
                "send Authorization: Bearer <token> or the session cookie",
            );
            return wire::write_json(out, s, &[("WWW-Authenticate", "Bearer")], &b);
        };
        self.dispatched.fetch_add(1, Ordering::SeqCst);
        let (scope, rest) = match self.scope_of(&r.path) {
            Ok(x) => x,
            Err((s, b)) => return wire::write_json(out, s, &[], &b),
        };
        let mut words = rest.splitn(2, '/');
        let (kind, name) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
        let (status, body) = match (r.method.as_str(), kind) {
            ("GET", "views") if !name.is_empty() => self.read(&scope, name),
            ("GET", "channels") if !name.is_empty() => return self.stream(r, &scope, name, out),
            ("POST", "commands") if !name.is_empty() => self.command(r, &scope, name, &principal),
            ("GET", "actions") if name.is_empty() => self.actions(&scope, &principal),
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

    /// The scope of a path `/scopes/{architecture}[/{part}[/{instance}]]/api/...` and the rest
    /// after `/api/`.
    fn scope_of<'a>(&self, path: &'a str) -> Result<(Scope, &'a str), (u16, String)> {
        let prefix = interface::scope_prefix();
        let no_scope = || {
            error(
                404,
                "NoSuchScope",
                format!("{path}: the source answers below {prefix}<architecture_id>"),
            )
        };
        let below = path.strip_prefix(&prefix).ok_or_else(no_scope)?;
        let (scope, rest) = below.split_once("/api/").ok_or_else(no_scope)?;
        let segs: Vec<&str> = scope.split('/').collect();
        let instance = match segs.get(2) {
            Some(s) => Some(s.parse::<u32>().map_err(|_| no_scope())?),
            None => None,
        };
        if segs.len() > 3 || segs[0].is_empty() {
            return Err(no_scope());
        }
        let scope = Scope {
            architecture: segs[0].to_string(),
            part: segs.get(1).map(|s| s.to_string()),
            instance,
        };
        if scope.architecture != self.data.file.architecture_id {
            return Err(error(
                404,
                "NoSuchScope",
                format!("no architecture {}", scope.architecture),
            ));
        }
        if let Some(part) = &scope.part {
            let known = self
                .data
                .views
                .get("agentic.architecture.Parts")
                .into_iter()
                .flatten()
                .any(|p| {
                    p.str("name") == Some(part)
                        && p.str("architecture_id") == Some(&scope.architecture)
                });
            if !known {
                return Err(error(
                    404,
                    "NoSuchScope",
                    format!("no part {part} in {}", scope.architecture),
                ));
            }
        }
        Ok((scope, rest))
    }

    /// The placements of the scope.
    fn placements(&self, scope: &Scope) -> Vec<&Ordered> {
        self.data
            .file
            .placements
            .iter()
            .filter(|p| {
                p.str("architecture_id") == Some(&scope.architecture)
                    && scope
                        .part
                        .as_ref()
                        .is_none_or(|part| p.str("part_path") == Some(part))
                    && scope.instance.is_none_or(|i| {
                        p.get("instance").and_then(Value::as_u64) == Some(u64::from(i))
                    })
            })
            .collect()
    }

    fn placed(&self, scope: &Scope) -> BTreeSet<String> {
        self.placements(scope)
            .iter()
            .filter_map(|p| p.str("entity_id").map(String::from))
            .collect()
    }

    fn rows_in_scope(&self, scope: &Scope, view: &str) -> Option<Vec<Ordered>> {
        if view == "agentic.architecture.Placements" {
            return Some(self.placements(scope).into_iter().cloned().collect());
        }
        let rows = self.data.views.get(view)?;
        if interface::r3_views().iter().any(|v| v == view) {
            return Some(
                rows.iter()
                    .filter(|r| r.str("architecture_id") == Some(&scope.architecture))
                    .filter(|r| {
                        view != "agentic.architecture.Parts"
                            || scope.part.as_ref().is_none_or(|p| r.str("name") == Some(p))
                    })
                    .cloned()
                    .collect(),
            );
        }
        let placed = self.placed(scope);
        Some(
            rows.iter()
                .filter(|r| r.in_scope_at(&placed, &scope.architecture, &scope.path()))
                .cloned()
                .collect(),
        )
    }

    fn read(&self, scope: &Scope, view: &str) -> (u16, String) {
        let Some(mut rows) = self.rows_in_scope(scope, view) else {
            return error(404, "NoSuchView", format!("no fixture answers {view}"));
        };
        if self.faults.drop_row.as_deref() == Some(view) && !rows.is_empty() {
            rows.remove(0);
        }
        if self.faults.leak_row.as_deref() == Some(view) {
            let placed = self.placed(scope);
            if let Some(out) = self
                .data
                .views
                .get(view)
                .into_iter()
                .flatten()
                .find(|r| !r.in_scope_at(&placed, &scope.architecture, &scope.path()))
            {
                rows.push(out.clone());
            }
        }
        let rows = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into());
        (200, format!("{{\"rows\":{rows}}}"))
    }

    fn stream(
        &self,
        r: &Request,
        scope: &Scope,
        channel: &str,
        out: &mut TcpStream,
    ) -> io::Result<()> {
        let Some(script) = self.data.scripts.get(channel) else {
            let (s, b) = error(404, "NoSuchChannel", format!("no script plays {channel}"));
            return wire::write_json(out, s, &[], &b);
        };
        let placed = self.placed(scope);
        let after = r
            .header(&interface::cursor_header())
            .and_then(|v| v.trim().parse::<usize>().ok());
        out.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        )?;
        for (i, (event, payload)) in script.iter().enumerate() {
            let cursor = i + 1;
            let skip = match after {
                Some(c) if self.faults.resend_on_resume => cursor < c,
                Some(c) => cursor <= c,
                None => false,
            };
            let Some(obj) = payload.as_object() else {
                continue;
            };
            let outside_params = r
                .query
                .iter()
                .any(|(k, v)| obj.get(k).and_then(Value::as_str).is_some_and(|x| x != v));
            if skip
                || outside_params
                || !interface::in_scope_at(obj, &placed, &scope.architecture, &scope.path())
            {
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

    fn command(&self, r: &Request, scope: &Scope, command: &str, principal: &str) -> (u16, String) {
        let input: Map<String, Value> = match serde_json::from_slice(&r.body) {
            Ok(Value::Object(m)) => m,
            _ => return error(400, "InvalidRequest", "a command's input is a JSON object"),
        };
        let file = &self.data.file;
        let known = file.action_classes.contains_key(command)
            || file.commands.iter().any(|c| c.command == command);
        if !known {
            return error(404, "NoSuchCommand", format!("no command {command}"));
        }
        let path = scope.path();
        match evaluate(
            &file.grants,
            &file.action_classes,
            principal,
            command,
            &path,
        ) {
            Verdict::Allowed(_) => {}
            Verdict::Forbidden(grant) => {
                return (
                    422,
                    json!({"row": {"outcome": "forbidden", "error": "agentic.authority.GrantForbids", "grant_id": grant}}).to_string(),
                );
            }
            Verdict::NotCovered(class) => {
                return (
                    422,
                    json!({"row": {"outcome": "not-covered", "error": "agentic.authority.NoGrantCovers", "action_class": class, "scope": path}}).to_string(),
                );
            }
        }
        let found = file
            .commands
            .iter()
            .find(|c| c.command == command && c.input.iter().all(|(k, v)| input.get(k) == Some(v)));
        match found {
            Some(CommandFixture {
                accepted: Some(row),
                ..
            }) => (200, json!({ "row": row }).to_string()),
            Some(CommandFixture {
                refused: Some(row), ..
            }) => (422, json!({ "row": row }).to_string()),
            _ => error(
                501,
                "NoFixture",
                format!("no fixture answers {command} with this input"),
            ),
        }
    }

    fn actions(&self, scope: &Scope, principal: &str) -> (u16, String) {
        let file = &self.data.file;
        let path = scope.path();
        let list: Vec<Value> = file
            .action_classes
            .keys()
            .map(|cmd| {
                if self.faults.allow_everything {
                    return json!({"command": cmd, "allowed": true});
                }
                match evaluate(&file.grants, &file.action_classes, principal, cmd, &path) {
                    Verdict::Allowed(g) => json!({"command": cmd, "allowed": true, "grant_id": g}),
                    _ => json!({"command": cmd, "allowed": false}),
                }
            })
            .collect();
        (200, Value::Array(list).to_string())
    }
}
