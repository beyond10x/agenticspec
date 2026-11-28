//! The deployment bundle (`agenticspec-deployment/1`): one YAML file with every member inline,
//! the input a translator turns into an architecture of its target. [`build`] admits the
//! architecture, its contract, the binding and the packing, and only then builds the bundle;
//! the same inputs give the same bytes. The bundle names no secret value, host, provider or
//! target.

use super::admit::{Accepts, Finding, Plan, PlannedUnit, admit, sort};
use super::binding::{self as binding_form, Binding};
use super::grants;
use super::packing::{self, Packing, is_clock};
use super::{digest, sha256};
use crate::check::value;
use crate::contract::derive::{Contract, link_name};
use crate::doc::parse_yaml;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const FORMAT: &str = "agenticspec-deployment/1";

/// The members of a bundle, in the order they are written.
pub const MEMBERS: &[&str] = &[
    "format",
    "generator",
    "architecture",
    "nested",
    "contract",
    "binding",
    "packing",
    "units",
    "ingress",
    "requirements",
    "agents",
    "runtimes",
    "digest",
];

/// The claims of a unit the binding names none for.
pub const DEFAULT_CPU_MILLIS: u64 = 500;
pub const DEFAULT_MEMORY_MIB: u64 = 512;

/// What a bundle is built from.
pub struct Inputs<'a> {
    /// The architecture document as written, and as data.
    pub arch_bytes: &'a [u8],
    pub arch: &'a Value,
    pub contract: &'a Contract,
    pub binding: &'a Binding,
    pub packing: &'a Packing,
    pub accepts: &'a Accepts,
}

/// A built bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    pub value: Value,
    pub text: String,
    pub digest: String,
    pub units: usize,
}

fn text(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}

/// The revision digest of an architecture: SHA-256 of the canonical JSON of its parts, links
/// and border (`agentic.architecture.Digest`).
pub fn revision_digest(arch: &Value) -> String {
    let or = |v: &Value, d: Value| if v.is_null() { d } else { v.clone() };
    digest(&json!({
        "parts": or(&arch["parts"], json!({})),
        "links": or(&arch["links"], json!([])),
        "border": or(&arch["border"], json!([])),
    }))
}

/// An agent document the binding names, read and digested.
struct AgentDoc {
    digest: String,
    document: Value,
}

/// Read every agent document the binding names. A unit that stays an agent (`agent`, `job`,
/// `scheduled`) needs one for each agent part it holds; a named document must be readable and,
/// when the binding pins a digest, match it.
fn agent_documents(
    inputs: &Inputs,
    packing: &Packing,
    findings: &mut Vec<Finding>,
) -> BTreeMap<String, AgentDoc> {
    let mut docs = BTreeMap::new();
    for (part, file) in &inputs.binding.agent_files {
        let bytes = match std::fs::read(file) {
            Ok(b) => b,
            Err(e) => {
                findings.push(Finding {
                    refusal: "agent_document_missing",
                    detail: format!("part {part}: agent document {}: {e}", file.display()),
                });
                continue;
            }
        };
        let d = sha256(&bytes);
        if let Some(pin) = inputs.binding.doc["parts"][part]["digest"].as_str()
            && pin != d
        {
            findings.push(Finding {
                refusal: "agent_document_digest",
                detail: format!("part {part}: the agent document is {d}, the binding pins {pin}"),
            });
            continue;
        }
        let document = match std::str::from_utf8(&bytes)
            .map_err(|e| e.to_string())
            .and_then(parse_yaml)
        {
            Ok(v) => v,
            Err(e) => {
                findings.push(Finding {
                    refusal: "agent_document_missing",
                    detail: format!("part {part}: agent document {}: {e}", file.display()),
                });
                continue;
            }
        };
        docs.insert(
            part.clone(),
            AgentDoc {
                digest: d,
                document,
            },
        );
    }
    for u in &packing.units {
        if !matches!(u.kind.as_str(), "agent" | "job" | "scheduled") {
            continue;
        }
        for p in &u.parts {
            if inputs.arch["parts"][p]["block"] == "agent"
                && !inputs.binding.agent_files.contains_key(p)
            {
                findings.push(Finding {
                    refusal: "agent_document_missing",
                    detail: format!(
                        "unit {} stays an agent, and the binding names no agent document for its part {p}",
                        u.name
                    ),
                });
            }
        }
    }
    docs
}

