//! The agentic ladder (`agenticspec-ladder/1`): stages measured by throughput, the barriers
//! between them, and one architecture per rung. The check holds the ladder to its rule (more
//! throughput needs more trust, or the rung names the risk); the build writes the data file of
//! the ladder page (`agenticspec-ladder-data/1`) with what each rung adds and removes, a stable
//! layout and, when asked, a short simulation of each rung.

use crate::check;
use crate::doc::{load_yaml, num, number};
use crate::profile::Profile;
use crate::rng::Rng;
use crate::sim::{Model, simulate};
use rayon::prelude::*;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "agenticspec-ladder/1";
pub const DATA_FORMAT: &str = "agenticspec-ladder-data/1";

/// Rows of the layout grid: what wakes or feeds the agents above, the agents in the middle, the
/// trust parts below.
const UPPER: (i64, i64) = (0, 0);
const WORK: (i64, i64) = (1, 2);
const TRUST: (i64, i64) = (3, 4);

/// The ladder document and the directory its paths are relative to.
pub fn load(path: &Path) -> Result<(Value, PathBuf), String> {
    let doc = load_yaml(path)?;
    let dir = path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf();
    Ok((doc, dir))
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().map(str::trim).unwrap_or("")
}

fn list(v: &Value) -> &[Value] {
    v.as_array().map_or(&[], Vec::as_slice)
}

/// One rung as the check walked it.
struct Rung<'a> {
    id: String,
    doc: &'a Value,
    stage: Option<usize>,
    arch: Option<Value>,
    /// The trust parts in force at this rung, with what each guards.
    trust: BTreeMap<String, String>,
}

fn parts(arch: &Value) -> BTreeMap<String, Value> {
    arch["parts"]
        .as_object()
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}

fn link_key(l: &Value) -> String {
    format!(
        "{}>{}:{}",
        text(l, "from"),
        text(l, "to"),
        text(l, "carries")
    )
}

fn link_keys(arch: &Value) -> BTreeSet<String> {
    list(&arch["links"]).iter().map(link_key).collect()
}

fn count_of(p: &Value) -> i64 {
    num(check::value(&p["count"])).map_or(1, |c| c.round().max(0.0) as i64)
}

/// Agent copies that do work: every agent part's count, reviewers left out.
fn capacity(arch: &Value) -> i64 {
    parts(arch)
        .iter()
        .filter(|(n, p)| p["block"] == "agent" && !check::is_reviewer(arch, n))
        .map(|(_, p)| count_of(p))
        .sum()
}

