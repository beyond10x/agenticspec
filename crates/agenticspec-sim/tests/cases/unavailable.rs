//! Unavailable parts and dependencies in scenarios: the reader, the check against the
//! architecture and the blocks, the faults and refusals on the simulator, and the observables a
//! runtime checks and the simulator reports not checked.

use super::common::{repo_root, scratch, yaml};
use agenticspec_sim::cli;
use agenticspec_sim::reader::sha256_hex;
use agenticspec_sim::unavailable::{SOURCES, names};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn example(name: &str) -> PathBuf {
    repo_root()
        .join("scenarios/workstation")
        .join(format!("{name}.scenario.yaml"))
}

fn doc_ref(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    format!(
        "{{ ref: {}, digest: {} }}",
        path.display(),
        sha256_hex(&bytes)
    )
}

fn write(dir: &Path, name: &str, horizon: &str, body: &str) -> PathBuf {
    let text = format!(
        "format: agenticspec-scenario/1\nscenario: {name}\narchitecture: {}\nprofile: {}\nseed: 3\nhorizon: {horizon}\n{body}",
        doc_ref(&repo_root().join("architectures/workstation.yaml")),
        doc_ref(&repo_root().join("fields/coding/profiles/coding.yaml")),
    );
    let path = dir.join(format!("{name}.scenario.yaml"));
    std::fs::write(&path, text).unwrap();
    path
}

fn run(path: &Path, out: &Path) -> Value {
    cli::scenario("run", path, Some(out)).unwrap_or_else(|e| panic!("{e:?}"));
    serde_json::from_str(&std::fs::read_to_string(out.join("scenario-result.json")).unwrap())
        .unwrap()
}

const PROMPT: &str = "  - { at: operator, kind: prompt, author: owner }\n";

const EXAMPLES: &[(&str, &str, &str)] = &[
    ("chat-closed-view-error", "view", "inputs_refused"),
    ("model-route-down-empty-view", "view", "fault"),
    ("operator-exits-without-gateway", "exits", "inputs_refused"),
    ("build-pool-cut-claim-refused", "command", "fault"),
];

