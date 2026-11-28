//! The render model `agenticspec-render-model/1` and its one entry point, [`render`].
//!
//! A [`Page`] is a graph in neutral terms: nodes in groups, ports on nodes, edges between them,
//! regions that hold nodes, marks (a refusal or a piece of information) on a node or a port, and
//! panels of plain tables. [`render`] draws it as one complete HTML page in this crate's visual
//! language: the graph at its natural size with pan, zoom and fit, its legend, and the panels
//! beside it. The page holds its CSS, its script and its SVG inline, embeds the page document as
//! JSON, and loads nothing from the network. The output is a function of the page only.
//!
//! The JSON Schema of the format is `schema/render-model.schema.json` in this crate. Unknown
//! fields are refused, by the schema and by the types.
//!
//! How it draws:
//!
//! - A node is a card coloured by its `group`. Rows run top to bottom by the longest chain of
//!   edges that leads to a node; edges between two rows carry their label in the row between.
//! - A port is one row of its node's card; an edge end `node.port` lands on that card and its
//!   label names the port.
//! - A node with a `region` is drawn as a small card inside the card of the outermost region it
//!   lies in: the node whose id is the region's id, or a card of the region itself. Its edges land
//!   on that card.
//! - A mark is a chip on its card. A refusal also rings the card, or marks its port.
//! - A panel is a table in a card beside the graph.
//!
//! One page:
//!
//! ```json
//! {
//!   "format": "agenticspec-render-model/1",
//!   "title": "assistant",
//!   "summary": "One session per user; the session calls a model.",
//!   "regions": [{ "id": "session", "label": "session", "per": "session" }],
//!   "nodes": [
//!     { "id": "user", "label": "user", "group": "principal" },
//!     { "id": "session", "label": "session", "group": "agent", "badges": [{ "name": "count", "value": "8" }] },
//!     { "id": "scratch", "label": "scratch", "group": "place", "region": "session" },
//!     { "id": "model", "label": "model", "group": "intel" }
//!   ],
//!   "ports": [{ "node": "model", "name": "complete", "kind": "http", "direction": "in", "contract": "complete/1" }],
//!   "edges": [
//!     { "from": "user", "to": "session", "label": "prompt" },
//!     { "from": "session", "to": "model.complete", "label": "call", "count": 2 }
//!   ],
//!   "marks": [
//!     { "target": { "node": "model", "port": "complete" }, "name": "rate_limit", "detail": "over 60 a minute" },
//!     { "target": { "node": "session" }, "name": "budget", "detail": "40 % used", "level": "info" }
//!   ],
//!   "panels": [{ "title": "Owners", "columns": ["node", "owner"], "rows": [["session", "team a"], ["model", "team b"]] }]
//! }
//! ```

use crate::fmt::{self, esc};
use crate::graph;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

const CSS: &str = include_str!("../assets/page.css");
const JS: &str = include_str!("../assets/page.js");

/// The format name, the value of `format` of every [`Page`].
pub const FORMAT: &str = "agenticspec-render-model/1";

/// One page: a titled graph and the panels beside it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// Always [`FORMAT`].
    pub format: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Containment, as a tree. A node is drawn inside the outermost region its `region` lies in.
    #[serde(default)]
    pub regions: Vec<Region>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub ports: Vec<Port>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub marks: Vec<Mark>,
    #[serde(default)]
    pub panels: Vec<Panel>,
}

impl Page {
    /// An empty page of format [`FORMAT`] with this title.
    pub fn new(title: impl Into<String>) -> Self {
        Page {
            format: FORMAT.to_string(),
            title: title.into(),
            ..Page::default()
        }
    }
}

/// A region: a box that holds nodes. A region whose `id` is a node's id is that node's card; any
/// other outermost region is drawn as a card of its own. The nodes inside are headed `per <per>`
/// or `in <in>` (`in <label>` when neither is given).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#in: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per: Option<String>,
    /// The regions inside this one.
    #[serde(default)]
    pub regions: Vec<Region>,
}

/// A node: one card of the graph.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub label: String,
    /// Colour and legend entry. One of the palette names of the stylesheet (`principal`,
    /// `agent`, `collab`, `work`, `intel`, `capability`, `place`, `resource`, `nested`, `other`)
    /// takes that colour; any other name takes the next one no group of the page names.
    pub group: String,
    /// Pills at the card's top right: `count` reads `×<value>`, any other `<name> <value>`.
    #[serde(default)]
    pub badges: Vec<Badge>,
    /// The id of the region the node is drawn in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
    pub name: String,
    pub value: String,
}

