//! The Claude Code adapter: one session's transcript files, lifted into session IR commands.
//!
//! A session is `<project>/<session>.jsonl` and its sub-agents `<project>/<session>/subagents/
//! agent-<id>.jsonl` with `agent-<id>.meta.json` beside each (`parentAgentId` when a sub-agent
//! spawned it). The main agent is `main`; a sub-agent is its own id.
//!
//! What the transcript does not say is not invented: a prompt's author is `Known` only from a
//! structured field (a queued command's `origin.kind`), `Inferred` from a text prefix, and a plain
//! prompt with no mark is `Human`/`Inferred`. No `EndAgent` or `EndSession` is lifted: a
//! transcript does not record either.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use agenticspec_types::primitives::Timestamp;
use agenticspec_types::session as s;
use serde_json::Value;

use crate::redact::{Redactor, digest_hex};
use crate::time::epoch_ms;

/// Who issues a command, as the spec's actors name them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    HarnessRuntime,
    Human,
    Delegate,
    PeerSession,
    AgentLoop,
    PermissionAuthority,
    ToolProvider,
    Redactor,
}

#[derive(Debug, Clone)]
pub enum Cmd {
    OpenSession(s::OpenSession),
    StartAgent(s::StartAgent),
    SubmitPrompt(s::SubmitPrompt),
    EndTurn(s::EndTurn),
    RecordModelTurn(s::RecordModelTurn),
    CompactContext(s::CompactContext),
    RequestToolCall(s::RequestToolCall),
    DecideToolCall(s::DecideToolCall),
    CompleteToolCall(s::CompleteToolCall),
    RecordExposure(s::RecordExposure),
}

#[derive(Debug, Clone)]
pub struct Step {
    /// The instant the transcript records for this command.
    pub at_ms: i64,
    /// The replay order: `at_ms`, never earlier than the agent's previous step. Transcripts hold
    /// records stamped out of order; replay keeps each agent's own record order.
    pub order_ms: i64,
    pub actor: Actor,
    /// Where the record was observed: a transcript on disk or a call on a gateway's wire.
    pub source: s::Source,
    pub cmd: Cmd,
}

pub struct Lifted {
    pub session_id: String,
    /// The harness's title of the session: a title the user gave, else the agent name, else the
    /// generated title. Harness metadata, not part of the IR.
    pub title: Option<String>,
    pub agents: usize,
    pub files: Vec<PathBuf>,
    pub steps: Vec<Step>,
}

/// One sub-agent transcript: its agent id, the parent it names and the parent's tool call that
/// started it (`toolUseId` of the meta file).
pub struct SubAgentFile {
    pub path: PathBuf,
    pub id: String,
    pub parent: Option<String>,
    pub spawned_by: Option<String>,
}