#[test]
fn each_example_holds_on_the_simulator_and_leaves_its_observables_to_a_runtime() {
    for (name, first, simulated) in EXAMPLES {
        let out = scratch(&format!("unavailable-{name}"));
        let r = run(&example(name), &out);
        assert_eq!(r["verdict"], "held", "{name}");
        assert_eq!(r["unavailable"][0]["simulated"], *simulated, "{name}");
        let obs = r["observables"].as_array().unwrap();
        assert_eq!(obs[0]["observable"], *first, "{name}");
        assert!(
            obs.iter().all(|o| o["status"] == "not_checked"
                && o["reason"].as_str().unwrap().starts_with("the simulator")),
            "{name}: {obs:?}"
        );
        assert!(
            r["expectations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| e["status"] == "held"),
            "{name}: the events are checked on the simulator"
        );
    }
}

#[test]
fn the_examples_state_the_four_classes() {
    let read = |n: &str| yaml(&std::fs::read_to_string(example(n)).unwrap());
    let a = read("chat-closed-view-error");
    let op = &a["then"]["parts"]["operator"];
    assert_eq!(op["views"][0]["shows"], "error");
    assert_eq!(op["rest"], "answers");
    let b = read("model-route-down-empty-view");
    assert_eq!(b["then"]["parts"]["gateway"]["views"][0]["shows"], "empty");
    assert!(b["then"]["parts"]["gateway"]["views"][0]["error"].is_null());
    let c = read("operator-exits-without-gateway");
    assert_eq!(c["then"]["parts"]["operator"]["exits"], 1);
    assert_eq!(c["unavailable"][0]["from"], "0s");
    let d = read("build-pool-cut-claim-refused");
    let w = &d["then"]["parts"]["worker"];
    assert_eq!(w["commands"].as_array().unwrap().len(), 1);
    assert_eq!(w["commands"][0]["shows"], "refused");
    assert_eq!(w["rest"], "answers");
}

#[test]
fn the_same_unavailable_scenario_writes_the_same_bytes() {
    for (name, _, _) in EXAMPLES {
        let a = scratch(&format!("unavailable-same-a-{name}"));
        let b = scratch(&format!("unavailable-same-b-{name}"));
        run(&example(name), &a);
        run(&example(name), &b);
        assert_eq!(
            std::fs::read(a.join("scenario-result.json")).unwrap(),
            std::fs::read(b.join("scenario-result.json")).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn check_refuses_unknown_parts_dependencies_views_commands_and_errors_by_name() {
    let dir = scratch("unavailable-check");
    let cases: &[(&str, &str, &str)] = &[
        (
            "unknown-part",
            "unavailable:\n  - { part: store, from: 0s }\n",
            "unavailable.0.part: `store` is no part of the architecture",
        ),
        (
            "unknown-dependency",
            "unavailable:\n  - { part: worker, dependency: store, from: 0s }\n",
            "unavailable.0.dependency: `store` is no part of the architecture",
        ),
        (
            "not-reached",
            "unavailable:\n  - { part: builds, dependency: model, from: 0s }\n",
            "unavailable.0.dependency: `model` is not reached from `builds` over a link",
        ),
        (
            "itself",
            "unavailable:\n  - { part: worker, dependency: worker, from: 0s }\n",
            "unavailable.0.dependency: `worker` is the part itself",
        ),
        (
            "after-horizon",
            "unavailable:\n  - { part: builds, from: 3h }\n",
            "unavailable.0.from: not before the horizon",
        ),
        (
            "parts-unknown",
            "then:\n  parts:\n    store: { rest: answers }\n",
            "then.parts.store: `store` is no part of the architecture",
        ),
        (
            "parts-unavailable",
            "unavailable:\n  - { part: builds, from: 0s }\nthen:\n  parts:\n    builds: { rest: answers }\n",
            "then.parts.builds: `builds` is unavailable itself",
        ),
        (
            "unknown-view",
            "then:\n  parts:\n    gateway:\n      views:\n        - { view: agentic.gateway.Routes, shows: empty }\n",
            "then.parts.gateway.views.0.view: `agentic.gateway.Routes` is no view of a block",
        ),
        (
            "foreign-view",
            "then:\n  parts:\n    builds:\n      views:\n        - { view: agentic.gateway.Calls, shows: empty }\n",
            "then.parts.builds.views.0.view: `agentic.gateway.Calls` is a view of agentic.gateway, not of `builds` (block pool)",
        ),
        (
            "unknown-view-error",
            "then:\n  parts:\n    gateway:\n      views:\n        - { view: agentic.gateway.Calls, shows: error, error: agentic.gateway.Down }\n",
            "then.parts.gateway.views.0.error: `agentic.gateway.Down` is no error of a block",
        ),
        (
            "unknown-command",
            "then:\n  parts:\n    worker:\n      commands:\n        - { command: agentic.resource.Claim, shows: refused }\n",
            "then.parts.worker.commands.0.command: `agentic.resource.Claim` is no command of a block",
        ),
        (
            "foreign-command",
            "then:\n  parts:\n    gateway:\n      commands:\n        - { command: agentic.resource.RequestClaim, shows: refused }\n",
            "then.parts.gateway.commands.0.command: `agentic.resource.RequestClaim` is a command of agentic.resource, not of `gateway` (block gateway)",
        ),
        (
            "unknown-command-error",
            "then:\n  parts:\n    worker:\n      commands:\n        - { command: agentic.resource.RequestClaim, shows: refused, error: agentic.resource.Unreachable }\n",
            "then.parts.worker.commands.0.error: `agentic.resource.Unreachable` is no error of a block",
        ),
        (
            "exit-without-startup-dependency",
            "unavailable:\n  - { part: operator, dependency: gateway, from: 10m }\nthen:\n  parts:\n    operator: { exits: 1 }\n",
            "then.parts.operator.exits: a part exits before it listens only when a dependency of it is unavailable from 0s",
        ),
    ];
    for (name, body, want) in cases {
        let path = write(&dir, name, "2h", &format!("when:\n{PROMPT}{body}"));
        let err = cli::scenario("check", &path, None).unwrap_err();
        assert_eq!(err.code, "scenario_refused", "{name}: {}", err.detail);
        assert!(
            err.detail.contains(want),
            "{name}: {want} not in {}",
            err.detail
        );
    }
}

#[test]
fn the_reader_refuses_observables_that_do_not_fit_together() {
    let dir = scratch("unavailable-reader");
    let path = write(
        &dir,
        "shapes",
        "2h",
        &format!(
            "unavailable:\n  - {{ part: builds, since: 0s }}\n  - {{ part: chat, from: 0s, for: 0s }}\nwhen:\n{PROMPT}then:\n  parts:\n    operator:\n      views:\n        - {{ view: agentic.channel.Channels, shows: answer, error: agentic.channel.ChannelIsClosed }}\n        - {{ view: agentic.channel.Channels, shows: blank }}\n      rest: everything\n    worker:\n      exits: 1\n      views:\n        - {{ view: agentic.work.Runs, shows: empty }}\n    gateway:\n      exits: 0\n    tree: {{}}\n"
        ),
    );
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "scenario_invalid", "{}", err.detail);
    for p in [
        "unavailable.0.since: unknown key",
        "unavailable.0.from: missing",
        "unavailable.1.for: must be longer than 0s",
        "then.parts.operator.views.0.error: only a view that shows error names one",
        "then.parts.operator.views.1.shows:",
        "then.parts.operator.views.1.view: `agentic.channel.Channels` is named twice",
        "then.parts.operator.rest:",
        "then.parts.worker: a part that exits before it listens shows no view",
        "then.parts.gateway.exits: an exit status from 1 to 255",
        "then.parts.tree: names views, commands, exits or rest",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }
}

#[test]
fn an_unavailable_principal_is_away_and_its_question_takes_the_default() {
    let dir = scratch("unavailable-principal");
    let path = write(
        &dir,
        "owner-away",
        "2h",
        &format!(
            "unavailable:\n  - {{ part: owner, from: 1s }}\nwhen:\n{PROMPT}fixtures:\n  model:\n    - {{ for: worker, outcome: question, turns: 2, options: [A, B], default: A, needs_owner: true }}\n  principals:\n    - {{ part: owner, option: B, after: 1m }}\nthen:\n  events:\n    - {{ event: question_asked, part: operator, task: when.0 }}\n    - {{ event: default_taken, task: when.0, option: A }}\n    - {{ event: question_answered, absent: true }}\n"
        ),
    );
    let r = run(&path, &dir.join("out"));
    assert_eq!(r["verdict"], "held", "{r}");
    assert_eq!(r["unavailable"][0]["fault"]["kind"], "principal_away");
}

#[test]
fn an_entry_the_simulator_cannot_model_leaves_every_expectation_not_checked() {
    let dir = scratch("unavailable-not-modeled");
    let path = write(
        &dir,
        "worker-down",
        "1h",
        &format!(
            "unavailable:\n  - {{ part: worker, from: 0s }}\nwhen:\n{PROMPT}then:\n  events:\n    - {{ event: run_started, absent: true }}\n  kpis:\n    tasks_done: {{ max: 0 }}\n"
        ),
    );
    let r = run(&path, &dir.join("out"));
    assert_eq!(r["unavailable"][0]["simulated"], "not_modeled");
    assert_eq!(r["counts"]["not_checked"], 2);
    let why = r["expectations"][0]["reason"].as_str().unwrap();
    assert!(why.contains("does not model unavailable.0"), "{why}");
    assert_eq!(r["kpi_bounds"][0]["status"], "not_checked");
    assert!(
        r["kpi_bounds"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("worker")
    );
}

#[test]
fn a_down_channel_the_simulation_still_uses_is_reported_not_modeled() {
    let dir = scratch("unavailable-broken");
    let path = write(
        &dir,
        "chat-used",
        "2h",
        &format!(
            "unavailable:\n  - {{ part: chat, from: 1s }}\nwhen:\n{PROMPT}fixtures:\n  model:\n    - {{ for: worker, outcome: question, turns: 2, options: [A, B], default: A, needs_owner: true }}\nthen:\n  events:\n    - {{ event: task_created, task: when.0 }}\n"
        ),
    );
    let r = run(&path, &dir.join("out"));
    assert_eq!(r["unavailable"][0]["simulated"], "not_modeled", "{r}");
    assert!(
        r["unavailable"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("post_sent by `operator`")
    );
    assert_eq!(r["expectations"][0]["status"], "not_checked");
}

#[test]
fn an_input_over_a_cut_channel_is_refused_while_the_cut_lasts() {
    let dir = scratch("unavailable-window");
    let path = write(
        &dir,
        "window",
        "2h",
        &format!(
            "unavailable:\n  - {{ part: operator, dependency: chat, from: 10m, for: 20m }}\nwhen:\n{PROMPT}  - {{ at: operator, kind: prompt, author: owner, after: 15m }}\n  - {{ at: operator, kind: prompt, author: owner, after: 20m }}\nfixtures:\n  model:\n    - {{ for: worker, outcome: correct, turns: 2 }}\n    - {{ for: worker, outcome: correct, turns: 2 }}\n"
        ),
    );
    let r = run(&path, &dir.join("out"));
    let rules: Vec<Value> = r["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["rule"].clone())
        .collect();
    assert_eq!(
        rules,
        vec![Value::Null, "part_unavailable".into(), Value::Null],
        "{r}"
    );
    assert_eq!(r["unavailable"][0]["simulated"], "inputs_refused");
    assert_eq!(r["unavailable"][0]["until_seconds"], 1800.0);
}

#[test]
fn the_declared_names_cover_every_specification_file_of_the_system() {
    let inputs = yaml(&std::fs::read_to_string(repo_root().join("ess-inputs.yaml")).unwrap());
    let listed: Vec<&str> = SOURCES.iter().map(|(p, _)| *p).collect();
    for f in inputs["specification"].as_array().unwrap() {
        let f = f.as_str().unwrap();
        if f.ends_with("system.yaml") {
            continue;
        }
        assert!(listed.contains(&f), "{f} is not in unavailable::SOURCES");
        let (_, text) = SOURCES.iter().find(|(p, _)| *p == f).unwrap();
        assert_eq!(
            *text,
            std::fs::read_to_string(repo_root().join(f)).unwrap(),
            "{f}"
        );
    }
    let n = names();
    assert_eq!(
        n.views.get("agentic.resource.Queue").map(String::as_str),
        Some("agentic.resource")
    );
    assert_eq!(
        n.commands
            .get("agentic.scm.change.MergeChangeRequest")
            .map(String::as_str),
        Some("agentic.scm.change")
    );
    assert!(n.errors.contains("agentic.channel.ChannelIsClosed"));
}
