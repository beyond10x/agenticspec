// generated from sessionspec v1
// contract digest 7eaa4202b2be444349f069c24e3e8839a7d36b41fbc39fdf08c4042538b65d40
// do not edit: regenerate with `ess synthesize`

//! session-view — the `session-view` component of `sessionspec` v1.
//!
//! Serves the read models of the UI (`ui/sessionspec.ui.yaml`) over HTTP: the session list, one session, its agents and its events, and publishes their changes for a live page.
//!
//! The component's outer surface exactly as the specification declares it: accepted commands as
//! handlers, declared views as queries, published events as a typed outbox. The behaviour behind
//! every handler is an implementation obligation — see the `PLAN.md` beside this workspace — and
//! until one is satisfied, its stub answers with a typed refusal naming what is owed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event this component declares it publishes, on its way to the system's transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedEvent {
    /// `sessionspec.view.AgentUpdated`.
    AgentUpdated(sessionspec_types::view::AgentUpdated),
    /// `sessionspec.view.EventAppended`.
    EventAppended(sessionspec_types::view::EventAppended),
    /// `sessionspec.view.SessionUpdated`.
    SessionUpdated(sessionspec_types::view::SessionUpdated),
}

/// session-view — the port over the component's obligations.
///
/// `B` bundles every behaviour and query this component owes; constructing it over the domain's
/// `obligations::Unimplemented` yields a component that compiles and refuses, in the type system,
/// everything not yet implemented.
pub struct SessionView<B> {
    behaviors: B,
    outbox: Vec<PublishedEvent>,
}

impl<B> SessionView<B> {
    /// A new port over the given obligation implementations.
    pub fn new(behaviors: B) -> Self {
        Self {
            behaviors,
            outbox: Vec::new(),
        }
    }

    /// Hands over everything published since the last drain, in publication order.
    ///
    /// The system's transport calls this; anything else reading it is taking events the transport
    /// will then never deliver.
    pub fn drain_outbox(&mut self) -> Vec<PublishedEvent> {
        core::mem::take(&mut self.outbox)
    }
}

impl<B> SessionView<B>
where
    B: sessionspec_types::view::obligations::AgentTurnsQuery + sessionspec_types::view::obligations::OverviewFiguresQuery + sessionspec_types::view::obligations::SessionAgentsQuery + sessionspec_types::view::obligations::SessionByIdQuery + sessionspec_types::view::obligations::SessionChecksQuery + sessionspec_types::view::obligations::SessionEventCountsQuery + sessionspec_types::view::obligations::SessionEventsQuery + sessionspec_types::view::obligations::SessionListQuery + sessionspec_types::view::obligations::SessionTimelineQuery + sessionspec_types::view::obligations::SessionTurnsQuery,
{
    /// Serves `sessionspec.view.AgentTurns` at `eventual` consistency, from the owed projection.
    pub fn agent_turns(&self, session_id: sessionspec_types::session::SessionId, agent: sessionspec_types::session::AgentId) -> Result<Vec<sessionspec_types::view::AgentTurns>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.agent_turns(session_id, agent)
    }

    /// Serves `sessionspec.view.OverviewFigures` at `eventual` consistency, from the owed projection.
    pub fn overview_figures(&self, since: sessionspec_types::primitives::Timestamp, bucket_seconds: i64) -> Result<Vec<sessionspec_types::view::OverviewFigures>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.overview_figures(since, bucket_seconds)
    }

    /// Serves `sessionspec.view.SessionAgents` at `eventual` consistency, from the owed projection.
    pub fn session_agents(&self, session_id: sessionspec_types::session::SessionId) -> Result<Vec<sessionspec_types::view::SessionAgents>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_agents(session_id)
    }

    /// Serves `sessionspec.view.SessionById` at `eventual` consistency, from the owed projection.
    pub fn session_by_id(&self, session_id: sessionspec_types::session::SessionId) -> Result<Vec<sessionspec_types::view::SessionById>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_by_id(session_id)
    }

    /// Serves `sessionspec.view.SessionChecks` at `eventual` consistency, from the owed projection.
    pub fn session_checks(&self, session_id: sessionspec_types::session::SessionId) -> Result<Vec<sessionspec_types::view::SessionChecks>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_checks(session_id)
    }

    /// Serves `sessionspec.view.SessionEventCounts` at `eventual` consistency, from the owed projection.
    pub fn session_event_counts(&self, session_id: sessionspec_types::session::SessionId, agent: Option<sessionspec_types::session::AgentId>) -> Result<Vec<sessionspec_types::view::SessionEventCounts>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_event_counts(session_id, agent)
    }

    /// Serves `sessionspec.view.SessionEvents` at `eventual` consistency, from the owed projection.
    pub fn session_events(&self, session_id: sessionspec_types::session::SessionId, agent: Option<sessionspec_types::session::AgentId>, kind: Option<sessionspec_types::view::EventKind>, notable: Option<bool>, before: Option<i64>) -> Result<Vec<sessionspec_types::view::SessionEvents>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_events(session_id, agent, kind, notable, before)
    }

    /// Serves `sessionspec.view.SessionList` at `eventual` consistency, from the owed projection.
    pub fn session_list(&self, since: Option<sessionspec_types::primitives::Timestamp>, search: Option<String>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<sessionspec_types::view::SessionList>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_list(since, search, limit, offset)
    }

    /// Serves `sessionspec.view.SessionTimeline` at `eventual` consistency, from the owed projection.
    pub fn session_timeline(&self, session_id: sessionspec_types::session::SessionId, slot_seconds: i64) -> Result<Vec<sessionspec_types::view::SessionTimeline>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_timeline(session_id, slot_seconds)
    }

    /// Serves `sessionspec.view.SessionTurns` at `eventual` consistency, from the owed projection.
    pub fn session_turns(&self, session_id: sessionspec_types::session::SessionId) -> Result<Vec<sessionspec_types::view::SessionTurns>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.session_turns(session_id)
    }
}
