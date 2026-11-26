//! A schema change keeps the wire rows, an empty append queues nothing, and a long wire session
//! holds a bounded number of links. Synthetic data only.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Instant;

use agenticspec_records::claude::{Cmd, Step};
use agenticspec_records::store::{CHECKS, Projections, Store, WIRE_FORMAT};
use agenticspec_records::wire::{
    Answer, GatewayFacts, LINKS_HELD, Wire, WireCall, WireSession, lift_call, session_hash,
};
use agenticspec_types::session as s;
use serde_json::json;

fn fresh(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir.join("sessions.db")
}

/// A store holding one wire session of `n` calls, appended in one batch, and its projection.
fn wire_store(name: &str, n: usize) -> (PathBuf, String, String) {
    let db = fresh(name);
    let (hash, calls) = crate::long::calls_for("synthetic-schema-session", n);
    let store = Store::open(&db).unwrap();
    let program = agenticspec_records::sq::compile(CHECKS).unwrap();
    let all: Vec<Step> = calls.concat();
    store
        .append(
            &hash,
            s::Harness::ClaudeCode,
            &all,
            &program,
            &mut Projections::new(1),
        )
        .unwrap();
    let text = store.projection_text(&hash).unwrap();
    (db, hash, text)
}

fn sqlite(db: &Path) -> rusqlite::Connection {
    rusqlite::Connection::open(db).unwrap()
}

fn count(db: &Path, table: &str) -> i64 {
    sqlite(db)
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn meta(db: &Path, key: &str) -> Option<String> {
    sqlite(db)
        .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
        .ok()
}

#[test]
fn a_store_of_another_schema_keeps_its_wire_rows_and_projects_them_again() {
    let (db, hash, before) = wire_store("fix-schema-keep", 40);
    let rows = count(&db, "wire_steps");
    assert!(rows > 100, "the session has wire rows");
    // As a store an earlier build wrote: another schema, no recorded wire format.
    let c = sqlite(&db);
    c.execute("UPDATE meta SET value = '3' WHERE key = 'schema'", [])
        .unwrap();
    c.execute("DELETE FROM meta WHERE key = 'wire_format'", [])
        .unwrap();
    drop(c);

    let store = Store::open(&db).unwrap();
    assert_eq!(count(&db, "wire_steps"), rows, "no wire row is lost");
    assert_eq!(
        meta(&db, "schema").as_deref(),
        Some(
            agenticspec_records::store::SCHEMA_VERSION
                .to_string()
                .as_str()
        )
    );
    assert_eq!(
        meta(&db, "wire_format").as_deref(),
        Some(WIRE_FORMAT.to_string().as_str())
    );
    assert_eq!(
        store.projection_text(&hash).unwrap(),
        before,
        "the views are projected again from the wire rows"
    );
}

#[test]
fn a_store_whose_wire_rows_this_build_cannot_read_is_refused_and_unchanged() {
    let (db, _, _) = wire_store("fix-schema-refuse", 10);
    let c = sqlite(&db);
    c.execute("UPDATE meta SET value = '3' WHERE key = 'schema'", [])
        .unwrap();
    c.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('wire_format', '99')",
        [],
    )
    .unwrap();
    drop(c);
    let (rows, sessions) = (count(&db, "wire_steps"), count(&db, "sessions"));

    let Err(e) = Store::open(&db) else {
        panic!("a store in an unreadable wire format opened");
    };
    assert!(e.contains("wire format 99"), "{e}");
    assert!(e.contains(&format!("wire format {WIRE_FORMAT}")), "{e}");
    assert_eq!(count(&db, "wire_steps"), rows);
    assert_eq!(count(&db, "sessions"), sessions);
    assert_eq!(meta(&db, "schema").as_deref(), Some("3"));
    assert_eq!(meta(&db, "wire_format").as_deref(), Some("99"));
}

#[test]
fn an_empty_append_returns_true_and_queues_nothing() {
    let db = fresh("fix-empty-append");
    let (handle, join) = agenticspec_records::append::Appender::spawn(&db, 4).unwrap();
    assert!(handle.append("any-session", s::Harness::ClaudeCode, Vec::new()));
    let stats = handle.stats();
    assert_eq!(stats.queued.load(Ordering::Relaxed), 0);
    assert_eq!(stats.dropped.load(Ordering::Relaxed), 0);
    drop(handle);
    join.join().unwrap();
}

