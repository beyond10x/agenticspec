//! Replays lifted commands through the generated `session-ir` component and collects what it
//! publishes. The storage ports are the only part written here: the generated behaviours decide
//! every outcome.

use std::collections::BTreeMap;

use agentspec_system::{System, SystemEvent};
use agentspec_types::behaviour::{
    AgentListingStorage, AgentStorage, EventRowStorage, ExposureStorage, FigureBucketStorage,
    Generated, SessionCheckStorage, SessionListingStorage, SessionStorage, TimelineSlotStorage,
    ToolCallStorage,
};
use agentspec_types::session as s;
use agentspec_types::view as v;
use session_ir::SessionIr;
use session_view::SessionView;

use crate::claude::{Cmd, Step};

/// In-memory storage, one ordered map per entity, keyed by the identity's text.
#[derive(Default)]
pub struct Ports {
    agents: BTreeMap<String, s::AgentSnapshot>,
    exposures: BTreeMap<String, s::ExposureSnapshot>,
    sessions: BTreeMap<String, s::SessionSnapshot>,
    tool_calls: BTreeMap<String, s::ToolCallSnapshot>,
    agent_listings: BTreeMap<String, v::AgentListingSnapshot>,
    event_rows: BTreeMap<String, v::EventRowSnapshot>,
    session_listings: BTreeMap<String, v::SessionListingSnapshot>,
    session_checks: BTreeMap<String, v::SessionCheckSnapshot>,
    figure_buckets: BTreeMap<String, v::FigureBucketSnapshot>,
    timeline_slots: BTreeMap<String, v::TimelineSlotSnapshot>,
}

macro_rules! storage {
    ($trait:ident, $field:ident, $id:ty, $snap:ty, $key:ident) => {
        impl $trait for Ports {
            fn get(&self, identity: &$id) -> Option<$snap> {
                self.$field.get(&identity.0).cloned()
            }
            fn put(&mut self, snapshot: $snap) {
                self.$field.insert(snapshot.data.$key.0.clone(), snapshot);
            }
            fn delete(&mut self, identity: &$id) {
                self.$field.remove(&identity.0);
            }
            fn list(&self) -> Vec<$snap> {
                self.$field.values().cloned().collect()
            }
        }
    };
}

storage!(AgentStorage, agents, s::AgentId, s::AgentSnapshot, agent_id);
storage!(
    ExposureStorage,
    exposures,
    s::ExposureId,
    s::ExposureSnapshot,
    exposure_id
);
storage!(
    SessionStorage,
    sessions,
    s::SessionId,
    s::SessionSnapshot,
    session_id
);
storage!(
    ToolCallStorage,
    tool_calls,
    s::ToolCallId,
    s::ToolCallSnapshot,
    tool_call_id
);
storage!(
    AgentListingStorage,
    agent_listings,
    s::AgentId,
    v::AgentListingSnapshot,
    agent_id
);
storage!(
    EventRowStorage,
    event_rows,
    v::EventId,
    v::EventRowSnapshot,
    event_id
);
storage!(
    SessionListingStorage,
    session_listings,
    s::SessionId,
    v::SessionListingSnapshot,
    session_id
);
storage!(
    SessionCheckStorage,
    session_checks,
    v::CheckId,
    v::SessionCheckSnapshot,
    check_id
);
storage!(
    FigureBucketStorage,
    figure_buckets,
    v::BucketId,
    v::FigureBucketSnapshot,
    bucket_id
);
storage!(
    TimelineSlotStorage,
    timeline_slots,
    v::SlotId,
    v::TimelineSlotSnapshot,
    slot_id
);

/// Replay serves no view of `session-view`: its listing query answers nothing.
impl v::obligations::SessionListQuery for Ports {
    fn session_list(
        &self,
        _since: Option<agentspec_types::primitives::Timestamp>,
        _search: Option<String>,
        _limit: Option<i64>,
        _offset: Option<i64>,
    ) -> Result<Vec<v::SessionList>, agentspec_types::obligation::UnmetObligation> {
        Ok(Vec::new())
    }
}

impl v::obligations::SessionEventsQuery for Ports {
    fn session_events(
        &self,
        _session_id: s::SessionId,
        _agent: Option<s::AgentId>,
        _kind: Option<v::EventKind>,
        _notable: Option<bool>,
        _before: Option<i64>,
    ) -> Result<Vec<v::SessionEvents>, agentspec_types::obligation::UnmetObligation> {
        Ok(Vec::new())
    }
}

