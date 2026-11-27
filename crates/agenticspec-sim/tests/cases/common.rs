//! Small synthetic documents shared by the cases.

use agenticspec_sim::{Experiment, doc::parse_yaml};
use serde_json::Value;
use std::path::PathBuf;

pub const ARCH: &str = r#"
format: agenticspec-architecture/1
architecture: small
parts:
  owner:    { block: principal, kind: human }
  operator: { block: agent, harness: coding, acts_for: owner }
  worker:   { block: agent, harness: coding, count: { value: 2, vary: { min: 2, max: 6, step: 2 } } }
  gateway:  { block: gateway, rate_limit_per_minute: { value: 30, vary: [10, 30, 60] } }
  model:    { block: model, route: frontier }
  builds:   { block: pool, kind: build_slots, capacity: { value: 1, vary: [1, 2, 4] }, job_limit: 10m }
  chat:     { block: channel, kind: operator }
  tree:     { block: workspace, sharing: exclusive, per: worker }
  box:      { block: sandbox, isolation: process, egress: [gateway], per: worker }
links:
  - { from: owner,    to: operator, carries: prompt, over: chat }
  - { from: operator, to: owner,    carries: question, over: chat }
  - { from: worker,   to: operator, carries: question }
  - { from: operator, to: worker,   carries: order }
  - { from: operator, to: gateway,  carries: model_call }
  - { from: worker,   to: gateway,  carries: model_call }
  - { from: gateway,  to: model,    carries: model_call }
  - { from: worker,   to: builds,   carries: claim }
border: [operator]
policies:
  question_default_after: { value: 60m, vary: [5m, 15m, 60m] }
  operator_answers_alone: false
"#;

pub const PROFILE: &str = r#"
format: agenticspec-profile/1
profile: small
roles:
  coding:
    task_turns:          { dist: lognormal, median: 10, p90: 30 }
    turn_seconds:        { dist: lognormal, median: 10, p90: 30 }
    tokens_per_turn:     { input: 2000, cache_read: 20000, cache_write: 2000, output: 400 }
    builds_per_task:     { dist: poisson, mean: 2 }
    build_seconds:       { dist: lognormal, median: 60, p90: 200 }
    questions_per_task:  { dist: poisson, mean: 0.5 }
    question_needs_owner: 0.5
principals:
  human:
    answer_seconds:      { dist: lognormal, median: 900, p90: 3600 }
    away:                [{ from: 1h, for: 30m, every: 24h }]
agents:
  operator:
    answer_seconds:      { dist: lognormal, median: 30, p90: 120 }
load:
  tasks_per_hour: 4
prices:
  frontier: { input: 5.0, cache_read: 0.5, cache_write: 6.25, output: 25.0, per: 1M_tokens_usd }
"#;

pub fn experiment_yaml(mode: &str, budget: usize, limit: usize) -> String {
    format!(
        r#"
format: agenticspec-experiment/1
experiment: small-{mode}
architecture: small.yaml
profile: small-profile.yaml
horizon: 4h
seeds: 2
seed: 7
faults:
  - {{ kind: principal_away, part: owner, from: 2h, for: 1h }}
  - {{ kind: capacity_drop, part: builds, to: 0, from: 3h, for: 15m }}
optimize: marked
search:
  mode: {mode}
  budget: {budget}
  restarts: 2
  limit: {limit}
  weights:
    tasks_done: 1
    owner_wait_share: -50
    cost_usd: -0.01
  gates: [unreachable_parts, agent_without_model_path]
"#
    )
}

pub fn yaml(text: &str) -> Value {
    parse_yaml(text).expect("fixture parses")
}

pub fn experiment(mode: &str, budget: usize, limit: usize) -> Experiment {
    Experiment::from_docs(
        yaml(&experiment_yaml(mode, budget, limit)),
        yaml(ARCH),
        yaml(PROFILE),
    )
    .expect("experiment loads")
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A fresh directory under the target's test scratch directory.
pub fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("agenticspec-sim")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}
