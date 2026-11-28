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
        .map(|e| e["event"].as_str().unwrap().to_string())
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
        .find(|e| e["event"] == "agentic.question.QuestionAsked" && e["part"] == "worker")
        .unwrap();
    assert_eq!(asked["options"], serde_json::json!(["A", "B"]));
    assert_eq!(asked["default"], "A");
    let answered = trace
        .iter()
        .find(|e| e["event"] == "agentic.question.QuestionAnswered")
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
    assert_eq!(r["trace"][0]["command"], "agentic.channel.SendMessage");
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
        "when:\n  - { at: operator, kind: order, author: owner }\n  - { at: operator, kind: control, act: run, author: worker, proof: process_identity }\n  - { at: operator, kind: control, act: run, author: owner, proof: client_record, after: 1m }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\nthen:\n  events:\n    - { refusal: agentic.channel.SendMessage, rule: input_without_link }\n    - { refusal: agentic.channel.SendMessage, rule: control_without_authority, author: worker }\n    - { event: agentic.work.TaskCreated, task: when.2 }\n    - { event: agentic.work.RunStarted, part: worker, task: when.2 }\n",
    );
    let out = dir.join("out");
    let ran = run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(ran["held"], 4, "{ran}");
    let r = result(&out);
    let refusals: Vec<&Value> = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event"] == "refusal")
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
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\nthen:\n  events:\n    - { event: agentic.work.RunStarted, part: worker }\n    - { event: agentic.question.QuestionAsked, part: worker }\n  kpis:\n    tasks_done: { min: 2 }\n",
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
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\nthen:\n  events:\n    - { event: agentic.workspace.ChangeProposed, part: worker }\n    - { event: agentic.work.RunStarted, part: worker, process: review }\n    - { event: agentic.authority.ApprovalGranted }\n    - { event: agentic.work.RunStarted, part: worker }\n",
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
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: false_completion, turns: 2 }\nthen:\n  events:\n    - { event: agentic.work.TaskCompleted, part: worker, task: when.0 }\n  kpis:\n    false_completions_shipped: { min: 1, max: 1 }\n    tasks_correct: { max: 0 }\n",
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
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: wrong, turns: 2 }\nthen:\n  events:\n    - { event: agentic.work.RunStarted, part: worker, task: when.0 }\n    - { event: agentic.work.RunEnded, part: worker, task: when.0 }\n    - { event: agentic.work.RunStarted, part: worker, task: when.0 }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
    let r = result(&out);
    let ended = r["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event"] == "agentic.work.RunEnded")
        .unwrap();
    assert_eq!(ended["sim"], "task_failed");
}

#[test]
fn a_question_with_no_reply_takes_its_default() {
    let dir = scratch("scenario-default");
    let path = write(
        &dir,
        "default",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: question, turns: 2, options: [A, B], default: A, needs_owner: true }\nthen:\n  events:\n    - { event: agentic.question.QuestionAsked, part: worker, options: [A, B] }\n    - { event: agentic.question.DefaultTaken, part: worker, option: A }\n    - { event: agentic.question.QuestionAnswered, absent: true }\n  kpis:\n    questions_defaulted: { min: 1, max: 1 }\n",
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
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: question, turns: 2, options: [A, B], default: A, needs_owner: true }\n  principals:\n    - { part: owner, option: B, after: 20m }\nthen:\n  events:\n    - { event: agentic.question.DefaultTaken, option: A }\n    - { event: agentic.question.QuestionAnswered, absent: true }\n",
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
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { outcome: correct, turns: 2, builds: 2 }\n  tools:\n    - { tool: build, seconds: 30s }\n    - { tool: build, seconds: 11m }\nthen:\n  events:\n    - { event: agentic.resource.ClaimReleased, part: builds }\n    - { event: agentic.resource.ClaimKilled, part: builds }\n  kpis:\n    jobs: { min: 2, max: 2 }\n    jobs_killed: { min: 1, max: 1 }\n",
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
        "given:\n  tasks:\n    - { id: backlog-1 }\n  questions:\n    - { id: q-1, asked_by: worker, addressee: owner, options: [A, B, C], default: C }\nwhen: []\nfixtures:\n  model:\n    - { outcome: correct, turns: 2 }\n  principals:\n    - { part: owner, option: A, after: 5m }\nthen:\n  order: set\n  events:\n    - { event: agentic.question.QuestionAnswered, part: owner, task: q-1, option: A }\n    - { event: agentic.work.RunStarted, task: backlog-1 }\n    - { event: agentic.question.QuestionAsked, task: q-1, absent: true }\n    - { event: agentic.work.TaskCreated, absent: true }\n",
    );
    let out = dir.join("out");
    run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
}

