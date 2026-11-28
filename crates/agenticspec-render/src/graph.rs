//! A page's graph as an SVG, laid out deterministically.
//!
//! Nodes sit in rows by rank. Between two rows lies a label row, where every edge that crosses it
//! passes through an item of its own width: its label, a node drawn on the edge, or a narrow gap.
//! Items are ordered by barycentre sweeps and placed by isotonic regression, so no two items of a
//! row overlap and every edge label has the room its text needs.
//!
//! A large graph stays legible at its natural size: edges with one label into or out of one card
//! form a fan, with one label beside the card and one trunk through the rows between, which each
//! edge joins on a track of its own label row; a card that several cards of its row send to drops
//! to a row of its own; the passing items of a long line take one x where they fit, and a line
//! leaves a card shorter than its row straight, so no line crosses a card.

use crate::fmt::{esc, mono_w, px, text_w};
use crate::page::{self, Direction, Mark, Page, Region};
use std::fmt::Write as _;

const MARGIN: f64 = 18.0;
const PAD: f64 = 12.0;
const ICON: f64 = 30.0;
const HEAD: f64 = 52.0;
const PROP: f64 = 17.0;
const CHIP: f64 = 25.0;
const MIN_W: f64 = 168.0;
const LINE_GAP: f64 = 12.0;
const ROW_GAP: f64 = 20.0;
const PILL_H: f64 = 20.0;

/// The accessible title and description of the SVG.
#[derive(Default)]
pub(crate) struct Words {
    pub title: String,
    pub desc: String,
}

/// What a page of this crate draws beyond the render model: the rank and label of each group,
/// and per node, edge and mark what its card, line or chip shows besides. [`crate::page::render`]
/// draws with none of it.
#[derive(Default)]
pub(crate) struct Decor {
    pub words: Words,
    /// Group id, legend label, row. Without any, a node's row is the longest chain of edges that
    /// leads to it.
    pub groups: Vec<(String, String, u32)>,
    /// By node id.
    pub nodes: Vec<(String, NodeDecor)>,
    /// By edge index: a node the edge passes through. A node that is only ever passed through,
    /// between nodes of two rows, is drawn on the edge.
    pub vias: Vec<Option<String>>,
    /// By mark index: the chip drawn for the mark.
    pub chips: Vec<Option<Chip>>,
}

#[derive(Default)]
pub(crate) struct NodeDecor {
    /// The line under the label; the group's label without it.
    pub subtitle: Option<String>,
    pub icon: Option<&'static str>,
    /// `key value` lines on the card.
    pub properties: Vec<(String, String)>,
    /// A dashed ring around the card and this text as its tag.
    pub outline: Option<String>,
    /// The accessible name of the card.
    pub description: Option<String>,
}

/// The colour slots of the stylesheet: a card of slot `s` has the class `f-s`.
const PALETTE: [&str; 10] = [
    "principal",
    "agent",
    "collab",
    "work",
    "intel",
    "capability",
    "place",
    "resource",
    "nested",
    "other",
];

/// The colour slot of each group name of a page.
pub(crate) struct Palette {
    extra: Vec<(String, &'static str)>,
}

impl Palette {
    /// A group named as a slot takes that slot; any other name takes the next slot no group of
    /// the page names, in order of first appearance; an empty name is `other`.
    pub fn of(page: &Page, declared: &[(String, String, u32)]) -> Self {
        let names: Vec<&str> = declared
            .iter()
            .map(|g| g.0.as_str())
            .chain(page.nodes.iter().map(|n| n.group.as_str()))
            .collect();
        let free: Vec<&'static str> = PALETTE[..8]
            .iter()
            .copied()
            .filter(|s| !names.contains(s))
            .collect();
        let free = if free.is_empty() {
            PALETTE[..8].to_vec()
        } else {
            free
        };
        let mut extra: Vec<(String, &'static str)> = Vec::new();
        for n in names {
            if n.is_empty() || PALETTE.contains(&n) || extra.iter().any(|(e, _)| e == n) {
                continue;
            }
            let slot = free[extra.len() % free.len()];
            extra.push((n.to_string(), slot));
        }
        Palette { extra }
    }

    pub fn slot(&self, group: &str) -> &'static str {
        if let Some(s) = PALETTE.iter().find(|s| **s == group) {
            return s;
        }
        self.extra
            .iter()
            .find(|(n, _)| n == group)
            .map(|(_, s)| *s)
            .unwrap_or("other")
    }
}

/// Every region of a page with the id of the region it lies in.
fn all_regions(page: &Page) -> Vec<(&Region, Option<String>)> {
    fn walk<'a>(
        r: &'a Region,
        parent: Option<String>,
        out: &mut Vec<(&'a Region, Option<String>)>,
    ) {
        out.push((r, parent));
        for inner in &r.regions {
            walk(inner, Some(r.id.clone()), out);
        }
    }
    let mut out = Vec::new();
    for r in &page.regions {
        walk(r, None, &mut out);
    }
    out
}

/// A 14 by 14 stroke icon by name.
fn icon(name: &str) -> &'static str {
    match name {
        "person" => {
            "M7 6.6a2.4 2.4 0 1 0 0-4.8 2.4 2.4 0 0 0 0 4.8zM2.4 12.6c0-2.6 2-4.2 4.6-4.2s4.6 1.6 4.6 4.2"
        }
        "bot" => {
            "M3.5 4.5h7a1.5 1.5 0 0 1 1.5 1.5v4.5a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 2 10.5V6a1.5 1.5 0 0 1 1.5-1.5zM7 4.5V2M5.2 8.2h.01M8.8 8.2h.01"
        }
        "bubble" => "M2 2.5h10v7H6.5L3.5 12V9.5H2z",
        "gateway" => "M2 2.5v9M5 7h7M9 4l3 3-3 3",
        "chip" => {
            "M4 4h6v6H4zM5.5 1.5V4M8.5 1.5V4M5.5 10v2.5M8.5 10v2.5M1.5 5.5H4M1.5 8.5H4M10 5.5h2.5M10 8.5h2.5"
        }
        "wrench" => {
            "M9.6 1.8a3 3 0 0 0-3.4 4.1L2 10.1 3.9 12l4.2-4.2a3 3 0 0 0 4.1-3.4L10.4 6.2 8.6 5.4 7.8 3.6z"
        }
        "folder" => "M1.5 3.5h4l1.5 1.5h5.5v6.5h-11z",
        "shield" => "M7 1.5l5 2v3.6c0 2.9-2.1 4.7-5 5.4-2.9-.7-5-2.5-5-5.4V3.5z",
        "play" => "M4 2.5v9l7.5-4.5z",
        "stack" => "M2 2.5h10v2.5H2zM2 6.25h10v2.5H2zM2 10h10v2H2z",
        "checklist" => "M2 3.8l1.4 1.4L6 2.6M2 9l1.4 1.4L6 7.8M8 4.2h4.5M8 9.4h4.5",
        "lock" => "M3.5 6.5h7v5.5h-7zM5 6.5V4.8a2 2 0 0 1 4 0v1.7",
        "frame" => "M1.5 2.5h11v9h-11zM4 5h6v4H4zM1.5 7H4M10 7h2.5",
        _ => "M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10z",
    }
}

/// What a chip on a card (or beside the graph) shows: a field, its value and the value it would
/// change to, a range in brackets or a plain detail.
#[derive(Clone, Default)]
pub(crate) struct Chip {
    pub field: String,
    pub value: Option<String>,
    pub best: Option<String>,
    pub range: String,
    pub detail: String,
    /// Written as `data-place`.
    pub id: Option<String>,
    pub refusal: bool,
}

impl Chip {
    /// The chip of a mark of the page.
    fn of(m: &Mark) -> Self {
        Chip {
            field: match &m.target.port {
                Some(p) => format!("{p} · {}", m.name),
                None => m.name.clone(),
            },
            detail: m.detail.clone().unwrap_or_default(),
            refusal: m.level == page::Level::Refusal,
            ..Chip::default()
        }
    }

    pub fn changed(&self) -> bool {
        self.best
            .as_ref()
            .is_some_and(|b| Some(b) != self.value.as_ref())
    }

