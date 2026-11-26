//! The wire source (`design/interaction-records.md` § 2): one finished model call on a gateway,
//! lifted into the session IR's commands.
//!
//! A gateway hands each finished call to [`lift_call`] with the [`WireSession`] it keeps per
//! session hash. The call carries the inbound wire's raw request bytes, the answer (one JSON body
//! or the decoded data of each stream event) and the gateway's own facts. The lift reads only the
//! fields it needs from the three wires (`messages`, `chat`, `responses`) and keeps no text: a
//! prompt, a tool input and a tool output become a digest and a byte count.
//!
//! What the wire does not show is marked, never guessed: a prompt's author is `Inferred`, a tool
//! call's decision is `NotRecorded` (an accepted call whose result says it failed may have been
//! refused by a hook), and a sub-agent's parent is found by digest, so its attribution is
//! `Inferred`.

use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::Value;
use sessionspec_types::primitives::Timestamp;
use sessionspec_types::session as s;

use crate::claude::{Actor, Cmd, Step, category, prompt_class};
use crate::redact::digest_hex;

/// The gateway's name of a session: SHA-256 of `llm-gateway.session/1\0<source>\0<value>`, its
/// first 6 bytes as 12 lower-case hex digits. `source` is `claude-code` for the
/// `x-claude-code-session-id` header, `session-id` for `session-id`, `session_id` and the
/// caller-session header, `agent-address` for the user agent and client address.
pub fn session_hash(source: &str, value: &str) -> String {
    let full = digest_hex(format!("llm-gateway.session/1\0{source}\0{value}").as_bytes());
    full[..12].to_owned()
}

/// The inbound wire a client spoke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wire {
    Messages,
    Chat,
    Responses,
}

/// The answer of one call as the gateway relayed it.
#[derive(Debug, Clone)]
pub enum Answer {
    /// A whole JSON body (a call that did not stream).
    Body(Vec<u8>),
    /// The `data:` payload of each server-sent event, in order.
    Events(Vec<Vec<u8>>),
}

/// What the gateway itself knows about a finished call.
#[derive(Debug, Clone)]
pub struct GatewayFacts {
    /// `x-llm-gateway-call`.
    pub call_id: String,
    /// [`session_hash`] of the header that named the session.
    pub session_hash: String,
    pub harness: s::Harness,
    /// The request path; `/v1/messages/count_tokens` is no model turn and lifts to nothing.
    pub path: String,
    pub route: Option<String>,
    pub upstream: Option<String>,
    pub attempts: i64,
    pub queue_wait_ms: Option<i64>,
    pub ttft_ms: Option<i64>,
    /// Token counts times the price the operator declares for the route; absent without one.
    pub declared_cost_microdollars: Option<i64>,
    /// Why the answer is not whole, when it is not.
    pub incomplete: Option<s::IncompleteReason>,
    pub started_ms: i64,
    pub finished_ms: i64,
}

/// One finished call.
#[derive(Debug, Clone)]
pub struct WireCall {
    pub wire: Wire,
    pub request: Vec<u8>,
    pub answer: Answer,
    pub facts: GatewayFacts,
}

#[derive(Debug, Clone, Default)]
struct AgentState {
    id: String,
    idle: bool,
    /// Tool calls requested and not yet completed.
    open: HashSet<String>,
}

/// What a gateway keeps per session between calls: the agents found so far and the links that
/// tell the next call's agent.
#[derive(Debug, Default)]
pub struct WireSession {
    opened: bool,
    clock: i64,
    main: Option<String>,
    side: Option<String>,
    /// Agent by the digest of its conversation's first user message.
    by_first: HashMap<String, AgentState>,
    /// Chain key (first-message digest) by response id, for `previous_response_id`. Holds the
    /// [`LINKS_HELD`] most recent responses.
    by_response: Links<String>,
    /// Chain key by `conversation` id. Holds the [`LINKS_HELD`] most recent conversations.
    by_conversation: Links<String>,
    /// A spawn call's string inputs by digest: (agent, tool call). Holds the [`LINKS_HELD`] most
    /// recent inputs.
    spawns: Links<(String, String)>,
    /// Calls lifted, calls skipped (`count_tokens`, an unreadable request).
    pub lifted: u64,
    pub skipped: u64,
}

