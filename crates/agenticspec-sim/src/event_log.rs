//! The event log of a simulated run (`agenticspec-sim-events/1`): each moment of the timeline
//! written as the block event a runtime would emit for it, with its source (architecture, part
//! path, copy) and its subject key, one JSON object per line. A reader of a runtime's event log
//! reads it the same way.
//!
//! Writing the log changes no result: it reads a recorded run and draws nothing.

use crate::sim::Stamp;
use serde_json::Value;
use std::collections::BTreeMap;

/// The format of a log line.
pub const FORMAT: &str = "agenticspec-sim-events/1";

/// The file `agenticspec sim run --events` writes beside `result.json`.
pub const FILE: &str = "events.jsonl";

/// Where the subject key of an event comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// The number of the simulated entity the moment is about (`Stamp::subject`): `task-3`.
    Subject,
    /// The part path: the part is the entity (a gateway's upstream, a schedule).
    Part,
    /// A new entity per event, numbered in the order of the log: `run-0`, `run-1`.
    Fresh,
}

/// One kind of the timeline and the block event it is written as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub kind: &'static str,
    /// Only at a part of this block; any part when absent.
    pub block: Option<&'static str>,
    pub event: &'static str,
    /// The event's key field, the identifier of its subject entity.
    pub key: &'static str,
    pub from: Key,
}

/// A kind written at a part of any block.
const fn e(kind: &'static str, event: &'static str, key: &'static str, from: Key) -> Entry {
    Entry {
        kind,
        block: None,
        event,
        key,
        from,
    }
}

/// A kind written only at a part of `block`.
const fn on(
    kind: &'static str,
    block: &'static str,
    event: &'static str,
    key: &'static str,
    from: Key,
) -> Entry {
    Entry {
        block: Some(block),
        ..e(kind, event, key, from)
    }
}

use Key::{Fresh, Part, Subject};

/// Every kind of the timeline that is a block event.
#[rustfmt::skip]
pub const LOG: &[Entry] = &[
    e("task_arrived", "agentic.work.TaskCreated", "task_id", Subject),
    e("task_started", "agentic.work.RunStarted", "run_id", Fresh),
    e("task_reworked", "agentic.work.RunStarted", "run_id", Fresh),
    e("review_started", "agentic.work.RunStarted", "run_id", Fresh),
    e("task_failed", "agentic.work.StepRecorded", "step_id", Fresh),
    e("task_claimed", "agentic.work.DeliverableSubmitted", "deliverable_id", Subject),
    e("task_rejected", "agentic.work.DeliverableRejected", "deliverable_id", Subject),
    e("task_handed", "agentic.work.AssignmentOffered", "assignment_id", Fresh),
    e("task_done", "agentic.work.TaskCompleted", "task_id", Subject),
    e("duplicate_dropped", "agentic.work.TaskCancelled", "task_id", Subject),
    e("question_asked", "agentic.question.QuestionAsked", "question_id", Subject),
    e("question_forwarded", "agentic.channel.MessageSent", "message_id", Fresh),
    e("question_answered", "agentic.question.QuestionAnswered", "question_id", Subject),
    e("question_defaulted", "agentic.question.DefaultTaken", "question_id", Subject),
    e("build_queued", "agentic.resource.ClaimQueued", "claim_id", Subject),
    e("build_started", "agentic.resource.ClaimGranted", "claim_id", Subject),
    e("build_done", "agentic.resource.ClaimReleased", "claim_id", Subject),
    e("build_killed", "agentic.resource.ClaimKilled", "claim_id", Subject),
    e("rate_limited", "agentic.gateway.CallDecided", "inference_id", Fresh),
    on("capacity_dropped", "gateway", "agentic.gateway.UpstreamSaturated", "upstream_id", Part),
    on("capacity_restored", "gateway", "agentic.gateway.UpstreamRestored", "upstream_id", Part),
    e("posted", "agentic.channel.Posted", "post_id", Subject),
    e("schedule_ticked", "agentic.trigger.ScheduleAdvanced", "schedule_id", Part),
    e("trigger_fired", "agentic.trigger.TriggerFired", "firing_id", Subject),
    e("firing_queued", "agentic.trigger.FiringQueued", "firing_id", Subject),
    e("firing_skipped", "agentic.trigger.FiringSkipped", "firing_id", Subject),
    e("firing_started", "agentic.trigger.FiringStarted", "firing_id", Subject),
    e("firing_failed", "agentic.trigger.FiringFailed", "firing_id", Subject),
    e("merge_conflict", "agentic.workspace.ProposalRejected", "proposal_id", Fresh),
    e("input_delivered", "agentic.channel.MessageSent", "message_id", Fresh),
];

