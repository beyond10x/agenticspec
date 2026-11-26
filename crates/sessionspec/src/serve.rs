//! `sessionspec serve`: the `session-view` surface answered from the local store, the UI built
//! from `ui/sessionspec.ui.yaml`, and the live channels as server-sent events. Listens on
//! 127.0.0.1 only. Every request is answered as the one local `sessionspec.view.Viewer`.
//!
//! - `GET /view/views/<view>`: the generated `session-view` route table (`dispatch`), its
//!   queries implemented here over the store.
//! - `GET /view/channels/sessions` and `GET /view/channels/session_events?session_id=<id>`:
//!   `text/event-stream`, one SSE message per published event (`event:` the qualified event
//!   name, `data:` its wire JSON, `id:` the event's `seq` on `session_events`). A reconnect with
//!   `Last-Event-ID` gets the events after it first.
//! - Any other path: a file of the built UI, else its `index.html`.

use std::collections::HashMap;
use std::io::{self, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use session_ir::SessionIr;
use session_view::SessionView;
use sessionspec_server::{http, session_view as route, wire};
use sessionspec_system::System;
use sessionspec_types::actor::{Actor, Caller};
use sessionspec_types::behaviour::Generated;
use sessionspec_types::obligation::UnmetObligation;
use sessionspec_types::primitives::Timestamp;
use sessionspec_types::session as s;
use sessionspec_types::view as v;
use sessionspec_types::view::obligations as o;

use crate::store::Store;
use crate::time::epoch_ms;

/// How often the server syncs the window.
pub const SYNC_EVERY: Duration = Duration::from_secs(5);
/// The default port.
pub const PORT: u16 = 7420;

/// The `session-view` queries over the store.
pub struct StoreView {
    store: Arc<Store>,
}

impl StoreView {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }
}

fn unmet(_: rusqlite::Error) -> UnmetObligation {
    UnmetObligation {
        capability: "store",
        source: "the local store could not be read",
    }
}

/// An instant written as `Timestamp`, or a date, in milliseconds.
fn instant(t: &str) -> Option<i64> {
    epoch_ms(t).or_else(|| epoch_ms(&format!("{t}T00:00:00Z")))
}

macro_rules! event_row {
    ($ty:ident, $e:expr) => {{
        let e = $e;
        v::$ty {
            event_id: e.event_id,
            session_id: e.session_id,
            seq: e.seq,
            at: e.at,
            agent: e.agent,
            kind: e.kind,
            label: e.label,
            detail: e.detail,
            call: e.call,
            ok: e.ok,
            context_tokens: e.context_tokens,
            output_tokens: e.output_tokens,
            cache_read_tokens: e.cache_read_tokens,
            cache_write_tokens: e.cache_write_tokens,
            cost_microdollars: e.cost_microdollars,
            bytes: e.bytes,
            agent_cost_usd: e.agent_cost_usd,
            session_cost_usd: e.session_cost_usd,
        }
    }};
}

impl o::SessionListQuery for StoreView {
    fn session_list(
        &self,
        since: Option<Timestamp>,
        search: Option<String>,
        limit: Option<i64>,
    ) -> Result<Vec<v::SessionList>, UnmetObligation> {
        let since = since.and_then(|t| instant(&t.0));
        // `names` is lower case; the search is lowered here (spec/domains/view.yaml, UNMAPPED).
        let search = search.map(|t| t.to_lowercase()).filter(|t| !t.is_empty());
        Ok(self
            .store
            .sessions()
            .map_err(unmet)?
            .into_iter()
            .filter(|r| since.is_none_or(|t| instant(&r.last_at.0).is_some_and(|l| l >= t)))
            .filter(|r| search.as_deref().is_none_or(|q| r.names.contains(q)))
            .filter(|r| limit.is_none_or(|n| r.rank <= n))
            .map(|r| v::SessionList {
                session_id: r.session_id,
                project: r.project,
                title: r.title,
                names: r.names,
                rank: r.rank,
                harness: r.harness,
                first_at: r.first_at,
                last_at: r.last_at,
                agents: r.agents,
                cost_usd: r.cost_usd,
                output_tokens: r.output_tokens,
                context_max: r.context_max,
                cache_hit_share: r.cache_hit_share,
                busts: r.busts,
                human_prompts: r.human_prompts,
                tool_calls: r.tool_calls,
                refused_tool_calls: r.refused_tool_calls,
                exposures: r.exposures,
                working_seconds: r.working_seconds,
                waiting_on_human_seconds: r.waiting_on_human_seconds,
                waiting_share: r.waiting_share,
            })
            .collect())
    }
}

