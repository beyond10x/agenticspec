//! The architecture as an SVG graph, laid out deterministically.
//!
//! Parts sit in rows by block layer. Between two rows lies a label row, where every edge that
//! crosses it passes through an item of its own width: its label, a channel card, or a narrow
//! gap. Items are ordered by barycentre sweeps and placed by isotonic regression, so no two items
//! of a row overlap and every edge label has the room its text needs.
//!
//! A large architecture stays legible at its natural size: links with one label into or out of
//! one card form a fan, with one label beside the card and one trunk through the rows between,
//! which each link joins on a track of its own label row; a card that several cards of its row
//! send to drops to a row of its own; the passing items of a long line take one x where they fit,
//! and a line leaves a card shorter than its row straight, so no line crosses a card.

use crate::fmt::{self, esc, mono_w, px, text_w};
use crate::model::{Architecture, Outcome, Part, Place};
use std::fmt::Write as _;

const MARGIN: f64 = 18.0;
const PAD: f64 = 12.0;
const ICON: f64 = 30.0;
const HEAD: f64 = 52.0;
const PROP: f64 = 17.0;
const CHIP: f64 = 25.0;
const CHILD_HEAD: f64 = 18.0;
const CHILD: f64 = 38.0;
const CHILD_GAP: f64 = 6.0;
const MIN_W: f64 = 168.0;
const LINE_GAP: f64 = 12.0;
const ROW_GAP: f64 = 20.0;
const PILL_H: f64 = 20.0;

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

/// Names of the colour families, in legend order, with what they hold.
pub const FAMILIES: [(&str, &str); 10] = [
    ("principal", "principal"),
    ("agent", "agent, harness"),
    ("collab", "channel, question"),
    ("work", "work, authority"),
    ("intel", "gateway, model"),
    ("capability", "tool, knowledge"),
    ("place", "workspace, sandbox, runtime"),
    ("resource", "pool, resource"),
    ("nested", "nested architecture"),
    ("other", "other block"),
];

/// A 14 by 14 stroke icon for a block.
pub fn icon(block: &str) -> &'static str {
    match block {
        "principal" | "delegation" => {
            "M7 6.6a2.4 2.4 0 1 0 0-4.8 2.4 2.4 0 0 0 0 4.8zM2.4 12.6c0-2.6 2-4.2 4.6-4.2s4.6 1.6 4.6 4.2"
        }
        "agent" | "harness" => {
            "M3.5 4.5h7a1.5 1.5 0 0 1 1.5 1.5v4.5a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 2 10.5V6a1.5 1.5 0 0 1 1.5-1.5zM7 4.5V2M5.2 8.2h.01M8.8 8.2h.01"
        }
        "channel" | "question" => "M2 2.5h10v7H6.5L3.5 12V9.5H2z",
        "gateway" => "M2 2.5v9M5 7h7M9 4l3 3-3 3",
        "model" => {
            "M4 4h6v6H4zM5.5 1.5V4M8.5 1.5V4M5.5 10v2.5M8.5 10v2.5M1.5 5.5H4M1.5 8.5H4M10 5.5h2.5M10 8.5h2.5"
        }
        "tool" | "knowledge" => {
            "M9.6 1.8a3 3 0 0 0-3.4 4.1L2 10.1 3.9 12l4.2-4.2a3 3 0 0 0 4.1-3.4L10.4 6.2 8.6 5.4 7.8 3.6z"
        }
        "workspace" => "M1.5 3.5h4l1.5 1.5h5.5v6.5h-11z",
        "sandbox" => "M7 1.5l5 2v3.6c0 2.9-2.1 4.7-5 5.4-2.9-.7-5-2.5-5-5.4V3.5z",
        "runtime" => "M4 2.5v9l7.5-4.5z",
        "pool" | "resource" => "M2 2.5h10v2.5H2zM2 6.25h10v2.5H2zM2 10h10v2H2z",
        "work" | "process" => "M2 3.8l1.4 1.4L6 2.6M2 9l1.4 1.4L6 7.8M8 4.2h4.5M8 9.4h4.5",
        "authority" => "M3.5 6.5h7v5.5h-7zM5 6.5V4.8a2 2 0 0 1 4 0v1.7",
        "architecture" => "M1.5 2.5h11v9h-11zM4 5h6v4H4zM1.5 7H4M10 7h2.5",
        _ => "M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10z",
    }
}

