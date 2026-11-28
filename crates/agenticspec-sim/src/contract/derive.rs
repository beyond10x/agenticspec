//! The contract of one architecture document (`agenticspec-contract/1`): the ports its parts use,
//! each with the port's protocol, the settings of the part that narrow it and its number of
//! instances; the routing invariant of each link; the border; and the ingresses, each interface
//! resolved into operations (`super::ingress`). Derived from the document and the ports file
//! (`blocks/ports.json`) alone, with the KPI catalogue for the metrics of an interface, without a
//! runtime; `docs/contract.md` states the rules and the format.
//!
//! A contract is one document's: a document that names another as a part would get the contract
//! of each, composed by the document that names them (`docs/contract.md`, "Nested
//! architectures").

use super::ingress::{ContractIngress, OpKind, Operation};
use super::ports::PortsFile;
use super::predicate::{Clause, Predicate, Use};
use crate::check::value;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const FORMAT: &str = "agenticspec-contract/1";

/// One port one part uses.
#[derive(Debug, Clone, PartialEq)]
pub struct ContractPort {
    pub port: String,
    pub part: String,
    /// The port's protocol, copied from the ports file.
    pub protocol: String,
    /// The settings of the part that narrow the port, by name.
    pub settings: BTreeMap<String, Value>,
    /// The part's count, times the count of the part it is `per`, along that chain.
    pub instances: u64,
}

/// The routing invariant of one link.
#[derive(Debug, Clone, PartialEq)]
pub struct Invariant {
    /// `<from> -> <to> carries <value>`, then ` over <channel>` when the link names one.
    pub link: String,
    /// The port of the link's `from` part the invariant is about.
    pub port: String,
    /// The port of the link's `to` part every use of `port` lies inside, when there is one.
    pub within: Option<String>,
    /// The predicate as text; [`Predicate::parse`] reads it.
    pub predicate: String,
}

/// The contract of one architecture document.
#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    pub architecture: String,
    /// The `sources_sha256` of the ports file the contract was derived from.
    pub ports_digest: String,
    pub ports: Vec<ContractPort>,
    pub invariants: Vec<Invariant>,
    pub border: Vec<String>,
    /// The document's ingresses, each interface resolved into operations; empty when the
    /// document declares none, and then not written.
    pub ingress: Vec<ContractIngress>,
}

/// The ports a part of a block uses by being there. Blocks of an inner domain (principal, tool,
/// knowledge, work, experiment, kpi) use none: the specification runs them. A principal uses the
/// question port when a link carrying `question` reaches it (see [`link_uses`]).
fn block_ports(doc: &Value, name: &str, part: &Value) -> Result<Vec<&'static str>, String> {
    let block = part["block"].as_str().unwrap_or("");
    Ok(match block {
        "agent" => vec!["harness.session"],
        "model" => vec!["model.inference"],
        "gateway" => vec!["gateway.call"],
        "pool" => vec!["resource.claim"],
        "channel" => {
            if shared(part) {
                vec!["channel.post", "channel.subscription"]
            } else {
                vec!["channel.message"]
            }
        }
        "workspace" => vec!["workspace.lease", "workspace.proposal"],
        "sandbox" => vec!["sandbox.sandbox"],
        "schedule" => {
            // A trigger part that names the schedule fires on it; otherwise the schedule fires.
            let named = doc["parts"].as_object().is_some_and(|parts| {
                parts.values().any(|p| {
                    p["block"] == "trigger" && value(&p["schedule"]).as_str() == Some(name)
                })
            });
            if named {
                vec!["trigger.schedule"]
            } else {
                vec!["trigger.schedule", "trigger.firing"]
            }
        }
        "trigger" => vec!["trigger.firing"],
        "authority" => {
            if value(&part["kind"]).as_str() == Some("budget") {
                vec!["authority.budget"]
            } else {
                vec!["authority.decision"]
            }
        }
        "runtime" => vec!["runtime.instance"],
        "principal" | "tool" | "knowledge" | "work" | "experiment" | "kpi" => vec![],
        other => {
            return Err(format!(
                "part {name}: block `{other}` has no entry in the table of the ports a block uses"
            ));
        }
    })
}

