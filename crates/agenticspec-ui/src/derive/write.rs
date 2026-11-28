//! Writing a plan: the `ess-ui/1` document, its fixtures and the layout of the view package. Every
//! map the output walks is ordered, so the same inputs give the same bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde_json::{Map, Value, json};

use super::model::{self, Level};
use super::{Action, Entry, Plan, Section, channel_name, lit, pascal};

/// A column of an overview table: field, `as`, label.
pub type Column = (&'static str, Option<&'static str>, &'static str);

/// The overview's tables and their columns.
pub const TABLE_COLUMNS: [(&str, &[Column]); 2] = [
    (
        "parts",
        &[
            ("name", None, "Part"),
            ("kind", Some("badge"), "Kind"),
            ("count", Some("number"), "Copies"),
            ("on_border", None, "On the border"),
        ],
    ),
    (
        "reach",
        &[
            ("from_name", None, "From"),
            ("to_name", None, "To"),
            ("carries", Some("badge"), "Carries"),
            ("from_kind", Some("badge"), "From kind"),
            ("to_kind", Some("badge"), "To kind"),
        ],
    ),
];

/// A YAML scalar: plain when it reads back as the same string, else a JSON string.
fn q(s: &str) -> String {
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

/// An expression as a YAML scalar: a number or a boolean bare, anything else quoted as needed.
fn expr(e: &str) -> String {
    if e.parse::<f64>().is_ok() || e == "true" || e == "false" {
        e.to_string()
    } else if e.starts_with('\'') {
        Value::from(e).to_string()
    } else {
        q(e)
    }
}

fn flow(pairs: &[(String, String)]) -> String {
    let inner: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{k}: {}", expr(v)))
        .collect();
    format!("{{{}}}", inner.join(", "))
}

fn column(c: &Value) -> String {
    match c {
        Value::String(s) => s.clone(),
        Value::Object(o) => {
            let mut parts = Vec::new();
            for k in ["field", "as", "label"] {
                if let Some(v) = o.get(k).and_then(Value::as_str) {
                    parts.push(format!("{k}: {}", q(v)));
                }
            }
            format!("{{{}}}", parts.join(", "))
        }
        other => other.to_string(),
    }
}

fn action(a: &Action) -> String {
    let mut parts = vec![format!("name: {}", a.name), format!("does: {}", a.command)];
    if !a.bind.is_empty() {
        parts.push(format!("bind: {}", flow(&a.bind)));
    }
    parts.push(format!("label: {}", q(&a.label)));
    parts.push(format!(
        "visible: {}",
        Value::from(format!("actor.may({})", a.command))
    ));
    format!("{{{}}}", parts.join(", "))
}

/// The pages a navigation entry stands for: a part page, or every page of a nested level.
fn entry_pages(plan: &Plan<'_>, e: &Entry, out: &mut Vec<(String, String)>) {
    match e {
        Entry::Page(p) => {
            let label = plan
                .levels
                .iter()
                .flat_map(|l| &l.pages)
                .find(|x| x.page == *p)
                .map(|x| x.label.clone())
                .unwrap_or_else(|| p.clone());
            out.push((p.clone(), label));
        }
        Entry::Nested(i) => {
            let lp = &plan.levels[*i];
            out.push((lp.overview.clone(), format!("{} overview", lp.title)));
            for (_, _, entries) in &lp.sections {
                for e in entries {
                    entry_pages(plan, e, out);
                }
            }
        }
    }
}

/// A navigation section: its name, its label and its entries (page, label).
type NavSection = (String, String, Vec<(String, String)>);

/// The navigation sections of the whole document: the top level's, a nested level's inside the
/// section of the part that opens it.
fn navigation(plan: &Plan<'_>) -> Vec<NavSection> {
    let top = &plan.levels[0];
    let mut out = vec![(
        "architecture".to_string(),
        top.title.clone(),
        vec![(top.overview.clone(), "Overview".to_string())],
    )];
    for (name, label, entries) in &top.sections {
        let mut pages = Vec::new();
        for e in entries {
            entry_pages(plan, e, &mut pages);
        }
        out.push((name.clone(), label.clone(), pages));
    }
    out
}

fn instance_state(y: &mut String, n: u64, indent: &str) {
    let _ = writeln!(y, "{indent}state:");
    let _ = writeln!(
        y,
        "{indent}  instance: {{type: integer, class: page_state, store: url, default: 1}}"
    );
    let _ = n;
}

fn instance_bar(y: &mut String, n: u64) {
    y.push_str("      - name: instance_bar\n        component: filter_bar\n        binds: [state.instance]\n        choices:\n");
    let options: Vec<String> = (1..=n)
        .map(|i| format!("{{value: {i}, label: \"{i}\"}}"))
        .collect();
    let _ = writeln!(
        y,
        "          - {{name: instance, component: choice, binds: state.instance, placeholder: Instance, options: [{}]}}",
        options.join(", ")
    );
}

fn section(y: &mut String, s: &Section, plan: &Plan<'_>, channel: Option<&str>) {
    let _ = writeln!(y, "      - name: {}", s.name);
    let _ = writeln!(y, "        title: {}", q(&s.title));
    if s.row_actions.is_empty() {
        let _ = writeln!(y, "        component: {}", s.widget);
        let _ = writeln!(y, "        args: {}", flow(&s.args));
        return;
    }
    let p = plan.panels.get(&s.widget).expect("a placed panel exists");
    let args: BTreeMap<&str, &str> = s
        .args
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let params: Vec<(String, String)> = p
        .read_params
        .iter()
        .filter_map(|(k, v)| {
            v.strip_prefix("args.")
                .and_then(|a| args.get(a))
                .map(|x| (k.clone(), x.to_string()))
        })
        .collect();
    y.push_str("        component: collection\n        style: table\n");
    let mut reads = format!("view: {}, key: {}", p.widget, p.key);
    if !params.is_empty() {
        let _ = write!(reads, ", params: {}", flow(&params));
    }
    let _ = writeln!(y, "        reads: {{{reads}}}");
    if let (false, Some(c)) = (p.events.is_empty(), channel) {
        let _ = writeln!(y, "        live:\n          channel: {c}\n          on:");
        for e in &p.events {
            let _ = writeln!(y, "            - {e}");
        }
        y.push_str("          effect: refetch\n          coalesce: 1s\n");
    }
    y.push_str("        columns:\n");
    for c in &p.columns {
        let _ = writeln!(y, "          - {}", column(c));
    }
    y.push_str("        row_actions:\n");
    for a in &s.row_actions {
        let _ = writeln!(y, "          - {}", action(a));
    }
}

/// The `ess-ui/1` document.
pub fn document(plan: &Plan<'_>, name: &str, fixtures: &str) -> String {
    let top = &plan.tree.levels[0];
    let mut y = String::new();
    let _ = writeln!(
        y,
        "# Generated by `agenticspec ui derive` from {}. Never edited by hand: derive it again.",
        top.file
    );
    let _ = writeln!(
        y,
        "format: ess-ui/1\napp: {}\ntitle: {}",
        super::snake(name),
        q(&top.name)
    );
    let _ = writeln!(y, "model: {}", q(plan.model.system()));
    y.push_str("actor: from_session\nplacement_profile: fat\n");
    let _ = writeln!(
        y,
        "fixtures: {{dir: {fixtures}, index: {fixtures}/index.yaml}}\n"
    );
    y.push_str("shells:\n  app:\n    regions:\n      nav: {kind: navigation}\n      main: {kind: page_outlet}\n\n");

    y.push_str("navigation:\n");
    let _ = writeln!(y, "  home: {}", plan.home);
    y.push_str("  sections:\n");
    for (sname, label, pages) in navigation(plan) {
        let names: Vec<&str> = pages.iter().map(|(p, _)| p.as_str()).collect();
        let _ = writeln!(
            y,
            "    - {{name: {sname}, label: {}, pages: [{}]}}",
            q(&label),
            names.join(", ")
        );
    }

    y.push_str("\npage_kinds:\n  overview:\n    purpose: one level of the architecture - its parts, its links and its graph\n  part:\n    purpose: the panels of one part - its instances first, then each domain it uses\n");

    let channels: Vec<(&str, &BTreeSet<String>)> = plan
        .levels
        .iter()
        .filter_map(|l| l.channel.as_deref().map(|c| (c, &l.events)))
        .collect();
    if !channels.is_empty() {
        y.push_str("\nchannels:\n");
        for (c, events) in channels {
            let _ = writeln!(y, "  {c}:\n    carries:\n      events:");
            for e in events {
                let _ = writeln!(y, "        - {e}");
            }
            y.push_str("    direction: server_to_client\n    delivery: every_event\n    resume: from_last_seen\n    scope: param\n    session: {per: architecture_id}\n    lifecycle: [connecting, live, reconnecting, stale, closed]\n");
        }
    }

    let used: BTreeSet<&str> = plan
        .levels
        .iter()
        .flat_map(|l| &l.pages)
        .flat_map(|p| &p.frames)
        .flat_map(|f| &f.sections)
        .filter(|s| s.row_actions.is_empty())
        .map(|s| s.widget.as_str())
        .collect();
    if !used.is_empty() {
        y.push_str("\nwidgets:\n");
        for p in &plan.panels.list {
            if used.contains(p.widget.as_str()) {
                y.push_str(&p.text);
            }
        }
    }

    y.push_str("\npages:\n");
    for lp in &plan.levels {
        let level = &plan.tree.levels[lp.level];
        overview(&mut y, plan, lp, level);
        for page in &lp.pages {
            if page.frames.is_empty() {
                continue;
            }
            let _ = writeln!(y, "  {}:", page.page);
            let _ = writeln!(
                y,
                "    kind: part\n    title: {}\n    nav: {{label: {}}}",
                q(&page.label),
                q(&page.label)
            );
            if lp.instances > 1 {
                instance_state(&mut y, lp.instances, "    ");
            }
            if !page.actions.is_empty() || lp.instances > 1 {
                y.push_str("    header:\n");
                if !page.actions.is_empty() {
                    y.push_str("      actions:\n");
                    for a in &page.actions {
                        let _ = writeln!(y, "        - {}", action(a));
                    }
                }
                if lp.instances > 1 {
                    y.push_str("      filters: instance_bar\n");
                }
            }
            y.push_str("    layout:\n      areas:\n        grid: [");
            let mut grid: Vec<String> = page
                .frames
                .iter()
                .map(|f| format!("[{}]", f.area))
                .collect();
            if lp.instances > 1 {
                grid.insert(0, "[area_bar]".into());
            }
            y.push_str(&grid.join(", "));
            y.push_str("]\n        place:\n");
            if lp.instances > 1 {
                y.push_str("          area_bar: [instance_bar]\n");
            }
            for f in &page.frames {
                let names: Vec<&str> = f.sections.iter().map(|s| s.name.as_str()).collect();
                match &f.title {
                    None => {
                        let _ = writeln!(y, "          {}: [{}]", f.area, names.join(", "));
                    }
                    Some(t) => {
                        let _ = writeln!(
                            y,
                            "          {}: {{sections: [{}], frame: shared, title: {}{}}}",
                            f.area,
                            names.join(", "),
                            q(t),
                            if f.collapsed { ", collapsed: true" } else { "" }
                        );
                    }
                }
            }
            y.push_str("    sections:\n");
            if lp.instances > 1 {
                instance_bar(&mut y, lp.instances);
            }
            for f in &page.frames {
                for s in &f.sections {
                    section(&mut y, s, plan, lp.channel.as_deref());
                }
            }
        }
    }
    y
}

fn overview(y: &mut String, plan: &Plan<'_>, lp: &super::LevelPlan, level: &Level) {
    let _ = writeln!(y, "  {}:", lp.overview);
    let label = if level.path.is_empty() {
        "Overview".to_string()
    } else {
        format!("{} overview", lp.title)
    };
    let _ = writeln!(
        y,
        "    kind: overview\n    title: {}\n    nav: {{label: {}}}",
        q(&lp.title),
        q(&label)
    );
    y.push_str("    state:\n      reach_part: {type: string, class: page_state, store: memory}\n");
    if lp.instances > 1 {
        y.push_str("      instance: {type: integer, class: page_state, store: url, default: 1}\n    header:\n      filters: instance_bar\n");
    }
    y.push_str("    sections:\n");
    if lp.instances > 1 {
        instance_bar(y, lp.instances);
    }
    let lit_name = lit(&level.name);
    for (table, cols) in TABLE_COLUMNS {
        let hidden = lp.key.hide_columns.get(table).cloned().unwrap_or_default();
        let labels = lp.key.label_columns.get(table).cloned().unwrap_or_default();
        let _ = writeln!(y, "      - name: {table}");
        if table == "parts" {
            y.push_str(
                "        title: Parts\n        component: collection\n        style: table\n",
            );
            y.push_str(
                "        reads: {view: agentic.architecture.Parts, key: part_id, paging: none}\n",
            );
        } else {
            y.push_str(
                "        title: Reach\n        component: collection\n        style: table\n",
            );
            let _ = writeln!(
                y,
                "        reads: {{view: agentic.architecture.Reach, key: link_id, params: {{architecture_id: {}}}, paging: none, filter: \"row.from_name contains state.reach_part or row.to_name contains state.reach_part\"}}",
                Value::from(lit_name.clone())
            );
        }
        y.push_str("        columns:\n");
        for (field, as_, label) in cols.iter() {
            if hidden.contains(*field) {
                continue;
            }
            let label = labels
                .get(*field)
                .cloned()
                .unwrap_or_else(|| label.to_string());
            match as_ {
                Some(a) => {
                    let _ = writeln!(
                        y,
                        "          - {{field: {field}, as: {a}, label: {}}}",
                        q(&label)
                    );
                }
                None => {
                    let _ = writeln!(y, "          - {{field: {field}, label: {}}}", q(&label));
                }
            }
        }
        if table == "reach" {
            y.push_str("        states:\n          empty: {message: No link matches.}\n");
        }
    }
    let suffix = pascal(&level.path);
    let _ = writeln!(
        y,
        "      - name: graph\n        title: Graph\n        host: DraftGraph{suffix}\n        reason: draws the parts and links of this level on a canvas at natural size with pan, zoom and fit, each nested architecture as one box; no ess-ui composite lays out and draws an architecture graph\n        stands_in_for: none\n        reads: {{placeholder: agentic.architecture.Graph, fixture: graph.json, params: {{architecture: {}}}}}",
        Value::from(lit_name.clone())
    );
    let _ = writeln!(
        y,
        "      - name: settings\n        title: Policies and marked places\n        host: ArchitectureSettings{suffix}\n        reason: shows the policies and the marked places of the document read-only; no view of the blocks holds them\n        stands_in_for: none\n        reads: {{placeholder: agentic.architecture.Settings, fixture: settings.json, params: {{architecture: {}}}}}",
        Value::from(lit_name)
    );
    let _ = plan;
}

/// A stable id of a fixture row: a UUID shape from the SHA-256 of `what`.
fn id(what: &str) -> String {
    use sha2::{Digest, Sha256};
    let h: String = Sha256::digest(what.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("00000000-0000-4000-8000-{h}")
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default() + "\n"
}

/// Every event payload of the panels' channel scripts, by event.
pub fn scripts(dir: &Path) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let Ok(doc) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        for e in doc["events"].as_array().into_iter().flatten() {
            if let Some(name) = e["event"].as_str() {
                out.entry(name.to_string())
                    .or_insert_with(|| e["payload"].clone());
            }
        }
    }
    out
}

/// The marked places of a document (`{ value, vary }`), each with its value and its range.
fn marked(v: &Value, path: &str, out: &mut Vec<(String, String, String)>) {
    match v {
        Value::Object(o) if o.contains_key("vary") => {
            let range = match &o["vary"] {
                Value::Array(xs) => xs.iter().map(model::text).collect::<Vec<_>>().join(", "),
                Value::Object(r) => {
                    let mut s = format!(
                        "{} to {}",
                        model::text(r.get("min").unwrap_or(&Value::Null)),
                        model::text(r.get("max").unwrap_or(&Value::Null))
                    );
                    if let Some(step) = r.get("step") {
                        s.push_str(&format!(" by {}", model::text(step)));
                    }
                    s
                }
                other => model::text(other),
            };
            out.push((path.to_string(), model::text(v), range));
        }
        Value::Object(o) => {
            for (k, x) in o {
                marked(x, &format!("{path}.{k}"), out);
            }
        }
        Value::Array(xs) => {
            for (i, x) in xs.iter().enumerate() {
                marked(x, &format!("{path}.{i}"), out);
            }
        }
        _ => {}
    }
}

/// The fixtures beside the document, by their path below its fixtures directory.
pub fn fixtures(
    plan: &Plan<'_>,
    to_panels: &str,
    scripts: &BTreeMap<String, Value>,
) -> Vec<(String, String)> {
    let kinds = plan.model.variants("agentic.architecture.PartKind");
    let carries = plan.model.variants("agentic.architecture.Carries");
    let mut parts = Vec::new();
    let mut reach = Vec::new();
    let mut graphs = Vec::new();
    let mut settings = Vec::new();
    for lp in &plan.levels {
        let level = &plan.tree.levels[lp.level];
        let arch_id = id(&format!("architecture {}", level.name));
        let part_id = |name: &str| id(&format!("part {} {name}", level.name));
        for name in &lp.order {
            let p = level.part(name).expect("a part of the level");
            let mut row = Map::new();
            row.insert("part_id".into(), json!(part_id(name)));
            row.insert("architecture_id".into(), json!(arch_id));
            row.insert("name".into(), json!(name));
            let kind = pascal(&p.block);
            if kinds.contains(&kind) {
                row.insert("kind".into(), json!(kind));
            }
            row.insert("count".into(), json!(p.count));
            if let Some(per) = &p.per {
                row.insert("per_part_id".into(), json!(part_id(per)));
            }
            if let Some(a) = &p.acts_for {
                row.insert("acts_for_part_id".into(), json!(part_id(a)));
            }
            let s: Vec<Value> = p
                .settings
                .iter()
                .map(|(k, v)| json!({"name": k, "value": model::text(v)}))
                .collect();
            row.insert("settings".into(), Value::Array(s));
            row.insert("on_border".into(), json!(level.border.contains(name)));
            parts.push(Value::Object(row));
        }
        for (i, l) in level.links.iter().enumerate() {
            let mut row = Map::new();
            row.insert(
                "link_id".into(),
                json!(id(&format!("link {} {i}", level.name))),
            );
            row.insert("architecture_id".into(), json!(arch_id));
            row.insert("from_part_id".into(), json!(part_id(&l.from)));
            row.insert("from_name".into(), json!(l.from));
            row.insert("to_part_id".into(), json!(part_id(&l.to)));
            row.insert("to_name".into(), json!(l.to));
            let c = pascal(&l.carries);
            row.insert(
                "carries".into(),
                json!(if carries.contains(&c) {
                    c
                } else {
                    l.carries.clone()
                }),
            );
            if let Some(o) = &l.over {
                row.insert("over_part_id".into(), json!(part_id(o)));
            }
            for (field, part) in [("from_kind", &l.from), ("to_kind", &l.to)] {
                if let Some(p) = level.part(part) {
                    let k = pascal(&p.block);
                    if kinds.contains(&k) {
                        row.insert(field.into(), json!(k));
                    }
                }
            }
            reach.push(Value::Object(row));
        }
        let mut g = agenticspec_render::architecture::graph(&level.doc);
        if let Some(o) = g.as_object_mut() {
            o.insert("architecture".into(), json!(level.name));
            o.insert("draft_id".into(), json!(level.name));
            o.insert("revision".into(), json!(1));
        }
        graphs.push(g);
        for (k, v) in level.doc["policies"].as_object().into_iter().flatten() {
            let mut found = Vec::new();
            marked(v, &format!("policies.{k}"), &mut found);
            if found.is_empty() {
                settings.push(json!({"architecture": level.name, "place": format!("policies.{k}"), "value": model::text(v), "search": ""}));
            }
            for (place, value, range) in found {
                settings.push(json!({"architecture": level.name, "place": place, "value": value, "search": range}));
            }
        }
        let mut found = Vec::new();
        marked(&level.doc["parts"], "parts", &mut found);
        marked(&level.doc["links"], "links", &mut found);
        for (place, value, range) in found {
            settings.push(json!({"architecture": level.name, "place": place, "value": value, "search": range}));
        }
    }

    let mut files = Vec::new();
    let mut index = String::from("views:\n");
    index.push_str(
        "  agentic.architecture.Parts: parts.json\n  agentic.architecture.Reach: reach.json\n",
    );
    let views: BTreeSet<&str> = plan
        .levels
        .iter()
        .flat_map(|l| &l.pages)
        .flat_map(|p| &p.frames)
        .flat_map(|f| &f.sections)
        .map(|s| s.widget.as_str())
        .collect();
    for v in &views {
        let _ = writeln!(index, "  {v}: {to_panels}/{v}.json");
    }
    let channels: Vec<(&str, &BTreeSet<String>)> = plan
        .levels
        .iter()
        .filter_map(|l| l.channel.as_deref().map(|c| (c, &l.events)))
        .collect();
    if !channels.is_empty() {
        index.push_str("scripts:\n");
        for (c, events) in &channels {
            let _ = writeln!(index, "  {c}: scripts/{c}.json");
            let mut entries = Vec::new();
            for e in events.iter() {
                if let Some(p) = scripts.get(e) {
                    entries.push(
                        json!({"at": format!("{}s", entries.len() + 1), "event": e, "payload": p}),
                    );
                }
            }
            files.push((
                format!("scripts/{c}.json"),
                pretty(&json!({"channel": c, "events": entries})),
            ));
        }
    }
    files.insert(0, ("index.yaml".into(), index));
    files.push((
        "parts.json".into(),
        pretty(&json!({"view": "agentic.architecture.Parts", "rows": parts})),
    ));
    files.push((
        "reach.json".into(),
        pretty(&json!({"view": "agentic.architecture.Reach", "rows": reach})),
    ));
    files.push((
        "graph.json".into(),
        pretty(&json!({"view": "agentic.architecture.Graph", "rows": graphs})),
    ));
    files.push((
        "settings.json".into(),
        pretty(&json!({"view": "agentic.architecture.Settings", "rows": settings})),
    ));
    files
}

/// An argument expression as a JSON value for the layout: a literal as its value, a word as a
/// string, a state or parameter as null (the host gives it).
fn arg_value(e: &str) -> Value {
    if let Some(s) = e.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        return json!(s);
    }
    if let Ok(n) = e.parse::<i64>() {
        return json!(n);
    }
    if e.contains('.') {
        return Value::Null;
    }
    json!(e)
}

