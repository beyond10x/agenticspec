//! `layout: bands`: one horizontal band per top-level region, in the order of the list, each as
//! wide as the page; nested regions and nodes laid out inside their band; edges between bands
//! drawn through the gaps between them; the default layout unchanged.

use agenticspec_render::page::{Edge, Layout, Node, Page, Region, render};
use serde_json::json;

fn node(id: &str, region: Option<&str>) -> Node {
    Node {
        id: id.to_string(),
        label: format!("{id} label"),
        group: "service".to_string(),
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

fn region(id: &str, regions: Vec<Region>) -> Region {
    Region {
        id: id.to_string(),
        label: format!("{id} band"),
        regions,
        ..Region::default()
    }
}

fn numbers(s: &str) -> Vec<f64> {
    s.split(' ').map(|n| n.parse().expect("a number")).collect()
}

/// The quoted value of attribute `name` in the text from `at` on.
fn attr_after<'a>(html: &'a str, at: usize, name: &str) -> &'a str {
    let key = format!("{name}=\"");
    let start = at + html[at..].find(&key).unwrap_or_else(|| panic!("{name}")) + key.len();
    &html[start..start + html[start..].find('"').expect("closed")]
}

/// Every band: its id and its box, top to bottom as drawn.
fn bands(html: &str) -> Vec<(String, [f64; 4])> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(i) = html[at..].find("data-band=\"") {
        let start = at + i;
        let id = attr_after(html, start, "data-band").to_string();
        let b = numbers(attr_after(html, start, "data-band-box"));
        out.push((id, [b[0], b[1], b[2], b[3]]));
        at = start + 1;
    }
    out
}

fn band(html: &str, id: &str) -> [f64; 4] {
    bands(html)
        .into_iter()
        .find(|b| b.0 == id)
        .unwrap_or_else(|| panic!("band {id}"))
        .1
}

/// The box of the card of node `id`.
fn card(html: &str, id: &str) -> [f64; 4] {
    let at = html
        .match_indices(&format!("data-part=\"{id}\""))
        .map(|(p, _)| p)
        .find(|&p| {
            html[..p]
                .rfind("<g ")
                .is_some_and(|g| html[g..p].contains("class=\"node"))
        })
        .unwrap_or_else(|| panic!("card {id}"));
    let b = numbers(attr_after(html, at, "data-box"));
    [b[0], b[1], b[2], b[3]]
}

/// The box of the frame of nested region `id`.
fn frame(html: &str, id: &str) -> [f64; 4] {
    let at = html
        .find(&format!("data-region=\"{id}\""))
        .unwrap_or_else(|| panic!("frame {id}"));
    let n = |name| attr_after(html, at, name).parse::<f64>().expect("a number");
    [n("x"), n("y"), n("width"), n("height")]
}

fn inside(inner: [f64; 4], outer: [f64; 4]) -> bool {
    inner[0] >= outer[0]
        && inner[1] >= outer[1]
        && inner[0] + inner[2] <= outer[0] + outer[2]
        && inner[1] + inner[3] <= outer[1] + outer[3]
}

/// The `d` of the path of the edge `from` → `to`.
fn path<'a>(html: &'a str, from: &str, to: &str) -> &'a str {
    let at = html
        .find(&format!("data-from=\"{from}\" data-to=\"{to}\""))
        .unwrap_or_else(|| panic!("edge {from} → {to}"));
    let d = html[..at].rfind(" d=\"").expect("a path") + 4;
    &html[d..d + html[d..].find('"').expect("closed")]
}

/// Points along a path of `M` and `L` commands.
fn samples(d: &str) -> Vec<(f64, f64)> {
    let mut corners: Vec<(f64, f64)> = Vec::new();
    for piece in d.split(['M', 'L']).filter(|p| !p.is_empty()) {
        let v = numbers(piece.trim());
        assert_eq!(
            v.len(),
            2,
            "a crossing edge is drawn of straight lines: {d}"
        );
        corners.push((v[0], v[1]));
    }
    let mut out = vec![corners[0]];
    for w in corners.windows(2) {
        for k in 1..=20 {
            let t = k as f64 / 20.0;
            out.push((
                w[0].0 + (w[1].0 - w[0].0) * t,
                w[0].1 + (w[1].1 - w[0].1) * t,
            ));
        }
    }
    out
}

