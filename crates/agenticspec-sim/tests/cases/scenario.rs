//! Scenarios: the reader, the check against the architecture, the run on the simulator and the
//! oracle over its trace of block events.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli;
use agenticspec_sim::reader::sha256_hex;
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

/// A scenario for the workstation and the coding profile, written into `dir`.
fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let text = format!(
        "format: agenticspec-scenario/1\nscenario: {name}\narchitecture: {}\nprofile: {}\nseed: 3\nhorizon: 2h\n{body}",
        doc_ref(&repo_root().join("architectures/workstation.yaml")),
        doc_ref(&repo_root().join("fields/coding/profiles/coding.yaml")),
    );
    let path = dir.join(format!("{name}.scenario.yaml"));
    std::fs::write(&path, text).unwrap();
    path
}

fn run(path: &Path, out: &Path) -> Result<Value, cli::Refusal> {
    cli::scenario("run", path, Some(out))
}

fn result(out: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(out.join("scenario-result.json")).unwrap())
        .unwrap()
}

fn statuses(r: &Value) -> Vec<String> {
    r["expectations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["status"].as_str().unwrap().to_string())
        .collect()
}

fn names(r: &Value) -> Vec<String> {
    r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_examples_check_against_the_workstation() {
    for name in ["prompt-starts-run", "question-answered", "control-refused"] {
        let checked = cli::scenario("check", &example(name), None)
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(checked["scenario"], name);
        assert_eq!(checked["architecture"], "workstation");
        assert!(checked["inputs"].as_u64().unwrap() >= 1, "{checked}");
    }
}

#[test]
fn a_prompt_to_the_operator_starts_a_run_on_a_worker() {
    let out = scratch("scenario-prompt");
    let ran = run(&example("prompt-starts-run"), &out).unwrap();
    assert_eq!(ran["verdict"], "held");
    assert_eq!(ran["predicted"], "correct");
    let r = result(&out);
    assert_eq!(r["format"], "agenticspec-scenario-result/1");
    let trace = r["trace"].as_array().unwrap();
    let started = trace
        .iter()
        .find(|e| e["event"] == "agentic.work.RunStarted")
        .expect("a run started");
    assert_eq!(started["part"], "worker");
    assert_eq!(started["task"], "when.0");
    assert_eq!(started["sim"], "task_started");
    let sent = &trace[0];
    assert_eq!(sent["event"], "agentic.channel.MessageSent");
    assert_eq!(sent["author"], "owner");
    assert_eq!(sent["channel"], "chat");
    assert!(statuses(&r).contains(&"not_checked".to_string()), "{r}");
}

#[test]
fn a_question_is_answered_with_the_option_the_owner_picks() {
    let out = scratch("scenario-question");
    let ran = run(&example("question-answered"), &out).unwrap();
    assert_eq!(ran["verdict"], "held");
    assert_eq!(ran["predicted"], "question");
    let r = result(&out);
    let trace = r["trace"].as_array().unwrap();
    let asked = trace
        .iter()
        .find(|e| e["name"] == "question_asked" && e["part"] == "worker")
        .unwrap();
    assert_eq!(asked["options"], serde_json::json!(["A", "B"]));
    assert_eq!(asked["default"], "A");
    let answered = trace
        .iter()
        .find(|e| e["name"] == "question_answered")
        .unwrap();
    assert_eq!(answered["part"], "owner");
    assert_eq!(answered["option"], "B");
    let t_asked = asked["t_seconds"].as_f64().unwrap();
    let t_answered = answered["t_seconds"].as_f64().unwrap();
    assert!(t_answered - t_asked >= 600.0, "{t_asked} {t_answered}");
}

#[test]
fn a_control_input_without_proof_is_refused_by_its_rule_and_makes_no_task() {
    let out = scratch("scenario-refused");
    let ran = run(&example("control-refused"), &out).unwrap();
    assert_eq!(ran["predicted"], "refused");
    let r = result(&out);
    assert_eq!(names(&r), vec!["refusal"]);
    assert_eq!(r["trace"][0]["rule"], "control_without_proof");
    assert_eq!(r["inputs"][0]["status"], "refused");
    assert_eq!(r["kpis"]["tasks_arrived"]["value"], 0.0);
}

#[test]
fn inputs_are_refused_without_a_link_or_the_authority_and_admitted_with_proof() {
    let dir = scratch("scenario-admission");
    let path = write(
        &dir,
        "admission",
        "when:\n  - { at: operator, kind: order, author: owner }\n  - { at: operator, kind: control, act: run, author: worker, proof: process_identity }\n  - { at: operator, kind: control, act: run, author: owner, proof: client_record, after: 1m }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\nthen:\n  events:\n    - { event: refusal, rule: input_without_link }\n    - { event: refusal, rule: control_without_authority, author: worker }\n    - { event: task_created, task: when.2 }\n    - { event: run_started, part: worker, task: when.2 }\n",
    );
    let out = dir.join("out");
    let ran = run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(ran["held"], 4, "{ran}");
    let r = result(&out);
    let refusals: Vec<&Value> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["name"] == "refusal")
        .collect();
    assert_eq!(refusals[0]["error"], "agentic.channel.NotAParty");
    assert_eq!(refusals[1]["error"], "agentic.delegation.NotTheDelegator");
}

#[test]
fn an_unknown_key_is_refused_by_its_path() {
    let dir = scratch("scenario-unknown-key");
    let path = write(
        &dir,
        "unknown",
        "when:\n  - { at: operator, kind: prompt, author: owner, atx: 1 }\nfixtures:\n  model:\n    - { outcom: correct }\nthen:\n  events:\n    - { event: run_begun }\n  kpis:\n    tasks_finished: { min: 1 }\n",
    );
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "scenario_invalid");
    for p in [
        "when.0.atx: unknown key",
        "fixtures.model.0.outcom: unknown key",
        "fixtures.model.0.outcome: missing",
        "then.events.0.event: `run_begun`",
        "then.kpis.tasks_finished: unknown key",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }
}

