//! `agenticspec ui serve` on the command line: the acceptance of the story against the one
//! server this run starts, and `agenticspec ui conform --architecture` against it. What each
//! check judges, and a server with faults, are tested in the `agenticspec-ui` crate.

use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::repo;

const TOKEN: &str = "ui-serve-test";
const ARCH: &str = "architectures/workstation.yaml";

/// A running `agenticspec ui serve`; dropping it kills it.
struct Server {
    child: Child,
    url: String,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Server {
    /// `ui serve` of the workstation on a port the system picks, the contract derived.
    fn start() -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agenticspec"))
            .current_dir(repo())
            .args(["ui", "serve", ARCH, "--port", "0"])
            .env("AGENTICSPEC_UI_TOKEN", TOKEN)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stderr = child.stderr.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let _ = tx.send(line);
            }
        });
        let mut seen = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(30);
        while let Ok(line) = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            if let Some(rest) = line.split("serving ").nth(1) {
                let url = rest
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .trim_end_matches('/');
                return Server {
                    child,
                    url: url.to_string(),
                };
            }
            seen.push(line);
        }
        let _ = child.kill();
        let _ = child.wait();
        panic!("ui serve did not start: {seen:?}");
    }

    fn interface(&self) -> String {
        format!("{}/ingress/operator", self.url)
    }
}

fn send(method: &str, url: &str, headers: &[(&str, &str)], body: Option<&str>) -> (u16, Value) {
    let rest = url.strip_prefix("http://").unwrap();
    let (addr, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let addr: SocketAddr = addr.parse().unwrap();
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(5)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    for (k, v) in headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    let body = body.unwrap_or("");
    head.push_str(&format!("Content-Length: {}\r\n\r\n{body}", body.len()));
    std::io::Write::write_all(&mut s, head.as_bytes()).unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(&mut s, &mut text).unwrap();
    let status = text.split_whitespace().nth(1).unwrap().parse().unwrap();
    let json = text
        .split_once("\r\n\r\n")
        .and_then(|(_, b)| serde_json::from_str(b).ok())
        .unwrap_or(Value::Null);
    (status, json)
}

fn authed(method: &str, url: &str, body: Option<&str>) -> (u16, Value) {
    let auth = format!("Bearer {TOKEN}");
    send(
        method,
        url,
        &[
            ("Authorization", &auth),
            ("Content-Type", "application/json"),
        ],
        body,
    )
}

/// The events of a stream: `(id, event, data)` of the first `n`, read within 5 s.
fn events(url: &str, cursor: Option<&str>, n: usize) -> Vec<(String, String, Value)> {
    let rest = url.strip_prefix("http://").unwrap();
    let (addr, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut head = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {TOKEN}\r\nAccept: text/event-stream\r\n"
    );
    if let Some(c) = cursor {
        head.push_str(&format!("Last-Event-ID: {c}\r\n"));
    }
    head.push_str("\r\n");
    std::io::Write::write_all(&mut s, head.as_bytes()).unwrap();
    let mut out = Vec::new();
    let (mut id, mut event) = (String::new(), String::new());
    for line in BufReader::new(s).lines().map_while(Result::ok) {
        if let Some(v) = line.strip_prefix("id: ") {
            id = v.to_string();
        } else if let Some(v) = line.strip_prefix("event: ") {
            event = v.to_string();
        } else if let Some(v) = line.strip_prefix("data: ") {
            out.push((id.clone(), event.clone(), serde_json::from_str(v).unwrap()));
            if out.len() == n {
                break;
            }
        }
    }
    out
}

fn read_json(rel: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(repo().join(rel)).unwrap()).unwrap()
}

/// The operations of the operator ingress of the workstation's contract, of one kind.
fn operator(kind: &str) -> Vec<String> {
    let text = std::fs::read_to_string(repo().join("ui/views/examples/workstation.contract.yaml"))
        .unwrap();
    let at = text.find("  - name: operator\n").unwrap();
    text[at..]
        .lines()
        .skip(1)
        .take_while(|l| !l.starts_with("  - name:"))
        .filter_map(|l| {
            let l = l
                .trim()
                .strip_prefix(&format!("- {{kind: {kind}, name: "))?;
            Some(l.trim_end_matches('}').to_string())
        })
        .collect()
}

