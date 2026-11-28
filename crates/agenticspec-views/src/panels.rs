//! The block panels as an `ess-ui/1` document, from the compiled model alone.
//!
//! One widget per view, named as the view: a `collection` of the view's rows, keyed by its first field, one column
//! per field, an enum (the `state` among them) drawn as a badge. A panel takes a scope
//! (`source`, `architecture`, `part`, `instance`), the view's required parameters and, when
//! events change its rows, the channel of the page it stands on. Its `live` is read from the
//! model: the events an outcome emits whose subject is the view's source entity. When each of
//! them carries every field of the view, an event inserts or patches its row; otherwise it
//! re-reads the view. The rows are scoped by the runtime that answers the read, not here: a read
//! binds only the parameters its view declares.
//!
//! A gallery page per block places every panel of the block over the fixtures; a channel per
//! block carries the events its panels take. Nothing here names a block, a view or a command.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde_json::Value;

use crate::values::{self, At};

/// One block: a domain of the system and the layer directory its file is in.
#[derive(Clone, Debug)]
pub struct Block {
    /// The qualified domain, `agentic.agent`.
    pub domain: String,
    /// The domain without the system's prefix, `agent`.
    pub short: String,
    /// The layer number, from the directory's prefix.
    pub layer: u32,
    /// The layer directory, `4-agent`.
    pub dir: String,
}

/// The scope every panel takes. A view parameter of one of these names is refused.
pub const SCOPE: [(&str, &str, &str); 4] = [
    (
        "source",
        "{optional: string}",
        "the base URL of the runtime adapter that answers this scope; absent, the renderer's own",
    ),
    (
        "architecture",
        "{optional: ArchitectureId}",
        "the architecture the rows are scoped to; the runtime narrows the rows, not this document",
    ),
    (
        "part",
        "{optional: string}",
        "the path of the part within the architecture, such as team.lead",
    ),
    (
        "instance",
        "{optional: integer}",
        "the copy of a part that runs more than one",
    ),
];

/// The rows each view's fixture holds.
pub const ROWS: usize = 3;

struct Column {
    field: String,
    as_: Option<&'static str>,
    label: Option<String>,
}

struct Param {
    name: String,
    type_ref: Value,
}

struct Panel {
    view: String,
    widget: String,
    section: String,
    title: String,
    summary: String,
    doc: Option<String>,
    key: String,
    columns: Vec<Column>,
    params: Vec<Param>,
    events: Vec<String>,
    insert: bool,
    rows: Vec<Value>,
}

/// The generated files, by their path below `ui/views/`, in a fixed order.
///
/// # Errors
///
/// A view without fields, a view parameter named like a scope parameter, or a type the
/// fixtures have no value for; the message names it.
pub fn generate(ir: &Value, blocks: &[Block]) -> Result<Vec<(String, String)>, String> {
    let emitted = events_by_entity(ir);
    let mut by_block: Vec<(&Block, Vec<Panel>)> = Vec::new();
    for b in blocks {
        let mut panels = Vec::new();
        let mut views: Vec<(&String, &Value)> = ir["views"]
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(_, v)| v["domain"] == b.domain.as_str())
            .collect();
        views.sort_by(|a, b| a.0.cmp(b.0));
        for (name, v) in views {
            panels.push(panel(ir, name, v, &emitted)?);
        }
        if !panels.is_empty() {
            by_block.push((b, panels));
        }
    }

    let mut files = vec![("blocks.ui.yaml".to_string(), document(ir, &by_block))];
    files.push(("fixtures/index.yaml".to_string(), index(&by_block)));
    for (_, panels) in &by_block {
        for p in panels {
            files.push((format!("fixtures/{}.json", p.widget), fixture(p)));
        }
    }
    for (b, panels) in &by_block {
        if let Some(script) = script(ir, b, panels)? {
            files.push((format!("fixtures/scripts/{}.json", b.short), script));
        }
    }
    Ok(files)
}