fn strictly_in(p: (f64, f64), b: [f64; 4]) -> bool {
    p.0 > b[0] && p.0 < b[0] + b[2] && p.1 > b[1] && p.1 < b[1] + b[3]
}

fn bands_page(order: [&str; 2]) -> Page {
    let mut p = Page::new("shop");
    p.layout = Layout::Bands;
    p.regions = order.iter().map(|id| region(id, Vec::new())).collect();
    p.nodes = vec![
        node("web", Some("front")),
        node("api", Some("front")),
        node("db", Some("store")),
    ];
    p.edges = vec![edge("web", "api", "call"), edge("api", "db", "write")];
    p
}

#[test]
fn bands_follow_the_order_of_the_list() {
    // The graph layout puts `db`, where every edge ends, below `web` and `api`; the list puts
    // its band first.
    for (order, top) in [(["store", "front"], "store"), (["front", "store"], "front")] {
        let html = render(&bands_page(order));
        let found = bands(&html);
        let ids: Vec<&str> = found.iter().map(|b| b.0.as_str()).collect();
        assert_eq!(
            ids,
            order.to_vec(),
            "one band per top-level region, in list order"
        );
        assert_eq!(found[0].0, top);
        let (a, b) = (found[0].1, found[1].1);
        assert!(
            a[1] + a[3] < b[1],
            "the first band is above the second: {a:?} {b:?}"
        );
        assert_eq!(
            (a[0], a[2]),
            (b[0], b[2]),
            "every band is as wide as the page"
        );
        for (id, inside_of) in [("web", "front"), ("api", "front"), ("db", "store")] {
            assert!(
                inside(card(&html, id), band(&html, inside_of)),
                "{id} is inside band {inside_of}"
            );
        }
        assert!(html.contains(">front band<") && html.contains(">store band<"));
    }
}

#[test]
fn nested_regions_stay_inside_their_band() {
    let mut p = Page::new("racks");
    p.layout = Layout::Bands;
    p.regions = vec![
        region(
            "cell",
            vec![region("rack", vec![region("slot", Vec::new())])],
        ),
        region("edge", Vec::new()),
    ];
    p.nodes = vec![
        node("lb", Some("edge")),
        node("host", Some("cell")),
        node("disk", Some("rack")),
        node("chip", Some("slot")),
    ];
    p.edges = vec![
        edge("host", "disk", "mount"),
        edge("disk", "chip", "read"),
        edge("lb", "host", "route"),
    ];
    let html = render(&p);
    let cell = band(&html, "cell");
    for f in ["rack", "slot"] {
        assert!(
            inside(frame(&html, f), cell),
            "frame {f} is inside band cell"
        );
    }
    assert!(inside(frame(&html, "slot"), frame(&html, "rack")));
    for n in ["host", "disk", "chip"] {
        assert!(inside(card(&html, n), cell), "{n} is inside band cell");
    }
    assert!(inside(card(&html, "lb"), band(&html, "edge")));
    assert!(!html.contains("data-region=\"cell\""), "a band is no frame");
}

