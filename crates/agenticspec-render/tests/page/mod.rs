//! The render model `agenticspec-render-model/1`: a page renders, unknown fields are refused by
//! the types and by the committed JSON Schema, the schema covers every field of the types, and
//! the renderer names no agenticspec term.

use agenticspec_render::page::{
    Badge, Direction, Edge, FORMAT, Level, Mark, Node, Page, Panel, Port, Region, Target, render,
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
        regions: vec![Region {
            id: "rack".to_string(),
            label: "rack".to_string(),
            r#in: Some("rack".to_string()),
            per: Some("host".to_string()),
            regions: Vec::new(),
        }],
    }];
    p.nodes[0].badges = vec![Badge {
        name: "count".to_string(),
        value: "3".to_string(),
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
    let mut level = serde_json::to_value(full()).expect("serializes");
    level["marks"][0]["level"] = json!("warning");
    assert!(!validate(&s, &level, false).is_empty());
}

/// The schema names exactly the fields of the types: a page with every field set has every
/// property the schema declares, and no other.
#[test]
fn the_schema_and_the_types_name_the_same_fields() {
    let mut page = full();
    page.edges[1].count = Some(1);
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
    ] {
        for w in words {
            assert!(!src.contains(w), "src/{file} names {w:?}");
        }
    }
}

/// The errors of `v` against the subset of JSON Schema the committed schema uses: `$ref` into
/// `$defs`, `type`, `const`, `enum`, `properties`, `required`, `additionalProperties: false`,
/// `items`, `minimum`. With `all`, every declared property must be present too.
fn validate(root: &Value, v: &Value, all: bool) -> Vec<String> {
    let mut errs = Vec::new();
    check(root, root, v, "$", all, &mut errs);
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
