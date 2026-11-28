//! Structural check of an architecture document, before any simulation.
//!
//! Each refusal is named by its rule; the rule is also the gate KPI that counts it.

use crate::doc::num;
use serde_json::Value;
use std::collections::{BTreeSet, VecDeque};
use std::fmt;
use std::sync::OnceLock;

/// The domain of the architecture block, the one source of the part kinds and of what a link
/// carries.
const DOMAIN: &str = include_str!("../../../blocks/9-composition/architecture.yaml");

/// A variant of a domain enum as an architecture document writes it: `ModelCall` is `model_call`.
fn snake(variant: &str) -> String {
    let mut out = String::new();
    for (i, c) in variant.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// The variants of enum `name` of the domain, in snake case and in the order it declares them.
fn variants(name: &str) -> Vec<String> {
    let doc: Value = serde_yaml_ng::from_str(DOMAIN).unwrap_or(Value::Null);
    doc["types"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|t| t["name"] == name)
        .and_then(|t| t["variants"].as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(snake)
        .collect()
}

/// The blocks a part may instantiate: the variants of `agentic.architecture.PartKind`.
pub fn part_kinds() -> &'static [String] {
    static KINDS: OnceLock<Vec<String>> = OnceLock::new();
    KINDS.get_or_init(|| variants("agentic.architecture.PartKind"))
}

/// What a link may carry: the variants of `agentic.architecture.Carries`.
pub fn carries() -> &'static [String] {
    static CARRIES: OnceLock<Vec<String>> = OnceLock::new();
    CARRIES.get_or_init(|| variants("agentic.architecture.Carries"))
}

/// One refusal of an architecture, with the part it concerns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// A link's `from`, `to` or `over` names no part.
    UnknownLinkPart { link: usize, part: String },
    /// A part's `block` is not a part kind of the domain.
    UnknownBlock { part: String, block: String },
    /// A link's `carries` is not a value of the domain; `part` is the link's `from`.
    UnknownCarries {
        link: usize,
        part: String,
        carries: String,
    },
    /// A part's `per` names no part.
    PerWithoutPart { part: String, names: String },
    /// A part's `acts_for` names no part.
    ActsForWithoutPart { part: String, names: String },
    /// A part's `egress` names no part.
    EgressWithoutPart { part: String, names: String },
    /// An agent part reaches no model part over `model_call` links.
    AgentWithoutModelPath { part: String },
    /// A `question` link ends at a part that cannot answer (only principals and agents answer).
    QuestionWithoutAnswerer { link: usize, part: String },
    /// A `review` link ends at a part that is not an agent with `role: reviewer`.
    ReviewWithoutReviewer { link: usize, part: String },
    /// `border` names no part.
    BorderWithoutPart { part: String },
    /// A `trigger` link starts at a part that is neither a trigger nor a schedule.
    TriggerSourceNotTrigger { link: usize, part: String },
    /// A `trigger` link ends at a part that is neither an agent nor a runtime.
    TriggerTargetNotAgent { link: usize, part: String },
    /// A trigger part names no source it can fire on.
    TriggerWithoutSource { part: String, detail: String },
    /// A trigger's policy and `max_parallel` do not fit together.
    InvalidConcurrency { part: String },
    /// A schedule part has no positive `every`.
    InvalidSchedule { part: String },
    /// `coordination` or `start_stagger` on a part that is not an agent, or an unknown coordination.
    SwarmNeedsAgent { part: String },
    /// No link, `per`, `acts_for`, `egress` or `border` reaches the part.
    UnreachablePart { part: String },
}

/// The values of `coordination` on an agent part (agentic.architecture.Coordination).
pub const COORDINATIONS: &[&str] = &["orchestrator", "board", "mesh", "none"];
/// The values of `policy` on a trigger part (agentic.trigger.ConcurrencyPolicy).
pub const POLICIES: &[&str] = &["skip", "queue", "parallel"];
/// The values of `distribution` on a channel part (agentic.channel.Distribution).
pub const DISTRIBUTIONS: &[&str] = &["direct", "topic", "queue", "board"];

/// Every rule, in the order the check applies them; each is a gate KPI.
pub const GATE_KPIS: &[&str] = &[
    "unknown_link_parts",
    "unknown_blocks",
    "unknown_carries",
    "per_without_part",
    "acts_for_without_part",
    "egress_without_part",
    "agent_without_model_path",
    "question_without_answerer",
    "review_without_reviewer",
    "border_without_part",
    "trigger_source_not_trigger",
    "trigger_target_not_agent",
    "trigger_without_source",
    "invalid_concurrency",
    "invalid_schedule",
    "swarm_needs_agent",
    "unreachable_parts",
];

