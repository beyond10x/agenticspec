//! The render model `agenticspec-render-model/1`: a page renders, unknown fields are refused by
//! the types and by the committed JSON Schema, the schema covers every field of the types, and
//! the renderer names no agenticspec term.

use agenticspec_render::page::{
    Badge, Colour, Direction, Edge, FORMAT, Group, ICONS, Icon, Layout, Level, Mark, NAMED_COLOURS,
    Node, On, Page, Panel, Place, Port, Region, Target, render,
};
use serde_json::{Value, json};

const SCHEMA: &str = include_str!("../../schema/render-model.schema.json");

fn schema() -> Value {
    serde_json::from_str(SCHEMA).expect("the schema is JSON")
}

fn node(id: &str, group: &str) -> Node {
    Node {
        id: id.to_string(),
        label: format!("{id} label"),
        group: group.to_string(),
        ..Node::default()
    }
}

fn edge(from: &str, to: &str, label: &str) -> Edge {
    Edge {
        from: from.to_string(),
        to: to.to_string(),
        label: label.to_string(),
        count: None,
    }
}

fn mark(node: &str, port: Option<&str>, name: &str, level: Level) -> Mark {
    Mark {
        target: Target {
            node: node.to_string(),
            port: port.map(str::to_string),
        },
        name: name.to_string(),
        detail: Some(format!("{name} detail")),
        level,
    }
}

fn minimal() -> Page {
    let mut p = Page::new("checkout");
    p.nodes = vec![
        node("shop", "service"),
        node("pay", "service"),
        node("db", "store"),
    ];
    p.edges = vec![edge("shop", "pay", "charge"), edge("pay", "db", "write")];
    p.marks = vec![mark("pay", None, "no_retry", Level::Refusal)];
    p.panels = vec![Panel {
        title: "Owners".to_string(),
        columns: vec!["node".to_string(), "owner".to_string()],
        rows: vec![vec!["shop".to_string(), "team-cell-a".to_string()]],
        ..Panel::default()
    }];
    p
}

/// Every field set, every list filled, a region inside a region.
fn full() -> Page {
    let mut p = minimal();
    p.summary = Some("a summary".to_string());
    p.regions = vec![Region {
        id: "zone".to_string(),
        label: "zone".to_string(),
        r#in: Some("zone".to_string()),
        per: Some("tenant".to_string()),
        collapsed: false,
        regions: vec![Region {
            id: "rack".to_string(),
            label: "rack".to_string(),
            r#in: Some("rack".to_string()),
            per: Some("host".to_string()),
            regions: Vec::new(),
            collapsed: false,
        }],
    }];
    p.nodes[0].badges = vec![Badge {
        name: "count".to_string(),
        value: "3".to_string(),
        on: On::Card,
    }];
    p.nodes[1].badges = vec![Badge {
        name: "owner".to_string(),
        value: "team-pay".to_string(),
        on: On::Panel,
    }];
    p.groups = vec![Group {
        name: "service".to_string(),
        colour: Some(Colour::new("#2f7d5b").expect("a hex colour")),
        icon: Some(Icon::new("lock").expect("an icon name")),
    }];
    p.nodes[2].region = Some("rack".to_string());
    p.ports = vec![Port {
        node: "pay".to_string(),
        name: "charge".to_string(),
        kind: "http".to_string(),
        direction: Direction::In,
        contract: Some("charge/1".to_string()),
        via: Some("sip + rtp".to_string()),
        unresolved: true,
    }];
    p.edges[0].to = "pay.charge".to_string();
    p.edges[0].count = Some(2);
    p.marks[0] = mark("pay", Some("charge"), "use_unresolved", Level::Refusal);
    p
}

