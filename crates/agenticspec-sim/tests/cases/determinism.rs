//! Same experiment and seeds give the same result, whatever the thread count.

use super::common::experiment;
use agenticspec_sim::run;

#[test]
fn two_runs_give_the_same_bytes() {
    let e = experiment("fit", 10, 256);
    let a = serde_json::to_string(&run(&e, 1).unwrap()).unwrap();
    let b = serde_json::to_string(&run(&e, 1).unwrap()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn one_and_four_threads_give_the_same_bytes() {
    let e = experiment("fit", 10, 256);
    let a = serde_json::to_string(&run(&e, 1).unwrap()).unwrap();
    let b = serde_json::to_string(&run(&e, 4).unwrap()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn the_failure_dynamics_and_a_reviewer_keep_one_and_four_threads_byte_equal() {
    use serde_json::json;
    let e = experiment("fit", 6, 256);
    let mut arch = e.architecture.clone();
    arch["parts"]["reviewer"] = json!({"block": "agent", "harness": "coding", "role": "reviewer",
        "count": {"value": 1, "vary": [1, 2]}, "strictness": "balanced"});
    let links = arch["links"].as_array_mut().unwrap();
    links.push(json!({"from": "worker", "to": "reviewer", "carries": "review"}));
    links.push(json!({"from": "reviewer", "to": "gateway", "carries": "model_call"}));
    let mut profile = e.profile.clone();
    let r = &mut profile["roles"]["coding"];
    r["step_turns"] = json!({"dist": "lognormal", "median": 3, "p90": 8});
    r["step_pass_after_pass"] = json!(0.702);
    r["step_pass_after_fail"] = json!(0.141);
    r["rework_turns"] = json!(2);
    r["default_wrong"] = json!(0.3);
    r["rework_share"] = json!(0.5);
    r["false_completion"] = json!(0.701);
    profile["prices"]["agent_hour_usd"] = json!(0.5);
    let e = agenticspec_sim::Experiment::from_docs(e.doc.clone(), arch, profile).unwrap();
    let a = serde_json::to_string(&run(&e, 1).unwrap()).unwrap();
    let b = serde_json::to_string(&run(&e, 4).unwrap()).unwrap();
    assert_eq!(a, b);
    assert!(a.contains("\"review_calls\""), "the reviewer ran");
}

#[test]
fn another_seed_gives_another_result() {
    let e = experiment("fit", 1, 256);
    let mut doc = e.doc.clone();
    doc["seed"] = serde_json::json!(8);
    let other =
        agenticspec_sim::Experiment::from_docs(doc, e.architecture.clone(), e.profile.clone())
            .unwrap();
    let a = run(&e, 2).unwrap();
    let b = run(&other, 2).unwrap();
    assert_ne!(a["baseline"]["kpis"], b["baseline"]["kpis"]);
}

#[test]
fn triggers_boards_staggers_and_partitions_keep_one_and_four_threads_byte_equal() {
    use super::common::yaml;
    use serde_json::json;
    let e = experiment("fit", 1, 256);
    let mut arch = yaml(super::swarm::SHARED);
    arch["parts"]["inbox"]["distribution"] = json!({"value": "board", "vary": ["queue", "board"]});
    arch["parts"]["team"]["start_stagger"] = json!({"value": "30s", "vary": ["0s", "30s"]});
    arch["parts"]["hourly"] = json!({"block": "schedule", "every": "1h"});
    arch["parts"]["wake"] = json!({"block": "trigger", "kind": "on_schedule", "schedule": "hourly",
        "policy": {"value": "skip", "vary": ["skip", "queue"]}, "task_template": "sweep"});
    arch["parts"]["sweeper"] = json!({"block": "agent", "harness": "coding", "count": 2,
        "partition": "scope", "coordination": "mesh"});
    let links = arch["links"].as_array_mut().unwrap();
    links.push(json!({"from": "wake", "to": "sweeper", "carries": "trigger"}));
    links.push(json!({"from": "sweeper", "to": "model", "carries": "model_call"}));
    let mut profile = super::swarm::profile();
    profile["coordination"]["scope_overlap"] = json!(0.3);
    let mut doc = e.doc.clone();
    doc["bursts"] = json!([{"at": "0s", "tasks": 12}]);
    doc["faults"] = json!([]);
    doc["search"]["mode"] = json!("exhaustive");
    doc["search"]["gates"] = json!(["unreachable_parts"]);
    let e = agenticspec_sim::Experiment::from_docs(doc, arch, profile).unwrap();
    let a = serde_json::to_string(&run(&e, 1).unwrap()).unwrap();
    let b = serde_json::to_string(&run(&e, 4).unwrap()).unwrap();
    assert_eq!(a, b);
    assert!(a.contains("\"claim_conflicts\""), "the board ran");
    let r: serde_json::Value = serde_json::from_str(&a).unwrap();
    assert_eq!(r["variants"].as_array().unwrap().len(), 8);
}
