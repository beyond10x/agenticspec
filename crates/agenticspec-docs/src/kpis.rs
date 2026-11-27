//! The KPI catalogue, `kpis/catalogue.yaml` (format `agenticspec-kpis/1`).

use serde_json::Value;
use std::collections::BTreeSet;

pub const KINDS: [&str; 3] = ["correctness", "completeness", "quality"];

#[derive(Debug, Clone, PartialEq)]
pub struct Kpi {
    pub id: String,
    pub block: String,
    pub kind: String,
    pub unit: String,
    pub summary: String,
    pub how: String,
    pub direction: String,
    /// Work mode and its direction there, in the order written.
    pub modes: Vec<(String, String)>,
    pub gate: bool,
    pub reads: Vec<String>,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalogue {
    pub kpis: Vec<Kpi>,
}

fn field(v: &Value, key: &str, id: &str) -> Result<String, String> {
    match &v[key] {
        Value::String(s) => Ok(s.trim().to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Err(format!("KPI `{id}`: needs `{key}`")),
        _ => Err(format!("KPI `{id}`: `{key}` is not text")),
    }
}

fn one_of(value: String, allowed: &[&str], key: &str, id: &str) -> Result<String, String> {
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(format!(
            "KPI `{id}`: {key} `{value}` is not one of {}",
            allowed.join(", ")
        ))
    }
}

impl Catalogue {
    pub fn parse(text: &str) -> Result<Self, String> {
        let doc: Value = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
        match doc["format"].as_str() {
            Some("agenticspec-kpis/1") => {}
            other => return Err(format!("format {other:?} is not agenticspec-kpis/1")),
        }
        let mut kpis = Vec::new();
        let mut seen = BTreeSet::new();
        for v in doc["kpis"].as_array().into_iter().flatten() {
            let id = v["id"].as_str().ok_or("a KPI without `id`")?.to_string();
            if !seen.insert(id.clone()) {
                return Err(format!("KPI `{id}` is listed twice"));
            }
            let modes = v["modes"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, d)| (k.clone(), d.as_str().unwrap_or("").to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let reads = v["reads"]
                .as_array()
                .map(|r| {
                    r.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            kpis.push(Kpi {
                block: field(v, "block", &id)?,
                kind: one_of(field(v, "kind", &id)?, &KINDS, "kind", &id)?,
                unit: field(v, "unit", &id)?,
                summary: field(v, "summary", &id)?,
                how: field(v, "how", &id).unwrap_or_default(),
                direction: field(v, "direction", &id).unwrap_or_default(),
                modes,
                gate: v["gate"].as_bool().unwrap_or(false),
                reads,
                status: one_of(
                    field(v, "status", &id)?,
                    &["computed", "simulated", "both", "planned"],
                    "status",
                    &id,
                )?,
                source: field(v, "source", &id).unwrap_or_default(),
                id,
            });
        }
        Ok(Catalogue { kpis })
    }

    /// The KPIs of one block, in catalogue order.
    pub fn of<'a>(&'a self, block: &'a str) -> impl Iterator<Item = &'a Kpi> + 'a {
        self.kpis.iter().filter(move |k| k.block == block)
    }
}

/// How a status reads on a page.
pub fn status_words(status: &str) -> &'static str {
    match status {
        "computed" => "computed today",
        "simulated" => "simulated",
        "both" => "computed and simulated",
        _ => "planned",
    }
}