impl o::SessionByIdQuery for StoreView {
    fn session_by_id(
        &self,
        session_id: s::SessionId,
    ) -> Result<Vec<v::SessionById>, UnmetObligation> {
        let Some(r) = self
            .store
            .sessions()
            .map_err(unmet)?
            .into_iter()
            .find(|r| r.session_id == session_id)
        else {
            return Ok(Vec::new());
        };
        let Some(kpis) = self.store.session_kpis(&session_id.0).map_err(unmet)? else {
            return Ok(Vec::new());
        };
        Ok(vec![v::SessionById {
            session_id: r.session_id,
            project: r.project,
            title: r.title,
            harness: r.harness,
            first_at: r.first_at,
            last_at: r.last_at,
            agents: r.agents,
            cost_usd: r.cost_usd,
            output_tokens: r.output_tokens,
            context_max: r.context_max,
            cache_hit_share: r.cache_hit_share,
            busts: r.busts,
            human_prompts: r.human_prompts,
            tool_calls: r.tool_calls,
            refused_tool_calls: r.refused_tool_calls,
            exposures: r.exposures,
            working_seconds: r.working_seconds,
            waiting_on_human_seconds: r.waiting_on_human_seconds,
            waiting_share: r.waiting_share,
            kpis,
        }])
    }
}

impl o::SessionAgentsQuery for StoreView {
    fn session_agents(
        &self,
        session_id: s::SessionId,
    ) -> Result<Vec<v::SessionAgents>, UnmetObligation> {
        Ok(self
            .store
            .agents(&session_id.0)
            .map_err(unmet)?
            .into_iter()
            .map(|a| v::SessionAgents {
                agent_id: a.agent_id,
                session_id: a.session_id,
                parent: a.parent,
                role: a.role,
                model: a.model,
                started_at: a.started_at,
                turns: a.turns,
                context_tokens: a.context_tokens,
                context_max: a.context_max,
                output_tokens: a.output_tokens,
                cost_usd: a.cost_usd,
                tool_calls: a.tool_calls,
                busts: a.busts,
            })
            .collect())
    }
}

impl o::SessionEventsQuery for StoreView {
    fn session_events(
        &self,
        session_id: s::SessionId,
    ) -> Result<Vec<v::SessionEvents>, UnmetObligation> {
        Ok(self
            .store
            .events(&session_id.0, 0, None, None)
            .map_err(unmet)?
            .into_iter()
            .map(|e| event_row!(SessionEvents, e))
            .collect())
    }
}

impl o::SessionTurnsQuery for StoreView {
    fn session_turns(
        &self,
        session_id: s::SessionId,
    ) -> Result<Vec<v::SessionTurns>, UnmetObligation> {
        Ok(self
            .store
            .events(&session_id.0, 0, Some(v::EventKind::ModelTurn), None)
            .map_err(unmet)?
            .into_iter()
            .map(|e| event_row!(SessionTurns, e))
            .collect())
    }
}

impl o::AgentTurnsQuery for StoreView {
    fn agent_turns(
        &self,
        session_id: s::SessionId,
        agent: s::AgentId,
    ) -> Result<Vec<v::AgentTurns>, UnmetObligation> {
        Ok(self
            .store
            .events(
                &session_id.0,
                0,
                Some(v::EventKind::ModelTurn),
                Some(&agent.0),
            )
            .map_err(unmet)?
            .into_iter()
            .map(|e| event_row!(AgentTurns, e))
            .collect())
    }
}

impl o::SessionChecksQuery for StoreView {
    fn session_checks(
        &self,
        session_id: s::SessionId,
    ) -> Result<Vec<v::SessionChecks>, UnmetObligation> {
        self.store.checks(&session_id.0).map_err(unmet)
    }
}

type Sys = System<Generated<crate::replay::Ports>, StoreView>;

/// The system the server dispatches into: an idle `session-ir` and `session-view` over the store.
pub fn system(store: Arc<Store>) -> Sys {
    System::new(
        SessionIr::new(Generated::new(crate::replay::Ports::default())),
        SessionView::new(StoreView::new(store)),
    )
}

/// Answers one request of the `session-view` route table as the local viewer.
pub fn answer(system: &mut Sys, request: &http::Request) -> http::Response {
    let viewer = Caller {
        actor: Actor::Viewer,
    };
    route::dispatch(system, Some(&viewer), request)
}

/// One SSE message.
fn message(event: &str, id: Option<i64>, data: &str) -> String {
    let mut out = format!("event: {event}\n");
    if let Some(id) = id {
        out.push_str(&format!("id: {id}\n"));
    }
    out.push_str("data: ");
    out.push_str(data);
    out.push_str("\n\n");
    out
}

#[derive(Clone, PartialEq, Eq)]
enum Channel {
    Sessions,
    SessionEvents(String),
}

/// Subscribers of the live channels.
#[derive(Default)]
pub struct Hub {
    subs: Mutex<Vec<(Channel, Sender<String>)>>,
}