#[test]
fn a_minimal_page_renders_every_node_edge_mark_and_panel_cell() {
    let page = minimal();
    let html = render(&page);
    assert!(html.starts_with("<!doctype html>") && html.ends_with("</html>\n"));
    for n in &page.nodes {
        assert!(html.contains(&format!(">{}<", n.label)), "node {}", n.id);
        assert!(html.contains(&format!("data-part=\"{}\"", n.id)));
    }
    for e in &page.edges {
        assert!(html.contains(&format!(">{}<", e.label)), "edge {}", e.label);
    }
    assert!(html.contains(">no_retry<"), "the mark is drawn");
    assert!(html.contains(">team-cell-a<"), "the panel cell is drawn");
    assert!(html.contains(&format!("{FORMAT} · rendered by agenticspec")));
    assert!(!html.contains("http://") && !html.contains("https://"));
    assert_eq!(html, render(&page), "deterministic");
}

/// The page and the architecture page share one viewer: its view asks to open at Fit, and its
/// script fits a graph larger than the view (wider than its card or taller than its highest)
/// and leaves one that fits at its natural size.
#[test]
fn the_graph_opens_fitted_when_larger_than_its_view() {
    let arch = serde_json::from_str::<Value>(
        r#"{"architecture":"a","parts":{"x":{"block":"agent"}},"links":[]}"#,
    )
    .expect("JSON");
    for html in [
        render(&minimal()),
        agenticspec_render::architecture::page(&arch, None),
    ] {
        assert_eq!(
            html.matches("<div class=\"graph-view\" data-initial=\"fit\"")
                .count(),
            1,
            "the view asks to open at Fit"
        );
        let script = &html[html.rfind("<script>").expect("the viewer script")..];
        for rule in [
            "var initial = view ? view.getAttribute(\"data-initial\") : null;",
            "return r.w > 0 && (natural.width > r.w || natural.height > r.h);",
            "return Math.min(1, r.w / natural.width, r.h / natural.height);",
            "if (initial === \"fit\" && natural && larger()) setZoom(fitZoom());",
            "else if (initial === \"fit\" && zoom !== 1) setZoom(1);",
            "new ResizeObserver(opening).observe(view);",
        ] {
            assert!(script.contains(rule), "the script holds {rule}");
        }
        assert!(
            html.contains("data-zoom=\"fit\" aria-label=\"Fit the whole graph\""),
            "Fit is the same view the page opens at"
        );
    }
}

/// The opening tag of the group of the card of node `id`.
fn card_of(html: &str, id: &str) -> String {
    let start = html
        .find(&format!("data-part=\"{id}\""))
        .unwrap_or_else(|| panic!("{id} is drawn"));
    let open = html[..start].rfind("<g ").expect("the card's group");
    html[open..start].to_string()
}

#[test]
fn a_refusal_rings_its_card_and_port_and_information_does_not() {
    let mut page = minimal();
    page.marks.push(mark("shop", None, "budget", Level::Info));
    let html = render(&page);
    let card = |id: &str| card_of(&html, id);
    assert!(card("pay").contains("refused"), "{}", card("pay"));
    assert!(!card("shop").contains("refused"), "{}", card("shop"));
    assert!(html.contains("class=\"refusal-ring\""));
    assert!(html.contains("<g class=\"chip refusal\""));
    assert!(html.contains("<g class=\"chip\"><title>budget: budget detail</title>"));
    let html = render(&full());
    assert!(html.contains("<g class=\"port refused unresolved\" data-port=\"pay.charge\""));
    assert!(
        !card_of(&html, "pay").contains("refused"),
        "a refused port, not its card"
    );
    assert!(html.contains(">×3<"), "a count badge");
    assert!(
        html.contains("class=\"edge multi\""),
        "an edge with a count"
    );
    assert!(
        html.contains(">charge ×2 · charge<"),
        "the label names count and port"
    );
}

