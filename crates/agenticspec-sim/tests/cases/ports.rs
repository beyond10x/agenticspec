//! `agenticspec sim ports`: the classification of commands and events, the derived ports, the
//! committed `blocks/ports.json`, its digest check and the comparison with an expected list.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, PortsOptions};
use agenticspec_sim::contract::ports::{
    self, Class, Expected, Kind, Port, PortsFile, compare, derive,
};
use agenticspec_sim::contract::protocol::Protocol;
use serde_json::Value;
use std::path::Path;

fn small() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/ports/small.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn a_command_a_binding_invokes_is_internal_and_the_deployers_is_configuration() {
    let d = derive(&small()).unwrap();
    assert_eq!(d.commands["small.job.Start"], Class::Internal);
    assert_eq!(d.commands["small.registry.Retire"], Class::Configuration);
    assert_eq!(d.commands["small.job.Finish"], Class::Entry);
    assert_eq!(d.commands["small.ask.Answer"], Class::Entry);
    assert!(d.events["small.job.Requested"], "consumed by the binding");
    assert!(!d.events["small.job.Started"], "an exit");
}

#[test]
fn an_exit_that_waits_on_an_entry_is_an_outbound_port() {
    let d = derive(&small()).unwrap();
    let job = d.ports.iter().find(|p| p.name == "job.job").unwrap();
    assert_eq!(job.kind, Kind::Outbound);
    assert_eq!(job.block, "job");
    assert_eq!(job.intent_event.as_deref(), Some("small.job.Started"));
    assert_eq!(job.waiting_state.as_deref(), Some("small.job.Job.Started"));
    assert_eq!(job.reports, ["small.job.Finish", "small.job.RecordStep"]);
    assert_eq!(job.protocol.to_string(), "Start RecordStep* Finish");
}

#[test]
fn a_wait_a_principal_ends_is_a_principal_port() {
    let d = derive(&small()).unwrap();
    let q = d.ports.iter().find(|p| p.name == "ask.question").unwrap();
    assert_eq!(q.kind, Kind::Principal);
    assert_eq!(q.protocol.to_string(), "Ask Answer");
}

#[test]
fn a_state_only_the_deployer_moves_on_is_configuration_and_no_port() {
    let d = derive(&small()).unwrap();
    let names: Vec<&str> = d.ports.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["ask.question", "job.job"]);
}

#[test]
fn a_binding_that_moves_the_waiting_entity_takes_the_wait_away() {
    let mut ir = small();
    ir["bindings"]["finish-job"] =
        serde_json::json!({ "event": "small.job.Started", "command": "small.job.Finish" });
    let d = derive(&ir).unwrap();
    assert!(d.ports.iter().all(|p| p.name != "job.job"), "{:?}", d.ports);
}

#[test]
fn a_clock_reading_attachment_is_refused_rather_than_missed() {
    let mut ir = small();
    ir["types"] = serde_json::json!({ "small.job.Reading": { "body": { "reading": {} } } });
    let refusals = derive(&ir).unwrap_err();
    assert!(refusals[0].contains("clock-reading"), "{refusals:?}");
}

#[test]
fn a_protocol_prints_parses_and_matches_its_traces() {
    let p = Protocol::parse("SendMessage (MarkLost | MarkDelivered MarkRead?)").unwrap();
    assert_eq!(
        p.to_string(),
        "SendMessage (MarkDelivered MarkRead? | MarkLost)"
    );
    assert_eq!(Protocol::parse(&p.to_string()).unwrap(), p);
    assert!(p.accepts(&["SendMessage", "MarkLost"]).unwrap());
    assert!(p.accepts(&["SendMessage", "MarkDelivered"]).unwrap());
    assert!(!p.accepts(&["SendMessage"]).unwrap());
    assert!(!p.accepts(&["MarkLost"]).unwrap());
    let e = Protocol::parse("Ask (Answer | )").unwrap_err();
    assert!(e.contains("offset 14"), "{e}");
    let e = Protocol::parse("Ask (Answer").unwrap_err();
    assert!(e.contains("expected `)` at offset 11"), "{e}");
}

fn committed() -> PortsFile {
    PortsFile::load(&repo_root().join("blocks/ports.json")).unwrap()
}

#[test]
fn the_committed_ports_hold_the_model_port() {
    let file = committed();
    let model = file
        .ports
        .iter()
        .find(|p| {
            p.block == "model"
                && p.intent_event.as_deref() == Some("agentic.model.InferenceStarted")
        })
        .unwrap();
    assert_eq!(model.kind, Kind::Outbound);
    assert_eq!(
        model.waiting_state.as_deref(),
        Some("agentic.model.Inference.Started")
    );
    assert_eq!(model.reports, ["agentic.model.FinishInference"]);
    assert_eq!(model.protocol.to_string(), "StartInference FinishInference");
}

#[test]
fn no_committed_port_reports_a_configuration_command() {
    for p in committed().ports {
        for c in ["RegisterPrincipal", "DefineAgent", "DefineKpi"] {
            assert!(
                p.reports.iter().all(|r| !r.ends_with(&format!(".{c}"))),
                "{} reports {c}",
                p.name
            );
        }
    }
}

#[test]
fn every_committed_protocol_names_only_its_own_commands() {
    for p in committed().ports {
        let own: std::collections::BTreeSet<String> = p
            .reports
            .iter()
            .map(|r| r.rsplit('.').next().unwrap().to_owned())
            .chain(p.protocol.first())
            .collect();
        assert!(
            p.protocol.names().is_subset(&own),
            "{}: {}",
            p.name,
            p.protocol
        );
        p.protocol.compile().unwrap();
    }
}

