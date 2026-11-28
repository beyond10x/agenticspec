//! Rows of the building blocks: the entities and types `blocks/` declares, embedded at build
//! time, and the check of one row against them. A row is `{ entity, id, state, fields,
//! not_recorded, evidence }`: a required field is in `fields` or named in `not_recorded`, where
//! the record a row was lifted from does not hold it.

use crate::reader::Reader;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

macro_rules! sources {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_str!(concat!("../../../blocks/", $path)))),*]
    };
}

/// Every block file, by its path under `blocks/`, as built into the binary.
pub const SOURCES: &[(&str, &str)] = sources![
    "0-foundation/shared.yaml",
    "1-principals/delegation.yaml",
    "1-principals/principal.yaml",
    "2-intelligence/gateway.yaml",
    "2-intelligence/model.yaml",
    "3-capability/knowledge.yaml",
    "3-capability/tool.yaml",
    "4-agent/agent.yaml",
    "4-agent/harness.yaml",
    "5-work/process.yaml",
    "5-work/work.yaml",
    "6-governance/authority.yaml",
    "7-collaboration/channel.yaml",
    "7-collaboration/question.yaml",
    "7-collaboration/trigger.yaml",
    "8-place/resource.yaml",
    "8-place/runtime.yaml",
    "8-place/sandbox.yaml",
    "8-place/workspace.yaml",
    "9-composition/architecture.yaml",
    "10-evaluation/experiment.yaml",
    "10-evaluation/kpi.yaml",
];

/// One entity: its identity, its fields with their types, and the states of its lifecycle.
#[derive(Debug, Clone)]
pub struct Entity {
    pub identity: String,
    pub identity_type: String,
    pub fields: Vec<(String, String)>,
    pub states: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TypeDecl {
    Enum(Vec<String>),
    Struct(Vec<(String, String)>),
    Newtype(String),
    Union,
}

/// The declarations of every block.
#[derive(Debug, Default)]
pub struct Blocks {
    pub entities: BTreeMap<String, Entity>,
    pub types: BTreeMap<String, TypeDecl>,
}

fn named_types(list: &Value) -> Vec<(String, String)> {
    list.as_array()
        .into_iter()
        .flatten()
        .filter_map(|f| {
            Some((
                f["name"].as_str()?.to_string(),
                f["type"].as_str()?.to_string(),
            ))
        })
        .collect()
}

/// Reads the declarations of block documents.
pub fn parse<'a>(docs: impl IntoIterator<Item = &'a str>) -> Result<Blocks, String> {
    let mut b = Blocks::default();
    for text in docs {
        let doc: Value = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
        for t in doc["types"].as_array().into_iter().flatten() {
            let Some(name) = t["name"].as_str() else {
                continue;
            };
            let decl = match t["kind"].as_str() {
                Some("enum") => TypeDecl::Enum(
                    t["variants"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect(),
                ),
                Some("struct") => TypeDecl::Struct(named_types(&t["fields"])),
                Some("newtype") => TypeDecl::Newtype(t["of"].as_str().unwrap_or("").to_string()),
                _ => TypeDecl::Union,
            };
            b.types.insert(name.to_string(), decl);
        }
        for e in doc["entities"].as_array().into_iter().flatten() {
            let Some(name) = e["name"].as_str() else {
                continue;
            };
            let states = e["lifecycle"]["states"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| s.as_str().map(str::to_string))
                .collect();
            b.entities.insert(
                name.to_string(),
                Entity {
                    identity: e["identity"]["name"].as_str().unwrap_or("").to_string(),
                    identity_type: e["identity"]["type"].as_str().unwrap_or("").to_string(),
                    fields: named_types(&e["fields"]),
                    states,
                },
            );
        }
    }
    Ok(b)
}

/// The declarations of the blocks built into the binary, read once.
pub fn blocks() -> &'static Blocks {
    static BLOCKS: OnceLock<Blocks> = OnceLock::new();
    BLOCKS.get_or_init(|| {
        parse(SOURCES.iter().map(|(_, t)| *t)).expect("the block files built in are YAML")
    })
}

/// One row of a block entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub entity: String,
    pub id: String,
    pub state: Option<String>,
    pub fields: Map<String, Value>,
    pub not_recorded: Vec<String>,
    /// The record the row was lifted from, such as a node of a session (`<session>#<n>`).
    pub evidence: Option<String>,
}

pub const ROW_KEYS: &[&str] = &[
    "entity",
    "id",
    "state",
    "fields",
    "not_recorded",
    "evidence",
];

/// Reads one row at `p`; the check against the blocks is [`check_row`].
pub fn read_row(r: &mut Reader, v: &Value, p: &str) -> Option<Row> {
    let o = r.obj(v, p, ROW_KEYS, &["entity", "id"])?;
    let entity = r.text(o, "entity", p);
    let id = r.text(o, "id", p);
    let state = r.text(o, "state", p);
    let evidence = r.text(o, "evidence", p);
    let not_recorded = if o.contains_key("not_recorded") {
        r.texts(o, "not_recorded", p).unwrap_or_default()
    } else {
        Vec::new()
    };
    let fields = match o.get("fields") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(m)) => m.clone(),
        Some(_) => {
            r.err(&format!("{p}.fields"), "expected a mapping");
            Map::new()
        }
    };
    Some(Row {
        entity: entity?,
        id: id?,
        state,
        fields,
        not_recorded,
        evidence,
    })
}

