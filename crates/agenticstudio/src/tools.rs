//! The four tools of a studio session, as answers in JSON: `query` runs a selector over the facts
//! of a draft's current revision, `apply` applies a patch (or its dry run), `check` runs the
//! structural check and `simulate` runs the simulator for a few seeds. Each answer carries a
//! `summary`, one or a few lines that explain it: the rows found, the revision a patch made, the
//! rule and error that refused it, each KPI with what it measures.

use std::collections::BTreeMap;
use std::path::{Component, Path};

use agenticspec_patch::{Change, Refused};
use agenticspec_records::sq::architecture;
use agenticspec_records::sq::eval;
use agenticspec_records::sq::syntax;
use agenticspec_sim::check::{self, Refusal};
use agenticspec_sim::profile::Profile;
use agenticspec_sim::rng::Rng;
use agenticspec_sim::sim::{Model, simulate as run_once};
use serde_json::{Map, Value, json};

use crate::routes::{to_json, val};
use crate::studio::Outcome;

/// The profile a simulate runs with when it names none.
pub const DEFAULT_PROFILE: &str = "fields/coding/profiles/coding.yaml";
/// The seeds a simulate runs when it names no budget, and the most it runs.
pub const DEFAULT_BUDGET: u64 = 4;
pub const MAX_BUDGET: u64 = 16;
/// The simulated time of a simulate that names no horizon.
pub const DEFAULT_HORIZON: &str = "24h";
/// The KPI catalogue a simulate explains its KPIs from.
pub const CATALOGUE: &str = "kpis/catalogue.yaml";

/// The check as JSON: `passed` and each refusal with its rule, part and message.
pub fn check_json(refusals: &[Refusal]) -> Value {
    json!({
        "passed": refusals.is_empty(),
        "refusals": refusals
            .iter()
            .map(|r| json!({"rule": r.rule(), "part": r.part(), "message": r.to_string()}))
            .collect::<Vec<_>>(),
    })
}

pub fn diff_json(diff: &[Change]) -> Value {
    Value::from(
        diff.iter()
            .map(|c| json!({"path": c.path, "before": c.before, "after": c.after}))
            .collect::<Vec<_>>(),
    )
}

pub fn refusal_json(r: &Refused) -> Value {
    json!({"rule": r.rule, "error": r.error.name(), "message": r.message})
}

fn check_summary(refusals: &[Refusal]) -> String {
    if refusals.is_empty() {
        return "The check passes.".into();
    }
    let lines: Vec<String> = refusals
        .iter()
        .map(|r| format!("{} on part `{}`: {r}", r.rule(), r.part()))
        .collect();
    format!("The check refuses: {}", lines.join("; "))
}

/// Why a patch was refused, in one line: the rule's position and the error, or the check's
/// rule and the part it names.
pub fn refusal_summary(r: &Refused) -> String {
    let head = match r.rule {
        Some(n) => format!("Refused at rule {n}: {} — {}", r.error.name(), r.message),
        None => format!("Refused: {} — {}", r.error.name(), r.message),
    };
    if r.check.is_empty() {
        return head;
    }
    let lines: Vec<String> = r
        .check
        .iter()
        .map(|c| format!("{} on part `{}`: {c}", c.rule(), c.part()))
        .collect();
    format!("{head}; {}", lines.join("; "))
}

/// The answer of `check` on a document.
pub fn check(doc: &Value) -> Value {
    let refusals = check::check(doc);
    json!({"summary": check_summary(&refusals), "check": check_json(&refusals)})
}