/// Build the bundle of an admitted architecture, or every finding of admission.
pub fn build(inputs: &Inputs) -> Result<Bundle, Vec<Finding>> {
    let mut findings = Vec::new();
    let plan = match admit(
        inputs.arch,
        &inputs.binding.doc,
        inputs.packing,
        inputs.accepts,
    ) {
        Ok(plan) => Some(plan),
        Err(f) => {
            findings.extend(f);
            None
        }
    };
    let docs = agent_documents(inputs, inputs.packing, &mut findings);
    findings.extend(grants::check(
        inputs.arch,
        &inputs.binding.doc,
        inputs.packing,
    ));
    findings.extend(addresses(inputs.arch));
    let Some(plan) = plan.filter(|_| findings.is_empty()) else {
        sort(&mut findings);
        return Err(findings);
    };

    let binding = &inputs.binding.doc;
    let mut units = Vec::new();
    for pu in &plan.units {
        units.push(unit(inputs, &plan, pu, &docs));
    }
    let mut agents = Map::new();
    let mut by_digest: BTreeMap<&str, (Vec<&str>, &Value)> = BTreeMap::new();
    for (part, d) in &docs {
        let e = by_digest
            .entry(d.digest.as_str())
            .or_insert((Vec::new(), &d.document));
        e.0.push(part);
    }
    for (d, (parts, document)) in by_digest {
        agents.insert(
            d.to_owned(),
            json!({ "parts": parts, "document": document }),
        );
    }
    let mut runtimes = Map::new();
    for (name, r) in binding["runtimes"].as_object().into_iter().flatten() {
        let image = binding_form::image_reference(binding, name).unwrap_or_default();
        runtimes.insert(
            text(&r["digest"]).to_owned(),
            json!({ "name": name, "image": image }),
        );
    }
    let packing = inputs.packing.to_value();
    let contract = inputs.contract.to_value();
    let mut bundle = json!({
        "format": FORMAT,
        "generator": format!("agenticspec {}", env!("CARGO_PKG_VERSION")),
        "architecture": {
            "name": text(&inputs.arch["architecture"]),
            "revision": revision_digest(inputs.arch),
            "sha256": sha256(inputs.arch_bytes),
            "document": inputs.arch,
        },
        "nested": {},
        "contract": { "digest": digest(&contract), "document": contract },
        "binding": {
            "name": text(&binding["binding"]),
            "digest": digest(binding),
            "document": binding,
        },
        "packing": {
            "digest": digest(&packing),
            "strategy": inputs.packing.strategy,
            "document": packing,
        },
        "units": units,
        "ingress": [],
        "requirements": inputs.arch.get("requirements").cloned().unwrap_or(json!([])),
        "agents": agents,
        "runtimes": runtimes,
    });
    let d = digest(&bundle);
    bundle["digest"] = json!(d);
    Ok(Bundle {
        text: write(&bundle),
        digest: d,
        units: plan.units.len(),
        value: bundle,
    })
}

/// The bundle as YAML, members in the order of [`MEMBERS`], keys sorted inside each.
pub fn write(bundle: &Value) -> String {
    let mut s = String::new();
    for m in MEMBERS {
        if let Some(v) = bundle.get(*m) {
            let mut one = Map::new();
            one.insert((*m).to_owned(), v.clone());
            s.push_str(&serde_yaml_ng::to_string(&Value::Object(one)).unwrap_or_default());
        }
    }
    s
}

/// Recompute the digest of a bundle read back: the canonical JSON of every member but `digest`.
pub fn recompute(bundle: &Value) -> String {
    let mut v = bundle.clone();
    if let Value::Object(m) = &mut v {
        m.remove("digest");
    }
    digest(&v)
}

/// The keys that hold where something is reached. An address is the binding's (per export, and
/// the named destinations of a part's `egress`), never the architecture's, which the bundle
/// copies as written.
pub const ADDRESS_KEYS: &[&str] = &["address", "host", "hostname", "url"];

