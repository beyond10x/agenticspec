//! The agentic ladder: `ladder/ladder.yaml` checked and turned into the data file of its page.

use super::common::{repo_root, scratch};
use agenticspec_sim::{cli, ladder};
use serde_json::Value;
use std::path::{Path, PathBuf};

const ONE: &str = r#"
format: agenticspec-architecture/1
architecture: one
parts:
  you:   { block: principal, kind: human }
  one:   { block: agent, harness: coding }
  model: { block: model, route: frontier }
  box:   { block: sandbox, isolation: process, egress: [model], per: one }
links:
  - { from: you, to: one,   carries: prompt }
  - { from: one, to: you,   carries: question }
  - { from: one, to: model, carries: model_call }
"#;

const MANY: &str = r#"
format: agenticspec-architecture/1
architecture: many
parts:
  you:   { block: principal, kind: human }
  one:   { block: agent, harness: coding, role: operator, acts_for: you }
  many:  { block: agent, harness: coding, count: 3 }
  model: { block: model, route: frontier }
  box:   { block: sandbox, isolation: process, egress: [model], per: one }
  tests: { block: tool, category: execute }
links:
  - { from: you,  to: one,   carries: prompt }
  - { from: one,  to: you,   carries: question }
  - { from: one,  to: model, carries: model_call }
  - { from: one,  to: many,  carries: order }
  - { from: many, to: one,   carries: question }
  - { from: many, to: model, carries: model_call }
  - { from: many, to: tests, carries: call }
"#;

const STAGES: &str = r#"
stages:
  - { id: S1, name: Paired, marker: one session, trust_need: every diff read, unit_of_work: a diff }
  - { id: S2, name: Orchestrator, marker: many sessions, trust_need: tests, unit_of_work: a task }
"#;

/// A ladder of two rungs over ONE and MANY in a fresh directory; `extra` is appended verbatim.
fn fixture(name: &str, second_trust: &str, extra: &str) -> PathBuf {
    let dir = scratch(&format!("ladder-{name}"));
    std::fs::write(dir.join("one.yaml"), ONE).unwrap();
    std::fs::write(dir.join("many.yaml"), MANY).unwrap();
    let text = format!(
        r#"format: agenticspec-ladder/1
ladder: small
title: Small
summary: two rungs
rule: throughput needs trust
{STAGES}
barriers:
  - {{ id: B1, name: Meta, between: [S1, S2], decision: never grunt work again }}
rungs:
  - id: first
    stage: S1
    title: One
    changed: one session in a sandbox
    architecture: one.yaml
    trust: {{ box: sandbox }}
  - id: second
    stage: S2
    title: Many
    changed: sub-agents
    architecture: many.yaml
{second_trust}
{extra}"#
    );
    std::fs::write(dir.join("ladder.yaml"), text).unwrap();
    dir.join("ladder.yaml")
}

fn accepted(path: &Path) -> Value {
    let (doc, dir) = ladder::load(path).expect("ladder loads");
    let refusals = ladder::check(&doc, &dir);
    assert!(refusals.is_empty(), "refused: {refusals:#?}");
    ladder::build(&doc, &dir).expect("ladder builds")
}

fn refusals(path: &Path) -> Vec<String> {
    let (doc, dir) = ladder::load(path).expect("ladder loads");
    ladder::check(&doc, &dir)
}

fn has(refusals: &[String], rule: &str) -> bool {
    refusals.iter().any(|r| r.starts_with(rule))
}

const TESTS_TRUST: &str = "    trust: { tests: tests prove the work }";

#[test]
fn committed_ladder_is_accepted_and_its_data_file_is_current() {
    let path = repo_root().join("ladder/ladder.yaml");
    let (doc, dir) = ladder::load(&path).expect("ladder loads");
    let refusals = ladder::check(&doc, &dir);
    assert!(refusals.is_empty(), "refused: {refusals:#?}");
    let text = ladder::render(&path).expect("ladder renders");
    let committed =
        std::fs::read_to_string(repo_root().join("website/static/ladder/ladder.json")).unwrap();
    assert!(
        text == committed,
        "website/static/ladder/ladder.json is stale: run task ladder:generate"
    );
}

