//! Unavailability in scenarios: `unavailable` entries name a part, or a dependency of a part,
//! that is unavailable from a time on, and `then.parts` names what each remaining part shows: a
//! view that answers, is empty or errors, a command that is accepted or refused, an exit status
//! before the part listens, and whether the rest of it answers. Views, commands and errors are
//! named as the building blocks declare them; the declarations are read from the block files the
//! crate is built with.
//!
//! The simulator makes a part unavailable where it can model it: a pool or a gateway loses its
//! capacity, a principal is away (the fault injection of experiments), and a border part that is
//! down refuses the inputs that reach it. It serves no views and starts no processes, so the
//! observables of `then.parts` are `not_checked` on it; a runtime conformance run checks them.

use crate::check::block;
use crate::doc::parse_yaml;
use crate::reader::Reader;
use crate::sim::{Fault, FaultKind};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::OnceLock;

/// The rule a scenario run refuses an input by when its part, its author or the channel it comes
/// over is unavailable.
pub const RULE: &str = "part_unavailable";

/// One `unavailable` entry: `part` (or only its `dependency`) is unavailable from `from` for
/// `length` seconds, to the end of the run when `length` is not given.
#[derive(Debug, Clone, PartialEq)]
pub struct Unavailable {
    pub part: String,
    pub dependency: Option<String>,
    pub from: f64,
    pub length: Option<f64>,
}

