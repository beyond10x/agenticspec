//! The lift of one session into rows of the building blocks (`blocks/`), and the draft scenario
//! of its outside inputs. The rules are `queries/blocks.sq`, run by sq over the session's fact
//! base; this module turns their answers into rows `{ entity, id, state, fields, not_recorded,
//! evidence }`, the row form of a scenario's `given`. A field the session does not record is
//! named in `not_recorded`, never guessed. Identifiers are UUIDs derived from the session's node,
//! agent or party they stand for, so two lifts of one session write the same bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::sq::{self, FactBase, Program, facts::Val};
use crate::time::rfc3339;

/// The lift's rules.
pub const RULES: &str = include_str!("../../../queries/blocks.sq");

/// The block entities a session cannot show: no relation of the session IR records them, so the
/// lift writes no row of them.
pub const NOT_OBSERVABLE: &[&str] = &["agentic.resource.Claim"];

/// The entities the lift writes, in the order rows of one node are listed.
pub const ENTITIES: &[&str] = &[
    "agentic.work.Task",
    "agentic.work.Assignment",
    "agentic.work.Run",
    "agentic.question.Question",
    "agentic.channel.Message",
    "agentic.work.Step",
];

/// One row of a block entity, lifted from one node of the session.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockRow {
    pub entity: &'static str,
    pub id: String,
    pub state: Option<&'static str>,
    pub fields: Map<String, Value>,
    pub not_recorded: Vec<&'static str>,
    /// The node the row was lifted from, `<session>#<n>`.
    pub evidence: String,
    /// The session agent the row is about: the one that works a run, takes a step, asks a
    /// question, sends or receives a message.
    pub agent: String,
}

impl BlockRow {
    /// The row in the form of a scenario's `given.rows`.
    pub fn value(&self) -> Value {
        let mut m = Map::new();
        m.insert("entity".into(), json!(self.entity));
        m.insert("id".into(), json!(self.id));
        if let Some(s) = self.state {
            m.insert("state".into(), json!(s));
        }
        m.insert("fields".into(), Value::Object(self.fields.clone()));
        if !self.not_recorded.is_empty() {
            m.insert("not_recorded".into(), json!(self.not_recorded));
        }
        m.insert("evidence".into(), json!(self.evidence));
        Value::Object(m)
    }
}

/// One outside input of the session: a prompt of the main agent by a human, a delegate or
/// another session.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub node: String,
    /// `Human`, `Delegate` or `PeerSession`.
    pub author: String,
    pub at_ms: i64,
    /// The node of the turn that received it; `None` when no turn followed.
    pub turn: Option<String>,
}

/// The rows of one session and its outside inputs.
#[derive(Debug, Clone)]
pub struct Lift {
    pub session_id: String,
    /// The instant of the session's first node.
    pub opened_ms: i64,
    /// The instant of its last node.
    pub last_ms: i64,
    /// The id of the session's main agent.
    pub main_agent: String,
    pub rows: Vec<BlockRow>,
    pub inputs: Vec<Input>,
    pub not_observable: Vec<&'static str>,
}

/// The lift's rules, compiled.
pub fn program() -> Program {
    sq::compile(RULES).expect("queries/blocks.sq compiles")
}

/// Lifts one main transcript and its sub-agents.
pub fn lift_file(main: &Path) -> std::io::Result<Lift> {
    Ok(lift(&program(), &sq::fact_base(main)?))
}

