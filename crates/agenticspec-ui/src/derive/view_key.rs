//! The `view:` key of an architecture document: presentation only. It may hold `home`, `order`
//! (of parts and of panels), `labels` (of parts, panels and the overview's table columns), `hide`
//! (panels, overview columns, actions) and `groups` (display groups of parts, labels only). An
//! entry that names a part, panel, column or action that does not exist, or that holds a read, a
//! filter, a field list or a command, is refused with the entry's node path.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::Refusal;

/// The words that would make the key more than presentation.
const NOT_PRESENTATION: [(&str, &str); 7] = [
    ("reads", "a read"),
    ("view", "a read"),
    ("filter", "a filter"),
    ("fields", "a field list"),
    ("columns_of", "a field list"),
    ("does", "a command"),
    ("command", "a command"),
];

/// The tables of an overview whose columns the key may label or hide.
pub const TABLES: [&str; 2] = ["parts", "reach"];

/// One display group of the overview's navigation.
#[derive(Clone, Debug, Default)]
pub struct Group {
    pub name: String,
    pub label: String,
    pub parts: Vec<String>,
}

/// A checked `view:` key.
#[derive(Clone, Debug, Default)]
pub struct ViewKey {
    pub home: Option<String>,
    pub order_parts: Vec<String>,
    pub order_panels: BTreeMap<String, Vec<String>>,
    pub label_parts: BTreeMap<String, String>,
    pub label_panels: BTreeMap<String, String>,
    pub label_columns: BTreeMap<String, BTreeMap<String, String>>,
    pub hide_panels: BTreeSet<String>,
    pub hide_columns: BTreeMap<String, BTreeSet<String>>,
    pub hide_actions: BTreeSet<String>,
    pub groups: Vec<Group>,
}

/// What the key's entries may name, on one document's pages.
pub struct Names<'a> {
    /// The part paths of the document, relative to it.
    pub parts: &'a BTreeSet<String>,
    /// The panels (widget names) any page of the document places.
    pub panels: &'a BTreeSet<String>,
    /// The panels of a block view that a part's page places, by part.
    pub panels_of: &'a BTreeMap<String, BTreeSet<String>>,
    /// The columns of each overview table.
    pub columns: &'a BTreeMap<String, BTreeSet<String>>,
}

fn refusal(path: &str, message: impl Into<String>) -> Refusal {
    Refusal {
        path: path.to_string(),
        message: message.into(),
    }
}

fn strings(v: &Value, path: &str, out: &mut Vec<Refusal>) -> Vec<String> {
    match v {
        Value::Array(xs) => xs
            .iter()
            .enumerate()
            .filter_map(|(i, x)| match x.as_str() {
                Some(s) => Some(s.to_string()),
                None => {
                    out.push(refusal(&format!("{path}/{i}"), "a name is a string"));
                    None
                }
            })
            .collect(),
        _ => {
            out.push(refusal(path, "a list of names"));
            Vec::new()
        }
    }
}

fn map<'v>(
    v: &'v Value,
    path: &str,
    out: &mut Vec<Refusal>,
) -> Option<&'v serde_json::Map<String, Value>> {
    match v {
        Value::Object(o) => Some(o),
        _ => {
            out.push(refusal(path, "a map"));
            None
        }
    }
}

fn only(o: &serde_json::Map<String, Value>, keys: &[&str], path: &str, out: &mut Vec<Refusal>) {
    for k in o.keys() {
        if !keys.contains(&k.as_str()) {
            out.push(refusal(
                &format!("{path}/{k}"),
                format!(
                    "`{k}` is no entry of the view key here; it takes {}",
                    keys.join(", ")
                ),
            ));
        }
    }
}

