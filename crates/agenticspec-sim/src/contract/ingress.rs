//! The ingresses of an architecture document (its `ingress:` key): for whom the architecture is
//! reachable from outside (an audience, a principal part), what that audience may use (an
//! interface: selection rules over the blocks and the KPI catalogue) and over which protocols
//! (exports). The check of the key, and the resolution of each interface into operations
//! (`{kind, name}`) for the contract. `docs/contract.md` states the rules and the format.
//!
//! An interface only selects: it names no field, no type and no protocol, and an export names no
//! operation.

use super::derive::{Contract, link_name, link_port};
use super::ports::{Kind, PortsFile, Role};
use crate::check::value;
use crate::doc::num;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// The actor whose commands answer a principal port.
const PRINCIPAL: &str = super::ports::PRINCIPAL;

/// The kind of an operation of an interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpKind {
    Command,
    View,
    Feed,
    Metric,
}

impl OpKind {
    pub const ALL: [OpKind; 4] = [OpKind::Command, OpKind::View, OpKind::Feed, OpKind::Metric];

    pub fn as_str(self) -> &'static str {
        match self {
            OpKind::Command => "command",
            OpKind::View => "view",
            OpKind::Feed => "feed",
            OpKind::Metric => "metric",
        }
    }

    pub fn parse(text: &str) -> Option<OpKind> {
        OpKind::ALL.into_iter().find(|k| k.as_str() == text)
    }
}

/// Each protocol an export may name, with the operation kinds it carries.
pub const PROTOCOLS: &[(&str, &[OpKind])] = &[
    ("http", &[OpKind::Command, OpKind::View]),
    ("event_stream", &[OpKind::Feed]),
    ("cli", &[OpKind::Command, OpKind::View]),
    ("tool", &[OpKind::Command, OpKind::View]),
    ("agent", &[OpKind::Command, OpKind::Feed]),
    ("metrics_scrape", &[OpKind::Metric]),
    (
        "ui",
        &[OpKind::Command, OpKind::View, OpKind::Feed, OpKind::Metric],
    ),
];

/// The operation kinds a protocol carries; `None` for a protocol outside [`PROTOCOLS`].
pub fn carries(protocol: &str) -> Option<&'static [OpKind]> {
    PROTOCOLS
        .iter()
        .find(|(p, _)| *p == protocol)
        .map(|(_, k)| *k)
}

/// One operation of a resolved interface.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Operation {
    pub kind: OpKind,
    /// The full name of the command, view or event; a KPI's id for a metric.
    pub name: String,
}

/// One ingress of a contract: its interface resolved into operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractIngress {
    pub name: String,
    /// The principal part the ingress serves.
    pub audience: String,
    pub authenticated: bool,
    /// The protocols of its exports, in the order written.
    pub exports: Vec<String>,
    /// Ordered by kind (command, view, feed, metric), then name.
    pub operations: Vec<Operation>,
}

/// Every rule of the check of `ingress:`, in the order it applies them.
pub const RULES: &[&str] = &[
    "ingress_malformed",
    "ingress_reserved_part",
    "ingress_none_not_default",
    "ingress_reserved_audience",
    "ingress_audience_not_principal",
    "ingress_without_export",
    "ingress_unknown_protocol",
    "ingress_unknown_part",
    "ingress_unknown_operation",
    "ingress_actor_is_implementation",
    "ingress_outside_scope",
    "ingress_command_off_border",
    "ingress_operation_not_carried",
    "ingress_ui_without_wire",
    "ingress_unauthenticated_command",
    "ingress_agent_without_channel",
    "principal_port_without_ingress",
];

/// One refusal of the check of `ingress:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// One of [`RULES`].
    pub rule: &'static str,
    /// `ingress <name>`, or `part <name>` for a principal port no ingress carries.
    pub subject: String,
    pub detail: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.rule, self.subject, self.detail)
    }
}

/// One KPI of the catalogue, as an interface reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kpi {
    pub block: String,
    pub status: String,
    /// Its `unit`, as a metric list gives it.
    pub unit: String,
    /// Its `summary`, the help of its metric.
    pub summary: String,
}

