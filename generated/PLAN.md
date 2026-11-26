<!--
  generated from sessionspec v1
  contract digest 7eaa4202b2be444349f069c24e3e8839a7d36b41fbc39fdf08c4042538b65d40
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — sessionspec v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

178 capabilities: **175 generated**, **3 obligations**, **0 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `sessionspec.cli.AgentScope` |
| domain type | `sessionspec.cli.AnalyzeInput` |
| domain type | `sessionspec.cli.AnalyzeResult` |
| domain type | `sessionspec.cli.AutonomyFigures` |
| domain type | `sessionspec.cli.BustCause` |
| domain type | `sessionspec.cli.BustFigures` |
| domain type | `sessionspec.cli.CacheFigures` |
| domain type | `sessionspec.cli.Certainty` |
| domain type | `sessionspec.cli.CheckFigures` |
| domain type | `sessionspec.cli.CheckVerdict` |
| domain type | `sessionspec.cli.ConcurrencyFigures` |
| domain type | `sessionspec.cli.Dashboard` |
| domain type | `sessionspec.cli.DecisionFigures` |
| domain type | `sessionspec.cli.ExportInput` |
| domain type | `sessionspec.cli.ExportResult` |
| domain type | `sessionspec.cli.ExposureFigures` |
| domain type | `sessionspec.cli.KpiInput` |
| domain type | `sessionspec.cli.KpiReport` |
| domain type | `sessionspec.cli.LiftInput` |
| domain type | `sessionspec.cli.LiftResult` |
| domain type | `sessionspec.cli.LiftedCommand` |
| domain type | `sessionspec.cli.MarkKind` |
| domain type | `sessionspec.cli.MineInput` |
| domain type | `sessionspec.cli.MineReport` |
| domain type | `sessionspec.cli.ModelFigures` |
| domain type | `sessionspec.cli.Pattern` |
| domain type | `sessionspec.cli.Procedure` |
| domain type | `sessionspec.cli.ProcedureInput` |
| domain type | `sessionspec.cli.ProcedureParameter` |
| domain type | `sessionspec.cli.ProcedureStep` |
| domain type | `sessionspec.cli.PromptFigures` |
| domain type | `sessionspec.cli.QueryInput` |
| domain type | `sessionspec.cli.QueryReport` |
| domain type | `sessionspec.cli.QueryRow` |
| domain type | `sessionspec.cli.Refusal` |
| domain type | `sessionspec.cli.ServeInput` |
| domain type | `sessionspec.cli.ServeResult` |
| domain type | `sessionspec.cli.SessionSummary` |
| domain type | `sessionspec.cli.StepKind` |
| domain type | `sessionspec.cli.SyncInput` |
| domain type | `sessionspec.cli.SyncResult` |
| domain type | `sessionspec.cli.TimelineMark` |
| domain type | `sessionspec.cli.ToolFigures` |
| domain type | `sessionspec.session.Agent.State` |
| domain type | `sessionspec.session.AgentId` |
| domain type | `sessionspec.session.AgentRole` |
| domain type | `sessionspec.session.Attribution` |
| domain type | `sessionspec.session.Content` |
| domain type | `sessionspec.session.Decider` |
| domain type | `sessionspec.session.Decision` |
| domain type | `sessionspec.session.Exposure.State` |
| domain type | `sessionspec.session.ExposureChannel` |
| domain type | `sessionspec.session.ExposureClass` |
| domain type | `sessionspec.session.ExposureId` |
| domain type | `sessionspec.session.Harness` |
| domain type | `sessionspec.session.IncompleteReason` |
| domain type | `sessionspec.session.PromptAuthor` |
| domain type | `sessionspec.session.PromptClass` |
| domain type | `sessionspec.session.Session.State` |
| domain type | `sessionspec.session.SessionId` |
| domain type | `sessionspec.session.Source` |
| domain type | `sessionspec.session.ToolCall.State` |
| domain type | `sessionspec.session.ToolCallId` |
| domain type | `sessionspec.session.ToolCategory` |
| domain type | `sessionspec.session.Usage` |
| domain type | `sessionspec.session.Verdict` |
| domain type | `sessionspec.view.Agent` |
| domain type | `sessionspec.view.AgentListing.State` |
| domain type | `sessionspec.view.BucketId` |
| domain type | `sessionspec.view.CheckId` |
| domain type | `sessionspec.view.CountId` |
| domain type | `sessionspec.view.Document` |
| domain type | `sessionspec.view.Event` |
| domain type | `sessionspec.view.EventCount.State` |
| domain type | `sessionspec.view.EventId` |
| domain type | `sessionspec.view.EventKind` |
| domain type | `sessionspec.view.EventRow.State` |
| domain type | `sessionspec.view.FigureBucket.State` |
| domain type | `sessionspec.view.SessionCheck.State` |
| domain type | `sessionspec.view.SessionListing.State` |
| domain type | `sessionspec.view.SlotId` |
| domain type | `sessionspec.view.TimelineSlot.State` |
| domain type | `sessionspec.view.Update` |
| entity lifecycle | `sessionspec.session.Agent` |
| entity lifecycle | `sessionspec.session.Exposure` |
| entity lifecycle | `sessionspec.session.Session` |
| entity lifecycle | `sessionspec.session.ToolCall` |
| entity lifecycle | `sessionspec.view.AgentListing` |
| entity lifecycle | `sessionspec.view.EventCount` |
| entity lifecycle | `sessionspec.view.EventRow` |
| entity lifecycle | `sessionspec.view.FigureBucket` |
| entity lifecycle | `sessionspec.view.SessionCheck` |
| entity lifecycle | `sessionspec.view.SessionListing` |
| entity lifecycle | `sessionspec.view.TimelineSlot` |
| command contract | `sessionspec.session.CompactContext` |
| command behaviour | `sessionspec.session.CompactContext` |
| command contract | `sessionspec.session.CompleteToolCall` |
| command behaviour | `sessionspec.session.CompleteToolCall` |
| command contract | `sessionspec.session.DecideToolCall` |
| command behaviour | `sessionspec.session.DecideToolCall` |
| command contract | `sessionspec.session.EndAgent` |
| command behaviour | `sessionspec.session.EndAgent` |
| command contract | `sessionspec.session.EndSession` |
| command behaviour | `sessionspec.session.EndSession` |
| command contract | `sessionspec.session.EndTurn` |
| command behaviour | `sessionspec.session.EndTurn` |
| command contract | `sessionspec.session.OpenSession` |
| command behaviour | `sessionspec.session.OpenSession` |
| command contract | `sessionspec.session.RecordExposure` |
| command behaviour | `sessionspec.session.RecordExposure` |
| command contract | `sessionspec.session.RecordModelTurn` |
| command behaviour | `sessionspec.session.RecordModelTurn` |
| command contract | `sessionspec.session.RequestToolCall` |
| command behaviour | `sessionspec.session.RequestToolCall` |
| command contract | `sessionspec.session.StartAgent` |
| command behaviour | `sessionspec.session.StartAgent` |
| command contract | `sessionspec.session.SubmitPrompt` |
| command behaviour | `sessionspec.session.SubmitPrompt` |
| event type | `sessionspec.session.AgentEnded` |
| event type | `sessionspec.session.AgentStarted` |
| event type | `sessionspec.session.ContextCompacted` |
| event type | `sessionspec.session.ExposureDetected` |
| event type | `sessionspec.session.ModelTurnRecorded` |
| event type | `sessionspec.session.PromptSubmitted` |
| event type | `sessionspec.session.SessionEnded` |
| event type | `sessionspec.session.SessionOpened` |
| event type | `sessionspec.session.ToolCallCompleted` |
| event type | `sessionspec.session.ToolCallDecided` |
| event type | `sessionspec.session.ToolCallRequested` |
| event type | `sessionspec.session.TurnEnded` |
| event type | `sessionspec.view.AgentUpdated` |
| event type | `sessionspec.view.EventAppended` |
| event type | `sessionspec.view.SessionUpdated` |
| error type | `sessionspec.session.AgentStateConflict` |
| error type | `sessionspec.session.SessionStateConflict` |
| error type | `sessionspec.session.ToolCallStateConflict` |
| error type | `sessionspec.session.UnknownAgent` |
| view type | `sessionspec.session.Agents` |
| view query | `sessionspec.session.Agents` |
| view type | `sessionspec.session.Exposures` |
| view query | `sessionspec.session.Exposures` |
| view type | `sessionspec.session.Sessions` |
| view query | `sessionspec.session.Sessions` |
| view type | `sessionspec.session.ToolCalls` |
| view query | `sessionspec.session.ToolCalls` |
| view type | `sessionspec.view.AgentTurns` |
| view query | `sessionspec.view.AgentTurns` |
| view type | `sessionspec.view.OverviewFigures` |
| view query | `sessionspec.view.OverviewFigures` |
| view type | `sessionspec.view.SessionAgents` |
| view query | `sessionspec.view.SessionAgents` |
| view type | `sessionspec.view.SessionById` |
| view query | `sessionspec.view.SessionById` |
| view type | `sessionspec.view.SessionChecks` |
| view query | `sessionspec.view.SessionChecks` |
| view type | `sessionspec.view.SessionEventCounts` |
| view type | `sessionspec.view.SessionEvents` |
| view type | `sessionspec.view.SessionList` |
| view type | `sessionspec.view.SessionTimeline` |
| view query | `sessionspec.view.SessionTimeline` |
| view type | `sessionspec.view.SessionTurns` |
| view query | `sessionspec.view.SessionTurns` |
| actor grants | `sessionspec.session.AgentLoop` |
| actor grants | `sessionspec.session.Delegate` |
| actor grants | `sessionspec.session.HarnessRuntime` |
| actor grants | `sessionspec.session.Human` |
| actor grants | `sessionspec.session.PeerSession` |
| actor grants | `sessionspec.session.PermissionAuthority` |
| actor grants | `sessionspec.session.Redactor` |
| actor grants | `sessionspec.session.ToolProvider` |
| actor grants | `sessionspec.view.Viewer` |
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
| view query | `sessionspec.view.SessionEventCounts` | kept an obligation by a comparison with the optional view parameter `agent`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:` | a query answering `sessionspec.view.SessionEventCounts` with rows projected from `sessionspec.view.EventCount` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent))` |
| view query | `sessionspec.view.SessionEvents` | kept an obligation by a comparison with the optional view parameter `agent`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:` | a query answering `sessionspec.view.SessionEvents` with rows projected from `sessionspec.view.EventRow` at `eventual` consistency, containing instances where `(session_id == param.session_id and (not (defined(param.agent)) or agent == param.agent) and (not (defined(param.kind)) or kind == param.kind) and (not (defined(param.notable)) or notable == param.notable) and (not (defined(param.before)) or ord < param.before))` |
| view query | `sessionspec.view.SessionList` | kept an obligation by a comparison with the optional view parameter `since`, which the interpreted target refuses a read that leaves out, where a generated query would hide rows, in `filter:` | a query answering `sessionspec.view.SessionList` with rows projected from `sessionspec.view.SessionListing` at `eventual` consistency, containing instances where `((not (defined(param.since)) or last_at >= param.since as timestamp) and (not (defined(param.search)) or names contains {param: search}) and (not (defined(param.limit)) or rank <= param.limit) and (not (defined(param.offset)) or rank > param.offset))` |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