impl v::obligations::SessionEventCountsQuery for Ports {
    fn session_event_counts(
        &self,
        _session_id: s::SessionId,
        _agent: Option<s::AgentId>,
    ) -> Result<Vec<v::SessionEventCounts>, agentspec_types::obligation::UnmetObligation> {
        Ok(Vec::new())
    }
}

/// One replayed command: what was sent and the outcome the component answered.
pub struct Logged {
    pub command: &'static str,
    pub outcome: String,
    pub refused: bool,
}

pub struct Replayed {
    pub events: Vec<SystemEvent>,
    pub log: Vec<Logged>,
    pub refused: usize,
}

/// The variant name of a `Debug` rendering: `WrongState { .. }` is `WrongState`.
fn variant(debug: &str) -> String {
    debug.split([' ', '{', '(']).next().unwrap_or("").to_owned()
}

pub fn command_name(cmd: &Cmd) -> &'static str {
    match cmd {
        Cmd::OpenSession(_) => "OpenSession",
        Cmd::StartAgent(_) => "StartAgent",
        Cmd::SubmitPrompt(_) => "SubmitPrompt",
        Cmd::EndTurn(_) => "EndTurn",
        Cmd::RecordModelTurn(_) => "RecordModelTurn",
        Cmd::CompactContext(_) => "CompactContext",
        Cmd::RequestToolCall(_) => "RequestToolCall",
        Cmd::DecideToolCall(_) => "DecideToolCall",
        Cmd::CompleteToolCall(_) => "CompleteToolCall",
        Cmd::RecordExposure(_) => "RecordExposure",
    }
}

pub fn replay(steps: &[Step]) -> Replayed {
    let mut r = Replayer::new();
    let mut log = Vec::with_capacity(steps.len());
    let mut events = Vec::new();
    for step in steps {
        log.push(r.push(step, &mut events));
    }
    Replayed {
        events,
        log,
        refused: r.refused,
    }
}

/// The replay as a fold: the generated system kept between steps, so a live source replays only
/// the steps a session appended. [`replay`] is this fold over a whole session.
pub struct Replayer {
    system: System<Generated<Ports>, Generated<Ports>>,
    /// Commands the component refused so far.
    pub refused: usize,
}

impl Default for Replayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Replayer {
    pub fn new() -> Replayer {
        Replayer {
            system: System::new(
                SessionIr::new(Generated::new(Ports::default())),
                SessionView::new(Generated::new(Ports::default())),
            ),
            refused: 0,
        }
    }

    /// Sends one step through the component and appends what it published to `events`.
    pub fn push(&mut self, step: &Step, events: &mut Vec<SystemEvent>) -> Logged {
        let system = &mut self.system;
        let ir = &mut system.session_ir;
        let answer = match step.cmd.clone() {
            Cmd::OpenSession(i) => ir.open_session(i).map(|o| format!("{o:?}")),
            Cmd::StartAgent(i) => ir.start_agent(i).map(|o| format!("{o:?}")),
            Cmd::SubmitPrompt(i) => ir.submit_prompt(i).map(|o| format!("{o:?}")),
            Cmd::EndTurn(i) => ir.end_turn(i).map(|o| format!("{o:?}")),
            Cmd::RecordModelTurn(i) => ir.record_model_turn(i).map(|o| format!("{o:?}")),
            Cmd::CompactContext(i) => ir.compact_context(i).map(|o| format!("{o:?}")),
            Cmd::RequestToolCall(i) => ir.request_tool_call(i).map(|o| format!("{o:?}")),
            Cmd::DecideToolCall(i) => ir.decide_tool_call(i).map(|o| format!("{o:?}")),
            Cmd::CompleteToolCall(i) => ir.complete_tool_call(i).map(|o| format!("{o:?}")),
            Cmd::RecordExposure(i) => ir.record_exposure(i).map(|o| format!("{o:?}")),
        };
        let outcome = match answer {
            Ok(debug) => variant(&debug),
            Err(e) => format!("Unmet({e:?})"),
        };
        let is_refused = matches!(outcome.as_str(), "WrongState" | "NoSuchAgent")
            || outcome.starts_with("Unmet");
        if is_refused {
            self.refused += 1;
        }
        if system.pump().is_ok() {
            events.extend(system.take_published());
        }
        Logged {
            command: command_name(&step.cmd),
            outcome,
            refused: is_refused,
        }
    }
}