/// The KPI catalogue (`kpis/catalogue.yaml`, `agenticspec-kpis/1`) by id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Kpis(pub BTreeMap<String, Kpi>);

impl Kpis {
    pub fn parse(doc: &Value) -> Result<Kpis, String> {
        if doc["format"] != "agenticspec-kpis/1" {
            return Err("not agenticspec-kpis/1".into());
        }
        let mut out = BTreeMap::new();
        for k in doc["kpis"].as_array().ok_or("no `kpis`")? {
            let id = k["id"].as_str().ok_or("a KPI without `id`")?;
            out.insert(
                id.to_owned(),
                Kpi {
                    block: k["block"].as_str().unwrap_or_default().to_owned(),
                    status: k["status"].as_str().unwrap_or_default().to_owned(),
                    unit: k["unit"].as_str().unwrap_or_default().to_owned(),
                    summary: k["summary"].as_str().unwrap_or_default().to_owned(),
                },
            );
        }
        Ok(Kpis(out))
    }

    pub fn load(path: &Path) -> Result<Kpis, String> {
        let doc = crate::doc::load_yaml(path)?;
        Kpis::parse(&doc).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// Whether the document declares an `ingress:` key.
pub fn declares(doc: &Value) -> bool {
    !doc["ingress"].is_null()
}

/// The reserved part path of the implied audiences of the default control plane. No part of a
/// document may take it.
pub const CONTROL: &str = "control";

/// The implied audiences: principal parts of the contract only, never of the document, each with
/// its principal kind.
pub const IMPLIED: &[(&str, &str)] = &[
    ("control.operator", "human"),
    ("control.monitor", "service"),
];

/// The default ingresses, by name, as a document writes them (the entries of `ingress:`): every
/// architecture has each one unless it declares an ingress of that name or sets it to `none`.
/// [`default_text`] prints them; [`default_ingress`] reads them.
pub const DEFAULTS: &[(&str, &str)] = &[
    (
        "operator",
        "  operator:
    audience: control.operator
    interface:
      scope: all
      commands: { principal: all, plays: administration }
      views: true
      feeds: true
    exports: [{ protocol: http }, { protocol: event_stream }, { protocol: ui }, { protocol: tool }]
",
    ),
    (
        "metrics",
        "  metrics:
    audience: control.monitor
    authenticated: false
    interface: { scope: all, metrics: computed }
    exports: [{ protocol: metrics_scrape }]
",
    ),
];

/// The default ingress of that name, as the document would declare it.
pub fn default_ingress(name: &str) -> Option<Value> {
    let (_, text) = DEFAULTS.iter().find(|(n, _)| *n == name)?;
    let doc = crate::doc::parse_yaml(text).ok()?;
    doc.get(name).cloned()
}

/// The names of the defaults in effect: those the document neither declares nor sets to `none`.
pub fn defaults_in_effect(doc: &Value) -> Vec<&'static str> {
    let ingress = &doc["ingress"];
    if ingress.as_str() == Some("none") {
        return Vec::new();
    }
    DEFAULTS
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| ingress.get(*n).is_none())
        .collect()
}

/// The `ingress:` section of the defaults in effect, to paste into the document and change:
/// pasted, it gives the same contract. A comment line when no default is in effect.
pub fn default_text(doc: &Value) -> String {
    let names = defaults_in_effect(doc);
    if names.is_empty() {
        return "# no default ingress is in effect: the document declares or opts out of each\n"
            .into();
    }
    let mut s = String::from("ingress:\n");
    for (n, text) in DEFAULTS {
        if names.contains(n) {
            s.push_str(text);
        }
    }
    s
}