/// Walks the rungs in order: loads and checks each architecture and carries the trust parts.
fn walk<'a>(doc: &'a Value, dir: &Path, stages: &[&str], out: &mut Vec<String>) -> Vec<Rung<'a>> {
    let mut rungs: Vec<Rung<'a>> = Vec::new();
    let mut ids = BTreeSet::new();
    let mut blocks: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut last_stage = 0;
    for (i, r) in list(&doc["rungs"]).iter().enumerate() {
        let id = match text(r, "id") {
            "" => format!("#{}", i + 1),
            x => x.to_string(),
        };
        if !ids.insert(id.clone()) {
            out.push(format!("duplicate_rung: rung `{id}` appears twice"));
        }
        for key in ["title", "changed", "architecture"] {
            if text(r, key).is_empty() {
                out.push(format!("rung_fields: rung `{id}` has no `{key}`"));
            }
        }
        let stage = stages.iter().position(|s| *s == text(r, "stage"));
        match stage {
            None => out.push(format!(
                "rung_unknown_stage: rung `{id}` names stage `{}`",
                text(r, "stage")
            )),
            Some(s) if s < last_stage => out.push(format!(
                "rung_out_of_order: rung `{id}` is on stage `{}` after a later stage",
                stages[s]
            )),
            Some(s) => last_stage = s,
        }
        let arch = match text(r, "architecture") {
            "" => None,
            file => match load_yaml(&dir.join(file)) {
                Ok(a) => Some(a),
                Err(e) => {
                    out.push(format!("architecture_missing: rung `{id}`: {e}"));
                    None
                }
            },
        };
        if let Some(a) = &arch {
            for refusal in check::check(a) {
                out.push(format!("architecture_refused: rung `{id}`: {refusal}"));
            }
            for (name, p) in parts(a) {
                let block = text(&p, "block").to_string();
                match blocks.get(&name) {
                    Some((b, first)) if *b != block => out.push(format!(
                        "part_block_changed: part `{name}` is a {b} at rung `{first}` and a {block} at rung `{id}`"
                    )),
                    Some(_) => {}
                    None => {
                        blocks.insert(name, (block, id.clone()));
                    }
                }
            }
        }
        let present = arch.as_ref().map(parts).unwrap_or_default();
        let mut trust: BTreeMap<String, String> = rungs
            .last()
            .map(|p| p.trust.clone())
            .unwrap_or_default()
            .into_iter()
            .filter(|(k, _)| present.contains_key(k))
            .collect();
        for (part, what) in r["trust"].as_object().into_iter().flatten() {
            match what.as_str() {
                None => {
                    trust.remove(part);
                }
                Some(w) => {
                    if arch.is_some() && !present.contains_key(part) {
                        out.push(format!(
                            "trust_without_part: rung `{id}` names trust part `{part}`, which its architecture does not have"
                        ));
                    } else {
                        trust.insert(part.clone(), w.trim().to_string());
                    }
                }
            }
        }
        if arch.is_some() && trust.is_empty() {
            out.push(format!(
                "rung_without_trust: rung `{id}` has no trust part in force"
            ));
        }
        rungs.push(Rung {
            id,
            doc: r,
            stage,
            arch,
            trust,
        });
    }
    rungs
}

/// The refusals of a ladder, each starting with its rule; empty when the ladder is accepted.
pub fn check(doc: &Value, dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if doc["format"].as_str() != Some(FORMAT) {
        out.push(format!(
            "format: `{}`, expected {FORMAT}",
            doc["format"].as_str().unwrap_or("")
        ));
    }
    let mut stages: Vec<&str> = Vec::new();
    for (i, s) in list(&doc["stages"]).iter().enumerate() {
        for key in ["id", "name", "marker", "trust_need", "unit_of_work"] {
            if text(s, key).is_empty() {
                out.push(format!("stage_fields: stage {} has no `{key}`", i + 1));
            }
        }
        let id = text(s, "id");
        if stages.contains(&id) {
            out.push(format!("duplicate_stage: stage `{id}` appears twice"));
        }
        stages.push(id);
    }
    if stages.is_empty() {
        out.push("stage_fields: the ladder has no stages".into());
    }
    for b in list(&doc["barriers"]) {
        let id = text(b, "id");
        for key in ["id", "name", "decision"] {
            if text(b, key).is_empty() {
                out.push(format!("barrier_fields: barrier `{id}` has no `{key}`"));
            }
        }
        let between: Vec<&str> = list(&b["between"])
            .iter()
            .filter_map(Value::as_str)
            .collect();
        let at: Vec<Option<usize>> = between
            .iter()
            .map(|s| stages.iter().position(|x| x == s))
            .collect();
        match at.as_slice() {
            [Some(a), Some(z)] if *z == a + 1 => {}
            [Some(_), Some(_)] => out.push(format!(
                "barrier_not_adjacent: barrier `{id}` sits between `{}` and `{}`, which are not neighbours in that order",
                between[0], between[1]
            )),
            _ => out.push(format!(
                "barrier_unknown_stage: barrier `{id}` names `between: {between:?}`; it needs two stages of the ladder"
            )),
        }
    }
    let rungs = walk(doc, dir, &stages, &mut out);
    if rungs.is_empty() {
        out.push("rung_fields: the ladder has no rungs".into());
    }
    for (i, s) in stages.iter().enumerate() {
        if !rungs.iter().any(|r| r.stage == Some(i)) {
            out.push(format!("stage_without_rung: stage `{s}` has no rung"));
        }
    }
    for pair in rungs.windows(2) {
        let (prev, cur) = (&pair[0], &pair[1]);
        let (Some(pa), Some(ca)) = (&prev.arch, &cur.arch) else {
            continue;
        };
        let more = capacity(ca) > capacity(pa);
        let dropped = prev.trust.keys().any(|k| !cur.trust.contains_key(k));
        let added = cur.trust.keys().any(|k| !prev.trust.contains_key(k));
        if (more || dropped) && !added && text(cur.doc, "risk").is_empty() {
            let what = if more {
                format!(
                    "raises the agent copies at work from {} to {}",
                    capacity(pa),
                    capacity(ca)
                )
            } else {
                "drops a trust part".to_string()
            };
            out.push(format!(
                "throughput_without_trust: rung `{}` {what} and adds no trust part; add one or name the risk",
                cur.id
            ));
        }
    }
    out
}

