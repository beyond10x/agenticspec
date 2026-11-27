//! Behaviour of the simulation, read through the baseline KPIs of one evaluation.

use super::common::{experiment, yaml};
use agenticspec_sim::{Experiment, run};
use serde_json::{Value, json};

fn baseline(arch_edit: impl Fn(&mut Value), exp_edit: impl Fn(&mut Value)) -> Value {
    let e = experiment("fit", 1, 256);
    let mut arch = e.architecture.clone();
    arch_edit(&mut arch);
    let mut doc = e.doc.clone();
    exp_edit(&mut doc);
    let e = Experiment::from_docs(doc, arch, e.profile.clone()).unwrap();
    run(&e, 2).unwrap()["baseline"].clone()
}

fn k(b: &Value, name: &str) -> f64 {
    b["kpis"][name]["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("{name} missing"))
}

#[test]
fn a_short_default_makes_owner_questions_take_their_default() {
    let b = baseline(
        |a| a["policies"]["question_default_after"] = json!("1s"),
        |_| {},
    );
    assert!(k(&b, "questions_asked") > 0.0);
    assert!(k(&b, "questions_defaulted") > 0.0);
    let long = baseline(
        |a| a["policies"]["question_default_after"] = json!("24h"),
        |_| {},
    );
    assert_eq!(k(&long, "questions_defaulted"), 0.0);
    assert!(k(&long, "owner_wait_share") > k(&b, "owner_wait_share"));
}

#[test]
fn an_operator_answering_alone_never_waits_on_the_owner() {
    let b = baseline(
        |a| a["policies"]["operator_answers_alone"] = json!(true),
        |_| {},
    );
    assert_eq!(k(&b, "owner_wait_share"), 0.0);
    assert!(k(&b, "operator_decided_owner_questions") > 0.0);
}

#[test]
fn a_tight_rate_limit_queues_model_calls() {
    let loose = baseline(
        |a| a["parts"]["gateway"]["rate_limit_per_minute"] = json!(600),
        |_| {},
    );
    let tight = baseline(
        |a| a["parts"]["gateway"]["rate_limit_per_minute"] = json!(1),
        |_| {},
    );
    assert_eq!(k(&loose, "rate_limited_seconds"), 0.0);
    assert!(k(&tight, "rate_limited_seconds") > 0.0);
    assert!(k(&tight, "tasks_done") < k(&loose, "tasks_done"));
}

#[test]
fn a_pool_at_zero_runs_no_build_and_finishes_no_task_that_needs_one() {
    let b = baseline(
        |_| {},
        |d| d["faults"] = json!([{ "kind": "capacity_drop", "part": "builds", "to": 0, "from": "0s", "for": "4h" }]),
    );
    assert_eq!(k(&b, "pool_utilization"), 0.0);
    assert_eq!(b["per_part"]["builds"]["jobs"]["value"], 0.0);
    let open = baseline(|_| {}, |d| d["faults"] = json!([]));
    assert!(k(&open, "pool_utilization") > 0.0);
    assert!(k(&open, "tasks_done") > k(&b, "tasks_done"));
}

#[test]
fn queue_wait_percentiles_count_the_jobs_that_started_at_once() {
    let b = baseline(
        |a| a["parts"]["builds"]["capacity"] = json!(1000),
        |d| d["faults"] = json!([{ "kind": "capacity_drop", "part": "builds", "to": 0, "from": "0s", "for": "20m" }]),
    );
    assert!(
        k(&b, "slot_wait_share") > 0.0,
        "some jobs queued in the drop"
    );
    assert_eq!(k(&b, "queue_wait_p50_seconds"), 0.0);
}

#[test]
fn builds_longer_than_the_job_limit_are_killed() {
    let b = baseline(|a| a["parts"]["builds"]["job_limit"] = json!("30s"), |_| {});
    assert!(
        b["per_part"]["builds"]["jobs_killed"]["value"]
            .as_f64()
            .unwrap()
            > 0.0
    );
}

#[test]
fn an_owner_away_all_day_lets_every_owner_question_default() {
    let b = baseline(
        |_| {},
        |d| {
            d["faults"] =
                json!([{ "kind": "principal_away", "part": "owner", "from": "0s", "for": "4h" }])
        },
    );
    assert!(k(&b, "questions_defaulted") > 0.0);
    assert_eq!(b["per_part"]["owner"]["questions_answered"]["value"], 0.0);
}

#[test]
fn cost_follows_the_tokens_and_the_price() {
    let b = baseline(|_| {}, |_| {});
    let cost = k(&b, "tokens_input") * 5.0 / 1e6
        + k(&b, "tokens_cache_read") * 0.5 / 1e6
        + k(&b, "tokens_cache_write") * 6.25 / 1e6
        + k(&b, "tokens_output") * 25.0 / 1e6;
    assert!(
        (k(&b, "cost_usd") - cost).abs() < 1e-6 * cost.max(1.0),
        "{} vs {cost}",
        k(&b, "cost_usd")
    );
    let _ = yaml("a: 1");
}
