//! The replay of a simulated run for `agenticspec ui serve --sim`: the state its event log leaves
//! in the views of the blocks, each row at the places its events happened, and the events with
//! their cursors.

use super::common::repo_root;
use agenticspec_sim::event_log::{Event, LOG};
use agenticspec_sim::replay::{replay, replay_in, run};
use agenticspec_ui::serve::At;
use serde_json::{Value, json};
use std::collections::BTreeSet;

const BLOCK: &str = r"
entities:
  - name: agentic.thing.Item
    identity: {name: item_id, type: agentic.thing.ItemId}
    lifecycle:
      initial: Open
      states: [Open, Done]
      transitions:
        - {name: finish, from: [Open], to: Done}
  - name: agentic.thing.Pool
    identity: {name: pool_id, type: agentic.thing.PoolId}
commands:
  - name: agentic.thing.OpenItem
    outcomes:
      - {name: opened, creates: agentic.thing.Item, emits: [agentic.thing.ItemOpened]}
  - name: agentic.thing.FinishItem
    outcomes:
      - name: finished
        updates: agentic.thing.Pool
        affects:
          - {entity: agentic.thing.Item, moves: agentic.thing.Item.finish}
        emits: [agentic.thing.ItemFinished]
  - name: agentic.thing.TouchItem
    outcomes:
      - {name: touched, updates: agentic.thing.Item, emits: [agentic.thing.ItemTouched]}
events:
  - name: agentic.thing.ItemOpened
  - name: agentic.thing.ItemFinished
  - name: agentic.thing.ItemTouched
views:
  - name: agentic.thing.Items
    source: agentic.thing.Item
    fields: [{name: item_id}, {name: label}, {name: state}]
  - name: agentic.thing.OpenItems
    source: agentic.thing.Item
    filter: {all: [state == Open, owner == param.owner]}
    fields: [{name: item_id}, {name: label}]
  - name: agentic.thing.NotOpen
    source: agentic.thing.Item
    filter: state != Open
    fields: [{name: item_id}, {name: state}]
  - name: agentic.thing.ByLabel
    source: agentic.thing.Item
    fields: [{name: label}, {name: item_id}]
";

fn event(seq: u64, name: &'static str, part: &str, copy: Option<usize>, subject: &str) -> Event {
    Event {
        seq,
        t_seconds: seq as f64,
        event: name,
        architecture: "a".into(),
        part: part.into(),
        instance: copy,
        key: "item_id",
        subject_key: subject.into(),
    }
}

fn rows(r: &agenticspec_ui::serve::Replay, view: &str) -> Vec<Value> {
    r.views[view]
        .iter()
        .map(|x| serde_json::to_value(&x.row).unwrap())
        .collect()
}

fn at(part: &str, instance: Option<u32>) -> At {
    At {
        part: part.into(),
        instance,
    }
}

/// `creates` opens an entity in its initial state, `moves` under `affects` moves it, `updates`
/// leaves its state; a view keeps its `state ==` and `state !=` rows, holds its identity first
/// and its state when it declares one, and a view whose first field is not the identity answers
/// nothing. A row remembers every place its events happened, a copy counted from 1.
#[test]
fn the_state_of_a_log_is_folded_by_the_outcomes_that_emit_its_events() {
    let log = [
        event(1, "agentic.thing.ItemOpened", "worker", Some(0), "item-0"),
        event(2, "agentic.thing.ItemOpened", "worker", Some(2), "item-1"),
        event(
            3,
            "agentic.thing.ItemFinished",
            "team.reviewer",
            None,
            "item-0",
        ),
        event(4, "agentic.thing.ItemTouched", "worker", Some(2), "item-1"),
    ];
    let r = replay_in([BLOCK], &log).unwrap();
    assert_eq!(
        rows(&r, "agentic.thing.Items"),
        [
            json!({"item_id": "item-0", "state": "Done"}),
            json!({"item_id": "item-1", "state": "Open"}),
        ]
    );
    assert_eq!(
        rows(&r, "agentic.thing.OpenItems"),
        [json!({"item_id": "item-1"})]
    );
    assert_eq!(
        rows(&r, "agentic.thing.NotOpen"),
        [json!({"item_id": "item-0", "state": "Done"})]
    );
    assert!(!r.views.contains_key("agentic.thing.ByLabel"));
    // The key comes first, as the view declares it.
    let first = &r.views["agentic.thing.Items"][0];
    assert_eq!(first.row.0[0].0, "item_id");
    assert_eq!(first.at, [at("worker", Some(1)), at("team.reviewer", None)]);
    assert_eq!(
        r.views["agentic.thing.Items"][1].at,
        [at("worker", Some(3))]
    );
    let seqs: Vec<u64> = r.events.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, [1, 2, 3, 4]);
    assert_eq!(r.events[1].payload, json!({"item_id": "item-1"}));
    assert_eq!(r.events[1].at, at("worker", Some(3)));
    assert_eq!(r.channels, BTreeSet::from(["thing".to_string()]));
}

