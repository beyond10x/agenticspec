// generated from sessionspec v1
// contract digest 5a5e91b41741f78d8f88d9bbbc3f8590bdcf3ccc51173eb11b074eef53627627
// do not edit: regenerate with `ess synthesize`

//! view — `sessionspec.view`.
//!
//! What a session renderer reads: one session as agents and a flat, ordered list of view events derived from the session IR's published events, and the read models of the UI (`ui/sessionspec.ui.yaml`): sessions, agents, events and check verdicts. It holds no content, only names, counts and times, except the harness's session title, which stays in the host's local store. A renderer takes a whole document or an update that appends events; the UI reads the views and takes their changes live. Renderers are generated from the UI document (`ess generate ui`); a gateway UI reuses the document's `embed.*` pages.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// Agent — `sessionspec.view.Agent`.
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

/// The states of `sessionspec.view.AgentListing`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `AgentListing<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentListingState {
    /// `Listed`.
    Listed,
}

/// CheckId — `sessionspec.view.CheckId`: a distinct wrapper around `String`.
///
/// One check of one session, `<session_id>.<check>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckId(pub String);

/// Document — `sessionspec.view.Document`.
///
/// One session for a renderer, format `sessionspec-view/1`.
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
    /// `agents` — `List<sessionspec.view.Agent>`.
    pub agents: Vec<Agent>,
    /// `events` — `List<sessionspec.view.Event>`.
    pub events: Vec<Event>,
}

/// Event — `sessionspec.view.Event`.
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
    /// `kind` — `sessionspec.view.EventKind`.
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

/// EventId — `sessionspec.view.EventId`: a distinct wrapper around `String`.
///
/// One view event of one session, `<session_id>.<seq>`; a dot, since a row key is one segment of an ess-ui node path, which `/` separates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventId(pub String);

/// EventKind — `sessionspec.view.EventKind`: one of a closed set of names.
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

/// The states of `sessionspec.view.EventRow`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `EventRow<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventRowState {
    /// `Logged`.
    Logged,
}

/// The states of `sessionspec.view.SessionCheck`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `SessionCheck<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCheckState {
    /// `Judged`.
    Judged,
}

/// The states of `sessionspec.view.SessionListing`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `SessionListing<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionListingState {
    /// `Listed`.
    Listed,
}

/// Update — `sessionspec.view.Update`.
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
    /// `agents` — `List<sessionspec.view.Agent>`.
    pub agents: Vec<Agent>,
    /// `events` — `List<sessionspec.view.Event>`.
    pub events: Vec<Event>,
}

/// What AgentListing — `sessionspec.view.AgentListing` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`AgentListing<S>`], and at a boundary by [`AgentListingSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentListingData {
    /// The identity: `agent_id` — `sessionspec.session.AgentId`.
    pub agent_id: crate::session::AgentId,
    /// `session_id` — `sessionspec.session.SessionId`.
    ///
    /// Carries `agents_of`: `sessionspec.view.SessionListing` owns many `sessionspec.view.AgentListing`.
    pub session_id: crate::session::SessionId,
    /// `parent` — `Optional<sessionspec.session.AgentId>`.
    ///
    /// Carries `parent_agent`: `sessionspec.view.AgentListing` references one `sessionspec.view.AgentListing`.
    pub parent: Option<crate::session::AgentId>,
    /// `role` — `sessionspec.session.AgentRole`.
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

/// The states of `sessionspec.view.AgentListing`, at the type level.
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

/// AgentListing — `sessionspec.view.AgentListing` — with its lifecycle state carried by the type.
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

/// `sessionspec.view.AgentListing` as it crosses a boundary: the state as a value beside the data.
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

/// What EventRow — `sessionspec.view.EventRow` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`EventRow<S>`], and at a boundary by [`EventRowSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRowData {
    /// The identity: `event_id` — `sessionspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `sessionspec.session.SessionId`.
    ///
    /// Carries `events_of`: `sessionspec.view.SessionListing` owns many `sessionspec.view.EventRow`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `sessionspec.session.AgentId`.
    ///
    /// Carries `of_agent`: `sessionspec.view.EventRow` references one `sessionspec.view.AgentListing`.
    pub agent: crate::session::AgentId,
    /// `kind` — `sessionspec.view.EventKind`.
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
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// The states of `sessionspec.view.EventRow`, at the type level.
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

/// EventRow — `sessionspec.view.EventRow` — with its lifecycle state carried by the type.
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

