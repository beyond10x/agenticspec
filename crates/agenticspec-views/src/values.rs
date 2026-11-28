//! Synthetic values of the compiled model's types, for the fixture rows and the script payloads
//! the panels render against.
//!
//! A value is a function of its type, the field it fills, the row and the field's position, and
//! of nothing else, so the fixtures are the same bytes on every run. An identifier is the same
//! UUID wherever its type stands in row `n`, so a row of one view and the rows of another that
//! name it agree.

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

/// Where a value stands: the field it fills, the row (from 0) and the field's position in its
/// row or record (from 0).
#[derive(Clone, Copy)]
pub struct At<'a> {
    pub field: &'a str,
    pub row: usize,
    pub pos: usize,
}

/// The value of `type_ref` at `at`, or `None` for an optional value that row leaves out (every
/// third row from the second on).
///
/// # Errors
///
/// A type the model does not declare, or a kind of type or primitive this generator has no
/// value for; the message names it.
pub fn value(ir: &Value, type_ref: &Value, at: At) -> Result<Option<Value>, String> {
    match type_ref["kind"].as_str().unwrap_or("") {
        "optional" => {
            if at.row % 3 == 1 {
                Ok(None)
            } else {
                value(ir, &type_ref["of"], at)
            }
        }
        "list" => Ok(Some(Value::Array(
            value(ir, &type_ref["of"], at)?.into_iter().collect(),
        ))),
        "map" => Ok(Some(Value::Object(Map::new()))),
        "primitive" => primitive(type_ref["name"].as_str().unwrap_or(""), at.field, at).map(Some),
        "declared" => declared(ir, type_ref["name"].as_str().unwrap_or(""), at).map(Some),
        other => Err(format!(
            "field {}: no value for a type of kind `{other}`",
            at.field
        )),
    }
}

fn declared(ir: &Value, name: &str, at: At) -> Result<Value, String> {
    let body = &ir["types"][name]["body"];
    match body["kind"].as_str() {
        Some("newtype") => {
            let of = &body["of"];
            if of["kind"] == "primitive" {
                primitive(of["name"].as_str().unwrap_or(""), name, at)
            } else {
                value(ir, of, at)?.ok_or_else(|| format!("{name}: an optional inside a newtype"))
            }
        }
        Some("enum") => {
            let variants = body["variants"].as_array().map_or(&[][..], Vec::as_slice);
            variants
                .get(at.row % variants.len().max(1))
                .cloned()
                .ok_or_else(|| format!("{name}: an enum with no variant"))
        }
        Some("struct") => {
            let mut out = Map::new();
            for (pos, f) in body["fields"].as_array().into_iter().flatten().enumerate() {
                let field = f["name"].as_str().unwrap_or("");
                let inner = At {
                    field,
                    row: at.row,
                    pos,
                };
                if let Some(v) = value(ir, &f["type_ref"], inner)? {
                    out.insert(field.to_string(), v);
                }
            }
            Ok(Value::Object(out))
        }
        Some("union") => {
            let tag = body["tag"].as_str().unwrap_or("kind");
            let variants = body["variants"]
                .as_object()
                .ok_or_else(|| format!("{name}: a union without variants"))?;
            let (variant, payload) = variants
                .iter()
                .nth(at.row % variants.len().max(1))
                .ok_or_else(|| format!("{name}: a union with no variant"))?;
            let mut out = Map::new();
            out.insert(tag.to_string(), json!(variant));
            if !payload.is_null()
                && let Some(v) = value(ir, payload, at)?
            {
                out.insert("value".to_string(), v);
            }
            Ok(Value::Object(out))
        }
        Some(other) => Err(format!("{name}: no value for a type of kind `{other}`")),
        None => Err(format!("{name}: not a type of the model")),
    }
}

/// A primitive value. `key` names what a UUID identifies: the newtype, or the field of a bare
/// `uuid`.
fn primitive(name: &str, key: &str, at: At) -> Result<Value, String> {
    let n = at.row + 1;
    Ok(match name {
        "string" => json!(format!("{} {n}", at.field.replace('_', " "))),
        "integer" => json!(n),
        "binary64" => json!(n as f64 / 2.0),
        "decimal" => json!(format!("{n}.50")),
        "boolean" => json!(at.row.is_multiple_of(2)),
        "timestamp" => json!(format!(
            "2026-11-20T{:02}:{:02}:00Z",
            9 + at.row % 12,
            at.pos % 60
        )),
        "date" => json!(format!("2026-11-{:02}", 20 + at.row % 8)),
        "duration" => json!(format!("PT{}M", 5 * n)),
        "uuid" => json!(uuid(key, at.row)),
        other => {
            return Err(format!(
                "field {}: no value for the primitive `{other}`",
                at.field
            ));
        }
    })
}

/// A UUID in canonical form, the same for the same `key` and row: the first four bytes of the
/// SHA-256 of `key`, then the row.
pub fn uuid(key: &str, row: usize) -> String {
    let d = Sha256::digest(key.as_bytes());
    format!(
        "00000000-0000-4000-8000-{:02x}{:02x}{:02x}{:02x}{:04x}",
        d[0],
        d[1],
        d[2],
        d[3],
        row & 0xffff
    )
}
