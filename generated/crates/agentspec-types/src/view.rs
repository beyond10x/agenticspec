// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! view — `agentspec.view`.
//!
//! What a session renderer reads: one session as agents and a flat, ordered list of view events derived from the session IR's published events, and the read models of the UI (`ui/agentspec.ui.yaml`): sessions, agents, events and check verdicts. It holds no content, only names, counts and times, except the harness's session title, which stays in the host's local store. A renderer takes a whole document or an update that appends events; the UI reads the views and takes their changes live. The UI is written by hand (`ui/app/`) over TypeScript types generated from this domain (`ess generate types`); a gateway UI reuses its embed pages and components.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// Agent — `agentspec.view.Agent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// `id` — `String`.
    pub id: String,
    /// `parent` — `Optional<String>`.
    pub parent: Option<String>,
    /// `role` — `String`.
    pub role: String,
    /// `spawned_by` — `Optional<String>`.
    pub spawned_by: Option<String>,
    /// `started_at` — `Integer`.
    pub started_at: i64,
}

/// The states of `agentspec.view.AgentListing`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `AgentListing<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentListingState {
    /// `Listed`.
    Listed,
}

/// BucketId — `agentspec.view.BucketId`: a distinct wrapper around `String`.
///
/// One time bucket of the overview, its start in milliseconds since the epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BucketId(pub String);

/// CheckId — `agentspec.view.CheckId`: a distinct wrapper around `String`.
///
/// One check of one session, `<session_id>.<check>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckId(pub String);

/// CountId — `agentspec.view.CountId`: a distinct wrapper around `String`.
///
/// The count of one event kind in one session, `<session_id>.<agent or *>.<kind>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountId(pub String);

/// Document — `agentspec.view.Document`.
///
/// One session for a renderer, format `agentspec-view/1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// `format` — `String`.
    pub format: String,
    /// `session_id` — `String`.
    pub session_id: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `project` — `String`.
    pub project: String,
    /// `harness` — `String`.
    pub harness: String,
    /// `agents` — `List<agentspec.view.Agent>`.
    pub agents: Vec<Agent>,
    /// `events` — `List<agentspec.view.Event>`.
    pub events: Vec<Event>,
}

/// Event — `agentspec.view.Event`.
///
/// One thing that happened, at an instant in milliseconds since the epoch. `label` names it: the tool, the prompt author, the model, the decider, the exposure kind, the bust cause. Numbers that do not apply are absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `at` — `Integer`.
    pub at: i64,
    /// `agent` — `String`.
    pub agent: String,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `label` — `String`.
    pub label: String,
    /// `detail` — `Optional<String>`.
    pub detail: Option<String>,
    /// `call` — `Optional<String>`.
    pub call: Option<String>,
    /// `ok` — `Optional<Boolean>`.
    pub ok: Option<bool>,
    /// `context_tokens` — `Optional<Integer>`.
    pub context_tokens: Option<i64>,
    /// `output_tokens` — `Optional<Integer>`.
    pub output_tokens: Option<i64>,
    /// `cache_read_tokens` — `Optional<Integer>`.
    pub cache_read_tokens: Option<i64>,
    /// `cache_write_tokens` — `Optional<Integer>`.
    pub cache_write_tokens: Option<i64>,
    /// `cost_microdollars` — `Optional<Integer>`.
    pub cost_microdollars: Option<i64>,
    /// `bytes` — `Optional<Integer>`.
    pub bytes: Option<i64>,
}

/// The states of `agentspec.view.EventCount`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `EventCount<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCountState {
    /// `Counted`.
    Counted,
}

/// EventId — `agentspec.view.EventId`: a distinct wrapper around `String`.
///
/// One view event of one session, `<session_id>.<seq>`; a dot, since a row key is one segment of an ess-ui node path, which `/` separates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventId(pub String);

/// EventKind — `agentspec.view.EventKind`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// `agent_started`.
    AgentStarted,
    /// `prompt`.
    Prompt,
    /// `model_turn`.
    ModelTurn,
    /// `tool_call`.
    ToolCall,
    /// `decision`.
    Decision,
    /// `tool_result`.
    ToolResult,
    /// `turn_end`.
    TurnEnd,
    /// `compaction`.
    Compaction,
    /// `exposure`.
    Exposure,
    /// `cache_bust`.
    CacheBust,
}

/// The states of `agentspec.view.EventRow`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `EventRow<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventRowState {
    /// `Logged`.
    Logged,
}

/// The states of `agentspec.view.FigureBucket`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `FigureBucket<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigureBucketState {
    /// `Counted`.
    Counted,
}

/// The states of `agentspec.view.SessionCheck`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `SessionCheck<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCheckState {
    /// `Judged`.
    Judged,
}

/// The states of `agentspec.view.SessionListing`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `SessionListing<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionListingState {
    /// `Listed`.
    Listed,
}

/// SlotId — `agentspec.view.SlotId`: a distinct wrapper around `String`.
///
/// One time slot of one agent of one session, `<session_id>.<agent>.<start in ms>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotId(pub String);

/// The states of `agentspec.view.TimelineSlot`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `TimelineSlot<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineSlotState {
    /// `Counted`.
    Counted,
}

/// Update — `agentspec.view.Update`.
///
/// What a live source sends after the first document: the agents so far and the events from `from_seq` on. With `reset` the renderer drops what it holds first, because the session's earlier events changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    /// `format` — `String`.
    pub format: String,
    /// `session_id` — `String`.
    pub session_id: String,
    /// `reset` — `Boolean`.
    pub reset: bool,
    /// `from_seq` — `Integer`.
    pub from_seq: i64,
    /// `agents` — `List<agentspec.view.Agent>`.
    pub agents: Vec<Agent>,
    /// `events` — `List<agentspec.view.Event>`.
    pub events: Vec<Event>,
}

