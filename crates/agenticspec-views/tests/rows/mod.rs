//! The fixtures of the real blocks, generated in memory, against the compiled model: every row
//! of every view and every payload of every script holds each required field in the wire form
//! of its type, and nothing else. `ess ui check` reads no fixture row, so this test does.

use serde_json::Value;

use crate::{generated, model, parsed, repo};

/// Whether `v` is a wire value of `t`.
fn conforms(ir: &Value, t: &Value, v: &Value, at: &str) -> Result<(), String> {
    let bad = || Err(format!("{at}: {v} is not a {t}"));
    match t["kind"].as_str().unwrap_or("") {
        "optional" => conforms(ir, &t["of"], v, at),
        "list" => match v.as_array() {
            Some(items) => items.iter().try_for_each(|x| conforms(ir, &t["of"], x, at)),
            None => bad(),
        },
        "map" => {
            if v.is_object() {
                Ok(())
            } else {
                bad()
            }
        }
        "primitive" => {
            let ok = match t["name"].as_str().unwrap_or("") {
                "string" | "decimal" => v.is_string(),
                "integer" => v.is_i64(),
                "binary64" => v.is_number(),
                "boolean" => v.is_boolean(),
                "timestamp" => v
                    .as_str()
                    .is_some_and(|s| s.len() == 20 && s.ends_with('Z') && s.as_bytes()[10] == b'T'),
                "date" => v.as_str().is_some_and(|s| s.len() == 10),
                "duration" => v.as_str().is_some_and(|s| s.starts_with('P')),
                "uuid" => v.as_str().is_some_and(|s| {
                    s.len() == 36
                        && s.chars().enumerate().all(|(i, c)| {
                            if [8, 13, 18, 23].contains(&i) {
                                c == '-'
                            } else {
                                c.is_ascii_hexdigit()
                            }
                        })
                }),
                _ => false,
            };
            if ok { Ok(()) } else { bad() }
        }
        "declared" => {
            let name = t["name"].as_str().unwrap_or("");
            let body = &ir["types"][name]["body"];
            match body["kind"].as_str().unwrap_or("") {
                "newtype" => conforms(ir, &body["of"], v, at),
                "enum" => {
                    if body["variants"].as_array().is_some_and(|vs| vs.contains(v)) {
                        Ok(())
                    } else {
                        bad()
                    }
                }
                "struct" => record(
                    ir,
                    body["fields"].as_array().map_or(&[][..], Vec::as_slice),
                    v,
                    &format!("{at}.{name}"),
                ),
                "union" => {
                    let tag = body["tag"].as_str().unwrap_or("kind");
                    let variant = v[tag].as_str().unwrap_or("");
                    let payload = &body["variants"][variant];
                    if !body["variants"]
                        .as_object()
                        .is_some_and(|m| m.contains_key(variant))
                    {
                        return bad();
                    }
                    if payload.is_null() {
                        Ok(())
                    } else {
                        conforms(ir, payload, &v["value"], at)
                    }
                }
                _ => bad(),
            }
        }
        _ => bad(),
    }
}

/// Whether `v` is an object with every required field of `fields`, each of its type, and no
/// other member.
fn record(ir: &Value, fields: &[Value], v: &Value, at: &str) -> Result<(), String> {
    let obj = v
        .as_object()
        .ok_or_else(|| format!("{at}: {v} is not an object"))?;
    for f in fields {
        let name = f["name"].as_str().unwrap_or("");
        match obj.get(name) {
            Some(x) => conforms(ir, &f["type_ref"], x, &format!("{at}.{name}"))?,
            None if f["type_ref"]["kind"] == "optional" => {}
            None => return Err(format!("{at}: the required field {name} is missing")),
        }
    }
    for k in obj.keys() {
        if !fields.iter().any(|f| f["name"] == k.as_str()) {
            return Err(format!("{at}: {k} is no field"));
        }
    }
    Ok(())
}

#[test]
fn every_fixture_row_and_script_payload_of_the_blocks_is_a_wire_value_of_its_type() {
    let (ir, blocks) = model(&repo().join("blocks"));
    let files = generated(&ir, &blocks);
    let views = ir["views"].as_object().expect("views");
    let mut rows = 0;
    for (name, v) in views {
        let fixture = parsed(&files, &format!("fixtures/{name}.json"));
        assert_eq!(fixture["view"], name.as_str());
        let fields = v["fields"].as_array().expect("fields");
        let list = fixture["rows"].as_array().expect("rows");
        assert_eq!(list.len(), agenticspec_views::panels::ROWS, "{name}");
        let key = fields[0]["name"].as_str().expect("a key");
        let mut keys: Vec<&Value> = list.iter().map(|r| &r[key]).collect();
        keys.dedup();
        assert_eq!(
            keys.len(),
            list.len(),
            "{name}: the rows have distinct keys"
        );
        for (i, r) in list.iter().enumerate() {
            record(&ir, fields, r, &format!("{name} row {i}")).unwrap_or_else(|e| panic!("{e}"));
            rows += 1;
        }
    }
    assert_eq!(rows, views.len() * agenticspec_views::panels::ROWS);

    let doc = parsed(&files, "blocks.ui.yaml");
    let mut scripts = 0;
    for (path, _) in files
        .iter()
        .filter(|(p, _)| p.starts_with("fixtures/scripts/"))
    {
        let script = parsed(&files, path);
        let channel = script["channel"].as_str().expect("a channel");
        let carried = doc["channels"][channel]["carries"]["events"]
            .as_array()
            .expect("the channel is declared");
        for entry in script["events"].as_array().expect("events") {
            let event = entry["event"].as_str().expect("an event");
            assert!(
                carried.contains(&entry["event"]),
                "{channel} carries {event}"
            );
            let fields = ir["events"][event]["fields"]
                .as_array()
                .expect("event fields");
            record(&ir, fields, &entry["payload"], event).unwrap_or_else(|e| panic!("{e}"));
            scripts += 1;
        }
    }
    let carried: usize = doc["channels"]
        .as_object()
        .expect("channels")
        .values()
        .map(|c| c["carries"]["events"].as_array().map_or(0, Vec::len))
        .sum();
    assert_eq!(scripts, carried, "every carried event is played once");
}