/// `sessionspec.view.EventRow` as it crosses a boundary: the state as a value beside the data.
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

/// What SessionCheck — `sessionspec.view.SessionCheck` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`SessionCheck<S>`], and at a boundary by [`SessionCheckSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCheckData {
    /// The identity: `check_id` — `sessionspec.view.CheckId`.
    pub check_id: CheckId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `check` — `String`.
    pub check: String,
    /// `about` — `String`.
    pub about: String,
    /// `severity` — `String`.
    pub severity: String,
    /// `verdict` — `sessionspec.session.Verdict`.
    pub verdict: crate::session::Verdict,
    /// `rows` — `Integer`.
    pub rows: i64,
    /// `possible` — `Integer`.
    pub possible: i64,
}

/// The states of `sessionspec.view.SessionCheck`, at the type level.
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

/// SessionCheck — `sessionspec.view.SessionCheck` — with its lifecycle state carried by the type.
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

/// `sessionspec.view.SessionCheck` as it crosses a boundary: the state as a value beside the data.
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

/// What SessionListing — `sessionspec.view.SessionListing` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`SessionListing<S>`], and at a boundary by [`SessionListingSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListingData {
    /// The identity: `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `names` — `String`.
    pub names: String,
    /// `rank` — `Integer`.
    pub rank: i64,
    /// `harness` — `sessionspec.session.Harness`.
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
    /// `kpis` — `sessionspec.cli.KpiReport`.
    pub kpis: crate::cli::KpiReport,
}

/// The states of `sessionspec.view.SessionListing`, at the type level.
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

/// SessionListing — `sessionspec.view.SessionListing` — with its lifecycle state carried by the type.
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

/// `sessionspec.view.SessionListing` as it crosses a boundary: the state as a value beside the data.
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

/// AgentUpdated — the event `sessionspec.view.AgentUpdated`.
///
/// An agent started or its figures changed; the payload is its whole row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUpdated {
    /// `agent_id` — `sessionspec.session.AgentId`.
    pub agent_id: crate::session::AgentId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `parent` — `Optional<sessionspec.session.AgentId>`.
    pub parent: Option<crate::session::AgentId>,
    /// `role` — `sessionspec.session.AgentRole`.
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

/// EventAppended — the event `sessionspec.view.EventAppended`.
///
/// One event of a running session; the payload is its row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventAppended {
    /// `event_id` — `sessionspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `sessionspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `sessionspec.view.EventKind`.
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
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// SessionUpdated — the event `sessionspec.view.SessionUpdated`.
///
/// The figures of a session changed; the payload is its whole row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionUpdated {
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `names` — `String`.
    pub names: String,
    /// `rank` — `Integer`.
    pub rank: i64,
    /// `harness` — `sessionspec.session.Harness`.
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

/// Model turns of an agent — one row of the view `sessionspec.view.AgentTurns`.
///
/// Projects `sessionspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and agent == param.agent and kind == model_turn)`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurns {
    /// `event_id` — `sessionspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `sessionspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `sessionspec.view.EventKind`.
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
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// Agents of a session — one row of the view `sessionspec.view.SessionAgents`.
///
/// Projects `sessionspec.view.AgentListing` at `eventual` consistency, containing instances where `session_id == param.session_id`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAgents {
    /// `agent_id` — `sessionspec.session.AgentId`.
    pub agent_id: crate::session::AgentId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `parent` — `Optional<sessionspec.session.AgentId>`.
    pub parent: Option<crate::session::AgentId>,
    /// `role` — `sessionspec.session.AgentRole`.
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

/// Session — one row of the view `sessionspec.view.SessionById`.
///
/// Projects `sessionspec.view.SessionListing` at `eventual` consistency, containing instances where `session_id == param.session_id`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionById {
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `harness` — `sessionspec.session.Harness`.
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
    /// `kpis` — `sessionspec.cli.KpiReport`.
    pub kpis: crate::cli::KpiReport,
}

/// Checks of a session — one row of the view `sessionspec.view.SessionChecks`.
///
/// Projects `sessionspec.view.SessionCheck` at `eventual` consistency, containing instances where `session_id == param.session_id`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionChecks {
    /// `check_id` — `sessionspec.view.CheckId`.
    pub check_id: CheckId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `check` — `String`.
    pub check: String,
    /// `about` — `String`.
    pub about: String,
    /// `severity` — `String`.
    pub severity: String,
    /// `verdict` — `sessionspec.session.Verdict`.
    pub verdict: crate::session::Verdict,
    /// `rows` — `Integer`.
    pub rows: i64,
    /// `possible` — `Integer`.
    pub possible: i64,
}

