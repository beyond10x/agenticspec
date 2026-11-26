// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! cli — `agentspec.cli`.
//!
//! The typed inputs, results and refusals of the `agentspec` command line, presented by the ess-cli/2 binding in `spec/cli.yaml`. It declares no entity. The KPI report is typed here, so every figure the report carries is a declared field.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// AgentScope — `agentspec.cli.AgentScope`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentScope {
    /// `main`.
    Main,
    /// `sub`.
    Sub,
}

/// AnalyzeInput — `agentspec.cli.AnalyzeInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzeInput {
    /// `root` — `Optional<String>`.
    pub root: Option<String>,
    /// `since_hours` — `Optional<Integer>`.
    pub since_hours: Option<i64>,
    /// `out` — `Optional<String>`.
    pub out: Option<String>,
}

/// AnalyzeResult — `agentspec.cli.AnalyzeResult`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzeResult {
    /// `path` — `String`.
    pub path: String,
    /// `sessions` — `Integer`.
    pub sessions: i64,
}

/// AutonomyFigures — `agentspec.cli.AutonomyFigures`.
///
/// Prompts and decisions of main agents. Working is time in which at least one agent of the session was Working (gaps up to 10 minutes). Waiting on the human is time in which no agent was Working, from a main agent's turn end to its next Human or Delegate prompt; over one hour it counts as away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutonomyFigures {
    /// `working_seconds` — `Integer`.
    pub working_seconds: i64,
    /// `waiting_on_human_seconds` — `Integer`.
    pub waiting_on_human_seconds: i64,
    /// `away_seconds` — `Integer`.
    pub away_seconds: i64,
    /// `prompts` — `List<agentspec.cli.PromptFigures>`.
    pub prompts: Vec<PromptFigures>,
    /// `decisions` — `List<agentspec.cli.DecisionFigures>`.
    pub decisions: Vec<DecisionFigures>,
}

/// BustCause — `agentspec.cli.BustCause`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BustCause {
    /// `ttl_expired`.
    TtlExpired,
    /// `model_changed`.
    ModelChanged,
    /// `compacted`.
    Compacted,
    /// `unexplained`.
    Unexplained,
}

/// BustFigures — `agentspec.cli.BustFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BustFigures {
    /// `scope` — `agentspec.cli.AgentScope`.
    pub scope: AgentScope,
    /// `cause` — `agentspec.cli.BustCause`.
    pub cause: BustCause,
    /// `count` — `Integer`.
    pub count: i64,
    /// `tokens_written_again` — `Integer`.
    pub tokens_written_again: i64,
}

/// CacheFigures — `agentspec.cli.CacheFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheFigures {
    /// `scope` — `agentspec.cli.AgentScope`.
    pub scope: AgentScope,
    /// `read_tokens` — `Integer`.
    pub read_tokens: i64,
    /// `write_tokens` — `Integer`.
    pub write_tokens: i64,
    /// `uncached_tokens` — `Integer`.
    pub uncached_tokens: i64,
    /// `hit_permille` — `Integer`.
    pub hit_permille: i64,
}

/// Certainty — `agentspec.cli.Certainty`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Certainty {
    /// `definite`.
    Definite,
    /// `possible`.
    Possible,
}

/// CheckFigures — `agentspec.cli.CheckFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckFigures {
    /// `check` — `String`.
    pub check: String,
    /// `about` — `String`.
    pub about: String,
    /// `severity` — `String`.
    pub severity: String,
    /// `ok` — `Integer`.
    pub ok: i64,
    /// `gap` — `Integer`.
    pub gap: i64,
    /// `unk` — `Integer`.
    pub unk: i64,
}

/// CheckVerdict — `agentspec.cli.CheckVerdict`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckVerdict {
    /// `check` — `String`.
    pub check: String,
    /// `session_id` — `String`.
    pub session_id: String,
    /// `verdict` — `agentspec.session.Verdict`.
    pub verdict: crate::session::Verdict,
    /// `rows` — `Integer`.
    pub rows: i64,
}