impl WireSession {
    pub fn new() -> WireSession {
        WireSession::default()
    }

    /// How many links the session holds now: response ids, conversation ids, spawn inputs. Each
    /// is at most [`LINKS_HELD`], however long the session runs.
    pub fn links_held(&self) -> (usize, usize, usize) {
        (
            self.by_response.len(),
            self.by_conversation.len(),
            self.spawns.len(),
        )
    }
}

/// How many links of each kind a [`WireSession`] keeps: the most recent response ids,
/// conversation ids and spawn inputs. An older link is evicted. A later call that names an
/// evicted response or carries an evicted spawn input is not attached to the agent it belonged
/// to: it starts an agent of its own without a parent, the same as a chain the session never saw,
/// so a lost link shows as an unknown parent and never as a wrong one.
pub const LINKS_HELD: usize = 4096;

/// A map that keeps the [`LINKS_HELD`] most recently inserted keys.
#[derive(Debug, Default)]
struct Links<V> {
    map: HashMap<String, V>,
    order: VecDeque<String>,
}

impl<V> Links<V> {
    fn get(&self, key: &str) -> Option<&V> {
        self.map.get(key)
    }

    /// Inserts or replaces `key`; a new key evicts the oldest when the map is full.
    fn insert(&mut self, key: String, value: V) {
        if self.map.insert(key.clone(), value).is_none() {
            self.order.push_back(key);
            while self.order.len() > LINKS_HELD {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                }
            }
        }
    }

    /// Inserts `key` only when it is not held.
    fn insert_new(&mut self, key: &str, value: impl FnOnce() -> V) {
        if !self.map.contains_key(key) {
            self.insert(key.to_owned(), value());
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

fn ts(ms: i64) -> Timestamp {
    Timestamp(crate::time::rfc3339(ms))
}

fn content(bytes: &[u8]) -> s::Content {
    s::Content {
        digest: digest_hex(bytes),
        bytes: bytes.len() as i64,
    }
}

/// A value without the keys that move between calls of one conversation (`cache_control`), as
/// canonical JSON text.
fn canonical(v: &Value) -> String {
    fn strip(v: &Value) -> Value {
        match v {
            Value::Object(m) => Value::Object(
                m.iter()
                    .filter(|(k, _)| k.as_str() != "cache_control")
                    .map(|(k, x)| (k.clone(), strip(x)))
                    .collect(),
            ),
            Value::Array(a) => Value::Array(a.iter().map(strip).collect()),
            x => x.clone(),
        }
    }
    strip(v).to_string()
}

/// The text blocks of a message content: a string, or the `text` of each text-like block.
fn texts(content: &Value) -> Vec<String> {
    match content {
        Value::String(t) => vec![t.clone()],
        Value::Array(blocks) => blocks
            .iter()
            .filter(|b| {
                matches!(
                    b.get("type").and_then(Value::as_str),
                    Some("text" | "input_text")
                )
            })
            .filter_map(|b| b.get("text").and_then(Value::as_str).map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// A tool result the request hands back: the call id, whether it failed, its bytes.
struct ToolResult {
    id: String,
    ok: bool,
    output: Vec<u8>,
}

/// What a request says, wire-neutral.
struct Request {
    model: String,
    tools: bool,
    /// Messages in the conversation the request carries, system prompts not counted.
    history: usize,
    /// The conversation's first user message, canonical; None when the server holds the history.
    first: Option<String>,
    /// Its text blocks, for the spawn link.
    first_texts: Vec<String>,
    previous_response: Option<String>,
    conversation: Option<String>,
    /// The last message, when it is the user's: its text blocks and its tool results.
    last_texts: Vec<String>,
    results: Vec<ToolResult>,
}

fn result_output(v: Option<&Value>) -> Vec<u8> {
    match v {
        Some(Value::String(t)) => t.clone().into_bytes(),
        Some(x) => x.to_string().into_bytes(),
        None => Vec::new(),
    }
}

fn request_of(wire: Wire, body: &Value) -> Option<Request> {
    let model = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let tools = body
        .get("tools")
        .and_then(Value::as_array)
        .is_some_and(|t| !t.is_empty());
    let mut req = Request {
        model,
        tools,
        history: 0,
        first: None,
        first_texts: Vec::new(),
        previous_response: None,
        conversation: None,
        last_texts: Vec::new(),
        results: Vec::new(),
    };
    match wire {
        Wire::Messages | Wire::Chat => {
            let msgs: Vec<&Value> = body
                .get("messages")?
                .as_array()?
                .iter()
                .filter(|m| {
                    !matches!(
                        m.get("role").and_then(Value::as_str),
                        Some("system" | "developer")
                    )
                })
                .collect();
            req.history = msgs.len();
            let first = msgs
                .iter()
                .find(|m| m.get("role").and_then(Value::as_str) == Some("user"))?;
            req.first = Some(canonical(first.get("content").unwrap_or(&Value::Null)));
            req.first_texts = texts(first.get("content").unwrap_or(&Value::Null));
            if wire == Wire::Messages {
                let last = msgs.last()?;
                if last.get("role").and_then(Value::as_str) == Some("user") {
                    let c = last.get("content").unwrap_or(&Value::Null);
                    req.last_texts = texts(c);
                    if let Some(blocks) = c.as_array() {
                        for b in blocks {
                            if b.get("type").and_then(Value::as_str) == Some("tool_result") {
                                req.results.push(ToolResult {
                                    id: b
                                        .get("tool_use_id")
                                        .and_then(Value::as_str)
                                        .unwrap_or("")
                                        .to_owned(),
                                    ok: !b
                                        .get("is_error")
                                        .and_then(Value::as_bool)
                                        .unwrap_or(false),
                                    output: result_output(b.get("content")),
                                });
                            }
                        }
                    }
                }
            } else {
                // Chat: the tool messages after the last assistant message are its results, and a
                // trailing user message is a prompt. Chat carries no error flag: a result is ok.
                let after = msgs
                    .iter()
                    .rposition(|m| m.get("role").and_then(Value::as_str) == Some("assistant"))
                    .map_or(0, |i| i + 1);
                for m in &msgs[after..] {
                    match m.get("role").and_then(Value::as_str) {
                        Some("tool") => req.results.push(ToolResult {
                            id: m
                                .get("tool_call_id")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_owned(),
                            ok: true,
                            output: result_output(m.get("content")),
                        }),
                        Some("user") => {
                            req.last_texts = texts(m.get("content").unwrap_or(&Value::Null));
                        }
                        _ => {}
                    }
                }
            }
        }
        Wire::Responses => {
            req.previous_response = body
                .get("previous_response_id")
                .and_then(Value::as_str)
                .map(str::to_owned);
            req.conversation = match body.get("conversation") {
                Some(Value::String(c)) => Some(c.clone()),
                Some(Value::Object(o)) => o.get("id").and_then(Value::as_str).map(str::to_owned),
                _ => None,
            };
            let items: Vec<Value> = match body.get("input") {
                Some(Value::String(t)) => {
                    vec![serde_json::json!({"type": "message", "role": "user", "content": t})]
                }
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let is_user = |i: &Value| {
                i.get("role").and_then(Value::as_str) == Some("user")
                    && i.get("type").and_then(Value::as_str).unwrap_or("message") == "message"
            };
            req.history = items
                .iter()
                .filter(|i| i.get("role").and_then(Value::as_str) != Some("system"))
                .filter(|i| i.get("role").and_then(Value::as_str) != Some("developer"))
                .count();
            if req.previous_response.is_none() && req.conversation.is_none() {
                let first = items.iter().find(|i| is_user(i))?;
                req.first = Some(canonical(first.get("content").unwrap_or(&Value::Null)));
                req.first_texts = texts(first.get("content").unwrap_or(&Value::Null));
            }
            let after = items
                .iter()
                .rposition(|i| {
                    i.get("role").and_then(Value::as_str) == Some("assistant")
                        || i.get("type").and_then(Value::as_str) == Some("function_call")
                })
                .map_or(0, |i| i + 1);
            for i in &items[after..] {
                if i.get("type").and_then(Value::as_str) == Some("function_call_output") {
                    req.results.push(ToolResult {
                        id: i
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_owned(),
                        ok: true,
                        output: result_output(i.get("output")),
                    });
                } else if is_user(i) {
                    req.last_texts = texts(i.get("content").unwrap_or(&Value::Null));
                }
            }
        }
    }
    Some(req)
}

/// A tool use the answer asks for.
struct ToolUse {
    id: String,
    name: String,
    input: String,
}

/// What an answer says, wire-neutral.
#[derive(Default)]
struct Reply {
    id: Option<String>,
    model: Option<String>,
    usage: Option<(s::Usage, i64)>,
    uses: Vec<ToolUse>,
    /// The answer ended its turn (no tool use pending).
    ends: bool,
}

fn usage_openai(u: &Value) -> (s::Usage, i64) {
    let g = |k: &str| u.get(k).and_then(Value::as_i64).unwrap_or(0);
    let input = g("prompt_tokens").max(g("input_tokens"));
    let cached = u
        .get("prompt_tokens_details")
        .or_else(|| u.get("input_tokens_details"))
        .and_then(|d| d.get("cached_tokens"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let output = g("completion_tokens").max(g("output_tokens"));
    let usage = s::Usage {
        uncached_input: (input - cached).max(0),
        cache_read: cached,
        cache_write_5m: 0,
        cache_write_1h: 0,
        output,
    };
    (usage, input)
}

fn parse(bytes: &[u8]) -> Option<Value> {
    serde_json::from_slice(bytes).ok()
}

fn reply_messages(answer: &Answer) -> Reply {
    let mut r = Reply::default();
    let message = |r: &mut Reply, m: &Value| {
        r.id = m.get("id").and_then(Value::as_str).map(str::to_owned);
        r.model = m.get("model").and_then(Value::as_str).map(str::to_owned);
        if let Some(u) = m.get("usage") {
            r.usage = Some(crate::claude::usage_of(u));
        }
    };
    match answer {
        Answer::Body(b) => {
            let Some(m) = parse(b) else { return r };
            message(&mut r, &m);
            for b in m
                .get("content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if b.get("type").and_then(Value::as_str) == Some("tool_use") {
                    r.uses.push(ToolUse {
                        id: b.get("id").and_then(Value::as_str).unwrap_or("").to_owned(),
                        name: b
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_owned(),
                        input: canonical(b.get("input").unwrap_or(&Value::Null)),
                    });
                }
            }
            r.ends = m.get("stop_reason").and_then(Value::as_str) != Some("tool_use");
        }
        Answer::Events(events) => {
            // Tool uses by block index: id, name, the input's partial JSON; a block counts once
            // its stop arrived.
            let mut open: HashMap<i64, (String, String, String)> = HashMap::new();
            let mut raw_usage = serde_json::Map::new();
            for e in events.iter().filter_map(|e| parse(e)) {
                match e.get("type").and_then(Value::as_str) {
                    Some("message_start") => {
                        let m = e.get("message").unwrap_or(&Value::Null);
                        message(&mut r, m);
                        if let Some(Value::Object(u)) = m.get("usage") {
                            raw_usage.extend(u.clone());
                        }
                    }
                    Some("content_block_start") => {
                        let b = e.get("content_block").unwrap_or(&Value::Null);
                        if b.get("type").and_then(Value::as_str) == Some("tool_use") {
                            let i = e.get("index").and_then(Value::as_i64).unwrap_or(-1);
                            open.insert(
                                i,
                                (
                                    b.get("id").and_then(Value::as_str).unwrap_or("").to_owned(),
                                    b.get("name")
                                        .and_then(Value::as_str)
                                        .unwrap_or("")
                                        .to_owned(),
                                    String::new(),
                                ),
                            );
                        }
                    }
                    Some("content_block_delta") => {
                        let i = e.get("index").and_then(Value::as_i64).unwrap_or(-1);
                        if let (Some(t), Some(p)) = (
                            open.get_mut(&i),
                            e.get("delta")
                                .and_then(|d| d.get("partial_json"))
                                .and_then(Value::as_str),
                        ) {
                            t.2.push_str(p);
                        }
                    }
                    Some("content_block_stop") => {
                        let i = e.get("index").and_then(Value::as_i64).unwrap_or(-1);
                        if let Some((id, name, json)) = open.remove(&i) {
                            let input = parse(json.as_bytes()).unwrap_or(Value::Null);
                            r.uses.push(ToolUse {
                                id,
                                name,
                                input: canonical(&input),
                            });
                        }
                    }
                    Some("message_delta") => {
                        if let Some(Value::Object(u)) = e.get("usage") {
                            for (k, x) in u {
                                if !x.is_null() {
                                    raw_usage.insert(k.clone(), x.clone());
                                }
                            }
                        }
                        r.ends = e
                            .get("delta")
                            .and_then(|d| d.get("stop_reason"))
                            .and_then(Value::as_str)
                            != Some("tool_use");
                    }
                    _ => {}
                }
            }
            if !raw_usage.is_empty() {
                r.usage = Some(crate::claude::usage_of(&Value::Object(raw_usage)));
            }
        }
    }
    r
}

fn reply_chat(answer: &Answer) -> Reply {
    let mut r = Reply::default();
    let mut calls: Vec<(String, String, String)> = Vec::new();
    let mut finish: Option<String> = None;
    let chunks: Vec<Value> = match answer {
        Answer::Body(b) => parse(b).into_iter().collect(),
        Answer::Events(es) => es.iter().filter_map(|e| parse(e)).collect(),
    };
    for c in &chunks {
        if r.id.is_none() {
            r.id = c.get("id").and_then(Value::as_str).map(str::to_owned);
        }
        if r.model.is_none() {
            r.model = c.get("model").and_then(Value::as_str).map(str::to_owned);
        }
        if let Some(u) = c.get("usage").filter(|u| !u.is_null()) {
            r.usage = Some(usage_openai(u));
        }
        for ch in c
            .get("choices")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(f) = ch.get("finish_reason").and_then(Value::as_str) {
                finish = Some(f.to_owned());
            }
            let m = ch.get("message").or_else(|| ch.get("delta"));
            for t in m
                .and_then(|m| m.get("tool_calls"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let i = t
                    .get("index")
                    .and_then(Value::as_u64)
                    .map_or(calls.len(), |i| i as usize);
                if calls.len() <= i {
                    calls.resize(i + 1, Default::default());
                }
                let slot = &mut calls[i];
                if let Some(id) = t.get("id").and_then(Value::as_str) {
                    slot.0 = id.to_owned();
                }
                let f = t.get("function");
                if let Some(n) = f.and_then(|f| f.get("name")).and_then(Value::as_str) {
                    slot.1.push_str(n);
                }
                if let Some(a) = f.and_then(|f| f.get("arguments")).and_then(Value::as_str) {
                    slot.2.push_str(a);
                }
            }
        }
    }
    for (id, name, args) in calls.into_iter().filter(|c| !c.0.is_empty()) {
        let input = parse(args.as_bytes()).unwrap_or(Value::Null);
        r.uses.push(ToolUse {
            id,
            name,
            input: canonical(&input),
        });
    }
    r.ends = finish.as_deref() != Some("tool_calls");
    r
}

fn reply_responses(answer: &Answer) -> Reply {
    let mut r = Reply::default();
    let mut response: Option<Value> = None;
    let mut items: Vec<Value> = Vec::new();
    match answer {
        Answer::Body(b) => response = parse(b),
        Answer::Events(es) => {
            for e in es.iter().filter_map(|e| parse(e)) {
                match e.get("type").and_then(Value::as_str) {
                    Some("response.created" | "response.completed" | "response.incomplete") => {
                        response = e.get("response").cloned();
                    }
                    Some("response.output_item.done") => {
                        if let Some(i) = e.get("item") {
                            items.push(i.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(resp) = &response {
        r.id = resp.get("id").and_then(Value::as_str).map(str::to_owned);
        r.model = resp.get("model").and_then(Value::as_str).map(str::to_owned);
        if let Some(u) = resp.get("usage").filter(|u| !u.is_null()) {
            r.usage = Some(usage_openai(u));
        }
        if items.is_empty() {
            items = resp
                .get("output")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
        }
    }
    for i in &items {
        if i.get("type").and_then(Value::as_str) == Some("function_call") {
            let args = i.get("arguments").and_then(Value::as_str).unwrap_or("");
            r.uses.push(ToolUse {
                id: i
                    .get("call_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned(),
                name: i
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned(),
                input: canonical(&parse(args.as_bytes()).unwrap_or(Value::Null)),
            });
        }
    }
    r.ends = r.uses.is_empty();
    r
}

/// The string values of a tool input, each the text a spawned agent may start from.
fn strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(t) => out.push(t.clone()),
        Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
        Value::Object(m) => m.values().for_each(|x| strings(x, out)),
        _ => {}
    }
}

struct Out<'a> {
    session: &'a mut WireSession,
    steps: Vec<Step>,
}

impl Out<'_> {
    fn push(&mut self, at_ms: i64, actor: Actor, cmd: Cmd) {
        let order_ms = at_ms.max(self.session.clock);
        self.session.clock = order_ms;
        self.steps.push(Step {
            at_ms,
            order_ms,
            actor,
            source: s::Source::Wire,
            cmd,
        });
    }
}

/// The IR commands one finished call adds to its session. `count_tokens` and a request that is
/// not JSON of the named wire lift to nothing.
pub fn lift_call(state: &mut WireSession, call: &WireCall) -> Vec<Step> {
    let f = &call.facts;
    if f.path.ends_with("/count_tokens") {
        state.skipped += 1;
        return Vec::new();
    }
    let Some(body) = parse(&call.request) else {
        state.skipped += 1;
        return Vec::new();
    };
    let Some(req) = request_of(call.wire, &body) else {
        state.skipped += 1;
        return Vec::new();
    };
    let reply = match call.wire {
        Wire::Messages => reply_messages(&call.answer),
        Wire::Chat => reply_chat(&call.answer),
        Wire::Responses => reply_responses(&call.answer),
    };
    state.lifted += 1;
    let (start, end) = (f.started_ms, f.finished_ms.max(f.started_ms));
    let session_id = s::SessionId(f.session_hash.clone());
    let mut out = Out {
        session: state,
        steps: Vec::new(),
    };
    if !out.session.opened {
        out.session.opened = true;
        out.push(
            start,
            Actor::HarnessRuntime,
            Cmd::OpenSession(s::OpenSession {
                session_id: session_id.clone(),
                harness: f.harness,
                source: Some(s::Source::Wire),
                session_hash: Some(f.session_hash.clone()),
                at: ts(start),
            }),
        );
    }

    // Which agent: a side call, a chain the server holds, or the conversation's first message.
    let side = !req.tools && req.history <= 1 && req.previous_response.is_none();
    let chain: Option<String> = if side {
        None
    } else if let Some(prev) = &req.previous_response {
        out.session.by_response.get(prev).cloned()
    } else if let Some(conv) = &req.conversation {
        out.session.by_conversation.get(conv).cloned()
    } else {
        req.first.as_ref().map(|t| digest_hex(t.as_bytes()))
    };
    let chain = match (side, chain) {
        (true, _) => "side".to_owned(),
        (false, Some(c)) => c,
        // A server-held chain the session never saw: a chain of its own, keyed by the id.
        (false, None) => digest_hex(
            req.previous_response
                .as_deref()
                .or(req.conversation.as_deref())
                .unwrap_or("")
                .as_bytes(),
        ),
    };
    if let Some(conv) = &req.conversation {
        out.session
            .by_conversation
            .insert_new(conv, || chain.clone());
    }
    let known = out.session.by_first.contains_key(&chain);
    if !known {
        let id = if side {
            format!("{}-side", f.session_hash)
        } else {
            format!("w-{}", &chain[..12])
        };
        // A spawned agent starts from a string its parent's spawn call carried.
        let link = req
            .first_texts
            .iter()
            .find_map(|t| out.session.spawns.get(&digest_hex(t.as_bytes())).cloned());
        let (role, parent, spawned_by) = if side {
            (s::AgentRole::SideCall, out.session.main.clone(), None)
        } else if let Some((p, call)) = &link {
            (s::AgentRole::SubAgent, Some(p.clone()), Some(call.clone()))
        } else if out.session.main.is_none() {
            (s::AgentRole::Main, None, None)
        } else {
            (s::AgentRole::SubAgent, None, None)
        };
        if role == s::AgentRole::Main {
            out.session.main = Some(id.clone());
        }
        if side {
            out.session.side = Some(id.clone());
        }
        out.push(
            start,
            Actor::HarnessRuntime,
            Cmd::StartAgent(s::StartAgent {
                agent_id: s::AgentId(id.clone()),
                session_id: session_id.clone(),
                parent_agent_id: parent.map(s::AgentId),
                role,
                spawned_by: spawned_by.map(s::ToolCallId),
                at: ts(start),
            }),
        );
        out.session.by_first.insert(
            chain.clone(),
            AgentState {
                id,
                idle: true,
                open: HashSet::new(),
            },
        );
    }
    let agent = out.session.by_first[&chain].id.clone();
    let agent_id = s::AgentId(agent.clone());

    // The request: results of the agent's open tool calls, then a prompt.
    let results: Vec<&ToolResult> = req
        .results
        .iter()
        .filter(|r| out.session.by_first[&chain].open.contains(&r.id))
        .collect();
    for r in &results {
        out.push(
            start,
            Actor::ToolProvider,
            Cmd::CompleteToolCall(s::CompleteToolCall {
                tool_call_id: s::ToolCallId(r.id.clone()),
                ok: r.ok,
                output: content(&r.output),
                at: ts(start),
            }),
        );
        if let Some(a) = out.session.by_first.get_mut(&chain) {
            a.open.remove(&r.id);
        }
    }
    let idle = out.session.by_first[&chain].idle;
    if idle && !req.last_texts.is_empty() {
        let text = req.last_texts.join("\n");
        let author = if side {
            s::PromptAuthor::Harness
        } else if !known && role_of(out.session, &agent) == Some(s::AgentRole::SubAgent) {
            s::PromptAuthor::ParentAgent
        } else {
            s::PromptAuthor::Human
        };
        out.push(
            start,
            if author == s::PromptAuthor::Human {
                Actor::Human
            } else {
                Actor::HarnessRuntime
            },
            Cmd::SubmitPrompt(s::SubmitPrompt {
                agent_id: agent_id.clone(),
                author,
                attribution: s::Attribution::Inferred,
                class: prompt_class(&text, author),
                content: content(text.as_bytes()),
                at: ts(start),
            }),
        );
        if let Some(a) = out.session.by_first.get_mut(&chain) {
            a.idle = false;
        }
    }

    // The answer: the model turn with the gateway's facts, then each tool use.
    let (usage, context) = reply.usage.clone().unwrap_or((
        s::Usage {
            uncached_input: 0,
            cache_read: 0,
            cache_write_5m: 0,
            cache_write_1h: 0,
            output: 0,
        },
        0,
    ));
    out.push(
        end,
        Actor::AgentLoop,
        Cmd::RecordModelTurn(s::RecordModelTurn {
            agent_id: agent_id.clone(),
            model: reply.model.clone().unwrap_or(req.model.clone()),
            usage,
            context_tokens: context,
            message_id: reply.id.clone(),
            call_id: Some(f.call_id.clone()),
            route: f.route.clone(),
            upstream: f.upstream.clone(),
            attempts: Some(f.attempts),
            queue_wait_ms: f.queue_wait_ms,
            ttft_ms: f.ttft_ms,
            declared_cost_microdollars: f.declared_cost_microdollars,
            incomplete: f.incomplete,
            at: ts(end),
        }),
    );
    if let Some(id) = &reply.id {
        out.session.by_response.insert(id.clone(), chain.clone());
    }
    if f.incomplete.is_none() {
        for u in &reply.uses {
            let cat = category(&u.name);
            out.push(
                end,
                Actor::AgentLoop,
                Cmd::RequestToolCall(s::RequestToolCall {
                    tool_call_id: s::ToolCallId(u.id.clone()),
                    agent_id: agent_id.clone(),
                    tool: u.name.clone(),
                    category: cat,
                    input: content(u.input.as_bytes()),
                    cwd: None,
                    background: false,
                    at: ts(end),
                }),
            );
            out.push(
                end,
                Actor::PermissionAuthority,
                Cmd::DecideToolCall(s::DecideToolCall {
                    tool_call_id: s::ToolCallId(u.id.clone()),
                    decision: s::Decision::Accept,
                    decider: s::Decider::NotRecorded,
                    reason: None,
                    at: ts(end),
                }),
            );
            if cat == s::ToolCategory::Spawn {
                let mut found = Vec::new();
                strings(
                    &parse(u.input.as_bytes()).unwrap_or(Value::Null),
                    &mut found,
                );
                for t in found {
                    out.session
                        .spawns
                        .insert(digest_hex(t.as_bytes()), (agent.clone(), u.id.clone()));
                }
            }
            if let Some(a) = out.session.by_first.get_mut(&chain) {
                a.open.insert(u.id.clone());
            }
        }
    }
    let working = !out.session.by_first[&chain].idle;
    if working && (reply.ends || f.incomplete.is_some() || reply.uses.is_empty()) {
        out.push(
            end,
            Actor::AgentLoop,
            Cmd::EndTurn(s::EndTurn {
                agent_id,
                at: ts(end),
            }),
        );
        if let Some(a) = out.session.by_first.get_mut(&chain) {
            a.idle = true;
        }
    }
    out.steps
}

fn role_of(session: &WireSession, agent: &str) -> Option<s::AgentRole> {
    if session.main.as_deref() == Some(agent) {
        Some(s::AgentRole::Main)
    } else if session.side.as_deref() == Some(agent) {
        Some(s::AgentRole::SideCall)
    } else {
        Some(s::AgentRole::SubAgent)
    }
}

/// Adds the gateway's facts of each wire model turn to the disk model turn with the same message
/// id. Returns how many disk turns were joined.
pub fn join(disk: &mut [Step], wire: &[Step]) -> usize {
    let by_id: HashMap<&str, &s::RecordModelTurn> = wire
        .iter()
        .filter_map(|st| match &st.cmd {
            Cmd::RecordModelTurn(t) => t.message_id.as_deref().map(|id| (id, t)),
            _ => None,
        })
        .collect();
    let mut joined = 0;
    for st in disk.iter_mut() {
        if let Cmd::RecordModelTurn(t) = &mut st.cmd
            && let Some(w) = t.message_id.as_deref().and_then(|id| by_id.get(id))
        {
            t.call_id = w.call_id.clone();
            t.route = w.route.clone();
            t.upstream = w.upstream.clone();
            t.attempts = w.attempts;
            t.queue_wait_ms = w.queue_wait_ms;
            t.ttft_ms = w.ttft_ms;
            t.declared_cost_microdollars = w.declared_cost_microdollars;
            t.incomplete = w.incomplete;
            joined += 1;
        }
    }
    joined
}