/// Colour family of a block, as the architecture pages draw it.
pub fn family(block: &str) -> &'static str {
    match block {
        "principal" | "delegation" => "principal",
        "agent" | "harness" => "agent",
        "channel" | "question" | "trigger" | "schedule" => "collab",
        "work" | "process" | "authority" => "work",
        "gateway" | "model" => "intel",
        "tool" | "knowledge" => "capability",
        "workspace" | "sandbox" | "runtime" => "place",
        "pool" | "resource" => "resource",
        "experiment" | "kpi" | "run" => "evaluation",
        _ => "other",
    }
}

/// A short line under a part's name: what kind of thing it is.
fn label(arch: &Value, name: &str, p: &Value) -> String {
    let kind = text(p, "kind").replace('_', " ");
    let block = text(p, "block");
    match block {
        "agent" if check::is_reviewer(arch, name) => "reviewer agent".into(),
        "agent" if !p["acts_for"].is_null() => format!("acts for {}", text(p, "acts_for")),
        "agent" if !p["method"].is_null() => format!("method {}", text(p, "method").to_uppercase()),
        "agent" if p["harness"] == "orchestrator" => "orchestrator".into(),
        "agent" => "agent".into(),
        "principal" => kind,
        "schedule" => format!("every {}", text(p, "every")),
        "trigger" => format!("trigger {}", kind.trim_start_matches("on ")),
        "channel" => format!("{} channel", text(p, "distribution")),
        "sandbox" => format!("sandbox, {}", text(p, "isolation")),
        "kpi" => "KPIs".into(),
        _ if !kind.is_empty() => kind,
        _ => block.to_string(),
    }
}

/// Parts a part is drawn near: its links and the parts it names.
fn neighbours(arch: &Value, name: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for l in list(&arch["links"]) {
        let (from, to) = (text(l, "from"), text(l, "to"));
        if from == name {
            out.insert(to.to_string());
        }
        if to == name {
            out.insert(from.to_string());
        }
    }
    let p = &arch["parts"][name];
    for key in ["per", "acts_for", "egress", "schedule", "channel"] {
        match &p[key] {
            Value::String(s) => {
                out.insert(s.clone());
            }
            Value::Array(xs) => out.extend(xs.iter().filter_map(Value::as_str).map(String::from)),
            _ => {}
        }
    }
    for (other, q) in parts(arch) {
        for key in ["per", "acts_for", "egress", "schedule", "channel"] {
            let named = match &q[key] {
                Value::String(s) => s == name,
                Value::Array(xs) => xs.iter().any(|x| x == name),
                _ => false,
            };
            if named {
                out.insert(other.clone());
            }
        }
    }
    out.remove(name);
    out
}