    fn class(&self) -> &'static str {
        match (self.changed(), self.refusal) {
            (true, _) => "chip changed",
            (false, true) => "chip refusal",
            (false, false) => "chip",
        }
    }

    /// The attributes, each with a leading space.
    fn attrs(&self) -> String {
        let mut s = String::new();
        if let Some(id) = &self.id {
            let _ = write!(s, " data-place=\"{}\"", esc(id));
        }
        let value = self.value.as_deref().unwrap_or("");
        match &self.best {
            Some(b) => {
                let _ = write!(
                    s,
                    " data-baseline=\"{}\" data-best=\"{}\"",
                    esc(value),
                    esc(b)
                );
            }
            None if self.value.is_some() => {
                let _ = write!(s, " data-value=\"{}\"", esc(value));
            }
            None => {}
        }
        s
    }

    /// One line of text: `capacity 16, best 24, range 8…32`, or `name: detail`.
    pub fn describe(&self) -> String {
        let Some(v) = &self.value else {
            return if self.detail.is_empty() {
                self.field.clone()
            } else {
                format!("{}: {}", self.field, self.detail)
            };
        };
        let mut s = format!("{} {v}", self.field);
        if let Some(b) = &self.best {
            s.push_str(&format!(", best {b}"));
        }
        if !self.range.is_empty() {
            s.push_str(&format!(", range {}", self.range));
        }
        s
    }

    fn width(&self) -> f64 {
        let mut w = 9.0 + text_w(&self.field, 11.5, false);
        if let Some(v) = &self.value {
            w += 7.0;
            w += mono_w(v, 11.5);
        }
        if let Some(b) = &self.best {
            w += 6.0 + mono_w("→", 11.5) + 6.0 + mono_w(b, 11.5) + 4.0;
        }
        if !self.range.is_empty() {
            w += 7.0 + mono_w(&format!("[{}]", self.range), 11.0);
        }
        if !self.detail.is_empty() {
            w += 7.0 + mono_w(&self.detail, 11.0);
        }
        w + 9.0
    }

    /// The chip as SVG, its left edge at `x`, its top at `y`.
    fn svg(&self, out: &mut String, x: f64, y: f64) {
        let w = self.width();
        let _ = write!(
            out,
            "<g class=\"{}\"{}><title>{}</title><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{PILL_H}\" rx=\"10\"/>",
            self.class(),
            self.attrs(),
            esc(&self.describe()),
            px(x),
            px(y),
            px(w)
        );
        let base = y + 14.0;
        let mut cx = x + 9.0;
        let _ = write!(
            out,
            "<text class=\"chip-k\" x=\"{}\" y=\"{}\">{}</text>",
            px(cx),
            px(base),
            esc(&self.field)
        );
        cx += text_w(&self.field, 11.5, false);
        let changed = self.changed();
        if let Some(v) = &self.value {
            cx += 7.0;
            let was = if changed { "chip-v was" } else { "chip-v" };
            let _ = write!(
                out,
                "<text class=\"{was}\" x=\"{}\" y=\"{}\">{}</text>",
                px(cx),
                px(base),
                esc(v)
            );
            cx += mono_w(v, 11.5);
        }
        if let Some(b) = &self.best {
            cx += 6.0;
            let _ = write!(
                out,
                "<text class=\"chip-arrow\" x=\"{}\" y=\"{}\">→</text>",
                px(cx),
                px(base)
            );
            cx += mono_w("→", 11.5) + 6.0;
            let now = if changed { "chip-v now" } else { "chip-v" };
            let _ = write!(
                out,
                "<text class=\"{now}\" x=\"{}\" y=\"{}\">{}</text>",
                px(cx),
                px(base),
                esc(b)
            );
            cx += mono_w(b, 11.5) + 4.0;
        }
        if !self.range.is_empty() {
            cx += 7.0;
            let _ = write!(
                out,
                "<text class=\"chip-r\" x=\"{}\" y=\"{}\">[{}]</text>",
                px(cx),
                px(base),
                esc(&self.range)
            );
            cx += mono_w(&format!("[{}]", self.range), 11.0);
        }
        if !self.detail.is_empty() {
            cx += 7.0;
            let _ = write!(
                out,
                "<text class=\"chip-r\" x=\"{}\" y=\"{}\">{}</text>",
                px(cx),
                px(base),
                esc(&self.detail)
            );
        }
        out.push_str("</g>");
    }

    /// The chip as HTML, with or without its field.
    pub fn html(&self, with_field: bool) -> String {
        let changed = self.changed();
        let field = if with_field || self.value.is_none() {
            format!("<span class=\"chip-k\">{}</span> ", esc(&self.field))
        } else {
            String::new()
        };
        let mut s = format!("<span class=\"{}\"{}>{field}", self.class(), self.attrs(),);
        if let Some(v) = &self.value {
            let _ = write!(
                s,
                "<span class=\"chip-v{}\">{}</span>",
                if changed { " was" } else { "" },
                esc(v)
            );
        }
        if let Some(b) = &self.best {
            s.push_str(&format!(
                " <span class=\"chip-arrow\">→</span> <span class=\"chip-v{}\">{}</span>",
                if changed { " now" } else { "" },
                esc(b)
            ));
        }
        if !self.range.is_empty() {
            s.push_str(&format!(
                " <span class=\"chip-r\">[{}]</span>",
                esc(&self.range)
            ));
        }
        if !self.detail.is_empty() {
            s.push_str(&format!(
                " <span class=\"chip-r\">{}</span>",
                esc(&self.detail)
            ));
        }
        s.push_str("</span>");
        s
    }
}

/// A node as the layout reads it.
struct Part {
    id: String,
    label: String,
    slot: &'static str,
    icon: &'static str,
    subtitle: String,
    badges: Vec<String>,
    outline: Option<String>,
    refused: bool,
    /// A region that is no node: drawn only as the frame around what it holds.
    frame_only: bool,
    props: Vec<(String, String)>,
    description: String,
    region: Option<String>,
    row: usize,
    ports: Vec<PortRow>,
    chips: Vec<Chip>,
}

/// A port as one row of its card.
struct PortRow {
    /// `node.port`.
    address: String,
    own: String,
    title: String,
    refused: bool,
    via: Option<String>,
    unresolved: bool,
    /// The names of the refusals that mark the port.
    refusals: Vec<String>,
}

impl PortRow {
    /// The text after `port`.
    fn text(&self) -> String {
        let mut s = self.own.clone();
        if let Some(v) = &self.via {
            s.push_str(&format!(" {v}"));
        }
        if self.unresolved && !self.refusals.is_empty() {
            s.push_str(&format!(" {}", self.refusals.join(", ")));
        }
        s
    }
}

/// An edge as the layout reads it.
struct Link {
    from: String,
    to: String,
    label: String,
    via: Option<String>,
    /// Whether the edge stands for more than one.
    multi: bool,
}

/// The nodes and edges of a page, resolved for layout.
struct Scene {
    parts: Vec<Part>,
    links: Vec<Link>,
    /// Each region with the region it lies in, the heading of what it holds and whether it reads
    /// "per".
    regions: Vec<(String, Option<String>, String, bool)>,
    /// The part whose frame each part is drawn in.
    parent: Vec<Option<usize>>,
}

impl Scene {
    fn part(&self, id: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.id == id)
    }

    /// The part an edge end names: a node, or the node of the port `<node>.<port>`.
    fn index(&self, name: &str) -> Option<usize> {
        self.parts.iter().position(|p| p.id == name).or_else(|| {
            let (head, _) = name.split_once('.')?;
            self.parts
                .iter()
                .position(|p| p.id == head && !p.ports.is_empty())
        })
    }

    fn read(page: &Page, decor: &Decor) -> Self {
        let palette = Palette::of(page, &decor.groups);
        let group_label = |g: &str| -> String {
            decor
                .groups
                .iter()
                .find(|d| d.0 == g)
                .map(|d| d.1.clone())
                .unwrap_or_else(|| {
                    if g.is_empty() {
                        "other".to_string()
                    } else {
                        g.to_string()
                    }
                })
        };
        let regions: Vec<(String, Option<String>, String, bool)> = all_regions(page)
            .into_iter()
            .map(|(r, parent)| {
                let heading = match (&r.per, &r.r#in) {
                    (Some(p), _) => format!("per {p}"),
                    (None, Some(i)) => format!("in {i}"),
                    (None, None) => format!("in {}", r.label),
                };
                (r.id.clone(), parent, heading, r.per.is_some())
            })
            .collect();
        let rows = rows(page, &decor.groups);
        let none = NodeDecor::default();
        let mut parts: Vec<Part> = page
            .nodes
            .iter()
            .zip(&rows)
            .map(|(n, &row)| {
                let d = decor
                    .nodes
                    .iter()
                    .find(|(id, _)| *id == n.id)
                    .map(|(_, d)| d)
                    .unwrap_or(&none);
                let subtitle = d.subtitle.clone().unwrap_or_else(|| group_label(&n.group));
                let badges: Vec<String> = n
                    .badges
                    .iter()
                    .map(|b| {
                        if b.name == "count" {
                            format!("×{}", b.value)
                        } else {
                            format!("{} {}", b.name, b.value)
                        }
                    })
                    .collect();
                let description = d.description.clone().unwrap_or_else(|| {
                    let mut s = format!("{}, {subtitle}", n.label);
                    for b in &badges {
                        s.push_str(&format!(", {b}"));
                    }
                    s
                });
                Part {
                    id: n.id.clone(),
                    label: n.label.clone(),
                    slot: palette.slot(&n.group),
                    icon: icon(d.icon.unwrap_or("")),
                    description,
                    subtitle,
                    badges,
                    outline: d.outline.clone(),
                    refused: false,
                    frame_only: false,
                    props: d.properties.clone(),
                    region: n.region.clone(),
                    row,
                    ports: Vec::new(),
                    chips: Vec::new(),
                }
            })
            .collect();
        // A region that is no node is a frame of its own, inside the region it lies in.
        for (r, parent) in all_regions(page) {
            if parts.iter().any(|p| p.id == r.id) {
                continue;
            }
            parts.push(Part {
                id: r.id.clone(),
                label: r.label.clone(),
                slot: "other",
                icon: icon("frame"),
                description: r.label.clone(),
                subtitle: String::new(),
                badges: Vec::new(),
                outline: None,
                refused: false,
                frame_only: true,
                props: Vec::new(),
                region: parent,
                row: usize::MAX,
                ports: Vec::new(),
                chips: Vec::new(),
            });
        }
        for port in &page.ports {
            let Some(p) = parts.iter_mut().find(|p| p.id == port.node) else {
                continue;
            };
            let address = format!("{}.{}", port.node, port.name);
            let mut title = if port.kind.is_empty() {
                address.clone()
            } else {
                format!("{} {address}", port.kind)
            };
            match port.direction {
                Direction::In => title.push_str(", in"),
                Direction::Out => title.push_str(", out"),
                Direction::Both => {}
            }
            if let Some(c) = &port.contract {
                title.push_str(&format!(", contract {c}"));
            }
            if let Some(v) = &port.via {
                title.push_str(&format!(", via {v}"));
            }
            if port.unresolved {
                title.push_str(", unresolved");
            }
            p.ports.push(PortRow {
                address,
                own: port.name.clone(),
                title,
                refused: false,
                via: port.via.clone(),
                unresolved: port.unresolved,
                refusals: Vec::new(),
            });
        }
        for (i, m) in page.marks.iter().enumerate() {
            let Some(p) = parts.iter_mut().find(|p| p.id == m.target.node) else {
                continue;
            };
            if m.level == page::Level::Refusal {
                match &m.target.port {
                    Some(name) => {
                        for port in p.ports.iter_mut().filter(|x| x.own == *name) {
                            port.refused = true;
                            port.refusals.push(m.name.clone());
                        }
                    }
                    None => p.refused = true,
                }
            }
            let chip = decor
                .chips
                .get(i)
                .and_then(|c| c.clone())
                .unwrap_or_else(|| Chip::of(m));
            p.chips.push(chip);
        }
        // The frame each part is drawn in: the part its region names, never itself, and no
        // chain that comes back to where it started.
        let n = parts.len();
        let mut parent: Vec<Option<usize>> = (0..n)
            .map(|i| {
                let r = parts[i].region.as_deref()?;
                parts.iter().position(|p| p.id == r).filter(|&j| j != i)
            })
            .collect();
        for i in 0..n {
            let mut cur = i;
            for _ in 0..=n {
                match parent[cur] {
                    Some(j) if j == i => {
                        parent[i] = None;
                        break;
                    }
                    Some(j) => cur = j,
                    None => break,
                }
            }
        }
        // A frame of its own sits in the top row of what it holds.
        for _ in 0..n {
            for i in 0..n {
                if !parts[i].frame_only {
                    continue;
                }
                let r = (0..n)
                    .filter(|&c| parent[c] == Some(i))
                    .map(|c| parts[c].row)
                    .min()
                    .unwrap_or(usize::MAX);
                parts[i].row = r;
            }
        }
        for p in &mut parts {
            if p.row == usize::MAX {
                p.row = 0;
            }
        }
        let links = page
            .edges
            .iter()
            .enumerate()
            .map(|(i, e)| Link {
                from: e.from.clone(),
                to: e.to.clone(),
                label: match e.count {
                    Some(c) => format!("{} ×{c}", e.label),
                    None => e.label.clone(),
                },
                via: decor.vias.get(i).cloned().flatten(),
                multi: e.count.is_some_and(|c| c > 1),
            })
            .collect();
        Scene {
            parts,
            links,
            regions,
            parent,
        }
    }

    /// The heading of what the frame of part `c` holds, and whether it reads "per".
    fn heading(&self, c: usize) -> (String, bool) {
        let p = &self.parts[c];
        match self.regions.iter().find(|r| r.0 == p.id) {
            Some((_, _, heading, per)) => (heading.clone(), *per),
            None => (format!("in {}", p.label), false),
        }
    }

    /// Whether part `c` is drawn as a frame: a region that is no node, or a node others are in.
    fn framed(&self, c: usize) -> bool {
        self.parts[c].frame_only || self.parent.contains(&Some(c))
    }

    /// The outermost part whose frame part `i` is drawn in.
    fn top(&self, i: usize) -> usize {
        let mut cur = i;
        while let Some(j) = self.parent[cur] {
            cur = j;
        }
        cur
    }
}