impl Unavailable {
    pub fn until(&self, horizon: f64) -> f64 {
        self.length
            .map_or(horizon, |l| (self.from + l).min(horizon))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewShows {
    /// The view answers without an error, with or without rows.
    Answer,
    /// The view answers with no rows and no error.
    Empty,
    /// The view answers with an error.
    Error,
}

pub const VIEW_SHOWS: &[(&str, ViewShows)] = &[
    ("answer", ViewShows::Answer),
    ("empty", ViewShows::Empty),
    ("error", ViewShows::Error),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandShows {
    Accepted,
    Refused,
}

pub const COMMAND_SHOWS: &[(&str, CommandShows)] = &[
    ("accepted", CommandShows::Accepted),
    ("refused", CommandShows::Refused),
];

#[derive(Debug, Clone, PartialEq)]
pub struct ViewExpect {
    pub view: String,
    pub shows: ViewShows,
    pub error: Option<String>,
    pub written: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandExpect {
    pub command: String,
    pub shows: CommandShows,
    pub error: Option<String>,
    pub written: Value,
}

/// What one part shows while something is unavailable.
#[derive(Debug, Clone, PartialEq)]
pub struct PartThen {
    pub part: String,
    pub views: Vec<ViewExpect>,
    pub commands: Vec<CommandExpect>,
    /// The status the part's process exits with before it listens.
    pub exits: Option<u64>,
    /// Every view and command of the part that is not named answers as usual.
    pub rest: bool,
}

/// Reads `unavailable`.
pub fn read_unavailable(r: &mut Reader, v: Option<&Value>) -> Vec<Unavailable> {
    let mut out = Vec::new();
    let Some(v) = v.filter(|v| !v.is_null()) else {
        return out;
    };
    let Some(items) = v.as_array() else {
        r.err("unavailable", "expected a list");
        return out;
    };
    for (i, e) in items.iter().enumerate() {
        let p = format!("unavailable.{i}");
        let Some(o) = r.obj(
            e,
            &p,
            &["part", "dependency", "from", "for"],
            &["part", "from"],
        ) else {
            continue;
        };
        let part = r.text(o, "part", &p);
        let dependency = r.text(o, "dependency", &p);
        let from = r.duration(o, "from", &p);
        let length = r.duration(o, "for", &p);
        if length == Some(0.0) {
            r.err(&format!("{p}.for"), "must be longer than 0s");
        }
        if let (Some(part), Some(from)) = (part, from) {
            out.push(Unavailable {
                part,
                dependency,
                from,
                length,
            });
        }
    }
    out
}

/// Reads `then.parts`: a mapping from a part to what it shows.
pub fn read_parts(r: &mut Reader, v: Option<&Value>) -> Vec<PartThen> {
    let mut out = Vec::new();
    let Some(v) = v.filter(|v| !v.is_null()) else {
        return out;
    };
    let Some(m) = v.as_object() else {
        r.err(
            "then.parts",
            "expected a mapping from a part to what it shows",
        );
        return out;
    };
    for (part, body) in m {
        let p = format!("then.parts.{part}");
        let Some(o) = r.obj(body, &p, &["views", "commands", "exits", "rest"], &[]) else {
            continue;
        };
        if o.is_empty() {
            r.err(&p, "names views, commands, exits or rest");
        }
        let mut views = Vec::new();
        let mut seen = BTreeSet::new();
        for (j, e) in r.list(o, "views", &p).iter().enumerate() {
            let q = format!("{p}.views.{j}");
            let Some(eo) = r.obj(e, &q, &["view", "shows", "error"], &["view", "shows"]) else {
                continue;
            };
            let view = r.text(eo, "view", &q);
            let shows = r.one_of(eo, "shows", &q, VIEW_SHOWS);
            let error = r.text(eo, "error", &q);
            if error.is_some() && shows.is_some_and(|s| s != ViewShows::Error) {
                r.err(
                    &format!("{q}.error"),
                    "only a view that shows error names one",
                );
            }
            if let Some(n) = &view
                && !seen.insert(n.clone())
            {
                r.err(&format!("{q}.view"), format!("`{n}` is named twice"));
            }
            if let (Some(view), Some(shows)) = (view, shows) {
                views.push(ViewExpect {
                    view,
                    shows,
                    error,
                    written: e.clone(),
                });
            }
        }
        let mut commands = Vec::new();
        let mut seen = BTreeSet::new();
        for (j, e) in r.list(o, "commands", &p).iter().enumerate() {
            let q = format!("{p}.commands.{j}");
            let Some(eo) = r.obj(e, &q, &["command", "shows", "error"], &["command", "shows"])
            else {
                continue;
            };
            let command = r.text(eo, "command", &q);
            let shows = r.one_of(eo, "shows", &q, COMMAND_SHOWS);
            let error = r.text(eo, "error", &q);
            if error.is_some() && shows.is_some_and(|s| s != CommandShows::Refused) {
                r.err(&format!("{q}.error"), "only a refused command names one");
            }
            if let Some(n) = &command
                && !seen.insert(n.clone())
            {
                r.err(&format!("{q}.command"), format!("`{n}` is named twice"));
            }
            if let (Some(command), Some(shows)) = (command, shows) {
                commands.push(CommandExpect {
                    command,
                    shows,
                    error,
                    written: e.clone(),
                });
            }
        }
        let exits = r.uint(o, "exits", &p);
        if exits.is_some_and(|s| !(1..=255).contains(&s)) {
            r.err(&format!("{p}.exits"), "an exit status from 1 to 255");
        }
        let rest = r
            .one_of(o, "rest", &p, &[("answers", true)])
            .unwrap_or(false);
        if exits.is_some() && (!views.is_empty() || !commands.is_empty() || rest) {
            r.err(
                &p,
                "a part that exits before it listens shows no view and takes no command",
            );
        }
        out.push(PartThen {
            part: part.clone(),
            views,
            commands,
            exits,
            rest,
        });
    }
    out
}

macro_rules! sources {
    ($($p:literal),* $(,)?) => {
        &[$(($p, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../", $p)))),*]
    };
}

/// The specification files of the building blocks, the core and the packs, as the crate was built.
pub const SOURCES: &[(&str, &str)] = sources![
    "blocks/0-foundation/shared.yaml",
    "blocks/1-principals/principal.yaml",
    "blocks/1-principals/delegation.yaml",
    "blocks/2-intelligence/model.yaml",
    "blocks/2-intelligence/gateway.yaml",
    "blocks/3-capability/tool.yaml",
    "blocks/3-capability/knowledge.yaml",
    "blocks/3-capability/store.yaml",
    "blocks/4-agent/agent.yaml",
    "blocks/4-agent/harness.yaml",
    "blocks/5-work/work.yaml",
    "blocks/5-work/process.yaml",
    "blocks/6-governance/authority.yaml",
    "blocks/7-collaboration/channel.yaml",
    "blocks/7-collaboration/question.yaml",
    "blocks/7-collaboration/trigger.yaml",
    "blocks/8-place/workspace.yaml",
    "blocks/8-place/sandbox.yaml",
    "blocks/8-place/runtime.yaml",
    "blocks/8-place/resource.yaml",
    "blocks/9-composition/architecture.yaml",
    "blocks/9-composition/deployment.yaml",
    "blocks/10-evaluation/kpi.yaml",
    "blocks/10-evaluation/experiment.yaml",
    "packs/source-control/blocks/forge.yaml",
    "packs/source-control/blocks/repository.yaml",
    "packs/source-control/blocks/check.yaml",
    "packs/source-control/blocks/change.yaml",
    "packs/tracking/blocks/issue.yaml",
    "packs/tracking/blocks/board.yaml",
    "packs/tracking/blocks/queue.yaml",
    "packs/files/blocks/toolset.yaml",
    "packs/shell/blocks/toolset.yaml",
];

/// The views, commands, errors and events the blocks declare: each view and command with its
/// domain, and each command with the errors its outcomes refuse it with.
#[derive(Debug, Default)]
pub struct Names {
    pub views: BTreeMap<String, String>,
    pub commands: BTreeMap<String, String>,
    pub errors: BTreeSet<String>,
    pub events: BTreeSet<String>,
    pub refusals: BTreeMap<String, BTreeSet<String>>,
}

/// Every `error` a command's outcomes name, at any depth.
fn outcome_errors(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                match (k.as_str(), x) {
                    ("error", Value::String(e)) => {
                        out.insert(e.clone());
                    }
                    _ => outcome_errors(x, out),
                }
            }
        }
        Value::Array(xs) => xs.iter().for_each(|x| outcome_errors(x, out)),
        _ => {}
    }
}

/// The declared names, read once from [`SOURCES`].
pub fn names() -> &'static Names {
    static NAMES: OnceLock<Names> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut n = Names::default();
        for (path, text) in SOURCES {
            let doc = parse_yaml(text).unwrap_or_else(|e| panic!("{path}: {e}"));
            let domain = doc["domain"].as_str().unwrap_or_default().to_string();
            let each = |key: &str| -> Vec<String> {
                doc[key]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|x| x["name"].as_str().map(str::to_string))
                    .collect()
            };
            for v in each("views") {
                n.views.insert(v, domain.clone());
            }
            for c in each("commands") {
                n.commands.insert(c, domain.clone());
            }
            n.errors.extend(each("errors"));
            n.events.extend(each("events"));
            for c in doc["commands"].as_array().into_iter().flatten() {
                if let Some(name) = c["name"].as_str() {
                    let mut errors = BTreeSet::new();
                    outcome_errors(&c["outcomes"], &mut errors);
                    n.refusals.insert(name.to_string(), errors);
                }
            }
        }
        n
    })
}