/// A grid position for every part, given once when the part first appears and kept after.
fn layout(rungs: &[Rung]) -> BTreeMap<String, (i64, i64)> {
    let trust_ever: BTreeSet<&String> = rungs.iter().flat_map(|r| r.trust.keys()).collect();
    let mut at: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    let mut taken: BTreeSet<(i64, i64)> = BTreeSet::new();
    for r in rungs {
        let Some(arch) = &r.arch else { continue };
        let mut new: Vec<String> = parts(arch)
            .keys()
            .filter(|k| !at.contains_key(*k))
            .cloned()
            .collect();
        if at.is_empty() {
            let first = parts(arch)
                .iter()
                .find(|(_, p)| p["block"] == "principal")
                .map(|(n, _)| n.clone())
                .or_else(|| new.first().cloned());
            if let Some(f) = first {
                at.insert(f.clone(), (0, WORK.0));
                taken.insert((0, WORK.0));
                new.retain(|n| *n != f);
            }
        }
        while !new.is_empty() {
            let near: Vec<(usize, &String)> = new
                .iter()
                .map(|n| {
                    let placed = neighbours(arch, n)
                        .iter()
                        .filter(|m| at.contains_key(*m))
                        .count();
                    (placed, n)
                })
                .collect();
            let (_, pick) = near
                .iter()
                .max_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(a.1)))
                .copied()
                .expect("a part is left");
            let pick = pick.clone();
            let p = &arch["parts"][&pick];
            let band = if trust_ever.contains(&pick) {
                TRUST
            } else if p["block"] == "agent" || p["block"] == "principal" {
                WORK
            } else {
                UPPER
            };
            let placed_near: Vec<(i64, i64)> = neighbours(arch, &pick)
                .iter()
                .filter_map(|m| at.get(m).copied())
                .collect();
            let max_col = at.values().map(|c| c.0).max().unwrap_or(0);
            let mut best: Option<((i64, i64), f64)> = None;
            for col in 0..=max_col + 1 {
                for row in band.0..=band.1 {
                    if taken.contains(&(col, row)) {
                        continue;
                    }
                    let pull: f64 = placed_near
                        .iter()
                        .map(|(c, r)| (col - c).abs() as f64 + 0.8 * (row - r).abs() as f64)
                        .sum();
                    let wider = if col > max_col { 0.6 } else { 0.0 };
                    let cost = pull + 0.3 * col as f64 + wider + 0.01 * (row - band.0) as f64;
                    if best.is_none_or(|(_, b)| cost < b - 1e-9) {
                        best = Some(((col, row), cost));
                    }
                }
            }
            let (cell, _) = best.expect("a free cell");
            taken.insert(cell);
            at.insert(pick.clone(), cell);
            new.retain(|n| *n != pick);
        }
    }
    at
}

fn round(x: f64, places: i32) -> Value {
    let f = 10f64.powi(places);
    number((x * f).round() / f)
}

