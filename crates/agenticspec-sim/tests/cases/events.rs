//! The event log of a simulated run: every event it writes is declared in `blocks/` with its
//! key field, the same seed writes the same bytes, and asking for it changes no result.

use super::common::{ARCH, PROFILE, experiment_yaml, repo_root, scratch};
use agenticspec_sim::doc::load_yaml;
use agenticspec_sim::events::{Event, LOG, SILENT, entry_of, jsonl, log};
use agenticspec_sim::sim::{EVENTS, Mark};
use agenticspec_sim::{Experiment, cli};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Every event of the ESS system `agentic` with the names of its fields.
fn block_events() -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();
    let blocks = repo_root().join("blocks");
    let mut files = Vec::new();
    for layer in std::fs::read_dir(&blocks).unwrap().flatten() {
        if layer.path().is_dir() {
            for f in std::fs::read_dir(layer.path()).unwrap().flatten() {
                if f.path().extension().is_some_and(|x| x == "yaml") {
                    files.push(f.path());
                }
            }
        }
    }
    for f in files {
        let doc = load_yaml(&f).unwrap();
        for e in doc["events"].as_array().into_iter().flatten() {
            let fields = e["fields"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|x| x["name"].as_str().map(String::from))
                .collect();
            out.insert(e["name"].as_str().unwrap().to_string(), fields);
        }
    }
    assert!(out.len() > 100, "blocks/ declares {} events", out.len());
    out
}

fn experiments() -> Vec<Experiment> {
    let dir = repo_root().join("experiments");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    paths.iter().map(|p| Experiment::load(p).unwrap()).collect()
}

fn small() -> Experiment {
    let y = |t: &str| agenticspec_sim::doc::parse_yaml(t).unwrap();
    Experiment::from_docs(y(&experiment_yaml("fit", 4, 256)), y(ARCH), y(PROFILE)).unwrap()
}

fn logged(e: &Experiment) -> (Vec<Mark>, Vec<Event>) {
    let shown = e.baseline_shown().unwrap();
    let events = log(&shown.architecture, &shown.run.timeline).unwrap();
    (shown.run.timeline, events)
}

#[test]
fn every_entry_names_a_block_event_and_one_of_its_fields() {
    let declared = block_events();
    for e in LOG {
        let fields = declared
            .get(e.event)
            .unwrap_or_else(|| panic!("`{}` ({}) is not an event of blocks/", e.event, e.kind));
        assert!(
            fields.contains(e.key),
            "`{}` has no field `{}`: {fields:?}",
            e.event,
            e.key
        );
    }
    for (event, kind) in EVENTS {
        let e = LOG.iter().find(|e| e.kind == *kind).unwrap();
        assert_eq!(
            e.event, *event,
            "an on_event trigger and the log disagree on `{kind}`"
        );
    }
    for (kind, block) in SILENT {
        assert!(
            !LOG.iter().any(|e| e.kind == *kind && e.block == *block),
            "`{kind}` is both written and silent"
        );
    }
}

#[test]
fn every_event_written_is_declared_in_the_blocks() {
    let declared = block_events();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for e in experiments().iter().chain([&small()]) {
        let (timeline, events) = logged(e);
        assert!(!events.is_empty(), "{}", e.doc["experiment"]);
        seen.extend(timeline.iter().map(|m| m.kind));
        for line in jsonl(&events).lines() {
            let v: Value = serde_json::from_str(line).unwrap();
            let name = v["event"].as_str().unwrap();
            let fields = declared
                .get(name)
                .unwrap_or_else(|| panic!("written `{name}` is not an event of blocks/"));
            let payload = v["payload"].as_object().unwrap();
            for (field, value) in payload {
                assert!(fields.contains(field), "`{name}` has no field `{field}`");
                assert_eq!(value, &v["subject_key"], "{line}");
            }
            assert!(
                v["part"].is_string() && v["architecture"].is_string(),
                "{line}"
            );
        }
    }
    let known: BTreeSet<&str> = LOG
        .iter()
        .map(|e| e.kind)
        .chain(SILENT.iter().map(|(k, _)| *k))
        .collect();
    // The baselines of the catalog and the small experiment reach every other kind.
    const UNREACHED: &[&str] = &[
        "claim_conflict",
        "duplicate_dropped",
        "firing_queued",
        "rate_limited",
    ];
    let unreached: Vec<&&str> = known
        .difference(&seen)
        .filter(|k| !UNREACHED.contains(k))
        .collect();
    assert!(unreached.is_empty(), "no run reaches {unreached:?}");
}