/// The namespace of the domains of an architecture block: `agentic.<block>`, a pool is
/// `agentic.resource` and a schedule `agentic.trigger`; a pack block covers every domain under
/// it (`scm` is `agentic.scm.change`, `agentic.scm.check` and the others).
pub fn namespace(block: &str) -> String {
    let name = match block {
        "pool" => "resource",
        "schedule" => "trigger",
        b => b,
    };
    format!("agentic.{name}")
}

fn in_namespace(domain: &str, ns: &str) -> bool {
    domain == ns
        || domain
            .strip_prefix(ns)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn s(v: &Value) -> Option<&str> {
    v.as_str()
}

/// The parts `start` depends on: those its links go to or over, its `egress`, the parts that
/// exist once `per` copy of it, and on from each of those over their links, breadth first.
pub fn reaches(arch: &Value, start: &str) -> Vec<String> {
    let links = arch["links"].as_array().cloned().unwrap_or_default();
    let parts = arch["parts"].as_object().cloned().unwrap_or_default();
    let mut seen = BTreeSet::from([start.to_string()]);
    let mut order = Vec::new();
    let mut queue = VecDeque::from([start.to_string()]);
    while let Some(at) = queue.pop_front() {
        let mut next: Vec<String> = Vec::new();
        for l in &links {
            if s(&l["from"]) == Some(at.as_str()) {
                next.extend(s(&l["to"]).map(str::to_string));
                next.extend(s(&l["over"]).map(str::to_string));
            }
        }
        match &arch["parts"][&at]["egress"] {
            Value::String(e) => next.push(e.clone()),
            Value::Array(es) => next.extend(es.iter().filter_map(|e| s(e).map(str::to_string))),
            _ => {}
        }
        for (n, p) in &parts {
            if s(&p["per"]) == Some(at.as_str()) {
                next.push(n.clone());
            }
        }
        for n in next {
            if parts.contains_key(&n) && seen.insert(n.clone()) {
                order.push(n.clone());
                queue.push_back(n);
            }
        }
    }
    order
}

/// The namespaces whose views and commands part `name` may show: its block's, those of the parts
/// it depends on, and for an agent the harness it runs in, the work it runs and the questions it
/// asks.
pub fn namespaces(arch: &Value, name: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(b) = block(arch, name) {
        out.insert(namespace(b));
        if b == "agent" {
            for extra in ["harness", "work", "question"] {
                out.insert(namespace(extra));
            }
        }
    }
    for p in reaches(arch, name) {
        if let Some(b) = block(arch, &p) {
            out.insert(namespace(b));
        }
    }
    out
}

/// Every problem of the `unavailable` entries and of `then.parts` against the architecture and
/// the declarations of the blocks.
pub fn check(
    arch: &Value,
    horizon: f64,
    entries: &[Unavailable],
    parts: &[PartThen],
) -> Vec<String> {
    let mut out = Vec::new();
    let is_part = |n: &str| arch["parts"].get(n).is_some();
    for (i, u) in entries.iter().enumerate() {
        let p = format!("unavailable.{i}");
        if !is_part(&u.part) {
            out.push(format!(
                "{p}.part: `{}` is no part of the architecture",
                u.part
            ));
        } else if let Some(d) = &u.dependency {
            if !is_part(d) {
                out.push(format!(
                    "{p}.dependency: `{d}` is no part of the architecture"
                ));
            } else if d == &u.part {
                out.push(format!("{p}.dependency: `{d}` is the part itself"));
            } else if !reaches(arch, &u.part).contains(d) {
                out.push(format!(
                    "{p}.dependency: `{d}` is not reached from `{}` over a link",
                    u.part
                ));
            }
        }
        if u.from >= horizon {
            out.push(format!("{p}.from: not before the horizon"));
        }
    }
    let n = names();
    for pt in parts {
        let p = format!("then.parts.{}", pt.part);
        if !is_part(&pt.part) {
            out.push(format!("{p}: `{}` is no part of the architecture", pt.part));
            continue;
        }
        if entries
            .iter()
            .any(|u| u.part == pt.part && u.dependency.is_none())
        {
            out.push(format!(
                "{p}: `{}` is unavailable itself; then.parts names what the remaining parts show",
                pt.part
            ));
        }
        let spaces = namespaces(arch, &pt.part);
        let owner = |domain: &str| spaces.iter().any(|ns| in_namespace(domain, ns));
        let blk = block(arch, &pt.part).unwrap_or("?");
        for (j, v) in pt.views.iter().enumerate() {
            match n.views.get(&v.view) {
                None => out.push(format!(
                    "{p}.views.{j}.view: `{}` is no view of a block",
                    v.view
                )),
                Some(d) if !owner(d) => out.push(format!(
                    "{p}.views.{j}.view: `{}` is a view of {d}, not of `{}` (block {blk}) or a part it depends on",
                    v.view, pt.part
                )),
                Some(_) => {}
            }
            if let Some(e) = &v.error
                && !n.errors.contains(e)
            {
                out.push(format!("{p}.views.{j}.error: `{e}` is no error of a block"));
            }
        }
        for (j, c) in pt.commands.iter().enumerate() {
            match n.commands.get(&c.command) {
                None => out.push(format!(
                    "{p}.commands.{j}.command: `{}` is no command of a block",
                    c.command
                )),
                Some(d) if !owner(d) => out.push(format!(
                    "{p}.commands.{j}.command: `{}` is a command of {d}, not of `{}` (block {blk}) or a part it depends on",
                    c.command, pt.part
                )),
                Some(_) => {}
            }
            if let Some(e) = &c.error
                && !n.errors.contains(e)
            {
                out.push(format!(
                    "{p}.commands.{j}.error: `{e}` is no error of a block"
                ));
            }
        }
        if pt.exits.is_some()
            && !entries
                .iter()
                .any(|u| u.part == pt.part && u.dependency.is_some() && u.from == 0.0)
        {
            out.push(format!(
                "{p}.exits: a part exits before it listens only when a dependency of it is unavailable from 0s, and no unavailable entry names one"
            ));
        }
    }
    out
}

/// How the simulator models one entry.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// A fault of the simulator: `capacity_drop` to 0 or `principal_away`, on a part.
    Fault { kind: &'static str, part: String },
    /// The inputs that meet the unavailable part are refused. The simulator models the entry
    /// only while no simulated event is about `part` (when given) over `channel` (when given)
    /// inside its window; otherwise the run reports it not modeled.
    Inputs {
        part: Option<String>,
        channel: Option<String>,
    },
    /// The simulator cannot model it, with the reason.
    NotModeled(String),
}

/// What a scenario run does about its `unavailable` entries.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub modes: Vec<Mode>,
    /// `(from, until)` of each entry; a part that exits before it listens is down from the start
    /// to the end of the run.
    pub windows: Vec<(f64, f64)>,
    pub faults: Vec<Fault>,
    /// `(part, from, until)`: parts that are down.
    pub down: Vec<(String, f64, f64)>,
    /// `(part, dependency, from, until)`: a part cut from a dependency.
    pub cuts: Vec<(String, String, f64, f64)>,
}

