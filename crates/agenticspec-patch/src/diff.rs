//! What a patch changed: every place whose value differs between two documents.

use std::collections::BTreeSet;

use serde_json::Value;

/// One changed place: its path (keys and list positions joined by `.`), its value before (none
/// when added) and after (none when removed).
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub path: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

fn walk(a: Option<&Value>, b: Option<&Value>, path: &str, out: &mut Vec<Change>) {
    match (a, b) {
        (Some(Value::Object(x)), Some(Value::Object(y))) => {
            let keys: BTreeSet<&String> = x.keys().chain(y.keys()).collect();
            for k in keys {
                walk(x.get(k), y.get(k), &join(path, k), out);
            }
        }
        (Some(Value::Array(x)), Some(Value::Array(y))) => {
            for i in 0..x.len().max(y.len()) {
                walk(x.get(i), y.get(i), &join(path, &i.to_string()), out);
            }
        }
        (a, b) if a != b => out.push(Change {
            path: path.to_string(),
            before: a.cloned(),
            after: b.cloned(),
        }),
        _ => {}
    }
}

/// The changes from `before` to `after`, ordered by path.
pub fn diff(before: &Value, after: &Value) -> Vec<Change> {
    let mut out = Vec::new();
    walk(Some(before), Some(after), "", &mut out);
    out
}

/// How many rows the changes touch: a part (`parts.<id>`), a link (`links.<n>`) or another
/// entry (`policies.<id>`), each counted once.
pub fn rows(changes: &[Change]) -> usize {
    changes
        .iter()
        .map(|c| c.path.splitn(3, '.').take(2).collect::<Vec<_>>().join("."))
        .collect::<BTreeSet<_>>()
        .len()
}