/// The answer of `query`: the selector's columns and one object per row, keyed by column. A
/// selector is a rule body over `part`, `setting`, `link` and `marker`.
pub fn query(doc: &Value, selector: &str) -> Result<Value, String> {
    let selector = selector.trim().trim_end_matches('.');
    if selector.is_empty() {
        return Err("InvalidSelector: the selector is empty".into());
    }
    let module = syntax::parse(&format!("?- {selector}."))
        .map_err(|e| format!("InvalidSelector: the selector does not parse: {e}"))?;
    if module.queries.len() != 1
        || !module.rules.is_empty()
        || !module.checks.is_empty()
        || !module.names.is_empty()
    {
        return Err("InvalidSelector: a selector is one rule body".into());
    }
    let program = eval::compile_in(&module, architecture::SCHEMA)
        .map_err(|e| format!("InvalidSelector: {e}"))?;
    let columns = program.queries[0].columns.clone();
    let answers = eval::run(&program, &agenticspec_patch::fact_base(doc));
    let rows: Vec<Value> = answers.queries[0]
        .rows
        .iter()
        .map(|(vals, _)| {
            let row: Map<String, Value> =
                columns.iter().cloned().zip(vals.iter().map(val)).collect();
            Value::Object(row)
        })
        .collect();
    let n = rows.len();
    let summary = match n {
        0 => "No row matches.".to_string(),
        1 => "1 row.".to_string(),
        n => format!("{n} rows."),
    };
    Ok(json!({"summary": summary, "columns": columns, "rows": rows}))
}

/// The answer of `apply` for a patch that could be read: what it changed or why it was refused.
pub fn apply(outcome: &Outcome) -> Value {
    let n = outcome.changed_rows;
    let rows = if n == 1 { "row" } else { "rows" };
    let (kind, summary) = match (&outcome.refusal, &outcome.revision) {
        (Some(r), _) => ("Refused", refusal_summary(r)),
        (None, Some(rev)) => (
            "Applied",
            format!("Applied as revision {}, {n} {rows} changed.", rev.number),
        ),
        (None, None) => (
            "DryRun",
            format!("Dry run: {n} {rows} would change; the draft keeps its revision."),
        ),
    };
    let check = match &outcome.refusal {
        Some(r) if r.check.is_empty() => Value::Null,
        _ => check_json(&outcome.check),
    };
    json!({
        "outcome": kind,
        "summary": summary,
        "refusal": outcome.refusal.as_ref().map(refusal_json),
        "application": outcome.application.as_ref().map(to_json),
        "patch": outcome.patch.as_ref().map(to_json),
        "diff": diff_json(&outcome.diff),
        "changed_rows": n,
        "check": check,
        "revision": outcome.revision.as_ref().map(to_json),
    })
}

/// The answer of `apply` for a patch that could not be read: refused, with nothing recorded.
pub fn unreadable(r: &Refused) -> Value {
    json!({
        "outcome": "Refused",
        "summary": refusal_summary(r),
        "refusal": refusal_json(r),
        "application": null,
        "patch": null,
        "diff": [],
        "changed_rows": 0,
        "check": null,
        "revision": null,
    })
}

/// What the catalogue says of one KPI.
struct Kpi {
    unit: String,
    direction: String,
    summary: String,
}

fn catalogue(root: &Path) -> BTreeMap<String, Kpi> {
    let Ok(doc) = agenticspec_sim::doc::load_yaml(&root.join(CATALOGUE)) else {
        return BTreeMap::new();
    };
    doc["kpis"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|k| {
            let s = |f: &str| k[f].as_str().unwrap_or("").to_string();
            Some((
                k["id"].as_str()?.to_string(),
                Kpi {
                    unit: s("unit"),
                    direction: s("direction"),
                    summary: s("summary"),
                },
            ))
        })
        .collect()
}

fn round(v: f64) -> Value {
    let r = (v * 10_000.0).round() / 10_000.0;
    serde_json::Number::from_f64(r).map_or(Value::Null, Value::Number)
}

/// A profile path: relative, without `..`, a `.yaml` file under `profiles/` or under a field's
/// `fields/<field>/profiles/`.
fn profile_path_ok(rel: &str) -> bool {
    let p = Path::new(rel);
    p.components().all(|c| matches!(c, Component::Normal(_)))
        && rel.ends_with(".yaml")
        && (p.starts_with("profiles")
            || (p.starts_with("fields")
                && p.components().nth(2) == Some(Component::Normal("profiles".as_ref()))))
}