/// ConcurrencyFigures — `agentspec.cli.ConcurrencyFigures`.
///
/// Agents active in the same minute; active means two of its records at most 5 minutes apart around the minute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrencyFigures {
    /// `scope` — `agentspec.cli.AgentScope`.
    pub scope: AgentScope,
    /// `p50` — `Integer`.
    pub p50: i64,
    /// `p90` — `Integer`.
    pub p90: i64,
    /// `max` — `Integer`.
    pub max: i64,
}

/// Dashboard — `agentspec.cli.Dashboard`.
///
/// What the dashboard file embeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dashboard {
    /// `generated_at` — `String`.
    pub generated_at: String,
    /// `overall` — `agentspec.cli.KpiReport`.
    pub overall: KpiReport,
    /// `sessions` — `List<agentspec.cli.SessionSummary>`.
    pub sessions: Vec<SessionSummary>,
}

/// DecisionFigures — `agentspec.cli.DecisionFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionFigures {
    /// `decider` — `agentspec.session.Decider`.
    pub decider: crate::session::Decider,
    /// `decision` — `agentspec.session.Decision`.
    pub decision: crate::session::Decision,
    /// `count` — `Integer`.
    pub count: i64,
}

/// ExportInput — `agentspec.cli.ExportInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportInput {
    /// `procedure` — `String`.
    pub procedure: String,
    /// `out` — `String`.
    pub out: String,
}

/// ExportResult — `agentspec.cli.ExportResult`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportResult {
    /// `protocol` — `String`.
    pub protocol: String,
    /// `bindings` — `String`.
    pub bindings: String,
    /// `validated` — `Boolean`.
    pub validated: bool,
    /// `detail` — `String`.
    pub detail: String,
}

/// ExposureFigures — `agentspec.cli.ExposureFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExposureFigures {
    /// `channel` — `agentspec.session.ExposureChannel`.
    pub channel: crate::session::ExposureChannel,
    /// `class` — `agentspec.session.ExposureClass`.
    pub class: crate::session::ExposureClass,
    /// `kind` — `String`.
    pub kind: String,
    /// `verdict` — `agentspec.session.Verdict`.
    pub verdict: crate::session::Verdict,
    /// `count` — `Integer`.
    pub count: i64,
    /// `sessions` — `Integer`.
    pub sessions: i64,
}

/// KpiInput — `agentspec.cli.KpiInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KpiInput {
    /// `root` — `Optional<String>`.
    pub root: Option<String>,
    /// `since_hours` — `Optional<Integer>`.
    pub since_hours: Option<i64>,
}

/// KpiReport — `agentspec.cli.KpiReport`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KpiReport {
    /// `window_from` — `String`.
    pub window_from: String,
    /// `window_to` — `String`.
    pub window_to: String,
    /// `transcripts` — `Integer`.
    pub transcripts: i64,
    /// `sessions` — `Integer`.
    pub sessions: i64,
    /// `sub_agents` — `Integer`.
    pub sub_agents: i64,
    /// `refused_commands` — `Integer`.
    pub refused_commands: i64,
    /// `cost_microdollars` — `Integer`.
    pub cost_microdollars: i64,
    /// `unpriced_messages` — `Integer`.
    pub unpriced_messages: i64,
    /// `models` — `List<agentspec.cli.ModelFigures>`.
    pub models: Vec<ModelFigures>,
    /// `cache` — `List<agentspec.cli.CacheFigures>`.
    pub cache: Vec<CacheFigures>,
    /// `busts` — `List<agentspec.cli.BustFigures>`.
    pub busts: Vec<BustFigures>,
    /// `concurrency` — `List<agentspec.cli.ConcurrencyFigures>`.
    pub concurrency: Vec<ConcurrencyFigures>,
    /// `autonomy` — `agentspec.cli.AutonomyFigures`.
    pub autonomy: AutonomyFigures,
    /// `exposures` — `List<agentspec.cli.ExposureFigures>`.
    pub exposures: Vec<ExposureFigures>,
}

