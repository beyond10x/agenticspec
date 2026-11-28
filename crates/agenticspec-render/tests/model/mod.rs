//! Edges inside one region of many nodes, edge ends that resolve to nothing, and panels below the
//! graph and with long values.

use agenticspec_render::page::{Edge, Node, Page, Panel, Place, Port, Region, render};
use serde_json::json;

fn node(id: &str, region: Option<&str>) -> Node {
    Node {
        id: id.to_string(),
        label: id.to_string(),
        group: ["agent", "work", "place"][id.len() % 3].to_string(),
        region: region.map(str::to_string),
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

/// One region of 68 nodes and 83 edges between them, some to a port; every edge is drawn.
#[test]
fn every_edge_between_cards_of_one_crowded_region_is_drawn() {
    for region_is_node in [false, true] {
        let mut p = Page::new("scope");
        p.regions = vec![Region {
            id: "scope".to_string(),
            label: "scope".to_string(),
            per: Some("tenant".to_string()),
            ..Region::default()
        }];
        if region_is_node {
            p.nodes.push(node("scope", None));
        }
        for i in 0..68 {
            p.nodes.push(node(&format!("n{i}"), Some("scope")));
        }
        p.ports.push(Port {
            node: "n3".to_string(),
            name: "in".to_string(),
            kind: "http".to_string(),
            ..Port::default()
        });
        let mut k: u64 = 7;
        for i in 0..83usize {
            k = k
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let a = i % 68;
            let mut b = ((k >> 33) % 68) as usize;
            if b == a {
                b = (a + 1) % 68;
            }
            let to = if b == 3 {
                "n3.in".to_string()
            } else {
                format!("n{b}")
            };
            p.edges.push(edge(&format!("n{a}"), &to, &format!("e{i}")));
        }
        assert!(p.check().is_ok());
        let html = render(&p);
        assert_eq!(
            html.matches("<path class=\"edge").count(),
            p.edges.len(),
            "every edge is drawn (region is a node: {region_is_node})"
        );
    }
}

#[test]
fn an_edge_end_that_names_no_node_or_port_is_refused() {
    let base = json!({
        "format": "agenticspec-render-model/1",
        "title": "t",
        "regions": [{ "id": "zone", "label": "zone" }],
        "nodes": [
            { "id": "a", "label": "a", "group": "g", "region": "zone" },
            { "id": "b", "label": "b", "group": "g" },
            { "id": "c.d", "label": "c.d", "group": "g" }
        ],
        "ports": [{ "node": "b", "name": "in", "kind": "http", "direction": "in" }],
        "edges": [
            { "from": "a", "to": "b.in", "label": "ok" },
            { "from": "c.d", "to": "a", "label": "dotted id" }
        ]
    });
    serde_json::from_value::<Page>(base.clone()).expect("every end resolves");
    for (from, to, bad) in [
        ("a", "ghost", "ghost"),
        ("b.out", "a", "b.out"),
        ("a.in", "b", "a.in"),
        ("a", "zone", "zone"),
    ] {
        let mut v = base.clone();
        v["edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "from": from, "to": to, "label": "x" }));
        let err = serde_json::from_value::<Page>(v.clone())
            .expect_err("an end that resolves to nothing")
            .to_string();
        assert!(err.contains("edge_end_unknown"), "{err}");
        assert!(
            err.contains(&format!("from {from:?} to {to:?}")),
            "names the edge: {err}"
        );
        assert!(
            err.contains(&format!("{bad:?} is no node")),
            "names the end: {err}"
        );
        let page = Page {
            edges: vec![edge(from, to, "x")],
            ..serde_json::from_value(base.clone()).unwrap()
        };
        assert!(page.check().is_err(), "Page::check refuses {bad}");
    }
}

fn with_panel(place: Place, value: &str) -> Page {
    let mut p = Page::new("t");
    p.nodes = vec![node("a", None)];
    p.panels = vec![Panel {
        title: "Inventory".to_string(),
        columns: vec!["name".to_string(), "value".to_string()],
        rows: vec![vec!["a".to_string(), value.to_string()]],
        place,
    }];
    p
}

#[test]
fn a_below_panel_spans_the_page_under_the_graph() {
    let html = render(&with_panel(Place::Below, "1"));
    let graph = html.find("<section class=\"top-grid\"").expect("the graph");
    let end = graph + html[graph..].find("</section>").expect("closed");
    let panel = html.find(">Inventory<").expect("the panel");
    assert!(panel > end, "the panel is under the graph's section");
    assert!(!html.contains("<aside class=\"side\">"), "no side column");
    let section = html[..panel].rfind("<section ").expect("its section");
    assert!(
        html[section..panel]
            .starts_with("<section class=\"section-gap\" aria-label=\"Inventory\">"),
        "a section of its own across the page: {}",
        &html[section..panel]
    );
    assert!(html[..graph].rfind("<main>").is_some());
    let side = render(&with_panel(Place::Side, "1"));
    let aside = side
        .find("<aside class=\"side\">")
        .expect("the side column");
    assert!(
        side.find(">Inventory<").unwrap() > aside,
        "side is the default place"
    );
}

#[test]
fn a_long_cell_value_carries_its_full_value_as_a_title() {
    let value = "arn:cloud:queue:eu-central-1:000000000000:orders-dead-letter-queue";
    let html = render(&with_panel(Place::Side, value));
    let cell = format!(
        "<td title=\"{value}\" style=\"white-space:normal;overflow-wrap:anywhere;min-width:12ch\">{value}</td>"
    );
    assert!(
        html.contains(&cell),
        "the long value carries its title and wraps"
    );
    let short = render(&with_panel(Place::Side, "eu"));
    assert!(
        short.contains("<td>eu</td>"),
        "a short value is a plain cell"
    );
    let quoted = render(&with_panel(Place::Side, &format!("\"{value}\"")));
    assert!(
        quoted.contains(&format!("title=\"&quot;{value}&quot;\"")),
        "the title is escaped"
    );
}
