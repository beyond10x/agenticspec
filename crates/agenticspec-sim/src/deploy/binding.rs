//! The binding (`agenticspec-binding/1`): for one architecture and one target, which concrete
//! thing plays each part, the transports of links that cross units, the runtimes pinned by
//! digest, the claims of each unit, and the packing (`packing:`). A binding may `refines:` a base
//! binding; [`load`] resolves the chain into one document, the child's keys over the base's.

use crate::doc::load_yaml;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "agenticspec-binding/1";

/// The keys a binding may hold.
const KEYS: &[&str] = &[
    "format",
    "binding",
    "summary",
    "architecture",
    "refines",
    "target",
    "parts",
    "transports",
    "runtimes",
    "claims",
    "packing",
];

/// The keys of one part's entry under `parts:`.
const PART_KEYS: &[&str] = &[
    "agent", "digest", "state", "inside", "secrets", "egress", "size",
];

/// The keys of a runtime pinned under `runtimes:`.
const RUNTIME_KEYS: &[&str] = &["image", "digest"];

/// The isolation levels of agenticspec, lowest first (`agentic.shared.IsolationLevel`).
pub const ISOLATION: &[&str] = &["none", "process", "container", "user_kernel", "micro_vm"];

/// The isolation a target gives one unit when the binding names none under `target:`.
pub const DEFAULT_ISOLATION: &str = "container";

/// A binding with its `refines:` chain resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    /// The resolved document, without `refines`.
    pub doc: Value,
    /// Per part that names an agent document, the file it names, read from the directory of the
    /// binding file that named it.
    pub agent_files: BTreeMap<String, PathBuf>,
}

/// `base` with `over` laid over it: objects merge key by key, any other value replaces.
fn merge(base: &mut Value, over: &Value) {
    match (base, over) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                merge(b.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (b, o) => *b = o.clone(),
    }
}

/// Read a binding file and every base it refines, nearest last, and resolve them into one.
pub fn load(path: &Path) -> Result<Binding, String> {
    let mut chain: Vec<(Value, PathBuf)> = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut at = path.to_path_buf();
    loop {
        let key = std::fs::canonicalize(&at).unwrap_or_else(|_| at.clone());
        if seen.contains(&key) {
            return Err(format!(
                "{}: `refines:` comes back to a binding of its own chain",
                at.display()
            ));
        }
        seen.push(key);
        let doc = load_yaml(&at)?;
        if doc["format"] != FORMAT {
            return Err(format!("{}: not {FORMAT}", at.display()));
        }
        let dir = at.parent().unwrap_or(Path::new("")).to_path_buf();
        let next = match &doc["refines"] {
            Value::Null => None,
            Value::String(base) => Some(dir.join(base)),
            _ => {
                return Err(format!(
                    "{}: `refines:` names one binding file",
                    at.display()
                ));
            }
        };
        chain.push((doc, dir));
        match next {
            Some(n) => at = n,
            None => break,
        }
    }
    let mut doc = Value::Object(Map::new());
    let mut agent_files = BTreeMap::new();
    for (layer, dir) in chain.iter().rev() {
        merge(&mut doc, layer);
        if let Some(parts) = layer["parts"].as_object() {
            for (name, entry) in parts {
                if let Some(file) = entry["agent"].as_str() {
                    agent_files.insert(name.clone(), dir.join(file));
                }
            }
        }
    }
    if let Value::Object(m) = &mut doc {
        m.remove("refines");
    }
    Ok(Binding { doc, agent_files })
}