/// Every event an outcome emits, by the entity that outcome is about.
fn events_by_entity(ir: &Value) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for c in ir["commands"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(_, c)| c)
    {
        for o in c["outcomes"].as_array().into_iter().flatten() {
            let Some(entity) = o["subject"]["entity"].as_str() else {
                continue;
            };
            for e in o["emits"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                out.entry(entity.to_string())
                    .or_default()
                    .insert(e.to_string());
            }
        }
    }
    out
}

fn panel(
    ir: &Value,
    name: &str,
    v: &Value,
    emitted: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Panel, String> {
    let short = name.rsplit('.').next().unwrap_or(name);
    let fields = v["fields"].as_array().map_or(&[][..], Vec::as_slice);
    let key = fields
        .first()
        .and_then(|f| f["name"].as_str())
        .ok_or_else(|| format!("{name}: a view without fields has no row key"))?
        .to_string();
    let source = v["source"].as_str().unwrap_or("");
    let title = v["naming"]["display"].as_str().unwrap_or(short).to_string();
    let entity = source.rsplit('.').next().unwrap_or(source);
    let summary = format!("Rows of the view {name} over {entity}, keyed by {key}.");

    let columns = fields
        .iter()
        .map(|f| Column {
            field: f["name"].as_str().unwrap_or("").to_string(),
            as_: shown_as(ir, &f["type_ref"]),
            label: f["naming"]["display"].as_str().map(str::to_string),
        })
        .collect::<Vec<_>>();

    let mut params = Vec::new();
    for p in v["params"].as_array().into_iter().flatten() {
        let pname = p["name"].as_str().unwrap_or("");
        if p["type_ref"]["kind"] == "optional" {
            continue;
        }
        if SCOPE.iter().any(|(s, _, _)| *s == pname) || pname == "channel" {
            return Err(format!(
                "{name}: its parameter `{pname}` has the name of a parameter every panel takes"
            ));
        }
        params.push(Param {
            name: pname.to_string(),
            type_ref: p["type_ref"].clone(),
        });
    }

    let events: Vec<String> = emitted.get(source).into_iter().flatten().cloned().collect();
    let names: BTreeSet<&str> = fields.iter().filter_map(|f| f["name"].as_str()).collect();
    let insert = !events.is_empty()
        && events.iter().all(|e| {
            let carried: BTreeSet<&str> = ir["events"][e.as_str()]["fields"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|f| f["name"].as_str())
                .collect();
            names.is_subset(&carried)
        });

    let mut rows = Vec::new();
    for row in 0..ROWS {
        let mut out = serde_json::Map::new();
        for (pos, f) in fields.iter().enumerate() {
            let field = f["name"].as_str().unwrap_or("");
            // A view parameter that is also a field narrows the rows by it, so every row holds
            // the value of the first and the gallery, which passes that value, shows them all.
            let r = if params.iter().any(|p| p.name == field) {
                0
            } else {
                row
            };
            let at = At { field, row: r, pos };
            if let Some(x) =
                values::value(ir, &f["type_ref"], at).map_err(|e| format!("{name}: {e}"))?
            {
                out.insert(field.to_string(), x);
            }
        }
        rows.push(Value::Object(out));
    }

    Ok(Panel {
        view: name.to_string(),
        // The widget has the qualified name of its view. A shorter `<block>.<view>` is read as
        // the expression `channel.<name>` for the block `channel` by `ess ui check`.
        widget: name.to_string(),
        section: snake(short),
        title,
        summary,
        doc: v["naming"]["summary"].as_str().map(str::to_string),
        key,
        columns,
        params,
        events,
        insert,
        rows,
    })
}

/// The `as` of a column: a number, a time, a badge for an enum, `json` for a record or a union,
/// a list; plain text otherwise. A decimal and a duration stay text, since their wire form is
/// text (`10.50`, `PT30M`).
fn shown_as(ir: &Value, t: &Value) -> Option<&'static str> {
    match t["kind"].as_str()? {
        "optional" => shown_as(ir, &t["of"]),
        "primitive" => match t["name"].as_str()? {
            "integer" | "binary64" => Some("number"),
            "timestamp" => Some("timestamp"),
            "date" => Some("date"),
            _ => None,
        },
        "declared" => {
            let body = &ir["types"][t["name"].as_str()?]["body"];
            match body["kind"].as_str()? {
                "newtype" => shown_as(ir, &body["of"]),
                "enum" => Some("badge"),
                _ => Some("json"),
            }
        }
        "list" => match shown_as(ir, &t["of"]) {
            Some("json") => Some("json"),
            _ => Some("list"),
        },
        _ => Some("json"),
    }
}