/// What AgentListing — `agentspec.view.AgentListing` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`AgentListing<S>`], and at a boundary by [`AgentListingSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentListingData {
    /// The identity: `agent_id` — `agentspec.session.AgentId`.
    pub agent_id: crate::session::AgentId,
    /// `session_id` — `agentspec.session.SessionId`.
    ///
    /// Carries `agents_of`: `agentspec.view.SessionListing` owns many `agentspec.view.AgentListing`.
    pub session_id: crate::session::SessionId,
    /// `parent` — `Optional<agentspec.session.AgentId>`.
    ///
    /// Carries `parent_agent`: `agentspec.view.AgentListing` references one `agentspec.view.AgentListing`.
    pub parent: Option<crate::session::AgentId>,
    /// `role` — `agentspec.session.AgentRole`.
    pub role: crate::session::AgentRole,
    /// `model` — `Optional<String>`.
    pub model: Option<String>,
    /// `started_at` — `Timestamp`.
    pub started_at: crate::primitives::Timestamp,
    /// `turns` — `Integer`.
    pub turns: i64,
    /// `context_tokens` — `Integer`.
    pub context_tokens: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `busts` — `Integer`.
    pub busts: i64,
}

/// The states of `agentspec.view.AgentListing`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](agent_listing_state::Marker), so [`AgentListing<S>`](AgentListing) can only ever rest in a real state.
pub mod agent_listing_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Listed {}
    }

    /// A declared state of `AgentListing`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::AgentListingState;
    }

    /// `Listed`. Where a new instance starts.
    pub struct Listed;

    impl Marker for Listed {
        const STATE: super::AgentListingState = super::AgentListingState::Listed;
    }
}

/// AgentListing — `agentspec.view.AgentListing` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Listed`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`AgentListingSnapshot`]
/// and [`AgentListingSnapshot::refine`].
pub struct AgentListing<S: agent_listing_state::Marker> {
    data: AgentListingData,
    state: core::marker::PhantomData<S>,
}

impl<S: agent_listing_state::Marker> AgentListing<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> AgentListingState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &AgentListingData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> AgentListingData {
        self.data
    }
}

impl AgentListing<agent_listing_state::Listed> {
    /// A new instance, resting in `Listed` — the only state the lifecycle starts one in.
    pub fn new(data: AgentListingData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `agentspec.view.AgentListing` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`AgentListingSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentListingSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: AgentListingState,
    /// What it holds.
    pub data: AgentListingData,
}

/// An `AgentListing` in whichever declared state it was found.
pub enum AnyAgentListing {
    /// Resting in `Listed`.
    Listed(AgentListing<agent_listing_state::Listed>),
}

impl AgentListingSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `AgentListingState` cannot spell one.
    pub fn refine(self) -> AnyAgentListing {
        match self.state {
            AgentListingState::Listed => AnyAgentListing::Listed(AgentListing {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyAgentListing {
    /// The state, as the runtime value.
    pub fn state(&self) -> AgentListingState {
        match self {
            Self::Listed(_) => AgentListingState::Listed,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> AgentListingSnapshot {
        match self {
            Self::Listed(instance) => AgentListingSnapshot {
                state: AgentListingState::Listed,
                data: instance.into_data(),
            },
        }
    }
}

/// What EventCount — `agentspec.view.EventCount` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`EventCount<S>`], and at a boundary by [`EventCountSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCountData {
    /// The identity: `count_id` — `agentspec.view.CountId`.
    pub count_id: CountId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `agent` — `Optional<agentspec.session.AgentId>`.
    pub agent: Option<crate::session::AgentId>,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `count` — `Integer`.
    pub count: i64,
}

/// The states of `agentspec.view.EventCount`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](event_count_state::Marker), so [`EventCount<S>`](EventCount) can only ever rest in a real state.
pub mod event_count_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Counted {}
    }

    /// A declared state of `EventCount`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::EventCountState;
    }

    /// `Counted`. Where a new instance starts.
    pub struct Counted;

    impl Marker for Counted {
        const STATE: super::EventCountState = super::EventCountState::Counted;
    }
}

/// EventCount — `agentspec.view.EventCount` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Counted`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`EventCountSnapshot`]
/// and [`EventCountSnapshot::refine`].
pub struct EventCount<S: event_count_state::Marker> {
    data: EventCountData,
    state: core::marker::PhantomData<S>,
}

impl<S: event_count_state::Marker> EventCount<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> EventCountState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &EventCountData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> EventCountData {
        self.data
    }
}

impl EventCount<event_count_state::Counted> {
    /// A new instance, resting in `Counted` — the only state the lifecycle starts one in.
    pub fn new(data: EventCountData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `agentspec.view.EventCount` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`EventCountSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCountSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: EventCountState,
    /// What it holds.
    pub data: EventCountData,
}

/// An `EventCount` in whichever declared state it was found.
pub enum AnyEventCount {
    /// Resting in `Counted`.
    Counted(EventCount<event_count_state::Counted>),
}

impl EventCountSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `EventCountState` cannot spell one.
    pub fn refine(self) -> AnyEventCount {
        match self.state {
            EventCountState::Counted => AnyEventCount::Counted(EventCount {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyEventCount {
    /// The state, as the runtime value.
    pub fn state(&self) -> EventCountState {
        match self {
            Self::Counted(_) => EventCountState::Counted,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> EventCountSnapshot {
        match self {
            Self::Counted(instance) => EventCountSnapshot {
                state: EventCountState::Counted,
                data: instance.into_data(),
            },
        }
    }
}

/// What EventRow — `agentspec.view.EventRow` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`EventRow<S>`], and at a boundary by [`EventRowSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRowData {
    /// The identity: `event_id` — `agentspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `agentspec.session.SessionId`.
    ///
    /// Carries `events_of`: `agentspec.view.SessionListing` owns many `agentspec.view.EventRow`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `ord` — `Integer`.
    pub ord: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `agentspec.session.AgentId`.
    ///
    /// Carries `of_agent`: `agentspec.view.EventRow` references one `agentspec.view.AgentListing`.
    pub agent: crate::session::AgentId,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `notable` — `Boolean`.
    pub notable: bool,
    /// `label` — `String`.
    pub label: String,
    /// `detail` — `Optional<String>`.
    pub detail: Option<String>,
    /// `call` — `Optional<String>`.
    pub call: Option<String>,
    /// `ok` — `Optional<Boolean>`.
    pub ok: Option<bool>,
    /// `context_tokens` — `Optional<Integer>`.
    pub context_tokens: Option<i64>,
    /// `output_tokens` — `Optional<Integer>`.
    pub output_tokens: Option<i64>,
    /// `cache_read_tokens` — `Optional<Integer>`.
    pub cache_read_tokens: Option<i64>,
    /// `cache_write_tokens` — `Optional<Integer>`.
    pub cache_write_tokens: Option<i64>,
    /// `cost_microdollars` — `Optional<Integer>`.
    pub cost_microdollars: Option<i64>,
    /// `bytes` — `Optional<Integer>`.
    pub bytes: Option<i64>,
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// The states of `agentspec.view.EventRow`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](event_row_state::Marker), so [`EventRow<S>`](EventRow) can only ever rest in a real state.
pub mod event_row_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Logged {}
    }

    /// A declared state of `EventRow`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::EventRowState;
    }