/// The story's acceptance against one `ui serve` of the workstation: R1 below
/// `/ingress/operator/scopes/` answers the block fixture's rows and the server listens on
/// 127.0.0.1 only; a reconnect from a cursor gets the scripted events after it; `DrainGateway`
/// answers its scripted outcome and the record lists it once; R5 allows every command of the
/// interface. Then `ui conform --architecture` passes against the same server.
#[test]
fn ui_serve_answers_the_default_control_plane_on_fixtures_and_passes_ui_conform() {
    let server = Server::start();
    let iface = server.interface();
    assert!(
        server.url.starts_with("http://127.0.0.1:"),
        "{}",
        server.url
    );
    let port: u16 = server.url.rsplit(':').next().unwrap().parse().unwrap();
    // Another address of this host does not answer (a UDP connect sends nothing).
    let other = UdpSocket::bind("0.0.0.0:0")
        .and_then(|u| u.connect("192.0.2.1:9").map(|()| u))
        .and_then(|u| u.local_addr())
        .map(|a| a.ip())
        .ok()
        .filter(|ip| !ip.is_loopback() && !ip.is_unspecified());
    if let Some(ip) = other {
        let addr = SocketAddr::new(ip, port);
        assert!(
            TcpStream::connect_timeout(&addr, Duration::from_secs(1)).is_err(),
            "{addr} answered"
        );
    }

    // R1 under /ingress/operator/scopes/: the fixture's rows.
    let (status, body) = authed(
        "GET",
        &format!("{iface}/scopes/workstation/gateway/api/views/agentic.gateway.Gateways"),
        None,
    );
    assert_eq!(status, 200, "{body}");
    let fixture = read_json("ui/views/fixtures/agentic.gateway.Gateways.json");
    assert_eq!(body["rows"], fixture["rows"]);
    let (status, _) = send(
        "GET",
        &format!("{iface}/scopes/workstation/gateway/api/views/agentic.gateway.Gateways"),
        &[],
        None,
    );
    assert_eq!(status, 401);

    // R2: the level's channel plays the scripts, each event the ingress carries; a reconnect
    // from the cursor of event 3 gets events 4 and 5.
    let feeds = operator("feed");
    let mut scripts: Vec<_> = std::fs::read_dir(repo().join("ui/views/fixtures/scripts"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    scripts.sort();
    let scripted: Vec<(String, Value)> = scripts
        .iter()
        .flat_map(|p| {
            let v: Value = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
            v["events"].as_array().unwrap().clone()
        })
        .filter(|e| feeds.iter().any(|f| f == e["event"].as_str().unwrap()))
        .map(|e| {
            (
                e["event"].as_str().unwrap().to_string(),
                e["payload"].clone(),
            )
        })
        .take(5)
        .collect();
    let url = format!("{iface}/scopes/workstation/api/channels/level");
    let first = events(&url, None, 5);
    let got: Vec<(String, Value)> = first
        .iter()
        .map(|(_, e, d)| (e.clone(), d.clone()))
        .collect();
    assert_eq!(got, scripted);
    let again = events(&url, Some(&first[2].0), 2);
    assert_eq!(again, first[3..5].to_vec());

    // R4: DrainGateway answers the scripted outcome, and the record lists it once.
    let (status, body) = authed(
        "POST",
        &format!("{iface}/scopes/workstation/gateway/api/commands/agentic.gateway.DrainGateway"),
        Some(r#"{"gateway_id": "00000000-0000-4000-8000-9121f2990001"}"#),
    );
    assert_eq!(status, 200, "{body}");
    let outcomes = read_json("ui/views/source/outcomes.json");
    let scripted = outcomes["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["command"] == "agentic.gateway.DrainGateway" && o.get("accepted").is_some())
        .unwrap();
    assert_eq!(body, json!({"row": scripted["accepted"]}));
    let (status, record) = authed("GET", &format!("{}/fixture/commands", server.url), None);
    assert_eq!(status, 200);
    let drains: Vec<&Value> = record
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["command"] == "agentic.gateway.DrainGateway")
        .collect();
    assert_eq!(drains.len(), 1, "{record}");
    assert_eq!(drains[0]["scope"], "workstation/gateway");

    // R5: every command of the interface is allowed.
    let (status, actions) = authed(
        "GET",
        &format!("{iface}/scopes/workstation/api/actions"),
        None,
    );
    assert_eq!(status, 200);
    let commands = operator("command");
    assert!(commands.contains(&"agentic.gateway.DrainGateway".to_string()));
    for c in &commands {
        let a = actions
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["command"] == c.as_str())
            .unwrap_or_else(|| panic!("R5 does not list {c}: {actions}"));
        assert_eq!(a["allowed"], true, "{a}");
    }

    // The conformance mode passes against the same server.
    let out = Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .current_dir(repo())
        .args([
            "--output",
            "json",
            "ui",
            "conform",
            "--architecture",
            ARCH,
            "--source",
        ])
        .arg(&iface)
        .env("AGENTICSPEC_UI_TOKEN", TOKEN)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice::<Value>(&out.stdout).unwrap()["result"].clone();
    assert_eq!(v["failed"], 0, "{v}");
    assert!(v["passed"].as_u64().unwrap() > 50, "{v}");
}

/// Against an address that answers nothing, the conformance mode exits non-zero and names each
/// check that failed.
#[test]
fn ui_conform_architecture_refuses_a_source_that_does_not_answer() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let out = Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .current_dir(repo())
        .args(["ui", "conform", "--architecture", ARCH, "--source"])
        .arg(format!("http://127.0.0.1:{port}/ingress/operator"))
        .env("AGENTICSPEC_UI_TOKEN", TOKEN)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("conformance_failed"), "{text}");
    assert!(
        text.contains("R1 agentic.gateway.Gateways at workstation/gateway"),
        "{text}"
    );
    assert!(text.contains("R2 resume level at workstation"), "{text}");
}