/// The sub-agent transcripts of a main transcript, parents before children.
pub fn sub_agent_files(main: &Path) -> Vec<SubAgentFile> {
    let dir = main.with_extension("").join("subagents");
    let mut out: Vec<(i64, SubAgentFile)> = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(id) = name
            .strip_prefix("agent-")
            .and_then(|n| n.strip_suffix(".jsonl"))
        else {
            continue;
        };
        let meta = fs::read_to_string(p.with_extension("meta.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok());
        let parent = meta
            .as_ref()
            .and_then(|m| m.get("parentAgentId").and_then(Value::as_str))
            .map(str::to_owned);
        let depth = meta
            .as_ref()
            .and_then(|m| m.get("spawnDepth").and_then(Value::as_i64))
            .unwrap_or(1);
        let spawned_by = meta
            .as_ref()
            .and_then(|m| m.get("toolUseId").and_then(Value::as_str))
            .map(str::to_owned);
        out.push((
            depth,
            SubAgentFile {
                path: p.clone(),
                id: id.to_owned(),
                parent,
                spawned_by,
            },
        ));
    }
    // Parents before children: a sub-agent transcript can open with records copied from another
    // agent of the session, and the agent read first keeps them.
    out.sort_by(|a, b| (a.0, &a.1.id).cmp(&(b.0, &b.1.id)));
    out.into_iter().map(|(_, f)| f).collect()
}

/// The class of a prompt from rules over its text. The text is read here, on the host, and only
/// the class is kept. Harness and hook prompts are `Unknown`.
pub fn prompt_class(text: &str, author: s::PromptAuthor) -> s::PromptClass {
    use s::PromptClass::*;
    if matches!(author, s::PromptAuthor::Harness | s::PromptAuthor::Hook) {
        return Unknown;
    }
    let t = text.trim().to_lowercase();
    let short = t.chars().count() <= 60;
    let first = t
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .find(|w| !w.is_empty())
        .unwrap_or("");
    const CONTINUE: [&str; 9] = [
        "continue",
        "go on",
        "keep going",
        "proceed",
        "go ahead",
        "carry on",
        "weiter",
        "resume",
        "next",
    ];
    if short && CONTINUE.iter().any(|p| t.starts_with(p)) {
        return Continue;
    }
    const APPROVE: [&str; 13] = [
        "yes", "yep", "yeah", "ok", "okay", "sure", "approved", "approve", "lgtm", "do it", "ja",
        "go", "ship it",
    ];
    let option = t.len() <= 3 && t.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    if short && (option || APPROVE.iter().any(|p| first == *p || t.starts_with(p))) {
        return Approval;
    }
    const CORRECT_FIRST: [&str; 7] = ["no", "nope", "wrong", "stop", "don't", "dont", "nein"];
    const CORRECT_ANY: [&str; 8] = [
        "that's not",
        "that is not",
        "not what i",
        "i said",
        "i told you",
        "you didn't",
        "why did you",
        "you forgot",
    ];
    let head: String = t.chars().take(120).collect();
    if CORRECT_FIRST.contains(&first) || CORRECT_ANY.iter().any(|p| head.contains(p)) {
        return Correction;
    }
    if t.ends_with('?') && t.chars().count() <= 400 {
        return Question;
    }
    Task
}

/// A working directory as `~/…` when it lies under the home directory.
fn home_relative(cwd: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && cwd.starts_with(&home) => {
            format!("~{}", &cwd[home.len()..])
        }
        _ => cwd.to_owned(),
    }
}

pub fn lift(main: &Path) -> std::io::Result<Lifted> {
    let session_id = main
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("session")
        .to_owned();
    let mut steps: Vec<Step> = Vec::new();
    let mut files = vec![main.to_path_buf()];
    let mut seen = Seen::default();

    let main_steps = lift_agent(
        main,
        AgentStart {
            id: "main",
            parent: None,
            spawned_by: None,
            role: s::AgentRole::Main,
        },
        &session_id,
        &mut seen,
    )?;
    let opened_at = main_steps.first().map(|st| st.at_ms);
    let subs = sub_agent_files(main);
    let mut agents = 1;
    let mut all = main_steps;
    for sub in &subs {
        let parent = sub.parent.clone().unwrap_or_else(|| "main".to_owned());
        match lift_agent(
            &sub.path,
            AgentStart {
                id: &sub.id,
                parent: Some(parent),
                spawned_by: sub.spawned_by.clone(),
                role: s::AgentRole::SubAgent,
            },
            &session_id,
            &mut seen,
        ) {
            Ok(st) => {
                agents += 1;
                files.push(sub.path.clone());
                all.extend(st);
            }
            Err(_) => continue,
        }
    }
    let first = all
        .iter()
        .map(|st| st.order_ms)
        .min()
        .or(opened_at)
        .unwrap_or(0);
    steps.push(Step {
        at_ms: first,
        order_ms: first,
        actor: Actor::HarnessRuntime,
        source: s::Source::Disk,
        cmd: Cmd::OpenSession(s::OpenSession {
            session_id: s::SessionId(session_id.clone()),
            harness: s::Harness::ClaudeCode,
            source: Some(s::Source::Disk),
            session_hash: Some(crate::wire::session_hash("claude-code", &session_id)),
            at: ts(first),
        }),
    });
    // A stable sort on the replay order keeps each agent's own order; StartAgent is its first step.
    all.sort_by_key(|st| st.order_ms);
    // A sub-agent's parent must exist before it: StartAgent steps of parents sort earlier because a
    // parent's first record precedes the spawn; a parent the transcript lacks is started as Main.
    let started: HashSet<String> = all
        .iter()
        .filter_map(|st| match &st.cmd {
            Cmd::StartAgent(a) => Some(a.agent_id.0.clone()),
            _ => None,
        })
        .collect();
    for st in &mut all {
        if let Cmd::StartAgent(a) = &mut st.cmd
            && let Some(p) = &a.parent_agent_id
            && !started.contains(&p.0)
        {
            a.parent_agent_id = None;
        }
    }
    steps.extend(all);
    let [custom, named, generated] = seen.titles.clone();
    let title = custom.or(named).or(generated);
    Ok(Lifted {
        session_id,
        title,
        agents,
        files,
        steps,
    })
}

