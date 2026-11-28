//! The architecture as a whole: KPIs of one architecture document, read from it alone, before any
//! simulation. They say what kind of system the architecture is: its size, its autonomy, its
//! exposure, its weak points and its fixed limits.
//!
//! A marked place counts at its value as built, as the simulator reads it.

use crate::check::value;
use crate::doc::num;
use serde_json::Value;
use std::collections::BTreeSet;

/// One KPI of the architecture as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measure {
    pub id: &'static str,
    /// `completeness` or `quality`, as the KPI catalogue names it.
    pub kind: &'static str,
    /// What it counts; the `summary` of its entry in `kpis/catalogue.yaml`.
    pub what: &'static str,
}

/// Every KPI of the architecture as a whole, in the order the check reports them.
pub const ARCHITECTURE_KPIS: &[Measure] = &[
    Measure {
        id: "agents_total",
        kind: "completeness",
        what: "Agent copies: the sum of count over the agent parts.",
    },
    Measure {
        id: "principals_total",
        kind: "completeness",
        what: "Principal parts.",
    },
    Measure {
        id: "human_touchpoints",
        kind: "quality",
        what: "Links that end at a principal and carry a question or a proposal.",
    },
    Measure {
        id: "autonomy_depth",
        kind: "quality",
        what: "Links in the longest chain of order and question links from agent to agent.",
    },
    Measure {
        id: "model_call_paths",
        kind: "completeness",
        what: "Distinct paths of model_call links from an agent part to a model part.",
    },
    Measure {
        id: "ungated_model_calls",
        kind: "quality",
        what: "Paths of model_call links from an agent to a model that pass no gateway part.",
    },
    Measure {
        id: "single_points",
        kind: "quality",
        what: "Parts whose removal leaves some agent with no model_call path to a model.",
    },
    Measure {
        id: "border_parts",
        kind: "quality",
        what: "Parts named in the border, the parts that take input from outside.",
    },
    Measure {
        id: "egress_targets",
        kind: "quality",
        what: "Distinct parts named in the egress of the sandbox parts.",
    },
    Measure {
        id: "shared_workspaces",
        kind: "quality",
        what: "Workspace parts whose sharing is not exclusive.",
    },
    Measure {
        id: "capacity_ratio",
        kind: "quality",
        what: "Agent copies that claim a pool divided by its capacity, the largest over the pools.",
    },
];

/// What a link to a principal carries when it asks the principal to decide.
pub const DECISION_CARRIES: &[&str] = &["question", "proposal"];

/// What a link from agent to agent carries when it passes work or a question on.
pub const AUTONOMY_CARRIES: &[&str] = &["order", "question"];

/// The KPI of the architecture as a whole named `id`.
pub fn measure_of(id: &str) -> Option<&'static Measure> {
    ARCHITECTURE_KPIS.iter().find(|m| m.id == id)
}

fn s(v: &Value) -> Option<&str> {
    v.as_str()
}

/// The parts of an architecture and its links, read once.
struct Doc<'a> {
    parts: Vec<(&'a str, &'a Value)>,
    links: Vec<(&'a str, &'a str, &'a str)>,
}

impl<'a> Doc<'a> {
    fn new(architecture: &'a Value) -> Self {
        let parts: Vec<(&str, &Value)> = architecture["parts"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(n, p)| (n.as_str(), p))
            .collect();
        let known: BTreeSet<&str> = parts.iter().map(|(n, _)| *n).collect();
        let links = architecture["links"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|l| Some((s(&l["from"])?, s(&l["to"])?, s(&l["carries"])?)))
            .filter(|(f, t, _)| known.contains(f) && known.contains(t))
            .collect();
        Doc { parts, links }
    }

