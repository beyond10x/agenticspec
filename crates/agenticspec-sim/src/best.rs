//! The best variant of a search as an architecture document: every marked place set to the value
//! of the best variant, its `vary` kept, so the document is searched again from there.

use crate::doc::parse_yaml;
use crate::search::{Experiment, run};
use serde_json::{Map, Value};
use std::fmt::Write as _;

/// `architecture` with each place of `params` set: a marker keeps its `vary` and takes the value
/// as its `value`; a place without a marker (under `optimize: all`) takes the value itself.
pub fn apply(architecture: &Value, params: &Map<String, Value>) -> Result<Value, String> {
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
        match at {
            Value::Object(o) if o.contains_key("vary") => {
                o.insert("value".into(), value.clone());
            }
            other => *other = value.clone(),
        }
    }
    Ok(out)
}

/// The best variant of a result (`agenticspec-sim-result/1`) as an architecture document, or
/// `None` when no variant passed the gates. A nested architecture stays a reference, as its
/// author writes it.
pub fn from_result(result: &Value) -> Result<Option<Value>, String> {
    let Some(params) = result["best"]["params"].as_object() else {
        return Ok(None);
    };
    apply(&result["architecture"], params)
        .map(|d| crate::nest::strip(&d))
        .map(Some)
}

/// The best variant of a result as YAML text, or `None` when no variant passed the gates. With
/// `source`, the text of the architecture as written, its keys keep their order.
pub fn yaml_of_result(result: &Value, source: Option<&str>) -> Result<Option<String>, String> {
    from_result(result)?
        .map(|d| match source {
            Some(text) => yaml_like(&d, text),
            None => yaml(&d),
        })
        .transpose()
}

/// The fitness of `architecture` as written under the experiment `e`: its seeds, horizon, faults,
/// profile and weights, the architecture's markers at their values. For the best variant of a run
/// of `e` this is the fitness of that best variant.
pub fn fitness_as_written(
    e: &Experiment,
    architecture: Value,
    threads: usize,
) -> Result<f64, String> {
    // Its nested architectures are the ones `e` resolved, when it pins the same.
    let architecture = crate::nest::graft(&architecture, &e.architecture);
    let mut again = Experiment::from_docs(e.doc.clone(), architecture, e.profile.clone())?;
    again.search.mode = "fit".into();
    again.search.budget = 1;
    let result = run(&again, threads)?;
    result["baseline"]["fitness"]
        .as_f64()
        .ok_or_else(|| "the run has no baseline fitness".to_string())
}

/// Top-level keys of an architecture document in the order they are written, when no source
/// gives the order.
const TOP: &[&str] = &[
    "format",
    "architecture",
    "summary",
    "parts",
    "links",
    "border",
    "ingress",
    "requirements",
    "policies",
    "view",
];

/// Keys inside a part, a link or a marker, when no source gives the order: first these, the rest
/// by name.
const INNER: &[&str] = &[
    "block", "kind", "harness", "acts_for", "from", "to", "carries", "over", "value", "vary",
    "min", "max", "step",
];

/// The order of the keys: the one of the source document where it has a mapping at the same
/// place, else a fixed list first and the rest by name.
struct Order {
    source: Option<serde_yaml_ng::Value>,
}

