//! `agenticspec sim contract --suite`: the blocks' conformance suite cut to the scenarios about a
//! command of a port of one contract, on a small suite whose ids have the shapes ESS writes.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, ContractOptions};
use agenticspec_sim::contract::derive::{self, Contract};
use agenticspec_sim::contract::ports::PortsFile;
use agenticspec_sim::contract::suite::{self, about};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn ports() -> PortsFile {
    PortsFile::load(&repo_root().join("blocks/ports.json")).unwrap()
}

fn contract_of(path: &str) -> Contract {
    let doc = agenticspec_sim::doc::load_yaml(&repo_root().join(path)).unwrap();
    derive::derive(&doc, &ports()).unwrap()
}

fn small_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/suite/small.json")
}

fn small() -> Value {
    serde_json::from_str(&std::fs::read_to_string(small_path()).unwrap()).unwrap()
}

fn kept(path: &str) -> BTreeSet<String> {
    let p = ports();
    let commands = suite::commands(&contract_of(path), &p);
    let cut = suite::cut(&small(), &commands, &p.bindings).unwrap();
    assert_eq!(cut.total, 15);
    cut.kept.into_iter().collect()
}

#[test]
fn a_scenario_is_about_the_command_its_id_names_or_its_last_command() {
    let bindings: BTreeMap<String, String> = [("b".to_owned(), "x.y.Z".to_owned())].into();
    let none = json!({});
    let steps = json!({"steps": [
        {"step": "execute_command", "command": "x.y.First"},
        {"step": "expect_outcome"},
        {"step": "execute_command", "command": "x.y.Last"},
        {"step": "expect_outcome"},
    ]});
    for (id, want) in [
        ("x.y.Do/outcome/done", Some("x.y.Do")),
        ("x.y.Do/outcome/taken/held/deadline/0", Some("x.y.Do")),
        ("x.y.E/transition/t/by/x.y.Do/done", Some("x.y.Do")),
        ("x.y.E/state/S/refuses/x.y.Do", Some("x.y.Do")),
        ("x.y.E/invariant/after/x.y.Do/done", Some("x.y.Do")),
        ("x.y.E/invariant-wrapped/after/x.y.Do/done", Some("x.y.Do")),
        ("b/binding/mapping", Some("x.y.Z")),
        ("unknown/binding/mapping", None),
        ("x.y/authored/case", None),
    ] {
        assert_eq!(about(id, &none, &bindings).as_deref(), want, "{id}");
    }
    assert_eq!(
        about("x.y/authored/case", &steps, &bindings).as_deref(),
        Some("x.y.Last")
    );
    assert_eq!(
        about("x.T/invariant/at/x.y.View/field", &steps, &bindings).as_deref(),
        Some("x.y.Last")
    );
}

#[test]
fn the_workstation_keeps_inference_claim_question_and_session_scenarios_only() {
    let kept = kept("architectures/workstation.yaml");
    let want: BTreeSet<String> = [
        "agentic.model.FinishInference/outcome/finished",
        "agentic.model.Inference/transition/finish/by/agentic.model.FinishInference/finished",
        "agentic.question.Question/invariant/after/agentic.question.Answer/answered",
        "start-inference-on-admitted-call/binding/mapping",
        "agentic.resource/authored/claim-withdrawn-from-queue",
        "agentic.shared.Usage/invariant/at/agentic.harness.Turns/usage",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(kept, want);
    for gone in [
        "StartInstance",
        "FireTrigger",
        "PostMessage",
        "TakePost",
        "CommitRevision",
    ] {
        assert!(!kept.iter().any(|id| id.contains(gone)), "{gone}");
    }
}

#[test]
fn scheduled_loops_keep_every_scenario_about_the_clock_port() {
    let kept = kept("architectures/scheduled-loops.yaml");
    let p = ports();
    let clock = p.port("trigger.schedule").unwrap();
    let bindings = &p.bindings;
    let suite = small();
    let about_clock: Vec<&String> = suite["scenarios"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(id, s)| {
            about(id, s, bindings).is_some_and(|c| {
                clock
                    .commands()
                    .iter()
                    .any(|n| c == format!("agentic.trigger.{n}"))
            })
        })
        .map(|(id, _)| id)
        .collect();
    assert_eq!(about_clock.len(), 2);
    for id in about_clock {
        assert!(kept.contains(id), "{id}");
    }
    assert!(kept.contains("agentic.trigger.FireTrigger/outcome/fired"));
}

#[test]
fn a_document_that_is_no_suite_is_refused() {
    let p = ports();
    let commands = suite::commands(&contract_of("architectures/solo.yaml"), &p);
    let err = suite::cut(&json!({"scenarios": {}}), &commands, &p.bindings).unwrap_err();
    assert!(err.contains("not a conformance suite"), "{err}");
}

#[test]
fn the_command_writes_the_cut_suite_with_the_suites_provenance_and_counts() {
    let root = repo_root();
    let out = scratch("contract-suite").join("workstation.suite.json");
    let answer = cli::contract(
        &ContractOptions {
            architecture: &root.join("architectures/workstation.yaml"),
            ports: Some(&root.join("blocks/ports.json")),
            out: Some(&out),
            check: None,
            suite: Some(&small_path()),
            trace: None,
            kpis: Some(&repo_root().join("kpis/catalogue.yaml")),
        },
        &mut std::io::sink(),
    )
    .unwrap();
    assert_eq!(answer["suite_scenarios"], 15);
    assert_eq!(answer["kept_scenarios"], 6);
    assert_eq!(answer["written"], true);
    assert!(answer.get("contract").is_none());
    let cut: Value = serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    let whole = small();
    assert_eq!(cut["provenance"], whole["provenance"]);
    assert_eq!(cut["events"], whole["events"]);
    assert_eq!(cut["scenarios"].as_object().unwrap().len(), 6);
    let id = "agentic.resource/authored/claim-withdrawn-from-queue";
    assert_eq!(
        cut["scenarios"][id], whole["scenarios"][id],
        "setup steps kept"
    );

    let err = cli::contract(
        &ContractOptions {
            architecture: &root.join("architectures/workstation.yaml"),
            ports: Some(&root.join("blocks/ports.json")),
            out: None,
            check: None,
            suite: Some(&root.join("architectures/workstation.yaml")),
            trace: None,
            kpis: Some(&repo_root().join("kpis/catalogue.yaml")),
        },
        &mut std::io::sink(),
    )
    .unwrap_err();
    assert_eq!(err.code, "suite_unreadable");
}