/// The ingresses in effect: the declared ones, and each default the document neither declares
/// nor sets to `none` (`ingress: none` removes both), by name; with the refusals of a key
/// `ingress:` that has no such shape.
pub fn effective(doc: &Value) -> (Map<String, Value>, Vec<Refusal>) {
    let mut out = Map::new();
    let mut refusals = Vec::new();
    let ingress = &doc["ingress"];
    if ingress.as_str() == Some("none") {
        return (out, refusals);
    }
    for (n, _) in DEFAULTS {
        if let Some(v) = default_ingress(n) {
            out.insert((*n).to_owned(), v);
        }
    }
    match ingress {
        Value::Null => {}
        Value::Object(m) => {
            for (name, v) in m {
                if v.as_str() == Some("none") {
                    if DEFAULTS.iter().any(|(n, _)| n == name) {
                        out.remove(name);
                    } else {
                        refusals.push(Refusal {
                            rule: "ingress_none_not_default",
                            subject: format!("ingress `{name}`"),
                            detail: format!(
                                "`none` opts out of a default ingress ({}), and `{name}` is none",
                                DEFAULTS
                                    .iter()
                                    .map(|(n, _)| *n)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        });
                    }
                } else {
                    out.insert(name.clone(), v.clone());
                }
            }
        }
        _ => refusals.push(Refusal {
            rule: "ingress_malformed",
            subject: "ingress".into(),
            detail: "`ingress` maps a name to each ingress, or is none".into(),
        }),
    }
    (out, refusals)
}

/// The refusals of the reserved names that need neither the ports file nor the KPI catalogue:
/// a part on the reserved path `control`, `none` for an ingress that is no default, and a key
/// `ingress:` of no known shape.
pub fn reserved(doc: &Value) -> Vec<Refusal> {
    let mut out: Vec<Refusal> = doc["parts"]
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(name, _)| *name == CONTROL || name.starts_with(&format!("{CONTROL}.")))
        .map(|(name, _)| Refusal {
            rule: "ingress_reserved_part",
            subject: format!("part `{name}`"),
            detail: format!(
                "the path `{CONTROL}` is reserved for the implied audiences ({})",
                IMPLIED
                    .iter()
                    .map(|(p, _)| *p)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        })
        .collect();
    out.extend(effective(doc).1);
    out
}

/// The domain of the blocks a part's block belongs to (`pool` is the resource block's).
pub fn block_domain(block: &str) -> String {
    let b = match block {
        "pool" => "resource",
        "schedule" => "trigger",
        other => other,
    };
    format!("agentic.{b}")
}

/// The domains of a part: its own block's, and the domains of the ports it uses in the contract.
pub fn part_domains(
    doc: &Value,
    contract: &Contract,
    ports: &PortsFile,
    part: &str,
) -> Vec<String> {
    let mut out = BTreeSet::new();
    if let Some(b) = doc["parts"][part]["block"].as_str() {
        out.insert(block_domain(b));
    }
    for p in contract.ports.iter().filter(|p| p.part == part) {
        if let Some(port) = ports.port(&p.port) {
            out.insert(format!("agentic.{}", port.block));
        }
    }
    out.into_iter().collect()
}

/// The commands of a port's intent, qualified: the first commands of its protocol, in the
/// domain of the port's block.
fn intents(ports: &PortsFile, port: &str) -> Vec<String> {
    let Some(p) = ports.port(port) else {
        return Vec::new();
    };
    let domain = format!("agentic.{}", p.block);
    p.protocol
        .first()
        .into_iter()
        .map(|short| {
            let own = format!("{domain}.{short}");
            if ports.surface.role(&own).is_some() || ports.surface.domains.is_empty() {
                return own;
            }
            ports
                .surface
                .domains
                .values()
                .flat_map(|d| d.commands.keys())
                .find(|c| c.rsplit('.').next() == Some(short.as_str()))
                .cloned()
                .unwrap_or(own)
        })
        .collect()
}

/// The commands that answer a principal port: its reports the principal actor may issue.
fn answers(ports: &PortsFile, port: &str) -> Vec<String> {
    let may: BTreeSet<&String> = ports
        .surface
        .actors
        .get(PRINCIPAL)
        .into_iter()
        .flatten()
        .collect();
    ports
        .port(port)
        .filter(|p| p.kind == Kind::Principal)
        .map(|p| {
            p.reports
                .iter()
                .filter(|r| may.contains(r))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn strings(v: &Value) -> Option<Vec<String>> {
    match v {
        Value::Null => Some(Vec::new()),
        Value::Array(a) => a.iter().map(|x| x.as_str().map(str::to_owned)).collect(),
        _ => None,
    }
}

/// `true`, `false`, absent, or `{except: [...]}`: whether everything is selected, and the
/// names left out.
fn selection(v: &Value) -> Option<(bool, Vec<String>)> {
    match v {
        Value::Null | Value::Bool(false) => Some((false, Vec::new())),
        Value::Bool(true) => Some((true, Vec::new())),
        Value::Object(m) if m.keys().all(|k| k == "except") => {
            Some((true, strings(m.get("except").unwrap_or(&Value::Null))?))
        }
        _ => None,
    }
}

/// The `commands:` rules: `principal`, a list of `principal` and `{plays, except}` mappings, or
/// one mapping with `principal`, `plays` and `except`. `principal: all` answers the principal
/// ports of every principal part in scope; `plays: administration` plays every actor of the
/// scope an audience may play.
#[derive(Default)]
struct CommandRules {
    principal: bool,
    principal_all: bool,
    plays: Vec<String>,
    plays_administration: bool,
    except: Vec<String>,
}

fn command_rules(v: &Value) -> Option<CommandRules> {
    let mut out = CommandRules::default();
    let mut add = |item: &Value| -> Option<()> {
        match item {
            Value::String(s) if s == "principal" => out.principal = true,
            Value::Object(m) => {
                for (k, x) in m {
                    match (k.as_str(), x) {
                        ("principal", Value::String(s)) if s == "all" => out.principal_all = true,
                        ("principal", x) => out.principal |= x.as_bool()?,
                        ("plays", Value::String(s)) if s == "administration" => {
                            out.plays_administration = true
                        }
                        ("plays", x) => out.plays.extend(strings(x)?),
                        ("except", x) => out.except.extend(strings(x)?),
                        _ => return None,
                    }
                }
            }
            _ => return None,
        }
        Some(())
    };
    match v {
        Value::Null => {}
        Value::Array(items) => {
            for i in items {
                add(i)?;
            }
        }
        other => add(other)?,
    }
    Some(out)
}

const INGRESS_KEYS: &[&str] = &[
    "audience",
    "authenticated",
    "interface",
    "exports",
    "rate_limit_per_minute",
    "summary",
];
const INTERFACE_KEYS: &[&str] = &["scope", "commands", "views", "feeds", "metrics"];

struct Ctx<'a> {
    doc: &'a Value,
    contract: &'a Contract,
    ports: &'a PortsFile,
    kpis: &'a Kpis,
    parts: &'a Map<String, Value>,
    border: BTreeSet<String>,
}

impl Ctx<'_> {
    /// A principal part of the document, or an implied audience.
    fn is_principal(&self, part: &str) -> bool {
        self.is_document_principal(part) || IMPLIED.iter().any(|(p, _)| *p == part)
    }

    fn is_document_principal(&self, part: &str) -> bool {
        self.parts
            .get(part)
            .is_some_and(|p| p["block"] == "principal")
    }

    /// Whether an actor's `may:` holds the report of a port that is no principal port: the
    /// implementation's to issue, so no audience plays it.
    fn implementation<'a>(&self, may: &'a [String]) -> Vec<&'a String> {
        may.iter()
            .filter(|c| {
                self.ports
                    .ports
                    .iter()
                    .any(|p| p.kind != Kind::Principal && p.reports.contains(c))
            })
            .collect()
    }

    /// The links from `audience` whose value travels in a port of it, with the intent commands
    /// of that port: the inputs the audience sends.
    fn inputs(&self, audience: &str) -> Vec<(&Value, Vec<String>)> {
        self.doc["links"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|l| l["from"].as_str() == Some(audience))
            .filter_map(|l| link_port(self.doc, l).map(|p| (l, intents(self.ports, p))))
            .collect()
    }

    fn known_command(&self, name: &str) -> bool {
        self.ports.surface.role(name).is_some()
    }
}

