<!--
  generated from agentspec v1
  contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — agentspec v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

178 capabilities: **175 generated**, **3 obligations**, **0 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `agentspec.cli.AgentScope` |
| domain type | `agentspec.cli.AnalyzeInput` |
| domain type | `agentspec.cli.AnalyzeResult` |
| domain type | `agentspec.cli.AutonomyFigures` |
| domain type | `agentspec.cli.BustCause` |
| domain type | `agentspec.cli.BustFigures` |
| domain type | `agentspec.cli.CacheFigures` |
| domain type | `agentspec.cli.Certainty` |
| domain type | `agentspec.cli.CheckFigures` |
| domain type | `agentspec.cli.CheckVerdict` |
| domain type | `agentspec.cli.ConcurrencyFigures` |
| domain type | `agentspec.cli.Dashboard` |
| domain type | `agentspec.cli.DecisionFigures` |
| domain type | `agentspec.cli.ExportInput` |
| domain type | `agentspec.cli.ExportResult` |
| domain type | `agentspec.cli.ExposureFigures` |
| domain type | `agentspec.cli.KpiInput` |
| domain type | `agentspec.cli.KpiReport` |
| domain type | `agentspec.cli.LiftInput` |
| domain type | `agentspec.cli.LiftResult` |
| domain type | `agentspec.cli.LiftedCommand` |
| domain type | `agentspec.cli.MarkKind` |
| domain type | `agentspec.cli.MineInput` |
| domain type | `agentspec.cli.MineReport` |
| domain type | `agentspec.cli.ModelFigures` |
| domain type | `agentspec.cli.Pattern` |
| domain type | `agentspec.cli.Procedure` |
| domain type | `agentspec.cli.ProcedureInput` |
| domain type | `agentspec.cli.ProcedureParameter` |
| domain type | `agentspec.cli.ProcedureStep` |
| domain type | `agentspec.cli.PromptFigures` |
| domain type | `agentspec.cli.QueryInput` |
| domain type | `agentspec.cli.QueryReport` |
| domain type | `agentspec.cli.QueryRow` |
| domain type | `agentspec.cli.Refusal` |
| domain type | `agentspec.cli.ServeInput` |
| domain type | `agentspec.cli.ServeResult` |
| domain type | `agentspec.cli.SessionSummary` |
| domain type | `agentspec.cli.StepKind` |
| domain type | `agentspec.cli.SyncInput` |
| domain type | `agentspec.cli.SyncResult` |
| domain type | `agentspec.cli.TimelineMark` |
| domain type | `agentspec.cli.ToolFigures` |
| domain type | `agentspec.session.Agent.State` |
| domain type | `agentspec.session.AgentId` |
| domain type | `agentspec.session.AgentRole` |
| domain type | `agentspec.session.Attribution` |
| domain type | `agentspec.session.Content` |
| domain type | `agentspec.session.Decider` |
| domain type | `agentspec.session.Decision` |
| domain type | `agentspec.session.Exposure.State` |
| domain type | `agentspec.session.ExposureChannel` |
| domain type | `agentspec.session.ExposureClass` |
| domain type | `agentspec.session.ExposureId` |
| domain type | `agentspec.session.Harness` |
| domain type | `agentspec.session.IncompleteReason` |
| domain type | `agentspec.session.PromptAuthor` |
| domain type | `agentspec.session.PromptClass` |
| domain type | `agentspec.session.Session.State` |
| domain type | `agentspec.session.SessionId` |
| domain type | `agentspec.session.Source` |
| domain type | `agentspec.session.ToolCall.State` |
| domain type | `agentspec.session.ToolCallId` |
| domain type | `agentspec.session.ToolCategory` |
| domain type | `agentspec.session.Usage` |
| domain type | `agentspec.session.Verdict` |
| domain type | `agentspec.view.Agent` |
| domain type | `agentspec.view.AgentListing.State` |
| domain type | `agentspec.view.BucketId` |
| domain type | `agentspec.view.CheckId` |
| domain type | `agentspec.view.CountId` |
| domain type | `agentspec.view.Document` |
| domain type | `agentspec.view.Event` |
| domain type | `agentspec.view.EventCount.State` |
| domain type | `agentspec.view.EventId` |
| domain type | `agentspec.view.EventKind` |
| domain type | `agentspec.view.EventRow.State` |
| domain type | `agentspec.view.FigureBucket.State` |
| domain type | `agentspec.view.SessionCheck.State` |
| domain type | `agentspec.view.SessionListing.State` |
| domain type | `agentspec.view.SlotId` |
| domain type | `agentspec.view.TimelineSlot.State` |
| domain type | `agentspec.view.Update` |
| entity lifecycle | `agentspec.session.Agent` |
| entity lifecycle | `agentspec.session.Exposure` |
| entity lifecycle | `agentspec.session.Session` |
| entity lifecycle | `agentspec.session.ToolCall` |
| entity lifecycle | `agentspec.view.AgentListing` |
| entity lifecycle | `agentspec.view.EventCount` |
| entity lifecycle | `agentspec.view.EventRow` |
| entity lifecycle | `agentspec.view.FigureBucket` |
| entity lifecycle | `agentspec.view.SessionCheck` |
| entity lifecycle | `agentspec.view.SessionListing` |
| entity lifecycle | `agentspec.view.TimelineSlot` |
| command contract | `agentspec.session.CompactContext` |
| command behaviour | `agentspec.session.CompactContext` |
| command contract | `agentspec.session.CompleteToolCall` |
| command behaviour | `agentspec.session.CompleteToolCall` |
| command contract | `agentspec.session.DecideToolCall` |
| command behaviour | `agentspec.session.DecideToolCall` |
| command contract | `agentspec.session.EndAgent` |
| command behaviour | `agentspec.session.EndAgent` |
| command contract | `agentspec.session.EndSession` |
| command behaviour | `agentspec.session.EndSession` |
| command contract | `agentspec.session.EndTurn` |
| command behaviour | `agentspec.session.EndTurn` |
| command contract | `agentspec.session.OpenSession` |
| command behaviour | `agentspec.session.OpenSession` |
| command contract | `agentspec.session.RecordExposure` |
| command behaviour | `agentspec.session.RecordExposure` |
| command contract | `agentspec.session.RecordModelTurn` |
| command behaviour | `agentspec.session.RecordModelTurn` |
| command contract | `agentspec.session.RequestToolCall` |
| command behaviour | `agentspec.session.RequestToolCall` |
| command contract | `agentspec.session.StartAgent` |
| command behaviour | `agentspec.session.StartAgent` |
| command contract | `agentspec.session.SubmitPrompt` |
| command behaviour | `agentspec.session.SubmitPrompt` |
| event type | `agentspec.session.AgentEnded` |
| event type | `agentspec.session.AgentStarted` |
| event type | `agentspec.session.ContextCompacted` |
| event type | `agentspec.session.ExposureDetected` |
| event type | `agentspec.session.ModelTurnRecorded` |
| event type | `agentspec.session.PromptSubmitted` |
| event type | `agentspec.session.SessionEnded` |
| event type | `agentspec.session.SessionOpened` |
| event type | `agentspec.session.ToolCallCompleted` |
| event type | `agentspec.session.ToolCallDecided` |
| event type | `agentspec.session.ToolCallRequested` |
| event type | `agentspec.session.TurnEnded` |
| event type | `agentspec.view.AgentUpdated` |
| event type | `agentspec.view.EventAppended` |
| event type | `agentspec.view.SessionUpdated` |
| error type | `agentspec.session.AgentStateConflict` |
| error type | `agentspec.session.SessionStateConflict` |
| error type | `agentspec.session.ToolCallStateConflict` |
| error type | `agentspec.session.UnknownAgent` |
| view type | `agentspec.session.Agents` |
| view query | `agentspec.session.Agents` |
| view type | `agentspec.session.Exposures` |
| view query | `agentspec.session.Exposures` |
| view type | `agentspec.session.Sessions` |
| view query | `agentspec.session.Sessions` |
| view type | `agentspec.session.ToolCalls` |
| view query | `agentspec.session.ToolCalls` |
| view type | `agentspec.view.AgentTurns` |
| view query | `agentspec.view.AgentTurns` |
| view type | `agentspec.view.OverviewFigures` |
| view query | `agentspec.view.OverviewFigures` |
| view type | `agentspec.view.SessionAgents` |
| view query | `agentspec.view.SessionAgents` |
| view type | `agentspec.view.SessionById` |
| view query | `agentspec.view.SessionById` |
| view type | `agentspec.view.SessionChecks` |
| view query | `agentspec.view.SessionChecks` |
| view type | `agentspec.view.SessionEventCounts` |
| view type | `agentspec.view.SessionEvents` |
| view type | `agentspec.view.SessionList` |
| view type | `agentspec.view.SessionTimeline` |
| view query | `agentspec.view.SessionTimeline` |
| view type | `agentspec.view.SessionTurns` |
| view query | `agentspec.view.SessionTurns` |
| actor grants | `agentspec.session.AgentLoop` |
| actor grants | `agentspec.session.Delegate` |
| actor grants | `agentspec.session.HarnessRuntime` |
| actor grants | `agentspec.session.Human` |
| actor grants | `agentspec.session.PeerSession` |
| actor grants | `agentspec.session.PermissionAuthority` |
| actor grants | `agentspec.session.Redactor` |
| actor grants | `agentspec.session.ToolProvider` |
| actor grants | `agentspec.view.Viewer` |
| component port | `session-ir` |
| component transport | `session-ir` |
| component port | `session-view` |
| component transport | `session-view` |

