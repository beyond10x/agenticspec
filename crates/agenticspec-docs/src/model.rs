//! The blocks of the compiled `agentic` system: layers, members and the dependencies between them.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The layer a pack's blocks sort under: above every layer of the core.
pub const PACK_LAYER: u32 = 11;

/// One block: one domain of the system, in one layer directory of the core or in one pack.
#[derive(Debug, Clone)]
pub struct Block {
    /// `agentic.question`, `agentic.scm.change`
    pub domain: String,
    /// `question`, `scm.change`: the domain without `agentic.`, as the KPI catalogues name it
    pub name: String,
    pub layer: u32,
    /// `7-collaboration`, or the pack: `source-control`
    pub dir: String,
    /// `blocks` for the core, `packs` for a pack
    pub section: &'static str,
    /// The page's file name without `.md`: `question`, `change`
    pub page: String,
    /// `blocks/7-collaboration/question.yaml`
    pub file: String,
    pub summary: String,
    pub family: &'static str,
}

impl Block {
    /// The page, relative to the repository root, without `.md`.
    pub fn doc_id(&self) -> String {
        format!("docs/{}/{}/{}", self.section, self.dir, self.page)
    }

    pub fn route(&self) -> String {
        format!("/docs/{}/{}/{}", self.section, self.dir, self.page)
    }

    /// The layer's name: `collaboration`; for a pack, the pack.
    pub fn layer_name(&self) -> &str {
        layer_name(&self.dir)
    }

    /// Whether the block is a pack's.
    pub fn in_pack(&self) -> bool {
        self.section == "packs"
    }

    /// Where the block lives, for lists: `layer 7`, `pack source-control`.
    pub fn place(&self) -> String {
        if self.in_pack() {
            format!("pack {}", self.dir)
        } else {
            format!("layer {}", self.layer)
        }
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
        "tool" | "knowledge" | "store" => "capability",
        "workspace" | "sandbox" | "runtime" => "place",
        "pool" | "resource" => "resource",
        "shared" => "foundation",
        "architecture" | "deployment" => "composition",
        "run" | "kpi" | "experiment" => "evaluation",
        _ => "other",
    }
}

/// The blocks, ordered by layer and then by name, read from `blocks/ess-inputs.yaml`, the domain
/// line of each specification file and the compiled IR.
pub fn blocks(blocks_dir: &Path, ir: &Value) -> Result<Vec<Block>, String> {
    let out = core_blocks(blocks_dir, ir)?;
    every_domain_placed(&out, ir)?;
    Ok(out)
}

fn every_domain_placed(blocks: &[Block], ir: &Value) -> Result<(), String> {
    if let Some(domains) = ir["domains"].as_object() {
        for d in domains.keys() {
            if !blocks.iter().any(|b| &b.domain == d) {
                return Err(format!("{d} has no file in a layer or pack directory"));
            }
        }
    }
    Ok(())
}

/// One pack: `packs/<pack>/pack.yaml` and its blocks.
#[derive(Debug, Clone)]
pub struct Pack {
    /// `source-control`
    pub name: String,
    /// `agentic.scm`
    pub namespace: String,
    pub summary: String,
    /// The packs it uses.
    pub uses: Vec<String>,
    /// `packs/source-control/scenarios/...yaml`
    pub scenarios: Vec<String>,
    pub blocks: Vec<Block>,
}

