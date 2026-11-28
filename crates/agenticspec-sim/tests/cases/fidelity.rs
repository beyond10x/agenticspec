//! Where the simulation follows the architecture instead of a name: the answering agent and the
//! profile role come from the document, an agent splits its calls over several priced models, a
//! runtime or process part executes the runs it is given, and an `on_event` trigger fires on its
//! event.

use super::common::{PROFILE, yaml};
use agenticspec_sim::check::check;
use agenticspec_sim::{Experiment, run};
use serde_json::{Value, json};

const EXPERIMENT: &str = r#"
format: agenticspec-experiment/1
experiment: fidelity
architecture: fidelity.yaml
profile: fidelity-profile.yaml
horizon: 8h
seeds: 2
seed: 11
optimize: marked
search:
  mode: exhaustive
  limit: 16
  weights: { tasks_done: 1, cost_usd: -0.01 }
"#;

/// Workers ask an agent that acts for the owner; it answers or passes the question on. The
/// harness is a kind of harness, not a profile role.
const DESK: &str = r#"
format: agenticspec-architecture/1
architecture: desk
parts:
  owner:    { block: principal, kind: human }
  operator: { block: agent, harness: claude-code, role: operator, acts_for: owner }
  worker:   { block: agent, harness: claude-code, count: 3 }
  model:    { block: model, route: frontier }
  chat:     { block: channel, kind: operator }
links:
  - { from: owner,    to: operator, carries: prompt, over: chat }
  - { from: operator, to: owner,    carries: question, over: chat }
  - { from: worker,   to: operator, carries: question }
  - { from: operator, to: model,    carries: model_call }
  - { from: worker,   to: model,    carries: model_call }
border: [operator]
policies:
  question_default_after: 60m
"#;

/// One worker kind that reaches two models through one gateway.
const TWO_MODELS: &str = r#"
format: agenticspec-architecture/1
architecture: two-models
parts:
  owner:   { block: principal, kind: human }
  worker:  { block: agent, count: 2, model_share: { big: 3, small: 1 } }
  gateway: { block: gateway }
  big:     { block: model, route: frontier }
  small:   { block: model, route: fast }
links:
  - { from: owner,   to: worker,  carries: prompt }
  - { from: worker,  to: gateway, carries: model_call }
  - { from: gateway, to: big,     carries: model_call }
  - { from: gateway, to: small,   carries: model_call }
"#;

/// An agent hands every task to a workflow runtime as a run; the runtime calls a model in its
/// steps and asks the owner in a person step.
const RUNTIME: &str = r#"
format: agenticspec-architecture/1
architecture: runtime
parts:
  owner: { block: principal, kind: human }
  lead:  { block: agent }
  flow:  { block: runtime, role: workflow, count: 2 }
  model: { block: model, route: frontier }
links:
  - { from: owner, to: lead,  carries: prompt }
  - { from: lead,  to: flow,  carries: run }
  - { from: flow,  to: owner, carries: question }
  - { from: lead,  to: model, carries: model_call }
  - { from: flow,  to: model, carries: model_call }
"#;

/// Every task a worker ships wakes a notifier runtime, which makes one notice from its template.
const ON_EVENT: &str = r#"
format: agenticspec-architecture/1
architecture: on-event
parts:
  owner:    { block: principal, kind: human }
  worker:   { block: agent, count: 2 }
  on_done:  { block: trigger, kind: on_event, event_name: agentic.work.TaskCompleted, source: worker, policy: parallel, max_parallel: 4, task_template: notice }
  notifier: { block: runtime, role: notify, count: 4 }
  model:    { block: model, route: frontier }
links:
  - { from: owner,   to: worker,   carries: prompt }
  - { from: on_done, to: notifier, carries: trigger }
  - { from: worker,  to: model,    carries: model_call }
"#;

/// Every task a worker ships wakes a process part whose program has two steps.
const PROCESS: &str = r#"
format: agenticspec-architecture/1
architecture: process
parts:
  owner:   { block: principal, kind: human }
  worker:  { block: agent, count: 2 }
  on_done: { block: trigger, kind: on_event, event_name: agentic.work.TaskCompleted, source: worker, policy: parallel, max_parallel: 4, task_template: notice }
  release: { block: process, steps: 2, role: notify, count: 4 }
  model:   { block: model, route: frontier }
links:
  - { from: owner,   to: worker,  carries: prompt }
  - { from: on_done, to: release, carries: trigger }
  - { from: worker,  to: model,   carries: model_call }
