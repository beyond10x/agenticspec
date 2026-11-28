//! `agenticspec sim contract`: the contract of an architecture derived from its document and
//! `blocks/ports.json`, its check, its text, and the predicate language of routing invariants.

use super::common::{repo_root, scratch, yaml};
use agenticspec_sim::cli::{self, ContractOptions};
use agenticspec_sim::contract::derive::{self, Contract};
use agenticspec_sim::contract::ports::PortsFile;
use agenticspec_sim::contract::predicate::{Clause, Predicate, Use};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn ports() -> PortsFile {
    PortsFile::load(&repo_root().join("blocks/ports.json")).unwrap()
}

fn document(path: &str) -> Value {
    agenticspec_sim::doc::load_yaml(&repo_root().join(path)).unwrap()
}

fn contract_of(path: &str) -> Contract {
    derive::derive(&document(path), &ports()).unwrap()
}

fn workstation() -> Contract {
    contract_of("architectures/workstation.yaml")
}

/// The 19 documents `agenticspec sim check` accepts: the architectures and the ladder stages.
fn accepted() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    for dir in ["architectures", "architectures/ladder"] {
        let mut files: Vec<PathBuf> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.extension().is_some_and(|x| x == "yaml")
                    && p.file_name().is_some_and(|n| n != "ladder.yaml")
            })
            .collect();
        files.sort();
        out.extend(files);
    }
    out
}

fn port_names(c: &Contract) -> BTreeSet<&str> {
    c.ports.iter().map(|p| p.port.as_str()).collect()
}

#[test]
fn every_predicate_form_prints_and_parses_back() {
    for text in [
        "every model.inference@worker inside gateway.call@gateway",
        "every question.question@operator to owner",
        "every question.question@operator to owner over chat",
        "every channel.message@owner to operator over chat and every harness.tool-call@a_1 to tests",
    ] {
        let p = Predicate::parse(text).unwrap();
        assert_eq!(p.to_string(), text);
        assert_eq!(Predicate::parse(&p.to_string()).unwrap(), p);
    }
    let p =
        Predicate::parse("every   model.inference@worker   inside gateway.call@gateway").unwrap();
    assert_eq!(
        p.clauses,
        [Clause::Inside {
            every: Use {
                port: "model.inference".into(),
                part: "worker".into()
            },
            within: Use {
                port: "gateway.call".into(),
                part: "gateway".into()
            },
        }]
    );
}

#[test]
fn a_malformed_predicate_is_refused_with_its_offset() {
    for (text, want) in [
        ("", "expected `every` at offset 0"),
        (
            "each model.inference@worker to x",
            "expected `every` at offset 0",
        ),
        ("every model@worker to x", "expected `.` at offset 11"),
        ("every model.inference to x", "expected `@` at offset 21"),
        (
            "every model.inference@ to x",
            "expected a part at offset 22",
        ),
        (
            "every Model.inference@w to x",
            "expected a port (`block.entity`) at offset 6",
        ),
        (
            "every model.inference@w near x",
            "expected `inside` or `to` at offset 24",
        ),
        (
            "every model.inference@w inside gateway@g",
            "expected `.` at offset 38",
        ),
        (
            "every model.inference@w to x over",
            "expected a channel part at offset 33",
        ),
        (
            "every model.inference@w to x or every a.b@c to d",
            "expected `and` or the end at offset 29",
        ),
        (
            "every model.inference@w to x and",
            "expected `every` at offset 32",
        ),
    ] {
        let e = Predicate::parse(text).unwrap_err();
        assert!(e.contains(want), "{text:?}: {e}");
    }
}

#[test]
fn the_workstation_uses_the_ports_of_the_designs_example() {
    // The design's agent.session, model.infer, gateway.call, channel.deliver, principal.answer,
    // pool.lease, workspace.provide and sandbox.enforce; workspace.provide is two derived ports.
    let c = workstation();
    assert_eq!(
        port_names(&c),
        BTreeSet::from([
            "channel.message",
            "gateway.call",
            "harness.session",
            "model.inference",
            "question.question",
            "resource.claim",
            "sandbox.sandbox",
            "workspace.lease",
            "workspace.proposal",
        ])
    );
    for absent in ["runtime.instance", "trigger.firing", "channel.post"] {
        assert!(!port_names(&c).contains(absent), "{absent}");
    }
}