fn action_json(a: &Action) -> Value {
    let bind: Map<String, Value> = a
        .bind
        .iter()
        .map(|(k, v)| (k.clone(), json!(v.strip_prefix("row.").unwrap_or(v))))
        .collect();
    json!({"name": a.name, "command": a.command, "label": a.label, "bind": bind})
}

/// The composition as the view package reads it (`agenticspec-ui-layout/1`).
pub fn layout(plan: &Plan<'_>) -> String {
    let mut levels = Vec::new();
    for lp in &plan.levels {
        let level = &plan.tree.levels[lp.level];
        let parts: Vec<Value> = lp
            .order
            .iter()
            .filter_map(|n| level.part(n))
            .map(|p| {
                let mut o = json!({
                    "name": p.name,
                    "path": p.path,
                    "label": lp.key.label_parts.get(&p.name).cloned().unwrap_or_else(|| p.name.clone()),
                    "block": p.block,
                    "count": p.count,
                });
                match p.nested {
                    Some(i) => o["nested"] = json!(plan.tree.levels[i].path),
                    None => o["page"] = json!(super::part_page(&p.path)),
                }
                o
            })
            .collect();
        let pages: Vec<Value> = lp
            .pages
            .iter()
            .filter(|p| !p.frames.is_empty())
            .map(|p| {
                let frames: Vec<Value> = p
                    .frames
                    .iter()
                    .map(|f| {
                        let sections: Vec<Value> = f
                            .sections
                            .iter()
                            .map(|s| {
                                let args: Map<String, Value> = s
                                    .args
                                    .iter()
                                    .filter(|(k, _)| !matches!(k.as_str(), "architecture" | "part" | "instance" | "source"))
                                    .map(|(k, v)| (k.clone(), arg_value(v)))
                                    .collect();
                                let part = s
                                    .args
                                    .iter()
                                    .find(|(k, _)| k == "part")
                                    .map(|(_, v)| arg_value(v))
                                    .unwrap_or(Value::Null);
                                json!({
                                    "name": s.name,
                                    "title": s.title,
                                    "widget": s.widget,
                                    "part": part,
                                    "args": args,
                                    "row_actions": s.row_actions.iter().map(action_json).collect::<Vec<_>>(),
                                })
                            })
                            .collect();
                        json!({"title": f.title, "collapsed": f.collapsed, "sections": sections})
                    })
                    .collect();
                json!({
                    "part": p.part,
                    "page": p.page,
                    "label": p.label,
                    "block": p.block,
                    "actions": p.actions.iter().map(action_json).collect::<Vec<_>>(),
                    "frames": frames,
                })
            })
            .collect();
        levels.push(json!({
            "path": level.path,
            "name": level.name,
            "overview": lp.overview,
            "channel": lp.channel.clone().unwrap_or_else(|| channel_name(level)),
            "instances": lp.instances,
            "parts": parts,
            "pages": pages,
        }));
    }
    pretty(&json!({
        "format": "agenticspec-ui-layout/1",
        "architecture": plan.tree.levels[0].name,
        "home": plan.home,
        "levels": levels,
    }))
}