/// A UUID that stands for `key` of `kind` (`task`, `session`, `party`, ...): the first 16 bytes
/// of a SHA-256, in the 8-4-4-4-12 form.
pub fn uid(kind: &str, key: &str) -> String {
    let mut h = Sha256::new();
    h.update(b"agenticspec.blocks\0");
    h.update(kind.as_bytes());
    h.update(b"\0");
    h.update(key.as_bytes());
    let d = h.finalize();
    let hex: String = d[..16].iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// The position of a node in its session, from `<session>#<n>`.
pub fn seq(node: &str) -> i64 {
    node.rsplit('#')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(-1)
}

fn text(v: &Val) -> String {
    match v {
        Val::Str(s) => s.to_string(),
        Val::Null => String::new(),
        other => other.to_string(),
    }
}

fn int(v: &Val) -> i64 {
    match v {
        Val::Int(n) => *n,
        Val::Str(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

/// The answers of derived relation `name`, sorted, each a list of values.
fn answers(p: &Program, a: &sq::Answers, name: &str) -> Vec<Vec<Val>> {
    let mut out: Vec<Vec<Val>> = p
        .derived
        .iter()
        .position(|d| d.name == name)
        .map(|i| a.derived[i].rows.iter().map(|(v, _)| v.clone()).collect())
        .unwrap_or_default();
    out.sort();
    out.dedup();
    out
}

fn content(bytes: &Val, digest: &Val, class: &str) -> Value {
    json!({ "digest": text(digest), "length": int(bytes), "class": class })
}

/// The rows and inputs of one session, from the answers of [`RULES`] over its fact base.
pub fn lift(p: &Program, fb: &FactBase) -> Lift {
    let a = sq::eval::run(p, fb);
    let s = fb.session.as_str();
    let session_party = |agent: &str| uid("session", &format!("{s}/{agent}"));
    let party = |author: &str| uid("party", &format!("{s}/{author}"));
    let mut rows = Vec::new();

    // Tasks, and who asked for each: the human of the session; a delegate's or another
    // session's request names no principal the record holds.
    let mut requester: BTreeMap<String, Option<String>> = BTreeMap::new();
    for v in answers(p, &a, "block_task") {
        let (node, agent, author) = (text(&v[0]), text(&v[1]), text(&v[2]));
        let mut fields = Map::new();
        fields.insert("goal".into(), content(&v[3], &v[4], "prompt"));
        let mut not_recorded = vec!["process_id"];
        let by = (author == "Human").then(|| party(&author));
        match &by {
            Some(id) => {
                fields.insert("requested_by".into(), json!(id));
            }
            None => not_recorded.push("requested_by"),
        }
        requester.insert(node.clone(), by);
        rows.push(BlockRow {
            entity: "agentic.work.Task",
            id: uid("task", &node),
            state: None,
            fields,
            not_recorded,
            evidence: node,
            agent,
        });
    }

    // Runs, each with the assignment that binds its agent to the task.
    let mut run_agent: BTreeMap<String, String> = BTreeMap::new();
    for v in answers(p, &a, "block_run") {
        let (node, agent, task, started) = (text(&v[0]), text(&v[1]), text(&v[2]), int(&v[3]));
        run_agent.insert(node.clone(), agent.clone());
        let mut fields = Map::new();
        fields.insert("task_id".into(), json!(uid("task", &task)));
        fields.insert("assigned_at".into(), json!(rfc3339(started)));
        let mut not_recorded = vec!["agent_revision_id"];
        match requester.get(&task).cloned().flatten() {
            Some(id) => {
                fields.insert("on_behalf_of".into(), json!(id));
            }
            None => not_recorded.push("on_behalf_of"),
        }
        rows.push(BlockRow {
            entity: "agentic.work.Assignment",
            id: uid("assignment", &node),
            state: None,
            fields,
            not_recorded,
            evidence: node.clone(),
            agent: agent.clone(),
        });
        let mut fields = Map::new();
        fields.insert("assignment_id".into(), json!(uid("assignment", &node)));
        fields.insert("harness_session_id".into(), json!(session_party(&agent)));
        fields.insert("started_at".into(), json!(rfc3339(started)));
        rows.push(BlockRow {
            entity: "agentic.work.Run",
            id: uid("run", &node),
            state: None,
            fields,
            not_recorded: Vec::new(),
            evidence: node,
            agent,
        });
    }

    // Steps: the run of each node, for the question rows too.
    let mut node_run: BTreeMap<String, String> = BTreeMap::new();
    for v in answers(p, &a, "block_step") {
        let (node, run, action, kind, result, at) = (
            text(&v[0]),
            text(&v[1]),
            text(&v[2]),
            text(&v[3]),
            text(&v[4]),
            int(&v[5]),
        );
        let agent = run_agent.get(&run).cloned().unwrap_or_default();
        node_run.insert(node.clone(), run.clone());
        let mut fields = Map::new();
        fields.insert("run_id".into(), json!(uid("run", &run)));
        fields.insert("action".into(), json!(action));
        fields.insert("kind".into(), json!(kind));
        fields.insert("harness_session_id".into(), json!(session_party(&agent)));
        fields.insert("recorded_at".into(), json!(rfc3339(at)));
        let mut not_recorded = Vec::new();
        if result == "NotRecorded" {
            not_recorded.push("result");
        } else {
            fields.insert("result".into(), json!(result));
        }
        rows.push(BlockRow {
            entity: "agentic.work.Step",
            id: uid("step", &node),
            state: Some("Recorded"),
            fields,
            not_recorded,
            evidence: node,
            agent,
        });
    }

    let answered: BTreeSet<String> = answers(p, &a, "block_answered")
        .iter()
        .map(|v| text(&v[0]))
        .collect();
    for v in answers(p, &a, "block_question") {
        let (node, agent, asked_by) = (text(&v[0]), text(&v[1]), text(&v[2]));
        let by_agent = asked_by == "Agent";
        let (from, to, class) = if by_agent {
            (session_party(&agent), party("Human"), "tool_input")
        } else {
            (party(&asked_by), session_party(&agent), "prompt")
        };
        let mut fields = Map::new();
        fields.insert("asked_by".into(), json!(from));
        fields.insert("addressee".into(), json!(to));
        fields.insert("text".into(), content(&v[3], &v[4], class));
        if let Some(run) = node_run.get(&node) {
            fields.insert("run_id".into(), json!(uid("run", run)));
        }
        let state = by_agent.then(|| {
            if answered.contains(&node) {
                "Answered"
            } else {
                "Open"
            }
        });
        rows.push(BlockRow {
            entity: "agentic.question.Question",
            id: uid("question", &node),
            state,
            fields,
            not_recorded: vec!["kind", "options", "default_option", "deadline"],
            evidence: node,
            agent,
        });
    }

    let received: BTreeMap<String, String> = answers(p, &a, "block_received")
        .iter()
        .map(|v| (text(&v[0]), text(&v[1])))
        .collect();
    for v in answers(p, &a, "block_message") {
        let (node, agent, from, call, at) = (
            text(&v[0]),
            text(&v[1]),
            text(&v[2]),
            text(&v[3]),
            int(&v[6]),
        );
        let mut fields = Map::new();
        let (state, not_recorded) = if from == "Agent" {
            fields.insert("from".into(), json!(session_party(&agent)));
            fields.insert("content".into(), content(&v[4], &v[5], "tool_input"));
            fields.insert("sent_at".into(), json!(rfc3339(at)));
            fields.insert(
                "sent_by_call".into(),
                json!(uid("call", &format!("{s}/{call}"))),
            );
            ("Sent", vec!["channel_id", "to", "act"])
        } else {
            fields.insert("from".into(), json!(party(&from)));
            fields.insert("to".into(), json!(session_party(&agent)));
            fields.insert("content".into(), content(&v[4], &v[5], "prompt"));
            fields.insert("read_at".into(), json!(rfc3339(at)));
            if let Some(turn) = received.get(&node) {
                fields.insert("received_in_turn".into(), json!(uid("turn", turn)));
            }
            ("Read", vec!["channel_id", "act", "sent_at"])
        };
        rows.push(BlockRow {
            entity: "agentic.channel.Message",
            id: uid("message", &node),
            state: Some(state),
            fields,
            not_recorded,
            evidence: node,
            agent,
        });
    }

    let rank = |e: &str| ENTITIES.iter().position(|x| *x == e).unwrap_or(usize::MAX);
    rows.sort_by(|x, y| {
        (seq(&x.evidence), rank(x.entity), &x.id).cmp(&(seq(&y.evidence), rank(y.entity), &y.id))
    });

    let mut inputs: Vec<Input> = answers(p, &a, "block_input")
        .iter()
        .map(|v| {
            let node = text(&v[0]);
            Input {
                turn: received.get(&node).cloned(),
                node,
                author: text(&v[2]),
                at_ms: int(&v[3]),
            }
        })
        .collect();
    inputs.sort_by_key(|i| seq(&i.node));

    let agents = &fb.tables["agent"];
    let (id, role) = (agents.col("id"), agents.col("role"));
    let main_agent = agents
        .rows
        .iter()
        .find(|r| role.is_some_and(|c| text(&r.vals[c]) == "Main"))
        .and_then(|r| id.map(|c| text(&r.vals[c])))
        .unwrap_or_default();
    let times: Vec<i64> = fb.nodes.values().map(|n| n.at).collect();
    Lift {
        session_id: fb.session.clone(),
        opened_ms: times.iter().copied().min().unwrap_or(0),
        last_ms: times.iter().copied().max().unwrap_or(0),
        main_agent,
        rows,
        inputs,
        not_observable: NOT_OBSERVABLE.to_vec(),
    }
}

/// What a draft scenario is written against: its name, the architecture and profile as
/// `(ref, digest)`, and the parts the session's agents and authors play.
#[derive(Debug, Clone)]
pub struct DraftArgs {
    pub name: String,
    pub architecture: (String, String),
    pub profile: (String, String),
    /// The border part the main agent plays; every input arrives there.
    pub main: String,
    /// The part every sub-agent plays.
    pub sub: Option<String>,
    /// The part that authors the human's prompts.
    pub human: Option<String>,
    /// The part that authors a delegate's prompts; the human's part when not given, as a
    /// delegate's message counts as its delegator's.
    pub delegate: Option<String>,
    /// The part that authors another session's messages.
    pub peer: Option<String>,
}

/// A draft scenario: its text, and what the extraction could not place.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    pub yaml: String,
    pub inputs: usize,
    pub expectations: usize,
    /// Inputs no turn followed, as `when.<n>` with their node: in the scenario, with no
    /// expectation.
    pub unjoined: Vec<String>,
    /// Inputs whose author plays no part, by node and author: left out of the scenario.
    pub unmapped: Vec<String>,
}

fn scalar(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
        && !s.starts_with(['-', '.'])
        && !matches!(s, "true" | "false" | "null" | "yes" | "no" | "on" | "off")
    {
        s.to_string()
    } else {
        serde_json::to_string(s).unwrap_or_default()
    }
}

fn flow(pairs: &[(&str, String)]) -> String {
    let inner: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{k}: {}", scalar(v)))
        .collect();
    format!("{{ {} }}", inner.join(", "))
}

fn seconds(ms: i64) -> String {
    format!("{}s", (ms.max(0) + 500) / 1000)
}

/// The draft scenario of a lifted session: each outside input whose author plays a part is a
/// `when` entry at the main agent's part; the rows of the turn that received it and of every
/// node after it up to the next input are its `then`, in block events, each naming the input's
/// task (`when.<n>`).
pub fn draft(lift: &Lift, args: &DraftArgs) -> Draft {
    let part_of = |agent: &str| -> Option<&str> {
        if agent == lift.main_agent {
            Some(args.main.as_str())
        } else {
            args.sub.as_deref()
        }
    };
    let author_part = |author: &str| -> Option<&str> {
        match author {
            "Human" => args.human.as_deref(),
            "Delegate" => args.delegate.as_deref().or(args.human.as_deref()),
            _ => args.peer.as_deref(),
        }
    };
    let mut when = Vec::new();
    let mut events = Vec::new();
    let mut unjoined = Vec::new();
    let mut unmapped = Vec::new();
    let mut prev = lift.opened_ms;
    for (k, input) in lift.inputs.iter().enumerate() {
        let Some(author) = author_part(&input.author) else {
            unmapped.push(format!("{} ({})", input.node, input.author));
            continue;
        };
        let task = format!("when.{}", when.len());
        let kind = if input.author == "Human" {
            "prompt"
        } else {
            "message"
        };
        let entry = flow(&[
            ("at", args.main.clone()),
            ("kind", kind.to_string()),
            ("author", author.to_string()),
            ("proof", "none".to_string()),
            ("after", seconds(input.at_ms - prev)),
        ]);
        prev = input.at_ms;
        let Some(turn) = &input.turn else {
            when.push(format!(
                "  # {}: no turn received it\n  - {entry}",
                input.node
            ));
            unjoined.push(format!("{task} ({})", input.node));
            continue;
        };
        when.push(format!(
            "  # {}: received in turn {turn}\n  - {entry}",
            input.node
        ));
        let from = seq(&input.node);
        let to = lift
            .inputs
            .get(k + 1)
            .map_or(i64::MAX, |next| seq(&next.node));
        for row in lift
            .rows
            .iter()
            .filter(|r| (from..to).contains(&seq(&r.evidence)))
        {
            let part = part_of(&row.agent).map(str::to_string);
            let mut push = |event: &str, part: Option<String>| {
                let mut pairs = vec![("event", event.to_string())];
                if let Some(p) = part {
                    pairs.push(("part", p));
                }
                pairs.push(("task", task.clone()));
                events.push(format!("    - {}  # {}", flow(&pairs), row.evidence));
            };
            match row.entity {
                "agentic.work.Task" => push("task_created", Some(args.main.clone())),
                "agentic.work.Run" => push("run_started", part),
                "agentic.question.Question" => {
                    push("question_asked", part);
                    if row.state == Some("Answered") {
                        push("question_answered", None);
                    }
                }
                "agentic.channel.Message"
                    if row.state == Some("Sent") && row.evidence != input.node =>
                {
                    push("post_sent", part)
                }
                _ => {}
            }
        }
    }
    let hours = (lift.last_ms - lift.opened_ms).max(0) / 3_600_000 + 1;
    let mut y = String::new();
    y.push_str("format: agenticspec-scenario/1\n");
    y.push_str(&format!("scenario: {}\n", scalar(&args.name)));
    y.push_str(&format!(
        "summary: {}\n",
        scalar(&format!(
            "Drafted by agenticspec blocks scenario from session {}: its outside inputs, and \
             the block events of the turns they started.",
            lift.session_id
        ))
    ));
    for (key, (r, d)) in [
        ("architecture", &args.architecture),
        ("profile", &args.profile),
    ] {
        y.push_str(&format!(
            "{key}: {}\n",
            flow(&[("ref", r.clone()), ("digest", d.clone())])
        ));
    }
    y.push_str(&format!("horizon: {hours}h\n\n"));
    if when.is_empty() {
        y.push_str("when: []\n");
    } else {
        y.push_str("when:\n");
        for w in &when {
            y.push_str(w);
            y.push('\n');
        }
    }
    if !events.is_empty() {
        y.push_str("\nthen:\n  order: sequence\n  events:\n");
        for e in &events {
            y.push_str(e);
            y.push('\n');
        }
    }
    Draft {
        yaml: y,
        inputs: when.len(),
        expectations: events.len(),
        unjoined,
        unmapped,
    }
}