/// The problems of a resolved binding's form for one architecture document, one line each:
/// an unknown key, a part that is not in the document, a value of the wrong kind. Empty when
/// the form is right. The packing section is checked by [`crate::deploy::packing::resolve`].
pub fn problems(binding: &Value, arch: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let Some(top) = binding.as_object() else {
        return vec!["a binding is a mapping".into()];
    };
    for k in top.keys() {
        if !KEYS.contains(&k.as_str()) {
            out.push(format!("unknown key `{k}`"));
        }
    }
    if !binding["binding"].is_string() {
        out.push("`binding:` names the binding".into());
    }
    if let Some(name) = binding["architecture"].as_str()
        && Some(name) != arch["architecture"].as_str()
    {
        out.push(format!(
            "`architecture: {name}` is not the document's `{}`",
            arch["architecture"].as_str().unwrap_or("")
        ));
    }
    match &binding["target"] {
        Value::Null => {}
        Value::Object(t) => {
            for (k, v) in t {
                match k.as_str() {
                    "isolation" if v.as_str().is_some_and(|l| ISOLATION.contains(&l)) => {}
                    "isolation" => out.push(format!(
                        "target: isolation is one of {}",
                        ISOLATION.join(", ")
                    )),
                    other => out.push(format!("target: unknown key `{other}`")),
                }
            }
        }
        _ => out.push("`target:` is a mapping".into()),
    }
    let parts = &arch["parts"];
    if let Some(entries) = binding["parts"].as_object() {
        for (name, entry) in entries {
            if parts.get(name).is_none() {
                out.push(format!("parts: `{name}` is not a part of the document"));
            }
            let Some(entry) = entry.as_object() else {
                out.push(format!("parts: `{name}` is a mapping"));
                continue;
            };
            for (k, v) in entry {
                if !PART_KEYS.contains(&k.as_str()) {
                    out.push(format!("parts: {name}: unknown key `{k}`"));
                    continue;
                }
                if k == "egress" {
                    egress_problems(name, v, parts, &mut out);
                    continue;
                }
                let ok = match k.as_str() {
                    "state" => matches!(v.as_str(), Some("memory" | "service")),
                    "secrets" => v.as_array().is_some_and(|a| a.iter().all(Value::is_string)),
                    _ => v.is_string(),
                };
                if !ok {
                    out.push(match k.as_str() {
                        "state" => format!("parts: {name}: state is memory or service"),
                        "secrets" => format!("parts: {name}: secrets is a list of names"),
                        _ => format!("parts: {name}: `{k}` is text"),
                    });
                }
            }
        }
    } else if !binding["parts"].is_null() {
        out.push("`parts:` is a mapping".into());
    }
    if let Some(t) = binding["transports"].as_object() {
        for (carries, v) in t {
            if !v.is_string() {
                out.push(format!("transports: `{carries}` names one transport"));
            }
        }
    } else if !binding["transports"].is_null() {
        out.push("`transports:` is a mapping from a carried value to a transport".into());
    }
    if let Some(r) = binding["runtimes"].as_object() {
        for (name, v) in r {
            let Some(m) = v.as_object() else {
                out.push(format!("runtimes: `{name}` is a mapping"));
                continue;
            };
            for k in m.keys() {
                if !RUNTIME_KEYS.contains(&k.as_str()) {
                    out.push(format!("runtimes: {name}: unknown key `{k}`"));
                }
            }
            if !m
                .get("digest")
                .and_then(Value::as_str)
                .is_some_and(is_digest)
            {
                out.push(format!(
                    "runtimes: {name}: `digest:` is sha256:<64 hex digits>; a runtime is pinned by its digest"
                ));
            }
            if !m.get("image").and_then(Value::as_str).is_some_and(is_image) {
                out.push(format!(
                    "runtimes: {name}: `image:` names the image without tag or digest (`registry/name`); the digest goes in `digest:`"
                ));
            }
        }
    } else if !binding["runtimes"].is_null() {
        out.push("`runtimes:` is a mapping".into());
    }
    if let Some(c) = binding["claims"].as_object() {
        for (unit, v) in c {
            let Some(m) = v.as_object() else {
                out.push(format!("claims: `{unit}` is a mapping"));
                continue;
            };
            for (k, x) in m {
                let positive = x.as_u64().is_some_and(|n| n > 0);
                match k.as_str() {
                    "cpu_millis" | "memory_mib" if positive => {}
                    "accelerators" if x.as_u64().is_some() => {}
                    "cpu_millis" | "memory_mib" | "accelerators" => {
                        out.push(format!("claims: {unit}: `{k}` is a whole number above 0"))
                    }
                    other => out.push(format!("claims: {unit}: unknown key `{other}`")),
                }
            }
        }
    } else if !binding["claims"].is_null() {
        out.push("`claims:` is a mapping from a unit name to its claims".into());
    }
    out
}

/// The problems of one part's `egress:`: a list of destinations, each a part of the document or
/// a name the target resolves ([`super::grants::is_name`]), written as the name or as
/// `{to: <name>, secret: <secret name>}`. Whether a secret is a name is checked by
/// [`super::grants::values`].
fn egress_problems(part: &str, v: &Value, parts: &Value, out: &mut Vec<String>) {
    let Some(list) = v.as_array() else {
        out.push(format!("parts: {part}: egress is a list of destinations"));
        return;
    };
    let destination = |to: &str| parts.get(to).is_some() || super::grants::is_name(to);
    for (i, e) in list.iter().enumerate() {
        let to = match e {
            Value::String(to) => Some(to.as_str()),
            Value::Object(m) => {
                for k in m.keys() {
                    if k != "to" && k != "secret" {
                        out.push(format!("parts: {part}: egress[{i}]: unknown key `{k}`"));
                    }
                }
                m.get("to").and_then(Value::as_str)
            }
            _ => None,
        };
        match to {
            Some(to) if destination(to) => {}
            Some(_) => out.push(format!(
                "parts: {part}: egress[{i}] names neither a part of the document nor a destination name (^[a-z][a-z0-9-]*$)"
            )),
            None => out.push(format!(
                "parts: {part}: egress[{i}] is a destination name or {{to: <name>, secret: <name>}}"
            )),
        }
    }
}

/// An image name without tag or digest: not empty, no space, no `@`, and no `:` after its last
/// `/` (a `:` before it is a registry's port).
pub fn is_image(s: &str) -> bool {
    let last = s.rsplit('/').next().unwrap_or(s);
    !s.is_empty() && !last.is_empty() && !s.contains(['@', ' ']) && !last.contains(':')
}

/// The image reference of a runtime the binding pins: `<image>@<digest>`.
pub fn image_reference(binding: &Value, runtime: &str) -> Option<String> {
    let r = &binding["runtimes"][runtime];
    Some(format!(
        "{}@{}",
        r["image"].as_str()?,
        r["digest"].as_str()?
    ))
}

/// `sha256:` and 64 lower-case hex digits.
pub fn is_digest(s: &str) -> bool {
    s.strip_prefix("sha256:").is_some_and(|h| {
        h.len() == 64
            && h.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    })
}

/// The isolation level the target gives one unit.
pub fn target_isolation(binding: &Value) -> &str {
    binding["target"]["isolation"]
        .as_str()
        .unwrap_or(DEFAULT_ISOLATION)
}

/// The rank of an isolation level, lowest 0; an unknown level ranks lowest.
pub fn isolation_rank(level: &str) -> usize {
    ISOLATION.iter().position(|l| *l == level).unwrap_or(0)
}