#[test]
fn the_workstations_tree_provides_twelve_workspaces_and_its_box_is_narrowed() {
    let c = workstation();
    for port in ["workspace.lease", "workspace.proposal"] {
        let used = c.ports.iter().find(|p| p.port == port).unwrap();
        assert_eq!((used.part.as_str(), used.instances), ("tree", 12), "{port}");
    }
    let sandbox = c
        .ports
        .iter()
        .find(|p| p.port == "sandbox.sandbox")
        .unwrap();
    assert_eq!(sandbox.part, "box");
    assert_eq!(sandbox.settings["isolation"], json!("process"));
    assert_eq!(sandbox.settings["egress"], json!(["gateway"]));
    assert_eq!(sandbox.instances, 12);
    let worker = c
        .ports
        .iter()
        .find(|p| p.port == "harness.session" && p.part == "worker")
        .unwrap();
    assert_eq!(
        worker.instances, 12,
        "the count of a marked place is its value"
    );
    assert_eq!(worker.settings["harness"], json!("coding"));
    // A port used through a link takes no setting of its part: the owner's kind is no channel's.
    let owner = |port: &str| {
        c.ports
            .iter()
            .find(|p| p.port == port && p.part == "owner")
            .unwrap()
    };
    assert!(owner("channel.message").settings.is_empty());
    assert_eq!(owner("question.question").settings["kind"], json!("human"));
}

#[test]
fn the_workstations_border_and_model_call_invariant() {
    let c = workstation();
    assert_eq!(c.border, ["operator"]);
    let inv = c
        .invariants
        .iter()
        .find(|i| i.link == "worker -> gateway carries model_call")
        .unwrap();
    assert_eq!(inv.port, "model.inference");
    assert_eq!(inv.within.as_deref(), Some("gateway.call"));
    assert_eq!(
        inv.predicate,
        "every model.inference@worker inside gateway.call@gateway"
    );
    let over = c
        .invariants
        .iter()
        .find(|i| i.link == "operator -> owner carries question over chat")
        .unwrap();
    assert_eq!(
        over.predicate,
        "every question.question@operator to owner over chat"
    );
}

#[test]
fn every_protocol_is_the_ports_protocol() {
    let ports = ports();
    for path in accepted() {
        let c = derive::derive(&agenticspec_sim::doc::load_yaml(&path).unwrap(), &ports).unwrap();
        assert_eq!(c.ports_digest, ports.sources_sha256);
        for used in &c.ports {
            let port = ports.port(&used.port).unwrap();
            assert_eq!(
                used.protocol,
                port.protocol.to_string(),
                "{}",
                path.display()
            );
        }
    }
}

#[test]
fn a_schedule_gives_the_clock_port_and_a_board_the_board_port() {
    // The clock port is the firing port: with no periodic cause in ESS (`C10`), its kind is
    // Outbound (epic § "Port derivation against the design's table").
    assert!(
        port_names(&contract_of("architectures/scheduled-loops.yaml")).contains("trigger.firing")
    );
    assert!(port_names(&contract_of("architectures/shared-board.yaml")).contains("channel.post"));
}

#[test]
fn every_accepted_document_has_a_contract_that_passes_its_check_with_the_same_bytes() {
    let docs = accepted();
    assert_eq!(docs.len(), 19, "{docs:?}");
    let ports = ports();
    for path in docs {
        let doc = agenticspec_sim::doc::load_yaml(&path).unwrap();
        let first = derive::derive(&doc, &ports).unwrap();
        let second = derive::derive(&doc, &ports).unwrap();
        assert_eq!(first.text(), second.text(), "{}", path.display());
        assert!(!first.ports.is_empty(), "{}", path.display());
        assert_eq!(derive::check(&first, &doc, &ports), Vec::<String>::new());
        for inv in &first.invariants {
            Predicate::parse(&inv.predicate).unwrap();
        }
        let read = Contract::parse(&first.text()).unwrap();
        assert_eq!(read.text(), first.text(), "{}", path.display());
    }
}

#[test]
fn the_command_writes_the_contract_and_refuses_a_refused_document_writing_nothing() {
    let root = repo_root();
    let dir = scratch("contract-cli");
    let out = dir.join("workstation.contract.yaml");
    let written = cli::contract(
        &ContractOptions {
            architecture: &root.join("architectures/workstation.yaml"),
            ports: Some(&root.join("blocks/ports.json")),
            out: Some(&out),
            check: None,
            suite: None,
            trace: None,
        },
        &mut std::io::sink(),
    )
    .unwrap();
    assert_eq!(written["written"], true);
    assert_eq!(std::fs::read_to_string(&out).unwrap(), workstation().text());
    let printed = cli::contract(
        &ContractOptions {
            architecture: &root.join("architectures/workstation.yaml"),
            ports: Some(&root.join("blocks/ports.json")),
            out: None,
            check: None,
            suite: None,
            trace: None,
        },
        &mut std::io::sink(),
    )
    .unwrap();
    assert_eq!(
        printed["contract"].as_str(),
        Some(workstation().text().as_str())
    );

    let refused_out = dir.join("refused.contract.yaml");
    let err = cli::contract(
        &ContractOptions {
            architecture: &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/cases/contract/unknown-link-part.yaml"),
            ports: Some(&root.join("blocks/ports.json")),
            out: Some(&refused_out),
            check: None,
            suite: None,
            trace: None,
        },
        &mut std::io::sink(),
    )
    .unwrap_err();
    assert_eq!(err.code, "architecture_refused");
    assert!(err.detail.contains("ghost"), "{}", err.detail);
    assert!(!refused_out.exists());
}