#[test]
fn set_order_matches_in_any_order_and_sequence_order_does_not() {
    let dir = scratch("scenario-order");
    let body = |order: &str| {
        format!(
            "when:\n  - {{ at: operator, kind: prompt, author: owner }}\nfixtures:\n  model:\n    - {{ outcome: correct, turns: 2 }}\nthen:\n  order: {order}\n  events:\n    - {{ event: agentic.work.TaskCompleted }}\n    - {{ event: agentic.work.RunStarted }}\n"
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

/// A scenario with one prompt at the operator and `then` events, written into `dir`.
fn with_then(dir: &Path, name: &str, events: &str) -> PathBuf {
    write(
        dir,
        name,
        &format!(
            "when:\n  - {{ at: operator, kind: prompt, author: owner }}\nfixtures:\n  model:\n    - {{ outcome: correct, turns: 2, builds: 1 }}\nthen:\n  events:\n{events}"
        ),
    )
}

#[test]
fn then_names_block_events_by_their_full_name_and_holds_on_the_trace() {
    let dir = scratch("scenario-qualified");
    let path = with_then(
        &dir,
        "qualified",
        "    - { event: agentic.work.RunStarted, part: worker, task: when.0 }\n    - { event: agentic.resource.ClaimGranted, part: builds, task: when.0 }\n",
    );
    let checked = cli::scenario("check", &path, None).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(checked["expectations"], 2);
    let out = dir.join("out");
    let ran = run(&path, &out).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(ran["held"], 2, "{ran}");
    let r = result(&out);
    assert_eq!(statuses(&r), vec!["held", "held"]);
    let at = r["expectations"][1]["trace"].as_u64().unwrap() as usize;
    assert_eq!(r["trace"][at]["event"], "agentic.resource.ClaimGranted");
    assert_eq!(r["trace"][at]["sim"], "build_started");
}

#[test]
fn an_event_the_blocks_do_not_declare_is_refused_as_unknown_event() {
    let dir = scratch("scenario-unknown-event");
    let path = with_then(
        &dir,
        "misspelt",
        "    - { event: agentic.resource.ClaimGrantd, part: builds }\n",
    );
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "unknown_event");
    assert!(
        err.detail.contains(
            "then.events.0.event: `agentic.resource.ClaimGrantd` is no event of the blocks; did you mean agentic.resource.ClaimGranted?"
        ),
        "{}",
        err.detail
    );
    let old = with_then(&dir, "old-name", "    - { event: run_started }\n");
    let err = cli::scenario("check", &old, None).unwrap_err();
    assert_eq!(err.code, "unknown_event");
    assert!(
        err.detail.contains("did you mean agentic.work.RunStarted?"),
        "{}",
        err.detail
    );
    let refusal = with_then(&dir, "old-refusal", "    - { event: refusal }\n");
    let err = cli::scenario("check", &refusal, None).unwrap_err();
    assert_eq!(err.code, "unknown_event");
    assert!(
        err.detail
            .contains("a refusal is expected by `refusal: <command>`"),
        "{}",
        err.detail
    );
}

#[test]
fn a_refusal_names_a_command_and_an_error_that_command_declares() {
    let dir = scratch("scenario-unknown-refusal");
    let wrong = with_then(
        &dir,
        "wrong-error",
        "    - { refusal: agentic.resource.RequestClaim, error: agentic.resource.ClaimStateConflict }\n",
    );
    let err = cli::scenario("check", &wrong, None).unwrap_err();
    assert_eq!(err.code, "unknown_refusal");
    assert!(
        err.detail.contains("then.events.0.error: `agentic.resource.RequestClaim` is not refused with `agentic.resource.ClaimStateConflict`")
            && err.detail.contains("agentic.resource.NoTimeLimit"),
        "{}",
        err.detail
    );
    let command = with_then(
        &dir,
        "no-command",
        "    - { refusal: agentic.resource.RequestClam }\n",
    );
    let err = cli::scenario("check", &command, None).unwrap_err();
    assert_eq!(err.code, "unknown_refusal");
    let right = with_then(
        &dir,
        "declared",
        "    - { refusal: agentic.resource.RequestClaim, error: agentic.resource.NoTimeLimit }\n    - { refusal: agentic.channel.SendMessage, error: agentic.delegation.NotTheDelegator, absent: true }\n",
    );
    cli::scenario("check", &right, None).unwrap_or_else(|e| panic!("{e:?}"));
    let out = dir.join("out");
    run(&right, &out).unwrap_or_else(|e| panic!("{e:?}"));
    let r = result(&out);
    assert_eq!(statuses(&r), vec!["not_checked", "held"]);
    assert_eq!(
        r["expectations"][0]["reason"],
        "the simulator refuses no agentic.resource.RequestClaim"
    );
}

#[test]
fn an_expectation_names_an_event_or_a_refusal_never_both() {
    let dir = scratch("scenario-event-and-refusal");
    let path = with_then(
        &dir,
        "both",
        "    - { event: agentic.work.RunStarted, rule: input_without_link }\n    - { part: worker }\n",
    );
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "scenario_invalid");
    for p in [
        "then.events.0: an expected event has no rule",
        "then.events.1: an expectation names an `event`, or a refusal",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }
}

#[test]
fn a_digest_that_does_not_match_the_architecture_is_architecture_digest_mismatch() {
    let dir = scratch("scenario-digest");
    let path = write(
        &dir,
        "digest",
        "when:\n  - { at: operator, kind: prompt, author: owner }\n",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let digest =
        sha256_hex(&std::fs::read(repo_root().join("architectures/workstation.yaml")).unwrap());
    std::fs::write(&path, text.replace(&digest, &sha256_hex(b"another file"))).unwrap();
    for action in ["check", "run"] {
        let err = cli::scenario(action, &path, Some(&dir.join("out"))).unwrap_err();
        assert_eq!(err.code, "architecture_digest_mismatch", "{action}");
        assert!(
            err.detail.contains("architecture.digest: "),
            "{}",
            err.detail
        );
    }
    assert!(
        !dir.join("out/scenario-result.json").exists(),
        "run refuses before the simulation starts"
    );
}

#[test]
fn an_input_outside_the_border_is_input_not_at_border() {
    let dir = scratch("scenario-border");
    let path = write(
        &dir,
        "border",
        "when:\n  - { at: worker, kind: prompt, author: owner }\n",
    );
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "input_not_at_border");
    assert!(
        err.detail.contains(
            "input_not_at_border: when.0.at: `worker` is not a border part (border: operator)"
        ),
        "{}",
        err.detail
    );
}

#[test]
fn a_fixture_of_a_part_the_architecture_does_not_have_is_fixture_without_part() {
    let dir = scratch("scenario-fixture-part");
    let path = write(
        &dir,
        "fixture-part",
        "when:\n  - { at: operator, kind: prompt, author: owner }\nfixtures:\n  model:\n    - { for: reviewer, outcome: correct }\n  principals:\n    - { part: boss, after: 1m }\n",
    );
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "fixture_without_part");
    for p in [
        "fixtures.model.0.for: `reviewer` is no part of the architecture",
        "fixtures.principals.0.part: `boss` is no part of the architecture",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }
}