/// Kinds of the timeline that no block event records, `(kind, block)`: a pool's capacity, a
/// principal's absence, a lost race for a board entry, a refused corpus entry or scenario input.
/// They stay on the timeline only.
pub const SILENT: &[(&str, Option<&str>)] = &[
    ("capacity_dropped", Some("pool")),
    ("capacity_restored", Some("pool")),
    ("principal_away", None),
    ("principal_back", None),
    ("claim_conflict", None),
    ("corpus_refused", None),
    ("input_refused", None),
];

fn matches(want: Option<&str>, block: Option<&str>) -> bool {
    want.is_none_or(|w| block == Some(w))
}

/// The entry a kind at a part of `block` is written as; `Ok(None)` for a silent kind.
pub fn entry_of(kind: &str, block: Option<&str>) -> Result<Option<&'static Entry>, String> {
    if let Some(e) = LOG
        .iter()
        .find(|e| e.kind == kind && matches(e.block, block))
    {
        return Ok(Some(e));
    }
    if SILENT.iter().any(|(k, b)| *k == kind && matches(*b, block)) {
        return Ok(None);
    }
    Err(format!(
        "timeline kind `{kind}` at a part of block `{}` is neither a block event nor silent",
        block.unwrap_or("-")
    ))
}

/// One line of the log.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// The cursor: 1 for the first event, in the order of the timeline.
    pub seq: u64,
    pub t_seconds: f64,
    /// The block event's qualified name.
    pub event: &'static str,
    pub architecture: String,
    /// The part path.
    pub part: String,
    /// The copy of the part, for a worker or reviewer copy.
    pub instance: Option<usize>,
    /// The event's key field.
    pub key: &'static str,
    /// Its value, the subject key.
    pub subject_key: String,
}

fn text(s: &str) -> String {
    Value::from(s).to_string()
}

impl Event {
    /// The line as written, without its newline. The payload holds the fields the simulator
    /// knows: the key field.
    pub fn line(&self) -> String {
        let instance = self
            .instance
            .map_or(String::new(), |i| format!(",\"instance\":{i}"));
        format!(
            "{{\"seq\":{},\"t_seconds\":{},\"event\":{},\"architecture\":{},\"part\":{}{instance},\"subject_key\":{},\"payload\":{{{}:{}}}}}",
            self.seq,
            Value::from(self.t_seconds),
            text(self.event),
            text(&self.architecture),
            text(&self.part),
            text(&self.subject_key),
            text(self.key),
            text(&self.subject_key),
        )
    }
}

/// The events of a recorded run of `architecture` (its resolved document), in timeline order.
/// Refuses a kind with no entry and a keyed kind recorded without its subject.
pub fn log(architecture: &Value, timeline: &[Stamp]) -> Result<Vec<Event>, String> {
    let name = architecture["architecture"]
        .as_str()
        .unwrap_or("architecture")
        .to_string();
    let mut fresh: BTreeMap<&str, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for m in timeline {
        let block = architecture["parts"][m.part.as_str()]["block"].as_str();
        let Some(e) = entry_of(m.kind, block)? else {
            continue;
        };
        let entity = e.key.strip_suffix("_id").unwrap_or(e.key);
        let subject_key = match e.from {
            Key::Part => m.part.clone(),
            Key::Subject => match m.subject {
                Some(n) => format!("{entity}-{n}"),
                None => {
                    return Err(format!(
                        "timeline kind `{}` at `{}` has no subject for `{}`",
                        m.kind, m.part, e.key
                    ));
                }
            },
            Key::Fresh => {
                let n = fresh.entry(e.key).or_default();
                *n += 1;
                format!("{entity}-{}", *n - 1)
            }
        };
        out.push(Event {
            seq: out.len() as u64 + 1,
            t_seconds: m.t,
            event: e.event,
            architecture: name.clone(),
            part: m.part.clone(),
            instance: m.copy,
            key: e.key,
            subject_key,
        });
    }
    Ok(out)
}

/// The log as written: one line per event, each ended by a newline.
pub fn jsonl(events: &[Event]) -> String {
    let mut s = String::new();
    for e in events {
        s.push_str(&e.line());
        s.push('\n');
    }
    s
}