#[test]
fn a_contract_whose_predicate_does_not_parse_is_refused_naming_the_link_and_offset() {
    // The fixture is the workstation's contract with one predicate broken, written here so that
    // its ports digest is always the committed one.
    let root = repo_root();
    let good = workstation().text();
    let broken = "every model.inference@ inside gateway.call@gateway";
    let fixture = good.replacen(
        "\"every model.inference@worker inside gateway.call@gateway\"",
        &format!("\"{broken}\""),
        1,
    );
    assert_ne!(fixture, good);
    let file = scratch("contract-bad-predicate").join("bad-predicate.contract.yaml");
    let options = ContractOptions {
        architecture: &root.join("architectures/workstation.yaml"),
        ports: Some(&root.join("blocks/ports.json")),
        out: None,
        check: Some(&file),
        suite: None,
        trace: None,
    };
    std::fs::write(&file, &good).unwrap();
    assert_eq!(
        cli::contract(&options, &mut std::io::sink()).unwrap()["written"],
        false,
        "the good contract passes"
    );
    std::fs::write(&file, &fixture).unwrap();
    let err = cli::contract(&options, &mut std::io::sink()).unwrap_err();
    assert_eq!(err.code, "contract_refused");
    assert!(
        err.detail.contains("worker -> gateway carries model_call"),
        "{}",
        err.detail
    );
    assert!(err.detail.contains("offset 22"), "{}", err.detail);
}

#[test]
fn the_check_refuses_unknown_ports_parts_links_and_an_invariant_off_its_from_part() {
    let doc = document("architectures/workstation.yaml");
    let ports = ports();
    let mut c = workstation();
    c.ports[0].port = "model.dream".into();
    c.ports[1].part = "ghost".into();
    c.invariants[0].link = "a -> b carries nothing".into();
    let inv = c
        .invariants
        .iter_mut()
        .find(|i| i.link == "worker -> gateway carries model_call")
        .unwrap();
    inv.port = "sandbox.sandbox".into();
    let refusals = derive::check(&c, &doc, &ports).join("\n");
    for want in [
        "model.dream",
        "ghost",
        "a -> b carries nothing",
        "sandbox.sandbox",
    ] {
        assert!(refusals.contains(want), "{want}: {refusals}");
    }
}

/// One link of each (carried value, block of `to`) group: the port the link makes its `from`
/// part use and the invariant's port, or none.
#[test]
fn each_carried_value_maps_to_the_port_its_from_part_uses_or_to_none() {
    let parts = yaml(
        "
        p:     { block: principal, kind: human }
        a:     { block: agent, harness: coding }
        b:     { block: agent, harness: coding }
        chat:  { block: channel, kind: operator }
        board: { block: channel, kind: agent_message, distribution: board }
        w:     { block: work, kind: spec }
        run:   { block: runtime, kind: container }
        t:     { block: tool, category: execute, kind: test_suite }
        g:     { block: gateway, rate_limit_per_minute: 120 }
        m:     { block: model, route: frontier }
        pool:  { block: pool, kind: build_slots, capacity: 16 }
        auth:  { block: authority, kind: permissions }
        k:     { block: knowledge, kind: context_files }
        ws:    { block: workspace, kind: worktree, sharing: exclusive }
        clock: { block: schedule, every: 30m }
        ",
    );
    let rows: &[(&str, &str, &str, Option<&str>)] = &[
        ("p", "a", "prompt", None),
        ("a", "b", "message", Some("channel.message")),
        ("a", "p", "message", Some("channel.message")),
        ("a", "b", "order", None),
        ("a", "board", "order", Some("channel.post")),
        ("a", "b", "question", Some("question.question")),
        ("a", "p", "question", Some("question.question")),
        ("a", "board", "read", Some("channel.subscription")),
        ("a", "w", "record", None),
        ("a", "run", "order", Some("runtime.instance")),
        ("a", "run", "run", Some("runtime.instance")),
        ("run", "g", "model_call", Some("model.inference")),
        ("run", "m", "model_call", Some("model.inference")),
        ("a", "b", "review", None),
        ("a", "t", "call", Some("harness.tool-call")),
        ("a", "g", "model_call", Some("model.inference")),
        ("g", "m", "model_call", Some("gateway.call")),
        ("a", "m", "model_call", Some("model.inference")),
        ("a", "pool", "claim", Some("resource.claim")),
        ("auth", "a", "grant", Some("authority.decision")),
        ("a", "p", "proposal", Some("workspace.proposal")),
        ("ws", "a", "checkout", Some("workspace.lease")),
        ("clock", "a", "trigger", Some("trigger.firing")),
        ("k", "a", "context", None),
        ("p", "w", "spec", None),
        ("a", "b", "result", None),
        ("a", "b", "handoff", None),
        ("a", "p", "report", None),
    ];
    let ports = ports();
    for (from, to, carries, want) in rows {
        let mut doc_parts = serde_json::Map::new();
        for n in [from, to] {
            doc_parts.insert((*n).to_owned(), parts[*n].clone());
        }
        let doc = json!({
            "architecture": "groups",
            "parts": doc_parts,
            "links": [{ "from": from, "to": to, "carries": carries }],
            "border": [from],
        });
        let c = derive::derive(&doc, &ports).unwrap();
        let link = format!("{from} -> {to} carries {carries}");
        let inv = c.invariants.iter().find(|i| i.link == link);
        match want {
            Some(port) => {
                assert!(
                    c.ports.iter().any(|p| p.port == *port && p.part == *from),
                    "{link}: {from} does not use {port}: {:?}",
                    c.ports
                );
                assert_eq!(inv.map(|i| i.port.as_str()), Some(*port), "{link}");
            }
            None => assert!(inv.is_none(), "{link}: {inv:?}"),
        }
        assert_eq!(
            derive::check(&c, &doc, &ports),
            Vec::<String>::new(),
            "{link}"
        );
    }
    // A principal a question reaches answers it: it uses the question port too.
    let doc = json!({
        "architecture": "asked",
        "parts": { "a": parts["a"], "p": parts["p"] },
        "links": [{ "from": "a", "to": "p", "carries": "question" }],
        "border": ["a"],
    });
    let c = derive::derive(&doc, &ports).unwrap();
    assert!(
        c.ports
            .iter()
            .any(|u| u.port == "question.question" && u.part == "p")
    );
    let q = ports.port("question.question").unwrap();
    assert_eq!(q.kind.as_str(), "Principal");
}

