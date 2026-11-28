//! The ports of the building blocks, derived from their compiled ESS system
//! (`ess specify compile --path blocks --format json`), and the committed file that holds them,
//! `blocks/ports.json` (`agenticspec-ports/1`). The rule is written for users on
//! `docs/contract.md`; in short:
//!
//! - a command an ESS binding invokes is **internal**; every other command is an **entry**, and
//!   an entry the actor `agentic.architecture.Deployer` may issue is **configuration**;
//! - an event a binding is triggered by is **consumed**, every other event an **exit**;
//! - a lifecycle state is a **waiting state** when something enters it, no binding moves its
//!   entity out of it, and the world moves it on or reports into it: a command that is no
//!   configuration leaves it, or keeps the entity in it (an update, a check);
//! - the waiting states of one entity are one port: its intent is the command that enters them
//!   from outside, its reports the commands that leave them or report into them, its protocol
//!   the regular expression of every path from the intent to a state that waits no more. A port
//!   one of whose reports the actor `agentic.architecture.Principal` may issue is a
//!   **Principal** port, every other an **Outbound** port; a periodic binding is a **Clock** port.

use super::protocol::{Automaton, Node, Protocol};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use std::process::Command;

/// The format of `blocks/ports.json`.
pub const FORMAT: &str = "agenticspec-ports/1";
/// The format of a list of expected ports (`sim ports --expect`).
pub const EXPECTED_FORMAT: &str = "agenticspec-ports-expected/1";
/// The actor whose commands are configuration.
pub const DEPLOYER: &str = "agentic.architecture.Deployer";
/// The actor whose commands end a wait on a principal.
pub const PRINCIPAL: &str = "agentic.architecture.Principal";

/// The kind of a port (`agentic.architecture.PortKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Outbound,
    Clock,
    Principal,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Outbound => "Outbound",
            Kind::Clock => "Clock",
            Kind::Principal => "Principal",
        }
    }

    pub fn parse(text: &str) -> Option<Kind> {
        match text {
            "Outbound" => Some(Kind::Outbound),
            "Clock" => Some(Kind::Clock),
            "Principal" => Some(Kind::Principal),
            _ => None,
        }
    }
}

/// One derived port, with the fields of `agentic.architecture.Port`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Port {
    /// `<block>.<entity>`, the entity of its waiting state in kebab case without the block's
    /// own name in front: `model.inference`, `harness.session`, `harness.tool-call`.
    pub name: String,
    /// The block (the domain's last segment) of its commands.
    pub block: String,
    pub kind: Kind,
    /// The event the intent publishes; absent for a clock port.
    pub intent_event: Option<String>,
    /// `<entity>.<state>`, the state the intent leaves the entity in; absent for a clock port.
    pub waiting_state: Option<String>,
    /// The commands that leave a waiting state of the port or report into it, qualified.
    pub reports: Vec<String>,
    pub protocol: Protocol,
}

impl Port {
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("name".into(), json!(self.name));
        m.insert("block".into(), json!(self.block));
        m.insert("kind".into(), json!(self.kind.as_str()));
        if let Some(e) = &self.intent_event {
            m.insert("intent_event".into(), json!(e));
        }
        if let Some(s) = &self.waiting_state {
            m.insert("waiting_state".into(), json!(s));
        }
        m.insert("reports".into(), json!(self.reports));
        m.insert("protocol".into(), json!(self.protocol.to_string()));
        Value::Object(m)
    }

    pub fn from_json(v: &Value) -> Result<Port, String> {
        let text = |k: &str| -> Result<String, String> {
            v[k].as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("port {v}: `{k}` is missing"))
        };
        let name = text("name")?;
        let kind =
            Kind::parse(&text("kind")?).ok_or_else(|| format!("port {name}: unknown kind"))?;
        Ok(Port {
            block: text("block")?,
            kind,
            intent_event: v["intent_event"].as_str().map(str::to_owned),
            waiting_state: v["waiting_state"].as_str().map(str::to_owned),
            reports: v["reports"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|r| r.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            protocol: Protocol::parse(&text("protocol")?)
                .map_err(|e| format!("port {name}: {e}"))?,
            name,
        })
    }

    /// The short names of every command of the port.
    pub fn commands(&self) -> BTreeSet<String> {
        self.protocol.names()
    }
}