/// Mean KPIs of each rung over the seeds, with the profile's load replaced by a load per worker.
fn simulate_rungs(sim: &Value, dir: &Path, rungs: &[Rung]) -> Result<Vec<Value>, String> {
    let mut profile = load_yaml(&dir.join(text(sim, "profile")))?;
    let rate = sim["tasks_per_hour_per_worker"]
        .as_f64()
        .ok_or("simulation: needs `tasks_per_hour_per_worker`")?;
    profile["load"] = json!({ "tasks_per_hour_per_worker": rate });
    let operator = profile["agents"]["operator"].clone();
    for r in rungs {
        for (name, p) in r.arch.as_ref().map(parts).unwrap_or_default() {
            if p["block"] == "agent"
                && !p["acts_for"].is_null()
                && profile["agents"][&name].is_null()
            {
                profile["agents"][&name] = operator.clone();
            }
        }
    }
    let profile = Profile::parse(&profile)?;
    let horizon = num(&sim["horizon"])
        .filter(|h| *h > 0.0)
        .ok_or("simulation: horizon must be a positive duration")?;
    let seeds = sim["seeds"].as_u64().unwrap_or(1).max(1);
    let seed = sim["seed"].as_u64().unwrap_or(0);
    let models = rungs
        .iter()
        .map(|r| {
            let arch = r
                .arch
                .as_ref()
                .ok_or("simulation: a rung has no architecture")?;
            Model::build(arch, &profile, horizon, &[], &[])
                .map_err(|e| format!("simulation: rung `{}`: {e}", r.id))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let jobs: Vec<(usize, u64)> = (0..models.len())
        .flat_map(|m| (0..seeds).map(move |s| (m, s)))
        .collect();
    let runs: Vec<BTreeMap<String, f64>> = jobs
        .par_iter()
        .map(|(m, s)| simulate(&models[*m], Rng::stream(seed, *s).next_u64(), false).kpis)
        .collect();
    let day = 86400.0 / horizon;
    Ok((0..models.len())
        .map(|m| {
            let mean = |k: &str| {
                runs[m * seeds as usize..(m + 1) * seeds as usize]
                    .iter()
                    .map(|r| r.get(k).copied().unwrap_or(0.0))
                    .sum::<f64>()
                    / seeds as f64
            };
            json!({
                "tasks_done": round(mean("tasks_done") * day, 1),
                "tasks_correct": round(mean("tasks_correct") * day, 1),
                "false_completions_shipped": round(mean("false_completions_shipped") * day, 1),
                "owner_wait_share": round(mean("owner_wait_share"), 3),
                "cost_usd": round(mean("cost_usd") * day, 0),
            })
        })
        .collect())
}

fn strings(xs: impl IntoIterator<Item = String>) -> Value {
    Value::Array(xs.into_iter().map(Value::String).collect())
}

/// The data file of the ladder page. Fails with the refusals when the check refuses the ladder.
pub fn build(doc: &Value, dir: &Path) -> Result<Value, String> {
    let refusals = check(doc, dir);
    if !refusals.is_empty() {
        return Err(refusals.join("\n"));
    }
    let stage_ids: Vec<&str> = list(&doc["stages"]).iter().map(|s| text(s, "id")).collect();
    let mut sink = Vec::new();
    let rungs = walk(doc, dir, &stage_ids, &mut sink);
    let at = layout(&rungs);
    let sims = match &doc["simulation"] {
        Value::Null => None,
        sim => Some(simulate_rungs(sim, dir, &rungs)?),
    };

    let stages: Vec<Value> = list(&doc["stages"])
        .iter()
        .enumerate()
        .map(|(i, s)| {
            json!({
                "id": text(s, "id"),
                "name": text(s, "name"),
                "marker": text(s, "marker"),
                "trust_need": text(s, "trust_need"),
                "unit_of_work": text(s, "unit_of_work"),
                "rungs": rungs.iter().enumerate().filter(|(_, r)| r.stage == Some(i)).map(|(n, _)| n).collect::<Vec<_>>(),
            })
        })
        .collect();
    let barriers: Vec<Value> = list(&doc["barriers"])
        .iter()
        .map(|b| {
            let far = stage_ids
                .iter()
                .position(|s| b["between"][1] == *s)
                .expect("checked");
            json!({
                "id": text(b, "id"),
                "name": text(b, "name"),
                "between": b["between"].clone(),
                "decision": text(b, "decision"),
                "before_rung": rungs.iter().position(|r| r.stage == Some(far)).expect("checked"),
            })
        })
        .collect();

    let empty = Value::Null;
    let mut out_rungs = Vec::new();
    for (i, r) in rungs.iter().enumerate() {
        let arch = r.arch.as_ref().expect("checked");
        let prev = i.checked_sub(1).map(|p| &rungs[p]);
        let prev_arch = prev.and_then(|p| p.arch.as_ref()).unwrap_or(&empty);
        let (now, before) = (parts(arch), parts(prev_arch));
        let (links_now, links_before) = (link_keys(arch), link_keys(prev_arch));
        let prev_trust = prev.map(|p| p.trust.clone()).unwrap_or_default();
        let part_rows: Vec<Value> = now
            .iter()
            .map(|(name, p)| {
                let block = text(p, "block");
                json!({
                    "id": name,
                    "block": block,
                    "family": family(block),
                    "label": label(arch, name, p),
                    "count": count_of(p),
                    "trust": r.trust.get(name),
                })
            })
            .collect();
        let cells: Vec<(i64, i64)> = now.keys().filter_map(|k| at.get(k).copied()).collect();
        let view = json!({
            "col0": cells.iter().map(|c| c.0).min().unwrap_or(0),
            "col1": cells.iter().map(|c| c.0).max().unwrap_or(0),
            "row0": cells.iter().map(|c| c.1).min().unwrap_or(0),
            "row1": cells.iter().map(|c| c.1).max().unwrap_or(0),
        });
        let mut row = json!({
            "id": r.id,
            "n": i + 1,
            "stage": text(r.doc, "stage"),
            "title": text(r.doc, "title"),
            "changed": text(r.doc, "changed"),
            "risk": match text(r.doc, "risk") { "" => Value::Null, x => Value::from(x) },
            "architecture": text(r.doc, "architecture"),
            "name": text(arch, "architecture"),
            "summary": text(arch, "summary"),
            "parts": part_rows,
            "links": list(&arch["links"]).iter().map(|l| json!({
                "from": text(l, "from"), "to": text(l, "to"), "carries": text(l, "carries"),
            })).collect::<Vec<_>>(),
            "added_parts": strings(now.keys().filter(|k| !before.contains_key(*k)).cloned()),
            "removed_parts": strings(before.keys().filter(|k| !now.contains_key(*k)).cloned()),
            "changed_parts": strings(now.iter().filter(|(k, v)| before.get(*k).is_some_and(|b| b != *v)).map(|(k, _)| k.clone())),
            "added_links": strings(links_now.difference(&links_before).cloned()),
            "removed_links": strings(links_before.difference(&links_now).cloned()),
            "trust_added": strings(r.trust.keys().filter(|k| !prev_trust.contains_key(*k)).cloned()),
            "agents_at_work": capacity(arch),
            "trust_parts": r.trust.len(),
            "view": view,
        });
        if let Some(s) = &sims {
            row["simulation"] = s[i].clone();
        }
        out_rungs.push(row);
    }

    let mut layout_parts = Map::new();
    for (name, (col, row)) in &at {
        layout_parts.insert(name.clone(), json!({ "col": col, "row": row }));
    }
    let mut data = json!({
        "format": DATA_FORMAT,
        "ladder": text(doc, "ladder"),
        "title": text(doc, "title"),
        "summary": text(doc, "summary"),
        "rule": text(doc, "rule"),
        "stages": stages,
        "barriers": barriers,
        "rungs": out_rungs,
        "layout": {
            "cols": at.values().map(|c| c.0).max().unwrap_or(0) + 1,
            "rows": TRUST.1 + 1,
            "bands": [
                { "name": "wakes and feeds", "row0": UPPER.0, "row1": UPPER.1 },
                { "name": "works", "row0": WORK.0, "row1": WORK.1 },
                { "name": "trust", "row0": TRUST.0, "row1": TRUST.1 },
            ],
            "parts": layout_parts,
        },
    });
    if sims.is_some() {
        let s = &doc["simulation"];
        data["simulation"] = json!({
            "profile": text(s, "profile"),
            "horizon": s["horizon"].clone(),
            "seeds": s["seeds"].clone(),
            "seed": s["seed"].clone(),
            "tasks_per_hour_per_worker": s["tasks_per_hour_per_worker"].clone(),
        });
    }
    Ok(data)
}

/// The data file as text: pretty JSON with a final newline.
pub fn render(path: &Path) -> Result<String, String> {
    let (doc, dir) = load(path)?;
    let data = build(&doc, &dir)?;
    let mut text = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text)
}
