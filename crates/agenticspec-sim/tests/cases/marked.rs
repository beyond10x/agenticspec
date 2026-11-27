//! Marked places and `optimize: all`.

use super::common::{repo_root, yaml};
use agenticspec_sim::marked::collect;
use serde_json::json;

fn workstation() -> serde_json::Value {
    yaml(&std::fs::read_to_string(repo_root().join("architectures/workstation.yaml")).unwrap())
}

#[test]
fn every_vary_node_is_a_marked_place_with_its_range() {
    let marked = collect(&workstation(), false).unwrap();
    let paths: Vec<&str> = marked.iter().map(|m| m.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "parts.builds.capacity",
            "parts.gateway.rate_limit_per_minute",
            "parts.worker.count",
            "policies.operator_answers_alone",
            "policies.question_default_after",
        ]
    );
    let count = &marked[2];
    assert_eq!(count.value, json!(12));
    assert_eq!(
        count.range,
        (1..=12).map(|i| json!(i * 2)).collect::<Vec<_>>()
    );
    assert_eq!(
        marked[4].range,
        vec![json!("5m"), json!("15m"), json!("60m")]
    );
}

#[test]
fn optimize_all_adds_every_numeric_value_from_half_to_double() {
    let marked = collect(&workstation(), true).unwrap();
    let job = marked
        .iter()
        .find(|m| m.path == "parts.builds.job_limit")
        .expect("job_limit is marked");
    assert_eq!(job.value, json!("10m"));
    assert_eq!(job.range.len(), 5);
    assert_eq!(job.range.first(), Some(&json!("5m")));
    assert_eq!(job.range[2], json!("10m"));
    assert_eq!(job.range.last(), Some(&json!("20m")));
    assert_eq!(
        marked
            .iter()
            .filter(|m| m.path == "parts.worker.count")
            .count(),
        1
    );
}

#[test]
fn a_value_off_the_grid_joins_its_range() {
    let a = yaml(
        "parts:\n  w: { block: agent, count: { value: 5, vary: { min: 2, max: 8, step: 2 } } }\n",
    );
    let marked = collect(&a, false).unwrap();
    assert_eq!(
        marked[0].range,
        vec![json!(2), json!(4), json!(5), json!(6), json!(8)]
    );
}

#[test]
fn a_vary_without_values_is_refused() {
    let a = yaml("parts:\n  w: { block: agent, count: { value: 5, vary: [] } }\n");
    assert!(collect(&a, false).unwrap_err().contains("parts.w.count"));
}