fn facts(hash: &str, i: i64) -> GatewayFacts {
    let start = 1_795_600_000_000 + i * 1000;
    GatewayFacts {
        call_id: format!("c{i}"),
        session_hash: hash.to_owned(),
        harness: s::Harness::ClaudeCode,
        path: "/v1/messages".to_owned(),
        route: None,
        upstream: None,
        attempts: 1,
        queue_wait_ms: None,
        ttft_ms: None,
        declared_cost_microdollars: None,
        incomplete: None,
        started_ms: start,
        finished_ms: start + 10,
    }
}

/// One call of the main agent whose answer spawns a sub-agent with the prompt `job <i>`.
fn spawning_call(hash: &str, i: i64) -> WireCall {
    let request = json!({"model": "claude-opus-5-5", "tools": [{"name": "Task"}],
        "messages": [{"role": "user", "content": [{"type": "text", "text": "Plan the work"}]}]});
    let body = json!({"id": format!("msg_{i}"), "model": "claude-opus-5-5", "stop_reason": "tool_use",
        "content": [{"type": "tool_use", "id": format!("toolu_{i}"), "name": "Task",
                     "input": {"prompt": format!("job {i}")}}],
        "usage": {"input_tokens": 1, "output_tokens": 1}});
    WireCall {
        wire: Wire::Messages,
        request: request.to_string().into_bytes(),
        answer: Answer::Body(body.to_string().into_bytes()),
        facts: facts(hash, i),
    }
}

/// The agents a call started: (parent, spawned by).
fn started(steps: &[Step]) -> Vec<(Option<String>, Option<String>)> {
    steps
        .iter()
        .filter_map(|st| match &st.cmd {
            Cmd::StartAgent(a) => Some((
                a.parent_agent_id.as_ref().map(|p| p.0.clone()),
                a.spawned_by.as_ref().map(|c| c.0.clone()),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn a_long_wire_session_holds_at_most_links_held_links() {
    let hash = session_hash("claude-code", "synthetic-long-links");
    let mut session = WireSession::new();
    let n: i64 = 6 * LINKS_HELD as i64;
    let t = Instant::now();
    for i in 0..n {
        lift_call(&mut session, &spawning_call(&hash, i));
        let (responses, conversations, spawns) = session.links_held();
        assert!(responses <= LINKS_HELD && conversations <= LINKS_HELD && spawns <= LINKS_HELD);
    }
    eprintln!(
        "a_long_wire_session: {n} calls in {:.2} s",
        t.elapsed().as_secs_f64()
    );
    assert_eq!(session.links_held().0, LINKS_HELD);
    assert_eq!(session.links_held().2, LINKS_HELD);

    // A sub-agent whose spawn input is still held is linked to the call that spawned it.
    let sub = |i: i64, at: i64| {
        WireCall {
        wire: Wire::Messages,
        request: json!({"model": "claude-opus-5-5", "tools": [{"name": "Bash"}],
            "messages": [{"role": "user", "content": [{"type": "text", "text": format!("job {i}")}]}]})
        .to_string()
        .into_bytes(),
        answer: Answer::Body(
            json!({"id": format!("sub_{i}"), "model": "claude-opus-5-5", "stop_reason": "end_turn",
                "content": [], "usage": {"input_tokens": 1, "output_tokens": 1}})
            .to_string()
            .into_bytes(),
        ),
        facts: facts(&hash, at),
    }
    };
    let held = started(&lift_call(&mut session, &sub(n - 1, n)));
    assert_eq!(held.len(), 1);
    assert_eq!(
        held[0].1.as_deref(),
        Some(format!("toolu_{}", n - 1).as_str())
    );
    assert!(held[0].0.is_some());

    // A sub-agent whose spawn input was evicted starts without a parent: unknown, not wrong.
    let lost = started(&lift_call(&mut session, &sub(0, n + 1)));
    assert_eq!(lost, [(None, None)]);

    // A responses call that names an evicted response id starts a chain of its own.
    let evicted = WireCall {
        wire: Wire::Responses,
        request: json!({"model": "gpt-6", "tools": [{"type": "function", "name": "shell"}],
            "previous_response_id": "msg_0",
            "input": [{"type": "function_call_output", "call_id": "toolu_0", "output": "done"}]})
        .to_string()
        .into_bytes(),
        answer: Answer::Body(
            json!({"id": "resp_late", "model": "gpt-6", "status": "completed", "output": [],
                "usage": {"input_tokens": 1, "output_tokens": 1}})
            .to_string()
            .into_bytes(),
        ),
        facts: GatewayFacts {
            path: "/v1/responses".to_owned(),
            ..facts(&hash, n + 2)
        },
    };
    assert_eq!(started(&lift_call(&mut session, &evicted)), [(None, None)]);
}
