//! The blocks of the compiled `agentic` system: layers, members and the dependencies between them.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One block: one domain of the system, in one layer directory.
#[derive(Debug, Clone)]
pub struct Block {
    /// `agentic.question`
    pub domain: String,
    /// `question`
    pub name: String,
    pub layer: u32,
    /// `7-collaboration`
    pub dir: String,
    /// `blocks/7-collaboration/question.yaml`
    pub file: String,
    pub summary: String,
    pub family: &'static str,
}

impl Block {
    /// The page, relative to the repository root, without `.md`.
    pub fn doc_id(&self) -> String {
        format!("docs/blocks/{}/{}", self.dir, self.name)
    }

    pub fn route(&self) -> String {
        format!("/docs/blocks/{}/{}", self.dir, self.name)
    }

    /// The layer's name: `collaboration`.
    pub fn layer_name(&self) -> &str {
        layer_name(&self.dir)
    }
}

pub fn layer_name(dir: &str) -> &str {
    dir.split_once('-').map_or(dir, |(_, n)| n)
}

/// The colour family of a block, the same as the architecture renderer's, with three more for the
/// blocks the renderer never draws as parts.
pub fn family(block: &str) -> &'static str {
    match block {
        "principal" | "delegation" => "principal",
        "agent" | "harness" => "agent",
        "channel" | "question" => "collab",
        "work" | "process" | "authority" => "work",
        "gateway" | "model" => "intel",
        "tool" | "knowledge" => "capability",
        "workspace" | "sandbox" | "runtime" => "place",
        "pool" | "resource" => "resource",
        "shared" => "foundation",
        "architecture" => "composition",
        "run" | "kpi" | "experiment" => "evaluation",
        _ => "other",
    }
}

/// The blocks, ordered by layer and then by name, read from `blocks/ess-inputs.yaml`, the domain
/// line of each specification file and the compiled IR.
pub fn blocks(blocks_dir: &Path, ir: &Value) -> Result<Vec<Block>, String> {
    let inputs_path = blocks_dir.join("ess-inputs.yaml");
    let inputs = std::fs::read_to_string(&inputs_path)
        .map_err(|e| format!("blocks/ess-inputs.yaml: {e}"))?;
    let inputs: Value =
        serde_yaml_ng::from_str(&inputs).map_err(|e| format!("blocks/ess-inputs.yaml: {e}"))?;
    let mut out = Vec::new();
    for file in inputs["specification"].as_array().into_iter().flatten() {
        let Some(file) = file.as_str() else { continue };
        let Some((dir, _)) = file.split_once('/') else {
            continue;
        };
        let text = std::fs::read_to_string(blocks_dir.join(file))
            .map_err(|e| format!("blocks/{file}: {e}"))?;
        let Some(domain) = text
            .lines()
            .find_map(|l| l.strip_prefix("domain:"))
            .map(|d| d.trim().to_string())
        else {
            continue;
        };
        let layer = dir
            .split_once('-')
            .and_then(|(n, _)| n.parse::<u32>().ok())
            .ok_or_else(|| format!("blocks/{file}: {domain} is not in a numbered layer"))?;
        let d = &ir["domains"][&domain];
        if d.is_null() {
            return Err(format!(
                "blocks/{file}: {domain} is not in the compiled system"
            ));
        }
        let name = domain
            .strip_prefix("agentic.")
            .unwrap_or(&domain)
            .to_string();
        out.push(Block {
            summary: d["naming"]["summary"].as_str().unwrap_or("").to_string(),
            family: family(&name),
            name,
            domain,
            layer,
            dir: dir.to_string(),
            file: format!("blocks/{file}"),
        });
    }
    out.sort_by(|a, b| (a.layer, &a.name).cmp(&(b.layer, &b.name)));
    if let Some(domains) = ir["domains"].as_object() {
        for d in domains.keys() {
            if !out.iter().any(|b| &b.domain == d) {
                return Err(format!("{d} has no file in a layer directory"));
            }
        }
    }
    Ok(out)
}

