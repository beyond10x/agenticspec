//! The projections of an ingress's exports, one per export, each generated from the contract and
//! the compiled blocks, never authored ([`FILES`] names the files of each protocol).
//!
//! For an `http` export (here): the `ess-http/1` protocol binding over exactly the interface's
//! commands and views, and the OpenAPI document of those operations. The binding leaves every
//! path open (a server binding that `refines:` it fills them in); the OpenAPI document is the one
//! ESS writes for the component that serves the blocks (`ess generate --kind openapi`), narrowed
//! to the interface's operations and what they reference, so its paths are the ones ESS derives.
//! The other protocols have a module each: `event_stream` the stream list ([`stream`]), `cli` an
//! `ess-cli/2` binding ([`cli`]), `tool` a tool list ([`tool`]), `agent` an agent card
//! ([`agent`]) and `metrics_scrape` a metric list ([`metrics`]). A `ui` export has none here: the
//! view layer derives it (`agenticspec ui derive`). `docs/contract.md` states every format.
//!
//! The functions here are pure: the command (`agenticspec ingress project`, in `crate::cli`)
//! calls `ess` once for the compiled blocks and once for their OpenAPI document.

pub mod agent;
pub mod cli;
pub mod metrics;
pub mod stream;
pub mod tool;

use super::derive::Contract;
use super::ingress::{ContractIngress, OpKind};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// The projection files of each protocol an export may name, by suffix after `<ingress>.`. A
/// `ui` export has none: the view layer derives it.
pub const FILES: &[(&str, &[&str])] = &[
    ("http", &["http.yaml", "openapi.yaml"]),
    ("event_stream", &["streams.yaml"]),
    ("cli", &["cli.yaml"]),
    ("tool", &["tools.yaml"]),
    ("agent", &["agent.yaml"]),
    ("metrics_scrape", &["metrics.yaml"]),
];

/// The projection files of an ingress's export of a protocol; empty for a protocol without one.
pub fn files_of(ingress: &str, protocol: &str) -> Vec<String> {
    FILES
        .iter()
        .find(|(p, _)| *p == protocol)
        .map(|(_, s)| s.iter().map(|s| format!("{ingress}.{s}")).collect())
        .unwrap_or_default()
}

/// Whether a file name is the name of a projection of some ingress.
pub fn is_projection(name: &str) -> bool {
    FILES
        .iter()
        .flat_map(|(_, s)| s.iter())
        .any(|s| name.len() > s.len() + 1 && name.ends_with(&format!(".{s}")))
}

/// Every export of a contract that has a projection, as `(ingress, protocol)`, ingresses in
/// contract order and each one's exports in the order written.
pub fn projected(contract: &Contract) -> Vec<(&ContractIngress, &str)> {
    contract
        .ingress
        .iter()
        .flat_map(|i| {
            i.exports
                .iter()
                .filter(|e| FILES.iter().any(|(p, _)| p == e))
                .map(move |e| (i, e.as_str()))
        })
        .collect()
}

/// The format of a protocol binding.
pub const HTTP_FORMAT: &str = "ess-http/1";

/// The specification a binding is pinned to: the `system`, `version` and `source_digest` of
/// the compiled blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub system: String,
    pub version: String,
    pub source_digest: String,
}

impl Pin {
    /// The pin of a compiled system, from the text `ess specify compile --format json` prints:
    /// the digest is the SHA-256 of its compact form, as ESS computes it.
    pub fn of_ir(text: &str) -> Result<Pin, String> {
        let ir: Value =
            serde_json::from_str(text).map_err(|e| format!("ess specify compile: {e}"))?;
        let system = ir["system"]
            .as_str()
            .ok_or("ess specify compile: no `system`")?;
        let version = match &ir["version"] {
            Value::Number(n) => format!("v{n}"),
            Value::String(s) => s.clone(),
            _ => return Err("ess specify compile: no `version`".into()),
        };
        let digest: String = Sha256::digest(compact(text).as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Ok(Pin {
            system: system.to_owned(),
            version,
            source_digest: format!("sha256:{digest}"),
        })
    }
}

/// JSON text without the whitespace between its tokens, every key in the order written.
pub fn compact(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escaped = false;
    for c in text.chars() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if !c.is_whitespace() {
            out.push(c);
        }
    }
    out
}