    /// `Logged`. Where a new instance starts.
    pub struct Logged;

    impl Marker for Logged {
        const STATE: super::EventRowState = super::EventRowState::Logged;
    }
}

/// EventRow — `agentspec.view.EventRow` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Logged`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`EventRowSnapshot`]
/// and [`EventRowSnapshot::refine`].
pub struct EventRow<S: event_row_state::Marker> {
    data: EventRowData,
    state: core::marker::PhantomData<S>,
}

impl<S: event_row_state::Marker> EventRow<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> EventRowState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &EventRowData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> EventRowData {
        self.data
    }
}

impl EventRow<event_row_state::Logged> {
    /// A new instance, resting in `Logged` — the only state the lifecycle starts one in.
    pub fn new(data: EventRowData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `agentspec.view.EventRow` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`EventRowSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRowSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: EventRowState,
    /// What it holds.
    pub data: EventRowData,
}

/// An `EventRow` in whichever declared state it was found.
pub enum AnyEventRow {
    /// Resting in `Logged`.
    Logged(EventRow<event_row_state::Logged>),
}

impl EventRowSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `EventRowState` cannot spell one.
    pub fn refine(self) -> AnyEventRow {
        match self.state {
            EventRowState::Logged => AnyEventRow::Logged(EventRow {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyEventRow {
    /// The state, as the runtime value.
    pub fn state(&self) -> EventRowState {
        match self {
            Self::Logged(_) => EventRowState::Logged,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> EventRowSnapshot {
        match self {
            Self::Logged(instance) => EventRowSnapshot {
                state: EventRowState::Logged,
                data: instance.into_data(),
            },
        }
    }
}

/// What FigureBucket — `agentspec.view.FigureBucket` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`FigureBucket<S>`], and at a boundary by [`FigureBucketSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureBucketData {
    /// The identity: `bucket_id` — `agentspec.view.BucketId`.
    pub bucket_id: BucketId,
    /// `start` — `Timestamp`.
    pub start: crate::primitives::Timestamp,
    /// `bucket_seconds` — `Integer`.
    pub bucket_seconds: i64,
    /// `sessions` — `Integer`.
    pub sessions: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `working_seconds` — `Integer`.
    pub working_seconds: i64,
    /// `waiting_share` — `Binary64`.
    pub waiting_share: crate::primitives::Binary64,
    /// `cache_hit_share` — `Binary64`.
    pub cache_hit_share: crate::primitives::Binary64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `refused_tool_calls` — `Integer`.
    pub refused_tool_calls: i64,
}

/// The states of `agentspec.view.FigureBucket`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](figure_bucket_state::Marker), so [`FigureBucket<S>`](FigureBucket) can only ever rest in a real state.
pub mod figure_bucket_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Counted {}
    }

    /// A declared state of `FigureBucket`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::FigureBucketState;
    }

    /// `Counted`. Where a new instance starts.
    pub struct Counted;

    impl Marker for Counted {
        const STATE: super::FigureBucketState = super::FigureBucketState::Counted;
    }
}

/// FigureBucket — `agentspec.view.FigureBucket` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Counted`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`FigureBucketSnapshot`]
/// and [`FigureBucketSnapshot::refine`].
pub struct FigureBucket<S: figure_bucket_state::Marker> {
    data: FigureBucketData,
    state: core::marker::PhantomData<S>,
}

impl<S: figure_bucket_state::Marker> FigureBucket<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> FigureBucketState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &FigureBucketData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> FigureBucketData {
        self.data
    }
}

impl FigureBucket<figure_bucket_state::Counted> {
    /// A new instance, resting in `Counted` — the only state the lifecycle starts one in.
    pub fn new(data: FigureBucketData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `agentspec.view.FigureBucket` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`FigureBucketSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureBucketSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: FigureBucketState,
    /// What it holds.
    pub data: FigureBucketData,
}

/// An `FigureBucket` in whichever declared state it was found.
pub enum AnyFigureBucket {
    /// Resting in `Counted`.
    Counted(FigureBucket<figure_bucket_state::Counted>),
}

impl FigureBucketSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `FigureBucketState` cannot spell one.
    pub fn refine(self) -> AnyFigureBucket {
        match self.state {
            FigureBucketState::Counted => AnyFigureBucket::Counted(FigureBucket {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyFigureBucket {
    /// The state, as the runtime value.
    pub fn state(&self) -> FigureBucketState {
        match self {
            Self::Counted(_) => FigureBucketState::Counted,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> FigureBucketSnapshot {
        match self {
            Self::Counted(instance) => FigureBucketSnapshot {
                state: FigureBucketState::Counted,
                data: instance.into_data(),
            },
        }
    }
}

/// What SessionCheck — `agentspec.view.SessionCheck` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`SessionCheck<S>`], and at a boundary by [`SessionCheckSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCheckData {
    /// The identity: `check_id` — `agentspec.view.CheckId`.
    pub check_id: CheckId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `check` — `String`.
    pub check: String,
    /// `about` — `String`.
    pub about: String,
    /// `severity` — `String`.
    pub severity: String,
    /// `verdict` — `agentspec.session.Verdict`.
    pub verdict: crate::session::Verdict,
    /// `rows` — `Integer`.
    pub rows: i64,
    /// `possible` — `Integer`.
    pub possible: i64,
}

/// The states of `agentspec.view.SessionCheck`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](session_check_state::Marker), so [`SessionCheck<S>`](SessionCheck) can only ever rest in a real state.
pub mod session_check_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Judged {}
    }