/// Resolve every ingress in effect (the declared ones and the defaults, [`effective`]) into its
/// operations, with every refusal of the check of `ingress:`. The contract is the document's
/// (for the ports each part uses).
pub fn resolve(
    doc: &Value,
    contract: &Contract,
    ports: &PortsFile,
    kpis: &Kpis,
) -> (Vec<ContractIngress>, Vec<Refusal>) {
    let mut out = Vec::new();
    let mut refusals = reserved(doc);
    let (ingresses, _) = effective(doc);
    let empty = Map::new();
    let ctx = Ctx {
        doc,
        contract,
        ports,
        kpis,
        parts: doc["parts"].as_object().unwrap_or(&empty),
        border: doc["border"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|b| b.as_str().map(str::to_owned))
            .collect(),
    };
    let mut answering: Vec<BTreeSet<String>> = Vec::new();
    for (name, ingress) in &ingresses {
        let mut refuse = |rule: &'static str, detail: String| {
            refusals.push(Refusal {
                rule,
                subject: format!("ingress `{name}`"),
                detail,
            })
        };
        if let Some((r, answers_for)) = one(&ctx, name, ingress, &mut refuse) {
            out.push(r);
            answering.push(answers_for);
        }
    }
    // Every principal port of the contract is answered through an ingress that answers for its
    // principal: one of its audience, or one with `principal: all` over a scope that holds it.
    for cp in &contract.ports {
        if !ctx.is_principal(&cp.part) {
            continue;
        }
        let answer = answers(ports, &cp.port);
        if answer.is_empty() {
            continue;
        }
        let served = out.iter().zip(&answering).any(|(i, answers_for)| {
            answers_for.contains(&cp.part)
                && i.exports
                    .iter()
                    .any(|e| carries(e).is_some_and(|k| k.contains(&OpKind::Command)))
                && i.operations
                    .iter()
                    .any(|o| o.kind == OpKind::Command && answer.contains(&o.name))
        });
        if !served {
            refusals.push(Refusal {
                rule: "principal_port_without_ingress",
                subject: format!("part `{}`", cp.part),
                detail: format!(
                    "no ingress that answers for `{}` carries an answer of its principal port {} ({})",
                    cp.part,
                    cp.port,
                    answer.join(", ")
                ),
            });
        }
    }
    refusals.sort_by_key(|r| RULES.iter().position(|x| *x == r.rule));
    (out, refusals)
}