    fn block(&self, name: &str) -> Option<&'a str> {
        self.parts
            .iter()
            .find(|(n, _)| *n == name)
            .and_then(|(_, p)| s(&p["block"]))
    }

    fn of_block(&self, block: &str) -> Vec<&'a str> {
        self.parts
            .iter()
            .filter(|(_, p)| p["block"] == block)
            .map(|(n, _)| *n)
            .collect()
    }

    fn part(&self, name: &str) -> &'a Value {
        self.parts
            .iter()
            .find(|(n, _)| *n == name)
            .map_or(&Value::Null, |(_, p)| p)
    }

    /// The copies of part `name`: its `count`, 1 when it names none.
    fn copies(&self, name: &str) -> f64 {
        num(value(&self.part(name)["count"]))
            .unwrap_or(1.0)
            .max(0.0)
            .round()
    }

    /// The targets of `model_call` links from `at`.
    fn calls(&self, at: &str) -> Vec<&'a str> {
        self.links
            .iter()
            .filter(|(f, _, c)| *f == at && *c == "model_call")
            .map(|(_, t, _)| *t)
            .collect()
    }

    /// Every simple path of `model_call` links from `start` to a model part, as the parts it
    /// passes between the two.
    fn model_paths(&self, start: &'a str) -> Vec<Vec<&'a str>> {
        let mut out = Vec::new();
        let mut path = vec![start];
        self.walk(&mut path, &mut out);
        out
    }

    fn walk(&self, path: &mut Vec<&'a str>, out: &mut Vec<Vec<&'a str>>) {
        let at = *path.last().expect("a path has a start");
        for to in self.calls(at) {
            if path.contains(&to) {
                continue;
            }
            if self.block(to) == Some("model") {
                out.push(path[1..].to_vec());
                continue;
            }
            path.push(to);
            self.walk(path, out);
            path.pop();
        }
    }

    /// Whether `start` reaches a model part over `model_call` links without passing `avoid`.
    fn reaches_model(&self, start: &str, avoid: &str) -> bool {
        let mut seen = BTreeSet::from([start]);
        let mut queue = vec![start];
        while let Some(at) = queue.pop() {
            for to in self.calls(at) {
                if to == avoid || !seen.insert(to) {
                    continue;
                }
                if self.block(to) == Some("model") {
                    return true;
                }
                queue.push(to);
            }
        }
        false
    }

    fn longest_chain(&self, path: &mut Vec<&'a str>) -> usize {
        let at = *path.last().expect("a chain has a start");
        let mut best = 0;
        for (f, t, c) in &self.links {
            if *f != at
                || !AUTONOMY_CARRIES.contains(c)
                || self.block(t) != Some("agent")
                || path.contains(t)
            {
                continue;
            }
            path.push(t);
            best = best.max(1 + self.longest_chain(path));
            path.pop();
        }
        best
    }
}

/// The parts whose removal leaves some agent with no `model_call` path to a model part, by name:
/// the cut vertices of the `model_call` graph between agents and models. An agent counts only when
/// it reaches a model with every part in place.
pub fn single_point_parts(architecture: &Value) -> Vec<String> {
    let d = Doc::new(architecture);
    let agents: Vec<&str> = d
        .of_block("agent")
        .into_iter()
        .filter(|a| d.reaches_model(a, ""))
        .collect();
    let mut out: Vec<String> = d
        .parts
        .iter()
        .map(|(n, _)| *n)
        .filter(|c| agents.iter().any(|a| a != c && !d.reaches_model(a, c)))
        .map(str::to_string)
        .collect();
    out.sort();
    out
}

/// The KPIs of the architecture as a whole, in the order of [`ARCHITECTURE_KPIS`].
pub fn measure(architecture: &Value) -> Vec<(&'static str, f64)> {
    let d = Doc::new(architecture);
    let agents = d.of_block("agent");
    let principals = d.of_block("principal");

    let touchpoints = d
        .links
        .iter()
        .filter(|(_, t, c)| d.block(t) == Some("principal") && DECISION_CARRIES.contains(c))
        .count();
    let depth = agents
        .iter()
        .map(|a| d.longest_chain(&mut vec![a]))
        .max()
        .unwrap_or(0);
    let paths: Vec<Vec<&str>> = agents.iter().flat_map(|a| d.model_paths(a)).collect();
    let ungated = paths
        .iter()
        .filter(|p| !p.iter().any(|x| d.block(x) == Some("gateway")))
        .count();
    let border: BTreeSet<&str> = match &architecture["border"] {
        Value::String(b) => BTreeSet::from([b.as_str()]),
        Value::Array(bs) => bs.iter().filter_map(s).collect(),
        _ => BTreeSet::new(),
    };
    let border = border.iter().filter(|b| d.block(b).is_some()).count();
    let egress: BTreeSet<&str> = d
        .of_block("sandbox")
        .iter()
        .flat_map(|b| match &d.part(b)["egress"] {
            Value::String(t) => vec![t.as_str()],
            Value::Array(ts) => ts.iter().filter_map(s).collect(),
            _ => Vec::new(),
        })
        .filter(|t| d.block(t).is_some())
        .collect();
    let shared = d
        .of_block("workspace")
        .iter()
        .filter(|w| s(value(&d.part(w)["sharing"])) != Some("exclusive"))
        .count();
    let capacity = d
        .of_block("pool")
        .iter()
        .filter_map(|p| {
            let cap = num(value(&d.part(p)["capacity"])).filter(|c| *c > 0.0)?;
            let claimants: BTreeSet<&str> = d
                .links
                .iter()
                .filter(|(f, t, c)| t == p && *c == "claim" && d.block(f) == Some("agent"))
                .map(|(f, _, _)| *f)
                .collect();
            Some(claimants.iter().map(|a| d.copies(a)).sum::<f64>() / cap)
        })
        .fold(0.0, f64::max);

    vec![
        ("agents_total", agents.iter().map(|a| d.copies(a)).sum()),
        ("principals_total", principals.len() as f64),
        ("human_touchpoints", touchpoints as f64),
        ("autonomy_depth", depth as f64),
        ("model_call_paths", paths.len() as f64),
        ("ungated_model_calls", ungated as f64),
        (
            "single_points",
            single_point_parts(architecture).len() as f64,
        ),
        ("border_parts", border as f64),
        ("egress_targets", egress.len() as f64),
        ("shared_workspaces", shared as f64),
        ("capacity_ratio", capacity),
    ]
}
