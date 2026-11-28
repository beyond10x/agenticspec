//! An architecture (`agenticspec-architecture/1`) as a render model page: parts as nodes in
//! groups by block layer, the parts of a `per` part inside the card of the part they belong to,
//! the border parts of a nested architecture as ports, links as edges, marked places as marks.

use super::model::{Architecture, Outcome, Part, Place, norm, range_text};
use crate::fmt;
use crate::graph::{Chip, Decor, NodeDecor};
use crate::page::{Badge, Direction, Edge, Level, Mark, Node, Page, Port, Region, Target};

/// Colour family and row of a block.
pub fn family(block: &str) -> (&'static str, usize) {
    match block {
        "principal" | "delegation" => ("principal", 0),
        "agent" | "harness" => ("agent", 1),
        "channel" | "question" => ("collab", 2),
        "work" | "process" | "authority" => ("work", 2),
        "gateway" | "model" => ("intel", 3),
        "tool" | "knowledge" => ("capability", 4),
        "workspace" | "sandbox" | "runtime" => ("place", 5),
        "pool" | "resource" => ("resource", 5),
        "architecture" => ("nested", 2),
        _ => ("other", 6),
    }
}

/// The colour families, in legend order, with what they hold and their row.
pub const FAMILIES: [(&str, &str, u32); 10] = [
    ("principal", "principal", 0),
    ("agent", "agent, harness", 1),
    ("collab", "channel, question", 2),
    ("work", "work, authority", 2),
    ("intel", "gateway, model", 3),
    ("capability", "tool, knowledge", 4),
    ("place", "workspace, sandbox, runtime", 5),
    ("resource", "pool, resource", 5),
    ("nested", "nested architecture", 2),
    ("other", "other block", 6),
];

/// The icon of a block.
fn icon(block: &str) -> &'static str {
    match block {
        "principal" | "delegation" => "person",
        "agent" | "harness" => "bot",
        "channel" | "question" => "bubble",
        "gateway" => "gateway",
        "model" => "chip",
        "tool" | "knowledge" => "wrench",
        "workspace" => "folder",
        "sandbox" => "shield",
        "runtime" => "play",
        "pool" | "resource" => "stack",
        "work" | "process" => "checklist",
        "authority" => "lock",
        "architecture" => "frame",
        _ => "circle",
    }
}

/// The subtitle of a part: `pool · build_slots`.
pub fn subtitle(p: &Part) -> String {
    if let Some(n) = &p.nested {
        return match (&n.name, n.parts, &n.reference) {
            (Some(name), Some(c), _) => {
                format!("{name} · {c} part{}", if c == 1 { "" } else { "s" })
            }
            (_, _, Some(r)) => r.clone(),
            _ => p.block.clone(),
        };
    }
    match (&p.kind, sub_prop(p)) {
        (Some(k), _) => format!("{} · {}", p.block, k),
        (None, Some(v)) => format!("{} · {}", p.block, v),
        (None, None) => p.block.clone(),
    }
}

/// The plain property that reads in the subtitle when a part has no kind.
fn sub_prop(p: &Part) -> Option<&str> {
    sub_key(p).and_then(|key| {
        p.props
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    })
}

fn sub_key(p: &Part) -> Option<&'static str> {
    if p.kind.is_some() {
        return None;
    }
    SUB_KEYS
        .iter()
        .copied()
        .find(|key| p.props.iter().any(|(k, _)| k == key))
}

const SUB_KEYS: [&str; 6] = [
    "role",
    "harness",
    "sharing",
    "isolation",
    "route",
    "provider",
];

/// The accessible name of a part's card.
fn label_for(arch: &Architecture, p: &Part) -> String {
    let mut s = format!("{}, {}", p.name, subtitle(p));
    if let Some(c) = &p.count {
        s.push_str(&format!(", count {c}"));
    }
    if let Some(per) = &p.per {
        s.push_str(&format!(", one per {per}"));
    }
    if arch.border.contains(&p.name) {
        s.push_str(", on the border");
    }
    if !p.places.is_empty() {
        s.push_str(&format!(", {} marked places", p.places.len()));
    }
    if let Some(n) = p.nested.as_ref().filter(|n| !n.ports.is_empty()) {
        s.push_str(&format!(", ports {}", n.ports.join(", ")));
    }
    s
}

/// The part a `per` names, as the graph resolves it: the part itself, or the nested architecture
/// whose port it addresses.
fn resolve<'a>(arch: &'a Architecture, name: &str) -> Option<&'a Part> {
    arch.part(name).or_else(|| {
        let (head, _) = name.split_once('.')?;
        arch.parts
            .iter()
            .find(|p| p.name == head && p.nested.as_ref().is_some_and(|n| !n.ports.is_empty()))
    })
}

