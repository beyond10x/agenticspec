//! Load as one backlog, the cost of a live agent, and the failure dynamics: steps, wrong
//! assumptions, false completion claims and review.

use super::common::{experiment, repo_root};
use agenticspec_sim::{Experiment, run};
use serde_json::{Value, json};

fn baseline(
    arch_edit: impl Fn(&mut Value),
    profile_edit: impl Fn(&mut Value),
    exp_edit: impl Fn(&mut Value),
) -> Value {
    let e = experiment("fit", 1, 256);
    let mut arch = e.architecture.clone();
    arch_edit(&mut arch);
    let mut profile = e.profile.clone();
    profile_edit(&mut profile);
    let mut doc = e.doc.clone();
    exp_edit(&mut doc);
    let e = Experiment::from_docs(doc, arch, profile).unwrap();
    run(&e, 2).unwrap()["baseline"].clone()
}

fn k(b: &Value, name: &str) -> f64 {
    b["kpis"][name]["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("KPI {name} missing"))
}

fn part(b: &Value, part: &str, name: &str) -> f64 {
    b["per_part"][part][name]["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("{part}.{name} missing"))
}

/// The step chain of 2611.07166 with a few turns per step and rework after a failed step.
fn chain(p: &mut Value) {
    let r = &mut p["roles"]["coding"];
    r["step_turns"] = json!({ "dist": "lognormal", "median": 3, "p90": 8 });
    r["step_pass_after_pass"] = json!(0.702);
    r["step_pass_after_fail"] = json!(0.141);
    r["rework_turns"] = json!({ "dist": "lognormal", "median": 2, "p90": 6 });
}

fn with_reviewer(a: &mut Value, false_accept: f64, false_reject: f64) {
    a["parts"]["reviewer"] = json!({
        "block": "agent", "harness": "coding", "role": "reviewer",
        "false_accept": false_accept, "false_reject": false_reject
    });
    let links = a["links"].as_array_mut().unwrap();
    links.push(json!({ "from": "worker", "to": "reviewer", "carries": "review" }));
    links.push(json!({ "from": "reviewer", "to": "gateway", "carries": "model_call" }));
}

fn none(_: &mut Value) {}

#[test]
fn step_pass_rates_converge_to_the_configured_values() {
    let b = baseline(none, chain, |d| {
        d["seeds"] = json!(24);
        d["horizon"] = json!("8h");
    });
    let after_pass =
        part(&b, "worker", "steps_passed_after_pass") / part(&b, "worker", "steps_after_pass");
    let after_fail =
        part(&b, "worker", "steps_passed_after_fail") / part(&b, "worker", "steps_after_fail");
    assert!(
        (after_pass - 0.702).abs() < 0.03,
        "pass after pass {after_pass}"
    );
    assert!(
        (after_fail - 0.141).abs() < 0.03,
        "pass after fail {after_fail}"
    );
    assert!(k(&b, "steps_failed") > 0.0 && k(&b, "steps") > k(&b, "steps_failed"));
    assert!(
        k(&b, "rework_turns") > 0.0,
        "a failed step adds rework turns"
    );
}

#[test]
fn a_defaulted_question_with_default_wrong_1_always_causes_rework() {
    let b = baseline(
        |a| a["policies"]["question_default_after"] = json!("1s"),
        |p| {
            p["roles"]["coding"]["default_wrong"] = json!(1.0);
            p["roles"]["coding"]["rework_share"] = json!(0.5);
        },
        none,
    );
    assert!(k(&b, "questions_defaulted") > 0.0);
    assert_eq!(k(&b, "wrong_assumptions"), k(&b, "questions_defaulted"));
    assert!(k(&b, "rework_turns") > 0.0);
}

#[test]
fn a_defaulted_question_with_default_wrong_0_never_causes_rework() {
    let b = baseline(
        |a| a["policies"]["question_default_after"] = json!("1s"),
        |p| {
            p["roles"]["coding"]["default_wrong"] = json!(0.0);
            p["roles"]["coding"]["rework_share"] = json!(0.5);
        },
        none,
    );
    assert!(k(&b, "questions_defaulted") > 0.0);
    assert_eq!(k(&b, "wrong_assumptions"), 0.0);
    assert_eq!(k(&b, "rework_turns"), 0.0);
}

#[test]
fn an_answered_question_causes_no_rework() {
    let b = baseline(
        |a| a["policies"]["question_default_after"] = json!("24h"),
        |p| {
            p["roles"]["coding"]["default_wrong"] = json!(1.0);
            p["roles"]["coding"]["rework_share"] = json!(0.5);
        },
        none,
    );
    assert!(k(&b, "questions_asked") > 0.0);
    assert_eq!(k(&b, "questions_defaulted"), 0.0);
    assert_eq!(k(&b, "wrong_assumptions"), 0.0);
    assert_eq!(k(&b, "rework_turns"), 0.0);
}

#[test]
fn without_a_verifier_false_completions_ship() {
    let b = baseline(
        none,
        |p| {
            chain(p);
            p["roles"]["coding"]["false_completion"] = json!(1.0);
        },
        none,
    );
    assert!(k(&b, "false_completions_shipped") > 0.0);
    assert_eq!(k(&b, "review_calls"), 0.0);
    assert!(
        (k(&b, "tasks_done") - k(&b, "tasks_correct") - k(&b, "false_completions_shipped")).abs()
            < 1e-9
    );
    assert_eq!(k(&b, "tasks_claimed_done"), k(&b, "tasks_done"));
}

#[test]
fn with_a_perfect_verifier_no_false_completion_ships() {
    let b = baseline(
        |a| with_reviewer(a, 0.0, 0.0),
        |p| {
            chain(p);
            p["roles"]["coding"]["false_completion"] = json!(1.0);
        },
        none,
    );
    assert_eq!(k(&b, "false_completions_shipped"), 0.0);
    assert!(k(&b, "tasks_correct") > 0.0);
    assert!(k(&b, "review_calls") > 0.0);
    assert!(k(&b, "reviews_rejected") > 0.0, "wrong tasks went back");
    assert!(k(&b, "tasks_claimed_done") > k(&b, "tasks_done"));
    assert!(part(&b, "reviewer", "reviews") > 0.0);
}

#[test]
fn honest_workers_without_a_verifier_ship_no_false_completion() {
    let b = baseline(
        none,
        |p| {
            chain(p);
            p["roles"]["coding"]["false_completion"] = json!(0.0);
        },
        none,
    );
    assert_eq!(k(&b, "false_completions_shipped"), 0.0);
    assert!(k(&b, "tasks_correct") > 0.0);
}

#[test]
fn a_reviewer_strictness_is_read_from_the_profile_and_an_unknown_one_is_refused() {
    let e = experiment("fit", 1, 256);
    let mut arch = e.architecture.clone();
    with_reviewer(&mut arch, 0.0, 0.0);
    let r = arch["parts"]["reviewer"].as_object_mut().unwrap();
    r.remove("false_accept");
    r.remove("false_reject");
    r.insert("strictness".into(), json!("pedantic"));
    let err = Experiment::from_docs(e.doc.clone(), arch.clone(), e.profile.clone())
        .expect_err("unknown strictness is refused");
    assert!(err.contains("pedantic"), "{err}");
    arch["parts"]["reviewer"]["strictness"] = json!("strict");
    Experiment::from_docs(e.doc.clone(), arch, e.profile.clone()).expect("strict is built in");
}

#[test]
fn tasks_arrive_for_the_whole_architecture_whatever_the_worker_count() {
    let few = baseline(|a| a["parts"]["worker"]["count"] = json!(1), none, none);
    let many = baseline(|a| a["parts"]["worker"]["count"] = json!(6), none, none);
    assert_eq!(k(&few, "tasks_arrived"), k(&many, "tasks_arrived"));
    assert!(k(&few, "backlog_wait_p90_seconds") > k(&many, "backlog_wait_p90_seconds"));
}

#[test]
fn the_old_per_worker_load_still_reads() {
    let new = baseline(none, none, none);
    let old = baseline(
        none,
        |p| p["load"] = json!({ "tasks_per_hour_per_worker": 2 }),
        none,
    );
    assert_eq!(k(&new, "tasks_arrived"), k(&old, "tasks_arrived"));
    let e = experiment("fit", 1, 256);
    let mut profile = e.profile.clone();
    profile["load"] = json!({ "tasks_per_hour": 4, "tasks_per_hour_per_worker": 2 });
    assert!(Experiment::from_docs(e.doc.clone(), e.architecture.clone(), profile).is_err());
}

#[test]
fn a_live_agent_costs_its_hours() {
    let free = baseline(none, none, none);
    let paid = baseline(none, |p| p["prices"]["agent_hour_usd"] = json!(1.0), none);
    // Two workers and the operator, alive for the 4 h horizon.
    assert_eq!(k(&free, "agent_time_cost_usd"), 0.0);
    assert!((k(&paid, "agent_time_cost_usd") - 12.0).abs() < 1e-9);
    assert!((k(&paid, "cost_usd") - k(&free, "cost_usd") - 12.0).abs() < 1e-6);
}

#[test]
fn the_new_kpis_carry_their_kinds() {
    let b = baseline(|a| with_reviewer(a, 0.2, 0.2), chain, none);
    for (name, kind) in [
        ("tasks_correct", "Quality"),
        ("tasks_claimed_done", "Quality"),
        ("false_completions_shipped", "Correctness"),
        ("rework_turns", "Quality"),
        ("review_calls", "Quality"),
        ("backlog_wait_p90_seconds", "Quality"),
    ] {
        assert_eq!(b["kpis"][name]["kind"], kind, "{name}");
    }
}

#[test]
fn the_experiments_load_and_reward_correct_work() {
    let ws = Experiment::load(&repo_root().join("experiments/workstation-tuning.yaml")).unwrap();
    let w = &ws.search.weights;
    assert!(w["tasks_correct"] > 0.0);
    assert!(w["false_completions_shipped"] < 0.0);
    assert!(!w.contains_key("tasks_done"));
    let rv = Experiment::load(&repo_root().join("experiments/reviewer-tuning.yaml")).unwrap();
    let paths: Vec<&str> = rv.marked.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"parts.worker.count"), "{paths:?}");
    let strictness = rv
        .marked
        .iter()
        .find(|m| m.path == "parts.reviewer.strictness")
        .expect("strictness is marked");
    assert_eq!(
        strictness.range,
        vec![json!("lenient"), json!("balanced"), json!("strict")]
    );
}