/// The answer of `simulate`: the draft's current revision at its marked values, run for
/// `budget` seeds (`arguments.budget`, default 4, at most 16) over `arguments.horizon` (default
/// 24h) with the behaviour profile `arguments.profile` (default `fields/coding/profiles/coding.yaml`). It
/// answers the mean of each KPI over the seeds, with what the KPI catalogue says it measures,
/// and the KPIs per part.
pub fn simulate(
    root: &Path,
    revision: usize,
    doc: &Value,
    arguments: &Value,
) -> Result<Value, String> {
    let budget = match &arguments["budget"] {
        Value::Null => DEFAULT_BUDGET,
        v => v
            .as_u64()
            .filter(|b| (1..=MAX_BUDGET).contains(b))
            .ok_or_else(|| {
                format!("InvalidBudget: a budget is a whole number of seeds from 1 to {MAX_BUDGET}")
            })?,
    };
    let horizon_text = arguments["horizon"].as_str().unwrap_or(DEFAULT_HORIZON);
    let horizon = agenticspec_sim::doc::parse_duration(horizon_text)
        .filter(|h| *h > 0.0)
        .ok_or_else(|| format!("InvalidHorizon: `{horizon_text}` is no duration such as 24h"))?;
    let profile_path = arguments["profile"].as_str().unwrap_or(DEFAULT_PROFILE);
    if !profile_path_ok(profile_path) {
        return Err(format!(
            "NoSuchProfile: {profile_path} is no profile under profiles/ or fields/<field>/profiles/"
        ));
    }
    let profile_doc = agenticspec_sim::doc::load_yaml(&root.join(profile_path))
        .map_err(|e| format!("NoSuchProfile: {e}"))?;
    let profile = Profile::parse(&profile_doc).map_err(|e| format!("InvalidProfile: {e}"))?;
    let arch = agenticspec_sim::marked::resolve(doc, &Map::new())
        .map_err(|e| format!("NotSimulable: {e}"))?;
    let model = Model::build(&arch, &profile, horizon, &[], &[])
        .map_err(|e| format!("NotSimulable: {e}"))?;
    let runs: Vec<_> = (0..budget)
        .map(|s| run_once(&model, Rng::stream(1, s).next_u64(), false))
        .collect();
    let n = runs.len() as f64;
    let mut kpis: BTreeMap<String, f64> = BTreeMap::new();
    let mut per_part: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    for r in &runs {
        for (k, v) in &r.kpis {
            *kpis.entry(k.clone()).or_default() += v / n;
        }
        for (p, m) in &r.per_part {
            let row = per_part.entry(p.clone()).or_default();
            for (k, v) in m {
                *row.entry(k.clone()).or_default() += v / n;
            }
        }
    }
    let known = catalogue(root);
    let mut lines = vec![format!(
        "Revision {revision}, {budget} seeds of {horizon_text} with {profile_path}; the mean of each KPI:"
    )];
    let kpi_rows: Vec<Value> = kpis
        .iter()
        .map(|(id, v)| {
            let k = known.get(id);
            let meaning = k.map_or(String::new(), |k| {
                let better = match k.direction.as_str() {
                    "lower" => ", lower is better",
                    "higher" => ", higher is better",
                    _ => "",
                };
                format!(" — {} ({}{better})", k.summary, k.unit)
            });
            lines.push(format!("  {id} = {}{meaning}", round(*v)));
            json!({
                "id": id,
                "value": round(*v),
                "unit": k.map(|k| k.unit.clone()),
                "direction": k.map(|k| k.direction.clone()),
                "summary": k.map(|k| k.summary.clone()),
            })
        })
        .collect();
    let per_part: Map<String, Value> = per_part
        .into_iter()
        .map(|(p, m)| {
            let row: Map<String, Value> = m.into_iter().map(|(k, v)| (k, round(v))).collect();
            (p, Value::Object(row))
        })
        .collect();
    Ok(json!({
        "summary": lines.join("\n"),
        "revision": revision,
        "budget": budget,
        "horizon": horizon_text,
        "profile": profile_path,
        "kpis": kpi_rows,
        "per_part": per_part,
    }))
}
