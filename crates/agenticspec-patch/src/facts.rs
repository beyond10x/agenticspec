//! The facts of an architecture document (`agenticspec-architecture/1`), read as
//! `agenticspec-sim` reads it: an untyped document whose `parts` map an id to a `block` and the
//! settings of that block, and whose `links` list `from`, `to`, `carries` and an optional `over`.

use agenticspec_records::sq::architecture::{self, Facts, Link, Marker, Part, Setting};
use agenticspec_records::sq::facts::{FactBase, Val, vs};
use agenticspec_sim::check::value;
use agenticspec_sim::doc::{is_duration, num, parse_duration};
use agenticspec_sim::marked;
use serde_json::Value;

/// A value of the document as sq holds it: null, a boolean, an integer or a string. A duration
/// (`10m`) is its milliseconds, as sq compares durations. A number that is not whole has no sq
/// type and is held as its decimal text; an object or a list as its JSON text.
pub fn val(v: &Value) -> Val {
    match v {
        Value::Null => Val::Null,
        Value::Bool(b) => Val::Bool(*b),
        Value::Number(n) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => Val::Int(i),
            (None, Some(x)) if x.fract() == 0.0 && x.abs() < 9.0e15 => Val::Int(x as i64),
            _ => vs(&n.to_string()),
        },
        Value::String(s) if is_duration(v) => match parse_duration(s) {
            Some(seconds) => Val::Int((seconds * 1000.0).round() as i64),
            None => vs(s),
        },
        Value::String(s) => vs(s),
        other => vs(&other.to_string()),
    }
}

/// A part's count as `agenticspec-sim` reads it: the number (or the `value` of a marked node),
/// rounded, 1 when absent.
pub fn count(part: &Value) -> i64 {
    num(value(&part["count"])).unwrap_or(1.0).max(0.0).round() as i64
}

/// The facts `part`, `setting`, `link` and `marker` of an architecture document.
pub fn facts(doc: &Value) -> Facts {
    let mut f = Facts {
        name: doc["architecture"].as_str().unwrap_or("").to_string(),
        ..Facts::default()
    };
    if let Some(parts) = doc["parts"].as_object() {
        for (id, p) in parts {
            f.parts.push(Part {
                id: id.clone(),
                block: val(&p["block"]),
                count: count(p),
            });
            let Some(keys) = p.as_object() else { continue };
            for (name, v) in keys {
                if name == "block" || name == "count" {
                    continue;
                }
                let elements = match value(v) {
                    Value::Array(xs) => xs.iter().collect(),
                    one => vec![one],
                };
                for e in elements {
                    f.settings.push(Setting {
                        part: id.clone(),
                        name: name.clone(),
                        value: val(e),
                    });
                }
            }
        }
    }
    if let Some(links) = doc["links"].as_array() {
        for (i, l) in links.iter().enumerate() {
            f.links.push(Link {
                index: i as i64,
                from: val(&l["from"]),
                to: val(&l["to"]),
                carries: val(&l["carries"]),
                over: val(&l["over"]),
            });
        }
    }
    for m in marked::collect(doc, false).unwrap_or_default() {
        f.markers.push(Marker {
            path: m.path,
            value: val(&m.value),
        });
    }
    f
}

/// The fact base a selector of a patch runs over.
pub fn fact_base(doc: &Value) -> FactBase {
    architecture::fact_base(&facts(doc))
}
