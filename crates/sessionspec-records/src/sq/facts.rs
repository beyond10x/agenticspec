//! The fact base of one session: the relations sq queries read, built from the lifted commands
//! and the outcome replay gave each. Only commands the session IR accepted become facts; a refused
//! one is a `refused_cmd` fact. No text is held: content is a digest and a length.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use sessionspec_types::session as s;

use crate::claude::{Actor, Cmd, Lifted, Step};
use crate::replay::Replayed;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Val {
    Null,
    Bool(bool),
    Int(i64),
    Str(Arc<str>),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Null => f.write_str("null"),
            Val::Bool(b) => write!(f, "{b}"),
            Val::Int(n) => write!(f, "{n}"),
            Val::Str(s) => f.write_str(s),
        }
    }
}

pub fn vs(x: &str) -> Val {
    Val::Str(Arc::from(x))
}

/// Why a fact, or one column of it, is only possible. `col` is `None` for the whole row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unk {
    pub col: Option<usize>,
    pub site: Arc<str>,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub vals: Vec<Val>,
    pub unk: Vec<Unk>,
}

#[derive(Debug, Default)]
pub struct Table {
    pub cols: Vec<&'static str>,
    pub rows: Vec<Row>,
    index: RefCell<HashMap<usize, HashMap<Val, Vec<usize>>>>,
}

impl Table {
    pub fn new(cols: Vec<&'static str>) -> Table {
        Table {
            cols,
            rows: Vec::new(),
            index: RefCell::new(HashMap::new()),
        }
    }

    pub fn col(&self, name: &str) -> Option<usize> {
        self.cols.iter().position(|c| *c == name)
    }

    pub fn push(&mut self, vals: Vec<Val>, unk: Vec<Unk>) {
        debug_assert_eq!(vals.len(), self.cols.len());
        self.rows.push(Row { vals, unk });
        self.index.borrow_mut().clear();
    }

    /// The rows whose column `col` equals `v`.
    pub fn lookup(&self, col: usize, v: &Val) -> Vec<usize> {
        let mut idx = self.index.borrow_mut();
        let by = idx.entry(col).or_insert_with(|| {
            let mut m: HashMap<Val, Vec<usize>> = HashMap::new();
            for (i, r) in self.rows.iter().enumerate() {
                m.entry(r.vals[col].clone()).or_default().push(i);
            }
            m
        });
        by.get(v).cloned().unwrap_or_default()
    }
}

/// Where a node sits: its agent and its position in the agent's and the session's order.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub agent: Arc<str>,
    pub agent_pos: usize,
    pub session_pos: usize,
    pub at: i64,
}

#[derive(Default)]
pub struct FactBase {
    pub session: String,
    pub tables: HashMap<&'static str, Table>,
    pub nodes: HashMap<Arc<str>, NodeInfo>,
    /// Nodes in order, per agent and for the whole session. Redactor commands are not in either.
    pub agent_order: HashMap<Arc<str>, Vec<Arc<str>>>,
    pub session_order: Vec<Arc<str>>,
    /// The shellspec shape of each shell call, by node.
    pub shapes: HashMap<Arc<str>, String>,
}

/// Every relation of the fact base and its columns, in the order `sessionspec query --schema`
/// would print them and positional atoms match them.
pub const SCHEMA: &[(&str, &[&str])] = &[
    (
        "session",
        &["id", "project", "harness", "opened_at", "ended_at"],
    ),
    (
        "agent",
        &[
            "id",
            "session",
            "parent",
            "role",
            "depth",
            "spawned_by",
            "started_at",
            "ended_at",
        ],
    ),
    (
        "cmd",
        &[
            "node", "session", "agent", "seq", "at", "actor", "kind", "outcome",
        ],
    ),
    (
        "prompt",
        &[
            "node",
            "agent",
            "author",
            "attribution",
            "class",
            "bytes",
            "digest",
            "mid_turn",
        ],
    ),
    (
        "call",
        &[
            "node",
            "id",
            "agent",
            "tool",
            "category",
            "bytes",
            "digest",
            "cwd",
            "background",
        ],
    ),
    (
        "decided",
        &["node", "call", "decision", "decider", "reason"],
    ),
    (
        "completed",
        &["node", "call", "ok", "bytes", "digest", "dur"],
    ),
    (
        "turn",
        &[
            "node",
            "agent",
            "model",
            "ctx",
            "uncached",
            "cache_read",
            "write_5m",
            "write_1h",
            "output",
        ],
    ),
    ("turn_end", &["node", "agent"]),
    ("compacted", &["node", "agent", "before"]),
    (
        "exposure",
        &[
            "node", "id", "agent", "channel", "class", "kind", "alias", "verdict", "call",
        ],
    ),
    ("refused_cmd", &["node", "error"]),
    ("at", &["node", "t"]),
    ("next", &["a", "b"]),
    ("turn_of", &["node", "prompt"]),
    (
        "interval",
        &["agent", "state", "on", "from", "to", "dur", "open"],
    ),
    ("bust", &["node", "agent", "cause", "written"]),
    ("spawned", &["call", "child"]),
];