"#;

/// The common profile in the form these cases need: the answer delay on the role of the agent
/// that answers, a second priced route and the roles of the runtimes.
fn profile() -> Value {
    let mut p = yaml(PROFILE);
    if let Some(o) = p.as_object_mut() {
        o.remove("agents");
    }
    p["roles"]["operator"] = json!({
        "answer_seconds": { "dist": "lognormal", "median": 30, "p90": 120 }
    });
    p["roles"]["planner"] = json!({
        "task_turns": 1, "turn_seconds": 10,
        "tokens_per_turn": { "input": 1000, "output": 400 }
    });
    p["roles"]["workflow"] = json!({
        "task_turns": 4, "turn_seconds": 30,
        "tokens_per_turn": { "input": 500, "output": 100 },
        "questions_per_task": 1
    });
    p["roles"]["notify"] = json!({ "task_turns": 1, "turn_seconds": 5 });
    p["prices"]["fast"] = json!({ "input": 1.0, "cache_read": 0.1, "cache_write": 1.25, "output": 5.0, "per": "1M_tokens_usd" });
    p
}

fn try_base(arch: &Value, profile: &Value) -> Result<Value, String> {
    let e = Experiment::from_docs(yaml(EXPERIMENT), arch.clone(), profile.clone())?;
    Ok(run(&e, 2)?["baseline"].clone())
}

fn base(arch: &Value) -> Value {
    try_base(arch, &profile()).unwrap()
}