fn border(arch: &Value) -> Vec<&str> {
    match &arch["border"] {
        Value::String(s) => vec![s.as_str()],
        Value::Array(xs) => xs.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

/// The parts with a link to `target` or over it, or with it in their `egress`.
fn users(arch: &Value, target: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for l in arch["links"].as_array().into_iter().flatten() {
        if (s(&l["to"]) == Some(target) || s(&l["over"]) == Some(target))
            && let Some(f) = s(&l["from"])
        {
            out.insert(f.to_string());
        }
    }
    for (n, p) in arch["parts"].as_object().into_iter().flatten() {
        let hit = match &p["egress"] {
            Value::String(e) => e == target,
            Value::Array(es) => es.iter().any(|e| s(e) == Some(target)),
            _ => false,
        };
        if hit {
            out.insert(n.clone());
        }
    }
    out
}

fn fault_of(arch: &Value, part: &str) -> Option<&'static str> {
    match block(arch, part) {
        Some("pool" | "gateway") => Some("capacity_drop"),
        Some("principal") => Some("principal_away"),
        _ => None,
    }
}

/// Plans the run of a resolved architecture: each entry as a fault, as a part down at the border,
/// or not modeled. A part that exits before it listens is down from the start.
pub fn plan(
    arch: &Value,
    horizon: f64,
    entries: &[Unavailable],
    parts: &[PartThen],
    has_given: bool,
) -> Plan {
    let mut plan = Plan::default();
    let edge = border(arch);
    let exits = |p: &str| parts.iter().any(|t| t.part == p && t.exits.is_some());
    let down_at_border = |p: &str| -> Mode {
        if !edge.contains(&p) {
            return Mode::NotModeled(format!(
                "the simulator keeps `{p}` at work: only a pool, a gateway, a principal or a border part can be down"
            ));
        }
        if has_given {
            return Mode::NotModeled(format!(
                "the given rows wait at `{p}`, and the simulator works them while it is down"
            ));
        }
        Mode::Inputs {
            part: Some(p.to_string()),
            channel: None,
        }
    };
    for u in entries {
        let until = u.until(horizon);
        let fault = |kind: &'static str, part: &str, plan: &mut Plan| {
            plan.faults.push(Fault {
                kind: match kind {
                    "principal_away" => FaultKind::PrincipalAway,
                    _ => FaultKind::CapacityDrop { to: 0.0 },
                },
                part: part.to_string(),
                from: u.from,
                until,
            });
            Mode::Fault {
                kind,
                part: part.to_string(),
            }
        };
        let mode = match &u.dependency {
            None => {
                plan.down.push((u.part.clone(), u.from, until));
                match fault_of(arch, &u.part) {
                    Some(kind) => fault(kind, &u.part, &mut plan),
                    None if block(arch, &u.part) == Some("channel") => Mode::Inputs {
                        part: None,
                        channel: Some(u.part.clone()),
                    },
                    None => down_at_border(&u.part),
                }
            }
            Some(d) => {
                plan.cuts.push((u.part.clone(), d.clone(), u.from, until));
                let only_user = users(arch, d) == BTreeSet::from([u.part.clone()]);
                let only_model = block(arch, &u.part) == Some("gateway")
                    && block(arch, d) == Some("model")
                    && crate::check::reach(arch, &u.part, "model_call")
                        .iter()
                        .filter(|m| block(arch, m) == Some("model"))
                        .count()
                        == 1;
                if exits(&u.part) && u.from == 0.0 {
                    plan.down.push((u.part.clone(), 0.0, horizon));
                    down_at_border(&u.part)
                } else if only_model {
                    fault("capacity_drop", &u.part, &mut plan)
                } else if let (true, Some(kind)) = (only_user, fault_of(arch, d)) {
                    fault(kind, d, &mut plan)
                } else if block(arch, d) == Some("channel") && edge.contains(&u.part.as_str()) {
                    Mode::Inputs {
                        part: Some(u.part.clone()),
                        channel: Some(d.clone()),
                    }
                } else {
                    Mode::NotModeled(format!(
                        "the simulator cannot take `{d}` from `{}` alone",
                        u.part
                    ))
                }
            }
        };
        let window = match &u.dependency {
            Some(_) if exits(&u.part) && u.from == 0.0 => (0.0, horizon),
            _ => (u.from, until),
        };
        plan.modes.push(mode);
        plan.windows.push(window);
    }
    plan
}

