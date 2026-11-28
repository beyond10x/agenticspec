//! The table from the simulator's event kinds to the events of the building blocks.

use super::common::repo_root;
use agenticspec_sim::events::{self, EXPECTABLE, REFUSAL, RULES, TABLE};
use std::collections::BTreeSet;

/// Every event kind a source of the simulator records: the string literal that ends each
/// `rec`, `rec_task` or `rec_reply` call, and the kinds the scenario runner writes itself.
fn recorded_kinds() -> BTreeSet<String> {
    let dir = repo_root().join("crates/agenticspec-sim/src");
    let mut kinds = BTreeSet::new();
    let mut files = vec![dir.join("sim.rs"), dir.join("scenario.rs")];
    for e in std::fs::read_dir(dir.join("sim")).unwrap() {
        files.push(e.unwrap().path());
    }
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        for call in ["rec(", "rec_task(", "rec_reply("] {
            for (at, _) in text.match_indices(call) {
                let rest = &text[at..];
                let end = rest.find(';').unwrap_or(rest.len());
                let args = &rest[..end];
                if args.starts_with("rec(&mut self") || args.starts_with("rec_task(&mut self") {
                    continue;
                }
                let quoted: Vec<&str> = args.split('"').skip(1).step_by(2).collect();
                if let Some(k) = quoted.first() {
                    kinds.insert(k.to_string());
                }
            }
        }
        for k in ["input_delivered", "input_refused"] {
            if text.contains(&format!("\"{k}\"")) {
                kinds.insert(k.to_string());
            }
        }
    }
    kinds
}

/// Every name a block declares under `key` (`events`, `errors`).
fn declared(key: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for layer in std::fs::read_dir(repo_root().join("blocks")).unwrap() {
        let layer = layer.unwrap().path();
        if !layer.is_dir() || layer.ends_with("scenarios") {
            continue;
        }
        for f in std::fs::read_dir(&layer).unwrap() {
            let doc: serde_json::Value =
                serde_yaml_ng::from_str(&std::fs::read_to_string(f.unwrap().path()).unwrap())
                    .unwrap();
            for item in doc[key].as_array().into_iter().flatten() {
                if let Some(n) = item["name"].as_str() {
                    names.insert(n.to_string());
                }
            }
        }
    }
    names
}

#[test]
fn every_event_kind_the_simulator_records_is_in_the_table() {
    let recorded = recorded_kinds();
    assert!(recorded.len() > 30, "{recorded:?}");
    let known: BTreeSet<String> = TABLE.iter().map(|k| k.kind.to_string()).collect();
    let missing: Vec<&String> = recorded.difference(&known).collect();
    assert!(missing.is_empty(), "not in events::TABLE: {missing:?}");
    let stale: Vec<&String> = known.difference(&recorded).collect();
    assert!(
        stale.is_empty(),
        "in events::TABLE but never recorded: {stale:?}"
    );
}

#[test]
fn every_block_event_and_error_of_the_tables_is_declared_by_a_block() {
    let evs = declared("events");
    let errors = declared("errors");
    assert!(evs.contains("agentic.work.RunStarted"));
    for events::Kind {
        kind,
        events: blocks,
        ..
    } in TABLE
    {
        for b in *blocks {
            assert!(
                *b == REFUSAL || evs.contains(*b),
                "{kind}: {b} is no event of blocks/"
            );
        }
    }
    for (name, b) in EXPECTABLE {
        assert!(
            *b == REFUSAL || evs.contains(*b),
            "{name}: {b} is no event of blocks/"
        );
    }
    for (rule, error, _) in RULES {
        if let Some(e) = error {
            assert!(errors.contains(*e), "{rule}: {e} is no error of blocks/");
        }
    }
}

#[test]
fn every_block_event_the_table_reaches_has_one_expected_name() {
    let mut names = BTreeSet::new();
    let mut meant = BTreeSet::new();
    for (n, b) in EXPECTABLE {
        assert!(names.insert(*n), "{n} twice");
        assert!(meant.insert(*b), "{b} named twice");
    }
    for events::Kind {
        kind,
        events: blocks,
        ..
    } in TABLE
    {
        for b in *blocks {
            let name = events::name_of(b).unwrap_or_else(|| panic!("{kind}: {b} has no name"));
            assert!(events::modeled(name), "{name}");
        }
    }
    // A refused input carries the rule of its admission; every other refusal is its own rule.
    for events::Kind {
        kind,
        events: blocks,
        ..
    } in TABLE
    {
        if blocks.contains(&REFUSAL) && *kind != "input_refused" {
            assert!(
                RULES.iter().any(|(r, _, _)| r == kind),
                "{kind} refuses by no rule"
            );
        }
    }
    assert!(events::modeled("run_started"));
    assert!(!events::modeled("proposal_opened"));
    assert_eq!(
        events::block_events("question_forwarded"),
        Some(
            &[
                "agentic.question.QuestionAsked",
                "agentic.channel.MessageSent"
            ][..]
        )
    );
}

#[test]
fn the_scenarios_page_names_every_expected_event_and_every_rule() {
    let page = std::fs::read_to_string(repo_root().join("docs/scenarios.md")).unwrap();
    for (name, block) in EXPECTABLE {
        let row = if *block == REFUSAL {
            format!("| `{name}` |")
        } else {
            format!("| `{name}` | `{block}` |")
        };
        assert!(page.contains(&row), "docs/scenarios.md has no row {row}");
    }
    for (rule, error, _) in RULES {
        let cell = match error {
            Some(e) => format!("`{e}`"),
            None => "none".to_string(),
        };
        let row = format!("| `{rule}` |");
        let line = page.lines().find(|l| l.starts_with(&row));
        assert!(
            line.is_some_and(|l| l.contains(&cell)),
            "docs/scenarios.md has no row {row} with {cell}"
        );
    }
}

#[test]
fn an_on_event_trigger_waits_for_the_trigger_rows_of_the_table() {
    let triggers: Vec<(&str, &str)> = events::triggers().collect();
    assert_eq!(
        triggers,
        [
            ("agentic.work.TaskCreated", "task_arrived"),
            ("agentic.work.TaskCompleted", "task_done"),
            ("agentic.work.DeliverableRejected", "task_rejected"),
            ("agentic.question.QuestionAsked", "question_asked"),
            ("agentic.question.QuestionAnswered", "question_answered"),
            ("agentic.question.DefaultTaken", "question_defaulted"),
            ("agentic.resource.ClaimKilled", "build_killed"),
            ("agentic.trigger.FiringFailed", "firing_failed"),
        ]
    );
    for (event, kind) in &triggers {
        assert_eq!(events::block_events(kind), Some(&[*event][..]));
        assert_eq!(events::trigger_kind(event), Some(*kind));
    }
    assert_eq!(events::trigger_kind("agentic.work.RunStarted"), None);
}
