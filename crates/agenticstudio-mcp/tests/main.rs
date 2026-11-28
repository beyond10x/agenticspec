//! Tests of agenticstudio-mcp. One binary (workspace rule). A scripted MCP client replays the
//! recorded exchange `tests/exchange.jsonl` against a studio in the same process (`Local`): no
//! model, no socket. A second client on the draft is a subscriber of its event channel.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use agenticstudio::live::{self, Event};
use agenticstudio::routes::lock;
use agenticstudio::studio::Studio;
use agenticstudio_mcp::{Backend, Http, Local, Server, Target};
use serde_json::{Value, json};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A studio with a draft of `architectures/workstation.yaml`, and the draft's id.
fn studio() -> (Local, String) {
    let local = Local(Arc::new(Mutex::new(Studio::new(root()))));
    let (status, v) = local
        .request(
            "POST",
            "/api/drafts",
            Some(&json!({"source": "architectures/workstation.yaml"})),
        )
        .unwrap();
    assert_eq!(status, 201, "{v}");
    let id = v["draft"]["draft_id"].as_str().unwrap().to_string();
    (local, id)
}

fn exchange() -> Vec<String> {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/exchange.jsonl"))
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

/// The line of the exchange with request id `id`.
fn line(id: u64) -> String {
    exchange()
        .into_iter()
        .find(|l| serde_json::from_str::<Value>(l).unwrap()["id"] == id)
        .unwrap_or_else(|| panic!("no request {id} in the exchange"))
}

fn revision(local: &Local, id: &str) -> Value {
    let (_, v) = local
        .request("GET", &format!("/api/drafts/{id}"), None)
        .unwrap();
    v["revision"]["number"].clone()
}

fn tool_calls(local: &Local, session: &str) -> Vec<Value> {
    let (status, v) = local
        .request(
            "GET",
            &format!("/api/views/studio.session.ToolCalls?session_id={session}"),
            None,
        )
        .unwrap();
    assert_eq!(status, 200, "{v}");
    v["rows"].as_array().unwrap().clone()
}

fn revised(rx: &Receiver<Event>, within: Duration) -> Option<Event> {
    let until = Instant::now() + within;
    loop {
        match rx.recv_timeout(until.saturating_duration_since(Instant::now())) {
            Ok(e) if e.name == live::DRAFT_REVISED => return Some(e),
            Ok(_) => continue,
            Err(_) => return None,
        }
    }
}

/// The tool result of a `tools/call` answer, checked to be one.
fn tool_result(answer: &Value) -> &Value {
    let r = &answer["result"];
    assert!(r["content"][0]["text"].is_string(), "{answer}");
    r
}

/// Every acceptance of the story, in the order of the recorded exchange.
#[test]
fn the_recorded_exchange_drives_the_draft_and_a_second_client_sees_it_live() {
    let (local, id) = studio();
    let second = lock(&local.0).subscribe(&id).unwrap();
    let mut server = Server::new(local.clone(), Target::Draft(id.clone()));

    let init = server.handle(&line(1)).unwrap();
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert!(
        init["result"]["capabilities"]["tools"].is_object(),
        "{init}"
    );
    assert_eq!(init["result"]["serverInfo"]["name"], "agenticstudio-mcp");
    assert!(
        server.handle(&exchange()[1]).is_none(),
        "a notification has no answer"
    );

    let list = server.handle(&line(2)).unwrap();
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["query", "apply", "check", "simulate"]);

    // query: the rows operator and worker and no other; one query call in the session.
    let q = server.handle(&line(3)).unwrap();
    let r = tool_result(&q);
    assert_eq!(r["isError"], false, "{q}");
    let rows: Vec<&str> = r["structuredContent"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["P"].as_str().unwrap())
        .collect();
    assert_eq!(rows, ["operator", "worker"]);
    assert!(
        r["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("2 rows."),
        "{q}"
    );
    let session = server.session().unwrap().to_string();
    let calls = tool_calls(&local, &session);
    assert_eq!(calls.len(), 1);
    assert_eq!(
        (&calls[0]["tool"], &calls[0]["caller"]),
        (&json!("Query"), &json!("Agent"))
    );

    // check: the result of agenticspec-sim check on revision 1, passing; still revision 1.
    let c = server.handle(&line(4)).unwrap();
    let doc = lock(&local.0)
        .draft(&id)
        .unwrap()
        .current()
        .document
        .clone();
    let expected = agenticspec_sim::check::check(&doc);
    assert!(expected.is_empty());
    assert_eq!(
        tool_result(&c)["structuredContent"]["check"],
        json!({"passed": true, "refusals": []})
    );
    assert_eq!(revision(&local, &id), 1);

    // apply, dry run: the diff and the check; still revision 1.
    let d = server.handle(&line(5)).unwrap();
    let a = &tool_result(&d)["structuredContent"];
    assert_eq!(a["outcome"], "DryRun", "{d}");
    assert!(!a["diff"].as_array().unwrap().is_empty(), "{a}");
    assert_eq!(a["check"]["passed"], true);
    assert_eq!(revision(&local, &id), 1);

    // apply, rule 2 an unknown verb: Refused at rule 2 with the error's name; still revision 1.
    let u = server.handle(&line(6)).unwrap();
    let a = &tool_result(&u)["structuredContent"];
    assert_eq!(a["outcome"], "Refused", "{u}");
    assert_eq!(
        (&a["refusal"]["rule"], &a["refusal"]["error"]),
        (&json!(2), &json!("UnknownVerb"))
    );
    assert!(
        tool_result(&u)["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("Refused at rule 2: UnknownVerb"),
        "{u}"
    );
    assert_eq!(revision(&local, &id), 1);
    assert!(
        revised(&second, Duration::from_millis(50)).is_none(),
        "no revision yet"
    );

    // apply: one answer with the diff and the check; revision 2; listed as an answered call
    // of the agent; the second client receives revision 2 without a reload.
    let before = tool_calls(&local, &session);
    assert!(
        !before
            .iter()
            .any(|c| c["dry_run"] == false && c["application"].is_string()),
        "no apply committed yet: {before:?}"
    );
    let ap = server.handle(&line(7)).unwrap();
    let a = &tool_result(&ap)["structuredContent"];
    assert_eq!(a["outcome"], "Applied", "{ap}");
    assert!(!a["diff"].as_array().unwrap().is_empty());
    assert_eq!(a["check"]["passed"], true);
    assert_eq!(revision(&local, &id), 2);
    let after = tool_calls(&local, &session);
    let applied: Vec<&Value> = after
        .iter()
        .filter(|c| c["tool"] == "Apply" && c["dry_run"] == false && c["state"] == "Answered")
        .collect();
    assert_eq!(
        applied.len(),
        2,
        "the refused apply and this one: {after:?}"
    );
    let last = applied[1];
    assert_eq!(
        (&last["caller"], &last["state"]),
        (&json!("Agent"), &json!("Answered"))
    );
    let e = revised(&second, Duration::from_secs(1)).expect("revision 2 on the channel");
    assert_eq!(e.data["number"], 2);
    assert_eq!(e.data["application"], last["application"]);

    // simulate: each KPI with what it measures.
    let s = server.handle(&line(8)).unwrap();
    let r = tool_result(&s);
    assert_eq!(r["isError"], false, "{s}");
    let text = r["content"][0]["text"].as_str().unwrap();
    assert!(text.starts_with("Revision 2, 1 seeds of 2h"), "{text}");
    assert!(text.contains("owner_wait_share = "), "{text}");

    assert_eq!(server.handle(&line(9)).unwrap()["result"], json!({}));
    server.close();
    let (_, v) = local.request("GET", "/api/sessions", None).unwrap();
    assert_eq!(v["sessions"][0]["state"], "Closed");
}

#[test]
fn the_tool_descriptions_carry_the_verbs_and_the_fact_relations() {
    let tools = agenticstudio_mcp::tools();
    let text = |name: &str| -> String {
        tools
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == name)
            .unwrap()["description"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let apply = text("apply");
    for needle in [
        "set(P, field, value)",
        "add(part{",
        "add(link{",
        "remove, connect, replicate, wrap, prepend, append, mark and unmark",
        "part{id, block, count}",
        "setting{part, name, value}",
        "link{index, from, to, carries, over}",
        "marker{path, value}",
    ] {
        assert!(apply.contains(needle), "apply lacks {needle}");
    }
    assert!(text("query").contains("link{index, from, to, carries, over}"));
    for t in tools.as_array().unwrap() {
        assert_eq!(t["inputSchema"]["type"], "object", "{t}");
    }
}

#[test]
fn the_protocol_answers_errors_as_json_rpc() {
    let (local, id) = studio();
    let mut server = Server::new(local, Target::Draft(id));
    let e = server.handle("{not json").unwrap();
    assert_eq!(e["error"]["code"], -32700);
    let e = server
        .handle(r#"{"jsonrpc":"2.0","id":"a","method":"resources/list"}"#)
        .unwrap();
    assert_eq!(
        (&e["id"], &e["error"]["code"]),
        (&json!("a"), &json!(-32601))
    );
    let e = server
        .handle(r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"remove"}}"#)
        .unwrap();
    assert_eq!(e["error"]["code"], -32602);
    let old = server
        .handle(r#"{"jsonrpc":"2.0","id":4,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#)
        .unwrap();
    assert_eq!(old["result"]["protocolVersion"], "2024-11-05");
    let unknown = server
        .handle(r#"{"jsonrpc":"2.0","id":5,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#)
        .unwrap();
    assert_eq!(unknown["result"]["protocolVersion"], "2025-06-18");
    let r = server
        .handle(r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"query","arguments":{}}}"#)
        .unwrap();
    assert_eq!(r["result"]["isError"], true, "{r}");
    let f = server
        .handle(r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"query","arguments":{"selector":"nothing{x: X}"}}}"#)
        .unwrap();
    assert_eq!(f["result"]["isError"], true, "{f}");
    assert!(
        f["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("Failed: InvalidSelector"),
        "{f}"
    );
}

#[test]
fn run_answers_line_by_line_and_closes_the_session_at_the_end() {
    let (local, _) = studio();
    let mut server = Server::new(local.clone(), Target::Newest);
    let input = format!("{}\n\n{}\n{}\n", line(1), exchange()[1], line(4));
    let mut out = Vec::new();
    server.run(input.as_bytes(), &mut out).unwrap();
    let lines: Vec<Value> = String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1]["id"], 4);
    assert_eq!(
        lines[1]["result"]["structuredContent"]["check"]["passed"],
        true
    );
    let (_, v) = local.request("GET", "/api/sessions", None).unwrap();
    assert_eq!(v["sessions"].as_array().unwrap().len(), 1);
    assert_eq!(v["sessions"][0]["state"], "Closed");
}

#[test]
fn a_draft_is_opened_found_or_explained_and_a_lost_session_is_started_again() {
    let local = Local(Arc::new(Mutex::new(Studio::new(root()))));
    let mut none = Server::new(local.clone(), Target::Newest);
    let r = none.handle(&line(4)).unwrap();
    assert_eq!(r["result"]["isError"], true);
    assert!(
        r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("NoDraft"),
        "{r}"
    );

    let mut opener = Server::new(
        local.clone(),
        Target::Open("architectures/solo.yaml".into()),
    );
    let r = opener.handle(&line(4)).unwrap();
    assert_eq!(r["result"]["isError"], false, "{r}");
    let draft = opener.draft().unwrap().to_string();
    let (_, v) = local
        .request("GET", &format!("/api/drafts/{draft}"), None)
        .unwrap();
    assert_eq!(v["draft"]["source"], "architectures/solo.yaml");

    let session = opener.session().unwrap().to_string();
    local
        .request("POST", &format!("/api/sessions/{session}/close"), None)
        .unwrap();
    let r = opener.handle(&line(4)).unwrap();
    assert_eq!(r["result"]["isError"], false, "{r}");
    assert_ne!(opener.session().unwrap(), session);

    let mut missing = Server::new(local, Target::Open("architectures/nope.yaml".into()));
    let r = missing.handle(&line(4)).unwrap();
    assert!(
        r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("NoSuchSource"),
        "{r}"
    );
}

#[test]
fn the_studio_address_is_read_from_its_url() {
    assert_eq!(
        Http::parse("http://127.0.0.1:7431/").unwrap(),
        Http {
            host: "127.0.0.1".into(),
            port: 7431
        }
    );
    assert_eq!(Http::parse("http://localhost").unwrap().port, 80);
    assert!(Http::parse("https://127.0.0.1:7431").is_err());
    assert!(Http::parse("http://127.0.0.1:x").is_err());
    assert!(Http::parse("http://:7431").is_err());
}