impl Plan {
    /// Whether an input at `at` from `author` over `over`, at time `t`, meets an unavailable
    /// part.
    pub fn refuses(&self, at: &str, author: &str, over: Option<&str>, t: f64) -> bool {
        let live = |from: f64, until: f64| from <= t && t < until;
        let down = |p: &str| self.down.iter().any(|(d, a, b)| d == p && live(*a, *b));
        down(at)
            || down(author)
            || over.is_some_and(down)
            || self.cuts.iter().any(|(p, d, a, b)| {
                p == at && live(*a, *b) && (d == author || Some(d.as_str()) == over)
            })
    }
}

impl Mode {
    /// Whether a simulated event about `part`, over `channel`, at `t` breaks an entry in mode
    /// `Inputs` whose window is `from..until`.
    pub fn broken_by(
        &self,
        from: f64,
        until: f64,
        t: f64,
        part: &str,
        channel: Option<&str>,
    ) -> bool {
        match self {
            Mode::Inputs {
                part: p,
                channel: c,
            } => {
                from <= t
                    && t < until
                    && p.as_deref().is_none_or(|p| p == part)
                    && c.as_deref().is_none_or(|c| Some(c) == channel)
            }
            _ => false,
        }
    }
}

/// The reason an observable of `then.parts` is not checked on the simulator.
pub fn not_checked_reason(observable: &str) -> &'static str {
    match observable {
        "view" => "the simulator serves no views",
        "command" => "the simulator decides no commands of a part",
        "exits" => "the simulator starts no processes",
        _ => "the simulator serves no views and decides no commands",
    }
}

