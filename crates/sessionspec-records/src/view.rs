//! The renderer's document (`sessionspec-view/1`, spec domain `sessionspec.view`): one session's
//! published events as a flat list a `<session-view>` element draws, and the update a live source
//! sends when the transcript grows.

use std::collections::HashMap;
use std::path::Path;

use sessionspec_system::SystemEvent;
use sessionspec_types::view as v;

use crate::time::epoch_ms;

pub const FORMAT: &str = "sessionspec-view/1";

const MINUTE: i64 = 60_000;
const BUST_FLOOR: i64 = 20_000;

fn name<T: std::fmt::Debug>(x: T) -> String {
    let d = format!("{x:?}");
    d.split([' ', '{', '(']).next().unwrap_or("").to_owned()
}

fn event(seq: usize, at: i64, agent: &str, kind: v::EventKind, label: String) -> v::Event {
    v::Event {
        seq: seq as i64,
        at,
        agent: agent.to_owned(),
        kind,
        label,
        detail: None,
        call: None,
        ok: None,
        context_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        cost_microdollars: None,
        bytes: None,
    }
}

/// The document of one main transcript.
pub fn document(main: &Path) -> std::io::Result<v::Document> {
    let lifted = crate::claude::lift(main)?;
    let replayed = crate::replay::replay(&lifted.steps);
    let project = main
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_owned();
    Ok(from_events(
        &lifted.session_id,
        lifted.title.clone(),
        project,
        &replayed.events,
    ))
}

pub fn from_events(
    session_id: &str,
    title: Option<String>,
    project: String,
    events: &[SystemEvent],
) -> v::Document {
    let mut p = Projector::default();
    let mut out: Vec<v::Event> = Vec::new();
    for e in events {
        p.push(e, &mut out);
    }
    v::Document {
        format: FORMAT.to_owned(),
        session_id: session_id.to_owned(),
        title,
        project,
        harness: "ClaudeCode".to_owned(),
        agents: p.agents,
        events: out,
    }
}

/// The view projection as a fold: one published event at a time, so a live source projects only
/// what a session appended. [`from_events`] is this fold over a whole session.
#[derive(Debug, Clone, Default)]
pub struct Projector {
    /// The agents started so far, in order.
    pub agents: Vec<v::Agent>,
    /// The view events projected so far; the next one's `seq`.
    pub seq: usize,
    call_agent: HashMap<String, String>,
    // Per agent: previous turn (context, at, model, 1-hour writes) and a compaction since.
    prev: HashMap<String, (i64, i64, String, bool, bool)>,
}

impl Projector {
    /// Projects one published event, appending its view events (none, one, or a turn and a
    /// cache bust) to `sink`.
    pub fn push(&mut self, e: &SystemEvent, sink: &mut Vec<v::Event>) {
        let mut out = Vec::new();
        self.step(e, &mut out);
        sink.extend(out);
    }