impl Hub {
    fn publish(&self, channel: &Channel, text: &str) {
        let mut subs = self.subs.lock().unwrap_or_else(|e| e.into_inner());
        subs.retain(|(c, tx)| c != channel || tx.send(text.to_owned()).is_ok());
    }

    fn subscribe(&self, channel: Channel) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel();
        self.subs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push((channel, tx));
        rx
    }
}

/// What the server last published per session: its row, its agents' rows, its last event.
#[derive(Default)]
pub struct Seen {
    sessions: HashMap<String, String>,
    agents: HashMap<(String, String), String>,
    last_seq: HashMap<String, i64>,
}

impl Seen {
    /// Takes what the store holds now as published.
    pub fn of(store: &Store) -> Self {
        let mut seen = Seen::default();
        for row in store.sessions().unwrap_or_default() {
            let id = row.session_id.0.clone();
            let mut text = String::new();
            wire::encode_event_sessionspec_view_session_updated(&row, &mut text);
            seen.sessions.insert(id.clone(), text);
            for a in store.agents(&id).unwrap_or_default() {
                let mut text = String::new();
                wire::encode_event_sessionspec_view_agent_updated(&a, &mut text);
                seen.agents.insert((id.clone(), a.agent_id.0.clone()), text);
            }
        }
        for (id, n) in store.event_counts().unwrap_or_default() {
            seen.last_seq.insert(id, n - 1);
        }
        seen
    }
}

/// The messages that bring a subscriber from `seen` to what the store holds, for the sessions in
/// `only` (every session when `None`), and `seen` moved forward.
fn diff(store: &Store, seen: &mut Seen, only: Option<&[String]>) -> Vec<(Channel, String)> {
    let mut out = Vec::new();
    let Ok(rows) = store.sessions() else {
        return out;
    };
    for row in rows {
        let id = row.session_id.0.clone();
        if only.is_some_and(|o| !o.contains(&id)) {
            continue;
        }
        let mut text = String::new();
        wire::encode_event_sessionspec_view_session_updated(&row, &mut text);
        if seen.sessions.get(&id) != Some(&text) {
            let m = message("sessionspec.view.SessionUpdated", None, &text);
            out.push((Channel::Sessions, m.clone()));
            out.push((Channel::SessionEvents(id.clone()), m));
            seen.sessions.insert(id.clone(), text);
        }
        for a in store.agents(&id).unwrap_or_default() {
            let mut text = String::new();
            wire::encode_event_sessionspec_view_agent_updated(&a, &mut text);
            let key = (id.clone(), a.agent_id.0.clone());
            if seen.agents.get(&key) != Some(&text) {
                out.push((
                    Channel::SessionEvents(id.clone()),
                    message("sessionspec.view.AgentUpdated", None, &text),
                ));
                seen.agents.insert(key, text);
            }
        }
        let from = seen.last_seq.get(&id).map_or(0, |s| s + 1);
        for e in store.events(&id, from, None, None).unwrap_or_default() {
            let mut text = String::new();
            let seq = e.seq;
            wire::encode_event_sessionspec_view_event_appended(&e, &mut text);
            out.push((
                Channel::SessionEvents(id.clone()),
                message("sessionspec.view.EventAppended", Some(seq), &text),
            ));
            seen.last_seq.insert(id.clone(), seq);
        }
    }
    out
}