impl Refusal {
    pub fn rule(&self) -> &'static str {
        match self {
            Refusal::UnknownLinkPart { .. } => "unknown_link_parts",
            Refusal::UnknownBlock { .. } => "unknown_blocks",
            Refusal::UnknownCarries { .. } => "unknown_carries",
            Refusal::PerWithoutPart { .. } => "per_without_part",
            Refusal::ActsForWithoutPart { .. } => "acts_for_without_part",
            Refusal::EgressWithoutPart { .. } => "egress_without_part",
            Refusal::AgentWithoutModelPath { .. } => "agent_without_model_path",
            Refusal::QuestionWithoutAnswerer { .. } => "question_without_answerer",
            Refusal::ReviewWithoutReviewer { .. } => "review_without_reviewer",
            Refusal::BorderWithoutPart { .. } => "border_without_part",
            Refusal::TriggerSourceNotTrigger { .. } => "trigger_source_not_trigger",
            Refusal::TriggerTargetNotAgent { .. } => "trigger_target_not_agent",
            Refusal::TriggerWithoutSource { .. } => "trigger_without_source",
            Refusal::InvalidConcurrency { .. } => "invalid_concurrency",
            Refusal::InvalidSchedule { .. } => "invalid_schedule",
            Refusal::SwarmNeedsAgent { .. } => "swarm_needs_agent",
            Refusal::UnreachablePart { .. } => "unreachable_parts",
        }
    }

    pub fn part(&self) -> &str {
        match self {
            Refusal::UnknownLinkPart { part, .. }
            | Refusal::UnknownBlock { part, .. }
            | Refusal::UnknownCarries { part, .. }
            | Refusal::PerWithoutPart { part, .. }
            | Refusal::ActsForWithoutPart { part, .. }
            | Refusal::EgressWithoutPart { part, .. }
            | Refusal::AgentWithoutModelPath { part }
            | Refusal::QuestionWithoutAnswerer { part, .. }
            | Refusal::ReviewWithoutReviewer { part, .. }
            | Refusal::BorderWithoutPart { part }
            | Refusal::TriggerSourceNotTrigger { part, .. }
            | Refusal::TriggerTargetNotAgent { part, .. }
            | Refusal::TriggerWithoutSource { part, .. }
            | Refusal::InvalidConcurrency { part }
            | Refusal::InvalidSchedule { part }
            | Refusal::SwarmNeedsAgent { part }
            | Refusal::UnreachablePart { part } => part,
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let detail = match self {
            Refusal::UnknownLinkPart { link, .. } => {
                format!("link {link} names a part that does not exist")
            }
            Refusal::UnknownBlock { block, .. } => format!(
                "block `{block}` is not a part kind of agentic.architecture ({})",
                part_kinds().join(", ")
            ),
            Refusal::UnknownCarries { link, carries: c, .. } => format!(
                "link {link} carries `{c}`, which agentic.architecture does not declare ({})",
                carries().join(", ")
            ),
            Refusal::PerWithoutPart { names, .. } => format!("per `{names}` names no part"),
            Refusal::ActsForWithoutPart { names, .. } => {
                format!("acts_for `{names}` names no part")
            }
            Refusal::EgressWithoutPart { names, .. } => format!("egress `{names}` names no part"),
            Refusal::AgentWithoutModelPath { .. } => {
                "an agent with no model_call path to a model".into()
            }
            Refusal::QuestionWithoutAnswerer { link, .. } => {
                format!("link {link} sends a question to a part that cannot answer")
            }
            Refusal::ReviewWithoutReviewer { link, .. } => {
                format!(
                    "link {link} sends a review to a part that is not an agent with role reviewer"
                )
            }
            Refusal::BorderWithoutPart { .. } => "border names a part that does not exist".into(),
            Refusal::TriggerSourceNotTrigger { link, .. } => {
                format!("link {link} carries trigger from a part that is not a trigger or schedule")
            }
            Refusal::TriggerTargetNotAgent { link, .. } => {
                format!("link {link} carries trigger to a part that is neither an agent nor a runtime")
            }
            Refusal::TriggerWithoutSource { detail, .. } => detail.clone(),
            Refusal::InvalidConcurrency { .. } => {
                "policy is skip, queue or parallel; max_parallel is at least 1, and above 1 only under parallel".into()
            }
            Refusal::InvalidSchedule { .. } => "a schedule needs a positive `every`".into(),
            Refusal::SwarmNeedsAgent { .. } => format!(
                "coordination ({}) and start_stagger belong to an agent part",
                COORDINATIONS.join(", ")
            ),
            Refusal::UnreachablePart { .. } => "nothing links to this part".into(),
        };
        write!(f, "{}: part `{}`: {detail}", self.rule(), self.part())
    }
}

fn s(v: &Value) -> Option<&str> {
    v.as_str()
}