#[test]
fn a_node_in_a_region_is_drawn_inside_the_frame_of_its_region() {
    let html = render(&full());
    assert!(
        html.contains("<g class=\"region per f-other\" data-region=\"zone\">"),
        "the region that is no node is a frame of its own"
    );
    assert!(
        html.contains("<g class=\"region per f-other\" data-region=\"rack\">"),
        "the region inside it is a frame inside it"
    );
    assert!(html.find("data-region=\"zone\"") < html.find("data-region=\"rack\""));
    assert!(html.contains("data-part=\"db\""), "db is a card of its own");
    assert!(
        html.contains(">per tenant<"),
        "headed by the outermost region"
    );
}

/// The markup of the port row of `address`, from its group's opening tag to its end.
fn port_row(html: &str, address: &str) -> String {
    let start = html
        .find(&format!("data-port=\"{address}\""))
        .unwrap_or_else(|| panic!("port {address} is drawn"));
    let open = html[..start].rfind("<g ").expect("the row's group");
    let end = start + html[start..].find("</g>").expect("the row ends");
    html[open..end].to_string()
}

#[test]
fn an_unresolved_port_leaves_its_row_as_a_dashed_stub_named_by_its_refusal() {
    let html = render(&full());
    let row = port_row(&html, "pay.charge");
    assert!(
        row.starts_with("<g class=\"port refused unresolved\""),
        "{row}"
    );
    assert!(
        row.contains("<path class=\"unresolved-end\" d=\"M"),
        "{row}"
    );
    assert!(
        row.contains("<tspan class=\"rule\">use_unresolved</tspan>"),
        "{row}"
    );
    let mut page = full();
    page.ports[0].unresolved = false;
    let row = port_row(&render(&page), "pay.charge");
    assert!(row.starts_with("<g class=\"port refused\""), "{row}");
    assert!(!row.contains("unresolved-end") && !row.contains("class=\"rule\""));
}

#[test]
fn the_via_of_a_port_is_drawn_on_its_row_after_its_name() {
    let html = render(&full());
    let row = port_row(&html, "pay.charge");
    let name = row.find("charge").expect("the port's name");
    let via = row
        .find("<tspan class=\"via\">sip + rtp</tspan>")
        .unwrap_or_else(|| panic!("{row}"));
    assert!(name < via, "{row}");
    let mut page = full();
    page.ports[0].via = None;
    assert!(!port_row(&render(&page), "pay.charge").contains("class=\"via\""));
}

/// The box `x y w h` of the card of node `id`.
fn box_of(html: &str, id: &str) -> [f64; 4] {
    let start = html
        .find(&format!("data-part=\"{id}\""))
        .unwrap_or_else(|| panic!("{id} is drawn"));
    let at = start + html[start..].find("data-box=\"").expect("a box") + 10;
    let nums: Vec<f64> = html[at..at + html[at..].find('"').expect("box ends")]
        .split(' ')
        .map(|n| n.parse().expect("a number"))
        .collect();
    [nums[0], nums[1], nums[2], nums[3]]
}

#[test]
fn an_edge_to_a_node_inside_a_region_lands_on_the_node_card() {
    let html = render(&full());
    let tag = "<path class=\"edge\" d=\"";
    let at = html
        .match_indices(tag)
        .map(|(i, _)| i + tag.len())
        .find(|&i| {
            html[i..].contains("data-to=\"db\"") && {
                let line = &html[i..i + html[i..].find('>').expect("tag ends")];
                line.contains("data-from=\"pay\" data-to=\"db\"")
            }
        })
        .expect("the edge pay -> db");
    let d = &html[at..at + html[at..].find('"').expect("d ends")];
    let nums: Vec<f64> = d
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("a number"))
        .collect();
    let (ex, ey) = (nums[nums.len() - 2], nums[nums.len() - 1]);
    let [x, y, w, h] = box_of(&html, "db");
    assert!(
        x <= ex && ex <= x + w,
        "end x {ex} within db's card {x}..{}",
        x + w
    );
    assert!(
        (ey - y).abs() <= 8.0 || (ey - (y + h)).abs() <= 8.0,
        "end y {ey} at db's card top {y} or bottom {}, not at its frame",
        y + h
    );
}

