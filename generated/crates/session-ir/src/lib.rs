// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! session-ir — the `session-ir` component of `agentspec` v1.
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
    B: agentspec_types::session::obligations::CompactContextBehavior + agentspec_types::session::obligations::CompleteToolCallBehavior + agentspec_types::session::obligations::DecideToolCallBehavior + agentspec_types::session::obligations::EndAgentBehavior + agentspec_types::session::obligations::EndSessionBehavior + agentspec_types::session::obligations::EndTurnBehavior + agentspec_types::session::obligations::OpenSessionBehavior + agentspec_types::session::obligations::RecordExposureBehavior + agentspec_types::session::obligations::RecordModelTurnBehavior + agentspec_types::session::obligations::RequestToolCallBehavior + agentspec_types::session::obligations::StartAgentBehavior + agentspec_types::session::obligations::SubmitPromptBehavior + agentspec_types::session::obligations::AgentsQuery + agentspec_types::session::obligations::ExposuresQuery + agentspec_types::session::obligations::SessionsQuery + agentspec_types::session::obligations::ToolCallsQuery,
{
    /// Accepts `agentspec.session.CompactContext`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn compact_context(&mut self, input: agentspec_types::session::CompactContext) -> Result<agentspec_types::session::CompactContextOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.compact_context(input)?;
        match &outcome {
            agentspec_types::session::CompactContextOutcome::Compacted { context_compacted, .. } => {
                self.outbox.push(PublishedEvent::ContextCompacted(context_compacted.clone()));
            }
            agentspec_types::session::CompactContextOutcome::NoSuchAgent { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.CompleteToolCall`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn complete_tool_call(&mut self, input: agentspec_types::session::CompleteToolCall) -> Result<agentspec_types::session::CompleteToolCallOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.complete_tool_call(input)?;
        match &outcome {
            agentspec_types::session::CompleteToolCallOutcome::Succeeded { tool_call_completed, .. } => {
                self.outbox.push(PublishedEvent::ToolCallCompleted(tool_call_completed.clone()));
            }
            agentspec_types::session::CompleteToolCallOutcome::Failed { tool_call_completed, .. } => {
                self.outbox.push(PublishedEvent::ToolCallCompleted(tool_call_completed.clone()));
            }
            agentspec_types::session::CompleteToolCallOutcome::WrongState { .. } => {}
            agentspec_types::session::CompleteToolCallOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.DecideToolCall`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn decide_tool_call(&mut self, input: agentspec_types::session::DecideToolCall) -> Result<agentspec_types::session::DecideToolCallOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.decide_tool_call(input)?;
        match &outcome {
            agentspec_types::session::DecideToolCallOutcome::Accepted { tool_call_decided, .. } => {
                self.outbox.push(PublishedEvent::ToolCallDecided(tool_call_decided.clone()));
            }
            agentspec_types::session::DecideToolCallOutcome::Refused { tool_call_decided, .. } => {
                self.outbox.push(PublishedEvent::ToolCallDecided(tool_call_decided.clone()));
            }
            agentspec_types::session::DecideToolCallOutcome::WrongState { .. } => {}
            agentspec_types::session::DecideToolCallOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.EndAgent`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn end_agent(&mut self, input: agentspec_types::session::EndAgent) -> Result<agentspec_types::session::EndAgentOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.end_agent(input)?;
        match &outcome {
            agentspec_types::session::EndAgentOutcome::Ended { agent_ended, .. } => {
                self.outbox.push(PublishedEvent::AgentEnded(agent_ended.clone()));
            }
            agentspec_types::session::EndAgentOutcome::WrongState { .. } => {}
            agentspec_types::session::EndAgentOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.EndSession`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn end_session(&mut self, input: agentspec_types::session::EndSession) -> Result<agentspec_types::session::EndSessionOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.end_session(input)?;
        match &outcome {
            agentspec_types::session::EndSessionOutcome::Ended { session_ended, .. } => {
                self.outbox.push(PublishedEvent::SessionEnded(session_ended.clone()));
            }
            agentspec_types::session::EndSessionOutcome::WrongState { .. } => {}
            agentspec_types::session::EndSessionOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.EndTurn`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn end_turn(&mut self, input: agentspec_types::session::EndTurn) -> Result<agentspec_types::session::EndTurnOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.end_turn(input)?;
        match &outcome {
            agentspec_types::session::EndTurnOutcome::Ended { turn_ended, .. } => {
                self.outbox.push(PublishedEvent::TurnEnded(turn_ended.clone()));
            }
            agentspec_types::session::EndTurnOutcome::WrongState { .. } => {}
            agentspec_types::session::EndTurnOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.OpenSession`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn open_session(&mut self, input: agentspec_types::session::OpenSession) -> Result<agentspec_types::session::OpenSessionOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.open_session(input)?;
        match &outcome {
            agentspec_types::session::OpenSessionOutcome::Opened { session_opened, .. } => {
                self.outbox.push(PublishedEvent::SessionOpened(session_opened.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.RecordExposure`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_exposure(&mut self, input: agentspec_types::session::RecordExposure) -> Result<agentspec_types::session::RecordExposureOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_exposure(input)?;
        match &outcome {
            agentspec_types::session::RecordExposureOutcome::Recorded { exposure_detected, .. } => {
                self.outbox.push(PublishedEvent::ExposureDetected(exposure_detected.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.RecordModelTurn`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_model_turn(&mut self, input: agentspec_types::session::RecordModelTurn) -> Result<agentspec_types::session::RecordModelTurnOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_model_turn(input)?;
        match &outcome {
            agentspec_types::session::RecordModelTurnOutcome::Recorded { model_turn_recorded, .. } => {
                self.outbox.push(PublishedEvent::ModelTurnRecorded(model_turn_recorded.clone()));
            }
            agentspec_types::session::RecordModelTurnOutcome::NoSuchAgent { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.RequestToolCall`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn request_tool_call(&mut self, input: agentspec_types::session::RequestToolCall) -> Result<agentspec_types::session::RequestToolCallOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.request_tool_call(input)?;
        match &outcome {
            agentspec_types::session::RequestToolCallOutcome::Requested { tool_call_requested, .. } => {
                self.outbox.push(PublishedEvent::ToolCallRequested(tool_call_requested.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.StartAgent`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn start_agent(&mut self, input: agentspec_types::session::StartAgent) -> Result<agentspec_types::session::StartAgentOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.start_agent(input)?;
        match &outcome {
            agentspec_types::session::StartAgentOutcome::Started { agent_started, .. } => {
                self.outbox.push(PublishedEvent::AgentStarted(agent_started.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `agentspec.session.SubmitPrompt`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn submit_prompt(&mut self, input: agentspec_types::session::SubmitPrompt) -> Result<agentspec_types::session::SubmitPromptOutcome, agentspec_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.submit_prompt(input)?;
        match &outcome {
            agentspec_types::session::SubmitPromptOutcome::Submitted { prompt_submitted, .. } => {
                self.outbox.push(PublishedEvent::PromptSubmitted(prompt_submitted.clone()));
            }
            agentspec_types::session::SubmitPromptOutcome::WrongState { .. } => {}
            agentspec_types::session::SubmitPromptOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Serves `agentspec.session.Agents` at `read_your_writes` consistency, from the owed projection.
    pub fn agents(&self) -> Result<Vec<agentspec_types::session::Agents>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.agents()
    }

    /// Serves `agentspec.session.Exposures` at `read_your_writes` consistency, from the owed projection.
    pub fn exposures(&self) -> Result<Vec<agentspec_types::session::Exposures>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.exposures()
    }

    /// Serves `agentspec.session.Sessions` at `read_your_writes` consistency, from the owed projection.
    pub fn sessions(&self) -> Result<Vec<agentspec_types::session::Sessions>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.sessions()
    }

    /// Serves `agentspec.session.ToolCalls` at `read_your_writes` consistency, from the owed projection.
    pub fn tool_calls(&self) -> Result<Vec<agentspec_types::session::ToolCalls>, agentspec_types::obligation::UnmetObligation> {
        self.behaviors.tool_calls()
    }
}