#[test]
fn the_check_passes_on_the_committed_file_without_ess() {
    let root = repo_root();
    let checked = cli::ports(
        &PortsOptions {
            blocks: Some(&root.join("blocks")),
            check: Some(&root.join("blocks/ports.json")),
            ..PortsOptions::default()
        },
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(checked["written"], false);
    assert_eq!(
        checked["ports"].as_u64(),
        Some(committed().ports.len() as u64)
    );
}

#[test]
fn the_check_refuses_a_file_whose_blocks_changed_and_names_the_task() {
    let root = repo_root();
    let dir = scratch("ports-stale");
    let blocks = dir.join("blocks");
    for file in ports::sources(&root.join("blocks")).unwrap() {
        let to = blocks.join(&file);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(root.join("blocks").join(&file), &to).unwrap();
    }
    let options = PortsOptions {
        blocks: Some(&blocks),
        check: Some(&root.join("blocks/ports.json")),
        ..PortsOptions::default()
    };
    cli::ports(&options, &mut Vec::new()).unwrap();
    let model = blocks.join("2-intelligence/model.yaml");
    let text = std::fs::read_to_string(&model).unwrap();
    std::fs::write(&model, format!("{text}\n# changed\n")).unwrap();
    let err = cli::ports(&options, &mut Vec::new()).unwrap_err();
    assert_eq!(err.code, "ports_stale");
    assert!(err.detail.contains("task ports:generate"), "{}", err.detail);
}

fn port(name: &str, block: &str, protocol: &str) -> Port {
    let protocol = Protocol::parse(protocol).unwrap();
    let first = protocol.first();
    Port {
        name: name.into(),
        block: block.into(),
        kind: Kind::Outbound,
        intent_event: None,
        waiting_state: None,
        reports: protocol
            .names()
            .difference(&first)
            .map(|c| format!("agentic.{block}.{c}"))
            .collect(),
        protocol,
    }
}

fn row(name: &str, block: &str, protocol: &str) -> Expected {
    Expected {
        port: name.into(),
        block: block.into(),
        kind: None,
        protocol: Protocol::parse(protocol).unwrap(),
    }
}

#[test]
fn the_comparison_names_each_difference_and_nothing_else() {
    let derived = [
        port("model.inference", "model", "StartInference FinishInference"),
        port(
            "channel.message",
            "channel",
            "SendMessage (MarkDelivered MarkRead | MarkLost)",
        ),
        port("work.task", "work", "CreateTask CompleteTask"),
    ];
    let expected = [
        row("model.infer", "model", "StartInference FinishInference"),
        row(
            "channel.deliver",
            "channel",
            "SendMessage (MarkDelivered MarkRead? | MarkLost)",
        ),
        row("clock.fire", "trigger", "FireTrigger SkipFiring"),
    ];
    let lines = compare(&derived, &expected);
    assert_eq!(
        lines,
        [
            "protocol channel.deliver -> channel.message: expected `SendMessage (MarkDelivered MarkRead? | MarkLost)`, derived `SendMessage (MarkDelivered MarkRead | MarkLost)`",
            "missing clock.fire: no derived port has any of its commands (FireTrigger, SkipFiring)",
            "extra work.task (Outbound): `CreateTask CompleteTask`",
        ]
    );
    assert!(compare(&derived[..1], &expected[..1]).is_empty());
}

#[test]
fn the_expected_list_is_the_designs_fifteen_rows() {
    let rows = ports::load_expected(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/ports-expected.yaml"),
    )
    .unwrap();
    assert_eq!(rows.len(), 15);
    assert_eq!(rows[0].port, "model.infer");
    assert_eq!(rows[14].port, "pool.lease");
}

#[test]
fn a_terminal_state_with_a_way_out_ends_the_protocol_or_goes_on() {
    let mut ir = small();
    let job = &mut ir["entities"]["small.job.Job"]["lifecycle"];
    job["states"] = serde_json::json!(["Archived", "Done", "Started"]);
    job["terminal"] = serde_json::json!(["Archived", "Done"]);
    ir["commands"]["small.job.Archive"] = serde_json::json!({
        "input": [],
        "outcomes": [{ "name": "archived", "condition": { "kind": "otherwise" },
            "subject": { "entity": "small.job.Job", "effect": "moves",
                         "transition": { "name": "archive", "from": ["Done"], "to": "Archived" } },
            "emits": [], "error": null }]
    });
    let d = derive(&ir).unwrap();
    let job = d.ports.iter().find(|p| p.name == "job.job").unwrap();
    assert_eq!(
        job.protocol.to_string(),
        "Start RecordStep* Finish Archive?"
    );
}

#[test]
fn every_binding_of_the_blocks_is_compiled_and_its_command_is_no_entry() {
    let components = std::fs::read_to_string(repo_root().join("blocks/components.yaml")).unwrap();
    let ids: Vec<&str> = components
        .lines()
        .filter_map(|l| l.strip_prefix("  - id: "))
        .map(str::trim)
        .collect();
    assert_eq!(ids.len(), 4, "{ids:?}");
    let file = committed();
    let bound: Vec<&str> = file.bindings.keys().map(String::as_str).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(
        bound, sorted,
        "the compiled bindings are those of blocks/components.yaml"
    );
    // The derivation classifies a command a binding invokes as internal (the first test of this
    // file); the file records which commands those are.
    assert_eq!(
        file.bindings["start-inference-on-admitted-call"],
        "agentic.model.StartInference"
    );
}

#[test]
fn the_committed_channel_ports_may_rest_delivered_and_posted() {
    let file = committed();
    let message = file.port("channel.message").unwrap();
    assert_eq!(
        message.protocol.to_string(),
        "SendMessage (MarkDelivered MarkRead? | MarkLost)"
    );
    let post = file.port("channel.post").unwrap();
    assert_eq!(post.protocol.to_string(), "PostMessage TakePost?");
}
