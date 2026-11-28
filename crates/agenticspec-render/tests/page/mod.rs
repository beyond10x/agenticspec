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
    }];
    p.edges[0].to = "pay.charge".to_string();
    p.edges[0].count = Some(2);
    p.marks[0].target.port = Some("charge".to_string());
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
    assert!(html.contains("<g class=\"port refused\" data-port=\"pay.charge\""));
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
fn a_node_in_a_region_is_drawn_inside_the_card_of_its_outermost_region() {
    let html = render(&full());
    assert!(
        html.contains("class=\"node region per f-other\" data-part=\"zone\""),
        "the region that is no node is a card of its own"
    );
    assert!(html.contains("class=\"node child f-"), "db is a child card");
    assert!(
        html.contains(">per tenant<"),
        "headed by the outermost region"
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
