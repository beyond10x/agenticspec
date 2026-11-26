// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! The `agentspec` system, v1: its components assembled, its bindings wired, and its one transport.
//!
//! The transport is derived from the specification, not chosen: `at_least_once` is the only
//! delivery guarantee the model declares, so published events land on an append-only log and a
//! pump delivers each to every binding that reacts to it. The log is the system's observable
//! record, and so is the record of what each binding invoked. What no specification determines
//! — how an escalation event is filled, behaviour behind the ports — stays an obligation; see
//! the `PLAN.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event on the system's log: everything any component publishes, and everything a binding
/// escalates into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemEvent {
    /// `agentspec.session.AgentEnded`.
    AgentEnded(agentspec_types::session::AgentEnded),
    /// `agentspec.session.AgentStarted`.
    AgentStarted(agentspec_types::session::AgentStarted),
    /// `agentspec.session.ContextCompacted`.
    ContextCompacted(agentspec_types::session::ContextCompacted),
    /// `agentspec.session.ExposureDetected`.
    ExposureDetected(agentspec_types::session::ExposureDetected),
    /// `agentspec.session.ModelTurnRecorded`.
    ModelTurnRecorded(agentspec_types::session::ModelTurnRecorded),
    /// `agentspec.session.PromptSubmitted`.
    PromptSubmitted(agentspec_types::session::PromptSubmitted),
    /// `agentspec.session.SessionEnded`.
    SessionEnded(agentspec_types::session::SessionEnded),
    /// `agentspec.session.SessionOpened`.
    SessionOpened(agentspec_types::session::SessionOpened),
    /// `agentspec.session.ToolCallCompleted`.
    ToolCallCompleted(agentspec_types::session::ToolCallCompleted),
    /// `agentspec.session.ToolCallDecided`.
    ToolCallDecided(agentspec_types::session::ToolCallDecided),
    /// `agentspec.session.ToolCallRequested`.
    ToolCallRequested(agentspec_types::session::ToolCallRequested),
    /// `agentspec.session.TurnEnded`.
    TurnEnded(agentspec_types::session::TurnEnded),
    /// `agentspec.view.AgentUpdated`.
    AgentUpdated(agentspec_types::view::AgentUpdated),
    /// `agentspec.view.EventAppended`.
    EventAppended(agentspec_types::view::EventAppended),
    /// `agentspec.view.SessionUpdated`.
    SessionUpdated(agentspec_types::view::SessionUpdated),
}

impl SystemEvent {
    /// The qualified name the specification declares this event under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::AgentEnded(_) => "agentspec.session.AgentEnded",
            Self::AgentStarted(_) => "agentspec.session.AgentStarted",
            Self::ContextCompacted(_) => "agentspec.session.ContextCompacted",
            Self::ExposureDetected(_) => "agentspec.session.ExposureDetected",
            Self::ModelTurnRecorded(_) => "agentspec.session.ModelTurnRecorded",
            Self::PromptSubmitted(_) => "agentspec.session.PromptSubmitted",
            Self::SessionEnded(_) => "agentspec.session.SessionEnded",
            Self::SessionOpened(_) => "agentspec.session.SessionOpened",
            Self::ToolCallCompleted(_) => "agentspec.session.ToolCallCompleted",
            Self::ToolCallDecided(_) => "agentspec.session.ToolCallDecided",
            Self::ToolCallRequested(_) => "agentspec.session.ToolCallRequested",
            Self::TurnEnded(_) => "agentspec.session.TurnEnded",
            Self::AgentUpdated(_) => "agentspec.view.AgentUpdated",
            Self::EventAppended(_) => "agentspec.view.EventAppended",
            Self::SessionUpdated(_) => "agentspec.view.SessionUpdated",
        }
    }
}

impl From<session_ir::PublishedEvent> for SystemEvent {
    fn from(event: session_ir::PublishedEvent) -> Self {
        match event {
            session_ir::PublishedEvent::AgentEnded(event) => Self::AgentEnded(event),
            session_ir::PublishedEvent::AgentStarted(event) => Self::AgentStarted(event),
            session_ir::PublishedEvent::ContextCompacted(event) => Self::ContextCompacted(event),
            session_ir::PublishedEvent::ExposureDetected(event) => Self::ExposureDetected(event),
            session_ir::PublishedEvent::ModelTurnRecorded(event) => Self::ModelTurnRecorded(event),
            session_ir::PublishedEvent::PromptSubmitted(event) => Self::PromptSubmitted(event),
            session_ir::PublishedEvent::SessionEnded(event) => Self::SessionEnded(event),
            session_ir::PublishedEvent::SessionOpened(event) => Self::SessionOpened(event),
            session_ir::PublishedEvent::ToolCallCompleted(event) => Self::ToolCallCompleted(event),
            session_ir::PublishedEvent::ToolCallDecided(event) => Self::ToolCallDecided(event),
            session_ir::PublishedEvent::ToolCallRequested(event) => Self::ToolCallRequested(event),
            session_ir::PublishedEvent::TurnEnded(event) => Self::TurnEnded(event),
        }
    }
}