#[test]
fn an_edge_between_bands_stays_out_of_every_other_band() {
    let mut p = Page::new("tiers");
    p.layout = Layout::Bands;
    p.regions = vec![
        region("a", Vec::new()),
        region("b", Vec::new()),
        region("c", Vec::new()),
    ];
    p.nodes = vec![
        node("a1", Some("a")),
        node("b1", Some("b")),
        node("b2", Some("b")),
        node("b3", Some("b")),
        node("c1", Some("c")),
    ];
    p.edges = vec![
        edge("a1", "c1", "down"),
        edge("c1", "a1", "up"),
        edge("a1", "b1", "near"),
        edge("b1", "b2", "inside"),
        edge("b2", "b3", "inside"),
        edge("a1", "b3", "deep"),
        edge("b3", "a1", "back"),
        edge("c1", "b1", "climb"),
    ];
    let html = render(&p);
    let (a, b, c) = (band(&html, "a"), band(&html, "b"), band(&html, "c"));
    for (from, to) in [("a1", "c1"), ("c1", "a1")] {
        let pts = samples(path(&html, from, to));
        for &pt in &pts {
            assert!(
                !strictly_in(pt, b),
                "{from} → {to} passes through band b at {pt:?}"
            );
            for n in ["b1", "b2", "b3"] {
                assert!(
                    !strictly_in(pt, card(&html, n)),
                    "{from} → {to} crosses {n}"
                );
            }
        }
        // It leaves its own card and reaches the other one.
        let (s, e) = (pts[0], pts[pts.len() - 1]);
        let (fc, tc) = (card(&html, from), card(&html, to));
        assert!(
            s.0 >= fc[0] && s.0 <= fc[0] + fc[2],
            "{from} → {to} starts on {from}"
        );
        assert!(
            e.0 >= tc[0] && e.0 <= tc[0] + tc[2],
            "{from} → {to} ends on {to}"
        );
        assert!((s.1 - fc[1]).abs() < 2.0 || (s.1 - fc[1] - fc[3]).abs() < 2.0);
        assert!((e.1 - tc[1]).abs() < 2.0 || (e.1 - tc[1] - tc[3]).abs() < 2.0);
    }
    // Inside the band it enters or leaves, it passes no card but its own.
    for (from, to) in [("a1", "b3"), ("b3", "a1"), ("c1", "b1")] {
        for pt in samples(path(&html, from, to)) {
            for n in ["a1", "b1", "b2", "b3", "c1"] {
                if n != from && n != to {
                    assert!(
                        !strictly_in(pt, card(&html, n)),
                        "{from} → {to} crosses {n} at {pt:?}"
                    );
                }
            }
        }
    }
    // An edge to the next band runs through the gap between the two.
    for pt in samples(path(&html, "a1", "b1")) {
        assert!(!strictly_in(pt, c), "a1 → b1 stays out of band c");
    }
    assert!(a[1] + a[3] < b[1] && b[1] + b[3] < c[1]);
    for label in ["down", "up", "near"] {
        assert!(
            html.contains(&format!(">{label}<")),
            "label {label} is drawn"
        );
    }
}

#[test]
fn a_node_in_no_top_level_region_goes_in_a_last_band_without_a_label() {
    let mut p = bands_page(["front", "store"]);
    p.nodes.push(node("loose", None));
    p.nodes.push(node("lost", Some("nowhere")));
    p.edges.push(edge("loose", "web", "poke"));
    let html = render(&p);
    let found = bands(&html);
    let ids: Vec<&str> = found.iter().map(|b| b.0.as_str()).collect();
    assert_eq!(ids, ["front", "store", ""]);
    let last = found[2].1;
    assert!(inside(card(&html, "loose"), last) && inside(card(&html, "lost"), last));
    let at = html.find("data-band=\"\"").expect("the last band");
    let group = &html[at..at + html[at..].find("</g>").expect("closed")];
    assert!(
        !group.contains("<text"),
        "the last band has no label: {group}"
    );
    // Without such a node there is no last band.
    assert_eq!(bands(&render(&bands_page(["front", "store"]))).len(), 2);
}

#[test]
fn bands_render_the_same_bytes_twice() {
    let mut p = bands_page(["store", "front"]);
    p.nodes.push(node("loose", None));
    p.edges.push(edge("db", "loose", "dump"));
    assert_eq!(render(&p), render(&p));
}

#[test]
fn without_the_field_the_page_is_drawn_as_before() {
    let mut p = bands_page(["front", "store"]);
    p.layout = Layout::Graph;
    let v = serde_json::to_value(&p).expect("serializes");
    assert!(v.get("layout").is_none(), "the default is not written");
    let back: Page = serde_json::from_value(v.clone()).expect("reads without the field");
    assert_eq!(back.layout, Layout::Graph);
    let mut explicit = v.clone();
    explicit["layout"] = json!("graph");
    let explicit: Page = serde_json::from_value(explicit).expect("graph is a layout");
    let html = render(&p);
    assert_eq!(html, render(&explicit), "layout: graph is the default");
    assert!(!html.contains("data-band"), "no band without the field");
    assert!(
        html.contains("data-region=\"front\""),
        "a region is a frame"
    );
    let mut banded = v;
    banded["layout"] = json!("bands");
    let banded: Page = serde_json::from_value(banded).expect("bands is a layout");
    assert_ne!(render(&banded), html);
}

#[test]
fn an_unknown_layout_is_refused() {
    let mut v = serde_json::to_value(bands_page(["front", "store"])).expect("serializes");
    assert_eq!(v["layout"], "bands");
    v["layout"] = json!("grid");
    let err = serde_json::from_value::<Page>(v).expect_err("grid is no layout");
    assert!(err.to_string().contains("grid"), "{err}");
}