    /// A declared state of `SessionCheck`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::SessionCheckState;
    }

    /// `Judged`. Where a new instance starts.
    pub struct Judged;

    impl Marker for Judged {
        const STATE: super::SessionCheckState = super::SessionCheckState::Judged;
    }
}

/// SessionCheck — `agentspec.view.SessionCheck` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Judged`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`SessionCheckSnapshot`]
/// and [`SessionCheckSnapshot::refine`].
pub struct SessionCheck<S: session_check_state::Marker> {
    data: SessionCheckData,
    state: core::marker::PhantomData<S>,
}

impl<S: session_check_state::Marker> SessionCheck<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> SessionCheckState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &SessionCheckData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> SessionCheckData {
        self.data
    }
}

impl SessionCheck<session_check_state::Judged> {
    /// A new instance, resting in `Judged` — the only state the lifecycle starts one in.
    pub fn new(data: SessionCheckData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `agentspec.view.SessionCheck` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`SessionCheckSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCheckSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: SessionCheckState,
    /// What it holds.
    pub data: SessionCheckData,
}

/// An `SessionCheck` in whichever declared state it was found.
pub enum AnySessionCheck {
    /// Resting in `Judged`.
    Judged(SessionCheck<session_check_state::Judged>),
}

impl SessionCheckSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `SessionCheckState` cannot spell one.
    pub fn refine(self) -> AnySessionCheck {
        match self.state {
            SessionCheckState::Judged => AnySessionCheck::Judged(SessionCheck {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnySessionCheck {
    /// The state, as the runtime value.
    pub fn state(&self) -> SessionCheckState {
        match self {
            Self::Judged(_) => SessionCheckState::Judged,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> SessionCheckSnapshot {
        match self {
            Self::Judged(instance) => SessionCheckSnapshot {
                state: SessionCheckState::Judged,
                data: instance.into_data(),
            },
        }
    }
}

/// What SessionListing — `agentspec.view.SessionListing` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`SessionListing<S>`], and at a boundary by [`SessionListingSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListingData {
    /// The identity: `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `names` — `String`.
    pub names: String,
    /// `rank` — `Integer`.
    pub rank: i64,
    /// `harness` — `agentspec.session.Harness`.
    pub harness: crate::session::Harness,
    /// `first_at` — `Timestamp`.
    pub first_at: crate::primitives::Timestamp,
    /// `last_at` — `Timestamp`.
    pub last_at: crate::primitives::Timestamp,
    /// `agents` — `Integer`.
    pub agents: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `cache_hit_share` — `Binary64`.
    pub cache_hit_share: crate::primitives::Binary64,
    /// `busts` — `Integer`.
    pub busts: i64,
    /// `human_prompts` — `Integer`.
    pub human_prompts: i64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `refused_tool_calls` — `Integer`.
    pub refused_tool_calls: i64,
    /// `exposures` — `Integer`.
    pub exposures: i64,
    /// `working_seconds` — `Integer`.
    pub working_seconds: i64,
    /// `waiting_on_human_seconds` — `Integer`.
    pub waiting_on_human_seconds: i64,
    /// `waiting_share` — `Binary64`.
    pub waiting_share: crate::primitives::Binary64,
    /// `kpis` — `agentspec.cli.KpiReport`.
    pub kpis: crate::cli::KpiReport,
}

/// The states of `agentspec.view.SessionListing`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](session_listing_state::Marker), so [`SessionListing<S>`](SessionListing) can only ever rest in a real state.
pub mod session_listing_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Listed {}
    }

    /// A declared state of `SessionListing`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::SessionListingState;
    }

    /// `Listed`. Where a new instance starts.
    pub struct Listed;

    impl Marker for Listed {
        const STATE: super::SessionListingState = super::SessionListingState::Listed;
    }
}

/// SessionListing — `agentspec.view.SessionListing` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Listed`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`SessionListingSnapshot`]
/// and [`SessionListingSnapshot::refine`].
pub struct SessionListing<S: session_listing_state::Marker> {
    data: SessionListingData,
    state: core::marker::PhantomData<S>,
}

impl<S: session_listing_state::Marker> SessionListing<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> SessionListingState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &SessionListingData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> SessionListingData {
        self.data
    }
}