/// One `address_in_architecture` finding per key of [`ADDRESS_KEYS`] anywhere in the
/// architecture document, by its path; never the address itself.
pub fn addresses(arch: &Value) -> Vec<Finding> {
    fn walk(v: &Value, at: &str, out: &mut Vec<Finding>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    let path = if at.is_empty() {
                        k.clone()
                    } else {
                        format!("{at}.{k}")
                    };
                    if ADDRESS_KEYS.contains(&k.as_str()) {
                        out.push(Finding {
                            refusal: "address_in_architecture",
                            detail: format!(
                                "the architecture has `{path}`, and an address is the binding's, never the architecture's"
                            ),
                        });
                    } else {
                        walk(x, &path, out);
                    }
                }
            }
            Value::Array(a) => {
                for (i, x) in a.iter().enumerate() {
                    walk(x, &format!("{at}[{i}]"), out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(arch, "", &mut out);
    out
}

/// Why a written bundle is refused when it is read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// Its form: the format, a missing member, a digest that does not hold, the binding's form or
    /// the packing's; one line each.
    Invalid(Vec<String>),
    /// Every finding of admission, sorted.
    NotAdmitted(Vec<Finding>),
}

/// Admit a written bundle again from what it carries, for a reader that did not write it: its
/// form and digests, then admission of its architecture under its binding and packing
/// ([`admit`]), its agent documents against the parts that need them and the digests the
/// binding pins, and the secrets and egress of the binding and of each unit
/// ([`grants::check`], [`grants::check_units`]).
pub fn check(bundle: &Value, accepts: &Accepts) -> Result<(), Refused> {
    let mut problems = Vec::new();
    if bundle["format"] != FORMAT {
        problems.push(format!("not {FORMAT}"));
        return Err(Refused::Invalid(problems));
    }
    for m in MEMBERS {
        if bundle.get(*m).is_none() {
            problems.push(format!("no member `{m}`"));
        }
    }
    if !problems.is_empty() {
        return Err(Refused::Invalid(problems));
    }
    if bundle["digest"].as_str() != Some(recompute(bundle).as_str()) {
        problems.push("`digest` is not the digest of the other members".into());
    }
    let arch = &bundle["architecture"]["document"];
    if bundle["architecture"]["revision"].as_str() != Some(revision_digest(arch).as_str()) {
        problems.push("architecture: `revision` is not the digest of its document".into());
    }
    for m in ["contract", "binding", "packing"] {
        if bundle[m]["digest"].as_str() != Some(digest(&bundle[m]["document"]).as_str()) {
            problems.push(format!("{m}: `digest` is not the digest of its document"));
        }
    }
    let binding = &bundle["binding"]["document"];
    problems.extend(
        binding_form::problems(&grants::without_values(binding), arch)
            .into_iter()
            .map(|p| format!("binding: {p}")),
    );
    let packing = match packing::resolve(arch, &bundle["packing"]["document"]) {
        Ok(p) => Some(p),
        Err(e) => {
            problems.extend(e.into_iter().map(|p| format!("packing: {p}")));
            None
        }
    };
    let Some(packing) = packing.filter(|_| problems.is_empty()) else {
        return Err(Refused::Invalid(problems));
    };

    let mut findings = admit(arch, binding, &packing, accepts)
        .err()
        .unwrap_or_default();
    let agents = bundle["agents"].as_object().cloned().unwrap_or_default();
    let agent_of = |part: &str| {
        agents.iter().find_map(|(d, a)| {
            a["parts"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|p| p == part)
                .then_some(d.as_str())
        })
    };
    for u in &packing.units {
        if !matches!(u.kind.as_str(), "agent" | "job" | "scheduled") {
            continue;
        }
        for p in u
            .parts
            .iter()
            .filter(|p| arch["parts"][*p]["block"] == "agent")
        {
            if agent_of(p).is_none() {
                findings.push(Finding {
                    refusal: "agent_document_missing",
                    detail: format!(
                        "unit {} stays an agent, and the bundle carries no agent document for its part {p}",
                        u.name
                    ),
                });
            }
        }
    }
    for (part, entry) in binding["parts"].as_object().into_iter().flatten() {
        if let (Some(pin), Some(d)) = (entry["digest"].as_str(), agent_of(part))
            && pin != d
        {
            findings.push(Finding {
                refusal: "agent_document_digest",
                detail: format!("part {part}: the agent document is {d}, the binding pins {pin}"),
            });
        }
    }
    findings.extend(grants::check(arch, binding, &packing));
    findings.extend(grants::check_units(bundle));
    findings.extend(addresses(arch));
    if findings.is_empty() {
        return Ok(());
    }
    sort(&mut findings);
    findings.dedup();
    Err(Refused::NotAdmitted(findings))
}

/// The volumes a unit claims: one per workspace part, read-write by one copy when it is
/// exclusive, read-only by many when shared read-only, read-write by many otherwise; per copy of
/// its part when it is `per` one.
fn volumes(inputs: &Inputs, parts: &[String]) -> Vec<Value> {
    let mut out = Vec::new();
    for p in parts {
        let part = &inputs.arch["parts"][p];
        if part["block"] != "workspace" {
            continue;
        }
        let access = match value(&part["sharing"]).as_str() {
            Some("exclusive") => "read_write_once",
            Some("shared_read_only") => "read_only_many",
            _ => "read_write_many",
        };
        let mut v = json!({
            "name": p,
            "access": access,
            "per_instance": !part["per"].is_null(),
        });
        if let Some(size) = inputs.binding.doc["parts"][p]["size"].as_str() {
            v["size"] = json!(size);
        }
        out.push(v);
    }
    out
}

fn unit(
    inputs: &Inputs,
    plan: &Plan,
    pu: &PlannedUnit,
    docs: &BTreeMap<String, AgentDoc>,
) -> Value {
    let u = &pu.unit;
    let arch = inputs.arch;
    let binding = &inputs.binding.doc;
    let parts = &arch["parts"];
    let holds = |p: &str| u.parts.iter().any(|x| x == p);
    let agentic = matches!(u.kind.as_str(), "agent" | "job" | "scheduled");

    let code = if agentic {
        let agents: Vec<Value> = u
            .parts
            .iter()
            .filter_map(|p| docs.get(p).map(|d| json!({ "part": p, "agent": d.digest })))
            .collect();
        let mut c = json!({ "role": "agent-runtime", "agents": agents });
        if let Some(r) = binding_form::image_reference(binding, "agent") {
            c["runtime"] = json!(r);
        }
        c
    } else {
        match u.kind.as_str() {
            "core" | "service" => match binding_form::image_reference(binding, "core") {
                Some(r) => json!({ "runtime": r }),
                None => json!({ "role": "core-runtime" }),
            },
            "model" => json!({ "role": "model-server" }),
            _ => json!({ "role": "store" }),
        }
    };

    let c = &binding["claims"][&u.name];
    let accelerators = c["accelerators"]
        .as_u64()
        .unwrap_or(u64::from(u.kind == "model"));
    let claims = json!({
        "cpu_millis": c["cpu_millis"].as_u64().unwrap_or(DEFAULT_CPU_MILLIS),
        "memory_mib": c["memory_mib"].as_u64().unwrap_or(DEFAULT_MEMORY_MIB),
        "accelerators": accelerators,
        "volumes": volumes(inputs, &u.parts),
    });

    let isolation = u
        .parts
        .iter()
        .filter(|p| parts[*p]["block"] == "sandbox")
        .filter_map(|p| value(&parts[p]["isolation"]).as_str())
        .max_by_key(|l| super::binding::isolation_rank(l))
        .unwrap_or("none");

    let checks = match u.kind.as_str() {
        "agent" | "job" | "scheduled" => json!({
            "startup": { "kind": "role", "role": "agent-runtime", "step": "check" },
        }),
        "core" | "service" => json!({
            "ready": { "kind": "http", "path": "/ready" },
            "alive": { "kind": "http", "path": "/alive" },
        }),
        "model" => json!({ "ready": { "kind": "role", "role": "model-server", "step": "ready" } }),
        _ => json!({}),
    };

    let secrets: BTreeSet<String> = u
        .parts
        .iter()
        .flat_map(|p| grants::declared_secrets(binding, p))
        .collect();

    // Every link that leaves the unit, then every named destination its parts declare; each
    // with the secret its declaration names.
    let mut egress = Vec::new();
    let secret_of = |from: &str, to: &str| {
        grants::declared_egress(binding, from)
            .into_iter()
            .find(|d| d.to == to)
            .and_then(|d| d.secret)
    };
    for l in arch["links"].as_array().into_iter().flatten() {
        let (from, to) = (text(&l["from"]), text(&l["to"]));
        if !holds(from) || holds(to) {
            continue;
        }
        let name = link_name(l);
        let mut e = json!({ "link": name, "from": from, "to": to, "carries": text(&l["carries"]) });
        if plan.external.iter().any(|x| x == to) {
            e["external"] = json!(true);
        } else if let Some(c) = plan.crossings.iter().find(|c| c.link == name) {
            e["unit"] = json!(c.to_unit);
            e["transport"] = json!(c.transport);
        }
        if let Some(s) = secret_of(from, to) {
            e["secret"] = json!(s);
        }
        egress.push(e);
    }
    for p in &u.parts {
        for d in grants::declared_egress(binding, p) {
            if parts.get(&d.to).is_some() {
                continue;
            }
            let mut e = json!({ "from": p, "to": d.to, "named": true });
            if let Some(s) = d.secret {
                e["secret"] = json!(s);
            }
            egress.push(e);
        }
    }
    egress.sort_by(|a, b| {
        (text(&a["link"]), text(&a["from"]), text(&a["to"])).cmp(&(
            text(&b["link"]),
            text(&b["from"]),
            text(&b["to"]),
        ))
    });

    let ports: Vec<Value> = inputs
        .contract
        .ports
        .iter()
        .filter(|p| holds(&p.part))
        .map(|p| {
            let settings: Map<String, Value> = p.settings.clone().into_iter().collect();
            json!({
                "port": p.port,
                "part": p.part,
                "instances": p.instances,
                "settings": settings,
            })
        })
        .collect();

    let mut v = json!({
        "name": u.name,
        "kind": u.kind,
        "parts": u.parts,
        "code": code,
        "claims": claims,
        "isolation": isolation,
        "checks": checks,
        "secrets": secrets.into_iter().collect::<Vec<_>>(),
        "egress": egress,
        "roles": pu.roles,
        "ports": ports,
        "source": { "unit": u.name, "parts": u.parts },
    });
    if let Some(r) = u.replicas {
        v["replicas"] = json!(r);
    }
    if u.kind == "scheduled"
        && let Some(s) = schedule(arch, &u.parts)
    {
        v["schedule"] = s;
    }
    v
}

/// The clock of a scheduled unit: the trigger or schedule that wakes its part, the schedule's
/// interval and first firing, and the trigger's `policy` and `max_parallel` as the overlap rule.
fn schedule(arch: &Value, parts: &[String]) -> Option<Value> {
    let l = arch["links"].as_array()?.iter().find(|l| {
        text(&l["carries"]) == "trigger"
            && parts.iter().any(|p| p == text(&l["to"]))
            && is_clock(arch, text(&l["from"]))
    })?;
    let from = text(&l["from"]);
    let p = &arch["parts"][from];
    let (trigger, sched) = if p["block"] == "trigger" {
        (Some(from), value(&p["schedule"]).as_str().unwrap_or(""))
    } else {
        (None, from)
    };
    let s = &arch["parts"][sched];
    let mut out = json!({ "schedule": sched });
    if let Some(t) = trigger {
        out["trigger"] = json!(t);
        for k in ["policy", "max_parallel"] {
            let v = value(&p[k]);
            if !v.is_null() {
                out[k] = v.clone();
            }
        }
    }
    for k in ["every", "next_fire_at"] {
        let v = value(&s[k]);
        if !v.is_null() {
            out[k] = v.clone();
        }
    }
    Some(out)
}