/// The row as a document value, in the form [`read_row`] reads.
pub fn row_value(row: &Row) -> Value {
    let mut m = Map::new();
    m.insert("entity".into(), Value::String(row.entity.clone()));
    m.insert("id".into(), Value::String(row.id.clone()));
    if let Some(s) = &row.state {
        m.insert("state".into(), Value::String(s.clone()));
    }
    m.insert("fields".into(), Value::Object(row.fields.clone()));
    if !row.not_recorded.is_empty() {
        m.insert(
            "not_recorded".into(),
            Value::Array(
                row.not_recorded
                    .iter()
                    .map(|f| Value::String(f.clone()))
                    .collect(),
            ),
        );
    }
    if let Some(e) = &row.evidence {
        m.insert("evidence".into(), Value::String(e.clone()));
    }
    Value::Object(m)
}

fn inner<'t>(ty: &'t str, wrapper: &str) -> Option<&'t str> {
    ty.strip_prefix(wrapper)?
        .strip_prefix('<')?
        .strip_suffix('>')
}

fn is_uuid(s: &str) -> bool {
    s.len() == 36
        && s.char_indices().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// Every problem of `v` as a value of type `ty`, each named by its path.
pub fn check_value(b: &Blocks, ty: &str, v: &Value, p: &str, out: &mut Vec<String>) {
    if let Some(t) = inner(ty, "Optional") {
        if !v.is_null() {
            check_value(b, t, v, p, out);
        }
        return;
    }
    if let Some(t) = inner(ty, "List") {
        match v.as_array() {
            Some(xs) => {
                for (i, x) in xs.iter().enumerate() {
                    check_value(b, t, x, &format!("{p}.{i}"), out);
                }
            }
            None => out.push(format!("{p}: expected a list of {t}")),
        }
        return;
    }
    let ok = match ty {
        "String" | "Timestamp" | "Duration" => v.is_string(),
        "Integer" => v.is_i64() || v.is_u64(),
        "Decimal" => v.is_number(),
        "Boolean" => v.is_boolean(),
        "Uuid" => v.as_str().is_some_and(is_uuid),
        _ => match b.types.get(ty) {
            Some(TypeDecl::Newtype(of)) => {
                check_value(b, of, v, p, out);
                return;
            }
            Some(TypeDecl::Enum(variants)) => {
                if !v.as_str().is_some_and(|s| variants.iter().any(|x| x == s)) {
                    out.push(format!(
                        "{p}: {v} is not one of {ty}: {}",
                        variants.join(", ")
                    ));
                }
                return;
            }
            Some(TypeDecl::Struct(fields)) => {
                let Some(o) = v.as_object() else {
                    out.push(format!("{p}: expected a mapping of {ty}"));
                    return;
                };
                for k in o.keys() {
                    if !fields.iter().any(|(n, _)| n == k) {
                        out.push(format!("{p}.{k}: no field of {ty}"));
                    }
                }
                for (n, t) in fields {
                    match o.get(n) {
                        Some(x) => check_value(b, t, x, &format!("{p}.{n}"), out),
                        None if inner(t, "Optional").is_none() => {
                            out.push(format!("{p}.{n}: missing, {ty} requires it"))
                        }
                        None => {}
                    }
                }
                return;
            }
            Some(TypeDecl::Union) | None => true,
        },
    };
    if !ok {
        out.push(format!("{p}: {v} is not of type {ty}"));
    }
}

/// Every problem of a row against the blocks, each named by its path under `p`: an entity no
/// block declares, an identifier, a state or a field value of the wrong type, a field the
/// entity does not have, a required field neither given nor named in `not_recorded`.
pub fn check_row(b: &Blocks, row: &Row, p: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(e) = b.entities.get(&row.entity) else {
        out.push(format!(
            "{p}.entity: `{}` is no entity of the building blocks",
            row.entity
        ));
        return out;
    };
    check_value(
        b,
        &e.identity_type,
        &Value::String(row.id.clone()),
        &format!("{p}.id"),
        &mut out,
    );
    if let Some(s) = &row.state
        && !e.states.contains(s)
    {
        out.push(format!(
            "{p}.state: `{s}` is no state of {}: {}",
            row.entity,
            e.states.join(", ")
        ));
    }
    for (k, v) in &row.fields {
        match e.fields.iter().find(|(n, _)| n == k) {
            Some((_, t)) => check_value(b, t, v, &format!("{p}.fields.{k}"), &mut out),
            None => out.push(format!("{p}.fields.{k}: no field of {}", row.entity)),
        }
    }
    for (i, n) in row.not_recorded.iter().enumerate() {
        if !e.fields.iter().any(|(f, _)| f == n) {
            out.push(format!(
                "{p}.not_recorded.{i}: `{n}` is no field of {}",
                row.entity
            ));
        } else if row.fields.contains_key(n) {
            out.push(format!(
                "{p}.not_recorded.{i}: `{n}` is given in fields as well"
            ));
        }
    }
    for (n, t) in &e.fields {
        if inner(t, "Optional").is_none()
            && !row.fields.contains_key(n)
            && !row.not_recorded.contains(n)
        {
            out.push(format!(
                "{p}.fields.{n}: missing, {} requires it (or name it in not_recorded)",
                row.entity
            ));
        }
    }
    out
}
