//! Requirements: the top-level `requirements:` key of an architecture, each a KPI with a target,
//! `- { kpi: cache_hit_share, at_least: 0.7 }`, optionally for one part and over a window
//! (`design/deployment.md` K12). The key is portable and outside the nesting digest, like
//! `ingress:`.
//!
//! A requirement is decided when its KPI can be read, its stage ([`Stage`]): a KPI of the
//! architecture as a whole (`whole.rs`) or a gate count of the check at check; a KPI the simulator
//! measures at evaluation (`sim run`); a KPI the catalogue computes or plans in operation; a cloud
//! KPI, named `cloudspec:<id>`, by cloudspec. A requirement that is not decided where it is read is
//! `undecided`. A missed requirement is reported, never refused: admission asks only for a known
//! KPI and a well-formed target (K14), and the search treats requirements as constraints.

use crate::check::GATE_KPIS;
use crate::contract::ingress::Kpis;
use crate::doc::is_duration;
use crate::sim::RUN_KPIS;
use crate::whole::measure_of;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt;

/// The top-level key of an architecture that holds its requirements.
pub const KEY: &str = "requirements";

/// The prefix of a cloud KPI's id: `cloudspec:<id>`, evaluated by cloudspec.
pub const CLOUD_PREFIX: &str = "cloudspec:";

/// Every rule of the check of `requirements:`.
pub const RULES: &[&str] = &["requirement_invalid", "requirement_kpi_unknown"];

/// The bounds a requirement may set, exactly one of them.
pub const BOUNDS: &[&str] = &["at_least", "at_most", "equals"];

/// The keys of a requirement.
const KEYS: &[&str] = &["kpi", "at_least", "at_most", "equals", "part", "window"];

/// How close a value is to an `equals` target to meet it.
const EQUALS_TOLERANCE: f64 = 1e-9;

/// When a requirement's KPI can be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Read from the document: a KPI of the architecture as a whole or a gate count.
    Check,
    /// Measured by the simulator in `sim run`.
    Evaluation,
    /// Computed from what a running deployment records.
    Operation,
    /// A cloudspec KPI, evaluated by cloudspec.
    Cloud,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Check => "check",
            Stage::Evaluation => "evaluation",
            Stage::Operation => "operation",
            Stage::Cloud => "cloud",
        }
    }
}

/// One requirement as written, its form checked.
#[derive(Clone, Debug, PartialEq)]
pub struct Requirement {
    /// Its index in `requirements:`.
    pub index: usize,
    pub kpi: String,
    /// One of [`BOUNDS`].
    pub bound: &'static str,
    pub target: f64,
    pub part: Option<String>,
    pub window: Option<String>,
    /// The entry as written.
    pub written: Value,
}

impl Requirement {
    /// Whether `value` meets the target.
    pub fn met_by(&self, value: f64) -> bool {
        match self.bound {
            "at_least" => value >= self.target,
            "at_most" => value <= self.target,
            _ => (value - self.target).abs() <= EQUALS_TOLERANCE,
        }
    }

    /// The entry as written with its stage, its state (`met`, `missed` or `undecided`) and the
    /// value it was decided on, when one is known.
    pub fn decided(&self, stage: Option<Stage>, value: Option<f64>) -> Value {
        let mut out = self.written.clone();
        if let Some(s) = stage {
            out["stage"] = json!(s.as_str());
        }
        match value {
            Some(v) => {
                out["state"] = json!(if self.met_by(v) { "met" } else { "missed" });
                out["value"] = crate::doc::number(v);
            }
            None => out["state"] = json!("undecided"),
        }
        out
    }
}

/// One refusal of the check of `requirements:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// One of [`RULES`].
    pub rule: &'static str,
    /// `requirements[<index>]`, or `requirements` for the key itself.
    pub subject: String,
    pub detail: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.rule, self.subject, self.detail)
    }
}

fn invalid(subject: String, detail: String) -> Refusal {
    Refusal {
        rule: "requirement_invalid",
        subject,
        detail,
    }
}

/// Whether the document declares requirements.
pub fn declares(doc: &Value) -> bool {
    !doc[KEY].is_null()
}