/// A marked place as a chip: its field, its value (at baseline when a result holds one), the best
/// value of the result and the search range.
pub fn chip(place: &Place, outcome: Option<&Outcome>) -> Chip {
    let mut value = fmt::value(&place.value);
    let mut best = None;
    let mut range = place.range();
    if let Some(o) = outcome {
        let (b, w) = o.at(place);
        if let Some(b) = b {
            value = fmt::value(b);
        }
        best = w.map(fmt::value);
        if range.is_empty()
            && let Some(m) = o.marked.iter().find(|m| norm(&m.path) == norm(&place.path))
        {
            range = range_text(&m.range);
        }
    }
    Chip {
        field: place.field.clone(),
        value: Some(value),
        best,
        range,
        detail: String::new(),
        id: Some(place.path.clone()),
        refusal: false,
    }
}

/// The page of an architecture, with the marks of a result when given, and what the
/// architecture page draws beyond it: subtitles, icons, properties and the border of each part,
/// the row of each block layer, the part a link passes over, the values at marked places.
pub fn to_page(arch: &Architecture, o: Option<&Outcome>) -> (Page, Decor) {
    let mut page = Page::new(&arch.name);
    if !arch.summary.is_empty() {
        page.summary = Some(arch.summary.clone());
    }
    let mut decor = Decor {
        groups: FAMILIES
            .iter()
            .map(|(id, label, rank)| (id.to_string(), label.to_string(), *rank))
            .collect(),
        ..Decor::default()
    };
    for p in &arch.parts {
        // A nested architecture is on the border when one of its ports is.
        let border = arch.border.contains(&p.name)
            || p.nested
                .as_ref()
                .is_some_and(|n| n.ports.iter().any(|a| arch.border.contains(a)));
        let mut properties = Vec::new();
        if let Some(a) = &p.acts_for {
            properties.push(("acts for".to_string(), a.clone()));
        }
        let in_sub = sub_key(p);
        for (k, v) in p.props.iter().filter(|(k, _)| Some(k.as_str()) != in_sub) {
            properties.push((k.replace('_', " "), v.clone()));
        }
        page.nodes.push(Node {
            id: p.name.clone(),
            label: p.name.clone(),
            group: family(&p.block).0.to_string(),
            badges: p
                .count
                .iter()
                .map(|c| Badge {
                    name: "count".to_string(),
                    value: c.clone(),
                })
                .collect(),
            region: p
                .per
                .as_deref()
                .and_then(|x| resolve(arch, x))
                .map(|h| h.name.clone())
                .or_else(|| p.per.clone()),
        });
        decor.nodes.push((
            p.name.clone(),
            NodeDecor {
                subtitle: Some(subtitle(p)),
                icon: Some(icon(&p.block)),
                properties,
                outline: border.then(|| "border".to_string()),
                description: Some(label_for(arch, p)),
            },
        ));
        for address in p.nested.iter().flat_map(|n| n.ports.iter()) {
            let own = address
                .strip_prefix(&p.name)
                .and_then(|r| r.strip_prefix('.'))
                .unwrap_or(address);
            page.ports.push(Port {
                node: p.name.clone(),
                name: own.to_string(),
                kind: "border part".to_string(),
                direction: Direction::Both,
                contract: None,
                via: None,
                unresolved: false,
            });
        }
        for pl in &p.places {
            let c = chip(pl, o);
            page.marks.push(Mark {
                target: Target {
                    node: p.name.clone(),
                    port: None,
                },
                name: c.field.clone(),
                detail: (!c.range.is_empty()).then(|| c.range.clone()),
                level: Level::Info,
            });
            decor.chips.push(Some(c));
        }
    }
    // One region per part that others are one per, nested as their own `per` nests them.
    let hosts: Vec<&Part> = arch
        .parts
        .iter()
        .filter(|h| {
            arch.parts.iter().any(|p| {
                p.per
                    .as_deref()
                    .and_then(|x| resolve(arch, x))
                    .is_some_and(|r| r.name == h.name && p.name != h.name)
            })
        })
        .collect();
    let within = |h: &Part| -> Option<String> {
        let up = h.per.as_deref().and_then(|x| resolve(arch, x))?;
        hosts
            .iter()
            .any(|x| x.name == up.name && up.name != h.name)
            .then(|| up.name.clone())
    };
    fn region(name: &str, hosts: &[&Part], within: &dyn Fn(&Part) -> Option<String>) -> Region {
        Region {
            id: name.to_string(),
            label: name.to_string(),
            r#in: None,
            per: Some(name.to_string()),
            regions: hosts
                .iter()
                .filter(|h| within(h).as_deref() == Some(name))
                .map(|h| region(&h.name, hosts, within))
                .collect(),
        }
    }
    for h in &hosts {
        if within(h).is_none() {
            page.regions.push(region(&h.name, &hosts, &within));
        }
    }
    for l in &arch.links {
        page.edges.push(Edge {
            from: l.from.clone(),
            to: l.to.clone(),
            label: l.carries.clone(),
            count: None,
        });
        decor.vias.push(l.over.clone());
    }
    (page, decor)
}