impl Order {
    fn at(&self, path: &[String]) -> Option<Vec<String>> {
        let mut v = self.source.as_ref()?;
        for k in path {
            v = match v {
                serde_yaml_ng::Value::Mapping(m) => m.get(k.as_str())?,
                serde_yaml_ng::Value::Sequence(xs) => xs.get(k.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        let m = v.as_mapping()?;
        Some(
            m.keys()
                .filter_map(|k| k.as_str().map(str::to_string))
                .collect(),
        )
    }

    fn keys<'a>(
        &self,
        o: &'a Map<String, Value>,
        path: &[String],
        fallback: &[&str],
    ) -> Vec<(&'a String, &'a Value)> {
        let first: Vec<String> = self
            .at(path)
            .unwrap_or_else(|| fallback.iter().map(|s| s.to_string()).collect());
        let mut out: Vec<(&String, &Value)> = Vec::with_capacity(o.len());
        for k in &first {
            if let Some((key, v)) = o.get_key_value(k.as_str()) {
                out.push((key, v));
            }
        }
        for (k, v) in o {
            if !first.iter().any(|f| f == k) {
                out.push((k, v));
            }
        }
        out
    }

    /// A value in flow style: `{ k: v }`, `[a, b]`, a scalar.
    fn flow(&self, v: &Value, path: &mut Vec<String>) -> String {
        match v {
            Value::Object(o) if o.is_empty() => "{}".into(),
            Value::Object(o) => {
                let mut items = Vec::with_capacity(o.len());
                for (k, x) in self.keys(o, path, INNER) {
                    path.push(k.clone());
                    items.push(format!("{}: {}", key(k), self.flow(x, path)));
                    path.pop();
                }
                format!("{{ {} }}", items.join(", "))
            }
            Value::Array(xs) => {
                let mut items = Vec::with_capacity(xs.len());
                for (i, x) in xs.iter().enumerate() {
                    path.push(i.to_string());
                    items.push(self.flow(x, path));
                    path.pop();
                }
                format!("[{}]", items.join(", "))
            }
            other => scalar(other),
        }
    }
}

/// A string written without quotes where YAML reads it back as the same string.
fn plain(s: &str) -> bool {
    let ok_chars = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-' | ' '));
    let starts = s
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
    ok_chars
        && starts
        && !s.ends_with(' ')
        && !s.contains("  ")
        && parse_yaml(s).ok() == Some(Value::String(s.to_string()))
}

fn scalar(v: &Value) -> String {
    match v {
        Value::String(s) if plain(s) => s.clone(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_default(),
        Value::Null => "null".into(),
        other => other.to_string(),
    }
}

fn key(k: &str) -> String {
    scalar(&Value::String(k.to_string()))
}

/// A long string as a folded block scalar, lines of at most about 100 columns; `None` when
/// folding would change it.
fn folded(s: &str) -> Option<String> {
    if s.len() <= 90
        || s.contains(['\n', '\r', '\t'])
        || s.contains("  ")
        || s.starts_with(' ')
        || s.ends_with(' ')
    {
        return None;
    }
    let mut out = String::from(">-\n");
    let mut line = String::new();
    for word in s.split(' ') {
        if !line.is_empty() && line.len() + 1 + word.len() > 98 {
            let _ = writeln!(out, "  {line}");
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    let _ = write!(out, "  {line}");
    Some(out)
}

/// An architecture document as YAML in the layout of `architectures/`: one line per part, link
/// and policy, keys in a fixed order. The text reads back as exactly `doc`, or this is an error.
pub fn yaml(doc: &Value) -> Result<String, String> {
    write_yaml(doc, None)
}

/// As [`yaml`], with the keys in the order of `source`, the text of the document it was derived
/// from (the architecture as written), wherever that has the same mapping.
pub fn yaml_like(doc: &Value, source: &str) -> Result<String, String> {
    let parsed = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(source)
        .map_err(|e| format!("the source document does not parse: {e}"))?;
    write_yaml(doc, Some(parsed))
}

fn write_yaml(doc: &Value, source: Option<serde_yaml_ng::Value>) -> Result<String, String> {
    let order = Order { source };
    let o = doc
        .as_object()
        .ok_or("an architecture document is a mapping")?;
    let mut s = String::new();
    let mut previous_block = false;
    let mut path: Vec<String> = Vec::new();
    for (i, (k, v)) in order.keys(o, &[], TOP).into_iter().enumerate() {
        let block = matches!(v, Value::Object(m) if !m.is_empty())
            || matches!(v, Value::Array(xs) if xs.iter().any(Value::is_object));
        if i > 0 && (block || previous_block) {
            s.push('\n');
        }
        previous_block = block;
        path.push(k.clone());
        match v {
            Value::Object(m) if !m.is_empty() => {
                let _ = writeln!(s, "{}:", key(k));
                let width = m.keys().map(|n| key(n).len() + 1).max().unwrap_or(0);
                for (name, x) in order.keys(m, &path, &[]) {
                    let label = format!("{}:", key(name));
                    path.push(name.clone());
                    let _ = writeln!(s, "  {label:<width$} {}", order.flow(x, &mut path));
                    path.pop();
                }
            }
            Value::Array(xs) if xs.iter().any(Value::is_object) => {
                let _ = writeln!(s, "{}:", key(k));
                for (n, x) in xs.iter().enumerate() {
                    path.push(n.to_string());
                    let _ = writeln!(s, "  - {}", order.flow(x, &mut path));
                    path.pop();
                }
            }
            Value::String(text) => match folded(text) {
                Some(f) => {
                    let _ = writeln!(s, "{}: {f}", key(k));
                }
                None => {
                    let _ = writeln!(s, "{}: {}", key(k), scalar(v));
                }
            },
            other => {
                let _ = writeln!(s, "{}: {}", key(k), order.flow(other, &mut path));
            }
        }
        path.pop();
    }
    let back = parse_yaml(&s).map_err(|e| format!("the written YAML does not parse: {e}"))?;
    if &back != doc {
        return Err("the written YAML does not read back as the document".into());
    }
    Ok(s)
}