/// The type of a widget parameter for a view parameter: a model type by its last segment, a
/// primitive by its `ess-ui/1` name.
fn param_type(t: &Value) -> String {
    match t["kind"].as_str().unwrap_or("") {
        "declared" => t["name"]
            .as_str()
            .unwrap_or("")
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_string(),
        "primitive" => match t["name"].as_str().unwrap_or("") {
            "integer" => "integer",
            "binary64" => "number",
            "boolean" => "boolean",
            "timestamp" => "timestamp",
            "date" => "date",
            _ => "string",
        }
        .to_string(),
        _ => "json".to_string(),
    }
}

/// `AgentRevisions` is `agent_revisions`.
fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// A YAML scalar: plain when it is a plain word or phrase, else a JSON string, which YAML reads
/// as a double-quoted scalar.
pub fn q(s: &str) -> String {
    let plain = s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '.' | '-'))
        && !s.ends_with(' ')
        && !matches!(
            s.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "y" | "n"
        );
    if plain {
        s.to_string()
    } else {
        Value::from(s).to_string()
    }
}

fn label(b: &Block, ir: &Value) -> String {
    ir["domains"][&b.domain]["naming"]["display"]
        .as_str()
        .unwrap_or(&b.short)
        .to_string()
}

fn document(ir: &Value, blocks: &[(&Block, Vec<Panel>)]) -> String {
    let system = ir["system"].as_str().unwrap_or("");
    let mut y = String::new();
    y.push_str("# Generated by `agenticspec ui panels` from blocks/. Never edited by hand: run `task ui:panels`.\n");
    y.push_str("format: ess-ui/1\napp: blocks\ntitle: Building blocks\n");
    let _ = writeln!(y, "model: {}", q(system));
    y.push_str("actor: from_session\nplacement_profile: fat\n");
    y.push_str("fixtures: {dir: fixtures, index: fixtures/index.yaml}\n\n");
    y.push_str("shells:\n  app:\n    regions:\n      nav: {kind: navigation}\n      main: {kind: page_outlet}\n\n");

    y.push_str("navigation:\n");
    if let Some((b, _)) = blocks.first() {
        let _ = writeln!(y, "  home: {}", b.short);
    }
    y.push_str("  sections:\n");
    let mut layers: BTreeMap<u32, (&str, Vec<&str>)> = BTreeMap::new();
    for (b, _) in blocks {
        layers
            .entry(b.layer)
            .or_insert((&b.dir, Vec::new()))
            .1
            .push(&b.short);
    }
    for (layer, (dir, pages)) in &layers {
        let words = dir.split_once('-').map_or(*dir, |(_, w)| w);
        let _ = writeln!(
            y,
            "    - {{name: layer_{layer}, label: {}, pages: [{}]}}",
            q(&format!("Layer {layer}: {words}")),
            pages.join(", ")
        );
    }

    y.push_str(
        "\npage_kinds:\n  gallery:\n    purpose: every panel of one block, over the fixtures\n",
    );

    let channels: Vec<(&Block, BTreeSet<&str>)> = blocks
        .iter()
        .map(|(b, ps)| {
            (
                *b,
                ps.iter()
                    .flat_map(|p| p.events.iter().map(String::as_str))
                    .collect(),
            )
        })
        .filter(|(_, e): &(&Block, BTreeSet<&str>)| !e.is_empty())
        .collect();
    if !channels.is_empty() {
        y.push_str("\nchannels:\n");
        for (b, events) in &channels {
            let _ = writeln!(y, "  {}:", b.short);
            y.push_str("    carries:\n      events:\n");
            for e in events {
                let _ = writeln!(y, "        - {e}");
            }
            y.push_str("    direction: server_to_client\n    delivery: every_event\n    resume: from_last_seen\n");
            y.push_str("    lifecycle: [connecting, live, reconnecting, stale, closed]\n");
        }
    }

    y.push_str("\nwidgets:\n");
    for (_, panels) in blocks {
        for p in panels {
            widget(&mut y, p);
        }
    }

    y.push_str("\npages:\n");
    for (b, panels) in blocks {
        let title = label(b, ir);
        let _ = writeln!(y, "  {}:", b.short);
        let _ = writeln!(
            y,
            "    kind: gallery\n    title: {}\n    nav: {{label: {}}}",
            q(&title),
            q(&title)
        );
        y.push_str("    sections:\n");
        for p in panels {
            let _ = writeln!(y, "      - name: {}", p.section);
            let _ = writeln!(y, "        title: {}", q(&p.title));
            let _ = writeln!(y, "        component: {}", p.widget);
            let mut args: Vec<String> = Vec::new();
            if !p.events.is_empty() {
                args.push(format!("channel: {}", b.short));
            }
            for prm in &p.params {
                let v = p.rows[0].get(&prm.name).cloned().unwrap_or_else(|| {
                    values::value(
                        ir,
                        &prm.type_ref,
                        At {
                            field: &prm.name,
                            row: 0,
                            pos: 0,
                        },
                    )
                    .ok()
                    .flatten()
                    .unwrap_or(Value::Null)
                });
                args.push(format!("{}: {}", prm.name, literal(&v)));
            }
            if !args.is_empty() {
                let _ = writeln!(y, "        args: {{{}}}", args.join(", "));
            }
        }
    }
    y
}