/// A runtime that executes runs calls its model through the gateway like an agent, and an
/// `on_event` trigger's firing port keeps the event and the part whose events fire it.
#[test]
fn a_runtimes_model_call_nests_in_the_gateway_and_an_on_event_trigger_keeps_its_event() {
    let doc = json!({
        "architecture": "runs",
        "parts": {
            "a": { "block": "agent", "harness": "coding", "role": "planner" },
            "run": { "block": "runtime", "kind": "container" },
            "g": { "block": "gateway", "rate_limit_per_minute": 120 },
            "m": { "block": "model", "route": "frontier" },
            "on_done": { "block": "trigger", "kind": "on_event", "event_name": "agentic.work.TaskCompleted",
                         "source": "a", "policy": "parallel" },
        },
        "links": [
            { "from": "a", "to": "run", "carries": "run" },
            { "from": "run", "to": "g", "carries": "model_call" },
            { "from": "g", "to": "m", "carries": "model_call" },
            { "from": "on_done", "to": "run", "carries": "trigger" },
        ],
        "border": ["a"],
    });
    let ports = ports();
    let c = derive::derive(&doc, &ports).unwrap();
    let inv = c
        .invariants
        .iter()
        .find(|i| i.link == "run -> g carries model_call")
        .unwrap();
    assert_eq!(
        inv.predicate.to_string(),
        "every model.inference@run inside gateway.call@g"
    );
    let firing = c
        .ports
        .iter()
        .find(|p| p.port == "trigger.firing" && p.part == "on_done")
        .unwrap();
    assert_eq!(
        firing.settings["event_name"],
        json!("agentic.work.TaskCompleted")
    );
    assert_eq!(firing.settings["source"], json!("a"));
    let session = c
        .ports
        .iter()
        .find(|p| p.port == "harness.session" && p.part == "a")
        .unwrap();
    assert_eq!(session.settings["role"], json!("planner"));
    assert_eq!(derive::check(&c, &doc, &ports), Vec::<String>::new());
}

#[test]
fn a_block_the_table_does_not_know_is_refused() {
    let doc = json!({
        "architecture": "unknown",
        "parts": { "x": { "block": "teleporter" } },
        "links": [],
        "border": [],
    });
    let refusals = derive::derive(&doc, &ports()).unwrap_err();
    assert!(refusals[0].contains("teleporter"), "{refusals:?}");
}

#[test]
fn the_contract_types_carry_no_open_marker() {
    let text = std::fs::read_to_string(repo_root().join("blocks/9-composition/architecture.yaml"))
        .unwrap();
    let start = text.find("name: agentic.architecture.PortKind\n").unwrap();
    let end = text
        .find("name: agentic.architecture.RoutingInvariant\n")
        .unwrap();
    assert_eq!(text[start..end].matches("UNMAPPED").count(), 0);
}