/// LiftInput — `agentspec.cli.LiftInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiftInput {
    /// `transcript` — `String`.
    pub transcript: String,
}

/// LiftResult — `agentspec.cli.LiftResult`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiftResult {
    /// `session_id` — `String`.
    pub session_id: String,
    /// `agents` — `Integer`.
    pub agents: i64,
    /// `commands` — `List<agentspec.cli.LiftedCommand>`.
    pub commands: Vec<LiftedCommand>,
    /// `refused` — `Integer`.
    pub refused: i64,
}

/// LiftedCommand — `agentspec.cli.LiftedCommand`.
///
/// One command of the lifted session, as replayed, with the outcome the session IR answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiftedCommand {
    /// `seq` — `Integer`.
    pub seq: i64,
    /// `at` — `String`.
    pub at: String,
    /// `actor` — `String`.
    pub actor: String,
    /// `command` — `String`.
    pub command: String,
    /// `outcome` — `String`.
    pub outcome: String,
    /// `input` — `Json`.
    pub input: crate::json::Value,
}

/// MarkKind — `agentspec.cli.MarkKind`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// `turn`.
    Turn,
    /// `human_prompt`.
    HumanPrompt,
    /// `delegate_prompt`.
    DelegatePrompt,
    /// `peer_prompt`.
    PeerPrompt,
    /// `harness_prompt`.
    HarnessPrompt,
    /// `compaction`.
    Compaction,
    /// `cache_bust`.
    CacheBust,
    /// `refusal`.
    Refusal,
}

/// MineInput — `agentspec.cli.MineInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineInput {
    /// `root` — `Optional<String>`.
    pub root: Option<String>,
    /// `since_hours` — `Optional<Integer>`.
    pub since_hours: Option<i64>,
    /// `level` — `Optional<String>`.
    pub level: Option<String>,
    /// `min_sessions` — `Optional<Integer>`.
    pub min_sessions: Option<i64>,
    /// `min_projects` — `Optional<Integer>`.
    pub min_projects: Option<i64>,
    /// `top` — `Optional<Integer>`.
    pub top: Option<i64>,
}

/// MineReport — `agentspec.cli.MineReport`.
///
/// The report of `agentspec mine`, format `agentspec-patterns/1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineReport {
    /// `format` — `String`.
    pub format: String,
    /// `level` — `String`.
    pub level: String,
    /// `sessions` — `Integer`.
    pub sessions: i64,
    /// `turns` — `Integer`.
    pub turns: i64,
    /// `patterns` — `List<agentspec.cli.Pattern>`.
    pub patterns: Vec<Pattern>,
}

/// ModelFigures — `agentspec.cli.ModelFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFigures {
    /// `scope` — `agentspec.cli.AgentScope`.
    pub scope: AgentScope,
    /// `model` — `String`.
    pub model: String,
    /// `messages` — `Integer`.
    pub messages: i64,
    /// `output_tokens` — `Integer`.
    pub output_tokens: i64,
    /// `context_p50` — `Integer`.
    pub context_p50: i64,
    /// `context_p90` — `Integer`.
    pub context_p90: i64,
    /// `context_max` — `Integer`.
    pub context_max: i64,
    /// `cost_microdollars` — `Integer`.
    pub cost_microdollars: i64,
}

/// Pattern — `agentspec.cli.Pattern`.
///
/// A run of tokens that recurs in the turns of agents across sessions. Support counts distinct sessions, not occurrences. The query is the sq text that finds the pattern again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    /// `id` — `String`.
    pub id: String,
    /// `tokens` — `List<String>`.
    pub tokens: Vec<String>,
    /// `sessions` — `Integer`.
    pub sessions: i64,
    /// `projects` — `Integer`.
    pub projects: i64,
    /// `occurrences` — `Integer`.
    pub occurrences: i64,
    /// `ok_permille` — `Integer`.
    pub ok_permille: i64,
    /// `query` — `String`.
    pub query: String,
}