fn names(v: &Value) -> Vec<&str> {
    match v {
        Value::String(x) => vec![x.as_str()],
        Value::Array(xs) => xs.iter().filter_map(s).collect(),
        _ => Vec::new(),
    }
}

/// The block of part `name`, if the part exists.
pub fn block<'a>(architecture: &'a Value, name: &str) -> Option<&'a str> {
    architecture["parts"]
        .get(name)
        .and_then(|p| p["block"].as_str())
}

/// Whether part `name` is an agent with `role: reviewer`, the only part a `review` link may end at.
pub fn is_reviewer(architecture: &Value, name: &str) -> bool {
    let p = &architecture["parts"][name];
    p["block"] == "agent" && p["role"] == "reviewer"
}

/// The parts reached from `start` over links that carry `carries`, breadth first, `start` excluded.
pub fn reach(architecture: &Value, start: &str, carries: &str) -> Vec<String> {
    let links = architecture["links"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut seen = BTreeSet::from([start.to_string()]);
    let mut order = Vec::new();
    let mut queue = VecDeque::from([start.to_string()]);
    while let Some(at) = queue.pop_front() {
        for l in &links {
            if s(&l["carries"]) == Some(carries)
                && s(&l["from"]) == Some(at.as_str())
                && let Some(to) = s(&l["to"])
                && seen.insert(to.to_string())
            {
                order.push(to.to_string());
                queue.push_back(to.to_string());
            }
        }
    }
    order
}

/// The value at a place, or the `value` of an optimization marker there.
pub fn value(v: &Value) -> &Value {
    match v {
        Value::Object(o) if o.contains_key("vary") => o.get("value").unwrap_or(&Value::Null),
        _ => v,
    }
}

/// The distribution of channel part `name` (default `direct`), if the part is a channel.
pub fn distribution<'a>(architecture: &'a Value, name: &str) -> Option<&'a str> {
    let p = &architecture["parts"][name];
    (p["block"] == "channel").then(|| s(value(&p["distribution"])).unwrap_or("direct"))
}

/// Why trigger part `p` cannot fire, or `None` when its kind names a source it can fire on.
fn trigger_source(architecture: &Value, p: &Value) -> Option<String> {
    match s(value(&p["kind"])) {
        Some("on_schedule") => {
            let to = s(value(&p["schedule"])).unwrap_or("");
            (block(architecture, to) != Some("schedule"))
                .then(|| format!("an on_schedule trigger names a schedule part; `{to}` is none"))
        }
        Some("on_message") => {
            let to = s(value(&p["channel"])).unwrap_or("");
            match distribution(architecture, to) {
                Some("topic" | "queue" | "board") => None,
                Some(d) => Some(format!(
                    "an on_message trigger listens on a topic, queue or board; `{to}` is {d}"
                )),
                None => Some(format!(
                    "an on_message trigger names a channel part; `{to}` is none"
                )),
            }
        }
        Some("on_event") => {
            if s(value(&p["event_name"])).is_none_or(str::is_empty) {
                return Some("an on_event trigger names its event_name".to_string());
            }
            let from = s(value(&p["source"]))?;
            block(architecture, from).is_none().then(|| {
                format!("an on_event trigger's source names the part whose events fire it; `{from}` is none")
            })
        }
        other => Some(format!(
            "kind {other:?}, expected on_message, on_schedule or on_event"
        )),
    }
}

