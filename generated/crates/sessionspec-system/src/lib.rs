// generated from sessionspec v1
// contract digest 48f7fe6c6e083e2c80c599aa79f7267068cfeadcc7eaa3ddca0e8128040b6cfd
// do not edit: regenerate with `ess synthesize`

//! The `sessionspec` system, v1: its components assembled, its bindings wired, and its one transport.
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
    /// `sessionspec.session.AgentEnded`.
    AgentEnded(sessionspec_types::session::AgentEnded),
    /// `sessionspec.session.AgentStarted`.
    AgentStarted(sessionspec_types::session::AgentStarted),
    /// `sessionspec.session.ContextCompacted`.
    ContextCompacted(sessionspec_types::session::ContextCompacted),
    /// `sessionspec.session.ExposureDetected`.
    ExposureDetected(sessionspec_types::session::ExposureDetected),
    /// `sessionspec.session.ModelTurnRecorded`.
    ModelTurnRecorded(sessionspec_types::session::ModelTurnRecorded),
    /// `sessionspec.session.PromptSubmitted`.
    PromptSubmitted(sessionspec_types::session::PromptSubmitted),
    /// `sessionspec.session.SessionEnded`.
    SessionEnded(sessionspec_types::session::SessionEnded),
    /// `sessionspec.session.SessionOpened`.
    SessionOpened(sessionspec_types::session::SessionOpened),
    /// `sessionspec.session.ToolCallCompleted`.
    ToolCallCompleted(sessionspec_types::session::ToolCallCompleted),
    /// `sessionspec.session.ToolCallDecided`.
    ToolCallDecided(sessionspec_types::session::ToolCallDecided),
    /// `sessionspec.session.ToolCallRequested`.
    ToolCallRequested(sessionspec_types::session::ToolCallRequested),
    /// `sessionspec.session.TurnEnded`.
    TurnEnded(sessionspec_types::session::TurnEnded),
    /// `sessionspec.view.AgentUpdated`.
    AgentUpdated(sessionspec_types::view::AgentUpdated),
    /// `sessionspec.view.EventAppended`.
    EventAppended(sessionspec_types::view::EventAppended),
    /// `sessionspec.view.SessionUpdated`.
    SessionUpdated(sessionspec_types::view::SessionUpdated),
}

impl SystemEvent {
    /// The qualified name the specification declares this event under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::AgentEnded(_) => "sessionspec.session.AgentEnded",
            Self::AgentStarted(_) => "sessionspec.session.AgentStarted",
            Self::ContextCompacted(_) => "sessionspec.session.ContextCompacted",
            Self::ExposureDetected(_) => "sessionspec.session.ExposureDetected",
            Self::ModelTurnRecorded(_) => "sessionspec.session.ModelTurnRecorded",
            Self::PromptSubmitted(_) => "sessionspec.session.PromptSubmitted",
            Self::SessionEnded(_) => "sessionspec.session.SessionEnded",
            Self::SessionOpened(_) => "sessionspec.session.SessionOpened",
            Self::ToolCallCompleted(_) => "sessionspec.session.ToolCallCompleted",
            Self::ToolCallDecided(_) => "sessionspec.session.ToolCallDecided",
            Self::ToolCallRequested(_) => "sessionspec.session.ToolCallRequested",
            Self::TurnEnded(_) => "sessionspec.session.TurnEnded",
            Self::AgentUpdated(_) => "sessionspec.view.AgentUpdated",
            Self::EventAppended(_) => "sessionspec.view.EventAppended",
            Self::SessionUpdated(_) => "sessionspec.view.SessionUpdated",
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

/// The `sessionspec` system: every component behind its port, and the transport between them.
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
    SessionIrBehaviors: sessionspec_types::session::obligations::CompactContextBehavior + sessionspec_types::session::obligations::CompleteToolCallBehavior + sessionspec_types::session::obligations::DecideToolCallBehavior + sessionspec_types::session::obligations::EndAgentBehavior + sessionspec_types::session::obligations::EndSessionBehavior + sessionspec_types::session::obligations::EndTurnBehavior + sessionspec_types::session::obligations::OpenSessionBehavior + sessionspec_types::session::obligations::RecordExposureBehavior + sessionspec_types::session::obligations::RecordModelTurnBehavior + sessionspec_types::session::obligations::RequestToolCallBehavior + sessionspec_types::session::obligations::StartAgentBehavior + sessionspec_types::session::obligations::SubmitPromptBehavior + sessionspec_types::session::obligations::AgentsQuery + sessionspec_types::session::obligations::ExposuresQuery + sessionspec_types::session::obligations::SessionsQuery + sessionspec_types::session::obligations::ToolCallsQuery,
    SessionViewBehaviors: sessionspec_types::view::obligations::AgentTurnsQuery + sessionspec_types::view::obligations::SessionAgentsQuery + sessionspec_types::view::obligations::SessionByIdQuery + sessionspec_types::view::obligations::SessionChecksQuery + sessionspec_types::view::obligations::SessionEventsQuery + sessionspec_types::view::obligations::SessionListQuery + sessionspec_types::view::obligations::SessionTurnsQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log. No binding
    /// reacts to anything this specification publishes, so collecting is the whole delivery.
    pub fn pump(&mut self) -> Result<(), sessionspec_types::obligation::UnmetObligation> {
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
