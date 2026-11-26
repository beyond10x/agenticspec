// generated from sessionspec v1
// contract digest 48f7fe6c6e083e2c80c599aa79f7267068cfeadcc7eaa3ddca0e8128040b6cfd
// do not edit: regenerate with `ess synthesize`

//! session-ir — the `session-ir` component of `sessionspec` v1.
//!
//! Replays the commands an adapter lifted from a transcript and holds the resulting session state.
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
}

/// session-ir — the port over the component's obligations.
///
/// `B` bundles every behaviour and query this component owes; constructing it over the domain's
/// `obligations::Unimplemented` yields a component that compiles and refuses, in the type system,
/// everything not yet implemented.
pub struct SessionIr<B> {
    behaviors: B,
    outbox: Vec<PublishedEvent>,
}

impl<B> SessionIr<B> {
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

impl<B> SessionIr<B>
where
    B: sessionspec_types::session::obligations::CompactContextBehavior + sessionspec_types::session::obligations::CompleteToolCallBehavior + sessionspec_types::session::obligations::DecideToolCallBehavior + sessionspec_types::session::obligations::EndAgentBehavior + sessionspec_types::session::obligations::EndSessionBehavior + sessionspec_types::session::obligations::EndTurnBehavior + sessionspec_types::session::obligations::OpenSessionBehavior + sessionspec_types::session::obligations::RecordExposureBehavior + sessionspec_types::session::obligations::RecordModelTurnBehavior + sessionspec_types::session::obligations::RequestToolCallBehavior + sessionspec_types::session::obligations::StartAgentBehavior + sessionspec_types::session::obligations::SubmitPromptBehavior + sessionspec_types::session::obligations::AgentsQuery + sessionspec_types::session::obligations::ExposuresQuery + sessionspec_types::session::obligations::SessionsQuery + sessionspec_types::session::obligations::ToolCallsQuery,
{
    /// Accepts `sessionspec.session.CompactContext`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn compact_context(&mut self, input: sessionspec_types::session::CompactContext) -> Result<sessionspec_types::session::CompactContextOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.compact_context(input)?;
        match &outcome {
            sessionspec_types::session::CompactContextOutcome::Compacted { context_compacted, .. } => {
                self.outbox.push(PublishedEvent::ContextCompacted(context_compacted.clone()));
            }
            sessionspec_types::session::CompactContextOutcome::NoSuchAgent { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.CompleteToolCall`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn complete_tool_call(&mut self, input: sessionspec_types::session::CompleteToolCall) -> Result<sessionspec_types::session::CompleteToolCallOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.complete_tool_call(input)?;
        match &outcome {
            sessionspec_types::session::CompleteToolCallOutcome::Succeeded { tool_call_completed, .. } => {
                self.outbox.push(PublishedEvent::ToolCallCompleted(tool_call_completed.clone()));
            }
            sessionspec_types::session::CompleteToolCallOutcome::Failed { tool_call_completed, .. } => {
                self.outbox.push(PublishedEvent::ToolCallCompleted(tool_call_completed.clone()));
            }
            sessionspec_types::session::CompleteToolCallOutcome::WrongState { .. } => {}
            sessionspec_types::session::CompleteToolCallOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.DecideToolCall`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn decide_tool_call(&mut self, input: sessionspec_types::session::DecideToolCall) -> Result<sessionspec_types::session::DecideToolCallOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.decide_tool_call(input)?;
        match &outcome {
            sessionspec_types::session::DecideToolCallOutcome::Accepted { tool_call_decided, .. } => {
                self.outbox.push(PublishedEvent::ToolCallDecided(tool_call_decided.clone()));
            }
            sessionspec_types::session::DecideToolCallOutcome::Refused { tool_call_decided, .. } => {
                self.outbox.push(PublishedEvent::ToolCallDecided(tool_call_decided.clone()));
            }
            sessionspec_types::session::DecideToolCallOutcome::WrongState { .. } => {}
            sessionspec_types::session::DecideToolCallOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.EndAgent`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn end_agent(&mut self, input: sessionspec_types::session::EndAgent) -> Result<sessionspec_types::session::EndAgentOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.end_agent(input)?;
        match &outcome {
            sessionspec_types::session::EndAgentOutcome::Ended { agent_ended, .. } => {
                self.outbox.push(PublishedEvent::AgentEnded(agent_ended.clone()));
            }
            sessionspec_types::session::EndAgentOutcome::WrongState { .. } => {}
            sessionspec_types::session::EndAgentOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.EndSession`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn end_session(&mut self, input: sessionspec_types::session::EndSession) -> Result<sessionspec_types::session::EndSessionOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.end_session(input)?;
        match &outcome {
            sessionspec_types::session::EndSessionOutcome::Ended { session_ended, .. } => {
                self.outbox.push(PublishedEvent::SessionEnded(session_ended.clone()));
            }
            sessionspec_types::session::EndSessionOutcome::WrongState { .. } => {}
            sessionspec_types::session::EndSessionOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.EndTurn`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn end_turn(&mut self, input: sessionspec_types::session::EndTurn) -> Result<sessionspec_types::session::EndTurnOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.end_turn(input)?;
        match &outcome {
            sessionspec_types::session::EndTurnOutcome::Ended { turn_ended, .. } => {
                self.outbox.push(PublishedEvent::TurnEnded(turn_ended.clone()));
            }
            sessionspec_types::session::EndTurnOutcome::WrongState { .. } => {}
            sessionspec_types::session::EndTurnOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.OpenSession`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn open_session(&mut self, input: sessionspec_types::session::OpenSession) -> Result<sessionspec_types::session::OpenSessionOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.open_session(input)?;
        match &outcome {
            sessionspec_types::session::OpenSessionOutcome::Opened { session_opened, .. } => {
                self.outbox.push(PublishedEvent::SessionOpened(session_opened.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.RecordExposure`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_exposure(&mut self, input: sessionspec_types::session::RecordExposure) -> Result<sessionspec_types::session::RecordExposureOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_exposure(input)?;
        match &outcome {
            sessionspec_types::session::RecordExposureOutcome::Recorded { exposure_detected, .. } => {
                self.outbox.push(PublishedEvent::ExposureDetected(exposure_detected.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.RecordModelTurn`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_model_turn(&mut self, input: sessionspec_types::session::RecordModelTurn) -> Result<sessionspec_types::session::RecordModelTurnOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_model_turn(input)?;
        match &outcome {
            sessionspec_types::session::RecordModelTurnOutcome::Recorded { model_turn_recorded, .. } => {
                self.outbox.push(PublishedEvent::ModelTurnRecorded(model_turn_recorded.clone()));
            }
            sessionspec_types::session::RecordModelTurnOutcome::NoSuchAgent { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.RequestToolCall`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn request_tool_call(&mut self, input: sessionspec_types::session::RequestToolCall) -> Result<sessionspec_types::session::RequestToolCallOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.request_tool_call(input)?;
        match &outcome {
            sessionspec_types::session::RequestToolCallOutcome::Requested { tool_call_requested, .. } => {
                self.outbox.push(PublishedEvent::ToolCallRequested(tool_call_requested.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.StartAgent`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn start_agent(&mut self, input: sessionspec_types::session::StartAgent) -> Result<sessionspec_types::session::StartAgentOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.start_agent(input)?;
        match &outcome {
            sessionspec_types::session::StartAgentOutcome::Started { agent_started, .. } => {
                self.outbox.push(PublishedEvent::AgentStarted(agent_started.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `sessionspec.session.SubmitPrompt`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn submit_prompt(&mut self, input: sessionspec_types::session::SubmitPrompt) -> Result<sessionspec_types::session::SubmitPromptOutcome, sessionspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.submit_prompt(input)?;
        match &outcome {
            sessionspec_types::session::SubmitPromptOutcome::Submitted { prompt_submitted, .. } => {
                self.outbox.push(PublishedEvent::PromptSubmitted(prompt_submitted.clone()));
            }
            sessionspec_types::session::SubmitPromptOutcome::WrongState { .. } => {}
            sessionspec_types::session::SubmitPromptOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Serves `sessionspec.session.Agents` at `read_your_writes` consistency, from the owed projection.
    pub fn agents(&self) -> Result<Vec<sessionspec_types::session::Agents>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.agents()
    }

    /// Serves `sessionspec.session.Exposures` at `read_your_writes` consistency, from the owed projection.
    pub fn exposures(&self) -> Result<Vec<sessionspec_types::session::Exposures>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.exposures()
    }

    /// Serves `sessionspec.session.Sessions` at `read_your_writes` consistency, from the owed projection.
    pub fn sessions(&self) -> Result<Vec<sessionspec_types::session::Sessions>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.sessions()
    }

    /// Serves `sessionspec.session.ToolCalls` at `read_your_writes` consistency, from the owed projection.
    pub fn tool_calls(&self) -> Result<Vec<sessionspec_types::session::ToolCalls>, sessionspec_types::obligation::UnmetObligation> {
        self.behaviors.tool_calls()
    }
}