/// The row of each node: its group's rank; below every ranked row for a group without one; the
/// longest chain of edges that leads to it when no group has a rank.
fn rows(page: &Page, groups: &[(String, String, u32)]) -> Vec<usize> {
    if !groups.is_empty() {
        let below = groups.iter().map(|g| g.2 as usize).max().unwrap_or(0) + 1;
        return page
            .nodes
            .iter()
            .map(|n| {
                groups
                    .iter()
                    .find(|g| g.0 == n.group)
                    .map(|g| g.2 as usize)
                    .unwrap_or(below)
            })
            .collect();
    }
    let n = page.nodes.len();
    let index = |end: &str| {
        page.nodes.iter().position(|x| x.id == end).or_else(|| {
            let (head, _) = end.split_once('.')?;
            page.nodes.iter().position(|x| x.id == head)
        })
    };
    let edges: Vec<(usize, usize)> = page
        .edges
        .iter()
        .filter_map(|e| Some((index(&e.from)?, index(&e.to)?)))
        .filter(|(a, b)| a != b)
        .collect();
    // Edges that close a cycle, found by a depth-first walk in page order, are left out.
    let mut state = vec![0u8; n];
    let mut kept: Vec<(usize, usize)> = Vec::new();
    for start in 0..n {
        if state[start] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        state[start] = 1;
        while let Some(top) = stack.last_mut() {
            let v = top.0;
            let out: Vec<usize> = edges.iter().filter(|e| e.0 == v).map(|e| e.1).collect();
            if top.1 < out.len() {
                let w = out[top.1];
                top.1 += 1;
                match state[w] {
                    0 => {
                        kept.push((v, w));
                        state[w] = 1;
                        stack.push((w, 0));
                    }
                    2 => kept.push((v, w)),
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    let mut row = vec![0usize; n];
    for _ in 0..n {
        let mut moved = false;
        for &(a, b) in &kept {
            if row[b] < row[a] + 1 {
                row[b] = row[a] + 1;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    row
}

/// Padding of a frame, the height of its heading and the gap between what it holds.
const FPAD: f64 = 10.0;
const FHEAD: f64 = 24.0;
const FGAP: f64 = 16.0;

/// The size of the card of node `p`.
fn node_size(sc: &Scene, p: usize) -> (f64, f64) {
    let p = &sc.parts[p];
    let name_w = text_w(&p.label, 13.5, true);
    let badge: f64 = p.badges.iter().map(|b| badge_w(b) + 8.0).sum();
    let tag = p.outline.as_ref().map(|t| tag_w(t) + 8.0).unwrap_or(0.0);
    let head = ICON + 10.0 + (name_w + badge).max(text_w(&p.subtitle, 11.5, false) + tag);
    let mut inner = head;
    for (k, v) in &p.props {
        inner = inner.max(text_w(k, 11.5, false) + 6.0 + text_w(v, 11.5, false));
    }
    for port in &p.ports {
        inner = inner.max(text_w(&format!("port {}", port.text()), 11.5, false) + 8.0);
    }
    for c in &p.chips {
        inner = inner.max(c.width());
    }
    let w = (inner + 2.0 * PAD).max(MIN_W).ceil();
    let mut h = HEAD;
    h += p.props.len() as f64 * PROP;
    h += p.ports.len() as f64 * PROP;
    h += p.chips.len() as f64 * CHIP;
    let h = if h == HEAD { HEAD } else { h + 6.0 };
    (w, h)
}

/// One item of a row: a node's card, or the frame of a region with the cards and frames it
/// holds side by side.
struct Card {
    w: f64,
    h: f64,
    row: usize,
    /// Each node card: its part, its offset from the item's top left and its size.
    nodes: Vec<(usize, f64, f64, f64, f64)>,
    /// Each frame, outermost first: its part, offset and size.
    frames: Vec<(usize, f64, f64, f64, f64)>,
    /// Two node cards side by side in one frame: their parts and the gap's left and right edge
    /// and top, from the item's top left.
    gaps: Vec<(usize, usize, f64, f64, f64)>,
}

impl Card {
    fn new(sc: &Scene, part: usize) -> Self {
        let mut c = Card {
            w: 0.0,
            h: 0.0,
            row: sc.parts[part].row,
            nodes: Vec::new(),
            frames: Vec::new(),
            gaps: Vec::new(),
        };
        let (w, h) = place(sc, part, 0.0, 0.0, &mut c);
        c.w = w;
        c.h = h;
        c
    }

    /// The node card of part `p`: its offset and size.
    fn node(&self, p: usize) -> Option<(f64, f64, f64, f64)> {
        self.nodes
            .iter()
            .find(|n| n.0 == p)
            .map(|n| (n.1, n.2, n.3, n.4))
    }

    /// Whether the item is one node card alone.
    fn single(&self) -> bool {
        self.frames.is_empty()
    }

    /// Mirrors what the frames hold left to right.
    fn mirror(&mut self) {
        let w = self.w;
        for n in self.nodes.iter_mut().chain(self.frames.iter_mut()) {
            n.1 = w - n.1 - n.3;
        }
        for g in &mut self.gaps {
            *g = (g.1, g.0, w - g.3, w - g.2, g.4);
        }
    }
}

/// The text over a frame: the region's label when it is no node, then its heading.
fn frame_head(sc: &Scene, p: usize) -> (Option<String>, String, bool) {
    let (heading, per) = sc.heading(p);
    let label = sc.parts[p].frame_only.then(|| sc.parts[p].label.clone());
    (label, heading, per)
}

fn frame_head_w(sc: &Scene, p: usize) -> f64 {
    let (label, heading, _) = frame_head(sc, p);
    label.map(|l| text_w(&l, 12.0, true) + 8.0).unwrap_or(0.0) + text_w(&heading, 11.0, false)
}

/// Lays out part `p` with its top left at `x`, `y`: its node card, or its frame holding its own
/// card first and then each part drawn in it, side by side. Returns the size.
fn place(sc: &Scene, p: usize, x: f64, y: f64, card: &mut Card) -> (f64, f64) {
    if !sc.framed(p) {
        let (w, h) = node_size(sc, p);
        card.nodes.push((p, x, y, w, h));
        return (w, h);
    }
    let fi = card.frames.len();
    card.frames.push((p, x, y, 0.0, 0.0));
    let mut members: Vec<usize> = Vec::new();
    if !sc.parts[p].frame_only {
        members.push(p);
    }
    members.extend((0..sc.parts.len()).filter(|&c| sc.parent[c] == Some(p)));
    let top = y + FHEAD;
    let mut cx = x + FPAD;
    let mut h: f64 = 0.0;
    let mut prev: Option<usize> = None;
    let plain = |m: usize| m == p || !sc.framed(m);
    for &m in &members {
        let left = cx;
        if let Some(a) = prev {
            let widest = if plain(a) && plain(m) {
                sc.links
                    .iter()
                    .filter(|l| {
                        let (f, t) = (sc.index(&l.from), sc.index(&l.to));
                        (f == Some(a) && t == Some(m)) || (f == Some(m) && t == Some(a))
                    })
                    .map(|l| text_w(&edge_label(sc, l, false), 11.5, false) + 18.0 + 24.0)
                    .fold(0.0, f64::max)
            } else {
                0.0
            };
            cx += FGAP.max(widest);
            if plain(a) && plain(m) {
                card.gaps.push((a, m, left, cx, top));
            }
        }
        let (mw, mh) = if m == p {
            let (w, h) = node_size(sc, p);
            card.nodes.push((p, cx, top, w, h));
            (w, h)
        } else {
            place(sc, m, cx, top, card)
        };
        cx += mw;
        h = h.max(mh);
        prev = Some(m);
    }
    let w = (cx - x + FPAD).max(frame_head_w(sc, p) + 2.0 * FPAD).ceil();
    let h = FHEAD + h.max(20.0) + FPAD;
    card.frames[fi] = (p, x, y, w, h);
    (w, h)
}

fn badge_w(text: &str) -> f64 {
    mono_w(text, 11.0) + 12.0
}

fn tag_w(text: &str) -> f64 {
    text_w(text, 11.0, false) + 14.0
}

#[derive(Clone, Copy, PartialEq)]
enum ItemKind {
    Node(usize),
    Dummy(usize),
    Label(usize),
    Fan(usize),
    Trunk(usize),
}

struct Item {
    kind: ItemKind,
    layer: usize,
    w: f64,
    h: f64,
    x: f64,
    y: f64,
}

/// Edges between two cards on different rows, drawn together.
struct Bundle {
    upper: usize,
    lower: usize,
    links: Vec<usize>,
    channel: Option<usize>,
    label_layer: usize,
    items: Vec<usize>,
    /// For a channel bundle: which edges pass left of the card.
    left: Vec<usize>,
    right: Vec<usize>,
    /// The fan this single edge shares its label with.
    fan: Option<usize>,
}

/// Single edges with one label at one side of one card. They share one label beside the card
/// and one trunk from there through every row between: each edge joins the trunk on a track of
/// the label row next to its other card, so the edges draw as a comb, not as parallel lines.
struct Fan {
    card: usize,
    /// The node card of the item the edges meet at.
    member: Option<usize>,
    at_lower: bool,
    label: String,
    /// The label row next to the card, where the label sits.
    layer: usize,
    /// The shared item of each layer the trunk crosses: the label at `layer`, a trunk elsewhere.
    items: Vec<(usize, usize)>,
}

impl Fan {
    fn item(&self, layer: usize) -> Option<usize> {
        self.items
            .iter()
            .find(|(l, _)| *l == layer)
            .map(|(_, it)| *it)
    }
}

/// Track spacing of the combs in a label row.
const TRACK: f64 = 6.0;

impl Bundle {
    fn label_size(&self, sc: &Scene) -> (f64, f64) {
        match self.channel {
            None => {
                let l = &sc.links[self.links[0]];
                (
                    text_w(&edge_label(sc, l, false), 11.5, false) + 18.0,
                    PILL_H,
                )
            }
            Some(c) => {
                let (wl, wr) = self.side_widths(sc);
                let (cw, ch) = channel_card_size(&sc.parts[c]);
                let nl = self.left.len() as f64;
                let nr = self.right.len() as f64;
                let w = wl + nl * LINE_GAP + 12.0 + cw + 12.0 + nr * LINE_GAP + wr;
                let rows = self.left.len().max(self.right.len()) as f64;
                (w, ch.max(rows * (PILL_H + 4.0)))
            }
        }
    }

    fn side_widths(&self, sc: &Scene) -> (f64, f64) {
        let side = |ls: &[usize]| {
            ls.iter()
                .map(|&l| text_w(&sc.links[l].label, 11.5, false) + 18.0 + 6.0)
                .fold(0.0, f64::max)
        };
        (side(&self.left), side(&self.right))
    }
}

fn channel_card_size(p: &Part) -> (f64, f64) {
    let w = 8.0 + 22.0 + 8.0 + text_w(&p.label, 13.0, true).max(text_w(&p.subtitle, 11.0, false));
    ((w + 12.0).ceil(), 44.0)
}

/// The port an edge end `node.port` addresses, by its own name.
fn port_of<'a>(sc: &Scene, end: &'a str) -> Option<&'a str> {
    let (head, own) = end.split_once('.')?;
    sc.part(head)?
        .ports
        .iter()
        .any(|p| p.address == end)
        .then_some(own)
}

fn edge_label(sc: &Scene, l: &Link, on_channel: bool) -> String {
    let base = match (&l.via, on_channel) {
        (Some(o), false) if sc.part(o).is_some() => format!("{} via {}", l.label, o),
        _ => l.label.clone(),
    };
    match port_of(sc, &l.to).or_else(|| port_of(sc, &l.from)) {
        Some(port) => format!("{base} · {port}"),
        None => base,
    }
}

/// Edges between two cards of one row.
struct Level {
    a: usize,
    b: usize,
    links: Vec<usize>,
}

pub(crate) struct Graph {
    pub svg: String,
    pub width: f64,
    pub height: f64,
}

fn pava(target: &[f64], weight: &[f64]) -> Vec<f64> {
    let mut blocks: Vec<(f64, f64, usize)> = Vec::new();
    for (i, &t) in target.iter().enumerate() {
        blocks.push((t * weight[i], weight[i], 1));
        while blocks.len() > 1 {
            let n = blocks.len();
            let (s1, w1, c1) = blocks[n - 2];
            let (s2, w2, c2) = blocks[n - 1];
            if s1 / w1 > s2 / w2 {
                blocks.truncate(n - 2);
                blocks.push((s1 + s2, w1 + w2, c1 + c2));
            } else {
                break;
            }
        }
    }
    let mut out = Vec::with_capacity(target.len());
    for (s, w, c) in blocks {
        out.extend(std::iter::repeat_n(s / w, c));
    }
    out
}

pub(crate) fn render(page: &Page, decor: &Decor) -> Graph {
    let scene = Scene::read(page, decor);
    let sc = &scene;
    let n = sc.parts.len();
    // An edge end `<node>.<port>` lands on the card of the node with that port.
    let index = |name: &str| sc.index(name);
    let endpoints: Vec<&str> = sc
        .links
        .iter()
        .flat_map(|l| [l.from.as_str(), l.to.as_str()])
        .collect();

    // A node in a region is drawn inside the frame of the outermost region it lies in; that frame
    // is one item of its row.
    let host: Vec<Option<usize>> = (0..n)
        .map(|i| Some(sc.top(i)).filter(|&t| t != i))
        .collect();
    let row_of = |i: usize| sc.parts[i].row;
    let resolve = |name: &str| index(name).map(|i| host[i].unwrap_or(i));

    // A node that is only ever passed through by edges between two rows is drawn on those edges.
    let mut on_edge = vec![false; n];
    for (i, p) in sc.parts.iter().enumerate() {
        if host[i].is_some()
            || p.region.is_some()
            || sc.framed(i)
            || endpoints.contains(&p.id.as_str())
        {
            continue;
        }
        on_edge[i] = sc.links.iter().any(|l| {
            l.via.as_deref() == Some(p.id.as_str())
                && match (resolve(&l.from), resolve(&l.to)) {
                    (Some(a), Some(b)) => row_of(a) != row_of(b),
                    _ => false,
                }
        });
    }

    // Cards in order of first appearance in the edges, then in the order of the page.
    let mut order: Vec<usize> = Vec::new();
    for name in &endpoints {
        if let Some(i) = resolve(name)
            && !order.contains(&i)
        {
            order.push(i);
        }
    }
    for i in 0..n {
        if host[i].is_none() && !on_edge[i] && !order.contains(&i) {
            order.push(i);
        }
    }
    let mut cards: Vec<Card> = order.iter().map(|&i| Card::new(sc, i)).collect();
    let card_of = |part: usize| order.iter().position(|&c| c == part);

    // A card that two or more cards of its own row send to moves to a row of its own just
    // below: its links then cross rows, where they share a label, instead of arching over the
    // cards between.
    let ends: Vec<(usize, usize)> = sc
        .links
        .iter()
        .filter_map(|l| Some((card_of(resolve(&l.from)?)?, card_of(resolve(&l.to)?)?)))
        .filter(|(a, b)| a != b && cards[*a].row == cards[*b].row)
        .collect();
    let sinks: Vec<bool> = (0..cards.len())
        .map(|c| {
            let mut from: Vec<usize> = ends.iter().filter(|e| e.1 == c).map(|e| e.0).collect();
            from.sort_unstable();
            from.dedup();
            from.len() >= 2
        })
        .collect();
    for (c, card) in cards.iter_mut().enumerate() {
        card.row = 2 * card.row + usize::from(sinks[c]);
    }

    let mut rows: Vec<usize> = cards.iter().map(|c| c.row).collect();
    rows.sort_unstable();
    rows.dedup();
    let layer_of_card: Vec<usize> = cards
        .iter()
        .map(|c| 2 * rows.iter().position(|&r| r == c.row).unwrap_or(0))
        .collect();
    let layers_n = if rows.is_empty() {
        1
    } else {
        2 * rows.len() - 1
    };

    // Bundles across rows, levels within a row.
    let mut bundles: Vec<Bundle> = Vec::new();
    let mut levels: Vec<Level> = Vec::new();
    for (li, l) in sc.links.iter().enumerate() {
        let (Some(a), Some(b)) = (resolve(&l.from), resolve(&l.to)) else {
            continue;
        };
        let (Some(ca), Some(cb)) = (card_of(a), card_of(b)) else {
            continue;
        };
        if ca == cb {
            continue;
        }
        if layer_of_card[ca] == layer_of_card[cb] {
            let (x, y) = (ca.min(cb), ca.max(cb));
            match levels.iter_mut().find(|v| v.a == x && v.b == y) {
                Some(v) => v.links.push(li),
                None => levels.push(Level {
                    a: x,
                    b: y,
                    links: vec![li],
                }),
            }
            continue;
        }
        let (upper, lower) = if layer_of_card[ca] < layer_of_card[cb] {
            (ca, cb)
        } else {
            (cb, ca)
        };
        let channel = l.via.as_deref().and_then(index).filter(|&c| on_edge[c]);
        let existing = channel.and_then(|c| {
            bundles
                .iter()
                .position(|bd| bd.upper == upper && bd.lower == lower && bd.channel == Some(c))
        });
        match existing {
            Some(bi) => bundles[bi].links.push(li),
            None => {
                let (lu, ll) = (layer_of_card[upper], layer_of_card[lower]);
                let mid = (lu + ll) / 2;
                let label_layer = if mid % 2 == 1 { mid } else { mid + 1 };
                bundles.push(Bundle {
                    upper,
                    lower,
                    links: vec![li],
                    channel,
                    label_layer,
                    items: Vec::new(),
                    left: Vec::new(),
                    right: Vec::new(),
                    fan: None,
                });
            }
        }
    }

    // Fans: two or more single links with one label that end at, or start from, one side of one
    // card share one label beside that card, and their lines meet there.
    let mut fans: Vec<Fan> = Vec::new();
    for at_end in [true, false] {
        let mut keys: Vec<(usize, bool, String, usize, Option<usize>)> = Vec::new();
        for (bi, bd) in bundles.iter().enumerate() {
            if bd.channel.is_some() || bd.fan.is_some() {
                continue;
            }
            let l = &sc.links[bd.links[0]];
            let end = if at_end { &l.to } else { &l.from };
            let Some(c) = resolve(end).and_then(card_of) else {
                continue;
            };
            keys.push((c, c == bd.lower, edge_label(sc, l, false), bi, index(end)));
        }
        for &(c, at_lower, ref label, _, member) in &keys {
            if fans.iter().any(|f| {
                f.card == c && f.member == member && f.at_lower == at_lower && f.label == *label
            }) {
                continue;
            }
            let members: Vec<usize> = keys
                .iter()
                .filter(|k| k.0 == c && k.4 == member && k.1 == at_lower && k.2 == *label)
                .map(|k| k.3)
                .filter(|&bi| bundles[bi].fan.is_none())
                .collect();
            if members.len() < 2 {
                continue;
            }
            let layer = if at_lower {
                layer_of_card[c] - 1
            } else {
                layer_of_card[c] + 1
            };
            for &bi in &members {
                bundles[bi].fan = Some(fans.len());
            }
            fans.push(Fan {
                card: c,
                member,
                at_lower,
                label: label.clone(),
                layer,
                items: Vec::new(),
            });
        }
    }
    for bd in &mut bundles {
        if bd.channel.is_some() {
            for (k, &l) in bd.links.iter().enumerate() {
                if k % 2 == 0 {
                    bd.left.push(l);
                } else {
                    bd.right.push(l);
                }
            }
        }
    }

    // Items: cards, then the label or gap of each bundle in each layer it crosses.
    let mut items: Vec<Item> = Vec::new();
    let mut layers: Vec<Vec<usize>> = vec![Vec::new(); layers_n];
    for (ci, c) in cards.iter().enumerate() {
        layers[layer_of_card[ci]].push(items.len());
        items.push(Item {
            kind: ItemKind::Node(ci),
            layer: layer_of_card[ci],
            w: c.w,
            h: c.h,
            x: 0.0,
            y: 0.0,
        });
    }
    for (bi, bd) in bundles.iter_mut().enumerate() {
        let (lu, ll) = (layer_of_card[bd.upper], layer_of_card[bd.lower]);
        for (layer, members) in layers.iter_mut().enumerate().take(ll).skip(lu + 1) {
            if let Some(f) = bd.fan {
                let it = match fans[f].item(layer) {
                    Some(it) => it,
                    None => {
                        let it = items.len();
                        fans[f].items.push((layer, it));
                        members.push(it);
                        let (kind, w, h) = if layer == fans[f].layer {
                            let w = text_w(&fans[f].label, 11.5, false) + 18.0 + 8.0;
                            (ItemKind::Fan(f), w, PILL_H)
                        } else {
                            (ItemKind::Trunk(f), LINE_GAP + 4.0, 0.0)
                        };
                        items.push(Item {
                            kind,
                            layer,
                            w,
                            h,
                            x: 0.0,
                            y: 0.0,
                        });
                        it
                    }
                };
                bd.items.push(it);
                continue;
            }
            let (kind, w, h) = if layer == bd.label_layer && bd.fan.is_none() {
                let (w, h) = bd.label_size(sc);
                (ItemKind::Label(bi), w, h)
            } else {
                (
                    ItemKind::Dummy(bi),
                    bd.links.len() as f64 * LINE_GAP + 4.0,
                    0.0,
                )
            };
            bd.items.push(items.len());
            members.push(items.len());
            items.push(Item {
                kind,
                layer,
                w,
                h,
                x: 0.0,
                y: 0.0,
            });
        }
    }
    // Neighbours between consecutive layers.
    let mut up: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
    let mut down: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
    for bd in &bundles {
        let mut chain = vec![bd.upper];
        chain.extend(bd.items.iter().copied());
        chain.push(bd.lower);
        for w in chain.windows(2) {
            down[w[0]].push(w[1]);
            up[w[1]].push(w[0]);
        }
    }
    // Cards joined within a row move as one group.
    let mut group: Vec<usize> = (0..items.len()).collect();
    fn find(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        g[i] = r;
        r
    }
    for v in &levels {
        let (ra, rb) = (find(&mut group, v.a), find(&mut group, v.b));
        if ra != rb {
            group[ra.max(rb)] = ra.min(rb);
        }
    }
    let groups: Vec<usize> = (0..items.len()).map(|i| find(&mut group, i)).collect();

    let mut pos = vec![0.0; items.len()];
    let renumber = |layers: &Vec<Vec<usize>>, pos: &mut Vec<f64>| {
        for layer in layers {
            for (k, &it) in layer.iter().enumerate() {
                pos[it] = k as f64;
            }
        }
    };
    renumber(&layers, &mut pos);
    let crossings = |pos: &[f64]| -> usize {
        let mut segs: Vec<(f64, f64)> = Vec::new();
        let mut total = 0;
        for li in 0..layers_n.saturating_sub(1) {
            segs.clear();
            for (a, ds) in down.iter().enumerate() {
                if items[a].layer == li {
                    segs.extend(ds.iter().map(|&b| (pos[a], pos[b])));
                }
            }
            for i in 0..segs.len() {
                for j in i + 1..segs.len() {
                    let (s, t) = (segs[i], segs[j]);
                    if (s.0 - t.0) * (s.1 - t.1) < 0.0 {
                        total += 1;
                    }
                }
            }
        }
        total
    };
    let mut best_layers = layers.clone();
    let mut best_cross = crossings(&pos);
    for _ in 0..8 {
        for downward in [true, false] {
            let seq: Vec<usize> = if downward {
                (1..layers_n).collect()
            } else {
                (0..layers_n.saturating_sub(1)).rev().collect()
            };
            for li in seq {
                let (nb, other) = if downward { (&up, &down) } else { (&down, &up) };
                let key = |it: usize| -> Option<f64> {
                    let ns = if nb[it].is_empty() {
                        &other[it]
                    } else {
                        &nb[it]
                    };
                    (!ns.is_empty())
                        .then(|| ns.iter().map(|&x| pos[x]).sum::<f64>() / ns.len() as f64)
                };
                let layer = &layers[li];
                let mut gkeys: Vec<(usize, f64, f64)> = Vec::new();
                for &it in layer {
                    let g = groups[it];
                    if gkeys.iter().any(|(x, _, _)| *x == g) {
                        continue;
                    }
                    let members: Vec<usize> =
                        layer.iter().copied().filter(|&m| groups[m] == g).collect();
                    let ks: Vec<f64> = members.iter().filter_map(|&m| key(m)).collect();
                    let first = members
                        .iter()
                        .map(|&m| pos[m])
                        .fold(f64::INFINITY, f64::min);
                    let k = if ks.is_empty() {
                        members.iter().map(|&m| pos[m]).sum::<f64>() / members.len() as f64
                    } else {
                        ks.iter().sum::<f64>() / ks.len() as f64
                    };
                    gkeys.push((g, k, first));
                }
                gkeys.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)));
                let mut next = Vec::with_capacity(layer.len());
                for (g, _, _) in &gkeys {
                    next.extend(layer.iter().copied().filter(|&m| groups[m] == *g));
                }
                layers[li] = next;
                renumber(&layers, &mut pos);
            }
            let c = crossings(&pos);
            if c < best_cross {
                best_cross = c;
                best_layers = layers.clone();
            }
        }
    }
    layers = best_layers;
    // In a group of three or more cards joined within a row, the card with the most links in the
    // row goes to the middle, so a star of links draws between neighbours rather than as arcs.
    let degree = |c: usize| levels.iter().filter(|v| v.a == c || v.b == c).count();
    for layer in &mut layers {
        let mut done: Vec<usize> = Vec::new();
        let mut k = 0;
        while k < layer.len() {
            let g = groups[layer[k]];
            let len = layer[k..].iter().take_while(|&&m| groups[m] == g).count();
            if len >= 3 && !done.contains(&g) {
                let members = &mut layer[k..k + len];
                let hub = (0..len)
                    .max_by(|&a, &b| degree(members[a]).cmp(&degree(members[b])).then(b.cmp(&a)))
                    .unwrap_or(0);
                let card = members[hub];
                let mut rest: Vec<usize> = members.iter().copied().filter(|&m| m != card).collect();
                rest.insert(len / 2, card);
                members.copy_from_slice(&rest);
            }
            done.push(g);
            k += len.max(1);
        }
    }
    // A frame draws its own node card first, on the left. When it has more links within the row
    // to its right neighbour than to its left one, it is mirrored, so that card faces them.
    for layer in &layers {
        for (k, &it) in layer.iter().enumerate() {
            if it >= cards.len() || cards[it].single() {
                continue;
            }
            let count = |o: Option<&usize>| -> usize {
                o.map(|&o| {
                    levels
                        .iter()
                        .filter(|v| (v.a == it && v.b == o) || (v.a == o && v.b == it))
                        .map(|v| v.links.len())
                        .sum()
                })
                .unwrap_or(0)
            };
            let left = k.checked_sub(1).and_then(|j| layer.get(j));
            if count(layer.get(k + 1)) > count(left) {
                cards[it].mirror();
            }
        }
    }

    // Horizontal placement.
    let kinds: Vec<ItemKind> = items.iter().map(|it| it.kind).collect();
    let gap = |a: usize, b: usize| -> f64 {
        match (kinds[a], kinds[b]) {
            (ItemKind::Node(x), ItemKind::Node(y)) => {
                let (x, y) = (x.min(y), x.max(y));
                let widest = levels
                    .iter()
                    .filter(|v| v.a == x && v.b == y)
                    .flat_map(|v| v.links.iter())
                    .map(|&l| text_w(&edge_label(sc, &sc.links[l], false), 11.5, false) + 18.0)
                    .fold(0.0, f64::max);
                if widest > 0.0 {
                    (widest + 40.0).max(48.0)
                } else {
                    40.0
                }
            }
            (ItemKind::Node(_), _) | (_, ItemKind::Node(_)) => 22.0,
            _ => 16.0,
        }
    };
    for layer in &layers {
        let mut x = 0.0;
        for (k, &it) in layer.iter().enumerate() {
            if k > 0 {
                x += gap(layer[k - 1], it);
            }
            items[it].x = x + items[it].w / 2.0;
            x += items[it].w;
        }
    }
    let place = |layer: &[usize], items: &mut Vec<Item>, desired: &[f64]| {
        let mut offs = vec![0.0; layer.len()];
        for k in 1..layer.len() {
            let (a, b) = (layer[k - 1], layer[k]);
            offs[k] = offs[k - 1] + (items[a].w + items[b].w) / 2.0 + gap(a, b);
        }
        let target: Vec<f64> = desired.iter().zip(&offs).map(|(d, o)| d - o).collect();
        let weight: Vec<f64> = layer
            .iter()
            .map(|&it| match items[it].kind {
                ItemKind::Node(_) => 1.0,
                ItemKind::Label(_) | ItemKind::Fan(_) => 1.5,
                ItemKind::Dummy(_) | ItemKind::Trunk(_) => 2.0,
            })
            .collect();
        let y = pava(&target, &weight);
        for (k, &it) in layer.iter().enumerate() {
            items[it].x = y[k] + offs[k];
        }
    };
    for _ in 0..40 {
        let seq: Vec<usize> = (0..layers_n).chain((0..layers_n).rev()).collect();
        for li in seq {
            let layer = layers[li].clone();
            let desired: Vec<f64> = layer
                .iter()
                .map(|&it| {
                    let ns: Vec<usize> = up[it].iter().chain(down[it].iter()).copied().collect();
                    if ns.is_empty() {
                        items[it].x
                    } else {
                        ns.iter().map(|&x| items[x].x).sum::<f64>() / ns.len() as f64
                    }
                })
                .collect();
            place(&layer, &mut items, &desired);
        }
    }
    // Straighten: the passing items of one bundle or trunk (a run of dummies) take one x where
    // every layer they cross has room for it, so a long line runs straight, not as a staircase.
    let slot_of: Vec<(usize, usize)> = {
        let mut s = vec![(0, 0); items.len()];
        for (li, layer) in layers.iter().enumerate() {
            for (k, &it) in layer.iter().enumerate() {
                s[it] = (li, k);
            }
        }
        s
    };
    let passing = |it: usize| matches!(kinds[it], ItemKind::Dummy(_) | ItemKind::Trunk(_));
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut seen = vec![false; items.len()];
    for bd in &bundles {
        let mut run: Vec<usize> = Vec::new();
        for &it in bd.items.iter().chain(std::iter::once(&usize::MAX)) {
            if it != usize::MAX && passing(it) && !seen[it] {
                seen[it] = true;
                run.push(it);
            } else if !run.is_empty() {
                runs.push(std::mem::take(&mut run));
            }
        }
    }
    runs.sort_by_key(|r| std::cmp::Reverse(r.len()));
    for run in runs.iter().filter(|r| r.len() >= 2) {
        let fits = |x: f64, items: &Vec<Item>| {
            run.iter().all(|&it| {
                let (li, k) = slot_of[it];
                let layer = &layers[li];
                let w = items[it].w / 2.0;
                let left_ok = k == 0 || {
                    let a = layer[k - 1];
                    items[a].x + items[a].w / 2.0 + gap(a, it) <= x - w + 0.01
                };
                let right_ok = k + 1 == layer.len() || {
                    let b = layer[k + 1];
                    x + w + gap(it, b) <= items[b].x - items[b].w / 2.0 + 0.01
                };
                left_ok && right_ok
            })
        };
        let cost =
            |x: f64, items: &Vec<Item>| run.iter().map(|&it| (items[it].x - x).abs()).sum::<f64>();
        let mut best: Option<(f64, f64)> = None;
        for &c in run {
            let x = items[c].x;
            if fits(x, &items) {
                let cst = cost(x, &items);
                if best.is_none_or(|(_, b)| cst < b) {
                    best = Some((x, cst));
                }
            }
        }
        if let Some((x, _)) = best {
            for &it in run {
                items[it].x = x;
            }
        }
    }
    let left = items
        .iter()
        .map(|it| it.x - it.w / 2.0)
        .fold(f64::INFINITY, f64::min);
    let shift = if left.is_finite() { MARGIN - left } else { 0.0 };
    for it in &mut items {
        it.x = (it.x + shift).round();
    }
    let width = items
        .iter()
        .map(|it| it.x + it.w / 2.0)
        .fold(0.0, f64::max)
        .ceil()
        + MARGIN;

    // Vertical placement.
    let mut layer_top = vec![0.0; layers_n];
    let mut layer_h = vec![0.0; layers_n];
    // A label row holds, top to bottom: the tracks of the combs that join a trunk going down,
    // the labels, the tracks of the combs that join a trunk going up.
    let join_of = |bd: &Bundle| -> Option<usize> {
        let f = bd.fan?;
        if fans[f].at_lower {
            bd.items.first().copied()
        } else {
            bd.items.last().copied()
        }
    };
    let mut combs: Vec<(usize, bool, usize)> = Vec::new(); // layer, going up, fan
    for bd in &bundles {
        if let (Some(it), Some(f)) = (join_of(bd), bd.fan) {
            let key = (items[it].layer, !fans[f].at_lower, f);
            if !combs.contains(&key) {
                combs.push(key);
            }
        }
    }
    combs.sort_unstable();
    let band = |li: usize, up: bool| -> f64 {
        let n = combs.iter().filter(|c| c.0 == li && c.1 == up).count();
        if n == 0 { 0.0 } else { n as f64 * TRACK + 4.0 }
    };
    let mut mid_top = vec![0.0; layers_n];
    let mut mid_h = vec![0.0; layers_n];
    let mut y = MARGIN + 6.0;
    for li in 0..layers_n {
        let h = layers[li].iter().map(|&it| items[it].h).fold(0.0, f64::max);
        let h = if li % 2 == 1 { h.max(16.0) } else { h };
        layer_top[li] = y;
        mid_top[li] = y + band(li, false);
        mid_h[li] = h;
        layer_h[li] = band(li, false) + h + band(li, true);
        y += layer_h[li] + ROW_GAP;
    }
    let height = (y - ROW_GAP + MARGIN + 6.0).ceil().max(80.0);
    for it in &mut items {
        it.y = mid_top[it.layer] + mid_h[it.layer] / 2.0;
    }
    let track_y = |li: usize, f: usize| -> f64 {
        let up = !fans[f].at_lower;
        let k = combs
            .iter()
            .filter(|c| c.0 == li && c.1 == up)
            .position(|c| c.2 == f)
            .unwrap_or(0) as f64;
        let start = if up {
            mid_top[li] + mid_h[li] + 2.0
        } else {
            layer_top[li] + 2.0
        };
        start + (k + 0.5) * TRACK
    };

    // Edge geometry: each link of a bundle as a list of points from the upper card down.
    struct Route {
        link: usize,
        bundle: usize,
        pts: Vec<(f64, f64)>,
    }
    let mut routes: Vec<Route> = Vec::new();
    for (bi, bd) in bundles.iter().enumerate() {
        let n_links = bd.links.len();
        for (k, &l) in bd.links.iter().enumerate() {
            let mut pts = Vec::new();
            let join = join_of(bd);
            for &it in &bd.items {
                let item = &items[it];
                if let ItemKind::Fan(f) | ItemKind::Trunk(f) = item.kind {
                    let (top, bottom) = match item.kind {
                        ItemKind::Fan(_) => (item.y, item.y),
                        _ => (
                            layer_top[item.layer],
                            layer_top[item.layer] + layer_h[item.layer],
                        ),
                    };
                    let (top, bottom) = match (join == Some(it), fans[f].at_lower) {
                        (true, true) => (track_y(item.layer, f), bottom),
                        (true, false) => (top, track_y(item.layer, f)),
                        _ => (top, bottom),
                    };
                    pts.push((item.x, top));
                    if bottom != top {
                        pts.push((item.x, bottom));
                    }
                    continue;
                }
                let x = match item.kind {
                    ItemKind::Label(_) if bd.channel.is_some() => {
                        let (wl, _) = bd.side_widths(sc);
                        let left_edge = item.x - item.w / 2.0;
                        let right_edge = item.x + item.w / 2.0;
                        let (_, wr) = bd.side_widths(sc);
                        if let Some(j) = bd.left.iter().position(|&x| x == l) {
                            left_edge + wl + (j as f64 + 0.5) * LINE_GAP
                        } else {
                            let j = bd.right.iter().position(|&x| x == l).unwrap_or(0);
                            right_edge - wr - (j as f64 + 0.5) * LINE_GAP
                        }
                    }
                    ItemKind::Label(_) => item.x,
                    _ => item.x + (k as f64 - (n_links as f64 - 1.0) / 2.0) * LINE_GAP,
                };
                let (top, bottom) = match item.kind {
                    ItemKind::Label(_) => (item.y - item.h / 2.0, item.y + item.h / 2.0),
                    _ => (
                        layer_top[item.layer],
                        layer_top[item.layer] + layer_h[item.layer],
                    ),
                };
                pts.push((x, top));
                pts.push((x, bottom));
            }
            routes.push(Route {
                link: l,
                bundle: bi,
                pts,
            });
        }
    }
    // The card of the node an edge end lands on inside an item: its centre x, top, bottom and
    // width.
    let node_box = |ci: usize, part: Option<usize>| -> (f64, f64, f64, f64) {
        let it = &items[ci];
        let c = &cards[ci];
        if c.single() {
            return (it.x, it.y - it.h / 2.0, it.y + it.h / 2.0, c.w);
        }
        let (x0, y0) = (it.x - c.w / 2.0, it.y - c.h / 2.0);
        match part.and_then(|p| c.node(p)) {
            Some((dx, dy, w, h)) => (x0 + dx + w / 2.0, y0 + dy, y0 + dy + h, w),
            None => (it.x, y0, y0 + c.h, c.w),
        }
    };
    // The node an end of a route lands on.
    let end_node = |ri: usize, upper_end: bool| -> Option<usize> {
        let bd = &bundles[routes[ri].bundle];
        let l = &sc.links[routes[ri].link];
        let from_upper = resolve(&l.from).and_then(card_of) == Some(bd.upper);
        index(if from_upper == upper_end {
            &l.from
        } else {
            &l.to
        })
    };
    // Ports: the ends of edges on one side of a card, spread in the order of where they go.
    // The links of a fan share one port at the fan's card: its key is past the routes'.
    let port_key = |ri: usize, upper_end: bool| -> usize {
        let bd = &bundles[routes[ri].bundle];
        let card = if upper_end { bd.upper } else { bd.lower };
        match bd.fan {
            Some(f) if fans[f].card == card => routes.len() + f,
            _ => ri,
        }
    };
    // card, bottom side, key, toward x, node
    let mut ports: Vec<(usize, bool, usize, f64, Option<usize>)> = Vec::new();
    for (ri, r) in routes.iter().enumerate() {
        let bd = &bundles[r.bundle];
        let first = r.pts.first().map(|p| p.0).unwrap_or(items[bd.lower].x);
        let last = r.pts.last().map(|p| p.0).unwrap_or(items[bd.upper].x);
        for (card, side, toward) in [(bd.upper, true, first), (bd.lower, false, last)] {
            let key = port_key(ri, side);
            if !ports.iter().any(|p| p.2 == key && p.1 == side) {
                ports.push((card, side, key, toward, end_node(ri, side)));
            }
        }
    }
    let mut port_x: Vec<(usize, bool, f64)> = Vec::new(); // port key, upper end, x
    for ci in 0..cards.len() {
        for side in [true, false] {
            let mut nodes: Vec<Option<usize>> = Vec::new();
            for p in ports.iter().filter(|p| p.0 == ci && p.1 == side) {
                if !nodes.contains(&p.4) {
                    nodes.push(p.4);
                }
            }
            for node in nodes {
                let (cx, _, _, w) = node_box(ci, node);
                let mut ends: Vec<&(usize, bool, usize, f64, Option<usize>)> = ports
                    .iter()
                    .filter(|p| p.0 == ci && p.1 == side && p.4 == node)
                    .collect();
                ends.sort_by(|a, b| a.3.total_cmp(&b.3).then(a.2.cmp(&b.2)));
                let m = ends.len();
                let span = if m > 1 {
                    (w - 40.0).min(26.0 * (m as f64 - 1.0))
                } else {
                    0.0
                };
                for (j, e) in ends.iter().enumerate() {
                    let x = if m > 1 {
                        cx - span / 2.0 + span * j as f64 / (m as f64 - 1.0)
                    } else {
                        cx
                    };
                    port_x.push((e.2, side, x));
                }
            }
        }
    }

    let mut svg = String::new();
    let _ = write!(
        svg,
        "<svg class=\"graph\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" role=\"group\" aria-labelledby=\"graph-title graph-desc\">",
        w = px(width),
        h = px(height)
    );
    let _ = write!(
        svg,
        "<title id=\"graph-title\">{}</title><desc id=\"graph-desc\">{}</desc>",
        esc(&decor.words.title),
        esc(&decor.words.desc)
    );
    svg.push_str(
        "<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0 0L10 5L0 10z\" class=\"arrowhead\"/></marker><marker id=\"arrow-hot\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0 0L10 5L0 10z\" class=\"arrowhead hot\"/></marker></defs>",
    );

    // Edges across rows.
    svg.push_str("<g class=\"edges\">");
    for (ri, r) in routes.iter().enumerate() {
        let bd = &bundles[r.bundle];
        let link = &sc.links[r.link];
        let ux = port_x
            .iter()
            .find(|p| p.0 == port_key(ri, true) && p.1)
            .map(|p| p.2)
            .unwrap_or(items[bd.upper].x);
        let lx = port_x
            .iter()
            .find(|p| p.0 == port_key(ri, false) && !p.1)
            .map(|p| p.2)
            .unwrap_or(items[bd.lower].x);
        // A card shorter than its row leaves it straight down (or up), so the line bends only
        // between rows, never beside a taller card of the same row.
        let (lu, ll) = (items[bd.upper].layer, items[bd.lower].layer);
        let row_bottom = layer_top[lu] + layer_h[lu];
        let upper_bottom = node_box(bd.upper, end_node(ri, true)).2;
        let mut pts = vec![(ux, upper_bottom)];
        if row_bottom - upper_bottom > 0.5 {
            pts.push((ux, row_bottom));
        }
        // A link of a fan runs straight to its track, then along it to the trunk.
        let comb = bd.fan.map(|f| fans[f].at_lower);
        if comb == Some(true)
            && let Some(&(_, ty)) = r.pts.first()
        {
            pts.push((ux, ty));
        }
        pts.extend(r.pts.iter().copied());
        if comb == Some(false)
            && let Some(&(_, ty)) = r.pts.last()
        {
            pts.push((lx, ty));
        }
        let lower_top = node_box(bd.lower, end_node(ri, false)).1 - 1.0;
        if lower_top - layer_top[ll] > 0.5 {
            pts.push((lx, layer_top[ll]));
        }
        pts.push((lx, lower_top));
        let from_upper = resolve(&link.from).and_then(card_of) == Some(bd.upper);
        if !from_upper {
            pts[0].1 += 1.0;
            pts.reverse();
        }
        let mut d = format!("M{} {}", px(pts[0].0), px(pts[0].1));
        for w in pts.windows(2) {
            let ((x1, y1), (x2, y2)) = (w[0], w[1]);
            if (x1 - x2).abs() < 0.5 || (y1 - y2).abs() < 0.5 {
                let _ = write!(d, "L{} {}", px(x2), px(y2));
            } else {
                let ym = (y1 + y2) / 2.0;
                let _ = write!(
                    d,
                    "C{} {} {} {} {} {}",
                    px(x1),
                    px(ym),
                    px(x2),
                    px(ym),
                    px(x2),
                    px(y2)
                );
            }
        }
        edge_path(&mut svg, &d, link);
    }
    // Edges within a row.
    let mut row_labels: Vec<(f64, f64, String)> = Vec::new();
    let mut arcs = vec![0usize; layers_n];
    for v in &levels {
        let (a, b) = (v.a, v.b);
        let (l, r) = if items[a].x <= items[b].x {
            (a, b)
        } else {
            (b, a)
        };
        let layer = &layers[items[a].layer];
        let pa = layer.iter().position(|&x| x == a).unwrap_or(0);
        let pb = layer.iter().position(|&x| x == b).unwrap_or(0);
        let adjacent = pa.abs_diff(pb) == 1;
        let m = v.links.len() as f64;
        for (k, &li) in v.links.iter().enumerate() {
            let link = &sc.links[li];
            let from_left = resolve(&link.from).and_then(card_of) == Some(l);
            let label = edge_label(sc, link, false);
            // Two items side by side, one of them a frame: the edge runs straight when the node
            // card at each end faces the other item and the two cards share the height of the
            // lanes.
            let straight = (adjacent && !(cards[l].single() && cards[r].single()))
                .then(|| {
                    let (ln, rn) = if from_left {
                        (index(&link.from), index(&link.to))
                    } else {
                        (index(&link.to), index(&link.from))
                    };
                    let edge_x = |ci: usize, part: Option<usize>, right: bool| -> Option<f64> {
                        let it = &items[ci];
                        let c = &cards[ci];
                        if c.single() {
                            return Some(it.x + if right { it.w / 2.0 } else { -it.w / 2.0 });
                        }
                        let (dx, _, w, _) = c.node(part?)?;
                        let x0 = it.x - c.w / 2.0;
                        let faces = if right {
                            c.nodes.iter().all(|n| n.1 + n.3 <= dx + w + 0.5)
                        } else {
                            c.nodes.iter().all(|n| n.1 >= dx - 0.5)
                        };
                        faces.then_some(x0 + if right { dx + w } else { dx })
                    };
                    let x1 = edge_x(l, ln, true)?;
                    let x2 = edge_x(r, rn, false)?;
                    let (_, lt, lb, _) = node_box(l, ln);
                    let (_, rt, rb, _) = node_box(r, rn);
                    let (top, bottom) = (lt.max(rt), lb.min(rb));
                    let half = (m - 1.0) / 2.0 * 26.0;
                    (bottom - top >= 2.0 * half + 24.0).then(|| {
                        let y = (top + bottom) / 2.0 + (k as f64 - (m - 1.0) / 2.0) * 26.0;
                        (x1 + 1.0, x2 - 1.0, y)
                    })
                })
                .flatten();
            if let Some((x1, x2, y)) = straight {
                let (sx, ex) = if from_left { (x1, x2) } else { (x2, x1) };
                let d = format!("M{} {}L{} {}", px(sx), px(y), px(ex), px(y));
                edge_path(&mut svg, &d, link);
                row_labels.push(((x1 + x2) / 2.0, y, label));
            } else if adjacent && cards[l].single() && cards[r].single() {
                let y = items[l].y + (k as f64 - (m - 1.0) / 2.0) * 26.0;
                let x1 = items[l].x + items[l].w / 2.0 + 1.0;
                let x2 = items[r].x - items[r].w / 2.0 - 1.0;
                let (sx, ex) = if from_left { (x1, x2) } else { (x2, x1) };
                let d = format!("M{} {}L{} {}", px(sx), px(y), px(ex), px(y));
                edge_path(&mut svg, &d, link);
                row_labels.push(((x1 + x2) / 2.0, y, label));
            } else {
                let top = items[l].y - items[l].h / 2.0;
                let j = arcs[items[a].layer];
                arcs[items[a].layer] += 1;
                let lift = 46.0 + 28.0 * j as f64;
                // An arc leaves the top of the node card it starts at and lands on the top of
                // the one it ends at, inside a frame or not.
                let (sx, sy, ex, ey) = if cards[l].single() && cards[r].single() {
                    let (sx, ex) = if from_left {
                        (items[l].x, items[r].x)
                    } else {
                        (items[r].x, items[l].x)
                    };
                    (sx, top, ex, top)
                } else {
                    let (fc, tc) = if from_left { (l, r) } else { (r, l) };
                    let (sx, sy, _, _) = node_box(fc, index(&link.from));
                    let (ex, ey, _, _) = node_box(tc, index(&link.to));
                    (sx, sy, ex, ey)
                };
                let d = format!(
                    "M{} {}C{} {} {} {} {} {}",
                    px(sx),
                    px(sy),
                    px(sx),
                    px(top - lift),
                    px(ex),
                    px(top - lift),
                    px(ex),
                    px(ey - 1.0)
                );
                edge_path(&mut svg, &d, link);
                row_labels.push(((sx + ex) / 2.0, top - lift * 0.75, label));
            }
        }
    }
    // Edges between two node cards side by side in one frame run straight across the gap.
    let mut in_gap: Vec<usize> = Vec::new();
    for link in &sc.links {
        let (Some(ca), Some(cb)) = (
            resolve(&link.from).and_then(card_of),
            resolve(&link.to).and_then(card_of),
        ) else {
            continue;
        };
        let (Some(pa), Some(pb)) = (index(&link.from), index(&link.to)) else {
            continue;
        };
        if ca != cb || pa == pb {
            continue;
        }
        let Some(g) = cards[ca]
            .gaps
            .iter()
            .position(|g| (g.0 == pa && g.1 == pb) || (g.0 == pb && g.1 == pa))
        else {
            continue;
        };
        let (_, _, gl, gr, gt) = cards[ca].gaps[g];
        let k = in_gap.iter().filter(|&&x| x == g + 1000 * ca).count() as f64;
        in_gap.push(g + 1000 * ca);
        let x0 = items[ca].x - cards[ca].w / 2.0;
        let y = items[ca].y - cards[ca].h / 2.0 + gt + 26.0 + k * 26.0;
        let (x1, x2) = (x0 + gl + 1.0, x0 + gr - 1.0);
        let (sx, ex) = if cards[ca].gaps[g].0 == pa {
            (x1, x2)
        } else {
            (x2, x1)
        };
        let d = format!("M{} {}L{} {}", px(sx), px(y), px(ex), px(y));
        edge_path(&mut svg, &d, link);
        row_labels.push(((x1 + x2) / 2.0, y, edge_label(sc, link, false)));
    }
    svg.push_str("</g>");

    // Labels and channel cards.
    svg.push_str("<g class=\"labels\">");
    for (cx, cy, label) in &row_labels {
        pill(&mut svg, *cx, *cy, label, None);
    }
    for fan in &fans {
        if let Some(it) = fan.item(fan.layer) {
            pill(&mut svg, items[it].x, items[it].y, &fan.label, None);
        }
    }
    for bd in &bundles {
        let Some(&it) = bd
            .items
            .iter()
            .find(|&&it| matches!(items[it].kind, ItemKind::Label(_)))
        else {
            continue;
        };
        let item = &items[it];
        match bd.channel {
            None => {
                let link = &sc.links[bd.links[0]];
                pill(&mut svg, item.x, item.y, &edge_label(sc, link, false), None);
            }
            Some(c) => {
                let (wl, wr) = bd.side_widths(sc);
                let (cw, ch) = channel_card_size(&sc.parts[c]);
                let left_edge = item.x - item.w / 2.0;
                let right_edge = item.x + item.w / 2.0;
                let cl = left_edge + wl + bd.left.len() as f64 * LINE_GAP + 12.0;
                let ct = item.y - ch / 2.0;
                let line_l = left_edge + wl + (bd.left.len() as f64 - 0.5) * LINE_GAP;
                let line_r = right_edge - wr - (bd.right.len() as f64 - 0.5) * LINE_GAP;
                let _ = write!(
                    svg,
                    "<path class=\"over\" d=\"M{} {}H{}M{} {}H{}\"/>",
                    px(line_l),
                    px(item.y),
                    px(cl),
                    px(cl + cw),
                    px(item.y),
                    px(line_r)
                );
                for (side, ls) in [(true, &bd.left), (false, &bd.right)] {
                    let rows = ls.len() as f64;
                    for (j, &l) in ls.iter().enumerate() {
                        let text = &sc.links[l].label;
                        let w = text_w(text, 11.5, false) + 18.0;
                        let y = item.y + (j as f64 - (rows - 1.0) / 2.0) * (PILL_H + 4.0);
                        let x = if side {
                            left_edge + wl - 6.0 - w / 2.0
                        } else {
                            right_edge - wr + 6.0 + w / 2.0
                        };
                        pill(&mut svg, x, y, text, Some(w));
                    }
                }
                channel_card(&mut svg, sc, c, cl, ct, cw, ch);
            }
        }
    }
    svg.push_str("</g>");

    // Cards.
    svg.push_str("<g class=\"nodes\">");
    for (ci, card) in cards.iter().enumerate() {
        let it = &items[ci];
        draw_card(&mut svg, sc, card, it.x - card.w / 2.0, it.y - card.h / 2.0);
    }
    svg.push_str("</g></svg>");
    Graph { svg, width, height }
}

fn edge_path(svg: &mut String, d: &str, link: &Link) {
    let over = link
        .via
        .as_ref()
        .map(|o| format!(" over {o}"))
        .unwrap_or_default();
    let _ = write!(
        svg,
        "<path class=\"edge{}\" d=\"{d}\" data-from=\"{}\" data-to=\"{}\"{}><title>{} → {}: {}{}</title></path>",
        if link.multi { " multi" } else { "" },
        esc(&link.from),
        esc(&link.to),
        link.via
            .as_ref()
            .map(|o| format!(" data-over=\"{}\"", esc(o)))
            .unwrap_or_default(),
        esc(&link.from),
        esc(&link.to),
        esc(&link.label),
        esc(&over)
    );
}

fn pill(svg: &mut String, cx: f64, cy: f64, text: &str, w: Option<f64>) {
    let w = w.unwrap_or_else(|| text_w(text, 11.5, false) + 18.0);
    let (x, y) = (cx - w / 2.0, cy - PILL_H / 2.0);
    let _ = write!(
        svg,
        "<g class=\"pill\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{PILL_H}\" rx=\"10\" data-box=\"{} {} {} {PILL_H}\"/><text x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text></g>",
        px(x),
        px(y),
        px(w),
        px(x),
        px(y),
        px(w),
        px(cx),
        px(cy + 4.0),
        esc(text)
    );
}

fn icon_tile(svg: &mut String, icon: &str, x: f64, y: f64, size: f64) {
    let scale = size / 30.0;
    let _ = write!(
        svg,
        "<rect class=\"icon-bg\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\"/><path class=\"icon\" transform=\"translate({} {}) scale({})\" d=\"{}\"/>",
        px(x),
        px(y),
        px(size),
        px(size),
        px(7.0 * scale),
        px(x + 8.0 * scale),
        px(y + 8.0 * scale),
        (scale * 100.0).round() / 100.0,
        icon
    );
}

fn channel_card(svg: &mut String, sc: &Scene, c: usize, x: f64, y: f64, w: f64, h: f64) {
    let p = &sc.parts[c];
    let _ = write!(
        svg,
        "<g class=\"node channel f-{}\" data-part=\"{}\"><rect class=\"card-bg\" tabindex=\"0\" role=\"button\" aria-pressed=\"false\" aria-label=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"10\" data-box=\"{} {} {} {}\"/>",
        p.slot,
        esc(&p.id),
        esc(&p.description),
        px(x),
        px(y),
        px(w),
        px(h),
        px(x),
        px(y),
        px(w),
        px(h)
    );
    icon_tile(svg, p.icon, x + 8.0, y + 11.0, 22.0);
    let _ = write!(
        svg,
        "<text class=\"name sm\" x=\"{}\" y=\"{}\">{}</text><text class=\"sub\" x=\"{}\" y=\"{}\">{}</text></g>",
        px(x + 38.0),
        px(y + 19.0),
        esc(&p.label),
        px(x + 38.0),
        px(y + 34.0),
        esc(&p.subtitle)
    );
}

fn draw_card(svg: &mut String, sc: &Scene, card: &Card, x: f64, y: f64) {
    // Frames first, outermost first, so the cards they hold sit on them.
    for &(p, dx, dy, w, h) in &card.frames {
        let (label, heading, per) = frame_head(sc, p);
        let part = &sc.parts[p];
        let (fx, fy) = (x + dx, y + dy);
        let _ = write!(
            svg,
            "<g class=\"region{} f-{}\" data-region=\"{}\"><rect class=\"region-bg\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"12\"/>",
            if per { " per" } else { "" },
            part.slot,
            esc(&part.id),
            px(fx),
            px(fy),
            px(w),
            px(h)
        );
        let mut tx = fx + FPAD;
        if let Some(l) = label {
            let _ = write!(
                svg,
                "<text class=\"region-name\" x=\"{}\" y=\"{}\">{}</text>",
                px(tx),
                px(fy + 16.0),
                esc(&l)
            );
            tx += text_w(&l, 12.0, true) + 8.0;
        }
        let _ = write!(
            svg,
            "<text class=\"region-sub\" x=\"{}\" y=\"{}\">{}</text></g>",
            px(tx),
            px(fy + 16.0),
            esc(&heading)
        );
    }
    for &(p, dx, dy, w, h) in &card.nodes {
        draw_node(svg, sc, p, x + dx, y + dy, w, h);
    }
}

fn draw_node(svg: &mut String, sc: &Scene, part: usize, x: f64, y: f64, w: f64, h: f64) {
    let p = &sc.parts[part];
    let border = p.outline.is_some();
    let _ = write!(
        svg,
        "<g class=\"node f-{}{}{}\" data-part=\"{}\">",
        p.slot,
        if border { " on-border" } else { "" },
        if p.refused { " refused" } else { "" },
        esc(&p.id),
    );
    if border {
        let _ = write!(
            svg,
            "<rect class=\"border-mark\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"14\"/>",
            px(x - 5.0),
            px(y - 5.0),
            px(w + 10.0),
            px(h + 10.0)
        );
    }
    if p.refused {
        let off = if border { 9.0 } else { 5.0 };
        let _ = write!(
            svg,
            "<rect class=\"refusal-ring\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\"/>",
            px(x - off),
            px(y - off),
            px(w + 2.0 * off),
            px(h + 2.0 * off),
            px(10.0 + off)
        );
    }
    let _ = write!(
        svg,
        "<rect class=\"card-bg\" tabindex=\"0\" role=\"button\" aria-pressed=\"false\" aria-label=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"10\" data-box=\"{} {} {} {}\"/>",
        esc(&p.description),
        px(x),
        px(y),
        px(w),
        px(h),
        px(x),
        px(y),
        px(w),
        px(h)
    );
    icon_tile(svg, p.icon, x + PAD, y + PAD, ICON);
    let tx = x + PAD + ICON + 10.0;
    let _ = write!(
        svg,
        "<text class=\"name\" x=\"{}\" y=\"{}\">{}</text><text class=\"sub\" x=\"{}\" y=\"{}\">{}</text>",
        px(tx),
        px(y + PAD + 12.0),
        esc(&p.label),
        px(tx),
        px(y + PAD + 27.0),
        esc(&p.subtitle)
    );
    // Badges sit at the top right, the last one rightmost.
    let mut right = x + w - PAD;
    for t in p.badges.iter().rev() {
        let bw = badge_w(t);
        let bx = right - bw;
        let _ = write!(
            svg,
            "<g class=\"badge\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"18\" rx=\"9\"/><text x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text></g>",
            px(bx),
            px(y + PAD - 1.0),
            px(bw),
            px(bx + bw / 2.0),
            px(y + PAD + 12.0),
            esc(t)
        );
        right = bx - 8.0;
    }
    if let Some(tag) = &p.outline {
        let tw = tag_w(tag);
        let bx = x + w - PAD - tw;
        let _ = write!(
            svg,
            "<g class=\"tag\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"16\" rx=\"8\"/><text x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text></g>",
            px(bx),
            px(y + PAD + 16.0),
            px(tw),
            px(bx + tw / 2.0),
            px(y + PAD + 27.5),
            esc(tag)
        );
    }
    let mut cy = y + HEAD;
    for (k, v) in &p.props {
        let _ = write!(
            svg,
            "<text class=\"prop\" x=\"{}\" y=\"{}\"><tspan class=\"prop-k\">{}</tspan> {}</text>",
            px(x + PAD),
            px(cy + 12.0),
            esc(k),
            esc(v)
        );
        cy += PROP;
    }
    // The ports of a node sit on the left edge of its card, one row each. An unresolved port
    // leaves its row as a dashed stub that reaches nothing, named by the refusals on it.
    for port in &p.ports {
        let mut class = String::from("port");
        if port.refused {
            class.push_str(" refused");
        }
        if port.unresolved {
            class.push_str(" unresolved");
        }
        let _ = write!(
            svg,
            "<g class=\"{class}\" data-port=\"{}\"><title>{}</title>",
            esc(&port.address),
            esc(&port.title),
        );
        if port.unresolved {
            let _ = write!(
                svg,
                "<path class=\"unresolved-end\" d=\"M{} {}H{}\"/>",
                px(x - 5.0),
                px(cy + 8.5),
                px(x - 21.0)
            );
        }
        let _ = write!(
            svg,
            "<rect class=\"port-mark\" x=\"{}\" y=\"{}\" width=\"10\" height=\"10\" rx=\"2\"/><text class=\"prop\" x=\"{}\" y=\"{}\"><tspan class=\"prop-k\">port</tspan> {}",
            px(x - 5.0),
            px(cy + 3.5),
            px(x + PAD),
            px(cy + 12.0),
            esc(&port.own)
        );
        if let Some(v) = &port.via {
            let _ = write!(svg, " <tspan class=\"via\">{}</tspan>", esc(v));
        }
        if port.unresolved && !port.refusals.is_empty() {
            let _ = write!(
                svg,
                " <tspan class=\"rule\">{}</tspan>",
                esc(&port.refusals.join(", "))
            );
        }
        svg.push_str("</text></g>");
        cy += PROP;
    }
    for chip in &p.chips {
        chip.svg(svg, x + PAD, cy + 2.0);
        cy += CHIP;
    }
    svg.push_str("</g>");
}