/// The name of the projection files of an ingress's `http` export.
pub fn file_names(ingress: &str) -> [String; 2] {
    [
        format!("{ingress}.http.yaml"),
        format!("{ingress}.openapi.yaml"),
    ]
}

/// The ingresses of a contract with an `http` export.
pub fn http_ingresses(contract: &Contract) -> Vec<&ContractIngress> {
    contract
        .ingress
        .iter()
        .filter(|i| i.exports.iter().any(|e| e == "http"))
        .collect()
}

fn header(contract: &Contract, ingress: &ContractIngress, what: &str) -> String {
    header_of(contract, ingress, "http", what)
}

/// The first lines of a projection file: what it is, of which export, and that it is generated.
fn header_of(contract: &Contract, ingress: &ContractIngress, protocol: &str, what: &str) -> String {
    format!(
        "# {what} of the {protocol} export of ingress `{}` of architecture `{}`.\n\
         # Generated by `agenticspec ingress project`; do not edit.\n",
        ingress.name, contract.architecture
    )
}

/// The `specification:` key of a projection pinned to the compiled blocks.
fn pin_text(pin: &Pin) -> String {
    format!(
        "specification:\n  system: {}\n  version: {}\n  source_digest: {}\n",
        pin.system, pin.version, pin.source_digest
    )
}

/// The `ingress:` key of a projection: the architecture, the ingress, its audience and whether
/// its callers authenticate.
fn ingress_text(contract: &Contract, ingress: &ContractIngress) -> String {
    field(
        "ingress",
        &json!({
            "architecture": contract.architecture,
            "name": ingress.name,
            "audience": ingress.audience,
            "authenticated": ingress.authenticated,
        }),
    )
}

/// `key: value` as YAML, the keys of the value in name order.
fn field(key: &str, v: &Value) -> String {
    let mut m = Map::new();
    m.insert(key.to_owned(), v.clone());
    serde_yaml_ng::to_string(&Value::Object(m)).unwrap_or_default()
}

/// A list of items under `key`, each item's keys in the order given; `key: []` when empty.
fn list(key: &str, items: &[Vec<(&str, Value)>]) -> String {
    if items.is_empty() {
        return format!("{key}: []\n");
    }
    let mut s = format!("{key}:\n");
    for item in items {
        for (i, (k, v)) in item.iter().enumerate() {
            for (j, line) in field(k, v).lines().enumerate() {
                let lead = if i == 0 && j == 0 { "  - " } else { "    " };
                s.push_str(format!("{lead}{line}").trim_end());
                s.push('\n');
            }
        }
    }
    s
}