/// A named port on a node, addressed by an edge end `node.port`. A port is one row of its
/// node's card.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub node: String,
    pub name: String,
    pub kind: String,
    pub direction: Direction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,
    /// Text on the port row after its name, such as the protocols it runs over.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    /// The port names a target that does not resolve: a dashed stub leaves its row and reaches
    /// nothing; a refusal mark on the port names why.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unresolved: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    In,
    Out,
    #[default]
    Both,
}

/// An edge from one node (or `node.port`) to another, drawn as an arrow with its label.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: String,
    /// How many edges this one stands for, shown as `×n` after the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}

/// What a mark is shown on.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
}

/// A mark on a node or a port, drawn as a chip on the card. A refusal also rings its card (or
/// its port); a piece of information does not.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mark {
    pub target: Target,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub level: Level,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    #[default]
    Refusal,
}

/// A plain table beside the graph.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// The page of `page`: one complete HTML document.
pub fn render(page: &Page) -> String {
    render_with(page, &Shell::of(page))
}

/// The header of a page: an eyebrow line, the heading, a summary and a list of facts.
pub(crate) struct Header {
    pub eyebrow: Option<String>,
    pub heading: String,
    pub summary: Option<String>,
    pub facts: Vec<String>,
}

/// Everything around the graph of a page. [`render`] builds it from the page alone; a host page
/// of this crate writes its own.
pub(crate) struct Shell {
    /// The document title.
    pub title: String,
    pub header: Header,
    /// The accessible name of the section that holds the graph.
    pub section: String,
    /// The heading of the graph's card and its hint.
    pub graph_heading: String,
    pub graph_hint: String,
    /// The legend, HTML.
    pub legend: String,
    /// The cards beside the graph, HTML.
    pub side: String,
    /// The sections after the graph's, HTML.
    pub after: String,
    /// The footer, HTML.
    pub footer: String,
    /// The JSON embedded in the page.
    pub data: serde_json::Value,
    /// What the graph draws beyond the page.
    pub decor: graph::Decor,
}

impl Shell {
    fn of(page: &Page) -> Self {
        let refusals = page
            .marks
            .iter()
            .filter(|m| m.level == Level::Refusal)
            .count();
        let mut facts = vec![
            plural(page.nodes.len(), "node"),
            plural(page.edges.len(), "edge"),
        ];
        if refusals > 0 {
            facts.push(plural(refusals, "refusal"));
        }
        let mut side = String::new();
        for p in &page.panels {
            panel(&mut side, p);
        }
        let format = if page.format.is_empty() {
            FORMAT
        } else {
            page.format.as_str()
        };
        Shell {
            title: page.title.clone(),
            header: Header {
                eyebrow: None,
                heading: page.title.clone(),
                summary: page.summary.clone().filter(|s| !s.is_empty()),
                facts,
            },
            section: "Graph".to_string(),
            graph_heading: "Graph".to_string(),
            graph_hint: "Select a node to follow its edges".to_string(),
            legend: legend(page),
            side,
            after: String::new(),
            footer: format!(
                "{} · rendered by agenticspec, self-contained, no network",
                esc(format)
            ),
            data: serde_json::to_value(page).unwrap_or_default(),
            decor: graph::Decor {
                words: graph::Words {
                    title: format!("Graph {}", page.title),
                    desc: format!(
                        "{} in rows, {} as arrows with their labels.",
                        plural(page.nodes.len(), "node"),
                        plural(page.edges.len(), "edge")
                    ),
                },
                ..graph::Decor::default()
            },
        }
    }
}

pub(crate) fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