#[test]
fn committed_ladder_has_seven_stages_three_barriers_and_a_rung_per_stage() {
    let data = accepted(&repo_root().join("ladder/ladder.yaml"));
    let stages: Vec<&str> = data["stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    assert_eq!(stages, ["S1", "S2", "S3", "S4", "S5", "S6", "S7"]);
    let between: Vec<(String, String)> = data["barriers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["between"][0].as_str().unwrap().to_string(),
                b["between"][1].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        between,
        [
            ("S1".to_string(), "S2".to_string()),
            ("S3".to_string(), "S4".to_string()),
            ("S6".to_string(), "S7".to_string())
        ]
    );
    let rungs = data["rungs"].as_array().unwrap();
    assert!((8..=10).contains(&rungs.len()), "{} rungs", rungs.len());
    for s in &stages {
        assert!(
            rungs.iter().any(|r| r["stage"] == *s),
            "stage {s} has no rung"
        );
    }
}

#[test]
fn a_two_rung_ladder_lists_what_each_rung_adds_and_removes() {
    let data = accepted(&fixture("diff", TESTS_TRUST, ""));
    let second = &data["rungs"][1];
    let ids = |v: &Value| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(ids(&second["added_parts"]), ["many", "tests"]);
    assert_eq!(ids(&second["removed_parts"]), Vec::<String>::new());
    assert_eq!(ids(&second["changed_parts"]), ["one"]);
    assert_eq!(ids(&second["trust_added"]), ["tests"]);
    let added_links = ids(&second["added_links"]);
    assert!(
        added_links.contains(&"one>many:order".to_string()),
        "{added_links:?}"
    );
    assert_eq!(added_links.len(), 4, "{added_links:?}");
    assert_eq!(
        ids(&data["rungs"][0]["added_parts"]),
        ["box", "model", "one", "you"]
    );
    // The barrier sits before the first rung of its far stage.
    assert_eq!(data["barriers"][0]["before_rung"], 1);
}

#[test]
fn every_part_keeps_one_position_and_no_two_parts_share_one() {
    let data = accepted(&repo_root().join("ladder/ladder.yaml"));
    let layout = data["layout"]["parts"].as_object().unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for (id, p) in layout {
        let cell = (p["col"].as_i64().unwrap(), p["row"].as_i64().unwrap());
        assert!(seen.insert(cell), "part {id} shares cell {cell:?}");
    }
    for rung in data["rungs"].as_array().unwrap() {
        for p in rung["parts"].as_array().unwrap() {
            let id = p["id"].as_str().unwrap();
            assert!(layout.contains_key(id), "part {id} has no position");
        }
        let v = &rung["view"];
        for p in rung["parts"].as_array().unwrap() {
            let at = &layout[p["id"].as_str().unwrap()];
            let (c, r) = (at["col"].as_i64().unwrap(), at["row"].as_i64().unwrap());
            assert!(
                c >= v["col0"].as_i64().unwrap()
                    && c <= v["col1"].as_i64().unwrap()
                    && r >= v["row0"].as_i64().unwrap()
                    && r <= v["row1"].as_i64().unwrap(),
                "rung {} view leaves out {}",
                rung["id"],
                p["id"]
            );
        }
    }
    // The owner stays put: the first part of the first rung is at the origin.
    assert_eq!(layout["you"]["col"], 0);
}

#[test]
fn a_rung_whose_architecture_file_is_missing_is_refused() {
    let path = fixture("missing", TESTS_TRUST, "");
    std::fs::remove_file(path.parent().unwrap().join("many.yaml")).unwrap();
    let r = refusals(&path);
    assert!(has(&r, "architecture_missing"), "{r:#?}");
}

#[test]
fn a_rung_whose_architecture_fails_the_check_is_refused() {
    let path = fixture("refused", TESTS_TRUST, "");
    let broken = MANY.replace("to: tests, carries: call", "to: test, carries: call");
    std::fs::write(path.parent().unwrap().join("many.yaml"), broken).unwrap();
    let r = refusals(&path);
    assert!(has(&r, "architecture_refused"), "{r:#?}");
    assert!(r.iter().any(|x| x.contains("unknown_link_parts")), "{r:#?}");
}

#[test]
fn a_barrier_between_stages_that_are_not_neighbours_or_do_not_exist_is_refused() {
    let path = fixture("barrier", TESTS_TRUST, "");
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("between: [S1, S2]", "between: [S2, S9]");
    std::fs::write(&path, text).unwrap();
    let r = refusals(&path);
    assert!(has(&r, "barrier_unknown_stage"), "{r:#?}");

    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("between: [S2, S9]", "between: [S2, S1]");
    std::fs::write(&path, text).unwrap();
    let r = refusals(&path);
    assert!(has(&r, "barrier_not_adjacent"), "{r:#?}");
}

#[test]
fn a_part_whose_block_changes_between_rungs_is_refused() {
    let path = fixture("block", TESTS_TRUST, "");
    let changed = MANY.replace(
        "box:   { block: sandbox, isolation: process, egress: [model], per: one }",
        "box:   { block: workspace, sharing: exclusive, per: one }",
    );
    std::fs::write(path.parent().unwrap().join("many.yaml"), changed).unwrap();
    let r = refusals(&path);
    assert!(has(&r, "part_block_changed"), "{r:#?}");
}

#[test]
fn a_trust_part_the_architecture_does_not_have_is_refused() {
    let path = fixture("trust", "    trust: { reviewer: a reviewer }", "");
    let r = refusals(&path);
    assert!(has(&r, "trust_without_part"), "{r:#?}");
}

#[test]
fn more_throughput_without_new_trust_is_refused_unless_the_rung_names_the_risk() {
    let path = fixture("rule", "", "");
    let r = refusals(&path);
    assert!(has(&r, "throughput_without_trust"), "{r:#?}");

    let path = fixture("rule-risk", "    risk: faster, not safer", "");
    let r = refusals(&path);
    assert!(r.is_empty(), "{r:#?}");
}

#[test]
fn a_stage_without_a_rung_is_refused() {
    let path = fixture("stage", TESTS_TRUST, "");
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "unit_of_work: a task }",
        "unit_of_work: a task }\n  - { id: S3, name: Always-on, marker: m, trust_need: t, unit_of_work: a night }",
    );
    std::fs::write(&path, text).unwrap();
    let r = refusals(&path);
    assert!(has(&r, "stage_without_rung"), "{r:#?}");
}