/// The last segment of a qualified name.
fn short(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// The domain of a qualified name without the system: `gateway` for
/// `agentic.gateway.DrainGateway`.
fn domain_short(name: &str) -> &str {
    let mut it = name.rsplit('.');
    it.next();
    it.next().unwrap_or(name)
}

/// A name in kebab case: `AgentRevisions` and `agent_revisions` give `agent-revisions`.
fn kebab(text: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in text.chars() {
        if c == '_' || c == '-' || c == ' ' {
            out.push('-');
            prev_lower = false;
        } else if c.is_ascii_uppercase() {
            if prev_lower {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else {
            out.push(c);
            prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        }
    }
    out
}

/// The declaration of a command, view or event in the compiled blocks; `None` when they lack it.
fn declared<'a>(ir: &'a Value, kind: OpKind, name: &str) -> Option<&'a Value> {
    let table = match kind {
        OpKind::Command => "commands",
        OpKind::View => "views",
        OpKind::Feed => "events",
        OpKind::Metric => return None,
    };
    ir[table].get(name).filter(|v| v.is_object())
}

/// What a declaration says of itself: its `naming.summary`, else its `naming.display`, else its
/// short name.
fn described(entry: &Value, name: &str) -> String {
    entry["naming"]["summary"]
        .as_str()
        .or_else(|| entry["naming"]["display"].as_str())
        .unwrap_or_else(|| short(name))
        .to_owned()
}

/// The display name of a declaration: its `naming.display`, else its short name.
fn displayed(entry: &Value, name: &str) -> String {
    entry["naming"]["display"]
        .as_str()
        .unwrap_or_else(|| short(name))
        .to_owned()
}

/// The ESS errors a command refuses with, by its outcomes, each once, in name order.
fn errors_of(entry: &Value) -> Vec<String> {
    let set: BTreeSet<String> = entry["outcomes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|o| o["error"].as_str().map(str::to_owned))
        .collect();
    set.into_iter().collect()
}

/// The operations of an OpenAPI document by `operationId`: the path and the operation.
fn routes(full: &Value) -> BTreeMap<&str, (&str, &Value)> {
    let mut out = BTreeMap::new();
    for (path, item) in full["paths"].as_object().into_iter().flatten() {
        for op in item.as_object().into_iter().flatten().map(|(_, op)| op) {
            if let Some(id) = op["operationId"].as_str() {
                out.insert(id, (path.as_str(), op));
            }
        }
    }
    out
}

/// A command's or view's input as one JSON Schema that stands alone, from its operation in the
/// OpenAPI document: a command's request body, a view's query parameters as the properties of an
/// object. Every schema it references is copied under `$defs`, as `ess generate --kind schema`
/// writes it.
fn input_schema(full: &Value, kind: OpKind, op: &Value) -> Result<Value, String> {
    let root = match kind {
        OpKind::Command => {
            let body = &op["requestBody"]["content"]["application/json"]["schema"];
            match body["$ref"].as_str().and_then(|r| r.strip_prefix('#')) {
                Some(p) => full
                    .pointer(p)
                    .cloned()
                    .ok_or_else(|| format!("the OpenAPI document of the blocks has no {p}"))?,
                None if body.is_object() => body.clone(),
                None => json!({"type": "object", "properties": {}, "additionalProperties": false}),
            }
        }
        _ => {
            let mut properties = Map::new();
            let mut required = Vec::new();
            for p in op["parameters"].as_array().into_iter().flatten() {
                let Some(name) = p["name"].as_str() else {
                    continue;
                };
                let mut schema = p["schema"].clone();
                if let (Some(d), Some(m)) = (p["description"].as_str(), schema.as_object_mut()) {
                    m.insert("description".into(), json!(d));
                }
                properties.insert(name.to_owned(), schema);
                if p["required"] == true {
                    required.push(json!(name));
                }
            }
            let mut root = json!({
                "type": "object",
                "properties": properties,
                "additionalProperties": false,
            });
            if !required.is_empty() {
                root["required"] = Value::Array(required);
            }
            root
        }
    };
    let mut pending = Vec::new();
    refs(&root, &mut pending);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut defs = Map::new();
    while let Some(p) = pending.pop() {
        if !seen.insert(p.clone()) {
            continue;
        }
        let Some(name) = p.strip_prefix("/components/schemas/") else {
            return Err(format!("a schema refers outside the schemas: {p}"));
        };
        let Some(v) = full.pointer(&p) else {
            return Err(format!("the OpenAPI document of the blocks has no {p}"));
        };
        refs(v, &mut pending);
        defs.insert(name.replace("~1", "/").replace("~0", "~"), local(v));
    }
    let mut out = local(&root);
    if !defs.is_empty() {
        out["$defs"] = Value::Object(defs);
    }
    Ok(out)
}

/// A schema with each reference to `#/components/schemas/<name>` pointing to `#/$defs/<name>`.
fn local(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, x)| match (k.as_str(), x) {
                    ("$ref", Value::String(r)) => (
                        k.clone(),
                        json!(match r.strip_prefix("#/components/schemas/") {
                            Some(name) => format!("#/$defs/{name}"),
                            None => r.clone(),
                        }),
                    ),
                    _ => (k.clone(), local(x)),
                })
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(local).collect()),
        other => other.clone(),
    }
}

