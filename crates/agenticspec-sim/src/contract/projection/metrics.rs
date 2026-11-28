//! The metric list of a `metrics_scrape` export: per KPI of the interface its catalogue `id` as
//! the metric's name (a runtime binding may set a prefix), its `summary` as help and its `unit`,
//! every KPI a gauge (a KPI is a value over a window, not a running total), each sample labelled
//! with the placement: architecture, part path and instance. It reads the KPI catalogue only.

use super::{Contract, ContractIngress, OpKind, header_of, ingress_text};
use crate::contract::ingress::Kpis;
use serde_json::{Value, json};

/// The format of a metric list.
pub const FORMAT: &str = "agenticspec-metrics/1";

/// The labels of every sample: the placement of the part it was measured on.
pub const LABELS: [&str; 3] = ["architecture", "part_path", "instance"];

/// The metric list of an ingress's `metrics_scrape` export. Refused, naming every one, when the
/// catalogue lacks a KPI of the interface or gives it no unit or no summary.
pub fn metrics(
    contract: &Contract,
    ingress: &ContractIngress,
    kpis: &Kpis,
) -> Result<String, String> {
    let mut items: Vec<Vec<(&str, Value)>> = Vec::new();
    let mut refused = Vec::new();
    for o in ingress
        .operations
        .iter()
        .filter(|o| o.kind == OpKind::Metric)
    {
        match kpis.0.get(&o.name) {
            Some(k) if !k.unit.is_empty() && !k.summary.is_empty() => items.push(vec![
                ("name", json!(o.name)),
                ("type", json!("gauge")),
                ("help", json!(k.summary)),
                ("unit", json!(k.unit)),
                ("block", json!(k.block)),
                ("status", json!(k.status)),
            ]),
            Some(_) => refused.push(format!("{} (no unit or no summary)", o.name)),
            None => refused.push(format!("{} (not in the catalogue)", o.name)),
        }
    }
    if !refused.is_empty() {
        return Err(format!(
            "ingress {}: no metric for KPI {}",
            ingress.name,
            refused.join(", ")
        ));
    }
    let mut s = header_of(contract, ingress, "metrics_scrape", "The metric list");
    s.push_str(
        "# Every KPI a gauge named by its catalogue id; a runtime binding may set a prefix.\n",
    );
    s.push_str(&format!("format: {FORMAT}\n"));
    s.push_str(&ingress_text(contract, ingress));
    s.push_str(&super::field("labels", &json!(LABELS)));
    s.push_str(&super::list("metrics", &items));
    Ok(s)
}