/// The built UI: `$SESSIONSPEC_UI_DIR`, else `target/ui/www` beside the binary's `target/`, else
/// `<state-dir>/ui`.
pub fn ui_dir(state_dir: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(d) = std::env::var("SESSIONSPEC_UI_DIR") {
        candidates.push(PathBuf::from(d));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(target) = exe.parent().and_then(Path::parent)
    {
        candidates.push(target.join("ui/www"));
    }
    candidates.push(state_dir.join("ui"));
    candidates
        .into_iter()
        .find(|d| d.join("index.html").is_file())
}

fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|x| x.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn static_file(stream: &mut TcpStream, ui: Option<&Path>, path: &str) -> io::Result<()> {
    let Some(root) = ui else {
        let body = "The UI is not built: run `task ui:build` in the sessionspec repository.\n";
        return write_raw(stream, 404, "text/plain; charset=utf-8", body.as_bytes());
    };
    let rel = Path::new(path.trim_start_matches('/'));
    let safe = rel.components().all(|c| matches!(c, Component::Normal(_)));
    let file = root.join(rel);
    let file = if safe && file.is_file() {
        file
    } else {
        root.join("index.html")
    };
    let body = std::fs::read(&file)?;
    write_raw(stream, 200, content_type(&file), &body)
}

fn write_raw(stream: &mut TcpStream, status: u16, ctype: &str, body: &[u8]) -> io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        http::reason(status),
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then(|| percent_decode(v))
    })
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(x) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(x);
            i += 3;
            continue;
        }
        out.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn events_stream(
    mut stream: TcpStream,
    hub: &Hub,
    store: &Store,
    request: &http::Request,
) -> io::Result<()> {
    let channel = match request.path.as_str() {
        "/view/channels/sessions" => Channel::Sessions,
        "/view/channels/session_events" => match query_param(&request.query, "session_id") {
            Some(id) => Channel::SessionEvents(id),
            None => {
                return write_raw(
                    &mut stream,
                    400,
                    "text/plain; charset=utf-8",
                    b"session_events takes ?session_id=<id>\n",
                );
            }
        },
        _ => {
            return write_raw(
                &mut stream,
                404,
                "text/plain; charset=utf-8",
                b"no such channel\n",
            );
        }
    };
    let rx = hub.subscribe(channel.clone());
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    stream.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
    )?;
    stream.flush()?;
    // A reconnect resumes after the last event the client saw.
    if let Channel::SessionEvents(id) = &channel
        && let Some(last) = request
            .headers
            .iter()
            .find(|(k, _)| k == "last-event-id")
            .and_then(|(_, v)| v.parse::<i64>().ok())
    {
        for e in store.events(id, last + 1, None, None).unwrap_or_default() {
            let mut text = String::new();
            let seq = e.seq;
            wire::encode_event_sessionspec_view_event_appended(&e, &mut text);
            stream.write_all(
                message("sessionspec.view.EventAppended", Some(seq), &text).as_bytes(),
            )?;
        }
        stream.flush()?;
    }
    loop {
        match rx.recv_timeout(Duration::from_secs(15)) {
            Ok(text) => stream.write_all(text.as_bytes())?,
            Err(RecvTimeoutError::Timeout) => stream.write_all(b": keep-alive\n\n")?,
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
        stream.flush()?;
    }
}

/// Everything one connection needs.
pub struct Shared {
    pub system: Mutex<Sys>,
    pub hub: Hub,
    pub store: Arc<Store>,
    pub ui: Option<PathBuf>,
}

/// Answers one connection.
pub fn connection(shared: &Shared, stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(http::READ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(http::WRITE_TIMEOUT));
    let mut reader = BufReader::new(stream);
    let request = match http::read(&mut reader) {
        Ok(r) => r,
        Err(refusal) => {
            let mut stream = reader.into_inner();
            let _ = http::write(&mut stream, &refusal);
            return;
        }
    };
    let mut stream = reader.into_inner();
    if request.path.starts_with("/view/channels/") {
        let _ = events_stream(stream, &shared.hub, &shared.store, &request);
    } else if request.path.starts_with("/view/views/")
        || request.path == "/openapi.json"
        || request.path == "/docs"
    {
        let answer = {
            let mut system = shared.system.lock().unwrap_or_else(|e| e.into_inner());
            answer(&mut system, &request)
        };
        let _ = http::write(&mut stream, &answer);
    } else {
        let _ = static_file(&mut stream, shared.ui.as_deref(), &request.path);
    }
}

/// Binds 127.0.0.1:`port`. Separate from [`run`] so the caller can refuse a taken port.
pub fn bind(port: u16) -> io::Result<TcpListener> {
    TcpListener::bind(("127.0.0.1", port))
}

/// Serves until the process ends: one thread per connection, and one that syncs `root` every
/// [`SYNC_EVERY`] and publishes what changed.
pub fn run(listener: TcpListener, shared: Arc<Shared>, root: PathBuf, since_hours: i64) {
    let syncer = Arc::clone(&shared);
    std::thread::spawn(move || {
        let mut seen = Seen::of(&syncer.store);
        loop {
            std::thread::sleep(SYNC_EVERY);
            let now = crate::time::now_ms();
            let Ok(done) = syncer.store.sync(&root, now - since_hours * 3_600_000) else {
                continue;
            };
            if done.changed.is_empty() {
                continue;
            }
            for (channel, text) in diff(&syncer.store, &mut seen, Some(&done.changed)) {
                syncer.hub.publish(&channel, &text);
            }
        }
    });
    for conn in listener.incoming() {
        let Ok(conn) = conn else {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        };
        let shared = Arc::clone(&shared);
        std::thread::spawn(move || connection(&shared, conn));
    }
}

/// Publishes what changed in the store for `changed` sessions; for tests and callers that sync
/// themselves.
pub fn publish_changes(shared: &Shared, seen: &mut Seen, changed: &[String]) -> usize {
    let messages = diff(&shared.store, seen, Some(changed));
    let n = messages.len();
    for (channel, text) in messages {
        shared.hub.publish(&channel, &text);
    }
    n
}