/// Every block event the simulator writes names an entity of the blocks built in by its key field,
/// and some view over that entity answers the subject.
#[test]
fn every_event_the_simulator_writes_lands_in_a_view_of_the_blocks() {
    let log: Vec<Event> = LOG
        .iter()
        .enumerate()
        .map(|(i, e)| Event {
            seq: i as u64 + 1,
            t_seconds: 0.0,
            event: e.event,
            architecture: "a".into(),
            part: "p".into(),
            instance: None,
            key: e.key,
            subject_key: format!("{}-{i}", e.key),
        })
        .collect();
    let r = replay(&log).unwrap();
    for e in &log {
        let found = r
            .views
            .values()
            .flatten()
            .any(|row| row.row.0.first() == Some(&(e.key.to_string(), json!(e.subject_key))));
        assert!(found, "{} ({}) lands in no view", e.event, e.key);
    }
}

/// A run of the workstation under the coding profile: its replay holds the run's events in the
/// order of the log, and the rows of its questions and claims are the subjects the log names, each
/// in a state of its entity's lifecycle.
#[test]
fn a_run_of_the_workstation_replays_its_log_and_its_state() {
    let root = repo_root();
    let r = run(
        &root.join("architectures/workstation.yaml"),
        &root.join("fields/coding/profiles/coding.yaml"),
    )
    .unwrap();
    assert!(r.events.len() > 10, "{} events", r.events.len());
    let seqs: Vec<u64> = r.replay.events.iter().map(|e| e.seq).collect();
    let want: Vec<u64> = (1..=r.events.len() as u64).collect();
    assert_eq!(seqs, want);
    let blocks = agenticspec_sim::rows::blocks();
    for (view, entity, key) in [
        (
            "agentic.question.Questions",
            "agentic.question.Question",
            "question_id",
        ),
        (
            "agentic.resource.Claims",
            "agentic.resource.Claim",
            "claim_id",
        ),
    ] {
        let subjects: BTreeSet<&str> = r
            .events
            .iter()
            .filter(|e| e.key == key)
            .map(|e| e.subject_key.as_str())
            .collect();
        assert!(!subjects.is_empty(), "the run names no {key}");
        let got: BTreeSet<String> = r.replay.views[view]
            .iter()
            .map(|x| x.row.str(key).unwrap().to_string())
            .collect();
        let want: BTreeSet<String> = subjects.iter().map(|s| s.to_string()).collect();
        assert_eq!(got, want, "{view}");
        let states = &blocks.entities[entity].states;
        for x in &r.replay.views[view] {
            let s = x
                .row
                .str("state")
                .unwrap_or_else(|| panic!("{view}: {:?}", x.row));
            assert!(states.iter().any(|t| t == s), "{view}: state {s}");
        }
    }
    let result = r.result();
    assert_eq!(result["format"], "agenticspec-sim-result/1");
    assert!(result["baseline"]["kpis"]["cost_usd"]["value"].is_number());
}