/// What a chip at a marked place shows.
pub struct Chip {
    pub path: String,
    pub field: String,
    pub value: String,
    pub best: Option<String>,
    pub changed: bool,
    pub range: String,
}

impl Chip {
    pub fn new(place: &Place, outcome: Option<&Outcome>) -> Self {
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
                && let Some(m) = o
                    .marked
                    .iter()
                    .find(|m| crate::model::norm(&m.path) == crate::model::norm(&place.path))
            {
                range = crate::model::range_text(&m.range);
            }
        }
        let changed = best.as_ref().is_some_and(|b| *b != value);
        Chip {
            path: place.path.clone(),
            field: place.field.clone(),
            value,
            best,
            changed,
            range,
        }
    }

    fn width(&self) -> f64 {
        let mut w = 9.0 + text_w(&self.field, 11.5, false) + 7.0 + mono_w(&self.value, 11.5);
        if let Some(b) = &self.best {
            w += 6.0 + mono_w("→", 11.5) + 6.0 + mono_w(b, 11.5) + 4.0;
        }
        if !self.range.is_empty() {
            w += 7.0 + mono_w(&format!("[{}]", self.range), 11.0);
        }
        w + 9.0
    }

    fn attrs(&self) -> String {
        match &self.best {
            Some(b) => format!(
                "data-place=\"{}\" data-baseline=\"{}\" data-best=\"{}\"",
                esc(&self.path),
                esc(&self.value),
                esc(b)
            ),
            None => format!(
                "data-place=\"{}\" data-value=\"{}\"",
                esc(&self.path),
                esc(&self.value)
            ),
        }
    }

    /// The chip as SVG, its left edge at `x`, its top at `y`.
    fn svg(&self, out: &mut String, x: f64, y: f64) {
        let w = self.width();
        let class = if self.changed { "chip changed" } else { "chip" };
        let _ = write!(
            out,
            "<g class=\"{class}\" {}><title>{}</title><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{PILL_H}\" rx=\"10\"/>",
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
        cx += text_w(&self.field, 11.5, false) + 7.0;
        let was = if self.changed { "chip-v was" } else { "chip-v" };
        let _ = write!(
            out,
            "<text class=\"{was}\" x=\"{}\" y=\"{}\">{}</text>",
            px(cx),
            px(base),
            esc(&self.value)
        );
        cx += mono_w(&self.value, 11.5);
        if let Some(b) = &self.best {
            cx += 6.0;
            let _ = write!(
                out,
                "<text class=\"chip-arrow\" x=\"{}\" y=\"{}\">→</text>",
                px(cx),
                px(base)
            );
            cx += mono_w("→", 11.5) + 6.0;
            let now = if self.changed { "chip-v now" } else { "chip-v" };
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
        }
        out.push_str("</g>");
    }

    pub fn describe(&self) -> String {
        let mut s = format!("{} {}", self.field, self.value);
        if let Some(b) = &self.best {
            s.push_str(&format!(", best {b}"));
        }
        if !self.range.is_empty() {
            s.push_str(&format!(", range {}", self.range));
        }
        s
    }

    /// The chip as HTML, for the side panel.
    pub fn html(&self, with_field: bool) -> String {
        let field = if with_field {
            format!("<span class=\"chip-k\">{}</span> ", esc(&self.field))
        } else {
            String::new()
        };
        let mut s = format!(
            "<span class=\"chip{}\" {}>{field}<span class=\"chip-v{}\">{}</span>",
            if self.changed { " changed" } else { "" },
            self.attrs(),
            if self.changed { " was" } else { "" },
            esc(&self.value)
        );
        if let Some(b) = &self.best {
            s.push_str(&format!(
                " <span class=\"chip-arrow\">→</span> <span class=\"chip-v{}\">{}</span>",
                if self.changed { " now" } else { "" },
                esc(b)
            ));
        }
        if !self.range.is_empty() {
            s.push_str(&format!(
                " <span class=\"chip-r\">[{}]</span>",
                esc(&self.range)
            ));
        }
        s.push_str("</span>");
        s
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

struct Card {
    part: usize,
    w: f64,
    h: f64,
    children: Vec<usize>,
    chips: Vec<Chip>,
    child_chips: Vec<Vec<Chip>>,
    props: Vec<(String, String)>,
    /// The ports of a nested architecture: full address and the border part's own name.
    ports: Vec<(String, String)>,
    border: bool,
    row: usize,
}

fn child_sub(p: &Part) -> String {
    match (p.kind.is_none() && sub_prop(p).is_none(), p.props.first()) {
        (true, Some((_, v))) => format!("{} · {v}", p.block),
        _ => subtitle(p),
    }
}

impl Card {
    fn new(arch: &Architecture, part: usize, children: Vec<usize>, o: Option<&Outcome>) -> Self {
        let p = &arch.parts[part];
        // A nested architecture is on the border when one of its ports is.
        let border = arch.border.contains(&p.name)
            || p.nested
                .as_ref()
                .is_some_and(|n| n.ports.iter().any(|a| arch.border.contains(a)));
        let mut props = Vec::new();
        if let Some(a) = &p.acts_for {
            props.push(("acts for".to_string(), a.clone()));
        }
        let shown_in_sub = subtitle(p);
        let in_sub = sub_key(p);
        for (k, v) in p.props.iter().filter(|(k, _)| Some(k.as_str()) != in_sub) {
            props.push((k.replace('_', " "), v.clone()));
        }
        let ports: Vec<(String, String)> = p
            .nested
            .iter()
            .flat_map(|n| n.ports.iter())
            .map(|a| {
                let own = a
                    .strip_prefix(&p.name)
                    .and_then(|r| r.strip_prefix('.'))
                    .unwrap_or(a);
                (a.clone(), own.to_string())
            })
            .collect();
        let chips: Vec<Chip> = p.places.iter().map(|pl| Chip::new(pl, o)).collect();
        let child_chips: Vec<Vec<Chip>> = children
            .iter()
            .map(|&c| {
                arch.parts[c]
                    .places
                    .iter()
                    .map(|pl| Chip::new(pl, o))
                    .collect()
            })
            .collect();
        let name_w = text_w(&p.name, 13.5, true);
        let badge = p.count.as_ref().map(|c| badge_w(c) + 8.0).unwrap_or(0.0);
        let tag = if border { tag_w() + 8.0 } else { 0.0 };
        let head = ICON + 10.0 + (name_w + badge).max(text_w(&shown_in_sub, 11.5, false) + tag);
        let mut inner = head;
        for (k, v) in &props {
            inner = inner.max(text_w(k, 11.5, false) + 6.0 + text_w(v, 11.5, false));
        }
        for (_, own) in &ports {
            inner = inner.max(text_w(&format!("port {own}"), 11.5, false) + 8.0);
        }
        for c in &chips {
            inner = inner.max(c.width());
        }
        for (i, &c) in children.iter().enumerate() {
            let cp = &arch.parts[c];
            let w = 8.0
                + 20.0
                + 8.0
                + text_w(&cp.name, 12.0, true).max(text_w(&child_sub(cp), 11.0, false))
                + 10.0;
            inner = inner.max(w);
            for ch in &child_chips[i] {
                inner = inner.max(ch.width() + 16.0);
            }
        }
        inner = inner.max(text_w(&format!("per {}", p.name), 11.0, false));
        let w = (inner + 2.0 * PAD).max(MIN_W).ceil();
        let mut h = HEAD;
        h += props.len() as f64 * PROP;
        h += ports.len() as f64 * PROP;
        h += chips.len() as f64 * CHIP;
        if !children.is_empty() {
            h += CHILD_HEAD;
            for cc in &child_chips {
                h += CHILD + cc.len() as f64 * CHIP + CHILD_GAP;
            }
        }
        let h = if h == HEAD { HEAD } else { h + 6.0 };
        Card {
            part,
            w,
            h,
            children,
            chips,
            child_chips,
            props,
            ports,
            border,
            row: family(&p.block).1,
        }
    }
}

fn badge_w(count: &str) -> f64 {
    mono_w(&format!("×{count}"), 11.0) + 12.0
}

fn tag_w() -> f64 {
    text_w("border", 11.0, false) + 14.0
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

/// Links between two cards on different rows, drawn together.
struct Bundle {
    upper: usize,
    lower: usize,
    links: Vec<usize>,
    channel: Option<usize>,
    label_layer: usize,
    items: Vec<usize>,
    /// For a channel bundle: which links pass left of the card.
    left: Vec<usize>,
    right: Vec<usize>,
    /// The fan this single link shares its label with.
    fan: Option<usize>,
}

/// Single links with one label at one side of one card. They share one label beside the card
/// and one trunk from there through every row between: each link joins the trunk on a track of
/// the label row next to its other card, so the links draw as a comb, not as parallel lines.
struct Fan {
    card: usize,
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
    fn label_size(&self, arch: &Architecture) -> (f64, f64) {
        match self.channel {
            None => {
                let l = &arch.links[self.links[0]];
                (
                    text_w(&edge_label(arch, l, false), 11.5, false) + 18.0,
                    PILL_H,
                )
            }
            Some(c) => {
                let (wl, wr) = self.side_widths(arch);
                let (cw, ch) = channel_card_size(&arch.parts[c]);
                let nl = self.left.len() as f64;
                let nr = self.right.len() as f64;
                let w = wl + nl * LINE_GAP + 12.0 + cw + 12.0 + nr * LINE_GAP + wr;
                let rows = self.left.len().max(self.right.len()) as f64;
                (w, ch.max(rows * (PILL_H + 4.0)))
            }
        }
    }

    fn side_widths(&self, arch: &Architecture) -> (f64, f64) {
        let side = |ls: &[usize]| {
            ls.iter()
                .map(|&l| text_w(&arch.links[l].carries, 11.5, false) + 18.0 + 6.0)
                .fold(0.0, f64::max)
        };
        (side(&self.left), side(&self.right))
    }
}

fn channel_card_size(p: &Part) -> (f64, f64) {
    let w = 8.0 + 22.0 + 8.0 + text_w(&p.name, 13.0, true).max(text_w(&subtitle(p), 11.0, false));
    ((w + 12.0).ceil(), 44.0)
}

/// The border part a link end addresses on a nested architecture, by its own name.
fn port_of<'a>(arch: &Architecture, end: &'a str) -> Option<&'a str> {
    let (head, own) = end.split_once('.')?;
    arch.part(head)?
        .nested
        .as_ref()?
        .ports
        .iter()
        .any(|p| p == end)
        .then_some(own)
}

fn edge_label(arch: &Architecture, l: &crate::model::Link, on_channel: bool) -> String {
    let base = match (&l.over, on_channel) {
        (Some(o), false) if arch.part(o).is_some() => format!("{} via {}", l.carries, o),
        _ => l.carries.clone(),
    };
    match port_of(arch, &l.to).or_else(|| port_of(arch, &l.from)) {
        Some(port) => format!("{base} · {port}"),
        None => base,
    }
}

/// Links between two cards of one row.
struct Level {
    a: usize,
    b: usize,
    links: Vec<usize>,
}

pub struct Graph {
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

pub fn render(arch: &Architecture, outcome: Option<&Outcome>) -> Graph {
    let n = arch.parts.len();
    // A link end `<part>.<border part>` lands on the box of the nested architecture `<part>`.
    let index = |name: &str| {
        arch.parts.iter().position(|p| p.name == name).or_else(|| {
            let (head, _) = name.split_once('.')?;
            arch.parts
                .iter()
                .position(|p| p.name == head && p.nested.is_some())
        })
    };
    let endpoints: Vec<&str> = arch
        .links
        .iter()
        .flat_map(|l| [l.from.as_str(), l.to.as_str()])
        .collect();

    // A part with `per` is drawn inside the part it belongs to.
    let mut host: Vec<Option<usize>> = vec![None; n];
    for (i, p) in arch.parts.iter().enumerate() {
        let mut cur = i;
        for _ in 0..8 {
            match arch.parts[cur].per.as_deref().and_then(index) {
                Some(next) if next != i => cur = next,
                _ => break,
            }
        }
        if cur != i && p.per.is_some() {
            host[i] = Some(cur);
        }
    }
    let row_of = |i: usize| family(&arch.parts[i].block).1;
    let resolve = |name: &str| index(name).map(|i| host[i].unwrap_or(i));

    // A part that is only ever the channel of links between two rows is drawn on those links.
    let mut on_edge = vec![false; n];
    for (i, p) in arch.parts.iter().enumerate() {
        if host[i].is_some() || p.per.is_some() || endpoints.contains(&p.name.as_str()) {
            continue;
        }
        on_edge[i] = arch.links.iter().any(|l| {
            l.over.as_deref() == Some(p.name.as_str())
                && match (resolve(&l.from), resolve(&l.to)) {
                    (Some(a), Some(b)) => row_of(a) != row_of(b),
                    _ => false,
                }
        });
    }

    // Cards in order of first appearance in the links, then in the order of the document.
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
    let mut cards: Vec<Card> = order
        .iter()
        .map(|&i| {
            let children = (0..n).filter(|&c| host[c] == Some(i)).collect();
            Card::new(arch, i, children, outcome)
        })
        .collect();
    let card_of = |part: usize| order.iter().position(|&c| c == part);

    // A card that two or more cards of its own row send to moves to a row of its own just
    // below: its links then cross rows, where they share a label, instead of arching over the
    // cards between.
    let ends: Vec<(usize, usize)> = arch
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
    for (li, l) in arch.links.iter().enumerate() {
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
        let channel = l.over.as_deref().and_then(index).filter(|&c| on_edge[c]);
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
        let mut keys: Vec<(usize, bool, String, usize)> = Vec::new();
        for (bi, bd) in bundles.iter().enumerate() {
            if bd.channel.is_some() || bd.fan.is_some() {
                continue;
            }
            let l = &arch.links[bd.links[0]];
            let Some(c) = resolve(if at_end { &l.to } else { &l.from }).and_then(card_of) else {
                continue;
            };
            keys.push((c, c == bd.lower, edge_label(arch, l, false), bi));
        }
        for &(c, at_lower, ref label, _) in &keys {
            if fans
                .iter()
                .any(|f| f.card == c && f.at_lower == at_lower && f.label == *label)
            {
                continue;
            }
            let members: Vec<usize> = keys
                .iter()
                .filter(|k| k.0 == c && k.1 == at_lower && k.2 == *label)
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
                let (w, h) = bd.label_size(arch);
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
                    .map(|&l| text_w(&edge_label(arch, &arch.links[l], false), 11.5, false) + 18.0)
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
                        let (wl, _) = bd.side_widths(arch);
                        let left_edge = item.x - item.w / 2.0;
                        let right_edge = item.x + item.w / 2.0;
                        let (_, wr) = bd.side_widths(arch);
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
    let mut ports: Vec<(usize, bool, usize, f64)> = Vec::new(); // card, bottom side, key, toward x
    for (ri, r) in routes.iter().enumerate() {
        let bd = &bundles[r.bundle];
        let first = r.pts.first().map(|p| p.0).unwrap_or(items[bd.lower].x);
        let last = r.pts.last().map(|p| p.0).unwrap_or(items[bd.upper].x);
        for (card, side, toward) in [(bd.upper, true, first), (bd.lower, false, last)] {
            let key = port_key(ri, side);
            if !ports.iter().any(|p| p.2 == key && p.1 == side) {
                ports.push((card, side, key, toward));
            }
        }
    }
    let mut port_x: Vec<(usize, bool, f64)> = Vec::new(); // port key, upper end, x
    for ci in 0..cards.len() {
        for side in [true, false] {
            let mut ends: Vec<&(usize, bool, usize, f64)> =
                ports.iter().filter(|p| p.0 == ci && p.1 == side).collect();
            ends.sort_by(|a, b| a.3.total_cmp(&b.3).then(a.2.cmp(&b.2)));
            let m = ends.len();
            let span = if m > 1 {
                (cards[ci].w - 40.0).min(26.0 * (m as f64 - 1.0))
            } else {
                0.0
            };
            for (j, e) in ends.iter().enumerate() {
                let x = if m > 1 {
                    items[ci].x - span / 2.0 + span * j as f64 / (m as f64 - 1.0)
                } else {
                    items[ci].x
                };
                port_x.push((e.2, side, x));
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
        "<title id=\"graph-title\">Architecture {}</title><desc id=\"graph-desc\">{} parts in rows by block layer, {} links as arrows labelled by what they carry. The tables of parts and links beside the graph list every part and every link as text.</desc>",
        esc(&arch.name),
        arch.parts.len(),
        arch.links.len()
    );
    svg.push_str(
        "<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0 0L10 5L0 10z\" class=\"arrowhead\"/></marker><marker id=\"arrow-hot\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0 0L10 5L0 10z\" class=\"arrowhead hot\"/></marker></defs>",
    );

    // Edges across rows.
    svg.push_str("<g class=\"edges\">");
    for (ri, r) in routes.iter().enumerate() {
        let bd = &bundles[r.bundle];
        let link = &arch.links[r.link];
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
        let upper_bottom = items[bd.upper].y + items[bd.upper].h / 2.0;
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
        let lower_top = items[bd.lower].y - items[bd.lower].h / 2.0 - 1.0;
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
            let link = &arch.links[li];
            let from_left = resolve(&link.from).and_then(card_of) == Some(l);
            let label = edge_label(arch, link, false);
            if adjacent {
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
                let (sx, ex) = if from_left {
                    (items[l].x, items[r].x)
                } else {
                    (items[r].x, items[l].x)
                };
                let d = format!(
                    "M{} {}C{} {} {} {} {} {}",
                    px(sx),
                    px(top),
                    px(sx),
                    px(top - lift),
                    px(ex),
                    px(top - lift),
                    px(ex),
                    px(top - 1.0)
                );
                edge_path(&mut svg, &d, link);
                row_labels.push(((items[l].x + items[r].x) / 2.0, top - lift * 0.75, label));
            }
        }
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
                let link = &arch.links[bd.links[0]];
                pill(
                    &mut svg,
                    item.x,
                    item.y,
                    &edge_label(arch, link, false),
                    None,
                );
            }
            Some(c) => {
                let (wl, wr) = bd.side_widths(arch);
                let (cw, ch) = channel_card_size(&arch.parts[c]);
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
                        let text = &arch.links[l].carries;
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
                channel_card(&mut svg, arch, c, cl, ct, cw, ch);
            }
        }
    }
    svg.push_str("</g>");

    // Cards.
    svg.push_str("<g class=\"nodes\">");
    for (ci, card) in cards.iter().enumerate() {
        let it = &items[ci];
        draw_card(
            &mut svg,
            arch,
            card,
            it.x - card.w / 2.0,
            it.y - card.h / 2.0,
        );
    }
    svg.push_str("</g></svg>");
    Graph { svg, width, height }
}

fn edge_path(svg: &mut String, d: &str, link: &crate::model::Link) {
    let over = link
        .over
        .as_ref()
        .map(|o| format!(" over {o}"))
        .unwrap_or_default();
    let _ = write!(
        svg,
        "<path class=\"edge\" d=\"{d}\" data-from=\"{}\" data-to=\"{}\"{}><title>{} → {}: {}{}</title></path>",
        esc(&link.from),
        esc(&link.to),
        link.over
            .as_ref()
            .map(|o| format!(" data-over=\"{}\"", esc(o)))
            .unwrap_or_default(),
        esc(&link.from),
        esc(&link.to),
        esc(&link.carries),
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

fn icon_tile(svg: &mut String, block: &str, x: f64, y: f64, size: f64) {
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
        icon(block)
    );
}

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

fn channel_card(svg: &mut String, arch: &Architecture, c: usize, x: f64, y: f64, w: f64, h: f64) {
    let p = &arch.parts[c];
    let (fam, _) = family(&p.block);
    let _ = write!(
        svg,
        "<g class=\"node channel f-{fam}\" data-part=\"{}\"><rect class=\"card-bg\" tabindex=\"0\" role=\"button\" aria-pressed=\"false\" aria-label=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"10\" data-box=\"{} {} {} {}\"/>",
        esc(&p.name),
        esc(&label_for(arch, p)),
        px(x),
        px(y),
        px(w),
        px(h),
        px(x),
        px(y),
        px(w),
        px(h)
    );
    icon_tile(svg, &p.block, x + 8.0, y + 11.0, 22.0);
    let _ = write!(
        svg,
        "<text class=\"name sm\" x=\"{}\" y=\"{}\">{}</text><text class=\"sub\" x=\"{}\" y=\"{}\">{}</text></g>",
        px(x + 38.0),
        px(y + 19.0),
        esc(&p.name),
        px(x + 38.0),
        px(y + 34.0),
        esc(&subtitle(p))
    );
}

fn draw_card(svg: &mut String, arch: &Architecture, card: &Card, x: f64, y: f64) {
    let p = &arch.parts[card.part];
    let (fam, _) = family(&p.block);
    let _ = write!(
        svg,
        "<g class=\"node f-{fam}{}\" data-part=\"{}\">",
        if card.border { " on-border" } else { "" },
        esc(&p.name),
    );
    if card.border {
        let _ = write!(
            svg,
            "<rect class=\"border-mark\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"14\"/>",
            px(x - 5.0),
            px(y - 5.0),
            px(card.w + 10.0),
            px(card.h + 10.0)
        );
    }
    let _ = write!(
        svg,
        "<rect class=\"card-bg\" tabindex=\"0\" role=\"button\" aria-pressed=\"false\" aria-label=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"10\" data-box=\"{} {} {} {}\"/>",
        esc(&label_for(arch, p)),
        px(x),
        px(y),
        px(card.w),
        px(card.h),
        px(x),
        px(y),
        px(card.w),
        px(card.h)
    );
    icon_tile(svg, &p.block, x + PAD, y + PAD, ICON);
    let tx = x + PAD + ICON + 10.0;
    let _ = write!(
        svg,
        "<text class=\"name\" x=\"{}\" y=\"{}\">{}</text><text class=\"sub\" x=\"{}\" y=\"{}\">{}</text>",
        px(tx),
        px(y + PAD + 12.0),
        esc(&p.name),
        px(tx),
        px(y + PAD + 27.0),
        esc(&subtitle(p))
    );
    if let Some(c) = &p.count {
        let t = format!("×{c}");
        let bw = badge_w(c);
        let bx = x + card.w - PAD - bw;
        let _ = write!(
            svg,
            "<g class=\"badge\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"18\" rx=\"9\"/><text x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text></g>",
            px(bx),
            px(y + PAD - 1.0),
            px(bw),
            px(bx + bw / 2.0),
            px(y + PAD + 12.0),
            esc(&t)
        );
    }
    if card.border {
        let tw = tag_w();
        let bx = x + card.w - PAD - tw;
        let _ = write!(
            svg,
            "<g class=\"tag\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"16\" rx=\"8\"/><text x=\"{}\" y=\"{}\" text-anchor=\"middle\">border</text></g>",
            px(bx),
            px(y + PAD + 16.0),
            px(tw),
            px(bx + tw / 2.0),
            px(y + PAD + 27.5)
        );
    }
    let mut cy = y + HEAD;
    for (k, v) in &card.props {
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
    // The ports of a nested architecture sit on the left edge of its box, one row each.
    for (address, own) in &card.ports {
        let _ = write!(
            svg,
            "<g class=\"port\" data-port=\"{}\"><title>{}</title><rect class=\"port-mark\" x=\"{}\" y=\"{}\" width=\"10\" height=\"10\" rx=\"2\"/><text class=\"prop\" x=\"{}\" y=\"{}\"><tspan class=\"prop-k\">port</tspan> {}</text></g>",
            esc(address),
            esc(&format!("border part {address}")),
            px(x - 5.0),
            px(cy + 3.5),
            px(x + PAD),
            px(cy + 12.0),
            esc(own)
        );
        cy += PROP;
    }
    for chip in &card.chips {
        chip.svg(svg, x + PAD, cy + 2.0);
        cy += CHIP;
    }
    if !card.children.is_empty() {
        let _ = write!(
            svg,
            "<text class=\"per-head\" x=\"{}\" y=\"{}\">per {}</text>",
            px(x + PAD),
            px(cy + 12.0),
            esc(&p.name)
        );
        cy += CHILD_HEAD;
        for (i, &c) in card.children.iter().enumerate() {
            let cp = &arch.parts[c];
            let (cfam, _) = family(&cp.block);
            let chips = &card.child_chips[i];
            let h = CHILD + chips.len() as f64 * CHIP;
            let w = card.w - 2.0 * PAD;
            let _ = write!(
                svg,
                "<g class=\"node child f-{cfam}\" data-part=\"{}\"><rect class=\"child-bg\" tabindex=\"0\" role=\"button\" aria-pressed=\"false\" aria-label=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"8\"/>",
                esc(&cp.name),
                esc(&label_for(arch, cp)),
                px(x + PAD),
                px(cy),
                px(w),
                px(h)
            );
            icon_tile(svg, &cp.block, x + PAD + 8.0, cy + 9.0, 20.0);
            let _ = write!(
                svg,
                "<text class=\"name xs\" x=\"{}\" y=\"{}\">{}</text><text class=\"sub xs\" x=\"{}\" y=\"{}\">{}</text>",
                px(x + PAD + 36.0),
                px(cy + 16.0),
                esc(&cp.name),
                px(x + PAD + 36.0),
                px(cy + 30.0),
                esc(&child_sub(cp))
            );
            let mut chy = cy + CHILD;
            for chip in chips {
                chip.svg(svg, x + PAD + 8.0, chy);
                chy += CHIP;
            }
            svg.push_str("</g>");
            cy += h + CHILD_GAP;
        }
    }
    svg.push_str("</g>");
}