/// The edge paths of a page as `(from, to)`, in the order they are drawn.
fn edge_paths(html: &str) -> Vec<(String, String)> {
    let attr = |tag: &str, name: &str| -> String {
        let at = tag.find(&format!("{name}=\"")).expect("the attribute") + name.len() + 2;
        tag[at..at + tag[at..].find('"').expect("closed")].to_string()
    };
    html.match_indices("<path class=\"edge")
        .map(|(i, _)| {
            let tag = &html[i..i + html[i..].find('>').expect("tag ends")];
            (attr(tag, "data-from"), attr(tag, "data-to"))
        })
        .collect()
}

/// A region with three nodes and two edges between them, one between nodes that are not side by
/// side, and a region inside it with an edge across its frame.
fn one_region() -> Page {
    let mut p = Page::new("cell");
    p.regions = vec![Region {
        id: "cell".to_string(),
        label: "cell".to_string(),
        regions: vec![Region {
            id: "core".to_string(),
            label: "core".to_string(),
            ..Region::default()
        }],
        ..Region::default()
    }];
    p.nodes = vec![
        node("a", "service"),
        node("b", "service"),
        node("c", "store"),
        node("d", "store"),
        node("e", "store"),
    ];
    for n in &mut p.nodes[..3] {
        n.region = Some("cell".to_string());
    }
    for n in &mut p.nodes[3..] {
        n.region = Some("core".to_string());
    }
    p.edges = vec![
        edge("a", "c", "reads"),
        edge("c", "b", "feeds"),
        edge("b", "d", "nests"),
        edge("d", "e", "inner"),
    ];
    p
}

#[test]
fn every_edge_inside_one_region_is_drawn() {
    let page = one_region();
    let html = render(&page);
    let drawn = edge_paths(&html);
    for e in &page.edges {
        assert!(
            drawn.contains(&(e.from.clone(), e.to.clone())),
            "edge {} -> {} is drawn: {drawn:?}",
            e.from,
            e.to
        );
        assert!(
            html.contains(&format!(">{}<", e.label)),
            "label {}",
            e.label
        );
    }
    assert_eq!(drawn.len(), page.edges.len(), "{drawn:?}");
    // No label overlaps a card or another label, and no edge passes through a card it does not
    // end at.
    let boxes: Vec<[f64; 4]> = html
        .split("data-box=\"")
        .skip(1)
        .map(|piece| {
            let v: Vec<f64> = piece[..piece.find('"').expect("closed")]
                .split(' ')
                .map(|n| n.parse().expect("a number"))
                .collect();
            [v[0], v[1], v[2], v[3]]
        })
        .collect();
    for (i, a) in boxes.iter().enumerate() {
        for b in &boxes[i + 1..] {
            let apart = a[0] + a[2] <= b[0]
                || b[0] + b[2] <= a[0]
                || a[1] + a[3] <= b[1]
                || b[1] + b[3] <= a[1];
            assert!(apart, "{a:?} and {b:?} overlap");
        }
    }
    for (from, to) in &drawn {
        let at = html
            .find(&format!("data-from=\"{from}\" data-to=\"{to}\""))
            .expect("the edge");
        let open = html[..at].rfind("<path").expect("the path");
        let d0 = open + html[open..].find("d=\"").expect("d") + 3;
        let d = &html[d0..d0 + html[d0..].find('"').expect("d ends")];
        assert!(!d.contains('C'), "a straight path: {d}");
        let pts: Vec<(f64, f64)> = d
            .split(['M', 'L'])
            .filter(|s| !s.is_empty())
            .map(|s| {
                let (x, y) = s.split_once(' ').expect("x y");
                (x.parse().expect("x"), y.parse().expect("y"))
            })
            .collect();
        for id in ["a", "b", "c", "d", "e"] {
            if id == from || id == to {
                continue;
            }
            let [x, y, w, h] = box_of(&html, id);
            for s in pts.windows(2) {
                for k in 0..=20 {
                    let t = k as f64 / 20.0;
                    let (px, py) = (
                        s[0].0 + (s[1].0 - s[0].0) * t,
                        s[0].1 + (s[1].1 - s[0].1) * t,
                    );
                    assert!(
                        !(px > x + 1.0 && px < x + w - 1.0 && py > y + 1.0 && py < y + h - 1.0),
                        "{from} -> {to} passes through {id}: {d}"
                    );
                }
            }
        }
    }
}