fn k(b: &Value, name: &str) -> f64 {
    b["kpis"][name]["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("KPI {name} missing"))
}

fn part(b: &Value, part: &str, name: &str) -> f64 {
    b["per_part"][part][name]["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("{part}.{name} missing in {}", b["per_part"][part]))
}

/// `arch` with part `from` renamed to `to` where a part is named: its key, the links, the border.
fn rename(arch: &Value, from: &str, to: &str) -> Value {
    let mut a = arch.clone();
    let parts = a["parts"].as_object_mut().unwrap();
    let p = parts.remove(from).unwrap();
    parts.insert(to.to_string(), p);
    for l in a["links"].as_array_mut().unwrap() {
        for key in ["from", "to", "over"] {
            if l[key] == from {
                l[key] = json!(to);
            }
        }
    }
    for b in a["border"].as_array_mut().into_iter().flatten() {
        if *b == from {
            *b = json!(to);
        }
    }
    a
}

#[test]
fn renaming_the_answering_agent_changes_no_result() {
    let desk = yaml(DESK);
    let a = base(&desk);
    assert!(part(&a, "operator", "questions_answered") > 0.0);
    let renamed = rename(&desk, "operator", "concierge");
    assert!(renamed["parts"]["operator"].is_null());
    let b = base(&renamed);
    assert_eq!(a["kpis"], b["kpis"]);
    assert_eq!(a["per_part"]["operator"], b["per_part"]["concierge"]);
}

#[test]
fn the_profile_role_comes_from_role_and_the_harness_is_only_its_kind() {
    let mut planner_harness = yaml(DESK);
    planner_harness["parts"]["worker"]["harness"] = json!("planner");
    let mut coding = yaml(DESK);
    coding["parts"]["worker"]["harness"] = json!("coding");
    assert_eq!(
        base(&planner_harness)["kpis"],
        base(&coding)["kpis"],
        "a harness named like a profile role does not pick that role"
    );
    let mut planner_role = yaml(DESK);
    planner_role["parts"]["worker"]["role"] = json!("planner");
    let p = base(&planner_role);
    let c = base(&coding);
    assert!(
        k(&p, "turns") < k(&c, "turns"),
        "role planner takes one turn a task"
    );
}

#[test]
fn an_answer_delay_keyed_by_part_name_is_refused() {
    let mut p = profile();
    p["agents"] = json!({ "operator": { "answer_seconds": 30 } });
    let err = try_base(&yaml(DESK), &p).unwrap_err();
    assert!(err.contains("roles.<role>.answer_seconds"), "{err}");
}

#[test]
fn an_answering_agent_whose_role_has_no_answer_delay_is_refused() {
    let mut desk = yaml(DESK);
    desk["parts"]["operator"]["role"] = json!("coding");
    let err = try_base(&desk, &profile()).unwrap_err();
    assert!(
        err.contains("operator") && err.contains("answer_seconds"),
        "{err}"
    );
}

#[test]
fn an_agent_splits_its_calls_over_the_models_it_reaches_by_their_share() {
    let b = base(&yaml(TWO_MODELS));
    let (big, small) = (part(&b, "big", "calls"), part(&b, "small", "calls"));
    assert!(small > 0.0, "the second model is called");
    assert!(
        big > 2.0 * small,
        "a share of 3 to 1: big {big}, small {small}"
    );
    assert_eq!(big + small, k(&b, "turns"));
    assert_eq!(part(&b, "gateway", "calls"), big + small);
    for (m, price) in [
        ("big", [5.0, 0.5, 6.25, 25.0]),
        ("small", [1.0, 0.1, 1.25, 5.0]),
    ] {
        let want: f64 = ["input", "cache_read", "cache_write", "output"]
            .iter()
            .zip(price)
            .map(|(c, p)| part(&b, m, &format!("tokens_{c}")) * p / 1e6)
            .sum();
        let got = part(&b, m, "cost_usd");
        assert!(
            (got - want).abs() < 1e-9 * want.max(1.0),
            "{m}: {got} vs {want}"
        );
        assert!(got > 0.0, "{m} is priced by its own route");
    }
    let mut even = yaml(TWO_MODELS);
    even["parts"]["worker"]
        .as_object_mut()
        .unwrap()
        .remove("model_share");
    let e = base(&even);
    let (big, small) = (part(&e, "big", "calls"), part(&e, "small", "calls"));
    assert!(
        small > 0.0 && big < 2.0 * small && small < 2.0 * big,
        "without shares the calls split evenly: big {big}, small {small}"
    );
}

#[test]
fn a_route_the_profile_does_not_price_is_refused_by_name() {
    let mut a = yaml(TWO_MODELS);
    a["parts"]["small"]["route"] = json!("cheap");
    let err = try_base(&a, &profile()).unwrap_err();
    assert!(err.contains("`cheap`") && err.contains("small"), "{err}");
    let mut a = yaml(TWO_MODELS);
    a["parts"]["worker"]["model_share"] = json!({ "big": 1, "elsewhere": 1 });
    let err = try_base(&a, &profile()).unwrap_err();
    assert!(err.contains("elsewhere"), "{err}");
}

#[test]
fn a_runtime_that_receives_runs_executes_their_steps_and_person_steps() {
    let b = base(&yaml(RUNTIME));
    let runs = part(&b, "flow", "tasks_started");
    assert!(runs > 0.0, "the runtime starts the runs the lead hands it");
    assert!(part(&b, "flow", "tasks_done") > 0.0, "and finishes them");
    let steps = part(&b, "flow", "run_steps");
    assert!(steps >= runs, "every run has steps");
    assert_eq!(part(&b, "flow", "model_calls"), steps, "it reaches a model");
    let person = part(&b, "flow", "person_steps");
    assert!(person > 0.0, "a person step is asked");
    assert_eq!(part(&b, "owner", "questions_received"), person);
    assert!(part(&b, "owner", "questions_answered") > 0.0);
    assert!(part(&b, "flow", "owner_wait_share") > 0.0);
    assert!(
        b["per_part"]["flow"]["agent_time_cost_usd"].is_null(),
        "a runtime is not an agent alive"
    );
    let mut offline = yaml(RUNTIME);
    offline["links"]
        .as_array_mut()
        .unwrap()
        .retain(|l| l["from"] != "flow" || l["to"] != "model");
    let o = base(&offline);
    assert!(part(&o, "flow", "run_steps") > 0.0);
    assert_eq!(part(&o, "flow", "model_calls"), 0.0);
    assert_eq!(part(&o, "flow", "cost_usd"), 0.0);
}

#[test]
fn a_runtime_that_executes_runs_names_its_role() {
    let mut a = yaml(RUNTIME);
    a["parts"]["flow"].as_object_mut().unwrap().remove("role");
    let err = try_base(&a, &profile()).unwrap_err();
    assert!(err.contains("flow") && err.contains("role"), "{err}");
}

#[test]
fn an_on_event_trigger_fires_each_time_its_event_occurs() {
    let b = base(&yaml(ON_EVENT));
    let shipped = part(&b, "worker", "tasks_done");
    assert!(shipped > 0.0);
    assert_eq!(part(&b, "on_done", "firings"), shipped);
    assert!(part(&b, "on_done", "firings_started") > 0.0);
    assert_eq!(
        part(&b, "notifier", "tasks_started"),
        part(&b, "on_done", "firings_started")
    );
}

#[test]
fn a_trigger_may_wake_a_runtime() {
    let refusals: Vec<String> = check(&yaml(ON_EVENT))
        .iter()
        .map(|r| r.to_string())
        .collect();
    assert!(refusals.is_empty(), "{refusals:?}");
}

#[test]
fn a_trigger_wakes_a_process_part_that_runs_the_steps_of_its_program() {
    let b = base(&yaml(PROCESS));
    let runs = part(&b, "release", "tasks_started");
    assert!(runs > 0.0, "the trigger starts runs of the process");
    assert_eq!(runs, part(&b, "on_done", "firings_started"));
    assert!(part(&b, "release", "tasks_done") > 0.0, "and they end");
    assert_eq!(
        part(&b, "release", "run_steps"),
        2.0 * runs,
        "each run executes the two steps of the program"
    );
    assert!(
        b["per_part"]["release"]["agent_time_cost_usd"].is_null(),
        "a process is not an agent alive"
    );
    let mut three = yaml(PROCESS);
    three["parts"]["release"]["steps"] = json!(3);
    let t = base(&three);
    let runs = part(&t, "release", "tasks_started");
    assert_eq!(part(&t, "release", "run_steps"), 3.0 * runs);
    let mut unlinked = yaml(PROCESS);
    unlinked["links"]
        .as_array_mut()
        .unwrap()
        .retain(|l| l["to"] != "release");
    let u = base(&unlinked);
    assert!(
        u["per_part"]["release"]["tasks_started"].is_null(),
        "a process nothing gives runs to does not run"
    );
}

#[test]
fn a_trigger_may_wake_a_process_and_the_process_names_its_role() {
    let refusals: Vec<String> = check(&yaml(PROCESS))
        .iter()
        .map(|r| r.to_string())
        .collect();
    assert!(refusals.is_empty(), "{refusals:?}");
    let mut a = yaml(PROCESS);
    a["parts"]["release"]
        .as_object_mut()
        .unwrap()
        .remove("role");
    let err = try_base(&a, &profile()).unwrap_err();
    assert!(err.contains("release") && err.contains("role"), "{err}");
}

#[test]
fn an_event_the_simulation_does_not_emit_is_refused_by_name() {
    let mut a = yaml(ON_EVENT);
    a["parts"]["on_done"]["event_name"] = json!("agentic.model.RouteDefined");
    let err = try_base(&a, &profile()).unwrap_err();
    assert!(err.contains("agentic.model.RouteDefined"), "{err}");
}

#[test]
fn runtimes_models_and_events_keep_one_and_four_threads_byte_equal() {
    let mut a = yaml(ON_EVENT);
    a["parts"]["worker"]["count"] = json!({ "value": 2, "vary": [1, 2, 3] });
    a["parts"]["worker"]["model_share"] = json!({ "model": 1, "small": 1 });
    a["parts"]["small"] = json!({ "block": "model", "route": "fast" });
    a["parts"]["flow"] = json!({ "block": "runtime", "role": "workflow" });
    let links = a["links"].as_array_mut().unwrap();
    links.push(json!({ "from": "worker", "to": "small", "carries": "model_call" }));
    links.push(json!({ "from": "worker", "to": "flow", "carries": "run" }));
    links.push(json!({ "from": "flow", "to": "owner", "carries": "question" }));
    links.push(json!({ "from": "flow", "to": "model", "carries": "model_call" }));
    a["parts"]["on_done"]["source"] = json!("flow");
    let e = Experiment::from_docs(yaml(EXPERIMENT), a, profile()).unwrap();
    let result = run(&e, 1).unwrap();
    let one = serde_json::to_string(&result).unwrap();
    let four = serde_json::to_string(&run(&e, 4).unwrap()).unwrap();
    assert_eq!(one, four);
    let b = &result["baseline"];
    assert!(part(b, "flow", "person_steps") > 0.0, "the runtime ran");
    assert!(
        part(b, "small", "calls") > 0.0,
        "the second model was called"
    );
    assert!(part(b, "notifier", "run_steps") > 0.0, "the event fired");
}

#[test]
fn an_on_event_source_that_names_no_part_is_refused() {
    let mut a = yaml(ON_EVENT);
    a["parts"]["on_done"]["source"] = json!("nowhere");
    let refusals: Vec<(String, String)> = check(&a)
        .iter()
        .map(|r| (r.rule().to_string(), r.part().to_string()))
        .collect();
    assert_eq!(
        refusals,
        vec![("trigger_without_source".to_string(), "on_done".to_string())]
    );
}