/// A literal argument: a string as a quoted expression literal, anything else as JSON.
fn literal(v: &Value) -> String {
    match v {
        Value::String(s) => Value::from(format!("'{s}'")).to_string(),
        other => other.to_string(),
    }
}

fn widget(y: &mut String, p: &Panel) {
    let _ = writeln!(y, "  {}:", p.widget);
    let _ = writeln!(y, "    summary: {}", q(&p.summary));
    if let Some(doc) = &p.doc {
        let _ = writeln!(y, "    doc: {}", q(doc));
    }
    y.push_str("    params:\n");
    for (name, ty, note) in SCOPE {
        let _ = writeln!(y, "      {name}: {{type: {ty}, note: {}}}", q(note));
    }
    if !p.events.is_empty() {
        let _ = writeln!(
            y,
            "      channel: {{type: {{ref: channel}}, required: true, note: {}}}",
            q("the channel of the page; it carries the events the panel takes")
        );
    }
    for prm in &p.params {
        let _ = writeln!(
            y,
            "      {}: {{type: {}, required: true, note: {}}}",
            prm.name,
            param_type(&prm.type_ref),
            q(&format!("the parameter {} of the view", prm.name))
        );
    }
    y.push_str(
        "    body:\n      - name: rows\n        component: collection\n        style: table\n",
    );
    let mut reads = format!("view: {}, key: {}", p.view, p.key);
    if !p.params.is_empty() {
        let bound: Vec<String> = p
            .params
            .iter()
            .map(|x| format!("{0}: args.{0}", x.name))
            .collect();
        let _ = write!(reads, ", params: {{{}}}", bound.join(", "));
    }
    let _ = writeln!(y, "        reads: {{{reads}}}");
    if !p.events.is_empty() {
        let effect = if p.insert {
            let only = if p.params.is_empty() {
                ""
            } else {
                ", only_if: matches(params)"
            };
            format!("effect: insert_or_patch, match: {}{only}", p.key)
        } else {
            "effect: refetch, coalesce: 1s".to_string()
        };
        let _ = writeln!(
            y,
            "        live:\n          channel: args.channel\n          on:"
        );
        for e in &p.events {
            let _ = writeln!(y, "            - {e}");
        }
        for part in effect.split(", ") {
            let _ = writeln!(y, "          {part}");
        }
    }
    y.push_str("        columns:\n");
    for c in &p.columns {
        let mut parts = vec![format!("field: {}", c.field)];
        if let Some(a) = c.as_ {
            parts.push(format!("as: {a}"));
        }
        if let Some(l) = &c.label {
            parts.push(format!("label: {}", q(l)));
        }
        if parts.len() == 1 {
            let _ = writeln!(y, "          - {}", c.field);
        } else {
            let _ = writeln!(y, "          - {{{}}}", parts.join(", "));
        }
    }
}