#[test]
fn a_listed_group_sets_the_colour_and_icon_of_its_cards_and_others_keep_the_palette() {
    let mut page = minimal();
    page.nodes.push(node("queue", "queue"));
    let before = render(&page);
    page.groups = vec![
        Group {
            name: "service".to_string(),
            colour: Some(Colour::new("hsl(150deg, 50%, 33%)").expect("hsl")),
            icon: Some(Icon::new("lock").expect("an icon name")),
        },
        Group {
            name: "store".to_string(),
            colour: None,
            icon: Some(Icon::new("DB").expect("a text icon")),
        },
    ];
    let html = render(&page);
    let style = "style=\"--c:hsl(150deg, 50%, 33%)\"";
    for id in ["shop", "pay"] {
        assert!(card_of(&html, id).contains(style), "{}", card_of(&html, id));
    }
    assert!(
        html.contains(&format!("{style}></span>service</li>")),
        "the legend swatch takes the colour"
    );
    let lock = "M3.5 6.5h7v5.5h-7zM5 6.5V4.8a2 2 0 0 1 4 0v1.7";
    assert!(
        html.contains(lock) && !before.contains(lock),
        "the lock icon"
    );
    assert!(html.contains("class=\"icon-text\""), "a text icon");
    assert!(html.contains(">DB</text>"), "the text icon reads DB");
    assert!(!card_of(&html, "db").contains("style="), "no colour named");
    for id in ["db", "queue"] {
        assert_eq!(
            card_of(&html, id),
            card_of(&before, id),
            "{id} keeps its palette slot"
        );
    }
}

#[test]
fn a_colour_that_is_not_a_plain_colour_is_refused() {
    for ok in [
        "#3a7",
        "#3a7c",
        "#33AA77",
        "#33aa77cc",
        "rgb(51 170 119)",
        "rgb(51, 170, 119)",
        "rgba(51,170,119,.5)",
        "rgb(51 170 119 / 50%)",
        "hsl(150deg, 50%, 43%)",
        "hsla(-30 50% 43% / 0.5)",
        "teal",
    ] {
        assert!(Colour::new(ok).is_ok(), "{ok} is a colour");
    }
    for bad in [
        "",
        "red;}body{display:none",
        "red\" onload=\"x",
        "url(x)",
        "var(--x)",
        "expression(1)",
        "Teal",
        "rebeccapurple",
        "#12345",
        "#ggg",
        "rgb(1,2)",
        "rgb(1,2,3,4,5)",
        "rgb(1,2,3,)",
        "rgb(1,,2,3)",
        "rgb(1-2-3)",
        "rgb(1 2 3);x",
        "rgb(calc(1) 2 3)",
        "hsl(1e3 2 3)",
    ] {
        assert!(Colour::new(bad).is_err(), "{bad:?} is refused");
    }
    let mut v = serde_json::to_value(full()).expect("serializes");
    v["groups"][0]["colour"] = json!("red;}*{display:none");
    let err = serde_json::from_value::<Page>(v)
        .expect_err("the types refuse the colour")
        .to_string();
    assert!(err.contains("colour"), "{err}");
    for icon in ["", "abc", "\u{7}", "  ", "rocket"] {
        assert!(Icon::new(icon).is_err(), "icon {icon:?} is refused");
        let mut v = serde_json::to_value(full()).expect("serializes");
        v["groups"][0]["icon"] = json!(icon);
        assert!(serde_json::from_value::<Page>(v.clone()).is_err());
        if icon.chars().count() > 2 {
            assert!(!validate(&schema(), &v, false).is_empty(), "schema: {icon}");
        }
    }
    // The schema lists the same named colours and icon names as the types.
    let s = schema();
    let list = |def: &str, branch: usize| -> Vec<String> {
        s["$defs"][def]["anyOf"][branch]["enum"]
            .as_array()
            .expect("an enum")
            .iter()
            .map(|x| x.as_str().expect("a name").to_string())
            .collect()
    };
    assert_eq!(list("Colour", 2), NAMED_COLOURS.to_vec());
    assert_eq!(list("Icon", 0), ICONS.to_vec());
}