/// The two-segment prefix of a qualified name: `agentic.question.Answer` is `agentic.question`.
pub fn prefix(name: &str) -> String {
    name.split('.').take(2).collect::<Vec<_>>().join(".")
}

fn declared(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(o) => {
            if o.get("kind").and_then(Value::as_str) == Some("declared")
                && let Some(n) = o.get("name").and_then(Value::as_str)
            {
                out.insert(n.to_string());
            }
            o.values().for_each(|x| declared(x, out));
        }
        Value::Array(xs) => xs.iter().for_each(|x| declared(x, out)),
        _ => {}
    }
}

/// Every dependency between two domains, counted the way `scripts/check-layers.sh` counts them:
/// one per (from, to, member) where a member of `from` names a type of `to` (directly or through
/// the types it uses), the identifier of an entity of `to`, or relates to an entity of `to`.
pub fn layer_edges(ir: &Value) -> BTreeMap<(String, String), usize> {
    let mut identity: BTreeMap<String, String> = BTreeMap::new();
    for e in ir["entities"]
        .as_object()
        .into_iter()
        .flat_map(|o| o.values())
    {
        if let (Some(t), Some(d)) = (
            e["identity"]["type_ref"]["name"].as_str(),
            e["domain"].as_str(),
        ) {
            identity.insert(t.to_string(), d.to_string());
        }
    }
    let mut direct: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (k, v) in ir["types"].as_object().into_iter().flatten() {
        let mut s = BTreeSet::new();
        declared(v, &mut s);
        direct.insert(k.clone(), s);
    }
    let owner = |t: &str| identity.get(t).cloned().unwrap_or_else(|| prefix(t));
    let mut raw: BTreeSet<(String, String, String)> = BTreeSet::new();
    for c in ["entities", "commands", "events", "views", "errors", "types"] {
        for (k, v) in ir[c].as_object().into_iter().flatten() {
            let from = v["domain"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| prefix(k));
            let mut set = BTreeSet::new();
            declared(v, &mut set);
            loop {
                let mut next = set.clone();
                for t in &set {
                    if let Some(d) = direct.get(t) {
                        next.extend(d.iter().cloned());
                    }
                }
                if next.len() == set.len() {
                    break;
                }
                set = next;
            }
            for t in &set {
                let to = owner(t);
                if to != from {
                    raw.insert((from.clone(), to, k.clone()));
                }
            }
        }
    }
    for e in ir["entities"]
        .as_object()
        .into_iter()
        .flat_map(|o| o.values())
    {
        let from = e["domain"].as_str().unwrap_or("").to_string();
        let name = e["name"].as_str().unwrap_or("");
        for r in e["relations"].as_array().into_iter().flatten() {
            let to = prefix(r["target"].as_str().unwrap_or(""));
            if to != from {
                raw.insert((
                    from.clone(),
                    to,
                    format!("{name}.{}", r["name"].as_str().unwrap_or("")),
                ));
            }
        }
    }
    // A binding makes the domain of its event depend on the domain of the command it invokes, as
    // scripts/check-layers.sh counts it.
    for (name, b) in ir["bindings"].as_object().into_iter().flatten() {
        let from = prefix(b["event"].as_str().unwrap_or(""));
        let to = prefix(b["command"].as_str().unwrap_or(""));
        if !from.is_empty() && to != from {
            raw.insert((from, to, format!("binding:{name}")));
        }
    }
    let mut out = BTreeMap::new();
    for (from, to, _) in raw {
        *out.entry((from, to)).or_insert(0) += 1;
    }
    out
}

/// The members of one domain, by kind, as the IR lists them.
pub fn members<'a>(ir: &'a Value, domain: &str, kind: &str) -> Vec<&'a Value> {
    let mut out: Vec<&Value> = ir["domains"][domain][kind]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|n| ir[kind].get(n))
        .collect();
    out.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    out
}

/// The last segment of a qualified name.
pub fn short(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}