/// Events of a session — one row of the view `sessionspec.view.SessionEvents`.
///
/// Projects `sessionspec.view.EventRow` at `eventual` consistency, containing instances where `session_id == param.session_id`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEvents {
    /// `event_id` — `sessionspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `sessionspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `sessionspec.view.EventKind`.
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
    /// `agent_cost_usd` — `Binary64`.
    pub agent_cost_usd: crate::primitives::Binary64,
    /// `session_cost_usd` — `Binary64`.
    pub session_cost_usd: crate::primitives::Binary64,
}

/// Sessions — one row of the view `sessionspec.view.SessionList`.
///
/// Projects `sessionspec.view.SessionListing` at `eventual` consistency, containing instances where `((not (defined(param.since)) or last_at >= param.since as timestamp) and (not (defined(param.search)) or names contains {param: search}) and (not (defined(param.limit)) or rank <= param.limit))`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionList {
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `names` — `String`.
    pub names: String,
    /// `rank` — `Integer`.
    pub rank: i64,
    /// `harness` — `sessionspec.session.Harness`.
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

/// Model turns of a session — one row of the view `sessionspec.view.SessionTurns`.
///
/// Projects `sessionspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and kind == model_turn)`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTurns {
    /// `event_id` — `sessionspec.view.EventId`.
    pub event_id: EventId,
    /// `session_id` — `sessionspec.session.SessionId`.
    pub session_id: crate::session::SessionId,
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `at` — `Timestamp`.
    pub at: crate::primitives::Timestamp,
    /// `agent` — `sessionspec.session.AgentId`.
    pub agent: crate::session::AgentId,
    /// `kind` — `sessionspec.view.EventKind`.
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
    /// The query `sessionspec.view.AgentTurns` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait AgentTurnsQuery {
        /// Serves `sessionspec.view.AgentTurns` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn agent_turns(&self, session_id: crate::session::SessionId, agent: crate::session::AgentId) -> Result<Vec<super::AgentTurns>, crate::obligation::UnmetObligation>;
    }

    /// The query `sessionspec.view.SessionAgents` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionAgentsQuery {
        /// Serves `sessionspec.view.SessionAgents` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_agents(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionAgents>, crate::obligation::UnmetObligation>;
    }

    /// The query `sessionspec.view.SessionById` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionByIdQuery {
        /// Serves `sessionspec.view.SessionById` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_by_id(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionById>, crate::obligation::UnmetObligation>;
    }

    /// The query `sessionspec.view.SessionChecks` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionChecksQuery {
        /// Serves `sessionspec.view.SessionChecks` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_checks(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionChecks>, crate::obligation::UnmetObligation>;
    }

    /// The query `sessionspec.view.SessionEvents` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionEventsQuery {
        /// Serves `sessionspec.view.SessionEvents` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_events(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionEvents>, crate::obligation::UnmetObligation>;
    }

    /// The query `sessionspec.view.SessionList` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a comparison with the optional view parameter `since`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:`.
    ///
    /// Contract: a query answering `sessionspec.view.SessionList` with rows projected from `sessionspec.view.SessionListing` at `eventual` consistency, containing instances where `((not (defined(param.since)) or last_at >= param.since as timestamp) and (not (defined(param.search)) or names contains {param: search}) and (not (defined(param.limit)) or rank <= param.limit))`.
    pub trait SessionListQuery {
        /// Serves `sessionspec.view.SessionList` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn session_list(&self, since: Option<crate::primitives::Timestamp>, search: Option<String>, limit: Option<i64>) -> Result<Vec<super::SessionList>, crate::obligation::UnmetObligation>;
    }

    /// The query `sessionspec.view.SessionTurns` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SessionTurnsQuery {
        /// Serves `sessionspec.view.SessionTurns` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn session_turns(&self, session_id: crate::session::SessionId) -> Result<Vec<super::SessionTurns>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl SessionListQuery for Unimplemented {
        fn session_list(&self, _since: Option<crate::primitives::Timestamp>, _search: Option<String>, _limit: Option<i64>) -> Result<Vec<super::SessionList>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "sessionspec.view.SessionList" })
        }
    }
}