/// Procedure — `agentspec.cli.Procedure`.
///
/// A mined pattern lifted into a procedure, format `agentspec-procedure/1`: steps, the parameters that vary across matches, and the sessions it came from. A draft until a person promotes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Procedure {
    /// `format` — `String`.
    pub format: String,
    /// `id` — `String`.
    pub id: String,
    /// `description` — `String`.
    pub description: String,
    /// `pattern` — `List<String>`.
    pub pattern: Vec<String>,
    /// `matches` — `Integer`.
    pub matches: i64,
    /// `sessions` — `List<String>`.
    pub sessions: Vec<String>,
    /// `parameters` — `List<agentspec.cli.ProcedureParameter>`.
    pub parameters: Vec<ProcedureParameter>,
    /// `steps` — `List<agentspec.cli.ProcedureStep>`.
    pub steps: Vec<ProcedureStep>,
    /// `query` — `String`.
    pub query: String,
}

/// ProcedureInput — `agentspec.cli.ProcedureInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureInput {
    /// `pattern` — `String`.
    pub pattern: String,
    /// `root` — `Optional<String>`.
    pub root: Option<String>,
    /// `since_hours` — `Optional<Integer>`.
    pub since_hours: Option<i64>,
    /// `level` — `Optional<String>`.
    pub level: Option<String>,
    /// `out` — `Optional<String>`.
    pub out: Option<String>,
}

/// ProcedureParameter — `agentspec.cli.ProcedureParameter`.
///
/// A value that varies across the matches of a pattern. Values are kept only when they are not content (a tool name, a category, a count), never a path or a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureParameter {
    /// `name` — `String`.
    pub name: String,
    /// `step` — `String`.
    pub step: String,
    /// `kind` — `String`.
    pub kind: String,
    /// `values` — `List<String>`.
    pub values: Vec<String>,
}

/// ProcedureStep — `agentspec.cli.ProcedureStep`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStep {
    /// `id` — `String`.
    pub id: String,
    /// `kind` — `agentspec.cli.StepKind`.
    pub kind: StepKind,
    /// `token` — `String`.
    pub token: String,
    /// `tool` — `String`.
    pub tool: String,
    /// `category` — `String`.
    pub category: String,
    /// `shape` — `Optional<String>`.
    pub shape: Option<String>,
    /// `effect` — `String`.
    pub effect: String,
    /// `after` — `List<String>`.
    pub after: Vec<String>,
    /// `retry_max` — `Integer`.
    pub retry_max: i64,
    /// `timeout_seconds` — `Integer`.
    pub timeout_seconds: i64,
    /// `ok_permille` — `Integer`.
    pub ok_permille: i64,
}

/// PromptFigures — `agentspec.cli.PromptFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptFigures {
    /// `author` — `agentspec.session.PromptAuthor`.
    pub author: crate::session::PromptAuthor,
    /// `attribution` — `agentspec.session.Attribution`.
    pub attribution: crate::session::Attribution,
    /// `count` — `Integer`.
    pub count: i64,
}

/// QueryInput — `agentspec.cli.QueryInput`.
///
/// An sq module, from a file or inline text, run over one session or over the sessions of a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryInput {
    /// `module` — `Optional<String>`.
    pub module: Option<String>,
    /// `text` — `Optional<String>`.
    pub text: Option<String>,
    /// `transcript` — `Optional<String>`.
    pub transcript: Option<String>,
    /// `root` — `Optional<String>`.
    pub root: Option<String>,
    /// `since_hours` — `Optional<Integer>`.
    pub since_hours: Option<i64>,
}

/// QueryReport — `agentspec.cli.QueryReport`.
///
/// The report of `agentspec query`, format `agentspec-query-report/1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryReport {
    /// `format` — `String`.
    pub format: String,
    /// `sessions` — `Integer`.
    pub sessions: i64,
    /// `queries` — `List<String>`.
    pub queries: Vec<String>,
    /// `rows` — `List<agentspec.cli.QueryRow>`.
    pub rows: Vec<QueryRow>,
    /// `checks` — `List<agentspec.cli.CheckFigures>`.
    pub checks: Vec<CheckFigures>,
    /// `verdicts` — `List<agentspec.cli.CheckVerdict>`.
    pub verdicts: Vec<CheckVerdict>,
}

