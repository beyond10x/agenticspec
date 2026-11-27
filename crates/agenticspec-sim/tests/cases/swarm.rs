//! Triggers, schedules, shared channels and swarm coordination: firings and their policies, the
//! claim race of a board, the calls of an orchestrator, mesh messages, a scope partition and a
//! start stagger.

use super::common::{PROFILE, repo_root, yaml};
use agenticspec_sim::{Experiment, run};
use serde_json::{Value, json};

const EXPERIMENT: &str = r#"
format: agenticspec-experiment/1
experiment: swarm
architecture: swarm.yaml
profile: swarm-profile.yaml
horizon: 4h
seeds: 2
seed: 3
optimize: marked
search:
  mode: exhaustive
  limit: 64
  weights: { tasks_correct: 10, false_completions_shipped: -5 }
"#;

/// A sweeper agent woken by an hourly schedule through a trigger.
pub const SCHEDULED: &str = r#"
format: agenticspec-architecture/1
architecture: scheduled
parts:
  owner:   { block: principal, kind: human }
  hourly:  { block: schedule, every: 1h }
  wake:    { block: trigger, kind: on_schedule, schedule: hourly, policy: skip, max_parallel: 1, task_template: sweep }
  sweeper: { block: agent, harness: coding }
  model:   { block: model, route: frontier }
links:
  - { from: owner, to: sweeper, carries: prompt }
  - { from: wake,  to: sweeper, carries: trigger }
  - { from: sweeper, to: model, carries: model_call }
"#;

/// The owner posts every task to a shared channel; a team is woken by each post.
pub const SHARED: &str = r#"
format: agenticspec-architecture/1
architecture: shared
parts:
  owner:   { block: principal, kind: human }
  inbox:   { block: channel, kind: agent_message, distribution: queue }
  on_post: { block: trigger, kind: on_message, channel: inbox, policy: parallel, max_parallel: 6 }
  team:    { block: agent, harness: coding, count: 6, coordination: board }
  model:   { block: model, route: frontier }
links:
  - { from: owner,   to: inbox, carries: message }
  - { from: on_post, to: team,  carries: trigger }
  - { from: team,    to: model, carries: model_call }
"#;

const ORCHESTRATED: &str = r#"
format: agenticspec-architecture/1
architecture: orchestrated
parts:
  owner:        { block: principal, kind: human }
  orchestrator: { block: agent, harness: orchestrator }
  workers:      { block: agent, harness: coding, count: 4, coordination: orchestrator, start_stagger: 0s }
  model:        { block: model, route: frontier }
links:
  - { from: owner,        to: orchestrator, carries: prompt }
  - { from: orchestrator, to: workers,      carries: order }
  - { from: orchestrator, to: model,        carries: model_call }
  - { from: workers,      to: model,        carries: model_call }
"#;

const TEAM: &str = r#"
format: agenticspec-architecture/1
architecture: team
parts:
  owner: { block: principal, kind: human }
  team:  { block: agent, harness: coding, count: 4, coordination: mesh }
  model: { block: model, route: frontier }
links:
  - { from: owner, to: team,  carries: prompt }
  - { from: team,  to: model, carries: model_call }
"#;

const PLANNED: &str = r#"
format: agenticspec-architecture/1
architecture: planned
parts:
  owner:    { block: principal, kind: human }
  planner:  { block: agent, harness: planner }
  plans:    { block: channel, kind: agent_message, distribution: queue }
  on_plan:  { block: trigger, kind: on_message, channel: plans, policy: parallel, max_parallel: 3 }
  executor: { block: agent, harness: coding, count: 3 }
  model:    { block: model, route: frontier }
links:
  - { from: owner,    to: planner,  carries: prompt }
  - { from: planner,  to: plans,    carries: message }
  - { from: on_plan,  to: executor, carries: trigger }
  - { from: planner,  to: model,    carries: model_call }
  - { from: executor, to: model,    carries: model_call }
"#;