/// The settings of a part of `block` copied into the contract for a port it uses; any other
/// setting is not copied. A port a part uses through a link (an agent's inferences, a principal
/// sending over a channel) takes no setting of that part, except the kind of a principal that
/// answers questions.
fn narrowing(block: &str, port: &str) -> &'static [&'static str] {
    match (block, port) {
        ("agent", "harness.session") => &["harness", "role", "acts_for"],
        ("agent", "harness.tool-call") => &["harness"],
        ("model", "model.inference") => &["route"],
        ("gateway", "gateway.call") => &["rate_limit_per_minute"],
        ("pool", "resource.claim") => &["kind", "capacity", "job_limit"],
        ("channel", _) => &["kind", "distribution"],
        ("principal", "question.question") => &["kind"],
        ("trigger" | "schedule", "trigger.firing") => &[
            "kind",
            "every",
            "schedule",
            "channel",
            "policy",
            "max_parallel",
            "event_name",
            "source",
        ],
        ("schedule", "trigger.schedule") => &["every", "next_fire_at"],
        ("workspace", _) => &["kind", "sharing"],
        ("sandbox", _) => &["isolation", "egress"],
        ("authority", _) => &["kind"],
        ("runtime", _) => &["kind"],
        _ => &[],
    }
}

fn shared(part: &Value) -> bool {
    matches!(
        value(&part["distribution"]).as_str(),
        Some("topic" | "queue" | "board")
    )
}

fn text(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}

/// The name of a link: `<from> -> <to> carries <value>`, then ` over <channel>`.
pub fn link_name(link: &Value) -> String {
    let mut name = format!(
        "{} -> {} carries {}",
        text(&link["from"]),
        text(&link["to"]),
        text(&link["carries"])
    );
    if let Some(over) = link["over"].as_str() {
        name.push_str(&format!(" over {over}"));
    }
    name
}

/// The port a channel part carries a sent value in: a post on a shared channel, a message on a
/// direct one.
fn sending_port(channel: &Value) -> &'static str {
    if shared(channel) {
        "channel.post"
    } else {
        "channel.message"
    }
}

/// What one link makes its parts use: the port of `from` the carried value travels in (the
/// invariant's port), with the part of `to` it lies inside when there is one, and the ports
/// `from` and `to` use because of it.
struct LinkUse {
    port: Option<&'static str>,
    within: Option<&'static str>,
    uses: Vec<(&'static str, String)>,
}

fn link_uses(doc: &Value, link: &Value) -> LinkUse {
    let parts = &doc["parts"];
    let from = text(&link["from"]);
    let to = text(&link["to"]);
    let from_block = text(&parts[from]["block"]);
    let to_part = &parts[to];
    let to_block = text(&to_part["block"]);
    let carries = text(&link["carries"]);
    let mut uses = Vec::new();
    let (port, within) = match (carries, from_block, to_block) {
        ("model_call", "agent" | "runtime", "gateway") => {
            (Some("model.inference"), Some("gateway.call"))
        }
        ("model_call", "agent" | "runtime", _) => (Some("model.inference"), None),
        ("model_call", "gateway", _) => (Some("gateway.call"), None),
        ("question", _, _) => {
            if to_block == "principal" {
                uses.push(("question.question", to.to_owned()));
            }
            (Some("question.question"), None)
        }
        ("call", "agent", "tool") => (Some("harness.tool-call"), None),
        ("claim", _, "pool") => (Some("resource.claim"), None),
        ("read", _, "channel") if shared(to_part) => (Some("channel.subscription"), None),
        (_, _, "channel") if carries != "read" => (Some(sending_port(to_part)), None),
        ("message", _, _) => (Some("channel.message"), None),
        ("proposal", "agent", _) => (Some("workspace.proposal"), None),
        ("order" | "run", _, "runtime") => (Some("runtime.instance"), None),
        ("trigger", "schedule" | "trigger", _) => (Some("trigger.firing"), None),
        ("grant", "authority", _) => {
            let budget = value(&parts[from]["kind"]).as_str() == Some("budget");
            (
                Some(if budget {
                    "authority.budget"
                } else {
                    "authority.decision"
                }),
                None,
            )
        }
        ("checkout", "workspace", _) => (Some("workspace.lease"), None),
        _ => (None, None),
    };
    // A value carried over a channel part travels in that channel's sending port of `from`.
    let over = link["over"]
        .as_str()
        .filter(|c| parts[*c]["block"] == "channel");
    let transport = over.map(|c| sending_port(&parts[c]));
    if let Some(t) = transport {
        uses.push((t, from.to_owned()));
    }
    let port = port.or(transport);
    if let Some(p) = port {
        uses.push((p, from.to_owned()));
    }
    LinkUse { port, within, uses }
}

/// The port of the link's `from` part its carried value travels in, when there is one.
pub fn link_port(doc: &Value, link: &Value) -> Option<&'static str> {
    link_uses(doc, link).port
}