/// The requirements of `doc` whose form holds, and a refusal for each that does not. `doc` is
/// read flattened: a `part` names a part of it, a nested one as `<part>.<inner part>`. The form:
/// a list of maps, each with a `kpi`, exactly one bound of [`BOUNDS`] that is a number, an
/// optional `part` the document has and an optional `window` that is a duration (`1h`).
pub fn parse(doc: &Value) -> (Vec<Requirement>, Vec<Refusal>) {
    let mut reqs = Vec::new();
    let mut out = Vec::new();
    let list = match &doc[KEY] {
        Value::Null => return (reqs, out),
        Value::Array(xs) => xs,
        _ => {
            out.push(invalid(
                KEY.into(),
                "is a list of `{kpi, at_least | at_most | equals, part?, window?}`".into(),
            ));
            return (reqs, out);
        }
    };
    for (i, r) in list.iter().enumerate() {
        let subject = format!("{KEY}[{i}]");
        let Some(o) = r.as_object() else {
            out.push(invalid(
                subject,
                "is a map `{kpi, at_least | at_most | equals, part?, window?}`".into(),
            ));
            continue;
        };
        let mut problems = Vec::new();
        for k in o.keys().filter(|k| !KEYS.contains(&k.as_str())) {
            problems.push(format!("unknown key `{k}`"));
        }
        let kpi = o.get("kpi").and_then(Value::as_str).unwrap_or("");
        if kpi.is_empty() {
            problems.push("names no `kpi`".into());
        }
        let bounds: Vec<&'static str> = BOUNDS
            .iter()
            .copied()
            .filter(|b| o.contains_key(*b))
            .collect();
        let target = match bounds.as_slice() {
            [b] => match o[*b].as_f64() {
                Some(t) => Some((*b, t)),
                None => {
                    problems.push(format!("`{b}` is not a number"));
                    None
                }
            },
            [] => {
                problems.push("sets no target: one of `at_least`, `at_most`, `equals`".into());
                None
            }
            _ => {
                problems.push(format!(
                    "sets {} targets; exactly one of `at_least`, `at_most`, `equals`",
                    bounds.len()
                ));
                None
            }
        };
        let part = match o.get("part") {
            None => None,
            Some(Value::String(p)) if doc["parts"].get(p).is_some() => Some(p.clone()),
            Some(Value::String(p)) => {
                problems.push(format!("`part` names `{p}`, which is no part"));
                None
            }
            Some(_) => {
                problems.push("`part` is not a part's name".into());
                None
            }
        };
        let window = match o.get("window") {
            None => None,
            Some(w) if is_duration(w) => w.as_str().map(str::to_owned),
            Some(w) => {
                problems.push(format!("`window` {w} is not a duration such as `1h`"));
                None
            }
        };
        if !problems.is_empty() {
            out.push(invalid(subject, problems.join("; ")));
            continue;
        }
        let Some((bound, target)) = target else {
            continue;
        };
        reqs.push(Requirement {
            index: i,
            kpi: kpi.to_owned(),
            bound,
            target,
            part,
            window,
            written: r.clone(),
        });
    }
    (reqs, out)
}

/// Whether a KPI is read from the document at check: a KPI of the architecture as a whole or a
/// gate count of the check.
pub fn is_design(kpi: &str) -> bool {
    measure_of(kpi).is_some() || GATE_KPIS.contains(&kpi)
}

/// The stage a KPI is decided at, or `None` for an id neither the simulator, the catalogue nor
/// cloudspec's naming knows.
pub fn stage(kpi: &str, kpis: &Kpis) -> Option<Stage> {
    if kpi
        .strip_prefix(CLOUD_PREFIX)
        .is_some_and(|id| !id.is_empty())
    {
        return Some(Stage::Cloud);
    }
    if is_design(kpi) {
        return Some(Stage::Check);
    }
    if RUN_KPIS.contains(&kpi) {
        return Some(Stage::Evaluation);
    }
    match kpis.0.get(kpi).map(|k| k.status.as_str()) {
        Some("simulated" | "both") => Some(Stage::Evaluation),
        Some(_) => Some(Stage::Operation),
        None => None,
    }
}

