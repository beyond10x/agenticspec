//! The search: fit improves on the baseline, exhaustive covers the product and refuses above its
//! limit, and the result carries the documented fields.

use super::common::experiment;
use agenticspec_sim::run;
use serde_json::Value;

fn f(v: &Value) -> f64 {
    v["fitness"].as_f64().expect("fitness is a number")
}

#[test]
fn fit_finds_a_better_feasible_variant_than_the_baseline() {
    let r = run(&experiment("fit", 16, 256), 4).unwrap();
    assert_eq!(r["best"]["feasible"], true);
    assert!(
        f(&r["best"]) > f(&r["baseline"]),
        "best {} baseline {}",
        r["best"]["fitness"],
        r["baseline"]["fitness"]
    );
    assert!(r["variants"].as_array().unwrap().len() <= 16);
    let trace = r["trace"].as_array().unwrap();
    assert_eq!(trace.len(), r["variants"].as_array().unwrap().len());
    let last = trace.last().unwrap()["best_so_far"].as_f64().unwrap();
    assert_eq!(last, f(&r["best"]));
}

#[test]
fn exhaustive_evaluates_every_combination() {
    let r = run(&experiment("exhaustive", 0, 100), 4).unwrap();
    assert_eq!(r["variants"].as_array().unwrap().len(), 81);
    let best = f(&r["best"]);
    for v in r["variants"].as_array().unwrap() {
        if v["feasible"] == true {
            assert!(f(v) <= best);
        }
    }
}

#[test]
fn exhaustive_refuses_above_its_limit() {
    let err = run(&experiment("exhaustive", 0, 80), 1).unwrap_err();
    assert!(err.contains("81 combinations exceed the limit 80"), "{err}");
}

#[test]
fn the_result_carries_the_documented_fields() {
    let r = run(&experiment("fit", 4, 256), 2).unwrap();
    assert_eq!(r["format"], "agenticspec-sim-result/1");
    for key in [
        "experiment",
        "architecture",
        "marked",
        "baseline",
        "best",
        "variants",
        "trace",
        "timeline",
    ] {
        assert!(!r[key].is_null(), "missing {key}");
    }
    let marked = r["marked"].as_array().unwrap();
    assert_eq!(marked.len(), 4);
    assert!(marked[0]["range"].is_array());
    let kpis = &r["baseline"]["kpis"];
    for (name, kind) in [
        ("tasks_done", "Quality"),
        ("owner_wait_share", "Quality"),
        ("questions_asked", "Quality"),
        ("questions_defaulted", "Quality"),
        ("queue_wait_p50_seconds", "Quality"),
        ("queue_wait_p90_seconds", "Quality"),
        ("pool_utilization", "Quality"),
        ("rate_limited_seconds", "Quality"),
        ("cost_usd", "Quality"),
        ("tokens_input", "Quality"),
        ("tokens_cache_read", "Quality"),
        ("tokens_cache_write", "Quality"),
        ("tokens_output", "Quality"),
        ("task_completion", "Completeness"),
        ("unreachable_parts", "Correctness"),
        ("agent_without_model_path", "Correctness"),
    ] {
        assert_eq!(kpis[name]["kind"], kind, "{name}");
        assert!(kpis[name]["value"].is_number(), "{name}");
    }
    assert!(kpis["tasks_done"]["value"].as_f64().unwrap() > 0.0);
    assert!(r["baseline"]["per_part"]["worker"]["tasks_done"]["value"].is_number());
    assert!(r["baseline"]["per_part"]["builds"]["queue_wait_p90_seconds"]["value"].is_number());
    let timeline = r["timeline"].as_array().unwrap();
    assert!(!timeline.is_empty() && timeline.len() <= 2000);
    assert!(
        timeline
            .windows(2)
            .all(|w| w[0]["t_seconds"].as_f64() <= w[1]["t_seconds"].as_f64())
    );
}

#[test]
fn an_unknown_weight_is_refused() {
    let e = experiment("fit", 4, 256);
    let mut doc = e.doc.clone();
    doc["search"]["weights"]["happiness"] = serde_json::json!(1);
    let err =
        agenticspec_sim::Experiment::from_docs(doc, e.architecture.clone(), e.profile.clone())
            .err()
            .unwrap();
    assert!(err.contains("happiness"), "{err}");
}

#[test]
fn a_gate_that_fails_makes_every_variant_infeasible() {
    let e = experiment("fit", 3, 256);
    let mut arch = e.architecture.clone();
    arch["parts"]["spare"] = serde_json::json!({"block": "pool", "capacity": 2});
    let e = agenticspec_sim::Experiment::from_docs(e.doc.clone(), arch, e.profile.clone()).unwrap();
    let r = run(&e, 2).unwrap();
    assert_eq!(r["baseline"]["feasible"], false);
    assert_eq!(r["baseline"]["kpis"]["unreachable_parts"]["value"], 1.0);
    assert!(r["best"].is_null());
}