impl From<session_view::PublishedEvent> for SystemEvent {
    fn from(event: session_view::PublishedEvent) -> Self {
        match event {
            session_view::PublishedEvent::AgentUpdated(event) => Self::AgentUpdated(event),
            session_view::PublishedEvent::EventAppended(event) => Self::EventAppended(event),
            session_view::PublishedEvent::SessionUpdated(event) => Self::SessionUpdated(event),
        }
    }
}

/// The `agentspec` system: every component behind its port, and the transport between them.
///
/// The component fields are public because commands enter the system through a component's own
/// port; the log and its delivery cursor are not, because publishing happens by pumping, not by
/// writing history directly.
pub struct System<SessionIrBehaviors, SessionViewBehaviors> {
    /// The `session-ir` component.
    pub session_ir: session_ir::SessionIr<SessionIrBehaviors>,
    /// The `session-view` component.
    pub session_view: session_view::SessionView<SessionViewBehaviors>,
    published: Vec<SystemEvent>,
    cursor: usize,
}

impl<SessionIrBehaviors, SessionViewBehaviors> System<SessionIrBehaviors, SessionViewBehaviors> {
    /// Assembles the system from its components.
    pub fn new(session_ir: session_ir::SessionIr<SessionIrBehaviors>, session_view: session_view::SessionView<SessionViewBehaviors>) -> Self {
        Self {
            session_ir,
            session_view,
            published: Vec::new(),
            cursor: 0,
        }
    }

    /// Everything published so far, in publication order — the system's observable record.
    pub fn published(&self) -> &[SystemEvent] {
        &self.published
    }

    /// Takes every event the pump has already delivered off the log, in publication order.
    ///
    /// A long-running shell calls this after each `pump`, or the log holds every event the
    /// process ever published. A `pump` returns with every logged event delivered: each
    /// reacting binding has had its attempt, and a binding whose attempt stopped holds the event in
    /// its own held-back list, not on the log. Events published since the last `pump` stay on the
    /// log, so the next `pump` still delivers them; taking never skips a binding.
    pub fn take_published(&mut self) -> Vec<SystemEvent> {
        let delivered: Vec<SystemEvent> = self.published.drain(..self.cursor).collect();
        self.cursor = 0;
        delivered
    }
}

impl<SessionIrBehaviors, SessionViewBehaviors> System<SessionIrBehaviors, SessionViewBehaviors>
where
    SessionIrBehaviors: agentspec_types::session::obligations::CompactContextBehavior + agentspec_types::session::obligations::CompleteToolCallBehavior + agentspec_types::session::obligations::DecideToolCallBehavior + agentspec_types::session::obligations::EndAgentBehavior + agentspec_types::session::obligations::EndSessionBehavior + agentspec_types::session::obligations::EndTurnBehavior + agentspec_types::session::obligations::OpenSessionBehavior + agentspec_types::session::obligations::RecordExposureBehavior + agentspec_types::session::obligations::RecordModelTurnBehavior + agentspec_types::session::obligations::RequestToolCallBehavior + agentspec_types::session::obligations::StartAgentBehavior + agentspec_types::session::obligations::SubmitPromptBehavior + agentspec_types::session::obligations::AgentsQuery + agentspec_types::session::obligations::ExposuresQuery + agentspec_types::session::obligations::SessionsQuery + agentspec_types::session::obligations::ToolCallsQuery,
    SessionViewBehaviors: agentspec_types::view::obligations::AgentTurnsQuery + agentspec_types::view::obligations::OverviewFiguresQuery + agentspec_types::view::obligations::SessionAgentsQuery + agentspec_types::view::obligations::SessionByIdQuery + agentspec_types::view::obligations::SessionChecksQuery + agentspec_types::view::obligations::SessionEventCountsQuery + agentspec_types::view::obligations::SessionEventsQuery + agentspec_types::view::obligations::SessionListQuery + agentspec_types::view::obligations::SessionTimelineQuery + agentspec_types::view::obligations::SessionTurnsQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log. No binding
    /// reacts to anything this specification publishes, so collecting is the whole delivery.
    pub fn pump(&mut self) -> Result<(), agentspec_types::obligation::UnmetObligation> {
        loop {
            self.collect();
            if self.cursor == self.published.len() {
                return Ok(());
            }
            self.cursor += 1;
        }
    }

    /// Moves every component's outbox onto the log, in component order.
    fn collect(&mut self) {
        for event in self.session_ir.drain_outbox() {
            self.published.push(SystemEvent::from(event));
        }
        for event in self.session_view.drain_outbox() {
            self.published.push(SystemEvent::from(event));
        }
    }
}