pub fn schema_text() -> String {
    let mut out = String::new();
    for (name, cols) in SCHEMA {
        out.push_str(&format!("{name}{{{}}}\n", cols.join(", ")));
    }
    out
}

fn dbg<T: std::fmt::Debug>(x: T) -> Val {
    let d = format!("{x:?}");
    vs(d.split([' ', '{', '(']).next().unwrap_or(""))
}

fn opt(x: Option<&str>) -> Val {
    x.map(vs).unwrap_or(Val::Null)
}

const MINUTE: i64 = 60_000;
const BUST_FLOOR: i64 = 20_000;

/// An agent as started: id, parent, role, the spawning call, start instant.
type AgentRow = (Arc<str>, Option<String>, String, Option<String>, i64);

#[derive(Default)]
struct AgentState {
    working: bool,
    turn_prompt: Option<Arc<str>>,
    idle_from: Option<i64>,
    work_from: Option<i64>,
    prev_turn: Option<(i64, i64, String, bool)>,
    compacted: bool,
    last_node: Option<Arc<str>>,
}

pub fn build(lifted: &Lifted, replayed: &Replayed, project: &str) -> FactBase {
    let mut fb = FactBase {
        session: lifted.session_id.clone(),
        ..FactBase::default()
    };
    for (name, cols) in SCHEMA {
        fb.tables.insert(name, Table::new(cols.to_vec()));
    }
    let sid = vs(&lifted.session_id);
    let push = |fb: &mut FactBase, rel: &'static str, vals: Vec<Val>, unk: Vec<Unk>| {
        if let Some(t) = fb.tables.get_mut(rel) {
            t.push(vals, unk);
        }
    };

    let mut agents: HashMap<Arc<str>, AgentState> = HashMap::new();
    let mut agent_rows: Vec<AgentRow> = Vec::new();
    let mut agent_last: HashMap<Arc<str>, i64> = HashMap::new();
    let mut call_agent: HashMap<String, Arc<str>> = HashMap::new();
    let mut call_at: HashMap<String, i64> = HashMap::new();
    let mut spawn_calls: Vec<(Arc<str>, Arc<str>, i64, String)> = Vec::new();
    let mut first_at = i64::MAX;
    let mut last_at = i64::MIN;

    for (seq, (step, logged)) in lifted.steps.iter().zip(&replayed.log).enumerate() {
        let Step {
            at_ms, actor, cmd, ..
        } = step;
        let at = *at_ms;
        first_at = first_at.min(at);
        last_at = last_at.max(at);
        let node: Arc<str> = Arc::from(format!("{}#{seq}", lifted.session_id));
        let agent: Option<Arc<str>> = match cmd {
            Cmd::OpenSession(_) => None,
            Cmd::StartAgent(x) => Some(Arc::from(x.agent_id.0.as_str())),
            Cmd::SubmitPrompt(x) => Some(Arc::from(x.agent_id.0.as_str())),
            Cmd::EndTurn(x) => Some(Arc::from(x.agent_id.0.as_str())),
            Cmd::RecordModelTurn(x) => Some(Arc::from(x.agent_id.0.as_str())),
            Cmd::CompactContext(x) => Some(Arc::from(x.agent_id.0.as_str())),
            Cmd::RequestToolCall(x) => Some(Arc::from(x.agent_id.0.as_str())),
            Cmd::DecideToolCall(x) => call_agent.get(&x.tool_call_id.0).cloned(),
            Cmd::CompleteToolCall(x) => call_agent.get(&x.tool_call_id.0).cloned(),
            Cmd::RecordExposure(x) => Some(Arc::from(x.agent_id.0.as_str())),
        };
        let kind = crate::replay::command_name(cmd);
        push(
            &mut fb,
            "cmd",
            vec![
                Val::Str(node.clone()),
                sid.clone(),
                agent.clone().map(Val::Str).unwrap_or(Val::Null),
                Val::Int(seq as i64),
                Val::Int(at),
                dbg(actor),
                vs(kind),
                vs(&logged.outcome),
            ],
            vec![],
        );
        push(
            &mut fb,
            "at",
            vec![Val::Str(node.clone()), Val::Int(at)],
            vec![],
        );
        if logged.refused {
            push(
                &mut fb,
                "refused_cmd",
                vec![Val::Str(node.clone()), vs(&logged.outcome)],
                vec![],
            );
            continue;
        }
        if let Some(a) = &agent
            && *actor != Actor::Redactor
        {
            let st = agents.entry(a.clone()).or_default();
            if let Some(prev) = st.last_node.replace(node.clone()) {
                push(
                    &mut fb,
                    "next",
                    vec![Val::Str(prev), Val::Str(node.clone())],
                    vec![],
                );
            }
            let order = fb.agent_order.entry(a.clone()).or_default();
            fb.nodes.insert(
                node.clone(),
                NodeInfo {
                    agent: a.clone(),
                    agent_pos: order.len(),
                    session_pos: fb.session_order.len(),
                    at,
                },
            );
            order.push(node.clone());
            fb.session_order.push(node.clone());
            agent_last.insert(a.clone(), at);
            if let Some(p) = &st.turn_prompt {
                push(
                    &mut fb,
                    "turn_of",
                    vec![Val::Str(node.clone()), Val::Str(p.clone())],
                    vec![],
                );
            }
        }
        let n = Val::Str(node.clone());
        let a = agent.clone().map(Val::Str).unwrap_or(Val::Null);
        let site = |why: &str| Arc::from(format!("{node} ({why})"));
        match cmd {
            Cmd::OpenSession(_) => {}
            Cmd::StartAgent(x) => {
                let id: Arc<str> = Arc::from(x.agent_id.0.as_str());
                agent_rows.push((
                    id.clone(),
                    x.parent_agent_id.as_ref().map(|p| p.0.clone()),
                    format!("{:?}", x.role),
                    x.spawned_by.as_ref().map(|c| c.0.clone()),
                    at,
                ));
                let st = agents.entry(id).or_default();
                st.idle_from = Some(at);
            }
            Cmd::SubmitPrompt(x) => {
                let st = agents
                    .entry(agent.clone().unwrap_or_else(|| Arc::from("")))
                    .or_default();
                let mid_turn = st.working;
                let mut unk = Vec::new();
                if x.attribution != s::Attribution::Known {
                    unk.push(Unk {
                        col: Some(2),
                        site: site("author inferred"),
                    });
                }
                let on = match x.author {
                    s::PromptAuthor::Human | s::PromptAuthor::Delegate => "human",
                    s::PromptAuthor::PeerSession => "peer",
                    s::PromptAuthor::ParentAgent => "parent",
                    s::PromptAuthor::Harness => "harness",
                    s::PromptAuthor::Hook => "hook",
                };
                if let Some(from) = st.idle_from.take() {
                    let iunk = if x.attribution != s::Attribution::Known {
                        vec![Unk {
                            col: Some(2),
                            site: site("author inferred"),
                        }]
                    } else {
                        vec![]
                    };
                    let row = vec![
                        a.clone(),
                        vs("Idle"),
                        vs(on),
                        Val::Int(from),
                        Val::Int(at),
                        Val::Int((at - from).max(0)),
                        Val::Bool(false),
                    ];
                    if let Some(t) = fb.tables.get_mut("interval") {
                        t.push(row, iunk);
                    }
                }
                if !st.working {
                    st.work_from = Some(at);
                    st.turn_prompt = Some(node.clone());
                }
                st.working = true;
                push(
                    &mut fb,
                    "prompt",
                    vec![
                        n.clone(),
                        a.clone(),
                        dbg(x.author),
                        dbg(x.attribution),
                        dbg(x.class),
                        Val::Int(x.content.bytes),
                        vs(&x.content.digest),
                        Val::Bool(mid_turn),
                    ],
                    unk,
                );
            }
            Cmd::EndTurn(_) => {
                let st = agents
                    .entry(agent.clone().unwrap_or_else(|| Arc::from("")))
                    .or_default();
                if let Some(from) = st.work_from.take() {
                    let row = vec![
                        a.clone(),
                        vs("Working"),
                        vs("none"),
                        Val::Int(from),
                        Val::Int(at),
                        Val::Int((at - from).max(0)),
                        Val::Bool(false),
                    ];
                    if let Some(t) = fb.tables.get_mut("interval") {
                        t.push(row, vec![]);
                    }
                }
                st.working = false;
                st.turn_prompt = None;
                st.idle_from = Some(at);
                push(&mut fb, "turn_end", vec![n.clone(), a.clone()], vec![]);
            }
            Cmd::RecordModelTurn(x) => {
                let u = &x.usage;
                push(
                    &mut fb,
                    "turn",
                    vec![
                        n.clone(),
                        a.clone(),
                        vs(&x.model),
                        Val::Int(x.context_tokens),
                        Val::Int(u.uncached_input),
                        Val::Int(u.cache_read),
                        Val::Int(u.cache_write_5m),
                        Val::Int(u.cache_write_1h),
                        Val::Int(u.output),
                    ],
                    vec![],
                );
                let st = agents
                    .entry(agent.clone().unwrap_or_else(|| Arc::from("")))
                    .or_default();
                let write = u.cache_write_5m + u.cache_write_1h;
                if let Some((pctx, pt, pmodel, p1h)) = &st.prev_turn
                    && *pctx > BUST_FLOOR
                    && u.cache_read * 2 < *pctx
                    && write * 2 > *pctx
                {
                    let ttl = if *p1h { 60 * MINUTE } else { 5 * MINUTE };
                    let cause = if st.compacted {
                        "Compacted"
                    } else if *pmodel != x.model {
                        "ModelChanged"
                    } else if at - pt > ttl {
                        "TtlExpired"
                    } else {
                        "Unexplained"
                    };
                    let row = vec![n.clone(), a.clone(), vs(cause), Val::Int(write)];
                    if let Some(t) = fb.tables.get_mut("bust") {
                        t.push(row, vec![]);
                    }
                }
                st.compacted = false;
                st.prev_turn = Some((
                    x.context_tokens,
                    at,
                    x.model.clone(),
                    u.cache_write_1h > u.cache_write_5m,
                ));
            }
            Cmd::CompactContext(x) => {
                if let Some(st) = agent.as_ref().and_then(|g| agents.get_mut(g)) {
                    st.compacted = true;
                }
                push(
                    &mut fb,
                    "compacted",
                    vec![n.clone(), a.clone(), Val::Int(x.context_tokens_before)],
                    vec![],
                );
            }
            Cmd::RequestToolCall(x) => {
                if let Some(ag) = &agent {
                    call_agent.insert(x.tool_call_id.0.clone(), ag.clone());
                    if x.category == s::ToolCategory::Spawn {
                        spawn_calls.push((node.clone(), ag.clone(), at, x.tool_call_id.0.clone()));
                    }
                }
                call_at.insert(x.tool_call_id.0.clone(), at);
                push(
                    &mut fb,
                    "call",
                    vec![
                        n.clone(),
                        vs(&x.tool_call_id.0),
                        a.clone(),
                        vs(&x.tool),
                        dbg(x.category),
                        Val::Int(x.input.bytes),
                        vs(&x.input.digest),
                        opt(x.cwd.as_deref()),
                        Val::Bool(x.background),
                    ],
                    vec![],
                );
            }
            Cmd::DecideToolCall(x) => {
                let unk = if x.decider == s::Decider::NotRecorded {
                    vec![
                        Unk {
                            col: Some(2),
                            site: site("decision not recorded"),
                        },
                        Unk {
                            col: Some(3),
                            site: site("decision not recorded"),
                        },
                    ]
                } else {
                    vec![]
                };
                push(
                    &mut fb,
                    "decided",
                    vec![
                        n.clone(),
                        vs(&x.tool_call_id.0),
                        dbg(x.decision),
                        dbg(x.decider),
                        opt(x.reason.as_deref()),
                    ],
                    unk,
                );
            }
            Cmd::CompleteToolCall(x) => {
                let dur = call_at
                    .get(&x.tool_call_id.0)
                    .map(|t| (at - t).max(0))
                    .unwrap_or(0);
                push(
                    &mut fb,
                    "completed",
                    vec![
                        n.clone(),
                        vs(&x.tool_call_id.0),
                        Val::Bool(x.ok),
                        Val::Int(x.output.bytes),
                        vs(&x.output.digest),
                        Val::Int(dur),
                    ],
                    vec![],
                );
            }
            Cmd::RecordExposure(x) => {
                push(
                    &mut fb,
                    "exposure",
                    vec![
                        n.clone(),
                        vs(&x.exposure_id.0),
                        a.clone(),
                        dbg(x.channel),
                        dbg(x.class),
                        vs(&x.kind),
                        vs(&x.alias),
                        dbg(x.verdict),
                        opt(x.tool_call_id.as_ref().map(|c| c.0.as_str())),
                    ],
                    vec![],
                );
            }
        }
    }

    // Intervals still open when the transcript ends.
    for (ag, st) in &agents {
        let end = agent_last.get(ag).copied().unwrap_or(last_at);
        let (state, from, on) = if st.working {
            ("Working", st.work_from, "none")
        } else {
            ("Idle", st.idle_from, "unk")
        };
        if let Some(from) = from {
            let unk = if state == "Idle" {
                vec![Unk {
                    col: Some(2),
                    site: Arc::from(format!("{}:{ag} (open interval)", lifted.session_id)),
                }]
            } else {
                vec![]
            };
            if let Some(t) = fb.tables.get_mut("interval") {
                t.push(
                    vec![
                        Val::Str(ag.clone()),
                        vs(state),
                        vs(on),
                        Val::Int(from),
                        Val::Int(end),
                        Val::Int((end - from).max(0)),
                        Val::Bool(true),
                    ],
                    unk,
                );
            }
        }
    }

    // Agents, their depth, and the spawn link: recorded, or inferred as the nearest earlier Spawn
    // call of the parent.
    let parent_of: HashMap<Arc<str>, Option<String>> = agent_rows
        .iter()
        .map(|(id, p, ..)| (id.clone(), p.clone()))
        .collect();
    for (id, parent, role, spawned_by, started) in &agent_rows {
        let mut depth = 0;
        let mut cur = parent.clone();
        while let Some(p) = cur {
            depth += 1;
            if depth > 32 {
                break;
            }
            cur = parent_of.get(p.as_str()).cloned().flatten();
        }
        let ended = agent_last.get(id).copied().unwrap_or(*started);
        push(
            &mut fb,
            "agent",
            vec![
                Val::Str(id.clone()),
                sid.clone(),
                opt(parent.as_deref()),
                vs(role),
                Val::Int(depth),
                opt(spawned_by.as_deref()),
                Val::Int(*started),
                Val::Int(ended),
            ],
            vec![],
        );
        match (spawned_by, parent) {
            (Some(call), _) => {
                if let Some((cnode, ..)) = spawn_calls.iter().find(|c| &c.3 == call) {
                    push(
                        &mut fb,
                        "spawned",
                        vec![Val::Str(cnode.clone()), Val::Str(id.clone())],
                        vec![],
                    );
                }
            }
            (None, Some(p)) => {
                if let Some((cnode, ..)) = spawn_calls
                    .iter()
                    .filter(|c| &*c.1 == p.as_str() && c.2 <= *started)
                    .max_by_key(|c| c.2)
                {
                    push(
                        &mut fb,
                        "spawned",
                        vec![Val::Str(cnode.clone()), Val::Str(id.clone())],
                        vec![Unk {
                            col: None,
                            site: Arc::from(format!("{cnode} (spawn link inferred)")),
                        }],
                    );
                }
            }
            _ => {}
        }
    }
    if first_at != i64::MAX {
        push(
            &mut fb,
            "session",
            vec![
                sid.clone(),
                vs(project),
                vs("ClaudeCode"),
                Val::Int(first_at),
                Val::Int(last_at),
            ],
            vec![],
        );
    }
    fb
}