#[test]
fn a_given_row_without_a_required_field_is_invalid_row() {
    let dir = scratch("scenario-invalid-row");
    let path = write(
        &dir,
        "invalid-row",
        "given:\n  rows:\n    - { entity: agentic.resource.Claim, id: 00000000-0000-4000-8000-000000000003, fields: { holder: 00000000-0000-4000-8000-000000000002, time_limit_seconds: 600, queued_at: \"2026-11-25T09:00:00.000Z\" } }\nwhen:\n  - { at: operator, kind: prompt, author: owner }\n",
    );
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "invalid_row");
    assert!(
        err.detail
            .contains("given.rows.0.fields.pool_id: missing, agentic.resource.Claim requires it"),
        "{}",
        err.detail
    );
}

#[test]
fn problems_of_different_codes_are_refused_as_scenario_refused_naming_each_code() {
    let dir = scratch("scenario-mixed");
    let path = with_then(&dir, "mixed", "    - { event: agentic.work.RunBegun }\n");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("at: operator", "at: worker")).unwrap();
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "scenario_refused");
    for p in [
        "input_not_at_border: when.0.at",
        "unknown_event: then.events.0.event",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }
}

#[test]
fn check_run_twice_on_one_file_answers_the_same_bytes() {
    let dir = scratch("scenario-check-twice");
    let path = with_then(
        &dir,
        "twice",
        "    - { event: agentic.resource.ClaimGrantd }\n    - { refusal: agentic.resource.RequestClaim, error: agentic.resource.UnknownPool }\n",
    );
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("at: operator", "at: worker")).unwrap();
    let a = cli::scenario("check", &path, None).unwrap_err();
    let b = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!((a.code, &a.detail), (b.code, &b.detail));
    let ok = cli::scenario("check", &example("prompt-starts-run"), None).unwrap();
    let again = cli::scenario("check", &example("prompt-starts-run"), None).unwrap();
    assert_eq!(ok.to_string(), again.to_string());
}