pub fn check(architecture: &Value) -> Vec<Refusal> {
    let empty = serde_json::Map::new();
    let parts = architecture["parts"].as_object().unwrap_or(&empty);
    let known = |n: &str| parts.contains_key(n);
    let links = architecture["links"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    let mut referenced: BTreeSet<&str> = BTreeSet::new();

    for (i, l) in links.iter().enumerate() {
        for key in ["from", "to", "over"] {
            if let Some(n) = s(&l[key]) {
                referenced.insert(n);
                if !known(n) {
                    out.push(Refusal::UnknownLinkPart {
                        link: i,
                        part: n.to_string(),
                    });
                }
            }
        }
    }
    for (name, p) in parts {
        if let Some(b) = s(&p["block"])
            && !part_kinds().iter().any(|k| k == b)
        {
            out.push(Refusal::UnknownBlock {
                part: name.clone(),
                block: b.to_string(),
            });
        }
    }
    for (i, l) in links.iter().enumerate() {
        if let Some(c) = s(&l["carries"])
            && !carries().iter().any(|k| k == c)
        {
            out.push(Refusal::UnknownCarries {
                link: i,
                part: s(&l["from"]).unwrap_or("").to_string(),
                carries: c.to_string(),
            });
        }
    }
    for (key, make) in [
        (
            "per",
            (|part, names| Refusal::PerWithoutPart { part, names })
                as fn(String, String) -> Refusal,
        ),
        ("acts_for", |part, names| Refusal::ActsForWithoutPart {
            part,
            names,
        }),
        ("egress", |part, names| Refusal::EgressWithoutPart {
            part,
            names,
        }),
    ] {
        for (name, p) in parts {
            for n in names(&p[key]) {
                referenced.insert(n);
                if !known(n) {
                    out.push(make(name.clone(), n.to_string()));
                }
            }
        }
    }
    for (name, p) in parts {
        if s(&p["block"]) == Some("agent")
            && !reach(architecture, name, "model_call")
                .iter()
                .any(|r| block(architecture, r) == Some("model"))
        {
            out.push(Refusal::AgentWithoutModelPath { part: name.clone() });
        }
    }
    for (i, l) in links.iter().enumerate() {
        if s(&l["carries"]) == Some("question")
            && let Some(to) = s(&l["to"])
            && let Some(b) = block(architecture, to)
            && b != "principal"
            && b != "agent"
        {
            out.push(Refusal::QuestionWithoutAnswerer {
                link: i,
                part: to.to_string(),
            });
        }
        if s(&l["carries"]) == Some("review")
            && let Some(to) = s(&l["to"])
            && known(to)
            && !is_reviewer(architecture, to)
        {
            out.push(Refusal::ReviewWithoutReviewer {
                link: i,
                part: to.to_string(),
            });
        }
    }
    for (i, l) in links.iter().enumerate() {
        if s(&l["carries"]) != Some("trigger") {
            continue;
        }
        if let Some(from) = s(&l["from"])
            && let Some(b) = block(architecture, from)
            && b != "trigger"
            && b != "schedule"
        {
            out.push(Refusal::TriggerSourceNotTrigger {
                link: i,
                part: from.to_string(),
            });
        }
        if let Some(to) = s(&l["to"])
            && let Some(b) = block(architecture, to)
            && b != "agent"
            && b != "runtime"
        {
            out.push(Refusal::TriggerTargetNotAgent {
                link: i,
                part: to.to_string(),
            });
        }
    }
    for (name, p) in parts {
        match s(&p["block"]) {
            Some("trigger") => {
                for key in ["schedule", "channel", "source"] {
                    if let Some(n) = s(value(&p[key])) {
                        referenced.insert(n);
                    }
                }
                if let Some(detail) = trigger_source(architecture, p) {
                    out.push(Refusal::TriggerWithoutSource {
                        part: name.clone(),
                        detail,
                    });
                }
                let policy = s(value(&p["policy"])).unwrap_or("skip");
                let max = match value(&p["max_parallel"]) {
                    Value::Null => Some(1.0),
                    v => v.as_f64(),
                };
                let fits = POLICIES.contains(&policy)
                    && max.is_some_and(|m| m >= 1.0 && m.fract() == 0.0)
                    && (policy == "parallel" || max == Some(1.0));
                if !fits {
                    out.push(Refusal::InvalidConcurrency { part: name.clone() });
                }
            }
            Some("schedule") if !num(value(&p["every"])).is_some_and(|e| e > 0.0) => {
                out.push(Refusal::InvalidSchedule { part: name.clone() });
            }
            _ => {}
        }
        let coordination = value(&p["coordination"]);
        let stagger = value(&p["start_stagger"]);
        let swarm = !coordination.is_null() || !stagger.is_null();
        let agent = s(&p["block"]) == Some("agent");
        let known =
            coordination.is_null() || s(coordination).is_some_and(|c| COORDINATIONS.contains(&c));
        let stagger_ok = stagger.is_null() || num(stagger).is_some_and(|x| x >= 0.0);
        if swarm && !(agent && known && stagger_ok) {
            out.push(Refusal::SwarmNeedsAgent { part: name.clone() });
        }
    }
    // The border parts of a flattened nested architecture are its interface: reached from outside
    // whether or not this document links to them.
    for n in architecture["nested"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|e| names(&e["border"]))
    {
        referenced.insert(n);
    }
    for n in names(&architecture["border"]) {
        referenced.insert(n);
        if !known(n) {
            out.push(Refusal::BorderWithoutPart {
                part: n.to_string(),
            });
        }
    }
    for (name, p) in parts {
        if !referenced.contains(name.as_str()) && p.get("per").is_none() {
            out.push(Refusal::UnreachablePart { part: name.clone() });
        }
    }
    out
}

/// The gate KPIs of an architecture: the count of refusals per rule, every rule present.
pub fn gate_counts(architecture: &Value) -> Vec<(&'static str, f64)> {
    let refusals = check(architecture);
    GATE_KPIS
        .iter()
        .map(|g| {
            (
                *g,
                refusals.iter().filter(|r| r.rule() == *g).count() as f64,
            )
        })
        .collect()
}