/// One field: `fields/<field>/field.yaml`.
#[derive(Debug, Clone)]
pub struct Field {
    /// `coding`
    pub name: String,
    pub summary: String,
    /// The packs it uses.
    pub packs: Vec<String>,
    /// `fields/coding/architectures/auto-merger.yaml`
    pub architectures: Vec<String>,
    /// `fields/coding/profiles/coding.yaml`
    pub profiles: Vec<String>,
    /// `fields/coding/kpis.yaml`, when the field has KPIs.
    pub kpis: Option<String>,
    /// `fields/coding/rows.yaml`, when the field registers rows of core catalog entities.
    pub rows: Option<String>,
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

pub(crate) fn read_yaml(root: &Path, rel: &str) -> Result<Value, String> {
    let text = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    serde_yaml_ng::from_str(&text).map_err(|e| format!("{rel}: {e}"))
}

fn subdirs(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

/// The whole system, from the root `ess-inputs.yaml` and its compiled IR: the core blocks and
/// every pack, each pack with its blocks ordered by name. Fails when a domain of the IR has no
/// file in a layer or pack directory.
pub fn system(root: &Path, ir: &Value) -> Result<(Vec<Block>, Vec<Pack>), String> {
    let core = core_blocks(&root.join("blocks"), ir)?;
    let mut packs = Vec::new();
    for name in subdirs(&root.join("packs")) {
        let rel = format!("packs/{name}/pack.yaml");
        if !root.join(&rel).exists() {
            continue;
        }
        let doc = read_yaml(root, &rel)?;
        let mut blocks = Vec::new();
        for file in strings(&doc["blocks"]) {
            let path = format!("packs/{name}/{file}");
            let text =
                std::fs::read_to_string(root.join(&path)).map_err(|e| format!("{path}: {e}"))?;
            let Some(domain) = text
                .lines()
                .find_map(|l| l.strip_prefix("domain:"))
                .map(|d| d.trim().to_string())
            else {
                continue;
            };
            let d = &ir["domains"][&domain];
            if d.is_null() {
                return Err(format!("{path}: {domain} is not in the compiled system"));
            }
            let short_name = domain
                .strip_prefix("agentic.")
                .unwrap_or(&domain)
                .to_string();
            blocks.push(Block {
                summary: d["naming"]["summary"].as_str().unwrap_or("").to_string(),
                family: "other",
                page: short(&domain).to_string(),
                name: short_name,
                domain,
                layer: PACK_LAYER,
                dir: name.clone(),
                section: "packs",
                file: path,
            });
        }
        blocks.sort_by(|a, b| a.name.cmp(&b.name));
        packs.push(Pack {
            namespace: doc["namespace"].as_str().unwrap_or("").to_string(),
            summary: doc["summary"].as_str().unwrap_or("").trim().to_string(),
            uses: strings(&doc["uses"]),
            scenarios: strings(&doc["scenarios"])
                .into_iter()
                .map(|s| format!("packs/{name}/{s}"))
                .collect(),
            blocks,
            name,
        });
    }
    let all: Vec<Block> = core
        .iter()
        .chain(packs.iter().flat_map(|p| p.blocks.iter()))
        .cloned()
        .collect();
    every_domain_placed(&all, ir)?;
    Ok((core, packs))
}

/// Every field under `fields/`, by name.
pub fn fields(root: &Path) -> Result<Vec<Field>, String> {
    let mut out = Vec::new();
    for name in subdirs(&root.join("fields")) {
        let rel = format!("fields/{name}/field.yaml");
        if !root.join(&rel).exists() {
            continue;
        }
        let doc = read_yaml(root, &rel)?;
        out.push(Field {
            summary: doc["summary"].as_str().unwrap_or("").trim().to_string(),
            packs: strings(&doc["packs"]),
            architectures: strings(&doc["architectures"])
                .into_iter()
                .map(|a| format!("fields/{name}/{a}"))
                .collect(),
            profiles: strings(&doc["profiles"])
                .into_iter()
                .map(|a| format!("fields/{name}/{a}"))
                .collect(),
            kpis: doc["kpis"].as_str().map(|f| format!("fields/{name}/{f}")),
            rows: doc["rows"].as_str().map(|f| format!("fields/{name}/{f}")),
            name,
        });
    }
    Ok(out)
}

fn core_blocks(blocks_dir: &Path, ir: &Value) -> Result<Vec<Block>, String> {
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
            page: name.clone(),
            name,
            domain,
            layer,
            dir: dir.to_string(),
            section: "blocks",
            file: format!("blocks/{file}"),
        });
    }
    out.sort_by(|a, b| (a.layer, &a.name).cmp(&(b.layer, &b.name)));
    Ok(out)
}

/// The two-segment prefix of a qualified name: `agentic.question.Answer` is `agentic.question`.
pub fn prefix(name: &str) -> String {
    name.split('.').take(2).collect::<Vec<_>>().join(".")
}

/// The domain that declares each member (type, entity, command, event, error, view) of the IR. A
/// three-segment domain such as `agentic.scm.change` is not the two-segment prefix of its members.
pub fn declared_in(ir: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (d, v) in ir["domains"].as_object().into_iter().flatten() {
        for kind in ["types", "entities", "commands", "events", "errors", "views"] {
            for m in v[kind]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                out.insert(m.to_string(), d.clone());
            }
        }
    }
    out
}

/// The domain of entity `name`, or its two-segment prefix when the IR does not hold it.
pub fn entity_domain(ir: &Value, name: &str) -> String {
    ir["entities"][name]["domain"]
        .as_str()
        .map_or_else(|| prefix(name), str::to_string)
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
    let declared_in = declared_in(ir);
    let owner = |t: &str| {
        identity
            .get(t)
            .or_else(|| declared_in.get(t))
            .cloned()
            .unwrap_or_else(|| prefix(t))
    };
    let mut raw: BTreeSet<(String, String, String)> = BTreeSet::new();
    for c in ["entities", "commands", "events", "views", "errors", "types"] {
        for (k, v) in ir[c].as_object().into_iter().flatten() {
            let from = v["domain"]
                .as_str()
                .map(str::to_string)
                .or_else(|| declared_in.get(k).cloned())
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
            let to = entity_domain(ir, r["target"].as_str().unwrap_or(""));
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
