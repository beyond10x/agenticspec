//! Marked places of an architecture: every `{ value, vary }` node, and under `optimize: all` every
//! other numeric value with a default range.

use crate::doc::{format_duration, is_duration, num, number};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Marked {
    /// Where the value sits, keys joined by `.`: `parts.worker.count`.
    pub path: String,
    /// The value as built.
    pub value: Value,
    /// The values the search may take there; it contains `value`.
    pub range: Vec<Value>,
}

impl Marked {
    pub fn baseline_index(&self) -> usize {
        self.range
            .iter()
            .position(|v| same(v, &self.value))
            .unwrap_or(0)
    }
}

fn is_marker(v: &Value) -> bool {
    v.as_object().is_some_and(|o| o.contains_key("vary"))
}

fn same(a: &Value, b: &Value) -> bool {
    match (num(a), num(b)) {
        (Some(x), Some(y)) => (x - y).abs() < 1e-9,
        _ => a == b,
    }
}

fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

fn grid(path: &str, vary: &Map<String, Value>) -> Result<Vec<Value>, String> {
    let get = |k: &str| {
        vary.get(k)
            .and_then(num)
            .ok_or_else(|| format!("{path}: vary needs a numeric `{k}`"))
    };
    let (min, max, step) = (get("min")?, get("max")?, get("step")?);
    if step <= 0.0 || max < min {
        return Err(format!("{path}: vary needs min <= max and a positive step"));
    }
    let durations = vary.get("min").is_some_and(is_duration);
    let mut out = Vec::new();
    let mut i = 0.0;
    loop {
        let x = ((min + i * step) * 1e9).round() / 1e9;
        if x > max + 1e-9 * step {
            break;
        }
        out.push(if durations {
            Value::from(format_duration(x))
        } else {
            number(x)
        });
        i += 1.0;
        if out.len() > 100_000 {
            return Err(format!("{path}: vary has more than 100000 values"));
        }
    }
    Ok(out)
}

fn with_value(mut range: Vec<Value>, value: &Value) -> Vec<Value> {
    if range.iter().any(|v| same(v, value)) {
        return range;
    }
    match num(value) {
        Some(x) if range.iter().all(|v| num(v).is_some()) => {
            let at = range
                .iter()
                .position(|v| num(v).unwrap_or(0.0) > x)
                .unwrap_or(range.len());
            range.insert(at, value.clone());
        }
        _ => range.push(value.clone()),
    }
    range
}

fn default_range(value: &Value) -> Option<Vec<Value>> {
    let x = num(value)?;
    if x == 0.0 || value.is_boolean() {
        return None;
    }
    let factors = [
        0.5,
        std::f64::consts::FRAC_1_SQRT_2,
        1.0,
        std::f64::consts::SQRT_2,
        2.0,
    ];
    let mut out: Vec<Value> = Vec::new();
    for f in factors {
        let v = if is_duration(value) {
            Value::from(format_duration(x * f))
        } else if value.as_i64().is_some() || value.as_u64().is_some() {
            number((x * f).round())
        } else {
            number(((x * f) * 1e6).round() / 1e6)
        };
        if !out.iter().any(|o| same(o, &v)) {
            out.push(v);
        }
    }
    (out.len() > 1).then(|| with_value(out, value))
}

fn walk(v: &Value, path: &str, all: bool, out: &mut Vec<Marked>) -> Result<(), String> {
    if is_marker(v) {
        let value = v
            .get("value")
            .cloned()
            .ok_or_else(|| format!("{path}: vary without value"))?;
        let range = match &v["vary"] {
            Value::Array(xs) if !xs.is_empty() => xs.clone(),
            Value::Object(o) => grid(path, o)?,
            _ => return Err(format!("{path}: vary has no values")),
        };
        out.push(Marked {
            path: path.to_string(),
            range: with_value(range, &value),
            value,
        });
        return Ok(());
    }
    match v {
        Value::Object(o) => {
            // A nested architecture is searched by its own experiment, not by this one.
            let nested = o.get("block").and_then(Value::as_str) == Some(crate::nest::BLOCK);
            for (k, x) in o {
                if nested && k == crate::nest::INNER {
                    continue;
                }
                // A requirement's target is a constraint of the search, never a place of it.
                if path.is_empty() && k == crate::requirements::KEY {
                    continue;
                }
                walk(x, &join(path, k), all, out)?;
            }
        }
        Value::Array(xs) => {
            for (i, x) in xs.iter().enumerate() {
                walk(x, &join(path, &i.to_string()), all, out)?;
            }
        }
        leaf if all && !path.is_empty() => {
            if let Some(range) = default_range(leaf) {
                out.push(Marked {
                    path: path.to_string(),
                    value: leaf.clone(),
                    range,
                });
            }
        }
        _ => {}
    }
    Ok(())
}

/// The marked places of `architecture`, ordered by path.
pub fn collect(architecture: &Value, all: bool) -> Result<Vec<Marked>, String> {
    let mut out = Vec::new();
    walk(architecture, "", all, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn replace_markers(v: &mut Value) {
    if is_marker(v) {
        *v = v.get("value").cloned().unwrap_or(Value::Null);
        return;
    }
    match v {
        Value::Object(o) => o.values_mut().for_each(replace_markers),
        Value::Array(xs) => xs.iter_mut().for_each(replace_markers),
        _ => {}
    }
}

/// The architecture with `params` set at their paths and every marker replaced by its value.
pub fn resolve(architecture: &Value, params: &Map<String, Value>) -> Result<Value, String> {
    let mut out = architecture.clone();
    for (path, value) in params {
        let mut at = &mut out;
        for key in path.split('.') {
            at = match at {
                Value::Object(o) => o.get_mut(key),
                Value::Array(xs) => key.parse::<usize>().ok().and_then(|i| xs.get_mut(i)),
                _ => None,
            }
            .ok_or_else(|| format!("marked place `{path}` is not in the architecture"))?;
        }
        *at = value.clone();
    }
    replace_markers(&mut out);
    Ok(out)
}