/// The observables of `then.parts` in order: `(part, observable, as written)`.
pub fn observables(parts: &[PartThen]) -> Vec<(String, &'static str, Value)> {
    let mut out = Vec::new();
    for p in parts {
        for v in &p.views {
            out.push((p.part.clone(), "view", v.written.clone()));
        }
        for c in &p.commands {
            out.push((p.part.clone(), "command", c.written.clone()));
        }
        if let Some(st) = p.exits {
            out.push((p.part.clone(), "exits", json!({ "exits": st })));
        }
        if p.rest {
            out.push((p.part.clone(), "rest", json!({ "rest": "answers" })));
        }
    }
    out
}

/// One entry of the result's `unavailable`.
pub fn entry_json(i: usize, u: &Unavailable, mode: &Mode, window: (f64, f64)) -> Value {
    let mut m = Map::new();
    m.insert("index".into(), json!(i));
    m.insert("part".into(), json!(u.part));
    if let Some(d) = &u.dependency {
        m.insert("dependency".into(), json!(d));
    }
    m.insert("from_seconds".into(), json!(window.0));
    m.insert("until_seconds".into(), json!(window.1));
    match mode {
        Mode::Fault { kind, part } => {
            m.insert("simulated".into(), json!("fault"));
            m.insert("fault".into(), json!({ "kind": kind, "part": part }));
        }
        Mode::Inputs { part, channel } => {
            m.insert("simulated".into(), json!("inputs_refused"));
            let mut q = Map::new();
            if let Some(p) = part {
                q.insert("part".into(), json!(p));
            }
            if let Some(c) = channel {
                q.insert("channel".into(), json!(c));
            }
            m.insert("quiet".into(), Value::Object(q));
        }
        Mode::NotModeled(reason) => {
            m.insert("simulated".into(), json!("not_modeled"));
            m.insert("reason".into(), json!(reason));
        }
    }
    Value::Object(m)
}