#[test]
fn check_names_an_input_outside_the_border_an_unknown_author_and_a_digest_that_does_not_match() {
    let dir = scratch("scenario-check");
    let path = write(
        &dir,
        "outside",
        "when:\n  - { at: worker, kind: prompt, author: owner }\n  - { at: operator, kind: prompt, author: stranger }\n",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let wrong = sha256_hex(b"another file");
    let digest =
        sha256_hex(&std::fs::read(repo_root().join("fields/coding/profiles/coding.yaml")).unwrap());
    std::fs::write(&path, text.replace(&digest, &wrong)).unwrap();
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "scenario_refused");
    for p in [
        "when.0.at: `worker` is not a border part (border: operator)",
        "when.1.author: `stranger` is no part",
        "profile.digest: ",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }
}

#[test]
fn a_failed_expectation_refuses_the_run_and_is_written() {
    let dir = scratch("scenario-failed");
    let path = write(
        &dir,
        "failed",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\nthen:\n  events:\n    - { event: run_started, part: worker }\n    - { event: question_asked, part: worker }\n  kpis:\n    tasks_done: { min: 2 }\n",
    );
    let out = dir.join("out");
    let err = run(&path, &out).unwrap_err();
    assert_eq!(err.code, "expectation_failed");
    let r = result(&out);
    assert_eq!(r["verdict"], "failed");
    assert_eq!(statuses(&r), vec!["held", "failed"]);
    assert_eq!(r["kpi_bounds"][0]["status"], "failed");
    assert_eq!(r["counts"]["failed"], 2);
}