    fn step(&mut self, e: &SystemEvent, out: &mut Vec<v::Event>) {
        use SystemEvent as E;
        let agents = &mut self.agents;
        let call_agent = &mut self.call_agent;
        let prev = &mut self.prev;
        let base = self.seq;
        let ts = |t: &str| epoch_ms(t).unwrap_or(0);
        {
            let n = base + out.len();
            match e {
                E::AgentStarted(x) => {
                    let at = ts(&x.at.0);
                    agents.push(v::Agent {
                        id: x.agent_id.0.clone(),
                        parent: x.parent_agent_id.as_ref().map(|p| p.0.clone()),
                        role: name(x.role),
                        spawned_by: x.spawned_by.as_ref().map(|c| c.0.clone()),
                        started_at: at,
                    });
                    let mut ev = event(
                        n,
                        at,
                        &x.agent_id.0,
                        v::EventKind::AgentStarted,
                        name(x.role),
                    );
                    ev.call = x.spawned_by.as_ref().map(|c| c.0.clone());
                    out.push(ev);
                }
                E::PromptSubmitted(x) => {
                    let mut ev = event(
                        n,
                        ts(&x.at.0),
                        &x.agent_id.0,
                        v::EventKind::Prompt,
                        name(x.author),
                    );
                    ev.detail = Some(format!("{} {}", name(x.class), name(x.attribution)));
                    ev.bytes = Some(x.content.bytes);
                    out.push(ev);
                }
                E::ModelTurnRecorded(x) => {
                    let at = ts(&x.at.0);
                    let u = &x.usage;
                    let write = u.cache_write_5m + u.cache_write_1h;
                    let mut ev = event(
                        n,
                        at,
                        &x.agent_id.0,
                        v::EventKind::ModelTurn,
                        x.model.clone(),
                    );
                    ev.context_tokens = Some(x.context_tokens);
                    ev.output_tokens = Some(u.output);
                    ev.cache_read_tokens = Some(u.cache_read);
                    ev.cache_write_tokens = Some(write);
                    ev.cost_microdollars =
                        crate::pricing::cost_nanodollars(&x.model, u).map(|c| c / 1000);
                    out.push(ev);
                    let p = prev.get(&x.agent_id.0).cloned();
                    if let Some((pctx, pt, pmodel, p1h, compacted)) = p
                        && pctx > BUST_FLOOR
                        && u.cache_read * 2 < pctx
                        && write * 2 > pctx
                    {
                        let ttl = if p1h { 60 * MINUTE } else { 5 * MINUTE };
                        let cause = if compacted {
                            "compacted"
                        } else if pmodel != x.model {
                            "model_changed"
                        } else if at - pt > ttl {
                            "ttl_expired"
                        } else {
                            "unexplained"
                        };
                        let mut b = event(
                            base + out.len(),
                            at,
                            &x.agent_id.0,
                            v::EventKind::CacheBust,
                            cause.to_owned(),
                        );
                        b.cache_write_tokens = Some(write);
                        out.push(b);
                    }
                    prev.insert(
                        x.agent_id.0.clone(),
                        (
                            x.context_tokens,
                            at,
                            x.model.clone(),
                            u.cache_write_1h > u.cache_write_5m,
                            false,
                        ),
                    );
                }
                E::ContextCompacted(x) => {
                    if let Some(p) = prev.get_mut(&x.agent_id.0) {
                        p.4 = true;
                    }
                    let mut ev = event(
                        n,
                        ts(&x.at.0),
                        &x.agent_id.0,
                        v::EventKind::Compaction,
                        "compaction".to_owned(),
                    );
                    ev.context_tokens = Some(x.context_tokens_before);
                    out.push(ev);
                }
                E::ToolCallRequested(x) => {
                    call_agent.insert(x.tool_call_id.0.clone(), x.agent_id.0.clone());
                    let mut ev = event(
                        n,
                        ts(&x.at.0),
                        &x.agent_id.0,
                        v::EventKind::ToolCall,
                        x.tool.clone(),
                    );
                    ev.detail = Some(name(x.category));
                    ev.call = Some(x.tool_call_id.0.clone());
                    ev.bytes = Some(x.input.bytes);
                    out.push(ev);
                }
                E::ToolCallDecided(x) => {
                    let agent = call_agent
                        .get(&x.tool_call_id.0)
                        .cloned()
                        .unwrap_or_default();
                    let mut ev = event(
                        n,
                        ts(&x.at.0),
                        &agent,
                        v::EventKind::Decision,
                        name(x.decider),
                    );
                    ev.detail = Some(name(x.decision));
                    ev.call = Some(x.tool_call_id.0.clone());
                    ev.ok = Some(x.decision == sessionspec_types::session::Decision::Accept);
                    out.push(ev);
                }
                E::ToolCallCompleted(x) => {
                    let agent = call_agent
                        .get(&x.tool_call_id.0)
                        .cloned()
                        .unwrap_or_default();
                    let mut ev = event(
                        n,
                        ts(&x.at.0),
                        &agent,
                        v::EventKind::ToolResult,
                        if x.ok { "ok" } else { "error" }.to_owned(),
                    );
                    ev.call = Some(x.tool_call_id.0.clone());
                    ev.ok = Some(x.ok);
                    ev.bytes = Some(x.output.bytes);
                    out.push(ev);
                }
                E::TurnEnded(x) => out.push(event(
                    n,
                    ts(&x.at.0),
                    &x.agent_id.0,
                    v::EventKind::TurnEnd,
                    "turn end".to_owned(),
                )),
                E::ExposureDetected(x) => {
                    let mut ev = event(
                        n,
                        ts(&x.at.0),
                        &x.agent_id.0,
                        v::EventKind::Exposure,
                        x.kind.clone(),
                    );
                    ev.detail = Some(format!("{} {}", name(x.channel), name(x.verdict)));
                    ev.call = x.tool_call_id.as_ref().map(|c| c.0.clone());
                    out.push(ev);
                }
                _ => {}
            }
        }
        self.seq = base + out.len();
    }
}

/// The update that brings a renderer holding `old` to `new`: the events after the common prefix,
/// or everything with `reset` when an event the renderer holds changed.
pub fn update(old: &v::Document, new: &v::Document) -> v::Update {
    let same = old.events.len() <= new.events.len()
        && old.events.iter().zip(&new.events).all(|(a, b)| a == b);
    let from = if same { old.events.len() } else { 0 };
    v::Update {
        format: FORMAT.to_owned(),
        session_id: new.session_id.clone(),
        reset: !same,
        from_seq: from as i64,
        agents: new.agents.clone(),
        events: new.events[from..].to_vec(),
    }
}