const TOPIC: &str = r#"
format: agenticspec-architecture/1
architecture: topic
parts:
  owner: { block: principal, kind: human }
  news:  { block: channel, kind: agent_message, distribution: topic }
  ta:    { block: trigger, kind: on_message, channel: news, policy: queue, max_parallel: 1 }
  tb:    { block: trigger, kind: on_message, channel: news, policy: queue, max_parallel: 1 }
  a:     { block: agent, harness: coding }
  b:     { block: agent, harness: coding }
  model: { block: model, route: frontier }
links:
  - { from: owner, to: news,  carries: message }
  - { from: ta,    to: a,     carries: trigger }
  - { from: tb,    to: b,     carries: trigger }
  - { from: a,     to: model, carries: model_call }
  - { from: b,     to: model, carries: model_call }
"#;

/// The common profile with the roles and the coordination values these cases need.
pub fn profile() -> Value {
    let mut p = yaml(PROFILE);
    p["roles"]["orchestrator"] = json!({
        "task_turns": 3, "turn_seconds": 5,
        "tokens_per_turn": { "input": 1000, "output": 200 }
    });
    p["roles"]["planner"] = json!({
        "task_turns": 2, "turn_seconds": 10,
        "tokens_per_turn": { "input": 1000, "output": 400 }
    });
    p["coordination"] = json!({
        "board_read_seconds": { "dist": "exponential", "mean": 60 },
        "claim_window_seconds": 20,
        "mesh_read_calls": 1,
        "scope_overlap": 0.0
    });
    p
}

