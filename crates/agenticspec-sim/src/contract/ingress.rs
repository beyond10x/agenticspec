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
/// one mapping with `principal`, `plays` and `except`.
#[derive(Default)]
struct CommandRules {
    principal: bool,
    plays: Vec<String>,
    except: Vec<String>,
}

fn command_rules(v: &Value) -> Option<CommandRules> {
    let mut out = CommandRules::default();
    let mut add = |item: &Value| -> Option<()> {
        match item {
            Value::String(s) if s == "principal" => out.principal = true,
            Value::Object(m) => {
                for (k, x) in m {
                    match k.as_str() {
                        "principal" => out.principal |= x.as_bool()?,
                        "plays" => out.plays.extend(strings(x)?),
                        "except" => out.except.extend(strings(x)?),
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
    fn is_principal(&self, part: &str) -> bool {
        self.parts
            .get(part)
            .is_some_and(|p| p["block"] == "principal")
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

/// Resolve every ingress of a document into its operations, with every refusal of the check of
/// `ingress:`. The contract is the document's (for the ports each part uses); a document
/// without `ingress:` resolves to none and is never refused here.
pub fn resolve(
    doc: &Value,
    contract: &Contract,
    ports: &PortsFile,
    kpis: &Kpis,
) -> (Vec<ContractIngress>, Vec<Refusal>) {
    let mut out = Vec::new();
    let mut refusals = Vec::new();
    if !declares(doc) {
        return (out, refusals);
    }
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
    let Some(ingresses) = doc["ingress"].as_object() else {
        refusals.push(Refusal {
            rule: "ingress_malformed",
            subject: "ingress".into(),
            detail: "`ingress` maps a name to each ingress".into(),
        });
        return (out, refusals);
    };
    for (name, ingress) in ingresses {
        let mut refuse = |rule: &'static str, detail: String| {
            refusals.push(Refusal {
                rule,
                subject: format!("ingress `{name}`"),
                detail,
            })
        };
        if let Some(r) = one(&ctx, name, ingress, &mut refuse) {
            out.push(r);
        }
    }
    // Every principal port of the contract is answered through an ingress of its principal.
    for cp in &contract.ports {
        if !ctx.is_principal(&cp.part) {
            continue;
        }
        let answer = answers(ports, &cp.port);
        if answer.is_empty() {
            continue;
        }
        let served = out.iter().any(|i| {
            i.audience == cp.part
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
                    "no ingress of `{}` carries an answer of its principal port {} ({})",
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

fn one(
    ctx: &Ctx,
    name: &str,
    ingress: &Value,
    refuse: &mut dyn FnMut(&'static str, String),
) -> Option<ContractIngress> {
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
    if !audience_ok {
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
    if rules.principal && audience_ok {
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
        let implementation: Vec<&String> = may
            .iter()
            .filter(|c| {
                ctx.ports
                    .ports
                    .iter()
                    .any(|p| p.kind != Kind::Principal && p.reports.contains(c))
            })
            .collect();
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

    // Metrics.
    match strings(&interface["metrics"]) {
        None => refuse("ingress_malformed", "`metrics` is a list of KPI ids".into()),
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