## Ports — yours to provide

What the specification fully determines is generated; what it cannot determine is an obligation. A generated command behaviour or view query reads and writes through the ports below, and they are yours to provide: synthesis generates each port's contract and never an implementation of one, so where instances live stays your decision.

| port | what it answers |
| --- | --- |
| storage | one per entity a generated behaviour or query reads or writes: the instance stored under an identity; storing, replacing and removing one; and every stored instance, in the order the store keeps them |
| context | where a generated behaviour asks it: the caller's attributes, every identity and value the specification says the implementation assigns, and whether each `external:` branch is taken |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| view query | `agentspec.view.SessionEventCounts` | kept an obligation by a comparison with the optional view parameter `agent`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:` | a query answering `agentspec.view.SessionEventCounts` with rows projected from `agentspec.view.EventCount` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent))` |
| view query | `agentspec.view.SessionEvents` | kept an obligation by a comparison with the optional view parameter `agent`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:` | a query answering `agentspec.view.SessionEvents` with rows projected from `agentspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent) and (not (defined(param.kind)) or kind == param.kind) and (not (defined(param.notable)) or notable == param.notable) and (not (defined(param.before)) or ord < param.before))` |
| view query | `agentspec.view.SessionList` | kept an obligation by a comparison with the optional view parameter `since`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:` | a query answering `agentspec.view.SessionList` with rows projected from `agentspec.view.SessionListing` at `eventual` consistency, containing instances where `((not (defined(param.since)) or last_at >= param.since as timestamp) and (not (defined(param.search)) or names contains {param: search}) and (not (defined(param.limit)) or rank <= param.limit) and (not (defined(param.offset)) or rank > param.offset))` |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