fn ts(ms: i64) -> Timestamp {
    Timestamp(crate::time::rfc3339(ms))
}

fn content(text: &str) -> s::Content {
    s::Content {
        digest: digest_hex(text.as_bytes()),
        bytes: text.len() as i64,
    }
}

fn text_of(content: &Value) -> String {
    match content {
        Value::String(t) => t.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(|b| match b.get("type").and_then(Value::as_str) {
                Some("text") => b.get("text").and_then(Value::as_str).map(str::to_owned),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn result_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(t)) => t.clone(),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

/// Text that carries harness bookkeeping and starts no turn.
fn bookkeeping(t: &str) -> bool {
    const P: [&str; 8] = [
        "<local-command-stdout>",
        "<local-command-stderr>",
        "<local-command-caveat>",
        "<command-name>",
        "<command-message>",
        "Caveat:",
        "<system-reminder>",
        "[Request interrupted",
    ];
    t.trim().is_empty() || P.iter().any(|p| t.starts_with(p))
}

/// The names of the sessions that act for the human on this host (delegates), from the
/// environment variable `AGENTICSPEC_DELEGATES`, comma-separated, else the older
/// `AGENTSPEC_DELEGATES` or `SESSIONSPEC_DELEGATES`. Empty when none is set.
pub fn delegates() -> &'static [String] {
    static D: OnceLock<Vec<String>> = OnceLock::new();
    D.get_or_init(|| {
        std::env::var("AGENTICSPEC_DELEGATES")
            .or_else(|_| std::env::var("AGENTSPEC_DELEGATES"))
            .or_else(|_| std::env::var("SESSIONSPEC_DELEGATES"))
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(str::to_owned)
            .collect()
    })
}

/// Author and attribution of a prompt from its text, with the delegates of [`delegates`].
pub fn classify(text: &str, sub_agent: bool) -> (s::PromptAuthor, s::Attribution) {
    classify_with(text, sub_agent, delegates())
}

/// Author and attribution of a prompt from its text. A message of another session is the
/// delegate's when its sender is named in `delegates` or it says it is sent on someone's order
/// ("'s order", "on behalf of"); a prompt that opens with "Typed", "Started" or "Resumed" and
/// says so, or names a delegate, is the delegate's too.
pub fn classify_with(
    text: &str,
    sub_agent: bool,
    delegates: &[String],
) -> (s::PromptAuthor, s::Attribution) {
    use s::Attribution::*;
    use s::PromptAuthor::*;
    let t = text.trim_start();
    let head: String = t.chars().take(200).collect();
    let typed_for = |h: &str| h.contains("'s order") || h.contains("on behalf of");
    if t.starts_with("<task-notification") {
        return (Harness, Known);
    }
    if t.starts_with("<cross-session-message")
        || t.starts_with("Another Claude session sent a message")
    {
        if delegates
            .iter()
            .any(|d| head.contains(&format!("from-name=\"{d}\"")))
            || typed_for(&head)
        {
            return (Delegate, Inferred);
        }
        return (PeerSession, Known);
    }
    if t.starts_with("Stop hook feedback") || t.starts_with("<user-prompt-submit-hook") {
        return (Hook, Known);
    }
    if t.starts_with("This session is being continued")
        || t.starts_with("Base directory for this skill")
    {
        return (Harness, Known);
    }
    if ((t.starts_with("Typed") || t.starts_with("Started") || t.starts_with("Resumed"))
        && (typed_for(&head) || delegates.iter().any(|d| head.contains(d.as_str()))))
        || delegates
            .iter()
            .any(|d| t.starts_with(d.as_str()) && typed_for(&head))
    {
        return (Delegate, Inferred);
    }
    if sub_agent {
        return (ParentAgent, Inferred);
    }
    const BRIEF: [&str; 6] = [
        "Your brief is",
        "Your full brief",
        "Read and follow",
        "Read your brief",
        "Read and execute the brief",
        "Follow every rule in",
    ];
    if BRIEF.iter().any(|p| t.starts_with(p)) {
        return (PeerSession, Inferred);
    }
    (Human, Inferred)
}

fn decider(pd: Option<&Value>) -> (s::Decision, s::Decider) {
    let Some(pd) = pd else {
        return (s::Decision::Accept, s::Decider::NotRecorded);
    };
    let decision = if pd.get("decision").and_then(Value::as_str) == Some("reject") {
        s::Decision::Refuse
    } else {
        s::Decision::Accept
    };
    let source = pd.get("source").and_then(Value::as_str).unwrap_or("");
    let reason = pd.get("reasonType").and_then(Value::as_str).unwrap_or("");
    let who = match source {
        "hook" => s::Decider::Hook,
        src if src.starts_with("user") => s::Decider::Human,
        "config" => match reason {
            "classifier" => s::Decider::Classifier,
            "safetyCheck" => s::Decider::SafetyCheck,
            "subcommandResults" => s::Decider::Rule,
            _ if pd.get("ruleSource").is_some() => s::Decider::Rule,
            _ => s::Decider::PermissionMode,
        },
        _ => s::Decider::NotRecorded,
    };
    (decision, who)
}

pub fn category(tool: &str) -> s::ToolCategory {
    use s::ToolCategory::*;
    match tool {
        "Read" | "NotebookRead" => Read,
        "Grep" | "Glob" | "LS" | "ToolSearch" => Search,
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => Edit,
        "Bash" | "BashOutput" | "KillShell" | "Monitor" | "TaskStop" => Execute,
        "WebFetch" | "WebSearch" => Network,
        "Agent" | "Task" => Spawn,
        "SendMessage" | "ListAgents" | "PushNotification" | "AskUserQuestion" => Message,
        "TodoWrite" | "EnterPlanMode" | "ExitPlanMode" | "Skill" => Plan,
        _ => Other,
    }
}

/// What the agents of one session already lifted: record uuids, the agent each model message
/// belongs to, the messages recorded as turns, and the tool calls decided.
#[derive(Default)]
pub struct Seen {
    titles: [Option<String>; 3],
    redactor: Redactor,
    exposure_seq: usize,
    uuids: HashSet<String>,
    messages: HashMap<String, String>,
    turns: HashSet<String>,
    decided: HashSet<String>,
}

struct AgentLift<'a> {
    agent: s::AgentId,
    sub: bool,
    working: bool,
    out: Vec<Step>,
    redactor: &'a mut Redactor,
    exposure_seq: &'a mut usize,
    recent_prompts: Vec<(i64, String)>,
    clock: i64,
}

impl AgentLift<'_> {
    fn push(&mut self, at_ms: i64, actor: Actor, cmd: Cmd) {
        let order_ms = if self.out.is_empty() {
            at_ms
        } else {
            at_ms.max(self.clock)
        };
        self.clock = order_ms;
        self.out.push(Step {
            at_ms,
            order_ms,
            actor,
            source: s::Source::Disk,
            cmd,
        });
    }

    fn scan(
        &mut self,
        at_ms: i64,
        text: &str,
        channel: s::ExposureChannel,
        tool_call: Option<&str>,
    ) {
        for f in self.redactor.scan(text, channel) {
            *self.exposure_seq += 1;
            let cmd = Cmd::RecordExposure(s::RecordExposure {
                exposure_id: s::ExposureId(format!("{}:{}", self.agent.0, self.exposure_seq)),
                agent_id: self.agent.clone(),
                channel: f.channel,
                class: f.class,
                kind: f.kind.to_owned(),
                alias: f.alias,
                verdict: f.verdict,
                tool_call_id: tool_call.map(|t| s::ToolCallId(t.to_owned())),
                at: ts(at_ms),
            });
            self.push(at_ms, Actor::Redactor, cmd);
        }
    }

    fn prompt(
        &mut self,
        at_ms: i64,
        text: &str,
        author: s::PromptAuthor,
        attribution: s::Attribution,
    ) {
        let c = content(text);
        // A queued prompt can appear twice: as a queued-command attachment and as a user record.
        if self
            .recent_prompts
            .iter()
            .any(|(t, d)| (at_ms - t).abs() <= 10_000 && *d == c.digest)
        {
            return;
        }
        self.recent_prompts.retain(|(t, _)| at_ms - t <= 10_000);
        self.recent_prompts.push((at_ms, c.digest.clone()));
        let actor = match author {
            s::PromptAuthor::Human => Actor::Human,
            s::PromptAuthor::Delegate => Actor::Delegate,
            s::PromptAuthor::PeerSession => Actor::PeerSession,
            s::PromptAuthor::ParentAgent => Actor::AgentLoop,
            s::PromptAuthor::Harness | s::PromptAuthor::Hook => Actor::HarnessRuntime,
        };
        self.scan(at_ms, text, s::ExposureChannel::Prompt, None);
        self.push(
            at_ms,
            actor,
            Cmd::SubmitPrompt(s::SubmitPrompt {
                agent_id: self.agent.clone(),
                author,
                attribution,
                class: prompt_class(text, author),
                content: c,
                at: ts(at_ms),
            }),
        );
        self.working = true;
    }

    fn end_turn(&mut self, at_ms: i64) {
        if self.working {
            self.working = false;
            self.push(
                at_ms,
                Actor::AgentLoop,
                Cmd::EndTurn(s::EndTurn {
                    agent_id: self.agent.clone(),
                    at: ts(at_ms),
                }),
            );
        }
    }
}