/// QueryRow — `agentspec.cli.QueryRow`.
///
/// One answer of a query. Possible when it rests on an inferred or missing fact; unknown names each such fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRow {
    /// `query` — `Integer`.
    pub query: i64,
    /// `session_id` — `String`.
    pub session_id: String,
    /// `values` — `List<String>`.
    pub values: Vec<String>,
    /// `certainty` — `agentspec.cli.Certainty`.
    pub certainty: Certainty,
    /// `unknown` — `List<String>`.
    pub unknown: Vec<String>,
}

/// Refusal — `agentspec.cli.Refusal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// `detail` — `String`.
    pub detail: String,
}

/// ServeInput — `agentspec.cli.ServeInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeInput {
    /// `root` — `Optional<String>`.
    pub root: Option<String>,
    /// `since_hours` — `Optional<Integer>`.
    pub since_hours: Option<i64>,
    /// `port` — `Optional<Integer>`.
    pub port: Option<i64>,
}

/// ServeResult — `agentspec.cli.ServeResult`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeResult {
    /// `url` — `String`.
    pub url: String,
}

/// SessionSummary — `agentspec.cli.SessionSummary`.
///
/// One session with its own KPI report. The title is the harness's title of the session and is shown only on the host that holds the transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummary {
    /// `session_id` — `String`.
    pub session_id: String,
    /// `project` — `String`.
    pub project: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `first_at` — `String`.
    pub first_at: String,
    /// `last_at` — `String`.
    pub last_at: String,
    /// `kpis` — `agentspec.cli.KpiReport`.
    pub kpis: KpiReport,
    /// `tools` — `List<agentspec.cli.ToolFigures>`.
    pub tools: Vec<ToolFigures>,
    /// `timeline` — `List<agentspec.cli.TimelineMark>`.
    pub timeline: Vec<TimelineMark>,
}

/// StepKind — `agentspec.cli.StepKind`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    /// `program`.
    Program,
    /// `judgement`.
    Judgement,
    /// `person`.
    Person,
    /// `child`.
    Child,
}

/// SyncInput — `agentspec.cli.SyncInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncInput {
    /// `root` — `Optional<String>`.
    pub root: Option<String>,
    /// `since_hours` — `Optional<Integer>`.
    pub since_hours: Option<i64>,
}

/// SyncResult — `agentspec.cli.SyncResult`.
///
/// One pass of the store over the transcripts of a window. A main transcript whose own file and sub-agent files keep their size and modification time is `unchanged`; every other one is lifted, redacted and written again in one transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncResult {
    /// `db` — `String`.
    pub db: String,
    /// `scanned` — `Integer`.
    pub scanned: i64,
    /// `lifted` — `Integer`.
    pub lifted: i64,
    /// `unchanged` — `Integer`.
    pub unchanged: i64,
    /// `failed` — `Integer`.
    pub failed: i64,
    /// `sessions` — `Integer`.
    pub sessions: i64,
    /// `ms` — `Integer`.
    pub ms: i64,
}

/// TimelineMark — `agentspec.cli.TimelineMark`.
///
/// One point of a session's timeline. For a turn the value is the context size, for a cache bust the tokens written again, otherwise 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineMark {
    /// `at` — `String`.
    pub at: String,
    /// `kind` — `agentspec.cli.MarkKind`.
    pub kind: MarkKind,
    /// `scope` — `agentspec.cli.AgentScope`.
    pub scope: AgentScope,
    /// `value` — `Integer`.
    pub value: i64,
}

/// ToolFigures — `agentspec.cli.ToolFigures`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolFigures {
    /// `category` — `agentspec.session.ToolCategory`.
    pub category: crate::session::ToolCategory,
    /// `calls` — `Integer`.
    pub calls: i64,
    /// `refused` — `Integer`.
    pub refused: i64,
    /// `failed` — `Integer`.
    pub failed: i64,
}