/// The page around its graph.
pub(crate) fn render_with(page: &Page, shell: &Shell) -> String {
    let g = graph::render(page, &shell.decor);
    let mut h = String::with_capacity(64 * 1024);
    let _ = write!(
        h,
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta name=\"color-scheme\" content=\"light dark\">\n<title>{}</title>\n<style>\n{CSS}</style>\n</head>\n<body>\n",
        esc(&shell.title)
    );
    header(&mut h, &shell.header);
    h.push_str("<main>\n");
    let _ = write!(
        h,
        "<section class=\"top-grid\" aria-label=\"{}\">\n<div class=\"card graph-card\"><div class=\"card-head\"><h2>{}</h2><span class=\"hint\">{}</span></div><div class=\"card-body\">",
        esc(&shell.section),
        esc(&shell.graph_heading),
        esc(&shell.graph_hint)
    );
    h.push_str(&shell.legend);
    // The graph is drawn at its natural size; the script shows the zoom tools when it is wider
    // than its card, and the card then pans by drag, wheel and arrow keys.
    h.push_str(
        "<div class=\"graph-tools\" hidden role=\"toolbar\" aria-label=\"Zoom\">\
<button type=\"button\" data-zoom=\"out\" aria-label=\"Zoom out\">−</button>\
<output class=\"zoom-level\" aria-live=\"polite\">100 %</output>\
<button type=\"button\" data-zoom=\"in\" aria-label=\"Zoom in\">+</button>\
<button type=\"button\" data-zoom=\"fit\" aria-label=\"Fit to width\">Fit</button>\
<button type=\"button\" data-zoom=\"actual\" aria-label=\"Actual size\">Actual size</button>\
<span class=\"hint\">drag, scroll or use the arrow keys to move around</span></div>",
    );
    let _ = write!(
        h,
        "<div class=\"graph-view\" style=\"--graph-w:{}px\">{}</div>",
        fmt::px(g.width),
        g.svg
    );
    h.push_str("</div></div>\n");
    if !shell.side.is_empty() {
        let _ = writeln!(h, "<aside class=\"side\">{}</aside>", shell.side);
    }
    h.push_str("</section>\n");
    h.push_str(&shell.after);
    h.push_str("</main>\n");
    let _ = writeln!(h, "<footer>{}</footer>", shell.footer);
    let json = serde_json::to_string(&shell.data)
        .unwrap_or_else(|_| "{}".to_string())
        .replace('<', "\\u003c");
    let _ = write!(
        h,
        "<script id=\"agenticspec-data\" type=\"application/json\">{json}</script>\n<script>\n{JS}</script>\n</body>\n</html>\n"
    );
    h
}

fn header(h: &mut String, hd: &Header) {
    h.push_str("<header class=\"top\"><div class=\"title\">");
    if let Some(e) = &hd.eyebrow {
        let _ = write!(h, "<div class=\"eyebrow\">{}</div>", esc(e));
    }
    let _ = write!(h, "<h1>{}</h1>", esc(&hd.heading));
    if let Some(s) = &hd.summary {
        let _ = write!(h, "<p class=\"summary\">{}</p>", esc(s));
    }
    h.push_str("<ul class=\"facts\">");
    for f in &hd.facts {
        let _ = write!(h, "<li>{}</li>", esc(f));
    }
    h.push_str("</ul></div>");
    h.push_str("<div class=\"segmented\" role=\"group\" aria-label=\"Theme\"><button type=\"button\" data-theme-choice=\"system\" aria-pressed=\"true\">System</button><button type=\"button\" data-theme-choice=\"light\" aria-pressed=\"false\">Light</button><button type=\"button\" data-theme-choice=\"dark\" aria-pressed=\"false\">Dark</button></div></header>\n");
}

/// The legend of a page: each group its nodes use, then regions, refusals and other marks.
fn legend(page: &Page) -> String {
    let mut h = String::from("<ul class=\"legend\" aria-label=\"Legend\">");
    let palette = graph::Palette::of(page, &[]);
    let mut groups: Vec<&str> = Vec::new();
    for n in &page.nodes {
        if !groups.contains(&n.group.as_str()) {
            groups.push(&n.group);
        }
    }
    for g in groups {
        let _ = write!(
            h,
            "<li><span class=\"swatch f-{}\"></span>{}</li>",
            palette.slot(g),
            esc(if g.is_empty() { "other" } else { g })
        );
    }
    if page.nodes.iter().any(|n| n.region.is_some()) {
        h.push_str(
            "<li><span class=\"swatch per-sample\"></span>a region, framed around its nodes</li>",
        );
    }
    if page.marks.iter().any(|m| m.level == Level::Refusal) {
        h.push_str("<li><span class=\"swatch refusal-sample\"></span>refused</li>");
    }
    if page.marks.iter().any(|m| m.level == Level::Info) {
        h.push_str("<li><span class=\"swatch chip-sample\"></span>information</li>");
    }
    h.push_str("</ul>");
    h
}

/// A panel as a card with one table; the first cell of a row heads it.
fn panel(h: &mut String, p: &Panel) {
    let _ = write!(
        h,
        "<div class=\"card\"><div class=\"card-head\"><h2>{}</h2><span class=\"hint\">{}</span></div><div class=\"table-wrap\"><table class=\"data mini\"><thead><tr>",
        esc(&p.title),
        plural(p.rows.len(), "row")
    );
    for c in &p.columns {
        let _ = write!(h, "<th scope=\"col\">{}</th>", esc(c));
    }
    h.push_str("</tr></thead><tbody>");
    for row in &p.rows {
        h.push_str("<tr>");
        for (i, cell) in row.iter().enumerate() {
            if i == 0 {
                let _ = write!(h, "<th scope=\"row\">{}</th>", esc(cell));
            } else {
                let _ = write!(h, "<td>{}</td>", esc(cell));
            }
        }
        h.push_str("</tr>");
    }
    h.push_str("</tbody></table></div></div>");
}