/// The style attribute of the edge path `from -> to`, or "".
fn edge_style(html: &str, from: &str, to: &str) -> String {
    let at = html
        .find(&format!("data-from=\"{from}\" data-to=\"{to}\""))
        .unwrap_or_else(|| panic!("edge {from} -> {to}"));
    let tag = &html[at..at + html[at..].find('>').expect("tag ends")];
    tag.find("style=\"")
        .map(|i| tag[i + 7..i + 7 + tag[i + 7..].find('"').expect("closed")].to_string())
        .unwrap_or_default()
}

#[test]
fn an_edge_is_drawn_heavier_as_its_count_grows() {
    let mut page = Page::new("weights");
    page.nodes = vec![
        node("a", "x"),
        node("b", "x"),
        node("c", "x"),
        node("d", "x"),
    ];
    page.edges = vec![
        edge("a", "b", "one"),
        edge("a", "c", "three"),
        edge("a", "d", "nine"),
    ];
    let plain = render(&page);
    assert!(!plain.contains("id=\"arrow-2\"") && !plain.contains("id=\"arrow-3\""));
    page.edges[0].count = Some(1);
    page.edges[1].count = Some(3);
    page.edges[2].count = Some(9);
    let html = render(&page);
    assert_eq!(
        edge_style(&html, "a", "b"),
        "",
        "one edge keeps the plain stroke"
    );
    let three = edge_style(&html, "a", "c");
    let nine = edge_style(&html, "a", "d");
    assert!(
        three.contains("stroke-width:2.4") && three.contains("url(#arrow-2)"),
        "{three}"
    );
    assert!(
        nine.contains("stroke-width:3.6") && nine.contains("url(#arrow-3)"),
        "{nine}"
    );
    assert!(html.contains("<marker id=\"arrow-2\"") && html.contains("<marker id=\"arrow-3\""));
    assert!(html.contains("class=\"edge multi\""));
}

#[test]
fn a_panel_badge_is_a_row_of_the_details_table_and_not_a_pill() {
    let plain = render(&minimal());
    assert!(
        !plain.contains("aria-label=\"Details\""),
        "no panel badge, no table"
    );
    let html = render(&full());
    let svg = &html[html.find("<svg").expect("svg")..html.find("</svg>").expect("svg ends")];
    assert!(svg.contains(">×3<"), "a card badge is a pill");
    assert!(!svg.contains("team-pay"), "a panel badge is no pill");
    let at = html
        .find("aria-label=\"Details\"")
        .expect("the details table");
    assert!(at > html.find("</svg>").unwrap(), "under the graph");
    let table = &html[at..at + html[at..].find("</section>").expect("ends")];
    assert!(
        table.contains("<tr><th scope=\"row\">pay label</th><td>owner</td><td>team-pay</td></tr>"),
        "{table}"
    );
}

#[test]
fn an_unknown_field_is_refused_by_the_types() {
    let mut ok = serde_json::to_value(minimal()).expect("serializes");
    let back: Page = serde_json::from_value(ok.clone()).expect("round trip");
    assert_eq!(back, minimal());
    ok["colour"] = json!("red");
    assert!(serde_json::from_value::<Page>(ok).is_err(), "top level");
    let mut inner = serde_json::to_value(minimal()).expect("serializes");
    inner["nodes"][0]["shape"] = json!("round");
    assert!(serde_json::from_value::<Page>(inner).is_err(), "in a node");
}