fn index(blocks: &[(&Block, Vec<Panel>)]) -> String {
    let mut y = String::from("views:\n");
    for (_, panels) in blocks {
        for p in panels {
            let _ = writeln!(y, "  {}: {}.json", p.view, p.widget);
        }
    }
    let scripted: Vec<&Block> = blocks
        .iter()
        .filter(|(_, ps)| ps.iter().any(|p| !p.events.is_empty()))
        .map(|(b, _)| *b)
        .collect();
    if !scripted.is_empty() {
        y.push_str("scripts:\n");
        for b in scripted {
            let _ = writeln!(y, "  {0}: scripts/{0}.json", b.short);
        }
    }
    y
}

/// One JSON object per line, its members in the order given.
fn object_line(members: &[(String, Value)]) -> String {
    let inner: Vec<String> = members
        .iter()
        .map(|(k, v)| format!("{}: {}", Value::from(k.as_str()), v))
        .collect();
    format!("{{{}}}", inner.join(", "))
}

fn ordered(row: &Value, order: &[String]) -> Vec<(String, Value)> {
    order
        .iter()
        .filter_map(|k| row.get(k).map(|v| (k.clone(), v.clone())))
        .collect()
}

fn fixture(p: &Panel) -> String {
    let order: Vec<String> = p.columns.iter().map(|c| c.field.clone()).collect();
    let rows: Vec<String> = p
        .rows
        .iter()
        .map(|r| format!("    {}", object_line(&ordered(r, &order))))
        .collect();
    format!(
        "{{\n  \"view\": {},\n  \"rows\": [\n{}\n  ]\n}}\n",
        Value::from(p.view.as_str()),
        rows.join(",\n")
    )
}

/// The script of a block's channel: each event its panels take, once, a second apart, with the
/// values of the first fixture row.
fn script(ir: &Value, b: &Block, panels: &[Panel]) -> Result<Option<String>, String> {
    let events: BTreeSet<&str> = panels
        .iter()
        .flat_map(|p| p.events.iter().map(String::as_str))
        .collect();
    if events.is_empty() {
        return Ok(None);
    }
    let mut entries = Vec::new();
    for (i, e) in events.iter().enumerate() {
        let mut payload = Vec::new();
        for (pos, f) in ir["events"][*e]["fields"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            let field = f["name"].as_str().unwrap_or("");
            if let Some(v) = values::value(ir, &f["type_ref"], At { field, row: 0, pos })
                .map_err(|x| format!("{e}: {x}"))?
            {
                payload.push((field.to_string(), v));
            }
        }
        entries.push(format!(
            "    {{\"at\": \"{}s\", \"event\": {}, \"payload\": {}}}",
            i + 1,
            Value::from(*e),
            object_line(&payload)
        ));
    }
    Ok(Some(format!(
        "{{\n  \"channel\": {},\n  \"events\": [\n{}\n  ]\n}}\n",
        Value::from(b.short.as_str()),
        entries.join(",\n")
    )))
}
