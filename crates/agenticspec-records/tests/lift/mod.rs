//! The lift of a session into rows of the building blocks (`queries/blocks.sq`, `blocks.rs`) and
//! the draft scenario of its outside inputs. Fixtures: `tests/fixtures/blocks/`, synthetic.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use agenticspec_records::blocks::{self, BlockRow, DraftArgs, Lift, uid};
use agenticspec_records::sq;
use agenticspec_sim::doc::parse_yaml;
use agenticspec_sim::reader::{Reader, sha256_hex};
use agenticspec_sim::{rows, scenario};
use serde_json::{Value, json};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/blocks")
        .join(format!("{name}.jsonl"))
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lifted(name: &str) -> Lift {
    blocks::lift_file(&fixture(name)).unwrap()
}

fn of<'a>(l: &'a Lift, entity: &str) -> Vec<&'a BlockRow> {
    l.rows.iter().filter(|r| r.entity == entity).collect()
}

fn text(l: &Lift) -> String {
    l.rows
        .iter()
        .map(|r| r.value().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A scenario path beside the examples (not written): references resolve from there.
fn scenario_path(name: &str) -> PathBuf {
    repo()
        .join("scenarios/workstation")
        .join(format!("{name}.scenario.yaml"))
}

fn doc_ref(file: &str) -> (String, String) {
    let bytes = std::fs::read(repo().join(file)).unwrap();
    (format!("../../{file}"), sha256_hex(&bytes))
}

/// Every problem of a scenario document against the workstation and the coding profile.
fn check(doc: &Value, name: &str) -> Vec<String> {
    let s = scenario::parse(doc).unwrap_or_else(|e| panic!("{e:?}"));
    let l = scenario::load_with(&scenario_path(name), s).unwrap();
    scenario::check(&l)
}

#[test]
fn a_session_lifts_into_one_task_two_runs_their_steps_a_question_and_a_message() {
    let l = lifted("lift-demo");
    assert_eq!(of(&l, "agentic.work.Task").len(), 1);
    let runs = of(&l, "agentic.work.Run");
    assert_eq!(runs.len(), 2, "the main agent and the sub-agent");
    let agents: Vec<&str> = runs.iter().map(|r| r.agent.as_str()).collect();
    assert_eq!(agents, ["main", "b1"]);
    assert_eq!(of(&l, "agentic.work.Assignment").len(), 2);
    for run in &runs {
        let steps = of(&l, "agentic.work.Step")
            .into_iter()
            .filter(|s| s.fields["run_id"] == json!(run.id))
            .count();
        assert!(steps >= 1, "run of {} has no step", run.agent);
    }
    let q = of(&l, "agentic.question.Question");
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].state, Some("Answered"));
    let m = of(&l, "agentic.channel.Message");
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].state, Some("Sent"));
    let task = of(&l, "agentic.work.Task")[0];
    let run_main = runs[0];
    let fb = sq::fact_base(&fixture("lift-demo")).unwrap();
    for row in &l.rows {
        assert!(
            fb.nodes.contains_key(row.evidence.as_str()),
            "{} names {}, no node of the session",
            row.entity,
            row.evidence
        );
    }
    assert_eq!(
        task.evidence, run_main.evidence,
        "the prompt starts the run"
    );
    assert_eq!(
        q[0].fields["run_id"],
        json!(run_main.id),
        "the main run waits on the question"
    );
}

#[test]
fn every_lifted_row_is_declared_by_the_blocks() {
    for name in ["lift-demo", "peer-message"] {
        let l = lifted(name);
        assert!(!l.rows.is_empty());
        for (i, row) in l.rows.iter().enumerate() {
            let p = format!("rows.{i}");
            let mut r = Reader::default();
            let read = rows::read_row(&mut r, &row.value(), &p).unwrap();
            assert!(r.errors.is_empty(), "{:?}", r.errors);
            let problems = rows::check_row(rows::blocks(), &read, &p);
            assert!(problems.is_empty(), "{name}: {problems:?}");
        }
    }
}

#[test]
fn a_scenario_whose_given_is_the_lifted_rows_passes_the_check() {
    let l = lifted("lift-demo");
    let (a, ad) = doc_ref("architectures/workstation.yaml");
    let (p, pd) = doc_ref("profiles/coding.yaml");
    let doc = json!({
        "format": "agenticspec-scenario/1",
        "scenario": "lifted-given",
        "architecture": { "ref": a, "digest": ad },
        "profile": { "ref": p, "digest": pd },
        "given": { "rows": l.rows.iter().map(BlockRow::value).collect::<Vec<_>>() },
        "when": [ { "at": "operator", "kind": "prompt", "author": "owner" } ],
    });
    assert_eq!(check(&doc, "lifted-given"), Vec::<String>::new());
    let mut broken = doc.clone();
    broken["given"]["rows"][0]["fields"]
        .as_object_mut()
        .unwrap()
        .remove("goal");
    let problems = check(&broken, "lifted-given");
    assert!(
        problems.iter().any(|p| p.contains(
            "given.rows.0.fields.goal: missing, agentic.work.Task requires it"
        )),
        "{problems:?}"
    );
}

#[test]
fn a_session_without_a_claim_lifts_no_claim_and_names_it_not_observable() {
    let l = lifted("lift-demo");
    assert!(of(&l, "agentic.resource.Claim").is_empty());
    assert_eq!(l.not_observable, ["agentic.resource.Claim"]);
}

#[test]
fn two_lifts_of_one_session_write_the_same_bytes() {
    for name in ["lift-demo", "peer-message"] {
        assert_eq!(text(&lifted(name)), text(&lifted(name)));
    }
}