#[test]
fn the_schema_accepts_a_page_and_refuses_an_unknown_field() {
    let s = schema();
    for page in [minimal(), full()] {
        let v = serde_json::to_value(&page).expect("serializes");
        let errs = validate(&s, &v, false);
        assert!(errs.is_empty(), "{errs:?}");
    }
    let mut top = serde_json::to_value(full()).expect("serializes");
    top["colour"] = json!("red");
    assert!(!validate(&s, &top, false).is_empty());
    let mut deep = serde_json::to_value(full()).expect("serializes");
    deep["regions"][0]["regions"][0]["shape"] = json!("round");
    assert!(!validate(&s, &deep, false).is_empty());
    let mut collapsed = serde_json::to_value(full()).expect("serializes");
    collapsed["regions"][0]["collapsed"] = json!(true);
    assert!(
        validate(&s, &collapsed, false).is_empty(),
        "collapsed: true"
    );
    collapsed["regions"][0]["collapsed"] = json!("yes");
    assert!(
        !validate(&s, &collapsed, false).is_empty(),
        "collapsed is a boolean"
    );
    let mut level = serde_json::to_value(full()).expect("serializes");
    level["marks"][0]["level"] = json!("warning");
    assert!(!validate(&s, &level, false).is_empty());
    let mut layout = serde_json::to_value(full()).expect("serializes");
    layout["layout"] = json!("bands");
    assert!(validate(&s, &layout, false).is_empty(), "bands is a layout");
    layout["layout"] = json!("grid");
    assert!(
        !validate(&s, &layout, false).is_empty(),
        "grid is no layout"
    );
}

/// The schema names exactly the fields of the types: a page with every field set has every
/// property the schema declares, and no other.
#[test]
fn the_schema_and_the_types_name_the_same_fields() {
    let mut page = full();
    page.edges[1].count = Some(1);
    page.layout = Layout::Bands;
    page.panels[0].place = Place::Below;
    page.nodes[0].badges[0].on = On::Panel;
    page.regions[0].collapsed = true;
    page.regions[0].regions[0].collapsed = true;
    for n in &mut page.nodes {
        n.region = Some("zone".to_string());
    }
    let v = serde_json::to_value(page).expect("serializes");
    let errs = validate(&schema(), &v, true);
    assert!(errs.is_empty(), "{errs:?}");
}

/// The example page of the rustdoc of `page` parses, validates and renders.
#[test]
fn the_documented_example_page_is_valid() {
    let src = include_str!("../../src/page.rs");
    let start = src.find("//! ```json\n").expect("an example") + "//! ```json\n".len();
    let end = start + src[start..].find("//! ```\n").expect("its end");
    let text: String = src[start..end]
        .lines()
        .map(|l| l.trim_start_matches("//!").trim_start())
        .collect::<Vec<_>>()
        .join("\n");
    let v: Value = serde_json::from_str(&text).expect("the example is JSON");
    assert!(validate(&schema(), &v, false).is_empty());
    let page: Page = serde_json::from_value(v).expect("the example is a page");
    let html = render(&page);
    assert!(html.contains(">rate_limit<") || html.contains(">complete · rate_limit<"));
    assert!(html.contains("data-port=\"model.complete\""));
}

#[test]
fn the_renderer_names_no_agenticspec_term() {
    let words = [
        "PartKind",
        "agentic.",
        "KPI",
        "kpi",
        "acts_for",
        "carries",
        "rchitecture",
        "Outcome",
        "fitness",
        "marked place",
    ];
    for (file, src) in [
        ("page.rs", include_str!("../../src/page.rs")),
        ("graph.rs", include_str!("../../src/graph.rs")),
        ("fmt.rs", include_str!("../../src/fmt.rs")),
        ("collapse.rs", include_str!("../../src/collapse.rs")),
    ] {
        for w in words {
            assert!(!src.contains(w), "src/{file} names {w:?}");
        }
    }
}