fn count(part: &Value) -> u64 {
    value(&part["count"])
        .as_f64()
        .map_or(1, |c| c.max(0.0) as u64)
}

/// The part's count times the count of the part it is `per`, along that chain.
fn instances(parts: &Map<String, Value>, name: &str) -> u64 {
    let mut seen = BTreeSet::new();
    let mut n = 1u64;
    let mut at = name;
    while let Some(part) = parts.get(at) {
        if !seen.insert(at) {
            break;
        }
        n = n.saturating_mul(count(part));
        match part["per"].as_str() {
            Some(next) => at = next,
            None => break,
        }
    }
    n
}

/// Derive the contract of a document. Refused, with every reason, when a part's block is not in
/// the table or a port the rules name is not in the ports file. A document whose nested
/// architectures are resolved (`nest::load`) is read flattened, its inner parts as
/// `<part>.<inner part>`.
pub fn derive(doc: &Value, ports: &PortsFile) -> Result<Contract, Vec<String>> {
    let flat = crate::nest::flatten(doc);
    let doc = &flat;
    let empty = Map::new();
    let parts = doc["parts"].as_object().unwrap_or(&empty);
    let links = doc["links"].as_array().cloned().unwrap_or_default();
    let mut refusals = Vec::new();
    let mut used: BTreeSet<(String, String)> = BTreeSet::new();
    for (name, part) in parts {
        match block_ports(doc, name, part) {
            Ok(list) => used.extend(list.into_iter().map(|p| (name.clone(), p.to_owned()))),
            Err(e) => refusals.push(e),
        }
    }
    let mut invariants = Vec::new();
    for link in &links {
        let lu = link_uses(doc, link);
        used.extend(
            lu.uses
                .iter()
                .map(|(p, part)| (part.clone(), (*p).to_owned())),
        );
        let Some(port) = lu.port else { continue };
        let from = text(&link["from"]).to_owned();
        let to = text(&link["to"]).to_owned();
        let every = Use {
            port: port.to_owned(),
            part: from,
        };
        let clause = match lu.within {
            Some(w) => Clause::Inside {
                every,
                within: Use {
                    port: w.to_owned(),
                    part: to,
                },
            },
            None => Clause::To {
                every,
                to,
                over: link["over"].as_str().map(str::to_owned),
            },
        };
        invariants.push(Invariant {
            link: link_name(link),
            port: port.to_owned(),
            within: lu.within.map(str::to_owned),
            predicate: Predicate {
                clauses: vec![clause],
            }
            .to_string(),
        });
    }
    let mut out = Vec::new();
    for (part, port) in &used {
        let Some(p) = ports.port(port) else {
            refusals.push(format!("part {part}: port {port} is not in the ports file"));
            continue;
        };
        let Some(this) = parts.get(part) else {
            refusals.push(format!("part {part}: not a part of the document"));
            continue;
        };
        let settings = narrowing(text(&this["block"]), port)
            .iter()
            .filter_map(|s| {
                let v = value(&this[*s]);
                (!v.is_null()).then(|| ((*s).to_owned(), v.clone()))
            })
            .collect();
        out.push(ContractPort {
            port: port.clone(),
            part: part.clone(),
            protocol: p.protocol.to_string(),
            settings,
            instances: instances(parts, part),
        });
    }
    if !refusals.is_empty() {
        return Err(refusals);
    }
    Ok(Contract {
        architecture: text(&doc["architecture"]).to_owned(),
        ports_digest: ports.sources_sha256.clone(),
        ports: out,
        invariants,
        border: doc["border"]
            .as_array()
            .map(|b| b.iter().map(|x| text(x).to_owned()).collect())
            .unwrap_or_default(),
        ingress: Vec::new(),
    })
}