fn base(arch: &str, edit: impl Fn(&mut Value, &mut Value, &mut Value)) -> Value {
    let mut a = yaml(arch);
    let mut p = profile();
    let mut d = yaml(EXPERIMENT);
    edit(&mut a, &mut p, &mut d);
    let e = Experiment::from_docs(d, a, p).unwrap();
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

fn no_load(p: &mut Value) {
    p["load"] = json!({ "tasks_per_hour": 0 });
}

fn long_tasks(p: &mut Value) {
    p["roles"]["coding"]["task_turns"] = json!(100);
}

#[test]
fn a_schedule_fires_every_interval_and_its_template_makes_a_task_each_time() {
    let b = base(SCHEDULED, |_, p, _| no_load(p));
    assert_eq!(k(&b, "firings"), 4.0, "ticks at 0h, 1h, 2h, 3h");
    assert_eq!(k(&b, "firings_started"), 4.0);
    assert_eq!(k(&b, "tasks_arrived"), 4.0);
    assert_eq!(part(&b, "wake", "firings"), 4.0);
    assert_eq!(part(&b, "hourly", "ticks"), 4.0);
}

#[test]
fn a_schedule_starts_at_its_next_fire_time() {
    let b = base(SCHEDULED, |a, p, _| {
        no_load(p);
        a["parts"]["hourly"]["next_fire_at"] = json!("90m");
    });
    assert_eq!(k(&b, "firings"), 3.0, "ticks at 1.5h, 2.5h, 3.5h");
}

#[test]
fn skip_drops_the_firings_that_come_while_a_run_is_going() {
    let b = base(SCHEDULED, |a, p, _| {
        no_load(p);
        long_tasks(p);
        a["parts"]["hourly"]["every"] = json!("1m");
    });
    assert_eq!(k(&b, "firings"), 240.0);
    assert!(k(&b, "firings_skipped") > 0.0);
    assert_eq!(k(&b, "firings_queued"), 0.0);
    assert_eq!(
        k(&b, "firings_started") + k(&b, "firings_skipped"),
        k(&b, "firings")
    );
    let share = k(&b, "firings_skipped") / k(&b, "firings");
    assert!((k(&b, "firing_skip_share") - share).abs() < 1e-9);
    assert_eq!(k(&b, "firing_start_wait_p90_seconds"), 0.0);
}

#[test]
fn queue_holds_the_firings_until_a_run_ends() {
    let b = base(SCHEDULED, |a, p, _| {
        no_load(p);
        long_tasks(p);
        a["parts"]["hourly"]["every"] = json!("1m");
        a["parts"]["wake"]["policy"] = json!("queue");
    });
    assert_eq!(k(&b, "firings_skipped"), 0.0);
    assert!(k(&b, "firings_queued") > 0.0);
    assert!(k(&b, "firings_started") < k(&b, "firings"));
    assert!(
        k(&b, "firing_start_wait_p90_seconds") > 60.0,
        "a queued firing waits for a run to end"
    );
}

#[test]
fn parallel_overlaps_runs_up_to_max_parallel_and_refuses_the_rest() {
    let skip = base(SCHEDULED, |a, p, _| {
        no_load(p);
        long_tasks(p);
        a["parts"]["hourly"]["every"] = json!("1m");
        a["parts"]["sweeper"]["count"] = json!(5);
    });
    let par = base(SCHEDULED, |a, p, _| {
        no_load(p);
        long_tasks(p);
        a["parts"]["hourly"]["every"] = json!("1m");
        a["parts"]["sweeper"]["count"] = json!(5);
        a["parts"]["wake"]["policy"] = json!("parallel");
        a["parts"]["wake"]["max_parallel"] = json!(3);
    });
    assert!(k(&par, "firings_started") > k(&skip, "firings_started"));
    assert!(
        k(&par, "firings_skipped") > 0.0,
        "beyond 3 runs a firing is refused"
    );
    assert!(
        part(&par, "sweeper", "busy_share") <= 3.0 / 5.0 + 1e-9,
        "at most 3 of 5 copies run: {}",
        part(&par, "sweeper", "busy_share")
    );
}

#[test]
fn a_run_with_nothing_to_take_is_empty_and_costs_one_call() {
    let b = base(SCHEDULED, |a, p, _| {
        no_load(p);
        a["parts"]["wake"]
            .as_object_mut()
            .unwrap()
            .remove("task_template");
        a["parts"]["hourly"]["every"] = json!("10m");
    });
    assert_eq!(k(&b, "firings"), 24.0);
    assert_eq!(k(&b, "firings_empty"), 24.0);
    assert_eq!(k(&b, "tasks_started"), 0.0);
    assert!(k(&b, "cost_usd") > 0.0, "an empty run still reads");
    assert_eq!(k(&b, "firing_failure_share"), 0.0);
}

#[test]
fn a_run_that_takes_all_drains_the_backlog() {
    let b = base(SCHEDULED, |a, _, _| {
        let w = a["parts"]["wake"].as_object_mut().unwrap();
        w.remove("task_template");
        w.insert("takes".into(), json!("all"));
        a["parts"]["hourly"]["every"] = json!("4h");
        a["parts"]["hourly"]["next_fire_at"] = json!("1h");
    });
    assert_eq!(k(&b, "firings_started"), 1.0);
    assert!(
        k(&b, "tasks_started") > 1.0,
        "one run took {} tasks",
        k(&b, "tasks_started")
    );
}

#[test]
fn firing_failure_share_counts_the_runs_that_end_wrong() {
    let wrong = base(SCHEDULED, |_, p, _| {
        no_load(p);
        let r = &mut p["roles"]["coding"];
        r["step_pass_after_pass"] = json!(0.0);
        r["step_pass_after_fail"] = json!(0.0);
        r["false_completion"] = json!(1.0);
    });
    assert_eq!(k(&wrong, "firing_failure_share"), 1.0);
    let right = base(SCHEDULED, |_, p, _| no_load(p));
    assert_eq!(k(&right, "firing_failure_share"), 0.0);
    assert_eq!(wrong["kpis"]["firing_failure_share"]["kind"], "Correctness");
}

#[test]
fn posts_on_a_queue_wake_the_team_without_a_claim_conflict() {
    let b = base(SHARED, |_, _, _| {});
    assert!(k(&b, "posts") > 0.0);
    assert_eq!(k(&b, "posts"), k(&b, "tasks_arrived"));
    assert!(k(&b, "firings") >= k(&b, "posts"));
    assert!(k(&b, "tasks_started") > 0.0);
    assert_eq!(k(&b, "claim_conflicts"), 0.0);
    assert_eq!(k(&b, "coordination_calls"), 0.0);
    assert_eq!(part(&b, "inbox", "posts"), k(&b, "posts"));
}

#[test]
fn posts_on_a_board_make_idle_copies_read_and_race_for_the_claim() {
    let b = base(SHARED, |a, _, _| {
        a["parts"]["inbox"]["distribution"] = json!("board");
    });
    assert!(
        k(&b, "coordination_calls") > 0.0,
        "idle copies read each post"
    );
    assert!(k(&b, "claim_conflicts") > 0.0);
    assert!(k(&b, "claim_conflicts") <= k(&b, "coordination_calls"));
    assert!(
        k(&b, "firing_start_wait_p90_seconds") > 0.0,
        "a board is read, not pushed"
    );
    assert_eq!(
        part(&b, "team", "claim_conflicts"),
        k(&b, "claim_conflicts")
    );
}

#[test]
fn a_start_stagger_holds_copies_back_and_cuts_the_claim_race_of_a_burst() {
    let burst = |a: &mut Value, p: &mut Value, d: &mut Value, stagger: &str| {
        p["prices"]["agent_hour_usd"] = json!(1.0);
        a["parts"]["inbox"]["distribution"] = json!("board");
        a["parts"]["team"]["start_stagger"] = json!(stagger);
        d["bursts"] = json!([{ "at": "0s", "tasks": 20 }]);
        d["horizon"] = json!("1h");
        d["seeds"] = json!(8);
    };
    let at_once = base(SHARED, |a, p, d| burst(a, p, d, "0s"));
    let staggered = base(SHARED, |a, p, d| burst(a, p, d, "60s"));
    assert!(k(&at_once, "tasks_arrived") >= 20.0);
    assert!(
        k(&staggered, "claim_conflicts") < k(&at_once, "claim_conflicts"),
        "staggered {} at once {}",
        k(&staggered, "claim_conflicts"),
        k(&at_once, "claim_conflicts")
    );
    assert!(
        k(&staggered, "agent_time_cost_usd") < k(&at_once, "agent_time_cost_usd"),
        "a copy not started yet is not alive"
    );
}

#[test]
fn bursts_add_tasks_at_their_time() {
    let plain = base(TEAM, |_, _, _| {});
    let burst = base(TEAM, |_, _, d| {
        d["bursts"] = json!([{ "at": "1h", "tasks": 10 }])
    });
    assert_eq!(
        k(&burst, "tasks_arrived"),
        k(&plain, "tasks_arrived") + 10.0
    );
}

#[test]
fn an_orchestrator_spends_its_calls_on_every_task_it_hands_out() {
    let b = base(ORCHESTRATED, |_, _, _| {});
    let handed = part(&b, "orchestrator", "tasks_claimed_done");
    assert!(handed > 0.0);
    assert_eq!(
        part(&b, "orchestrator", "tasks_done"),
        0.0,
        "it ships nothing"
    );
    let calls = part(&b, "orchestrator", "turns");
    let owed = 3.0 * part(&b, "orchestrator", "tasks_started");
    assert!(
        calls <= owed && owed - calls < 3.0 + 1e-9,
        "3 calls per task: {calls} of {owed}"
    );
    assert_eq!(
        k(&b, "coordination_calls"),
        part(&b, "orchestrator", "turns")
    );
    assert!(part(&b, "workers", "tasks_started") > 0.0);
    assert_eq!(k(&b, "tasks_done"), part(&b, "workers", "tasks_done"));
}

#[test]
fn mesh_messages_grow_with_the_copies() {
    for n in [2usize, 6] {
        let b = base(TEAM, |a, _, _| a["parts"]["team"]["count"] = json!(n));
        let started = k(&b, "tasks_started");
        assert!(started > 0.0);
        assert_eq!(
            k(&b, "coordination_messages"),
            started * (n as f64 - 1.0),
            "{n} copies"
        );
        assert_eq!(k(&b, "coordination_calls"), k(&b, "coordination_messages"));
    }
}

#[test]
fn a_scope_partition_conflicts_only_when_scopes_overlap() {
    let part_of = |overlap: f64, coordination: &'static str| {
        base(TEAM, move |a, p, _| {
            a["parts"]["team"]["partition"] = json!("scope");
            a["parts"]["team"]["coordination"] = json!(coordination);
            p["coordination"]["scope_overlap"] = json!(overlap);
            p["load"] = json!({ "tasks_per_hour": 12 });
        })
    };
    let clean = part_of(0.0, "mesh");
    assert_eq!(k(&clean, "merge_conflicts"), 0.0);
    assert_eq!(k(&clean, "rework_turns"), 0.0);
    let noticed = part_of(0.6, "mesh");
    assert!(k(&noticed, "merge_conflicts") > 0.0);
    assert!(
        k(&noticed, "rework_turns") > 0.0,
        "a notice turns a conflict into rework"
    );
    assert_eq!(k(&noticed, "false_completions_shipped"), 0.0);
    let silent = part_of(0.6, "none");
    assert!(k(&silent, "merge_conflicts") > 0.0);
    assert_eq!(k(&silent, "coordination_messages"), 0.0);
    assert!(
        k(&silent, "false_completions_shipped") > 0.0,
        "without notices a conflicting merge ships broken"
    );
}