fn read_records(path: &Path) -> std::io::Result<Vec<Value>> {
    let f = fs::File::open(path)?;
    let mut out = Vec::new();
    for line in BufReader::new(f).lines() {
        let Ok(line) = line else { continue };
        // Request blobs carry the whole prompt again and nothing this adapter reads.
        if line.starts_with("{\"type\":\"api-request") {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(&line) {
            out.push(v);
        }
    }
    Ok(out)
}

pub(crate) fn usage_of(u: &Value) -> (s::Usage, i64) {
    let g = |k: &str| u.get(k).and_then(Value::as_i64).unwrap_or(0);
    let cc = u.get("cache_creation");
    let w1h = cc
        .and_then(|c| c.get("ephemeral_1h_input_tokens"))
        .and_then(Value::as_i64);
    let w5m = cc
        .and_then(|c| c.get("ephemeral_5m_input_tokens"))
        .and_then(Value::as_i64);
    let write = g("cache_creation_input_tokens");
    let (w5m, w1h) = match (w5m, w1h) {
        (Some(a), Some(b)) => (a, b),
        _ => (write, 0),
    };
    let usage = s::Usage {
        uncached_input: g("input_tokens"),
        cache_read: g("cache_read_input_tokens"),
        cache_write_5m: w5m,
        cache_write_1h: w1h,
        output: g("output_tokens"),
    };
    let ctx = usage.uncached_input + usage.cache_read + write;
    (usage, ctx)
}

struct AgentStart<'a> {
    id: &'a str,
    parent: Option<String>,
    spawned_by: Option<String>,
    role: s::AgentRole,
}