/// One operation of an interface as a tool or a skill: its ESS wire name (the last segment of
/// its path in the OpenAPI document), what it says of itself, its input schema and, for a
/// command, the ESS errors it refuses with.
struct Callable {
    kind: OpKind,
    name: String,
    wire: String,
    display: String,
    description: String,
    errors: Vec<String>,
    input: Value,
}

/// The commands and views of an interface of the kinds given, as [`Callable`]s, in interface
/// order. Refused, naming every one, when the OpenAPI document lacks an operation; refused when
/// two operations have the same wire name.
fn callables(
    ingress: &ContractIngress,
    kinds: &[OpKind],
    ir: &Value,
    full: &Value,
    what: &str,
) -> Result<Vec<Callable>, String> {
    let routes = routes(full);
    let mut out = Vec::new();
    let mut missing = Vec::new();
    let mut wires: BTreeMap<String, String> = BTreeMap::new();
    for o in ingress
        .operations
        .iter()
        .filter(|o| kinds.contains(&o.kind))
    {
        let Some((path, op)) = routes.get(o.name.as_str()) else {
            missing.push(o.name.as_str());
            continue;
        };
        let wire = path.rsplit('/').next().unwrap_or(path).to_owned();
        if let Some(prev) = wires.insert(wire.clone(), o.name.clone()) {
            return Err(format!(
                "ingress {}: the {what} name {wire} is the wire name of both {prev} and {}",
                ingress.name, o.name
            ));
        }
        let entry = declared(ir, o.kind, &o.name)
            .cloned()
            .unwrap_or(Value::Null);
        out.push(Callable {
            kind: o.kind,
            name: o.name.clone(),
            wire,
            display: displayed(&entry, &o.name),
            description: described(&entry, &o.name),
            errors: errors_of(&entry),
            input: input_schema(full, o.kind, op)?,
        });
    }
    if !missing.is_empty() {
        return Err(format!(
            "ingress {}: the OpenAPI document of the blocks has no operation {}",
            ingress.name,
            missing.join(", ")
        ));
    }
    Ok(out)
}

/// The `ess-http/1` protocol binding of an ingress's `http` export: one open route per command
/// (`post`) and per view (`get`) of its interface, commands first, each by name.
pub fn http_binding(contract: &Contract, ingress: &ContractIngress, pin: &Pin) -> String {
    let mut s = header(contract, ingress, "The ess-http/1 protocol binding");
    s.push_str("# Every path is open: a server binding that refines this one fills it in.\n");
    s.push_str(&format!("type: {HTTP_FORMAT}\n"));
    s.push_str(&pin_text(pin));
    let routes: Vec<(&str, &str, &str)> = ingress
        .operations
        .iter()
        .filter_map(|o| match o.kind {
            OpKind::Command => Some(("command", o.name.as_str(), "post")),
            OpKind::View => Some(("view", o.name.as_str(), "get")),
            _ => None,
        })
        .collect();
    if routes.is_empty() {
        s.push_str("routes: []\n");
    } else {
        s.push_str("routes:\n");
    }
    for (kind, name, method) in routes {
        s.push_str(&format!("  - {{{kind}: {name}, method: {method}}}\n"));
    }
    s
}

/// The `$ref` targets in a value, as JSON pointers (`/components/schemas/<name>`).
fn refs(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                match (k.as_str(), x) {
                    ("$ref", Value::String(r)) => {
                        if let Some(p) = r.strip_prefix('#') {
                            out.push(p.to_owned());
                        }
                    }
                    _ => refs(x, out),
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|x| refs(x, out)),
        _ => {}
    }
}

/// The value at a JSON pointer, with the pointer as segments.
fn segments(pointer: &str) -> Vec<String> {
    pointer
        .split('/')
        .skip(1)
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .collect()
}