#[test]
fn the_log_keeps_order_and_ties_each_subject_to_its_start() {
    let (_, events) = logged(&small());
    let mut opened: BTreeSet<&str> = BTreeSet::new();
    let mut last = 0.0;
    for (n, e) in events.iter().enumerate() {
        assert_eq!(e.seq, n as u64 + 1);
        assert!(e.t_seconds >= last, "{e:?}");
        last = e.t_seconds;
        let starts = [
            "agentic.work.TaskCreated",
            "agentic.question.QuestionAsked",
            "agentic.resource.ClaimQueued",
            "agentic.resource.ClaimGranted",
            "agentic.trigger.TriggerFired",
            "agentic.work.DeliverableSubmitted",
        ];
        if starts.contains(&e.event) {
            opened.insert(&e.subject_key);
        } else if matches!(
            e.key,
            "task_id" | "question_id" | "claim_id" | "deliverable_id"
        ) {
            assert!(
                opened.contains(e.subject_key.as_str()),
                "{e:?} before its start"
            );
        }
    }
    for kind in ["question_id", "claim_id", "task_id"] {
        assert!(events.iter().any(|e| e.key == kind), "no `{kind}` event");
    }
}

#[test]
fn a_kind_outside_the_table_or_without_its_subject_is_refused() {
    assert!(entry_of("no_such_kind", None).is_err());
    assert_eq!(entry_of("capacity_dropped", Some("pool")).unwrap(), None);
    let arch: Value = serde_json::json!({"architecture": "a", "parts": {"w": {"block": "agent"}}});
    let mark = |kind, subject| Mark {
        t: 1.0,
        part: "w".into(),
        kind,
        subject,
        copy: Some(0),
    };
    assert!(log(&arch, &[mark("task_done", None)]).is_err());
    let ok = log(
        &arch,
        &[mark("task_done", Some(3)), mark("task_started", None)],
    )
    .unwrap();
    assert_eq!(
        jsonl(&ok),
        "{\"seq\":1,\"t_seconds\":1.0,\"event\":\"agentic.work.TaskCompleted\",\"architecture\":\"a\",\"part\":\"w\",\"instance\":0,\"subject_key\":\"task-3\",\"payload\":{\"task_id\":\"task-3\"}}\n\
         {\"seq\":2,\"t_seconds\":1.0,\"event\":\"agentic.work.RunStarted\",\"architecture\":\"a\",\"part\":\"w\",\"instance\":0,\"subject_key\":\"run-0\",\"payload\":{\"run_id\":\"run-0\"}}\n"
    );
}

fn run_into(dir: &Path, name: &str, events: bool) -> Value {
    cli::run(
        &dir.join("exp.yaml"),
        &dir.join(name),
        Some(2),
        events,
        &mut Vec::new(),
    )
    .unwrap()
}

#[test]
fn the_flag_writes_the_same_log_per_seed_and_changes_no_result() {
    let dir = scratch("events");
    std::fs::write(dir.join("small.yaml"), ARCH).unwrap();
    std::fs::write(dir.join("small-profile.yaml"), PROFILE).unwrap();
    std::fs::write(dir.join("exp.yaml"), experiment_yaml("fit", 4, 256)).unwrap();
    let plain = run_into(&dir, "plain", false);
    let a = run_into(&dir, "a", true);
    let b = run_into(&dir, "b", true);
    assert!(plain.get("events").is_none(), "{plain}");
    assert!(!dir.join("plain/events.jsonl").exists());
    let read = |p: &str| std::fs::read_to_string(dir.join(p)).unwrap();
    assert_eq!(read("plain/result.json"), read("a/result.json"));
    assert_eq!(read("a/events.jsonl"), read("b/events.jsonl"));
    let n = read("a/events.jsonl").lines().count() as u64;
    assert!(n > 0);
    assert_eq!(a["events"].as_u64(), Some(n));
    assert_eq!(b["events"].as_u64(), Some(n));
}
