//! A test source of the runtime interface that `agenticspec ui serve` is not: it does not serve
//! every view (the declared 404 `view_not_served`, or a plain 404), it accepts a write only
//! from its own origin with a proof header, and it answers each row at exactly the scopes the
//! test lists. Routes below `{url}/scopes/{scope}/api/`, like a runtime's adapter. One listener
//! on 127.0.0.1; dropping it stops it. Also used by the command line's tests
//! (`crates/agenticspec/tests/ui/`).

use std::collections::BTreeMap;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use agenticspec_ui::interface::{self, Ordered};
use agenticspec_ui::wire;
use serde_json::{Value, json};

/// What the source holds.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// The scopes answered are this one and those below it.
    pub architecture: String,
    pub token: String,
    /// Views answered with 404 `{"error": "view_not_served", "view": …}`.
    pub not_served: Vec<String>,
    /// Views answered with a plain 404 `NoSuchView`.
    pub missing: Vec<String>,
    /// The header a write must carry beside its own origin, `(name, value)`.
    pub proof: Option<(String, String)>,
    /// The commands R5 allows and R4 carries out.
    pub commands: Vec<String>,
    /// `(view, scope, row)`: the row is answered at exactly that scope (text). Any other view
    /// answers no rows. A row keeps the order of its fields, its key first.
    pub rows: Vec<(String, String, Ordered)>,
}

/// A running test source.
pub struct Partial {
    url: String,
    stop: Arc<AtomicBool>,
}

impl Partial {
    pub fn start(config: Config) -> Partial {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let shared = Arc::new((config, url.clone(), Arc::clone(&stop)));
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if shared.2.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(conn) = conn else { continue };
                let shared = Arc::clone(&shared);
                std::thread::spawn(move || answer(&shared.0, &shared.1, &shared.2, conn));
            }
        });
        Partial { url, stop }
    }

    /// `http://127.0.0.1:<port>`, the root of the interface.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Drop for Partial {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(addr) = self.url.strip_prefix("http://") {
            let _ = TcpStream::connect(addr);
        }
    }
}

fn answer(config: &Config, url: &str, stop: &AtomicBool, stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let Ok(mut out) = stream.try_clone() else {
        return;
    };
    let Ok(r) = wire::read_request(&mut BufReader::new(stream)) else {
        return;
    };
    let err = |status: u16, name: &str| (status, json!({"error": name}).to_string());
    let bearer = format!("Bearer {}", config.token);
    if r.header("authorization") != Some(bearer.as_str()) {
        let (s, b) = err(401, "Unauthenticated");
        let _ = wire::write_json(&mut out, s, &[], &b);
        return;
    }
    let Some((scope, rest)) = r
        .path
        .strip_prefix("/scopes/")
        .and_then(|p| p.split_once("/api/"))
    else {
        let (s, b) = err(404, "NoSuchPath");
        let _ = wire::write_json(&mut out, s, &[], &b);
        return;
    };
    if !interface::covers(&config.architecture, scope) {
        let (s, b) = err(404, "NoSuchScope");
        let _ = wire::write_json(&mut out, s, &[], &b);
        return;
    }
    let (kind, name) = rest.split_once('/').unwrap_or((rest, ""));
    let (status, body) = match (r.method.as_str(), kind) {
        ("GET", "views") if config.not_served.iter().any(|v| v == name) => (
            404,
            json!({"error": interface::view_not_served(), "view": name}).to_string(),
        ),
        ("GET", "views") if config.missing.iter().any(|v| v == name) => err(404, "NoSuchView"),
        ("GET", "views") => {
            let rows: Vec<&Ordered> = config
                .rows
                .iter()
                .filter(|(v, at, _)| v == name && at == scope)
                .map(|(_, _, row)| row)
                .collect();
            (
                200,
                format!("{{\"rows\":{}}}", serde_json::to_string(&rows).unwrap()),
            )
        }
        ("GET", "channels") => {
            let _ = stream_events(&r, stop, &mut out);
            return;
        }
        ("GET", "actions") => {
            let list: Vec<Value> = config
                .commands
                .iter()
                .map(|c| json!({"command": c, "allowed": true, "grant_id": "test-grant"}))
                .collect();
            (200, Value::Array(list).to_string())
        }
        ("POST", "commands") => {
            let origin = r.header("origin") == Some(url);
            let proof = config
                .proof
                .as_ref()
                .is_none_or(|(k, v)| r.header(k) == Some(v.as_str()));
            if !origin || !proof {
                err(403, "CrossOriginWrite")
            } else if config.commands.iter().any(|c| c == name) {
                (200, json!({"row": null}).to_string())
            } else {
                err(404, "NoSuchCommand")
            }
        }
        _ => err(404, "NoSuchPath"),
    };
    let _ = wire::write_json(&mut out, status, &[], &body);
}

/// Five events with cursors 1..5, the ones after `Last-Event-ID` on a reconnect, then
/// keep-alive comments until the client goes or the source stops.
fn stream_events(r: &wire::Request, stop: &AtomicBool, out: &mut TcpStream) -> std::io::Result<()> {
    let after = r
        .header("last-event-id")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0);
    out.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
    )?;
    let mut by_cursor = BTreeMap::new();
    for n in 1..=5u64 {
        by_cursor.insert(n, json!({"tick_id": format!("tick-{n}")}));
    }
    for (n, data) in by_cursor.range(after + 1..) {
        let id = n.to_string();
        out.write_all(
            wire::event_text("agentic.test.Ticked", Some(&id), &data.to_string()).as_bytes(),
        )?;
    }
    out.flush()?;
    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
        out.write_all(b": keep-alive\n\n")?;
        out.flush()?;
    }
    Ok(())
}