/// The errors of `v` against the subset of JSON Schema the committed schema uses: `$ref` into
/// `$defs`, `type`, `const`, `enum`, `properties`, `required`, `additionalProperties: false`,
/// `items`, `minimum`, `anyOf`, `minLength`, `maxLength`. `pattern` is not checked here: the
/// types check the same grammar, see `a_colour_that_is_not_a_plain_colour_is_refused`. With
/// `all`, every declared property must be present too.
fn validate(root: &Value, v: &Value, all: bool) -> Vec<String> {
    validate_at(root, root, v, all)
}

fn validate_at(root: &Value, s: &Value, v: &Value, all: bool) -> Vec<String> {
    let mut errs = Vec::new();
    check(root, s, v, "$", all, &mut errs);
    errs
}

fn check(root: &Value, s: &Value, v: &Value, at: &str, all: bool, errs: &mut Vec<String>) {
    if let Some(r) = s.get("$ref").and_then(Value::as_str) {
        let name = r.strip_prefix("#/$defs/").expect("a local ref");
        check(root, &root["$defs"][name], v, at, all, errs);
    }
    if let Some(c) = s.get("const")
        && c != v
    {
        errs.push(format!("{at}: not {c}"));
    }
    if let Some(e) = s.get("enum").and_then(Value::as_array)
        && !e.contains(v)
    {
        errs.push(format!("{at}: {v} not in {e:?}"));
    }
    if let Some(branches) = s.get("anyOf").and_then(Value::as_array)
        && branches
            .iter()
            .all(|b| !validate_at(root, b, v, all).is_empty())
    {
        errs.push(format!("{at}: {v} matches no branch of anyOf"));
    }
    if let (Some(n), Some(t)) = (s.get("minLength").and_then(Value::as_u64), v.as_str())
        && (t.chars().count() as u64) < n
    {
        errs.push(format!("{at}: shorter than {n}"));
    }
    if let (Some(n), Some(t)) = (s.get("maxLength").and_then(Value::as_u64), v.as_str())
        && (t.chars().count() as u64) > n
    {
        errs.push(format!("{at}: longer than {n}"));
    }
    if let Some(t) = s.get("type").and_then(Value::as_str) {
        let ok = match t {
            "string" => v.is_string(),
            "integer" => v.is_u64() || v.is_i64(),
            "object" => v.is_object(),
            "array" => v.is_array(),
            "boolean" => v.is_boolean(),
            other => panic!("type {other} is not in the subset"),
        };
        if !ok {
            errs.push(format!("{at}: not {t}"));
            return;
        }
    }
    if let (Some(min), Some(n)) = (s.get("minimum").and_then(Value::as_f64), v.as_f64())
        && n < min
    {
        errs.push(format!("{at}: below {min}"));
    }
    if let Some(o) = v.as_object() {
        let props = s.get("properties").and_then(Value::as_object);
        for req in s
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let k = req.as_str().expect("a name");
            if !o.contains_key(k) {
                errs.push(format!("{at}: {k} is required"));
            }
        }
        if let Some(props) = props {
            for (k, x) in o {
                match props.get(k) {
                    Some(ps) => check(root, ps, x, &format!("{at}.{k}"), all, errs),
                    None if s.get("additionalProperties") == Some(&json!(false)) => {
                        errs.push(format!("{at}: {k} is not declared"));
                    }
                    None => {}
                }
            }
            if all {
                for k in props.keys() {
                    if !o.contains_key(k) {
                        errs.push(format!("{at}: {k} is declared but the type wrote none"));
                    }
                }
            }
        }
    }
    if let (Some(items), Some(xs)) = (s.get("items"), v.as_array()) {
        for (i, x) in xs.iter().enumerate() {
            check(root, items, x, &format!("{at}[{i}]"), all, errs);
        }
    }
}