#[test]
fn a_planner_hands_its_plans_to_executors_over_a_queue() {
    let b = base(PLANNED, |_, _, _| {});
    assert!(part(&b, "planner", "tasks_claimed_done") > 0.0);
    assert_eq!(part(&b, "planner", "tasks_done"), 0.0);
    assert!(part(&b, "executor", "tasks_started") > 0.0);
    assert_eq!(k(&b, "tasks_done"), part(&b, "executor", "tasks_done"));
    assert_eq!(
        part(&b, "plans", "posts"),
        part(&b, "planner", "tasks_claimed_done")
    );
    assert_eq!(
        k(&b, "coordination_calls"),
        0.0,
        "a planner is not an orchestrator"
    );
}

#[test]
fn a_topic_post_wakes_every_subscriber_and_ships_once() {
    let b = base(TOPIC, |_, _, _| {});
    assert!(part(&b, "a", "tasks_started") > 0.0);
    assert!(part(&b, "b", "tasks_started") > 0.0);
    assert!(k(&b, "duplicate_runs") > 0.0);
    assert!(k(&b, "tasks_done") <= k(&b, "tasks_arrived"));
    assert_eq!(k(&b, "posts"), k(&b, "tasks_arrived"));
}

#[test]
fn the_swarm_architectures_are_accepted_and_say_what_is_not_modelled() {
    for name in [
        "scope-partition",
        "planner-executor",
        "scheduled-loops",
        "shared-board",
        "orchestrator-workers",
    ] {
        let path = repo_root().join(format!("architectures/{name}.yaml"));
        let a = yaml(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(
            agenticspec_sim::check::check(&a),
            Vec::new(),
            "{name} is refused"
        );
        let summary = a["summary"].as_str().unwrap_or("");
        assert!(
            summary.contains("does not model"),
            "{name}: the summary says what the simulator does not model"
        );
        let marked = agenticspec_sim::marked::collect(&a, false).unwrap();
        assert!(marked.len() >= 2, "{name} marks its trade-offs");
    }
}

#[test]
fn the_swarm_experiments_load_with_the_usual_weights() {
    let dir = repo_root().join("experiments");
    let mut seen = 0;
    for name in ["shared-board-tuning", "scheduled-loops-tuning"] {
        let e = Experiment::load(&dir.join(format!("{name}.yaml"))).unwrap();
        let w = &e.search.weights;
        assert_eq!(w["tasks_correct"], 10.0, "{name}");
        assert_eq!(w["false_completions_shipped"], -5.0, "{name}");
        seen += 1;
    }
    assert_eq!(seen, 2);
}