/// How a command is caused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// An ESS binding invokes it.
    Internal,
    /// No binding invokes it: the world issues it.
    Entry,
    /// An entry the deployer issues.
    Configuration,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Class::Internal => "internal",
            Class::Entry => "entry",
            Class::Configuration => "configuration",
        })
    }
}

/// What a command is to an interface of an architecture (`docs/contract.md`, "The surface"):
/// the first that applies, in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// An ESS binding invokes it.
    Internal,
    /// It reports into a waiting state of a port: the world's, or a principal's, answer.
    Report,
    /// It states the intent of a port (the first command of its protocol).
    Intent,
    /// An accepting outcome creates its entity: set up from the architecture document only.
    Configuration,
    /// Every accepting outcome moves or updates an existing entity: an operator's at run time.
    Administration,
    /// Any other entry command.
    Entry,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Internal => "internal",
            Role::Report => "report",
            Role::Intent => "intent",
            Role::Configuration => "configuration",
            Role::Administration => "administration",
            Role::Entry => "entry",
        }
    }

    pub fn parse(text: &str) -> Option<Role> {
        [
            Role::Internal,
            Role::Report,
            Role::Intent,
            Role::Configuration,
            Role::Administration,
            Role::Entry,
        ]
        .into_iter()
        .find(|r| r.as_str() == text)
    }
}

/// The commands, views and events of one domain of the blocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DomainSurface {
    pub commands: BTreeMap<String, Role>,
    pub views: Vec<String>,
    pub events: Vec<String>,
}

/// What an interface of an architecture selects from: per domain its commands with their role,
/// its views and its events, and per ESS actor the commands it may issue.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Surface {
    pub actors: BTreeMap<String, Vec<String>>,
    pub domains: BTreeMap<String, DomainSurface>,
}

impl Surface {
    /// The role of a command, when it is a command of the blocks.
    pub fn role(&self, command: &str) -> Option<Role> {
        let (domain, _) = command.rsplit_once('.')?;
        self.domains.get(domain)?.commands.get(command).copied()
    }

    pub fn has_view(&self, view: &str) -> bool {
        view.rsplit_once('.')
            .and_then(|(d, _)| self.domains.get(d))
            .is_some_and(|d| d.views.iter().any(|v| v == view))
    }

    pub fn has_event(&self, event: &str) -> bool {
        event
            .rsplit_once('.')
            .and_then(|(d, _)| self.domains.get(d))
            .is_some_and(|d| d.events.iter().any(|v| v == event))
    }

    fn to_json(&self) -> (Value, Value) {
        let domains: Map<String, Value> = self
            .domains
            .iter()
            .map(|(name, d)| {
                let commands: Map<String, Value> = d
                    .commands
                    .iter()
                    .map(|(c, r)| (c.clone(), json!(r.as_str())))
                    .collect();
                (
                    name.clone(),
                    json!({ "commands": commands, "views": d.views, "events": d.events }),
                )
            })
            .collect();
        (json!(self.actors), Value::Object(domains))
    }

    fn from_json(actors: &Value, domains: &Value) -> Result<Surface, String> {
        let mut out = Surface::default();
        for (name, may) in actors.as_object().into_iter().flatten() {
            out.actors.insert(name.clone(), strings(may));
        }
        for (name, d) in domains.as_object().into_iter().flatten() {
            let mut commands = BTreeMap::new();
            for (c, r) in d["commands"].as_object().into_iter().flatten() {
                let role = r
                    .as_str()
                    .and_then(Role::parse)
                    .ok_or_else(|| format!("command {c}: unknown role {r}"))?;
                commands.insert(c.clone(), role);
            }
            out.domains.insert(
                name.clone(),
                DomainSurface {
                    commands,
                    views: strings(&d["views"]),
                    events: strings(&d["events"]),
                },
            );
        }
        Ok(out)
    }
}