/// The refusals of the check of `ingress:` alone.
pub fn check(doc: &Value, contract: &Contract, ports: &PortsFile, kpis: &Kpis) -> Vec<Refusal> {
    resolve(doc, contract, ports, kpis).1
}

/// One ingress resolved, with the principal parts whose principal ports it answers for.
fn one(
    ctx: &Ctx,
    name: &str,
    ingress: &Value,
    refuse: &mut dyn FnMut(&'static str, String),
) -> Option<(ContractIngress, BTreeSet<String>)> {
    let Some(m) = ingress.as_object() else {
        refuse("ingress_malformed", "an ingress is a mapping".into());
        return None;
    };
    for k in m.keys() {
        if !INGRESS_KEYS.contains(&k.as_str()) {
            refuse(
                "ingress_malformed",
                format!(
                    "unknown key `{k}`; an ingress has {}",
                    INGRESS_KEYS.join(", ")
                ),
            );
        }
    }
    let interface = &ingress["interface"];
    let Some(im) = interface.as_object() else {
        refuse(
            "ingress_malformed",
            "an ingress has an `interface` mapping".into(),
        );
        return None;
    };
    for k in im.keys() {
        if !INTERFACE_KEYS.contains(&k.as_str()) {
            refuse(
                "ingress_malformed",
                format!(
                    "unknown interface key `{k}`; an interface has {}",
                    INTERFACE_KEYS.join(", ")
                ),
            );
        }
    }
    let authenticated = match &ingress["authenticated"] {
        Value::Null => true,
        Value::Bool(b) => *b,
        _ => {
            refuse(
                "ingress_malformed",
                "`authenticated` is true or false".into(),
            );
            true
        }
    };

    // The audience.
    let audience = ingress["audience"].as_str().unwrap_or_default().to_owned();
    let audience_ok = ctx.is_principal(&audience);
    if !audience_ok && audience.starts_with(&format!("{CONTROL}.")) {
        refuse(
            "ingress_reserved_audience",
            format!(
                "audience `{audience}` is on the reserved path `{CONTROL}`, which holds only {}",
                IMPLIED
                    .iter()
                    .map(|(p, _)| *p)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    } else if !audience_ok {
        refuse(
            "ingress_audience_not_principal",
            format!("audience `{audience}` names no principal part"),
        );
    }

    // The exports.
    let mut exports = Vec::new();
    match &ingress["exports"] {
        Value::Null => {}
        Value::Array(list) => {
            for e in list {
                match e["protocol"].as_str() {
                    Some(p) if exports.iter().any(|x| x == p) => refuse(
                        "ingress_malformed",
                        format!("protocol `{p}` exported twice"),
                    ),
                    Some(p) => {
                        if carries(p).is_none() {
                            refuse(
                                "ingress_unknown_protocol",
                                format!(
                                    "protocol `{p}`; an export names one of {}",
                                    PROTOCOLS
                                        .iter()
                                        .map(|(p, _)| *p)
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ),
                            );
                        }
                        exports.push(p.to_owned());
                    }
                    None => refuse(
                        "ingress_malformed",
                        "each export names its `protocol`".into(),
                    ),
                }
            }
        }
        _ => refuse("ingress_malformed", "`exports` is a list".into()),
    }
    if exports.is_empty() {
        refuse("ingress_without_export", "the ingress has no export".into());
    }

    // The scope: its parts, and their domains.
    let scope_parts: Vec<String> = match &interface["scope"] {
        Value::Null => linked(ctx, &audience),
        Value::String(s) if s == "linked" => linked(ctx, &audience),
        Value::String(s) if s == "all" => ctx.parts.keys().cloned().collect(),
        Value::Array(list) => {
            let mut out = Vec::new();
            for p in list {
                match p.as_str() {
                    Some(p) if ctx.parts.contains_key(p) => out.push(p.to_owned()),
                    Some(p) => refuse(
                        "ingress_unknown_part",
                        format!("scope names `{p}`, which is no part"),
                    ),
                    None => refuse("ingress_malformed", "a scope list holds part names".into()),
                }
            }
            out
        }
        _ => {
            refuse(
                "ingress_malformed",
                "`scope` is linked, all or a list of parts".into(),
            );
            Vec::new()
        }
    };
    let domains: BTreeSet<String> = scope_parts
        .iter()
        .flat_map(|p| part_domains(ctx.doc, ctx.contract, ctx.ports, p))
        .collect();
    let surface = &ctx.ports.surface;

    let mut ops: BTreeSet<(OpKind, String)> = BTreeSet::new();

    // Commands.
    let Some(rules) = command_rules(&interface["commands"]) else {
        refuse(
            "ingress_malformed",
            "`commands` is principal, or a list or mapping of principal, plays and except".into(),
        );
        return None;
    };
    let inputs = if audience_ok {
        ctx.inputs(&audience)
    } else {
        Vec::new()
    };
    let mut answers_for = BTreeSet::new();
    if rules.principal_all {
        for part in scope_parts.iter().filter(|p| ctx.is_document_principal(p)) {
            for cp in ctx.contract.ports.iter().filter(|p| &p.part == part) {
                for a in answers(ctx.ports, &cp.port) {
                    ops.insert((OpKind::Command, a));
                }
            }
            answers_for.insert(part.clone());
        }
    }
    if rules.principal && audience_ok {
        answers_for.insert(audience.clone());
        for cp in ctx.contract.ports.iter().filter(|p| p.part == audience) {
            for a in answers(ctx.ports, &cp.port) {
                ops.insert((OpKind::Command, a));
            }
        }
        for (link, commands) in &inputs {
            let to = link["to"].as_str().unwrap_or_default();
            if ctx.border.contains(to) {
                ops.extend(commands.iter().map(|c| (OpKind::Command, c.clone())));
            } else {
                refuse(
                    "ingress_command_off_border",
                    format!(
                        "the input `{}` ({}) reaches `{to}`, which is not on the border",
                        link_name(link),
                        commands.join(", ")
                    ),
                );
            }
        }
    }
    for actor in &rules.plays {
        let Some(may) = surface.actors.get(actor) else {
            refuse(
                "ingress_unknown_operation",
                format!("plays `{actor}`, which is no actor of the blocks"),
            );
            continue;
        };
        let implementation = ctx.implementation(may);
        if !implementation.is_empty() {
            refuse(
                "ingress_actor_is_implementation",
                format!(
                    "plays `{actor}`, whose may: holds the report of a port ({})",
                    implementation
                        .iter()
                        .map(|c| c.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
            continue;
        }
        let domain = actor.rsplit_once('.').map_or("", |x| x.0);
        if !domains.contains(domain) {
            refuse(
                "ingress_outside_scope",
                format!("plays `{actor}`, whose domain {domain} is no domain of the scope"),
            );
            continue;
        }
        for c in may {
            if surface.role(c) == Some(Role::Administration) {
                ops.insert((OpKind::Command, c.clone()));
            }
        }
    }
    if rules.plays_administration {
        // Every actor of a domain in scope that an audience may play (I4).
        for (actor, may) in &surface.actors {
            let domain = actor.rsplit_once('.').map_or("", |x| x.0);
            if !domains.contains(domain) || !ctx.implementation(may).is_empty() {
                continue;
            }
            for c in may {
                if surface.role(c) == Some(Role::Administration) {
                    ops.insert((OpKind::Command, c.clone()));
                }
            }
        }
    }
    for c in &rules.except {
        if !ctx.known_command(c) {
            refuse(
                "ingress_unknown_operation",
                format!("except names `{c}`, which is no command of the blocks"),
            );
        }
        ops.remove(&(OpKind::Command, c.clone()));
    }

    // Views and feeds.
    for (key, kind) in [("views", OpKind::View), ("feeds", OpKind::Feed)] {
        let Some((all, except)) = selection(&interface[key]) else {
            refuse(
                "ingress_malformed",
                format!("`{key}` is true, false or {{except: [...]}}"),
            );
            continue;
        };
        for x in &except {
            let known = match kind {
                OpKind::View => surface.has_view(x),
                _ => surface.has_event(x),
            };
            if !known {
                refuse(
                    "ingress_unknown_operation",
                    format!(
                        "except names `{x}`, which is no {} of the blocks",
                        if kind == OpKind::View {
                            "view"
                        } else {
                            "event"
                        }
                    ),
                );
            }
        }
        if !all {
            continue;
        }
        for d in &domains {
            let Some(ds) = surface.domains.get(d) else {
                continue;
            };
            let names = if kind == OpKind::View {
                &ds.views
            } else {
                &ds.events
            };
            for n in names {
                if !except.contains(n) {
                    ops.insert((kind, n.clone()));
                }
            }
        }
    }

    // Metrics: `computed`, every KPI of a block in scope that a runtime computes from observed
    // events, or a list of KPI ids.
    if interface["metrics"].as_str() == Some("computed") {
        for (id, k) in &ctx.kpis.0 {
            if matches!(k.status.as_str(), "computed" | "both")
                && domains.contains(&block_domain(&k.block))
            {
                ops.insert((OpKind::Metric, id.clone()));
            }
        }
    } else {
        metric_list(ctx, &interface["metrics"], &domains, &mut ops, refuse);
    }

    // What the exports carry.
    let carried: BTreeSet<OpKind> = exports
        .iter()
        .filter_map(|e| carries(e))
        .flatten()
        .copied()
        .collect();
    if !exports.is_empty() {
        let kinds: BTreeSet<OpKind> = ops.iter().map(|(k, _)| *k).collect();
        for k in kinds.difference(&carried) {
            refuse(
                "ingress_operation_not_carried",
                format!(
                    "no export carries its {} operations (exports: {})",
                    k.as_str(),
                    exports.join(", ")
                ),
            );
        }
    }
    finish(
        ctx,
        name,
        ingress,
        audience,
        authenticated,
        exports,
        ops,
        &inputs,
        refuse,
    )
    .map(|i| (i, answers_for))
}

/// The KPIs of a `metrics:` list, each of a block in scope.
fn metric_list(
    ctx: &Ctx,
    list: &Value,
    domains: &BTreeSet<String>,
    ops: &mut BTreeSet<(OpKind, String)>,
    refuse: &mut dyn FnMut(&'static str, String),
) {
    match strings(list) {
        None => refuse(
            "ingress_malformed",
            "`metrics` is computed or a list of KPI ids".into(),
        ),
        Some(ids) => {
            for id in ids {
                match ctx.kpis.0.get(&id) {
                    None => refuse(
                        "ingress_unknown_operation",
                        format!("metrics names `{id}`, which is no KPI of the catalogue"),
                    ),
                    Some(k) if !domains.contains(&block_domain(&k.block)) => refuse(
                        "ingress_outside_scope",
                        format!(
                            "metric `{id}` is of block {}, which no part of the scope is",
                            k.block
                        ),
                    ),
                    Some(_) => {
                        ops.insert((OpKind::Metric, id));
                    }
                }
            }
        }
    }
}

/// The checks of an ingress over its resolved operations and exports, and the ingress.
#[allow(clippy::too_many_arguments)]
fn finish(
    ctx: &Ctx,
    name: &str,
    ingress: &Value,
    audience: String,
    authenticated: bool,
    exports: Vec<String>,
    ops: BTreeSet<(OpKind, String)>,
    inputs: &[(&Value, Vec<String>)],
    refuse: &mut dyn FnMut(&'static str, String),
) -> Option<ContractIngress> {
    let has = |p: &str| exports.iter().any(|e| e == p);
    if has("ui") && !(has("http") && has("event_stream")) {
        refuse(
            "ingress_ui_without_wire",
            "a ui export needs http and event_stream exports beside it".into(),
        );
    }
    let commands: Vec<&String> = ops
        .iter()
        .filter(|(k, _)| *k == OpKind::Command)
        .map(|(_, n)| n)
        .collect();
    if !authenticated && !commands.is_empty() {
        refuse(
            "ingress_unauthenticated_command",
            format!(
                "authenticated: false beside commands ({})",
                commands
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    if has("agent") {
        let work: Vec<&Value> = inputs
            .iter()
            .filter(|(l, _)| l["to"].as_str().is_some_and(|t| ctx.border.contains(t)))
            .map(|(l, _)| *l)
            .collect();
        let bare: Vec<String> = work
            .iter()
            .filter(|l| {
                !l["over"]
                    .as_str()
                    .is_some_and(|c| ctx.parts.get(c).is_some_and(|p| p["block"] == "channel"))
            })
            .map(|l| link_name(l))
            .collect();
        if work.is_empty() || !bare.is_empty() {
            refuse(
                "ingress_agent_without_channel",
                if work.is_empty() {
                    "an agent export needs a work input over a channel part, and the audience sends none".into()
                } else {
                    format!(
                        "work inputs travel over no channel part: {}",
                        bare.join("; ")
                    )
                },
            );
        }
    }
    // The rate limit is a place of the document and may be marked; it resolves to no operation.
    let limit = value(&ingress["rate_limit_per_minute"]);
    if !limit.is_null() && !num(limit).is_some_and(|x| x > 0.0) {
        refuse(
            "ingress_malformed",
            "`rate_limit_per_minute` is a positive number".into(),
        );
    }

    Some(ContractIngress {
        name: name.to_owned(),
        audience,
        authenticated,
        exports,
        operations: ops
            .into_iter()
            .map(|(kind, name)| Operation { kind, name })
            .collect(),
    })
}

/// The parts the audience links to: the `to` and `over` of its links.
fn linked(ctx: &Ctx, audience: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for l in ctx.doc["links"].as_array().into_iter().flatten() {
        if l["from"].as_str() == Some(audience) {
            for k in ["to", "over"] {
                if let Some(p) = l[k].as_str().filter(|p| ctx.parts.contains_key(*p)) {
                    out.insert(p.to_owned());
                }
            }
        }
    }
    out.into_iter().collect()
}