/// Insert `v` at `path` under `root`, creating the objects on the way.
fn insert(root: &mut Map<String, Value>, path: &[String], v: Value) {
    let Some((last, rest)) = path.split_last() else {
        return;
    };
    let mut at = root;
    for p in rest {
        at = match at
            .entry(p.clone())
            .or_insert_with(|| Value::Object(Map::new()))
        {
            Value::Object(m) => m,
            _ => return,
        };
    }
    at.insert(last.clone(), v);
}

/// The OpenAPI document of an ingress's `http` export: ESS's document of the component that
/// serves the blocks (`full`), with only the operations of the interface (by `operationId`, the
/// command's or view's name), the tags they use and the components they reference. Refused
/// when an operation of the interface has no operation in `full`.
pub fn openapi(
    full: &Value,
    contract: &Contract,
    ingress: &ContractIngress,
) -> Result<Value, String> {
    let wanted: BTreeSet<&str> = ingress
        .operations
        .iter()
        .filter(|o| matches!(o.kind, OpKind::Command | OpKind::View))
        .map(|o| o.name.as_str())
        .collect();
    let mut found: BTreeSet<&str> = BTreeSet::new();
    let mut paths = Map::new();
    let mut tags: BTreeSet<String> = BTreeSet::new();
    let mut pending: Vec<String> = Vec::new();
    for (path, item) in full["paths"].as_object().into_iter().flatten() {
        let mut kept = Map::new();
        for (method, op) in item.as_object().into_iter().flatten() {
            let Some(id) = op["operationId"].as_str() else {
                continue;
            };
            let Some(name) = wanted.get(id) else {
                continue;
            };
            found.insert(name);
            for t in op["tags"].as_array().into_iter().flatten() {
                if let Some(t) = t.as_str() {
                    tags.insert(t.to_owned());
                }
            }
            refs(op, &mut pending);
            kept.insert(method.clone(), op.clone());
        }
        if !kept.is_empty() {
            paths.insert(path.clone(), Value::Object(kept));
        }
    }
    let missing: Vec<&str> = wanted.difference(&found).copied().collect();
    if !missing.is_empty() {
        return Err(format!(
            "ingress {}: the OpenAPI document of the blocks has no operation {}",
            ingress.name,
            missing.join(", ")
        ));
    }
    // Every component the kept operations reference, transitively.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut referenced = Map::new();
    while let Some(p) = pending.pop() {
        if !seen.insert(p.clone()) {
            continue;
        }
        let Some(v) = full.pointer(&p) else {
            return Err(format!("the OpenAPI document of the blocks has no {p}"));
        };
        refs(v, &mut pending);
        insert(&mut referenced, &segments(&p), v.clone());
    }
    let mut out = Map::new();
    for (k, v) in full.as_object().into_iter().flatten() {
        match k.as_str() {
            "paths" | "tags" | "components" => {}
            k if k.starts_with("x-") && !referenced.contains_key(k) => {}
            _ => {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    out.insert("paths".into(), Value::Object(paths));
    let kept_tags: Vec<Value> = full["tags"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|t| t["name"].as_str().is_some_and(|n| tags.contains(n)))
        .cloned()
        .collect();
    if !kept_tags.is_empty() {
        out.insert("tags".into(), Value::Array(kept_tags));
    }
    for (k, v) in referenced {
        out.insert(k, v);
    }
    if let Some(info) = out.get_mut("info").and_then(Value::as_object_mut) {
        info.insert(
            "title".into(),
            json!(format!(
                "{} ingress {}",
                contract.architecture, ingress.name
            )),
        );
        info.insert(
            "x-agenticspec-ingress".into(),
            json!({
                "architecture": contract.architecture,
                "ingress": ingress.name,
                "audience": ingress.audience,
                "authenticated": ingress.authenticated,
            }),
        );
    }
    Ok(Value::Object(out))
}

/// The OpenAPI document as written: a header, then YAML with every key in name order.
pub fn openapi_text(
    contract: &Contract,
    ingress: &ContractIngress,
    doc: &Value,
) -> Result<String, String> {
    let body = serde_yaml_ng::to_string(doc).map_err(|e| e.to_string())?;
    Ok(format!(
        "{}{body}",
        header(contract, ingress, "The OpenAPI document")
    ))
}