/// Every key anywhere below `v` that would make the key hold more than presentation.
fn presentation_only(v: &Value, path: &str, out: &mut Vec<Refusal>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                let at = format!("{path}/{k}");
                if let Some((_, what)) = NOT_PRESENTATION.iter().find(|(w, _)| *w == k) {
                    out.push(refusal(
                        &at,
                        format!("the view key holds presentation only, not {what}"),
                    ));
                    continue;
                }
                presentation_only(x, &at, out);
            }
        }
        Value::Array(xs) => {
            for (i, x) in xs.iter().enumerate() {
                presentation_only(x, &format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
}

fn part(p: &str, at: &str, names: &Names<'_>, out: &mut Vec<Refusal>) -> bool {
    if names.parts.contains(p) {
        return true;
    }
    out.push(refusal(at, format!("no part `{p}` in the document")));
    false
}

fn panel(p: &str, at: &str, names: &Names<'_>, out: &mut Vec<Refusal>) -> bool {
    if names.panels.contains(p) {
        return true;
    }
    out.push(refusal(
        at,
        format!("no page of the document places the panel `{p}`"),
    ));
    false
}

fn column(table: &str, field: &str, at: &str, names: &Names<'_>, out: &mut Vec<Refusal>) -> bool {
    match names.columns.get(table) {
        Some(cols) if cols.contains(field) => true,
        Some(_) => {
            out.push(refusal(
                at,
                format!("the table `{table}` has no column `{field}`"),
            ));
            false
        }
        None if names.panels.contains(table) => {
            out.push(refusal(
                at,
                format!(
                    "`{table}` is a block panel, copied byte for byte into every architecture; its columns are named by the block (naming.display), not here"
                ),
            ));
            false
        }
        None => {
            out.push(refusal(
                at,
                format!(
                    "no table `{table}`; the overview's tables are {}",
                    TABLES.join(" and ")
                ),
            ));
            false
        }
    }
}

/// The key at `doc.view`, checked against what the document's pages hold; `actions` are the
/// names of the actions derived for it. An absent key is the empty key.
pub fn read(
    doc: &Value,
    names: &Names<'_>,
    actions: &BTreeSet<String>,
) -> Result<ViewKey, Vec<Refusal>> {
    let mut out = Vec::new();
    let mut key = ViewKey::default();
    let Some(v) = doc.get("view") else {
        return Ok(key);
    };
    let root = "view";
    presentation_only(v, root, &mut out);
    if !out.is_empty() {
        return Err(out);
    }
    let Some(o) = map(v, root, &mut out) else {
        return Err(out);
    };
    only(
        o,
        &["home", "order", "labels", "hide", "groups"],
        root,
        &mut out,
    );

    if let Some(h) = o.get("home") {
        match h.as_str() {
            Some("overview") => key.home = Some("overview".into()),
            Some(p) => {
                if part(p, "view/home", names, &mut out) {
                    key.home = Some(p.to_string());
                }
            }
            None => out.push(refusal("view/home", "`overview` or a part path")),
        }
    }

    if let Some(ord) = o.get("order").and_then(|x| map(x, "view/order", &mut out)) {
        only(ord, &["parts", "panels"], "view/order", &mut out);
        if let Some(ps) = ord.get("parts") {
            for (i, p) in strings(ps, "view/order/parts", &mut out)
                .into_iter()
                .enumerate()
            {
                if part(&p, &format!("view/order/parts/{i}"), names, &mut out) {
                    key.order_parts.push(p);
                }
            }
        }
        if let Some(pm) = ord
            .get("panels")
            .and_then(|x| map(x, "view/order/panels", &mut out))
        {
            for (p, list) in pm {
                let at = format!("view/order/panels/{p}");
                if !part(p, &at, names, &mut out) {
                    continue;
                }
                let placed = names.panels_of.get(p).cloned().unwrap_or_default();
                let mut keep = Vec::new();
                for (i, w) in strings(list, &at, &mut out).into_iter().enumerate() {
                    if placed.contains(&w) {
                        keep.push(w);
                    } else {
                        out.push(refusal(
                            &format!("{at}/{i}"),
                            format!("the page of `{p}` places no panel `{w}`"),
                        ));
                    }
                }
                key.order_panels.insert(p.clone(), keep);
            }
        }
    }

    if let Some(l) = o
        .get("labels")
        .and_then(|x| map(x, "view/labels", &mut out))
    {
        only(l, &["parts", "panels", "columns"], "view/labels", &mut out);
        if let Some(pm) = l
            .get("parts")
            .and_then(|x| map(x, "view/labels/parts", &mut out))
        {
            for (p, label) in pm {
                let at = format!("view/labels/parts/{p}");
                match label.as_str() {
                    Some(s) if part(p, &at, names, &mut out) => {
                        key.label_parts.insert(p.clone(), s.to_string());
                    }
                    Some(_) => {}
                    None => out.push(refusal(&at, "a label is a string")),
                }
            }
        }
        if let Some(pm) = l
            .get("panels")
            .and_then(|x| map(x, "view/labels/panels", &mut out))
        {
            for (p, label) in pm {
                let at = format!("view/labels/panels/{p}");
                match label.as_str() {
                    Some(s) if panel(p, &at, names, &mut out) => {
                        key.label_panels.insert(p.clone(), s.to_string());
                    }
                    Some(_) => {}
                    None => out.push(refusal(&at, "a label is a string")),
                }
            }
        }
        if let Some(cm) = l
            .get("columns")
            .and_then(|x| map(x, "view/labels/columns", &mut out))
        {
            for (table, fields) in cm {
                let at = format!("view/labels/columns/{table}");
                let Some(fm) = map(fields, &at, &mut out) else {
                    continue;
                };
                for (f, label) in fm {
                    let fat = format!("{at}/{f}");
                    match label.as_str() {
                        Some(s) if column(table, f, &fat, names, &mut out) => {
                            key.label_columns
                                .entry(table.clone())
                                .or_default()
                                .insert(f.clone(), s.to_string());
                        }
                        Some(_) => {}
                        None => out.push(refusal(&fat, "a label is a string")),
                    }
                }
            }
        }
    }

    if let Some(h) = o.get("hide").and_then(|x| map(x, "view/hide", &mut out)) {
        only(h, &["panels", "columns", "actions"], "view/hide", &mut out);
        if let Some(ps) = h.get("panels") {
            for (i, p) in strings(ps, "view/hide/panels", &mut out)
                .into_iter()
                .enumerate()
            {
                if panel(&p, &format!("view/hide/panels/{i}"), names, &mut out) {
                    key.hide_panels.insert(p);
                }
            }
        }
        if let Some(cm) = h
            .get("columns")
            .and_then(|x| map(x, "view/hide/columns", &mut out))
        {
            for (table, fields) in cm {
                let at = format!("view/hide/columns/{table}");
                for (i, f) in strings(fields, &at, &mut out).into_iter().enumerate() {
                    if column(table, &f, &format!("{at}/{i}"), names, &mut out) {
                        key.hide_columns.entry(table.clone()).or_default().insert(f);
                    }
                }
            }
        }
        if let Some(acts) = h.get("actions") {
            for (i, a) in strings(acts, "view/hide/actions", &mut out)
                .into_iter()
                .enumerate()
            {
                if actions.contains(&a) {
                    key.hide_actions.insert(a);
                } else {
                    out.push(refusal(
                        &format!("view/hide/actions/{i}"),
                        format!("no action `{a}` is derived for the document"),
                    ));
                }
            }
        }
    }

    if let Some(gs) = o.get("groups") {
        match gs {
            Value::Array(xs) => {
                let mut seen = BTreeSet::new();
                for (i, g) in xs.iter().enumerate() {
                    let gname = g.get("name").and_then(Value::as_str).unwrap_or("");
                    let at = if gname.is_empty() {
                        format!("view/groups/{i}")
                    } else {
                        format!("view/groups/{gname}")
                    };
                    let Some(go) = map(g, &at, &mut out) else {
                        continue;
                    };
                    only(go, &["name", "label", "parts"], &at, &mut out);
                    if gname.is_empty()
                        || !gname.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        out.push(refusal(
                            &at,
                            "a group has a `name` of letters, digits and _",
                        ));
                        continue;
                    }
                    if !seen.insert(gname.to_string()) {
                        out.push(refusal(&at, format!("the group `{gname}` is named twice")));
                    }
                    let mut group = Group {
                        name: gname.to_string(),
                        label: go
                            .get("label")
                            .and_then(Value::as_str)
                            .unwrap_or(gname)
                            .to_string(),
                        parts: Vec::new(),
                    };
                    let ps = go.get("parts").cloned().unwrap_or(Value::Array(Vec::new()));
                    for (j, p) in strings(&ps, &format!("{at}/parts"), &mut out)
                        .into_iter()
                        .enumerate()
                    {
                        if part(&p, &format!("{at}/parts/{j}"), names, &mut out) {
                            group.parts.push(p);
                        }
                    }
                    key.groups.push(group);
                }
            }
            _ => out.push(refusal("view/groups", "a list of groups")),
        }
    }

    if out.is_empty() { Ok(key) } else { Err(out) }
}
