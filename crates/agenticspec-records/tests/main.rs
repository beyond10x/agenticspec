//! Tests of the records crate. One binary (workspace rule); fixtures are synthetic, written by
//! hand under `tests/fixtures/`.

mod wire {
    use std::path::{Path, PathBuf};

    use agenticspec_records::claude::{self, Cmd, Step};
    use agenticspec_records::replay::replay;
    use agenticspec_records::wire::{
        Answer, GatewayFacts, Wire, WireCall, WireSession, join, lift_call, session_hash,
    };
    use agenticspec_system::SystemEvent;
    use agenticspec_types::session as s;
    use serde_json::Value;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/wire")
            .join(name)
    }

    /// The calls of a fixture file and its session hash.
    fn calls(name: &str) -> (String, Vec<WireCall>) {
        let doc: Value =
            serde_json::from_str(&std::fs::read_to_string(fixture(name)).unwrap()).unwrap();
        let hash = session_hash("claude-code", doc["session"].as_str().unwrap());
        let calls = doc["calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                let f = &c["facts"];
                let int = |v: &Value| v.as_i64();
                WireCall {
                    wire: match c["wire"].as_str().unwrap() {
                        "messages" => Wire::Messages,
                        "chat" => Wire::Chat,
                        _ => Wire::Responses,
                    },
                    request: c["request"].to_string().into_bytes(),
                    answer: match c.get("events") {
                        Some(Value::Array(es)) => {
                            Answer::Events(es.iter().map(|e| e.to_string().into_bytes()).collect())
                        }
                        _ => Answer::Body(c["body"].to_string().into_bytes()),
                    },
                    facts: GatewayFacts {
                        call_id: f["call_id"].as_str().unwrap().to_owned(),
                        session_hash: hash.clone(),
                        harness: s::Harness::ClaudeCode,
                        path: c["path"].as_str().unwrap().to_owned(),
                        route: f["route"].as_str().map(str::to_owned),
                        upstream: f["upstream"].as_str().map(str::to_owned),
                        attempts: int(&f["attempts"]).unwrap_or(1),
                        queue_wait_ms: int(&f["queue_wait_ms"]),
                        ttft_ms: int(&f["ttft_ms"]),
                        declared_cost_microdollars: int(&f["declared_cost_microdollars"]),
                        incomplete: match f["incomplete"].as_str() {
                            Some("client_disconnected") => {
                                Some(s::IncompleteReason::ClientDisconnected)
                            }
                            Some("upstream_cut") => Some(s::IncompleteReason::UpstreamCut),
                            Some("wire_error") => Some(s::IncompleteReason::WireError),
                            Some("refused") => Some(s::IncompleteReason::Refused),
                            _ => None,
                        },
                        started_ms: c["started_ms"].as_i64().unwrap(),
                        finished_ms: c["finished_ms"].as_i64().unwrap(),
                    },
                }
            })
            .collect();
        (hash, calls)
    }

    fn lift_all(name: &str) -> (String, WireSession, Vec<Step>) {
        let (hash, calls) = calls(name);
        let mut session = WireSession::new();
        let steps = calls
            .iter()
            .flat_map(|c| lift_call(&mut session, c))
            .collect();
        (hash, session, steps)
    }

    fn agents(events: &[SystemEvent]) -> Vec<&s::AgentStarted> {
        events
            .iter()
            .filter_map(|e| match e {
                SystemEvent::AgentStarted(a) => Some(a),
                _ => None,
            })
            .collect()
    }

    fn turns(events: &[SystemEvent]) -> Vec<&s::ModelTurnRecorded> {
        events
            .iter()
            .filter_map(|e| match e {
                SystemEvent::ModelTurnRecorded(t) => Some(t),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_session_hash_is_the_gateways() {
        // Computed with `sha256sum` over `llm-gateway.session/1\0claude-code\0<value>`, outside
        // this code: the first 12 hex digits.
        assert_eq!(
            session_hash("claude-code", "6f1d3c1e-secret-session"),
            "57f5912d72e5"
        );
        assert_eq!(session_hash("claude-code", "s1"), "42c03d54b40a");
    }

    #[test]
    fn a_transcript_names_its_session_by_the_same_hash() {
        let main = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../agenticspec/tests/fixtures/project/s1.jsonl");
        let lifted = claude::lift(&main).unwrap();
        let open = lifted
            .steps
            .iter()
            .find_map(|st| match &st.cmd {
                Cmd::OpenSession(o) => Some(o),
                _ => None,
            })
            .unwrap();
        assert_eq!(open.session_hash.as_deref(), Some("42c03d54b40a"));
        assert_eq!(open.source, Some(s::Source::Disk));
        assert!(lifted.steps.iter().all(|st| st.source == s::Source::Disk));
    }

    #[test]
    fn a_spawn_a_side_call_and_a_failed_tool_on_the_messages_wire() {
        let (hash, session, steps) = lift_all("messages.json");
        assert_eq!(
            (session.lifted, session.skipped),
            (6, 1),
            "count_tokens is skipped"
        );
        assert!(steps.iter().all(|st| st.source == s::Source::Wire));
        let r = replay(&steps);
        assert_eq!(
            r.refused, 0,
            "every command is accepted by the generated component"
        );

        let started = agents(&r.events);
        assert_eq!(started.len(), 3);
        let main = started
            .iter()
            .find(|a| a.role == s::AgentRole::Main)
            .unwrap();
        let sub = started
            .iter()
            .find(|a| a.role == s::AgentRole::SubAgent)
            .unwrap();
        let side = started
            .iter()
            .find(|a| a.role == s::AgentRole::SideCall)
            .unwrap();
        assert_eq!(sub.parent_agent_id.as_ref(), Some(&main.agent_id));
        assert_eq!(
            sub.spawned_by.as_ref().map(|c| c.0.as_str()),
            Some("toolu_1")
        );
        assert_eq!(side.parent_agent_id.as_ref(), Some(&main.agent_id));
        assert!(matches!(&r.events[0], SystemEvent::SessionOpened(o)
            if o.session_id.0 == hash && o.source == Some(s::Source::Wire)));

        let failed = r.events.iter().any(|e| {
            matches!(e, SystemEvent::ToolCallCompleted(c) if c.tool_call_id.0 == "toolu_2" && !c.ok)
        });
        assert!(failed, "the sub-agent's command failed");
        let decided: Vec<&s::ToolCallDecided> = r
            .events
            .iter()
            .filter_map(|e| match e {
                SystemEvent::ToolCallDecided(d) => Some(d),
                _ => None,
            })
            .collect();
        assert!(decided.iter().all(|d| d.decider == s::Decider::NotRecorded));

        let t = turns(&r.events);
        let ids: Vec<&str> = t.iter().filter_map(|t| t.message_id.as_deref()).collect();
        assert_eq!(
            ids,
            ["msg_A1", "msg_B1", "msg_B2", "msg_S1", "msg_A2", "msg_A3"]
        );
        let a1 = t[0];
        assert_eq!(a1.call_id.as_deref(), Some("c1"));
        assert_eq!(a1.ttft_ms, Some(400));
        assert_eq!(a1.declared_cost_microdollars, Some(1200));
        assert_eq!(a1.usage.output, 20, "message_delta's usage wins");
        assert_eq!(t[2].attempts, Some(2));
        assert_eq!(
            t[4].incomplete,
            Some(s::IncompleteReason::ClientDisconnected)
        );

        let prompts: Vec<&s::PromptSubmitted> = r
            .events
            .iter()
            .filter_map(|e| match e {
                SystemEvent::PromptSubmitted(p) => Some(p),
                _ => None,
            })
            .collect();
        let authors: Vec<s::PromptAuthor> = prompts.iter().map(|p| p.author).collect();
        assert_eq!(
            authors,
            [
                s::PromptAuthor::Human,
                s::PromptAuthor::ParentAgent,
                s::PromptAuthor::Harness,
                s::PromptAuthor::Human
            ]
        );
        assert!(
            prompts
                .iter()
                .all(|p| p.attribution == s::Attribution::Inferred)
        );
    }

    #[test]
    fn no_text_leaves_the_wire_lift() {
        let (_, _, steps) = lift_all("messages.json");
        let text = agenticspec_records::store::encode_steps(&steps);
        for needle in [
            "Fix the build",
            "Find the failing test",
            "cargo test",
            "exit 101",
        ] {
            assert!(!text.contains(needle), "{needle} reached the records");
        }
    }

    #[test]
    fn a_responses_chain_follows_previous_response_id() {
        let (_, _, steps) = lift_all("responses.json");
        let r = replay(&steps);
        assert_eq!(r.refused, 0);
        assert_eq!(agents(&r.events).len(), 1);
        let ids: Vec<Option<&str>> = turns(&r.events)
            .iter()
            .map(|t| t.message_id.as_deref())
            .collect();
        assert_eq!(ids, [Some("resp_1"), Some("resp_2")]);
        assert!(r.events.iter().any(
            |e| matches!(e, SystemEvent::ToolCallCompleted(c) if c.tool_call_id.0 == "fc_1" && c.ok)
        ));
        let t = turns(&r.events);
        assert_eq!((t[1].usage.cache_read, t[1].usage.uncached_input), (30, 15));
    }

    #[test]
    fn a_streamed_tool_call_on_the_chat_wire() {
        let (_, _, steps) = lift_all("chat.json");
        let r = replay(&steps);
        assert_eq!(r.refused, 0);
        assert_eq!(
            agents(&r.events).len(),
            1,
            "the system message is not the chain"
        );
        let requested = r.events.iter().find_map(|e| match e {
            SystemEvent::ToolCallRequested(c) => Some(c),
            _ => None,
        });
        assert_eq!(requested.map(|c| c.tool.as_str()), Some("read_file"));
        assert!(r.events.iter().any(
            |e| matches!(e, SystemEvent::ToolCallCompleted(c) if c.tool_call_id.0 == "call_1")
        ));
        let ids: Vec<Option<&str>> = turns(&r.events)
            .iter()
            .map(|t| t.message_id.as_deref())
            .collect();
        assert_eq!(ids, [Some("chatcmpl-1"), Some("chatcmpl-2")]);
    }

    #[test]
    fn wire_facts_join_a_transcript_on_the_message_id() {
        let main = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../agenticspec/tests/fixtures/project/s1.jsonl");
        let mut disk = claude::lift(&main).unwrap().steps;
        let (_, mut calls) = calls("messages.json");
        // The first call's answer, as if the transcript's first message had passed the gateway.
        let mut call = calls.remove(0);
        call.answer = Answer::Body(
            serde_json::json!({
                "id": "m1", "model": "claude-opus-5-5", "stop_reason": "end_turn",
                "content": [], "usage": {"input_tokens": 3, "output_tokens": 1}
            })
            .to_string()
            .into_bytes(),
        );
        let wire = lift_call(&mut WireSession::new(), &call);
        assert_eq!(join(&mut disk, &wire), 1);
        let joined = disk
            .iter()
            .find_map(|st| match &st.cmd {
                Cmd::RecordModelTurn(t) if t.message_id.as_deref() == Some("m1") => Some(t),
                _ => None,
            })
            .unwrap();
        assert_eq!(joined.call_id.as_deref(), Some("c1"));
        assert_eq!(joined.route.as_deref(), Some("claude"));
        assert_eq!(joined.declared_cost_microdollars, Some(1200));
    }

    #[test]
    fn the_appender_writes_a_wire_session_to_the_store() {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("appender");
        let _ = std::fs::remove_dir_all(&dir);
        let db = dir.join("sessions.db");
        let (hash, calls) = calls("messages.json");
        let (handle, join) = agenticspec_records::append::Appender::spawn(&db, 16).unwrap();
        let mut session = WireSession::new();
        for c in &calls {
            assert!(handle.append(&hash, s::Harness::ClaudeCode, lift_call(&mut session, c)));
        }
        let stats = &handle.stats();
        let queued = stats.queued.load(std::sync::atomic::Ordering::Relaxed);
        drop(handle);
        join.join().unwrap();
        assert_eq!(queued, 6, "count_tokens appends nothing");

        let store = agenticspec_records::store::Store::open(&db).unwrap();
        let stored = store.wire_steps(&hash).unwrap();
        assert!(stored.iter().all(|st| st.source == s::Source::Wire));
        let rows = store.sessions().unwrap();
        let row = rows.iter().find(|r| r.session_id.0 == hash).unwrap();
        assert_eq!(row.agents, 3);
        assert_eq!(store.agents(&hash).unwrap().len(), 3);
        let loaded = store.load(Path::new(&format!("wire:{hash}"))).unwrap();
        assert_eq!(loaded.steps.len(), stored.len());
    }
}

/// A synthetic session of many calls on the messages wire, built in code: one main agent in a tool
/// loop, a new human prompt every 20 calls. No text from real traffic.
mod long {
    use agenticspec_records::claude::Step;
    use agenticspec_records::wire::{
        Answer, GatewayFacts, Wire, WireCall, WireSession, lift_call, session_hash,
    };
    use agenticspec_types::session as s;
    use serde_json::json;

    pub fn calls(n: usize) -> (String, Vec<Vec<Step>>) {
        calls_for("synthetic-long-session", n)
    }

    pub fn calls_for(session: &str, n: usize) -> (String, Vec<Vec<Step>>) {
        let hash = session_hash("claude-code", session);
        let mut session = WireSession::new();
        let mut out = Vec::with_capacity(n);
        let t0 = 1_795_600_000_000i64;
        for i in 0..n {
            let mut messages = vec![
                json!({"role": "user", "content": [{"type": "text", "text": "Fix the build"}]}),
            ];
            if i > 0 {
                let prev = format!("toolu_{}", i - 1);
                messages.push(json!({"role": "assistant", "content": [{"type": "tool_use", "id": prev, "name": "Bash", "input": {"command": "cargo test"}}]}));
                if i % 20 == 0 {
                    messages.push(json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": prev, "content": "ok"}, {"type": "text", "text": format!("next step {i}")}]}));
                } else {
                    messages.push(json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": prev, "is_error": i % 7 == 0, "content": "ok"}]}));
                }
            }
            let request = json!({"model": "claude-opus-5-5", "tools": [{"name": "Bash"}], "messages": messages});
            let body = json!({
                "id": format!("msg_{i}"), "model": "claude-opus-5-5", "stop_reason": "tool_use",
                "content": [{"type": "tool_use", "id": format!("toolu_{i}"), "name": "Bash", "input": {"command": "cargo test"}}],
                "usage": {"input_tokens": 10, "cache_read_input_tokens": 1000 + i as i64 * 50, "cache_creation_input_tokens": 50, "output_tokens": 20}
            });
            let start = t0 + i as i64 * 4000;
            let call = WireCall {
                wire: Wire::Messages,
                request: request.to_string().into_bytes(),
                answer: Answer::Body(body.to_string().into_bytes()),
                facts: GatewayFacts {
                    call_id: format!("c{i}"),
                    session_hash: hash.clone(),
                    harness: s::Harness::ClaudeCode,
                    path: "/v1/messages".to_owned(),
                    route: Some("claude".to_owned()),
                    upstream: Some("anthropic".to_owned()),
                    attempts: 1,
                    queue_wait_ms: Some(2),
                    ttft_ms: Some(300),
                    declared_cost_microdollars: Some(900),
                    incomplete: None,
                    started_ms: start,
                    finished_ms: start + 1500,
                },
            };
            out.push(lift_call(&mut session, &call));
        }
        (hash, out)
    }

    fn store(name: &str) -> agenticspec_records::store::Store {
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&dir);
        agenticspec_records::store::Store::open(&dir.join("sessions.db")).unwrap()
    }

    fn program() -> agenticspec_records::sq::Program {
        agenticspec_records::sq::compile(agenticspec_records::store::CHECKS).unwrap()
    }

    /// What a projection of the whole stream writes: the steps appended in one batch to a store
    /// that holds no projection of the session.
    fn full(name: &str, hash: &str, steps: &[Step]) -> String {
        let store = store(name);
        let mut open = agenticspec_records::store::Projections::new(1);
        store
            .append(hash, s::Harness::ClaudeCode, steps, &program(), &mut open)
            .unwrap();
        store.projection_text(hash).unwrap()
    }

    #[test]
    fn appending_call_by_call_writes_what_a_whole_projection_writes() {
        let (hash, calls) = calls(300);
        let program = program();
        let store = store("append-incremental");
        let mut open = agenticspec_records::store::Projections::new(8);
        for steps in &calls {
            store
                .append(&hash, s::Harness::ClaudeCode, steps, &program, &mut open)
                .unwrap();
        }
        store.flush_checks(&program, &mut open).unwrap();
        let all: Vec<Step> = calls.concat();
        let incremental = store.projection_text(&hash).unwrap();
        assert!(incremental.lines().count() > 900, "the session has events");
        assert_eq!(incremental, full("append-whole", &hash, &all));
    }

    #[test]
    fn the_appender_thread_writes_what_a_whole_projection_writes() {
        let (hash, calls) = calls(120);
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("append-thread");
        let _ = std::fs::remove_dir_all(&dir);
        let db = dir.join("sessions.db");
        let (handle, join) = agenticspec_records::append::Appender::spawn(&db, 1024).unwrap();
        for steps in &calls {
            assert!(handle.append(&hash, s::Harness::ClaudeCode, steps.clone()));
        }
        drop(handle);
        join.join().unwrap();
        let store = agenticspec_records::store::Store::open(&db).unwrap();
        let all: Vec<Step> = calls.concat();
        assert_eq!(
            store.projection_text(&hash).unwrap(),
            full("append-thread-whole", &hash, &all)
        );
    }

    #[test]
    fn an_earlier_call_appended_late_projects_the_session_again() {
        let (hash, calls) = calls(40);
        let program = program();
        let store = store("append-late");
        let mut open = agenticspec_records::store::Projections::new(8);
        // Calls 1.. first, then call 0, whose events are earlier than every one before.
        let order: Vec<&Vec<Step>> = calls[1..]
            .iter()
            .chain(std::iter::once(&calls[0]))
            .collect();
        for steps in &order {
            store
                .append(&hash, s::Harness::ClaudeCode, steps, &program, &mut open)
                .unwrap();
        }
        store.flush_checks(&program, &mut open).unwrap();
        let all: Vec<Step> = order.into_iter().flatten().cloned().collect();
        assert_eq!(
            store.projection_text(&hash).unwrap(),
            full("append-late-whole", &hash, &all)
        );
    }

    #[test]
    fn a_session_dropped_from_the_cache_is_projected_again_from_the_store() {
        let (hash, calls) = calls(30);
        let (other, other_calls) = {
            let (_, c) = calls_for("another-synthetic-session", 5);
            (
                agenticspec_records::wire::session_hash("claude-code", "another-synthetic-session"),
                c,
            )
        };
        let program = program();
        let store = store("append-evict");
        let mut open = agenticspec_records::store::Projections::new(1);
        for (i, steps) in calls.iter().enumerate() {
            store
                .append(&hash, s::Harness::ClaudeCode, steps, &program, &mut open)
                .unwrap();
            if i < other_calls.len() {
                // Another session takes the only slot.
                store
                    .append(
                        &other,
                        s::Harness::ClaudeCode,
                        &other_calls[i],
                        &program,
                        &mut open,
                    )
                    .unwrap();
            }
        }
        store.flush_checks(&program, &mut open).unwrap();
        let all: Vec<Step> = calls.concat();
        assert_eq!(
            store.projection_text(&hash).unwrap(),
            full("append-evict-whole", &hash, &all)
        );
    }

    /// Probe, not part of the gate: `cargo test -p agenticspec-records --release -- --ignored
    /// --nocapture append_cost`. Appends 2,000 calls one by one and prints the total and the mean
    /// of the last 100 appends.
    #[test]
    #[ignore = "timing probe"]
    fn append_cost() {
        let n: usize = std::env::var("APPEND_CALLS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2000);
        let (hash, calls) = calls(n);
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("append-cost");
        let _ = std::fs::remove_dir_all(&dir);
        let store = agenticspec_records::store::Store::open(&dir.join("sessions.db")).unwrap();
        let program = agenticspec_records::sq::compile(agenticspec_records::store::CHECKS).unwrap();
        let mut cache = agenticspec_records::store::Projections::new(8);
        let all = std::time::Instant::now();
        let mut last = std::time::Duration::ZERO;
        for (i, steps) in calls.iter().enumerate() {
            let t = std::time::Instant::now();
            store
                .append(&hash, s::Harness::ClaudeCode, steps, &program, &mut cache)
                .unwrap();
            if i + 100 >= calls.len() {
                last += t.elapsed();
            }
        }
        eprintln!(
            "append_cost: {n} calls, total {:.2} s, last 100 mean {:.2} ms",
            all.elapsed().as_secs_f64(),
            last.as_secs_f64() * 10.0
        );
    }
}

mod fixes;

mod legacy_dir {
    use agenticspec_records::store::adopt_legacy_dir;
    use std::path::{Path, PathBuf};

    fn base(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn moves_the_old_directory_when_the_new_one_is_missing() {
        let b = base("legacy-moves");
        let (old, new) = (b.join("sessionspec"), b.join("agenticspec"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("sessions.db"), b"x").unwrap();
        assert!(adopt_legacy_dir(&old, &new).unwrap());
        assert!(!old.exists());
        assert_eq!(std::fs::read(new.join("sessions.db")).unwrap(), b"x");
    }

    #[test]
    fn leaves_both_when_the_new_one_exists() {
        let b = base("legacy-keeps");
        let (old, new) = (b.join("sessionspec"), b.join("agenticspec"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        assert!(!adopt_legacy_dir(&old, &new).unwrap());
        assert!(old.exists() && new.exists());
    }

    #[test]
    fn tries_the_newest_earlier_name_first() {
        let names: Vec<_> = agenticspec_records::store::legacy_dirs()
            .iter()
            .map(|d| d.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["agentspec", "sessionspec"]);
        assert_eq!(
            agenticspec_records::store::default_dir().parent(),
            agenticspec_records::store::legacy_dirs()[0].parent()
        );
    }

    #[test]
    fn does_nothing_without_an_old_directory() {
        let b = base("legacy-none");
        let (old, new) = (b.join("sessionspec"), b.join("agenticspec"));
        assert!(!adopt_legacy_dir(&old, &new).unwrap());
        assert!(!new.exists());
    }
}