fn lift_agent(
    path: &Path,
    start: AgentStart<'_>,
    session_id: &str,
    seen: &mut Seen,
) -> std::io::Result<Vec<Step>> {
    let AgentStart {
        id: agent_id,
        parent,
        spawned_by,
        role,
    } = start;
    let records = read_records(path)?;
    let sub = role == s::AgentRole::SubAgent;
    // The final usage, model and stop reason of each model message: a message spans several records.
    let mut finals: HashMap<String, (Value, Option<String>, bool)> = HashMap::new();
    for r in &records {
        if r.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let m = &r["message"];
        let Some(id) = m.get("id").and_then(Value::as_str) else {
            continue;
        };
        let e = finals
            .entry(id.to_owned())
            .or_insert((Value::Null, None, false));
        if let Some(u) = m.get("usage") {
            e.0 = u.clone();
        }
        if let Some(model) = m.get("model").and_then(Value::as_str) {
            e.1 = Some(model.to_owned());
        }
        if m.get("stop_reason").and_then(Value::as_str) == Some("end_turn") {
            e.2 = true;
        }
    }

    let mut a = AgentLift {
        agent: s::AgentId(agent_id.to_owned()),
        sub,
        working: false,
        out: Vec::new(),
        redactor: &mut seen.redactor,
        exposure_seq: &mut seen.exposure_seq,
        recent_prompts: Vec::new(),
        clock: 0,
    };
    let mut started = false;
    let mut first_prompt = true;

    for r in &records {
        if !sub {
            let slot = match r.get("type").and_then(Value::as_str) {
                Some("custom-title") => Some((0, "customTitle")),
                Some("agent-name") => Some((1, "agentName")),
                Some("ai-title") => Some((2, "aiTitle")),
                _ => None,
            };
            if let Some((i, key)) = slot
                && let Some(t) = r.get(key).and_then(Value::as_str)
            {
                seen.titles[i] = Some(t.to_owned());
            }
        }
        let Some(at_ms) = r
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(epoch_ms)
        else {
            continue;
        };
        // A record another agent of the session already holds was copied into this transcript.
        if let Some(uuid) = r.get("uuid").and_then(Value::as_str)
            && !seen.uuids.insert(uuid.to_owned())
        {
            continue;
        }
        if !started {
            started = true;
            a.push(
                at_ms,
                if sub {
                    Actor::AgentLoop
                } else {
                    Actor::HarnessRuntime
                },
                Cmd::StartAgent(s::StartAgent {
                    agent_id: a.agent.clone(),
                    session_id: s::SessionId(session_id.to_owned()),
                    parent_agent_id: parent.clone().map(s::AgentId),
                    role,
                    spawned_by: spawned_by.clone().map(s::ToolCallId),
                    at: ts(at_ms),
                }),
            );
        }
        match r.get("type").and_then(Value::as_str).unwrap_or("") {
            "assistant" => {
                let m = &r["message"];
                let Some(id) = m.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let (usage, model, ends) =
                    finals
                        .get(id)
                        .cloned()
                        .unwrap_or((Value::Null, None, false));
                let model = model.unwrap_or_default();
                if model == "<synthetic>" {
                    continue;
                }
                let owner = seen
                    .messages
                    .entry(id.to_owned())
                    .or_insert_with(|| agent_id.to_owned());
                if owner != agent_id {
                    continue;
                }
                if seen.turns.insert(id.to_owned()) {
                    let (u, ctx) = usage_of(&usage);
                    a.push(
                        at_ms,
                        Actor::AgentLoop,
                        Cmd::RecordModelTurn(s::RecordModelTurn {
                            agent_id: a.agent.clone(),
                            model,
                            usage: u,
                            context_tokens: ctx,
                            message_id: Some(id.to_owned()),
                            call_id: None,
                            route: None,
                            upstream: None,
                            attempts: None,
                            queue_wait_ms: None,
                            ttft_ms: None,
                            declared_cost_microdollars: None,
                            incomplete: None,
                            at: ts(at_ms),
                        }),
                    );
                }
                if let Some(blocks) = m.get("content").and_then(Value::as_array) {
                    for b in blocks {
                        match b.get("type").and_then(Value::as_str) {
                            Some("tool_use") => {
                                let call =
                                    b.get("id").and_then(Value::as_str).unwrap_or("").to_owned();
                                let tool = b
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_owned();
                                let raw = b.get("input");
                                let input = raw.map(Value::to_string).unwrap_or_default();
                                let background = raw
                                    .and_then(|i| i.get("run_in_background"))
                                    .and_then(Value::as_bool)
                                    == Some(true);
                                let cwd = r.get("cwd").and_then(Value::as_str).map(home_relative);
                                a.scan(at_ms, &input, s::ExposureChannel::ToolInput, Some(&call));
                                a.push(
                                    at_ms,
                                    Actor::AgentLoop,
                                    Cmd::RequestToolCall(s::RequestToolCall {
                                        tool_call_id: s::ToolCallId(call),
                                        agent_id: a.agent.clone(),
                                        category: category(&tool),
                                        tool,
                                        input: content(&input),
                                        cwd,
                                        background,
                                        at: ts(at_ms),
                                    }),
                                );
                            }
                            Some("text") => {
                                let t = b.get("text").and_then(Value::as_str).unwrap_or("");
                                a.scan(at_ms, t, s::ExposureChannel::AgentText, None);
                            }
                            _ => {}
                        }
                    }
                }
                if ends && m.get("stop_reason").and_then(Value::as_str) == Some("end_turn") {
                    a.end_turn(at_ms);
                }
            }
            "user" => {
                if r.get("isMeta").and_then(Value::as_bool) == Some(true) {
                    continue;
                }
                let c = &r["message"]["content"];
                let results: Vec<&Value> = c
                    .as_array()
                    .map(|v| {
                        v.iter()
                            .filter(|b| {
                                b.get("type").and_then(Value::as_str) == Some("tool_result")
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if !results.is_empty() {
                    let (decision, who) = decider(r.get("permissionDecision"));
                    let reason = r
                        .get("permissionDecision")
                        .and_then(|p| p.get("reasonCode").or_else(|| p.get("reasonType")))
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    for b in results {
                        let call = b
                            .get("tool_use_id")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_owned();
                        // A transcript can hold one tool result twice; the call was decided once.
                        if !seen.decided.insert(call.clone()) {
                            continue;
                        }
                        let actor = if who == s::Decider::Human {
                            Actor::Human
                        } else {
                            Actor::PermissionAuthority
                        };
                        a.push(
                            at_ms,
                            actor,
                            Cmd::DecideToolCall(s::DecideToolCall {
                                tool_call_id: s::ToolCallId(call.clone()),
                                decision,
                                decider: who,
                                reason: reason.clone(),
                                at: ts(at_ms),
                            }),
                        );
                        let text = result_text(b.get("content"));
                        a.scan(at_ms, &text, s::ExposureChannel::ToolOutput, Some(&call));
                        if decision == s::Decision::Accept {
                            let ok = b.get("is_error").and_then(Value::as_bool) != Some(true);
                            a.push(
                                at_ms,
                                Actor::ToolProvider,
                                Cmd::CompleteToolCall(s::CompleteToolCall {
                                    tool_call_id: s::ToolCallId(call),
                                    ok,
                                    output: content(&text),
                                    at: ts(at_ms),
                                }),
                            );
                        }
                    }
                    continue;
                }
                let text = text_of(c);
                if bookkeeping(&text) {
                    continue;
                }
                let (author, attribution) = if a.sub && first_prompt {
                    (s::PromptAuthor::ParentAgent, s::Attribution::Known)
                } else {
                    classify(&text, a.sub)
                };
                first_prompt = false;
                a.prompt(at_ms, &text, author, attribution);
            }
            "attachment" => {
                let at = &r["attachment"];
                if at.get("type").and_then(Value::as_str) != Some("queued_command") {
                    continue;
                }
                let text = at
                    .get("prompt")
                    .map(|p| p.as_str().map(str::to_owned).unwrap_or_else(|| text_of(p)))
                    .unwrap_or_default();
                if text.trim().is_empty() {
                    continue;
                }
                let (author, attribution) = match at
                    .get("origin")
                    .and_then(|o| o.get("kind"))
                    .and_then(Value::as_str)
                {
                    Some("human") => (s::PromptAuthor::Human, s::Attribution::Known),
                    Some("peer") => classify(&text, a.sub),
                    Some("task-notification") => (s::PromptAuthor::Harness, s::Attribution::Known),
                    Some("auto-continuation") => (s::PromptAuthor::Harness, s::Attribution::Known),
                    _ => classify(&text, a.sub),
                };
                first_prompt = false;
                a.prompt(at_ms, &text, author, attribution);
            }
            "system" => match r.get("subtype").and_then(Value::as_str) {
                Some("compact_boundary") => {
                    let before = r
                        .get("compactMetadata")
                        .and_then(|m| m.get("preTokens"))
                        .and_then(Value::as_i64)
                        .unwrap_or(0);
                    a.push(
                        at_ms,
                        Actor::HarnessRuntime,
                        Cmd::CompactContext(s::CompactContext {
                            agent_id: a.agent.clone(),
                            context_tokens_before: before,
                            at: ts(at_ms),
                        }),
                    );
                }
                Some("turn_duration") => a.end_turn(at_ms),
                _ => {}
            },
            _ => {}
        }
    }
    Ok(a.out)
}