#[test]
fn an_event_the_simulator_does_not_record_is_not_checked() {
    let dir = scratch("scenario-not-checked");
    let path = write(
        &dir,
        "not-checked",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\nthen:\n  events:\n    - { event: proposal_opened, part: worker }\n    - { event: run_started, part: worker, process: review }\n    - { event: approval_granted }\n    - { event: run_started, part: worker }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap();
    let r = result(&out);
    assert_eq!(
        statuses(&r),
        vec!["not_checked", "not_checked", "not_checked", "held"]
    );
    assert!(
        r["expectations"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("agentic.workspace.ChangeProposed")
    );
}

#[test]
fn the_same_scenario_writes_the_same_bytes() {
    let a = scratch("scenario-same-a");
    let b = scratch("scenario-same-b");
    run(&example("question-answered"), &a).unwrap();
    run(&example("question-answered"), &b).unwrap();
    assert_eq!(
        std::fs::read(a.join("scenario-result.json")).unwrap(),
        std::fs::read(b.join("scenario-result.json")).unwrap()
    );
}

#[test]
fn a_false_completion_ships_a_task_that_is_not_correct() {
    let dir = scratch("scenario-false");
    let path = write(
        &dir,
        "false-completion",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: false_completion, turns: 2 }\nthen:\n  events:\n    - { event: task_completed, part: worker, task: when.0 }\n  kpis:\n    false_completions_shipped: { min: 1, max: 1 }\n    tasks_correct: { max: 0 }\n",
    );
    let out = dir.join("out");
    let ran = run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(ran["predicted"], "false_completion");
}

#[test]
fn a_wrong_run_ends_failed_and_the_task_is_worked_on_again() {
    let dir = scratch("scenario-wrong");
    let path = write(
        &dir,
        "wrong",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: wrong, turns: 2 }\nthen:\n  events:\n    - { event: run_started, part: worker, task: when.0 }\n    - { event: run_ended, part: worker, task: when.0 }\n    - { event: run_started, part: worker, task: when.0 }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
    let r = result(&out);
    let ended = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "run_ended")
        .unwrap();
    assert_eq!(ended["sim"], "task_failed");
}

#[test]
fn a_question_with_no_reply_takes_its_default() {
    let dir = scratch("scenario-default");
    let path = write(
        &dir,
        "default",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: question, turns: 2, options: [A, B], default: A, needs_owner: true }\nthen:\n  events:\n    - { event: question_asked, part: worker, options: [A, B] }\n    - { event: default_taken, part: worker, option: A }\n    - { event: question_answered, absent: true }\n  kpis:\n    questions_defaulted: { min: 1, max: 1 }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
}

#[test]
fn a_reply_after_the_default_deadline_comes_too_late() {
    let dir = scratch("scenario-late");
    let path = write(
        &dir,
        "late",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: question, turns: 2, options: [A, B], default: A, needs_owner: true }\n  principals:\n    - { part: owner, option: B, after: 20m }\nthen:\n  events:\n    - { event: default_taken, option: A }\n    - { event: question_answered, absent: true }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
}

#[test]
fn a_build_longer_than_the_job_limit_is_killed() {
    let dir = scratch("scenario-killed");
    let path = write(
        &dir,
        "killed",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2, builds: 2 }\n  tools:\n    - { tool: build, seconds: 30s }\n    - { tool: build, seconds: 11m }\nthen:\n  events:\n    - { event: claim_released, part: builds }\n    - { event: claim_killed, part: builds }\n  kpis:\n    jobs: { min: 2, max: 2 }\n    jobs_killed: { min: 1, max: 1 }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
}

#[test]
fn a_given_question_is_open_at_the_start_and_answered_by_the_reply() {
    let dir = scratch("scenario-given");
    let path = write(
        &dir,
        "given",
        "given:\n  tasks:\n    - { id: backlog-1 }\n  questions:\n    - { id: q-1, asked_by: worker, addressee: owner, options: [A, B, C], default: C }\nwhen: []\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\n  principals:\n    - { part: owner, option: A, after: 5m }\nthen:\n  order: set\n  events:\n    - { event: question_answered, part: owner, task: q-1, option: A }\n    - { event: run_started, task: backlog-1 }\n    - { event: question_asked, task: q-1, absent: true }\n    - { event: task_created, absent: true }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
}

#[test]
fn set_order_matches_in_any_order_and_sequence_order_does_not() {
    let dir = scratch("scenario-order");
    let body = |order: &str| {
        format!(
            "when:\n  - {{ at: operator, kind: prompt, author: owner }}\nfixtures:\n  model:\n    - {{ outcome: correct, turns: 2 }}\nthen:\n  order: {order}\n  events:\n    - {{ event: task_completed }}\n    - {{ event: run_started }}\n"
        )
    };
    let set = write(&dir, "set", &body("set"));
    run(&set, &dir.join("set")).unwrap_or_else(|e| panic!("{e:?}"));
    let seq = write(&dir, "sequence", &body("sequence"));
    let err = run(&seq, &dir.join("sequence")).unwrap_err();
    assert_eq!(err.code, "expectation_failed");
}

#[test]
fn run_needs_its_out_directory() {
    let err = cli::scenario("run", &example("control-refused"), None).unwrap_err();
    assert_eq!(err.code, "out_missing");
}