/// The check of a contract against its document and the ports file: every port is in the ports
/// file with the same protocol, every part, link and border part is in the document, every
/// invariant names a port its link's `from` part uses and its predicate parses, starting with
/// that use. One line per refusal; empty when it passes.
pub fn check(contract: &Contract, doc: &Value, ports: &PortsFile) -> Vec<String> {
    let flat = crate::nest::flatten(doc);
    let doc = &flat;
    let mut out = Vec::new();
    let empty = Map::new();
    let parts = doc["parts"].as_object().unwrap_or(&empty);
    if contract.ports_digest != ports.sources_sha256 {
        out.push(format!(
            "ports_digest {} is not the ports file's {}",
            contract.ports_digest, ports.sources_sha256
        ));
    }
    for used in &contract.ports {
        match ports.port(&used.port) {
            None => out.push(format!(
                "port {} of part {} is not in the ports file",
                used.port, used.part
            )),
            Some(p) if p.protocol.to_string() != used.protocol => out.push(format!(
                "port {} of part {}: protocol `{}` is not the ports file's `{}`",
                used.port, used.part, used.protocol, p.protocol
            )),
            Some(_) => {}
        }
        if !parts.contains_key(&used.part) {
            out.push(format!(
                "port {}: part {} is not in the document",
                used.port, used.part
            ));
        }
    }
    let links: BTreeMap<String, Value> = doc["links"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|l| (link_name(l), l.clone()))
        .collect();
    for inv in &contract.invariants {
        let Some(link) = links.get(&inv.link) else {
            out.push(format!(
                "invariant of link `{}`: no such link in the document",
                inv.link
            ));
            continue;
        };
        let from = text(&link["from"]);
        if !contract
            .ports
            .iter()
            .any(|u| u.port == inv.port && u.part == from)
        {
            out.push(format!(
                "invariant of link `{}`: port {} is not used by its from part {from}",
                inv.link, inv.port
            ));
        }
        match Predicate::parse(&inv.predicate) {
            Err(e) => out.push(format!(
                "invariant of link `{}`: predicate `{}` does not parse: {e}",
                inv.link, inv.predicate
            )),
            Ok(p) => {
                let first = p.clauses[0].every();
                if first.port != inv.port || first.part != from {
                    out.push(format!(
                        "invariant of link `{}`: predicate starts with {first}, not {}@{from}",
                        inv.link, inv.port
                    ));
                }
            }
        }
    }
    for b in &contract.border {
        if !parts.contains_key(b) {
            out.push(format!("border part {b} is not in the document"));
        }
    }
    for i in &contract.ingress {
        let declared = &doc["ingress"][&i.name];
        if !declared.is_object() {
            out.push(format!("ingress {}: not declared in the document", i.name));
            continue;
        }
        if declared["audience"].as_str() != Some(i.audience.as_str()) {
            out.push(format!(
                "ingress {}: audience {} is not the document's",
                i.name, i.audience
            ));
        }
        if declared["authenticated"].as_bool().unwrap_or(true) != i.authenticated {
            out.push(format!(
                "ingress {}: authenticated {} is not the document's",
                i.name, i.authenticated
            ));
        }
        let exports: Vec<&str> = declared["exports"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|e| e["protocol"].as_str())
            .collect();
        if exports != i.exports.iter().map(String::as_str).collect::<Vec<_>>() {
            out.push(format!(
                "ingress {}: exports [{}] are not the document's [{}]",
                i.name,
                i.exports.join(", "),
                exports.join(", ")
            ));
        }
    }
    let declared = doc["ingress"].as_object().map_or(0, |m| m.len());
    if declared != contract.ingress.len() {
        out.push(format!(
            "the contract has {} ingresses, the document declares {declared}",
            contract.ingress.len()
        ));
    }
    out
}

/// A string as YAML: plain when it is a simple word, else double-quoted (JSON's quoting, which
/// YAML reads).
pub fn scalar(s: &str) -> String {
    let plain = s.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
        && !matches!(s, "true" | "false" | "null" | "yes" | "no" | "on" | "off")
        && s.parse::<f64>().is_err();
    if plain {
        s.to_owned()
    } else {
        serde_json::to_string(s).unwrap_or_default()
    }
}

/// A value as flow YAML.
pub fn flow(v: &Value) -> String {
    match v {
        Value::String(s) => scalar(s),
        Value::Array(a) => format!("[{}]", a.iter().map(flow).collect::<Vec<_>>().join(", ")),
        Value::Object(m) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, x)| format!("{}: {}", scalar(k), flow(x)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => other.to_string(),
    }
}

impl Contract {
    /// The contract as data, read back from its text.
    pub fn to_value(&self) -> Value {
        crate::doc::parse_yaml(&self.text()).unwrap_or(Value::Null)
    }

    /// The contract digest: `sha256:` and the SHA-256 of the canonical JSON of the contract.
    pub fn digest(&self) -> String {
        crate::deploy::digest(&self.to_value())
    }