/// The stage of a KPI as the simulator sees it, without the catalogue: a cloud KPI, a design
/// KPI, one it measures, else one read in operation.
pub fn stage_in_run(kpi: &str) -> Stage {
    if kpi.starts_with(CLOUD_PREFIX) {
        Stage::Cloud
    } else if is_design(kpi) {
        Stage::Check
    } else if RUN_KPIS.contains(&kpi) {
        Stage::Evaluation
    } else {
        Stage::Operation
    }
}

/// Every refusal of `requirements:` of `doc` (read flattened) against the KPI catalogue: its
/// form ([`parse`]), an id no stage knows (`requirement_kpi_unknown`), and a `part` on a KPI of
/// the architecture as a whole, which has no value per part.
pub fn refusals(doc: &Value, kpis: &Kpis) -> Vec<Refusal> {
    let (reqs, mut out) = parse(doc);
    for r in &reqs {
        let subject = format!("{KEY}[{}]", r.index);
        match stage(&r.kpi, kpis) {
            None => out.push(Refusal {
                rule: "requirement_kpi_unknown",
                subject,
                detail: format!(
                    "KPI `{}` is in neither the KPI catalogue nor the simulator, and a cloud KPI is named `{CLOUD_PREFIX}<id>`",
                    r.kpi
                ),
            }),
            Some(Stage::Check) if r.part.is_some() => out.push(invalid(
                subject,
                format!(
                    "`{}` is a KPI of the architecture as a whole and has no value per part; drop `part`",
                    r.kpi
                ),
            )),
            _ => {}
        }
    }
    out.sort_by(|a, b| (a.subject.as_str(), a.rule).cmp(&(b.subject.as_str(), b.rule)));
    out
}

/// The design KPIs of a flattened document by id: the KPIs of the architecture as a whole and
/// the gate counts of the check.
pub fn design_values(flat: &Value) -> BTreeMap<String, f64> {
    let mut out: BTreeMap<String, f64> = crate::check::gate_counts(flat)
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    out.extend(
        crate::whole::measure(flat)
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v)),
    );
    out
}

/// Each requirement of a document the check accepts, as written with its stage, state and value:
/// a design KPI decided on `flat`, every other one `undecided`. The requirements are those of
/// `refusals`'s accepted form; call it first.
pub fn at_check(flat: &Value, kpis: &Kpis) -> Vec<Value> {
    let (reqs, _) = parse(flat);
    let values = design_values(flat);
    reqs.iter()
        .map(|r| {
            let stage = stage(&r.kpi, kpis);
            let value = (stage == Some(Stage::Check))
                .then(|| values.get(&r.kpi).copied())
                .flatten();
            r.decided(stage, value)
        })
        .collect()
}

/// The value of a requirement's KPI in a variant's KPIs, or per part for one with a `part`; `None`
/// when the simulator did not measure it.
pub fn value_in(
    r: &Requirement,
    kpis: &BTreeMap<String, f64>,
    per_part: &BTreeMap<String, BTreeMap<String, f64>>,
) -> Option<f64> {
    match &r.part {
        Some(p) => per_part.get(p).and_then(|m| m.get(&r.kpi)).copied(),
        None => kpis.get(&r.kpi).copied(),
    }
}

/// The requirements of an evaluated variant: each as written with its state and value, how many
/// were decided and how many of those were missed.
pub fn at_evaluation(
    reqs: &[Requirement],
    kpis: &BTreeMap<String, f64>,
    per_part: &BTreeMap<String, BTreeMap<String, f64>>,
) -> (Vec<Value>, usize, usize) {
    let mut decided = 0;
    let mut missed = 0;
    let entries = reqs
        .iter()
        .map(|r| {
            let value = value_in(r, kpis, per_part);
            if let Some(v) = value {
                decided += 1;
                if !r.met_by(v) {
                    missed += 1;
                }
            }
            r.decided(Some(stage_in_run(&r.kpi)), value)
        })
        .collect();
    (entries, decided, missed)
}