#[test]
fn the_simulation_gives_each_rung_its_numbers_and_the_same_seed_the_same_numbers() {
    let sim = format!(
        "simulation:\n  profile: {}\n  horizon: 4h\n  seeds: 2\n  seed: 3\n  tasks_per_hour_per_worker: 1\n",
        repo_root()
            .join("fields/coding/profiles/coding.yaml")
            .display()
    );
    let path = fixture("sim", TESTS_TRUST, &sim);
    let a = accepted(&path);
    let b = accepted(&path);
    assert_eq!(a, b);
    for rung in a["rungs"].as_array().unwrap() {
        let s = &rung["simulation"];
        for k in [
            "tasks_correct",
            "tasks_done",
            "owner_wait_share",
            "cost_usd",
        ] {
            assert!(s[k].is_number(), "rung {} has no {k}: {s}", rung["id"]);
        }
    }
    assert_eq!(a["simulation"]["horizon"], "4h");
}

#[test]
fn ladder_writes_the_data_file_and_check_compares_its_bytes() {
    let path = fixture("cli", TESTS_TRUST, "");
    let out_file = path.parent().unwrap().join("out/ladder.json");
    let err = cli::ladder(&path, &out_file, true).unwrap_err();
    assert_eq!(err.code, "ladder_stale", "missing file must fail --check");
    let written = cli::ladder(&path, &out_file, false).unwrap();
    assert_eq!(written["written"], true);
    assert_eq!(written["rungs"], 2);
    let current = cli::ladder(&path, &out_file, true).unwrap();
    assert_eq!(current["written"], false);
    let mut text = std::fs::read_to_string(&out_file).unwrap();
    text.push(' ');
    std::fs::write(&out_file, text).unwrap();
    let err = cli::ladder(&path, &out_file, true).unwrap_err();
    assert_eq!(err.code, "ladder_stale");
    assert!(err.detail.contains("ladder:generate"), "{}", err.detail);
}

#[test]
fn ladder_names_each_refusal() {
    let path = fixture("cli-refused", "", "");
    let err = cli::ladder(&path, &path.parent().unwrap().join("x.json"), false).unwrap_err();
    assert_eq!(err.code, "ladder_refused");
    assert!(
        err.detail.contains("throughput_without_trust"),
        "{}",
        err.detail
    );
}