/// The derivation: every command's class, every event's consumption, and the ports.
#[derive(Debug)]
pub struct Derivation {
    pub commands: BTreeMap<String, Class>,
    /// Event name to whether a binding is triggered by it.
    pub events: BTreeMap<String, bool>,
    /// Binding name to the command it invokes.
    pub bindings: BTreeMap<String, String>,
    pub ports: Vec<Port>,
    pub surface: Surface,
}

/// One change of state by one command.
#[derive(Clone, Debug)]
struct Edge {
    entity: String,
    from: Option<String>,
    to: String,
    command: String,
    event: Option<String>,
    default: bool,
    /// A side effect (`affects:`) of an outcome whose subject changes state itself: the entity
    /// mirrors another one and the move is not its own report.
    mirror: bool,
}

/// A command that keeps an entity in a state and reports into it.
#[derive(Clone, Debug)]
struct Report {
    entity: String,
    /// The state it reports into; every state that is not terminal when absent.
    state: Option<String>,
    command: String,
}

fn short(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

fn block_of(name: &str) -> String {
    name.split('.').nth(1).unwrap_or(name).to_owned()
}

fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `<block>.<entity>` for an entity `agentic.<block>.<Entity>`.
pub fn port_name(entity: &str) -> String {
    let block = block_of(entity);
    let e = short(entity);
    let lower = e.to_ascii_lowercase();
    let rest = if lower.starts_with(&block) && e.len() > block.len() {
        &e[block.len()..]
    } else {
        e
    };
    let rest = if rest.starts_with(|c: char| c.is_ascii_uppercase()) {
        rest
    } else {
        e
    };
    format!("{block}.{}", kebab(rest))
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Whether some outcome of `command` guards on the state of a related row of `entity`.
fn guards_state_of(command: &Value, entity: &str) -> bool {
    fn walk(v: &Value, entity: &str) -> bool {
        match v {
            Value::Object(m) => {
                if m.get("kind").and_then(Value::as_str) == Some("related")
                    && m.get("entity").and_then(Value::as_str) == Some(entity)
                    && m.get("test")
                        .is_some_and(|t| t.to_string().contains("state"))
                {
                    return true;
                }
                m.values().any(|x| walk(x, entity))
            }
            Value::Array(a) => a.iter().any(|x| walk(x, entity)),
            _ => false,
        }
    }
    command["outcomes"]
        .as_array()
        .is_some_and(|os| os.iter().any(|o| walk(&o["condition"], entity)))
}

/// Derive the ports from a compiled system.
pub fn derive(ir: &Value) -> Result<Derivation, Vec<String>> {
    let mut refusals = Vec::new();
    if let Some(types) = ir["types"].as_object()
        && let Some(t) = types
            .iter()
            .find(|(_, t)| t.to_string().contains("\"reading\""))
    {
        refusals.push(format!(
            "type {} has a clock-reading attachment, which this derivation does not read yet",
            t.0
        ));
    }
    let empty = Map::new();
    let commands = ir["commands"].as_object().unwrap_or(&empty);
    let entities = ir["entities"].as_object().unwrap_or(&empty);
    let bindings = ir["bindings"].as_object().unwrap_or(&empty);
    let actor = |name: &str| -> BTreeSet<String> {
        strings(&ir["actors"][name]["may"]).into_iter().collect()
    };
    let deployer = actor(DEPLOYER);
    let principal = actor(PRINCIPAL);

    let bound: BTreeSet<String> = bindings
        .values()
        .filter_map(|b| b["command"].as_str().map(str::to_owned))
        .collect();
    let consumed: BTreeSet<String> = bindings
        .values()
        .filter_map(|b| b["event"].as_str().map(str::to_owned))
        .collect();

    let mut class = BTreeMap::new();
    for name in commands.keys() {
        let c = if bound.contains(name) {
            Class::Internal
        } else if deployer.contains(name) {
            Class::Configuration
        } else {
            Class::Entry
        };
        class.insert(name.clone(), c);
    }
    let mut events = BTreeMap::new();
    if let Some(evs) = ir["events"].as_object() {
        for name in evs.keys() {
            events.insert(name.clone(), consumed.contains(name));
        }
    }

    let identity_of: BTreeMap<String, String> = entities
        .iter()
        .filter_map(|(n, e)| {
            e["identity"]["type_ref"]["name"]
                .as_str()
                .map(|t| (t.to_owned(), n.clone()))
        })
        .collect();
    let terminal = |entity: &str| -> BTreeSet<String> {
        strings(&entities[entity]["lifecycle"]["terminal"])
            .into_iter()
            .collect()
    };

    let mut edges: Vec<Edge> = Vec::new();
    let mut reports: Vec<Report> = Vec::new();
    // Records: a command that creates a row of another entity straight into a terminal state,
    // names a row of this entity by its identity and guards on that row's state (a turn of a
    // session, a step of a run). It reports into the waiting states of the named entity.
    let mut records: Vec<(String, String)> = Vec::new();
    for (cname, c) in commands {
        for o in c["outcomes"].as_array().into_iter().flatten() {
            if !o["error"].is_null() {
                continue;
            }
            let event = o["emits"][0].as_str().map(str::to_owned);
            let default = o["condition"]["kind"] == "otherwise";
            let subject = &o["subject"];
            let effect = subject["effect"].as_str().unwrap_or("");
            let entity = subject["entity"].as_str().unwrap_or("").to_owned();
            let affects_moves: Vec<&Value> = o["affects"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|a| a["moves"].is_object())
                .collect();
            let subject_moves = matches!(effect, "creates" | "moves");
            match effect {
                "creates" => {
                    let to = subject["into"]
                        .as_str()
                        .or_else(|| entities[&entity]["lifecycle"]["initial"].as_str())
                        .unwrap_or("")
                        .to_owned();
                    if terminal(&entity).contains(&to) {
                        for field in c["input"].as_array().into_iter().flatten() {
                            if let Some(named) = field["type_ref"]["name"]
                                .as_str()
                                .and_then(|t| identity_of.get(t))
                                && *named != entity
                                && guards_state_of(c, named)
                            {
                                records.push((named.clone(), cname.clone()));
                            }
                        }
                    }
                    edges.push(Edge {
                        entity: entity.clone(),
                        from: None,
                        to,
                        command: cname.clone(),
                        event: event.clone(),
                        default,
                        mirror: false,
                    });
                }
                "moves" => {
                    let to = subject["transition"]["to"]
                        .as_str()
                        .unwrap_or("")
                        .to_owned();
                    for from in strings(&subject["transition"]["from"]) {
                        if from == to {
                            reports.push(Report {
                                entity: entity.clone(),
                                state: Some(from),
                                command: cname.clone(),
                            });
                        } else {
                            edges.push(Edge {
                                entity: entity.clone(),
                                from: Some(from),
                                to: to.clone(),
                                command: cname.clone(),
                                event: event.clone(),
                                default,
                                mirror: false,
                            });
                        }
                    }
                }
                "updates" | "preserves" if affects_moves.is_empty() => {
                    let state = (o["condition"]["kind"] == "subject_state")
                        .then(|| o["condition"]["state"].as_str().map(str::to_owned))
                        .flatten();
                    reports.push(Report {
                        entity: entity.clone(),
                        state,
                        command: cname.clone(),
                    });
                }
                _ => {}
            }
            for a in affects_moves {
                let to = a["moves"]["to"].as_str().unwrap_or("").to_owned();
                for from in strings(&a["moves"]["from"]) {
                    if from != to {
                        edges.push(Edge {
                            entity: a["entity"].as_str().unwrap_or("").to_owned(),
                            from: Some(from),
                            to: to.clone(),
                            command: cname.clone(),
                            event: event.clone(),
                            default,
                            mirror: subject_moves,
                        });
                    }
                }
            }
        }
    }

    let is_config = |c: &str| class.get(c) == Some(&Class::Configuration);
    let is_internal = |c: &str| class.get(c) == Some(&Class::Internal);
    let mut ports = Vec::new();
    let mut intent_commands: BTreeSet<String> = BTreeSet::new();
    for (ename, e) in entities {
        let term = terminal(ename);
        let exits = |s: &str| -> Vec<&Edge> {
            edges
                .iter()
                .filter(|x| {
                    x.entity == *ename && !x.mirror && x.from.as_deref() == Some(s) && x.to != s
                })
                .collect()
        };
        // A terminal state may still have a way out (a message delivered and maybe read): the
        // protocol may end in it or go on. A report into it must name it.
        let states: Vec<String> = strings(&e["lifecycle"]["states"])
            .into_iter()
            .filter(|s| !term.contains(s) || exits(s).iter().any(|x| !is_config(&x.command)))
            .collect();
        let reports_into = |s: &str| -> BTreeSet<String> {
            reports
                .iter()
                .filter(|r| {
                    r.entity == *ename
                        && match r.state.as_deref() {
                            Some(x) => x == s,
                            None => !term.contains(s),
                        }
                })
                .map(|r| r.command.clone())
                .collect()
        };
        let entered = |s: &str| {
            edges
                .iter()
                .any(|x| x.entity == *ename && x.to == s && x.from.as_deref() != Some(s))
        };
        let waiting: BTreeSet<String> = states
            .iter()
            .filter(|s| {
                let out = exits(s);
                entered(s)
                    && out.iter().all(|x| !is_internal(&x.command))
                    && (out.iter().any(|x| !is_config(&x.command))
                        || reports_into(s).iter().any(|c| !is_config(c)))
            })
            .cloned()
            .collect();
        if waiting.is_empty() {
            continue;
        }
        let mut auto = Automaton::default();
        let mut intents: Vec<&Edge> = Vec::new();
        let mut report_set: BTreeSet<String> = BTreeSet::new();
        for x in edges.iter().filter(|x| x.entity == *ename) {
            let into_waiting = waiting.contains(&x.to);
            let from_waiting = x.from.as_ref().is_some_and(|f| waiting.contains(f));
            if into_waiting && !from_waiting && x.from.as_deref() != Some(x.to.as_str()) {
                intents.push(x);
                auto.edge(
                    Node::Start,
                    Node::State(x.to.clone()),
                    Protocol::name(short(&x.command)),
                );
            }
        }
        for s in &waiting {
            let out = exits(s);
            let rests = term.contains(s);
            let mut accepting = rests || out.iter().all(|x| is_config(&x.command));
            for x in out {
                if is_config(&x.command) {
                    accepting = true;
                    continue;
                }
                report_set.insert(x.command.clone());
                let to = if waiting.contains(&x.to) {
                    Node::State(x.to.clone())
                } else {
                    Node::End
                };
                auto.edge(
                    Node::State(s.clone()),
                    to,
                    Protocol::name(short(&x.command)),
                );
            }
            let mut inside: BTreeSet<String> = reports_into(s)
                .into_iter()
                .filter(|c| !is_config(c))
                .collect();
            if !rests {
                inside.extend(
                    records
                        .iter()
                        .filter(|(n, c)| n == ename && !is_config(c))
                        .map(|(_, c)| c.clone()),
                );
            }
            for c in inside {
                report_set.insert(c.clone());
                auto.edge(
                    Node::State(s.clone()),
                    Node::State(s.clone()),
                    Protocol::name(short(&c)),
                );
            }
            if accepting {
                auto.edge(Node::State(s.clone()), Node::End, Protocol::Empty);
            }
        }
        intents
            .sort_by(|a, b| (!a.default, &a.command, &a.to).cmp(&(!b.default, &b.command, &b.to)));
        let Some(first) = intents.first() else {
            continue;
        };
        let intent_names: BTreeSet<String> = intents
            .iter()
            .map(|x| short(&x.command).to_owned())
            .collect();
        let protocol = auto.protocol();
        let name = port_name(ename);
        let allowed: BTreeSet<String> = intent_names
            .iter()
            .cloned()
            .chain(report_set.iter().map(|r| short(r).to_owned()))
            .collect();
        if protocol == Protocol::Empty {
            refusals.push(format!("port {name}: no path leaves its waiting states"));
            continue;
        }
        if let Some(stray) = protocol.names().difference(&allowed).next() {
            refusals.push(format!(
                "port {name}: protocol `{protocol}` names {stray}, which is no command of the port"
            ));
        }
        if let Err(e) = Protocol::parse(&protocol.to_string()).and_then(|p| p.compile()) {
            refusals.push(format!(
                "port {name}: protocol `{protocol}` does not parse: {e}"
            ));
        }
        let kind = if report_set.iter().any(|r| principal.contains(r)) {
            Kind::Principal
        } else {
            Kind::Outbound
        };
        intent_commands.extend(intents.iter().map(|x| x.command.clone()));
        ports.push(Port {
            name,
            block: block_of(ename),
            kind,
            intent_event: first.event.clone(),
            waiting_state: Some(format!("{ename}.{}", first.to)),
            reports: report_set.into_iter().collect(),
            protocol,
        });
    }
    for (bname, b) in bindings {
        if b["event"].is_null()
            && let Some(cmd) = b["command"].as_str()
        {
            ports.push(Port {
                name: format!("{}.{bname}", block_of(cmd)),
                block: block_of(cmd),
                kind: Kind::Clock,
                intent_event: None,
                waiting_state: None,
                reports: vec![cmd.to_owned()],
                protocol: Protocol::name(short(cmd)),
            });
        }
    }
    ports.sort_by(|a, b| a.name.cmp(&b.name));
    let mut names = BTreeSet::new();
    for p in &ports {
        if !names.insert(p.name.clone()) {
            refusals.push(format!("port {}: two ports of one name", p.name));
        }
    }
    if refusals.is_empty() {
        let surface = surface(ir, &bound, &ports, &intent_commands);
        Ok(Derivation {
            commands: class,
            events,
            bindings: bindings
                .iter()
                .filter_map(|(n, b)| b["command"].as_str().map(|c| (n.clone(), c.to_owned())))
                .collect(),
            ports,
            surface,
        })
    } else {
        Err(refusals)
    }
}

/// The surface of a compiled system: each command's [`Role`], each domain's views and events,
/// each actor's `may:`.
fn surface(
    ir: &Value,
    bound: &BTreeSet<String>,
    ports: &[Port],
    intents: &BTreeSet<String>,
) -> Surface {
    let reports: BTreeSet<&str> = ports
        .iter()
        .flat_map(|p| p.reports.iter().map(String::as_str))
        .collect();
    let mut out = Surface::default();
    let domain_of = |v: &Value, name: &str| {
        v["domain"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| name.rsplit_once('.').map_or("", |x| x.0).to_owned())
    };
    for (name, c) in ir["commands"].as_object().into_iter().flatten() {
        let effects: Vec<&str> = c["outcomes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|o| o["error"].is_null())
            .map(|o| o["subject"]["effect"].as_str().unwrap_or(""))
            .collect();
        let role = if bound.contains(name) {
            Role::Internal
        } else if reports.contains(name.as_str()) {
            Role::Report
        } else if intents.contains(name) {
            Role::Intent
        } else if effects.contains(&"creates") {
            Role::Configuration
        } else if !effects.is_empty() && effects.iter().all(|e| matches!(*e, "moves" | "updates")) {
            Role::Administration
        } else {
            Role::Entry
        };
        out.domains
            .entry(domain_of(c, name))
            .or_default()
            .commands
            .insert(name.clone(), role);
    }
    for (key, list) in [("views", 0), ("events", 1)] {
        for (name, v) in ir[key].as_object().into_iter().flatten() {
            let d = out.domains.entry(domain_of(v, name)).or_default();
            if list == 0 {
                d.views.push(name.clone());
            } else {
                d.events.push(name.clone());
            }
        }
    }
    for d in out.domains.values_mut() {
        d.views.sort();
        d.events.sort();
    }
    for (name, a) in ir["actors"].as_object().into_iter().flatten() {
        let mut may = strings(&a["may"]);
        may.sort();
        out.actors.insert(name.clone(), may);
    }
    out
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The files of a blocks directory the digest covers: `ess-inputs.yaml` and every file its
/// `specification:` and `scenarios:` lists name, relative to the directory.
pub fn sources(blocks: &Path) -> Result<Vec<String>, String> {
    let inputs = blocks.join("ess-inputs.yaml");
    let doc = crate::doc::load_yaml(&inputs)?;
    let mut out = vec!["ess-inputs.yaml".to_owned()];
    for key in ["specification", "scenarios"] {
        out.extend(strings(&doc[key]));
    }
    Ok(out)
}

/// The SHA-256 digest of the sources of the ports: the `ess --version` line, then one
/// `<sha256> <path>` line per file of [`sources`].
pub fn sources_digest(blocks: &Path, ess: &str) -> Result<String, String> {
    let mut text = format!("{ess}\n");
    for file in sources(blocks)? {
        let path = blocks.join(&file);
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        text.push_str(&format!("{} {file}\n", hex(&bytes)));
    }
    Ok(hex(text.as_bytes()))
}

/// Run `ess` with these arguments: its standard output, or its standard error as the refusal.
pub fn ess(args: &[&str]) -> Result<String, String> {
    let out = Command::new("ess")
        .args(args)
        .output()
        .map_err(|e| format!("ess {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "ess {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The `ess --version` line.
pub fn ess_version() -> Result<String, String> {
    Ok(ess(&["--version"])?.trim().to_owned())
}

/// `ess specify compile --path <blocks> --format json`.
pub fn compile(blocks: &Path) -> Result<Value, String> {
    let path = blocks.display().to_string();
    let text = ess(&["specify", "compile", "--path", &path, "--format", "json"])?;
    serde_json::from_str(&text).map_err(|e| format!("ess specify compile: {e}"))
}

/// The ports file of a derivation, as written: pretty JSON with a final newline.
pub fn file_text(
    ports: &[Port],
    bindings: &BTreeMap<String, String>,
    surface: &Surface,
    ess: &str,
    digest: &str,
) -> String {
    let (actors, domains) = surface.to_json();
    let doc = json!({
        "format": FORMAT,
        "ess": ess,
        "sources_sha256": digest,
        "bindings": bindings,
        "ports": ports.iter().map(Port::to_json).collect::<Vec<_>>(),
        "actors": actors,
        "domains": domains,
    });
    let mut text = serde_json::to_string_pretty(&doc).unwrap_or_default();
    text.push('\n');
    text
}

/// A ports file read back.
#[derive(Debug, Clone)]
pub struct PortsFile {
    pub ess: String,
    pub sources_sha256: String,
    /// Binding name to the command it invokes: the commands no entry of the system.
    pub bindings: BTreeMap<String, String>,
    pub ports: Vec<Port>,
    /// What an interface selects from; empty in a file written before it was derived.
    pub surface: Surface,
}

impl PortsFile {
    pub fn parse(text: &str) -> Result<PortsFile, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if v["format"] != FORMAT {
            return Err(format!("not {FORMAT}"));
        }
        let ports = v["ports"]
            .as_array()
            .ok_or("no `ports`")?
            .iter()
            .map(Port::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PortsFile {
            ess: v["ess"].as_str().unwrap_or_default().to_owned(),
            sources_sha256: v["sources_sha256"].as_str().unwrap_or_default().to_owned(),
            bindings: v["bindings"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, c)| c.as_str().map(|c| (k.clone(), c.to_owned())))
                        .collect()
                })
                .unwrap_or_default(),
            ports,
            surface: Surface::from_json(&v["actors"], &v["domains"])?,
        })
    }

    pub fn load(path: &Path) -> Result<PortsFile, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        PortsFile::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn port(&self, name: &str) -> Option<&Port> {
        self.ports.iter().find(|p| p.name == name)
    }
}

/// One row of an expected list.
#[derive(Debug, Clone)]
pub struct Expected {
    pub port: String,
    pub block: String,
    pub kind: Option<Kind>,
    pub protocol: Protocol,
}

/// Read an expected list (`agenticspec-ports-expected/1`).
pub fn load_expected(path: &Path) -> Result<Vec<Expected>, String> {
    let doc = crate::doc::load_yaml(path)?;
    if doc["format"] != EXPECTED_FORMAT {
        return Err(format!("{}: not {EXPECTED_FORMAT}", path.display()));
    }
    doc["ports"]
        .as_array()
        .ok_or_else(|| format!("{}: no `ports`", path.display()))?
        .iter()
        .map(|row| {
            let port = row["port"].as_str().unwrap_or_default().to_owned();
            Ok(Expected {
                block: row["block"].as_str().unwrap_or_default().to_owned(),
                kind: match row["kind"].as_str() {
                    Some(k) => Some(Kind::parse(k).ok_or_else(|| format!("{port}: kind {k}"))?),
                    None => None,
                },
                protocol: Protocol::parse(row["protocol"].as_str().unwrap_or_default())
                    .map_err(|e| format!("{port}: {e}"))?,
                port,
            })
        })
        .collect()
}

fn list(names: &BTreeSet<String>) -> String {
    names.iter().cloned().collect::<Vec<_>>().join(", ")
}

/// One line per difference between derived ports and an expected list, in the order of the
/// list, then the derived ports no row matches. A row matches the derived ports whose intent is
/// a first command of its protocol; failing that, the ports that share any of its commands.
pub fn compare(derived: &[Port], expected: &[Expected]) -> Vec<String> {
    let intents = |p: &Port| p.protocol.first();
    let mut matched_by: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut lines = Vec::new();
    for x in expected {
        let first = x.protocol.first();
        let cmds = x.protocol.names();
        let mut hits: Vec<&Port> = derived
            .iter()
            .filter(|p| !intents(p).is_disjoint(&first))
            .collect();
        if hits.is_empty() {
            hits = derived
                .iter()
                .filter(|p| !p.commands().is_disjoint(&cmds))
                .collect();
        }
        for p in &hits {
            matched_by
                .entry(p.name.clone())
                .or_default()
                .push(x.port.clone());
        }
        match hits.as_slice() {
            [] => lines.push(format!(
                "missing {}: no derived port has any of its commands ({})",
                x.port,
                list(&cmds)
            )),
            [p] => {
                let at = format!("{} -> {}", x.port, p.name);
                if x.block != p.block {
                    lines.push(format!(
                        "block {at}: expected {}, derived {}",
                        x.block, p.block
                    ));
                }
                if let Some(k) = x.kind
                    && k != p.kind
                {
                    lines.push(format!(
                        "kind {at}: expected {}, derived {}",
                        k.as_str(),
                        p.kind.as_str()
                    ));
                }
                if first != intents(p) {
                    lines.push(format!(
                        "intent {at}: expected {}, derived {}",
                        list(&first),
                        list(&intents(p))
                    ));
                }
                let xr: BTreeSet<String> = cmds.difference(&first).cloned().collect();
                let pr: BTreeSet<String> = p.reports.iter().map(|r| short(r).to_owned()).collect();
                if xr != pr {
                    lines.push(format!(
                        "reports {at}: expected [{}], derived [{}]",
                        list(&xr),
                        list(&pr)
                    ));
                }
                if x.protocol.to_string() != p.protocol.to_string() {
                    lines.push(format!(
                        "protocol {at}: expected `{}`, derived `{}`",
                        x.protocol, p.protocol
                    ));
                }
            }
            many => {
                let names: Vec<&str> = many.iter().map(|p| p.name.as_str()).collect();
                lines.push(format!(
                    "split {}: its commands ({}) are in {}",
                    x.port,
                    list(&cmds),
                    names.join(", ")
                ));
            }
        }
    }
    for (name, rows) in &matched_by {
        if rows.len() > 1 {
            lines.push(format!("merged {} -> {name}", rows.join(", ")));
        }
    }
    for p in derived {
        if !matched_by.contains_key(&p.name) {
            lines.push(format!(
                "extra {} ({}): `{}`",
                p.name,
                p.kind.as_str(),
                p.protocol
            ));
        }
    }
    lines
}