impl SessionListing<session_listing_state::Listed> {
    /// A new instance, resting in `Listed` — the only state the lifecycle starts one in.
    pub fn new(data: SessionListingData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `agentspec.view.SessionListing` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`SessionListingSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListingSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: SessionListingState,
    /// What it holds.
    pub data: SessionListingData,
}

/// An `SessionListing` in whichever declared state it was found.
pub enum AnySessionListing {
    /// Resting in `Listed`.
    Listed(SessionListing<session_listing_state::Listed>),
}

impl SessionListingSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `SessionListingState` cannot spell one.
    pub fn refine(self) -> AnySessionListing {
        match self.state {
            SessionListingState::Listed => AnySessionListing::Listed(SessionListing {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnySessionListing {
    /// The state, as the runtime value.
    pub fn state(&self) -> SessionListingState {
        match self {
            Self::Listed(_) => SessionListingState::Listed,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> SessionListingSnapshot {
        match self {
            Self::Listed(instance) => SessionListingSnapshot {
                state: SessionListingState::Listed,
                data: instance.into_data(),
            },
        }
    }
}

/// What TimelineSlot — `agentspec.view.TimelineSlot` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`TimelineSlot<S>`], and at a boundary by [`TimelineSlotSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineSlotData {
    /// The identity: `slot_id` — `agentspec.view.SlotId`.
    pub slot_id: SlotId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `agent` — `agentspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `slot_seconds` — `Integer`.
    pub slot_seconds: i64,
    /// `start` — `Timestamp`.
    pub start: crate::primitives::Timestamp,
    /// `end` — `Timestamp`.
    pub end: crate::primitives::Timestamp,
    /// `events` — `Integer`.
    pub events: i64,
    /// `read_calls` — `Integer`.
    pub read_calls: i64,
    /// `search_calls` — `Integer`.
    pub search_calls: i64,
    /// `edit_calls` — `Integer`.
    pub edit_calls: i64,
    /// `execute_calls` — `Integer`.
    pub execute_calls: i64,
    /// `network_calls` — `Integer`.
    pub network_calls: i64,
    /// `spawn_calls` — `Integer`.
    pub spawn_calls: i64,
    /// `message_calls` — `Integer`.
    pub message_calls: i64,
    /// `plan_calls` — `Integer`.
    pub plan_calls: i64,
    /// `other_calls` — `Integer`.
    pub other_calls: i64,
    /// `human_prompts` — `Integer`.
    pub human_prompts: i64,
    /// `relayed_prompts` — `Integer`.
    pub relayed_prompts: i64,
    /// `refusals` — `Integer`.
    pub refusals: i64,
    /// `busts` — `Integer`.
    pub busts: i64,
    /// `compactions` — `Integer`.
    pub compactions: i64,
    /// `context_max` — `Optional<Integer>`.
    pub context_max: Option<i64>,
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// The states of `agentspec.view.TimelineSlot`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](timeline_slot_state::Marker), so [`TimelineSlot<S>`](TimelineSlot) can only ever rest in a real state.
pub mod timeline_slot_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Counted {}
    }

    /// A declared state of `TimelineSlot`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::TimelineSlotState;
    }

    /// `Counted`. Where a new instance starts.
    pub struct Counted;

    impl Marker for Counted {
        const STATE: super::TimelineSlotState = super::TimelineSlotState::Counted;
    }
}

/// TimelineSlot — `agentspec.view.TimelineSlot` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Counted`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`TimelineSlotSnapshot`]
/// and [`TimelineSlotSnapshot::refine`].
pub struct TimelineSlot<S: timeline_slot_state::Marker> {
    data: TimelineSlotData,
    state: core::marker::PhantomData<S>,
}

impl<S: timeline_slot_state::Marker> TimelineSlot<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> TimelineSlotState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &TimelineSlotData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> TimelineSlotData {
        self.data
    }
}

impl TimelineSlot<timeline_slot_state::Counted> {
    /// A new instance, resting in `Counted` — the only state the lifecycle starts one in.
    pub fn new(data: TimelineSlotData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `agentspec.view.TimelineSlot` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`TimelineSlotSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineSlotSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: TimelineSlotState,
    /// What it holds.
    pub data: TimelineSlotData,
}

/// An `TimelineSlot` in whichever declared state it was found.
pub enum AnyTimelineSlot {
    /// Resting in `Counted`.
    Counted(TimelineSlot<timeline_slot_state::Counted>),
}

impl TimelineSlotSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `TimelineSlotState` cannot spell one.
    pub fn refine(self) -> AnyTimelineSlot {
        match self.state {
            TimelineSlotState::Counted => AnyTimelineSlot::Counted(TimelineSlot {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyTimelineSlot {
    /// The state, as the runtime value.
    pub fn state(&self) -> TimelineSlotState {
        match self {
            Self::Counted(_) => TimelineSlotState::Counted,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> TimelineSlotSnapshot {
        match self {
            Self::Counted(instance) => TimelineSlotSnapshot {
                state: TimelineSlotState::Counted,
                data: instance.into_data(),
            },
        }
    }
}

/// AgentUpdated — the event `agentspec.view.AgentUpdated`.
///
/// An agent started or its figures changed; the payload is its whole row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUpdated {
    /// `agent_id` — `agentspec.session.AgentId`.
    pub agent_id: crate::session::AgentId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `parent` — `Optional<agentspec.session.AgentId>`.
    pub parent: Option<crate::session::AgentId>,
    /// `role` — `agentspec.session.AgentRole`.
    pub role: crate::session::AgentRole,
    /// `model` — `Optional<String>`.
    pub model: Option<String>,
    /// `started_at` — `Timestamp`.
    pub started_at: crate::primitives::Timestamp,
    /// `turns` — `Integer`.
    pub turns: i64,
    /// `context_tokens` — `Integer`.
    pub context_tokens: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `busts` — `Integer`.
    pub busts: i64,
}

/// EventAppended — the event `agentspec.view.EventAppended`.
///
/// One event of a running session; the payload is its row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventAppended {
    /// `event_id` — `agentspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `ord` — `Integer`.
    pub ord: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `agentspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `notable` — `Boolean`.
    pub notable: bool,
    /// `label` — `String`.
    pub label: String,
    /// `detail` — `Optional<String>`.
    pub detail: Option<String>,
    /// `call` — `Optional<String>`.
    pub call: Option<String>,
    /// `ok` — `Optional<Boolean>`.
    pub ok: Option<bool>,
    /// `context_tokens` — `Optional<Integer>`.
    pub context_tokens: Option<i64>,
    /// `output_tokens` — `Optional<Integer>`.
    pub output_tokens: Option<i64>,
    /// `cache_read_tokens` — `Optional<Integer>`.
    pub cache_read_tokens: Option<i64>,
    /// `cache_write_tokens` — `Optional<Integer>`.
    pub cache_write_tokens: Option<i64>,
    /// `cost_microdollars` — `Optional<Integer>`.
    pub cost_microdollars: Option<i64>,
    /// `bytes` — `Optional<Integer>`.
    pub bytes: Option<i64>,
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// SessionUpdated — the event `agentspec.view.SessionUpdated`.
///
/// The figures of a session changed; the payload is its whole row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionUpdated {
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `names` — `String`.
    pub names: String,
    /// `rank` — `Integer`.
    pub rank: i64,
    /// `harness` — `agentspec.session.Harness`.
    pub harness: crate::session::Harness,
    /// `first_at` — `Timestamp`.
    pub first_at: crate::primitives::Timestamp,
    /// `last_at` — `Timestamp`.
    pub last_at: crate::primitives::Timestamp,
    /// `agents` — `Integer`.
    pub agents: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `cache_hit_share` — `Binary64`.
    pub cache_hit_share: crate::primitives::Binary64,
    /// `busts` — `Integer`.
    pub busts: i64,
    /// `human_prompts` — `Integer`.
    pub human_prompts: i64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `refused_tool_calls` — `Integer`.
    pub refused_tool_calls: i64,
    /// `exposures` — `Integer`.
    pub exposures: i64,
    /// `working_seconds` — `Integer`.
    pub working_seconds: i64,
    /// `waiting_on_human_seconds` — `Integer`.
    pub waiting_on_human_seconds: i64,
    /// `waiting_share` — `Binary64`.
    pub waiting_share: crate::primitives::Binary64,
}

/// Model turns of an agent — one row of the view `agentspec.view.AgentTurns`.
///
/// Projects `agentspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and agent == param.agent and kind == model_turn)`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurns {
    /// `event_id` — `agentspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `ord` — `Integer`.
    pub ord: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `agentspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `notable` — `Boolean`.
    pub notable: bool,
    /// `label` — `String`.
    pub label: String,
    /// `detail` — `Optional<String>`.
    pub detail: Option<String>,
    /// `call` — `Optional<String>`.
    pub call: Option<String>,
    /// `ok` — `Optional<Boolean>`.
    pub ok: Option<bool>,
    /// `context_tokens` — `Optional<Integer>`.
    pub context_tokens: Option<i64>,
    /// `output_tokens` — `Optional<Integer>`.
    pub output_tokens: Option<i64>,
    /// `cache_read_tokens` — `Optional<Integer>`.
    pub cache_read_tokens: Option<i64>,
    /// `cache_write_tokens` — `Optional<Integer>`.
    pub cache_write_tokens: Option<i64>,
    /// `cost_microdollars` — `Optional<Integer>`.
    pub cost_microdollars: Option<i64>,
    /// `bytes` — `Optional<Integer>`.
    pub bytes: Option<i64>,
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// Overview figures — one row of the view `agentspec.view.OverviewFigures`.
///
/// Projects `agentspec.view.FigureBucket` at `eventual` consistency, containing instances where `(start >= param.since as timestamp and bucket_seconds == param.bucket_seconds)`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewFigures {
    /// `bucket_id` — `agentspec.view.BucketId`.
    pub bucket_id: BucketId,
    /// `start` — `Timestamp`.
    pub start: crate::primitives::Timestamp,
    /// `bucket_seconds` — `Integer`.
    pub bucket_seconds: i64,
    /// `sessions` — `Integer`.
    pub sessions: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `working_seconds` — `Integer`.
    pub working_seconds: i64,
    /// `waiting_share` — `Binary64`.
    pub waiting_share: crate::primitives::Binary64,
    /// `cache_hit_share` — `Binary64`.
    pub cache_hit_share: crate::primitives::Binary64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `refused_tool_calls` — `Integer`.
    pub refused_tool_calls: i64,
}

/// Agents of a session — one row of the view `agentspec.view.SessionAgents`.
///
/// Projects `agentspec.view.AgentListing` at `eventual` consistency, containing instances where `session_id == param.session_id`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAgents {
    /// `agent_id` — `agentspec.session.AgentId`.
    pub agent_id: crate::session::AgentId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `parent` — `Optional<agentspec.session.AgentId>`.
    pub parent: Option<crate::session::AgentId>,
    /// `role` — `agentspec.session.AgentRole`.
    pub role: crate::session::AgentRole,
    /// `model` — `Optional<String>`.
    pub model: Option<String>,
    /// `started_at` — `Timestamp`.
    pub started_at: crate::primitives::Timestamp,
    /// `turns` — `Integer`.
    pub turns: i64,
    /// `context_tokens` — `Integer`.
    pub context_tokens: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `busts` — `Integer`.
    pub busts: i64,
}

/// Session — one row of the view `agentspec.view.SessionById`.
///
/// Projects `agentspec.view.SessionListing` at `eventual` consistency, containing instances where `session_id == param.session_id`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionById {
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `harness` — `agentspec.session.Harness`.
    pub harness: crate::session::Harness,
    /// `first_at` — `Timestamp`.
    pub first_at: crate::primitives::Timestamp,
    /// `last_at` — `Timestamp`.
    pub last_at: crate::primitives::Timestamp,
    /// `agents` — `Integer`.
    pub agents: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `cache_hit_share` — `Binary64`.
    pub cache_hit_share: crate::primitives::Binary64,
    /// `busts` — `Integer`.
    pub busts: i64,
    /// `human_prompts` — `Integer`.
    pub human_prompts: i64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `refused_tool_calls` — `Integer`.
    pub refused_tool_calls: i64,
    /// `exposures` — `Integer`.
    pub exposures: i64,
    /// `working_seconds` — `Integer`.
    pub working_seconds: i64,
    /// `waiting_on_human_seconds` — `Integer`.
    pub waiting_on_human_seconds: i64,
    /// `waiting_share` — `Binary64`.
    pub waiting_share: crate::primitives::Binary64,
    /// `kpis` — `agentspec.cli.KpiReport`.
    pub kpis: crate::cli::KpiReport,
}

/// Checks of a session — one row of the view `agentspec.view.SessionChecks`.
///
/// Projects `agentspec.view.SessionCheck` at `eventual` consistency, containing instances where `session_id == param.session_id`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionChecks {
    /// `check_id` — `agentspec.view.CheckId`.
    pub check_id: CheckId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `check` — `String`.
    pub check: String,
    /// `about` — `String`.
    pub about: String,
    /// `severity` — `String`.
    pub severity: String,
    /// `verdict` — `agentspec.session.Verdict`.
    pub verdict: crate::session::Verdict,
    /// `rows` — `Integer`.
    pub rows: i64,
    /// `possible` — `Integer`.
    pub possible: i64,
}

/// Event counts of a session — one row of the view `agentspec.view.SessionEventCounts`.
///
/// Projects `agentspec.view.EventCount` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent))`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEventCounts {
    /// `count_id` — `agentspec.view.CountId`.
    pub count_id: CountId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `agent` — `Optional<agentspec.session.AgentId>`.
    pub agent: Option<crate::session::AgentId>,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `count` — `Integer`.
    pub count: i64,
}

/// Events of a session — one row of the view `agentspec.view.SessionEvents`.
///
/// Projects `agentspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent) and (not (defined(param.kind)) or kind == param.kind) and (not (defined(param.notable)) or notable == param.notable) and (not (defined(param.before)) or ord < param.before))`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEvents {
    /// `event_id` — `agentspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `ord` — `Integer`.
    pub ord: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `agentspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `notable` — `Boolean`.
    pub notable: bool,
    /// `label` — `String`.
    pub label: String,
    /// `detail` — `Optional<String>`.
    pub detail: Option<String>,
    /// `call` — `Optional<String>`.
    pub call: Option<String>,
    /// `ok` — `Optional<Boolean>`.
    pub ok: Option<bool>,
    /// `context_tokens` — `Optional<Integer>`.
    pub context_tokens: Option<i64>,
    /// `output_tokens` — `Optional<Integer>`.
    pub output_tokens: Option<i64>,
    /// `cache_read_tokens` — `Optional<Integer>`.
    pub cache_read_tokens: Option<i64>,
    /// `cache_write_tokens` — `Optional<Integer>`.
    pub cache_write_tokens: Option<i64>,
    /// `cost_microdollars` — `Optional<Integer>`.
    pub cost_microdollars: Option<i64>,
    /// `bytes` — `Optional<Integer>`.
    pub bytes: Option<i64>,
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// Sessions — one row of the view `agentspec.view.SessionList`.
///
/// Projects `agentspec.view.SessionListing` at `eventual` consistency, containing instances where `((not (defined(param.since)) or last_at >= param.since as timestamp) and (not (defined(param.search)) or names contains {param: search}) and (not (defined(param.limit)) or rank <= param.limit) and (not (defined(param.offset)) or rank > param.offset))`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionList {
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `names` — `String`.
    pub names: String,
    /// `rank` — `Integer`.
    pub rank: i64,
    /// `harness` — `agentspec.session.Harness`.
    pub harness: crate::session::Harness,
    /// `first_at` — `Timestamp`.
    pub first_at: crate::primitives::Timestamp,
    /// `last_at` — `Timestamp`.
    pub last_at: crate::primitives::Timestamp,
    /// `agents` — `Integer`.
    pub agents: i64,
    /// `cost_usd` — `Binary64`.
    pub cost_usd: crate::primitives::Binary64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `cache_hit_share` — `Binary64`.
    pub cache_hit_share: crate::primitives::Binary64,
    /// `busts` — `Integer`.
    pub busts: i64,
    /// `human_prompts` — `Integer`.
    pub human_prompts: i64,
    /// `tool_calls` — `Integer`.
    pub tool_calls: i64,
    /// `refused_tool_calls` — `Integer`.
    pub refused_tool_calls: i64,
    /// `exposures` — `Integer`.
    pub exposures: i64,
    /// `working_seconds` — `Integer`.
    pub working_seconds: i64,
    /// `waiting_on_human_seconds` — `Integer`.
    pub waiting_on_human_seconds: i64,
    /// `waiting_share` — `Binary64`.
    pub waiting_share: crate::primitives::Binary64,
}

/// Timeline of a session — one row of the view `agentspec.view.SessionTimeline`.
///
/// Projects `agentspec.view.TimelineSlot` at `eventual` consistency, containing instances where `(session_id == param.session_id and slot_seconds == param.slot_seconds)`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTimeline {
    /// `slot_id` — `agentspec.view.SlotId`.
    pub slot_id: SlotId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `agent` — `agentspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `slot_seconds` — `Integer`.
    pub slot_seconds: i64,
    /// `start` — `Timestamp`.
    pub start: crate::primitives::Timestamp,
    /// `end` — `Timestamp`.
    pub end: crate::primitives::Timestamp,
    /// `events` — `Integer`.
    pub events: i64,
    /// `read_calls` — `Integer`.
    pub read_calls: i64,
    /// `search_calls` — `Integer`.
    pub search_calls: i64,
    /// `edit_calls` — `Integer`.
    pub edit_calls: i64,
    /// `execute_calls` — `Integer`.
    pub execute_calls: i64,
    /// `network_calls` — `Integer`.
    pub network_calls: i64,
    /// `spawn_calls` — `Integer`.
    pub spawn_calls: i64,
    /// `message_calls` — `Integer`.
    pub message_calls: i64,
    /// `plan_calls` — `Integer`.
    pub plan_calls: i64,
    /// `other_calls` — `Integer`.
    pub other_calls: i64,
    /// `human_prompts` — `Integer`.
    pub human_prompts: i64,
    /// `relayed_prompts` — `Integer`.
    pub relayed_prompts: i64,
    /// `refusals` — `Integer`.
    pub refusals: i64,
    /// `busts` — `Integer`.
    pub busts: i64,
    /// `compactions` — `Integer`.
    pub compactions: i64,
    /// `context_max` — `Optional<Integer>`.
    pub context_max: Option<i64>,
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// Model turns of a session — one row of the view `agentspec.view.SessionTurns`.
///
/// Projects `agentspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and kind == model_turn)`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTurns {
    /// `event_id` — `agentspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `agentspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `ord` — `Integer`.
    pub ord: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `agentspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `agentspec.view.EventKind`.
    pub kind: EventKind,
    /// `notable` — `Boolean`.
    pub notable: bool,
    /// `label` — `String`.
    pub label: String,
    /// `detail` — `Optional<String>`.
    pub detail: Option<String>,
    /// `call` — `Optional<String>`.
    pub call: Option<String>,
    /// `ok` — `Optional<Boolean>`.
    pub ok: Option<bool>,
    /// `context_tokens` — `Optional<Integer>`.
    pub context_tokens: Option<i64>,
    /// `output_tokens` — `Optional<Integer>`.
    pub output_tokens: Option<i64>,
    /// `cache_read_tokens` — `Optional<Integer>`.
    pub cache_read_tokens: Option<i64>,
    /// `cache_write_tokens` — `Optional<Integer>`.
    pub cache_write_tokens: Option<i64>,
    /// `cost_microdollars` — `Optional<Integer>`.
    pub cost_microdollars: Option<i64>,
    /// `bytes` — `Optional<Integer>`.
    pub bytes: Option<i64>,
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The query `agentspec.view.AgentTurns` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait AgentTurnsQuery {
        /// Serves `agentspec.view.AgentTurns` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn agent_turns(&self, session_id: crate::session::SessionId, agent: crate::session::AgentId) -> Result<Vec<super::AgentTurns>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.OverviewFigures` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait OverviewFiguresQuery {
        /// Serves `agentspec.view.OverviewFigures` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn overview_figures(&self, since: crate::primitives::Timestamp, bucket_seconds: i64) -> Result<Vec<super::OverviewFigures>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionAgents` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionAgentsQuery {
        /// Serves `agentspec.view.SessionAgents` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_agents(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionAgents>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionById` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionByIdQuery {
        /// Serves `agentspec.view.SessionById` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_by_id(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionById>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionChecks` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionChecksQuery {
        /// Serves `agentspec.view.SessionChecks` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_checks(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionChecks>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionEventCounts` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a comparison with the optional view parameter `agent`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:`.
    ///
    /// Contract: a query answering `agentspec.view.SessionEventCounts` with rows projected from `agentspec.view.EventCount` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent))`.
    pub trait SessionEventCountsQuery {
        /// Serves `agentspec.view.SessionEventCounts` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn session_event_counts(&self, session_id: crate::session::SessionId, agent: Option<crate::session::AgentId>) -> Result<Vec<super::SessionEventCounts>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionEvents` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a comparison with the optional view parameter `agent`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:`.
    ///
    /// Contract: a query answering `agentspec.view.SessionEvents` with rows projected from `agentspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent) and (not (defined(param.kind)) or kind == param.kind) and (not (defined(param.notable)) or notable == param.notable) and (not (defined(param.before)) or ord < param.before))`.
    pub trait SessionEventsQuery {
        /// Serves `agentspec.view.SessionEvents` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn session_events(&self, session_id: crate::session::SessionId, agent: Option<crate::session::AgentId>, kind: Option<crate::view::EventKind>, notable: Option<bool>, before: Option<i64>) -> Result<Vec<super::SessionEvents>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionList` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a comparison with the optional view parameter `since`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:`.
    ///
    /// Contract: a query answering `agentspec.view.SessionList` with rows projected from `agentspec.view.SessionListing` at `eventual` consistency, containing instances where `((not (defined(param.since)) or last_at >= param.since as timestamp) and (not (defined(param.search)) or names contains {param: search}) and (not (defined(param.limit)) or rank <= param.limit) and (not (defined(param.offset)) or rank > param.offset))`.
    pub trait SessionListQuery {
        /// Serves `agentspec.view.SessionList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn session_list(&self, since: Option<crate::primitives::Timestamp>, search: Option<String>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<super::SessionList>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionTimeline` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionTimelineQuery {
        /// Serves `agentspec.view.SessionTimeline` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_timeline(&self, session_id: crate::session::SessionId, slot_seconds: i64) -> Result<Vec<super::SessionTimeline>, crate::obligation::UnmetObligation>;
    }

    /// The query `agentspec.view.SessionTurns` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionTurnsQuery {
        /// Serves `agentspec.view.SessionTurns` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_turns(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionTurns>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl SessionEventCountsQuery for Unimplemented {
        fn session_event_counts(&self, _session_id: crate::session::SessionId, _agent: Option<crate::session::AgentId>) -> Result<Vec<super::SessionEventCounts>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "agentspec.view.SessionEventCounts" })
        }
    }

    impl SessionEventsQuery for Unimplemented {
        fn session_events(&self, _session_id: crate::session::SessionId, _agent: Option<crate::session::AgentId>, _kind: Option<crate::view::EventKind>, _notable: Option<bool>, _before: Option<i64>) -> Result<Vec<super::SessionEvents>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "agentspec.view.SessionEvents" })
        }
    }

    impl SessionListQuery for Unimplemented {
        fn session_list(&self, _since: Option<crate::primitives::Timestamp>, _search: Option<String>, _limit: Option<i64>, _offset: Option<i64>) -> Result<Vec<super::SessionList>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "agentspec.view.SessionList" })
        }
    }
}