    /// The contract as written: YAML in a fixed order, the same bytes for the same contract.
    pub fn text(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("format: {FORMAT}\n"));
        s.push_str(&format!("architecture: {}\n", scalar(&self.architecture)));
        s.push_str(&format!("ports_digest: {}\n", scalar(&self.ports_digest)));
        s.push_str("ports:\n");
        for p in &self.ports {
            s.push_str(&format!("  - port: {}\n", scalar(&p.port)));
            s.push_str(&format!("    part: {}\n", scalar(&p.part)));
            s.push_str(&format!("    protocol: {}\n", scalar(&p.protocol)));
            s.push_str(&format!("    instances: {}\n", p.instances));
            let settings: Map<String, Value> = p.settings.clone().into_iter().collect();
            s.push_str(&format!(
                "    settings: {}\n",
                flow(&Value::Object(settings))
            ));
        }
        if self.invariants.is_empty() {
            s.push_str("invariants: []\n");
        } else {
            s.push_str("invariants:\n");
        }
        for i in &self.invariants {
            s.push_str(&format!("  - link: {}\n", scalar(&i.link)));
            s.push_str(&format!("    port: {}\n", scalar(&i.port)));
            if let Some(w) = &i.within {
                s.push_str(&format!("    within: {}\n", scalar(w)));
            }
            s.push_str(&format!("    predicate: {}\n", scalar(&i.predicate)));
        }
        let border: Vec<Value> = self
            .border
            .iter()
            .map(|b| Value::String(b.clone()))
            .collect();
        s.push_str(&format!("border: {}\n", flow(&Value::Array(border))));
        if !self.ingress.is_empty() {
            s.push_str("ingress:\n");
        }
        for i in &self.ingress {
            s.push_str(&format!("  - name: {}\n", scalar(&i.name)));
            s.push_str(&format!("    audience: {}\n", scalar(&i.audience)));
            s.push_str(&format!("    authenticated: {}\n", i.authenticated));
            let exports: Vec<Value> = i.exports.iter().map(|e| Value::String(e.clone())).collect();
            s.push_str(&format!("    exports: {}\n", flow(&Value::Array(exports))));
            if i.operations.is_empty() {
                s.push_str("    operations: []\n");
            } else {
                s.push_str("    operations:\n");
            }
            for o in &i.operations {
                s.push_str(&format!(
                    "      - {{kind: {}, name: {}}}\n",
                    o.kind.as_str(),
                    scalar(&o.name)
                ));
            }
        }
        s
    }

    /// Read a contract file back.
    pub fn parse(source: &str) -> Result<Contract, String> {
        let v = crate::doc::parse_yaml(source)?;
        if v["format"] != FORMAT {
            return Err(format!("not {FORMAT}"));
        }
        let field = |x: &Value, k: &str| -> Result<String, String> {
            x[k].as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("no `{k}`"))
        };
        let ports = v["ports"]
            .as_array()
            .ok_or("no `ports`")?
            .iter()
            .map(|p| {
                Ok(ContractPort {
                    port: field(p, "port")?,
                    part: field(p, "part")?,
                    protocol: field(p, "protocol")?,
                    instances: p["instances"].as_u64().ok_or("no `instances`")?,
                    settings: p["settings"]
                        .as_object()
                        .map(|m| m.clone().into_iter().collect())
                        .unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let invariants = v["invariants"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|i| {
                Ok(Invariant {
                    link: field(i, "link")?,
                    port: field(i, "port")?,
                    within: i["within"].as_str().map(str::to_owned),
                    predicate: field(i, "predicate")?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let ingress = v["ingress"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|i| {
                let name = field(i, "name")?;
                let operations = i["operations"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|o| {
                        let kind = field(o, "kind")?;
                        Ok(Operation {
                            kind: OpKind::parse(&kind).ok_or_else(|| {
                                format!("ingress {name}: unknown operation kind `{kind}`")
                            })?,
                            name: field(o, "name")?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Ok(ContractIngress {
                    audience: field(i, "audience")?,
                    authenticated: i["authenticated"]
                        .as_bool()
                        .ok_or_else(|| format!("ingress {name}: no `authenticated`"))?,
                    exports: i["exports"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|e| text(e).to_owned())
                        .collect(),
                    operations,
                    name,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Contract {
            architecture: field(&v, "architecture")?,
            ports_digest: field(&v, "ports_digest")?,
            ports,
            invariants,
            border: v["border"]
                .as_array()
                .map(|b| b.iter().map(|x| text(x).to_owned()).collect())
                .unwrap_or_default(),
            ingress,
        })
    }
}
