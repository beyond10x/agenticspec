// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! session-view — the `session-view` component of `agentspec` v1.
//!
//! Serves the read models of the UI (`ui/agentspec.ui.yaml`) over HTTP: the session list, one session, its agents and its events, and publishes their changes for a live page.
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
    /// `agentspec.view.AgentUpdated`.
    AgentUpdated(agentspec_types::view::AgentUpdated),
    /// `agentspec.view.EventAppended`.
    EventAppended(agentspec_types::view::EventAppended),
    /// `agentspec.view.SessionUpdated`.
    SessionUpdated(agentspec_types::view::SessionUpdated),
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
    B: agentspec_types::view::obligations::AgentTurnsQuery + agentspec_types::view::obligations::OverviewFiguresQuery + agentspec_types::view::obligations::SessionAgentsQuery + agentspec_types::view::obligations::SessionByIdQuery + agentspec_types::view::obligations::SessionChecksQuery + agentspec_types::view::obligations::SessionEventCountsQuery + agentspec_types::view::obligations::SessionEventsQuery + agentspec_types::view::obligations::SessionListQuery + agentspec_types::view::obligations::SessionTimelineQuery + agentspec_types::view::obligations::SessionTurnsQuery,
{
    /// Serves `agentspec.view.AgentTurns` at `eventual` consistency, from the owed projection.
    pub fn agent_turns(&self, session_id: agentspec_types::session::SessionId, agent: agentspec_types::session::AgentId) -> Result<Vec<agentspec_types::view::AgentTurns>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.agent_turns(session_id, agent)
    }

    /// Serves `agentspec.view.OverviewFigures` at `eventual` consistency, from the owed projection.
    pub fn overview_figures(&self, since: agentspec_types::primitives::Timestamp, bucket_seconds: i64) -> Result<Vec<agentspec_types::view::OverviewFigures>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.overview_figures(since, bucket_seconds)
    }

    /// Serves `agentspec.view.SessionAgents` at `eventual` consistency, from the owed projection.
    pub fn session_agents(&self, session_id: agentspec_types::session::SessionId) -> Result<Vec<agentspec_types::view::SessionAgents>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_agents(session_id)
    }

    /// Serves `agentspec.view.SessionById` at `eventual` consistency, from the owed projection.
    pub fn session_by_id(&self, session_id: agentspec_types::session::SessionId) -> Result<Vec<agentspec_types::view::SessionById>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_by_id(session_id)
    }

    /// Serves `agentspec.view.SessionChecks` at `eventual` consistency, from the owed projection.
    pub fn session_checks(&self, session_id: agentspec_types::session::SessionId) -> Result<Vec<agentspec_types::view::SessionChecks>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_checks(session_id)
    }

    /// Serves `agentspec.view.SessionEventCounts` at `eventual` consistency, from the owed projection.
    pub fn session_event_counts(&self, session_id: agentspec_types::session::SessionId, agent: Option<agentspec_types::session::AgentId>) -> Result<Vec<agentspec_types::view::SessionEventCounts>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_event_counts(session_id, agent)
    }

    /// Serves `agentspec.view.SessionEvents` at `eventual` consistency, from the owed projection.
    pub fn session_events(&self, session_id: agentspec_types::session::SessionId, agent: Option<agentspec_types::session::AgentId>, kind: Option<agentspec_types::view::EventKind>, notable: Option<bool>, before: Option<i64>) -> Result<Vec<agentspec_types::view::SessionEvents>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_events(session_id, agent, kind, notable, before)
    }

    /// Serves `agentspec.view.SessionList` at `eventual` consistency, from the owed projection.
    pub fn session_list(&self, since: Option<agentspec_types::primitives::Timestamp>, search: Option<String>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<agentspec_types::view::SessionList>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_list(since, search, limit, offset)
    }

    /// Serves `agentspec.view.SessionTimeline` at `eventual` consistency, from the owed projection.
    pub fn session_timeline(&self, session_id: agentspec_types::session::SessionId, slot_seconds: i64) -> Result<Vec<agentspec_types::view::SessionTimeline>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_timeline(session_id, slot_seconds)
    }

    /// Serves `agentspec.view.SessionTurns` at `eventual` consistency, from the owed projection.
    pub fn session_turns(&self, session_id: agentspec_types::session::SessionId) -> Result<Vec<agentspec_types::view::SessionTurns>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.session_turns(session_id)
    }
}