#[test]
fn the_rules_show_the_rows_as_a_query_module() {
    let p = sq::compile(blocks::RULES).unwrap();
    let results = sq::run_all(&p, &[fixture("lift-demo")]);
    let report = sq::report(&p, &results);
    let per_query = |q: i64| report.rows.iter().filter(|r| r.query == q).count();
    assert!(report.queries[0].starts_with("?- block_task("));
    assert_eq!(per_query(0), 1, "one task");
    assert_eq!(per_query(1), 2, "two runs");
}

#[test]
fn a_received_message_names_the_turn_that_received_it() {
    let l = lifted("peer-message");
    let fb = sq::fact_base(&fixture("peer-message")).unwrap();
    let turns: Vec<String> = fb.tables["turn"]
        .rows
        .iter()
        .map(|r| r.vals[0].to_string())
        .collect();
    let m = of(&l, "agentic.channel.Message");
    assert_eq!(m.len(), 2);
    let first = &l.inputs[0];
    let turn = first
        .turn
        .as_deref()
        .expect("a turn received the first message");
    assert!(turns.iter().any(|t| t == turn), "{turn} is a turn");
    assert_eq!(m[0].evidence, first.node);
    assert_eq!(m[0].fields["received_in_turn"], json!(uid("turn", turn)));
    assert_eq!(m[0].state, Some("Read"));
    assert!(
        !m[1].fields.contains_key("received_in_turn"),
        "no turn read the last message"
    );
}

fn args(name: &str) -> DraftArgs {
    DraftArgs {
        name: name.to_string(),
        architecture: doc_ref("architectures/workstation.yaml"),
        profile: doc_ref("profiles/coding.yaml"),
        main: "operator".into(),
        sub: Some("worker".into()),
        human: Some("owner".into()),
        delegate: None,
        peer: None,
    }
}

fn drafted(fixture_name: &str, name: &str) -> (blocks::Draft, Value) {
    let d = blocks::draft(&lifted(fixture_name), &args(name));
    let doc = parse_yaml(&d.yaml).unwrap();
    (d, doc)
}

#[test]
fn the_draft_scenario_holds_the_owners_prompt_and_the_task_and_run_it_started() {
    let (d, doc) = drafted("lift-demo", "lift-demo-draft");
    assert_eq!(check(&doc, "lift-demo-draft"), Vec::<String>::new());
    let first = &doc["when"][0];
    assert_eq!(
        (&first["at"], &first["author"], &first["kind"]),
        (&json!("operator"), &json!("owner"), &json!("prompt"))
    );
    let events: Vec<(String, String, String)> = doc["then"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let s = |k: &str| e[k].as_str().unwrap_or("").to_string();
            (s("event"), s("part"), s("task"))
        })
        .collect();
    for want in [
        ("task_created", "operator", "when.0"),
        ("run_started", "operator", "when.0"),
        ("run_started", "worker", "when.0"),
        ("question_asked", "operator", "when.0"),
        ("post_sent", "operator", "when.0"),
    ] {
        let want = (want.0.to_string(), want.1.to_string(), want.2.to_string());
        assert!(events.contains(&want), "{want:?} not in {events:?}");
    }
    assert_eq!(d.expectations, events.len());
}

#[test]
fn a_prompt_by_the_harness_is_no_input() {
    let fb = sq::fact_base(&fixture("lift-demo")).unwrap();
    let t = &fb.tables["prompt"];
    let (node, author) = (t.col("node").unwrap(), t.col("author").unwrap());
    let harness: Vec<String> = t
        .rows
        .iter()
        .filter(|r| r.vals[author].to_string() == "Harness")
        .map(|r| r.vals[node].to_string())
        .collect();
    assert_eq!(harness.len(), 1, "the fixture holds one task notification");
    let l = lifted("lift-demo");
    assert!(l.inputs.iter().all(|i| i.node != harness[0]));
    let (d, doc) = drafted("lift-demo", "lift-demo-draft");
    assert_eq!(d.inputs, 2);
    assert_eq!(doc["when"].as_array().unwrap().len(), 2);
}

#[test]
fn an_input_no_turn_received_is_unjoined_and_has_no_expectation() {
    let (d, doc) = drafted("lift-demo", "lift-demo-draft");
    assert_eq!(d.unjoined, ["when.1 (lift-demo#28)"]);
    assert_eq!(doc["when"].as_array().unwrap().len(), 2, "the input stays");
    let tasks: BTreeMap<String, usize> =
        doc["then"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .fold(BTreeMap::new(), |mut m, e| {
                *m.entry(e["task"].as_str().unwrap_or("").to_string())
                    .or_default() += 1;
                m
            });
    assert!(!tasks.contains_key("when.1"), "{tasks:?}");
}

#[test]
fn a_message_from_another_session_is_an_input_of_kind_message_or_unmapped() {
    let mut a = args("peer-draft");
    let d = blocks::draft(&lifted("peer-message"), &a);
    assert_eq!(d.inputs, 0);
    assert_eq!(
        d.unmapped,
        [
            "peer-message#2 (PeerSession)",
            "peer-message#9 (PeerSession)"
        ]
    );
    a.peer = Some("owner".into());
    let d = blocks::draft(&lifted("peer-message"), &a);
    let doc = parse_yaml(&d.yaml).unwrap();
    assert_eq!(check(&doc, "peer-draft"), Vec::<String>::new());
    assert_eq!(doc["when"][0]["kind"], "message");
    assert_eq!(d.unjoined, ["when.1 (peer-message#9)"]);
}

#[test]
fn two_drafts_of_one_session_write_the_same_bytes() {
    assert_eq!(
        drafted("lift-demo", "x").0.yaml,
        drafted("lift-demo", "x").0.yaml
    );
}
