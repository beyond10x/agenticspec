//! A wide row wraps: the cards of one row of a band or a page, and the cards a region's frame
//! holds, go into as many rows as keep them within `WRAP_WIDTH`, in the order the layout gives
//! them, read row by row; edges between the rows stay out of the cards they do not end at; a row
//! that fits is drawn as before. Inputs: pages built here, the 37-part fixture and
//! `tests/fixtures/bands-109.json`.

use super::graph::{attr, cards, large, numbers, samples, svg_of};
use agenticspec_render::architecture::page;
use agenticspec_render::page::{Edge, Layout, Node, Page, Region, WRAP_WIDTH, render};
use std::collections::BTreeMap;

/// The room of the graph's view the fit is measured against: a reference width, and the view's
/// highest height less its border (`max-height: 640px` in the stylesheet).
const VIEW: (f64, f64) = (1200.0, 638.0);

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

fn region(id: &str) -> Region {
    Region {
        id: id.to_string(),
        label: format!("{id} band"),
        ..Region::default()
    }
}

/// Two bands: `hub` with one node, and `big` with `n` nodes `n00`… in one row, none linked to
/// another; the hub sends to one in five. With `head`, `big` also holds a node that sends to each
/// of them (one in three with a label of its own), from the row above.
fn band_page(n: usize, head: bool) -> Page {
    let mut page = Page::new("wrap");
    page.layout = Layout::Bands;
    page.regions = vec![region("hub"), region("big")];
    page.nodes.push(node("hub", Some("hub")));
    if head {
        page.nodes.push(node("head", Some("big")));
    }
    for i in 0..n {
        let id = format!("n{i:02}");
        page.nodes.push(node(&id, Some("big")));
        let label = if i % 3 == 0 {
            format!("task {i}")
        } else {
            "assign".to_string()
        };
        if head {
            page.edges.push(edge("head", &id, &label));
        }
        if i % 5 == 0 {
            page.edges.push(edge("hub", &id, "call"));
        }
    }
    page
}

fn ids(n: usize, prefix: &str) -> Vec<String> {
    (0..n).map(|i| format!("{prefix}{i:02}")).collect()
}

/// The cards of `ids` in rows: grouped by top, top to bottom, each row left to right.
fn rows_of(boxes: &BTreeMap<String, [f64; 4]>, ids: &[String]) -> Vec<Vec<(String, [f64; 4])>> {
    let mut rows: Vec<Vec<(String, [f64; 4])>> = Vec::new();
    let mut all: Vec<(String, [f64; 4])> = ids
        .iter()
        .map(|id| {
            let b = boxes
                .get(id)
                .copied()
                .unwrap_or_else(|| panic!("card {id}"));
            (id.clone(), b)
        })
        .collect();
    all.sort_by(|a, b| a.1[1].total_cmp(&b.1[1]).then(a.1[0].total_cmp(&b.1[0])));
    for (id, b) in all {
        match rows.last_mut() {
            Some(r) if (r[0].1[1] - b[1]).abs() < 1.0 => r.push((id, b)),
            _ => rows.push(vec![(id, b)]),
        }
    }
    for r in &mut rows {
        r.sort_by(|a, b| a.1[0].total_cmp(&b.1[0]));
    }
    rows
}

fn span(row: &[(String, [f64; 4])]) -> f64 {
    let left = row.iter().map(|c| c.1[0]).fold(f64::INFINITY, f64::min);
    let right = row
        .iter()
        .map(|c| c.1[0] + c.1[2])
        .fold(f64::NEG_INFINITY, f64::max);
    right - left
}

/// Every edge that passes through a card it does not start or end at.
fn crossings(svg: &str) -> Vec<String> {
    let (boxes, channels) = cards(svg);
    let mut problems = Vec::new();
    for piece in svg.split("<path class=\"edge\"").skip(1) {
        let head = &piece[..piece.find('>').expect("tag ends")];
        let from = attr(head, "data-from").expect("from");
        let to = attr(head, "data-to").expect("to");
        let over = attr(head, "data-over");
        let own: Vec<[f64; 4]> = [Some(from), Some(to), over]
            .into_iter()
            .flatten()
            .filter_map(|p| boxes.get(p).copied())
            .collect();
        let d = attr(head, "d").expect("a path");
        let points = samples(d);
        for (part, b) in &boxes {
            if own.contains(b) || channels.contains(part) {
                continue;
            }
            let inside = |&(x, y): &(f64, f64)| {
                x > b[0] + 2.0 && x < b[0] + b[2] - 2.0 && y > b[1] + 2.0 && y < b[1] + b[3] - 2.0
            };
            if points.iter().any(inside) {
                problems.push(format!("edge {from} -> {to} crosses the card of {part}"));
            }
        }
    }
    problems
}

/// The box of a band: its `data-band-box`.
fn band_box(svg: &str, id: &str) -> [f64; 4] {
    let at = svg
        .find(&format!("data-band=\"{id}\""))
        .unwrap_or_else(|| panic!("band {id}"));
    let rest = &svg[at..];
    let key = "data-band-box=\"";
    let start = rest.find(key).expect("a band box") + key.len();
    let b = numbers(&rest[start..start + rest[start..].find('"').expect("closed")]);
    [b[0], b[1], b[2], b[3]]
}

/// The box of the frame of region `id`.
fn frame_box(svg: &str, id: &str) -> [f64; 4] {
    let at = svg
        .find(&format!("data-region=\"{id}\""))
        .unwrap_or_else(|| panic!("frame {id}"));
    let rect = &svg[at..at + svg[at..].find("/>").expect("rect ends")];
    let n = |name| {
        attr(rect, name)
            .unwrap_or_else(|| panic!("{name}"))
            .parse::<f64>()
            .expect("a number")
    };
    [n("x"), n("y"), n("width"), n("height")]
}

/// The zoom the page opens at in the reference view: `min(1, room / natural)`.
fn fit(svg: &str) -> (f64, f64, f64) {
    let v = numbers(attr(svg, "viewBox").expect("viewBox"));
    let (w, h) = (v[2], v[3]);
    (w, h, 1f64.min(VIEW.0 / w).min(VIEW.1 / h))
}

/// The rows of the 65 cards of band `big` are each within the wrap width, and so is the band.
fn wrapped(svg: &str) -> Vec<Vec<(String, [f64; 4])>> {
    let (boxes, _) = cards(svg);
    let rows = rows_of(&boxes, &ids(65, "n"));
    assert!(rows.len() >= 3, "65 cards in {} rows", rows.len());
    for (k, r) in rows.iter().enumerate() {
        assert!(
            span(r) <= WRAP_WIDTH,
            "row {k} spans {} > {WRAP_WIDTH}",
            span(r)
        );
    }
    let sizes: Vec<usize> = rows.iter().map(Vec::len).collect();
    let (lo, hi) = (sizes.iter().min(), sizes.iter().max());
    assert!(
        hi.zip(lo).is_some_and(|(h, l)| h - l <= 1),
        "the rows share the cards evenly: {sizes:?}"
    );
    let b = band_box(svg, "big");
    assert!(
        b[2] <= WRAP_WIDTH + 2.0 * 18.0,
        "the band is {} wide, the wrap width {WRAP_WIDTH}",
        b[2]
    );
    assert!(
        b[2] / b[3] < 4.0,
        "the band is {}x wider than tall",
        b[2] / b[3]
    );
    let crossed = crossings(svg);
    assert!(crossed.is_empty(), "{}", crossed.join("\n"));
    rows
}

#[test]
fn a_band_of_65_nodes_wraps_into_rows_within_the_wrap_width_in_order() {
    // No edge inside the band: the one row holds the cards in the order of the page.
    let html = render(&band_page(65, false));
    let rows = wrapped(svg_of(&html));
    let read: Vec<String> = rows.iter().flatten().map(|c| c.0.clone()).collect();
    assert_eq!(
        read,
        ids(65, "n"),
        "the cards read row by row in their order"
    );

    // Edges from a node above to every card: each of them runs between the rows.
    let html = render(&band_page(65, true));
    let svg = svg_of(&html);
    wrapped(svg);
    for i in [0, 1, 33, 64] {
        assert!(
            svg.contains(&format!("data-from=\"head\" data-to=\"n{i:02}\"")),
            "the edge head -> n{i:02} is drawn"
        );
    }
}

#[test]
fn a_band_of_5_nodes_stays_one_row() {
    for head in [false, true] {
        let html = render(&band_page(5, head));
        let svg = svg_of(&html);
        let (boxes, _) = cards(svg);
        let rows = rows_of(&boxes, &ids(5, "n"));
        assert_eq!(rows.len(), 1, "5 cards in one row: {rows:?}");
        assert!(crossings(svg).is_empty());
    }
}

/// A region's frame of 40 node cards in a page laid out as a graph, a node outside that sends to
/// some of them, and edges between cards of the frame that are not neighbours.
fn region_page() -> Page {
    let mut page = Page::new("pool");
    page.regions = vec![Region {
        id: "pool".to_string(),
        label: "pool".to_string(),
        ..Region::default()
    }];
    page.nodes.push(node("src", None));
    page.nodes.push(node("sink", None));
    for i in 0..40 {
        let id = format!("m{i:02}");
        page.nodes.push(node(&id, Some("pool")));
        if i % 4 == 0 {
            page.edges.push(edge("src", &id, "feed"));
        }
        if i % 9 == 0 {
            page.edges.push(edge(&id, "sink", "drain"));
        }
    }
    page.edges.push(edge("m00", "m30", "hand"));
    page.edges.push(edge("m12", "m02", "back"));
    page.edges.push(edge("m05", "m06", "next"));
    page
}

#[test]
fn a_region_of_40_nodes_wraps_its_cards_into_rows_in_order() {
    let html = render(&region_page());
    let svg = svg_of(&html);
    let (boxes, _) = cards(svg);
    let want = ids(40, "m");
    let rows = rows_of(&boxes, &want);
    assert!(rows.len() >= 2, "40 cards in {} rows", rows.len());
    for (k, r) in rows.iter().enumerate() {
        assert!(
            span(r) <= WRAP_WIDTH,
            "row {k} spans {} > {WRAP_WIDTH}",
            span(r)
        );
    }
    let read: Vec<String> = rows.iter().flatten().map(|c| c.0.clone()).collect();
    assert_eq!(read, want, "the cards read row by row in their order");
    let f = frame_box(svg, "pool");
    assert!(
        f[2] <= WRAP_WIDTH + 100.0,
        "the frame is {} wide, the wrap width {WRAP_WIDTH}",
        f[2]
    );
    for id in &want {
        let b = boxes[id];
        assert!(
            b[0] >= f[0]
                && b[1] >= f[1]
                && b[0] + b[2] <= f[0] + f[2]
                && b[1] + b[3] <= f[1] + f[3],
            "card {id} {b:?} lies in the frame {f:?}"
        );
    }
    let crossed = crossings(svg);
    assert!(crossed.is_empty(), "{}", crossed.join("\n"));
    for (from, to) in [
        ("m00", "m30"),
        ("m12", "m02"),
        ("m05", "m06"),
        ("src", "m36"),
    ] {
        assert!(
            svg.contains(&format!("data-from=\"{from}\" data-to=\"{to}\"")),
            "the edge {from} -> {to} is drawn"
        );
    }
}

#[test]
fn a_wrapped_region_inside_a_band_keeps_edges_out_of_its_cards() {
    // The region nested in a band, the nodes outside it in a band of their own.
    let mut page = region_page();
    page.layout = Layout::Bands;
    let pool = page.regions.remove(0);
    let mut inner = region("inner");
    inner.regions = vec![pool];
    page.regions = vec![region("outer"), inner];
    for n in &mut page.nodes {
        if n.region.is_none() {
            n.region = Some("outer".to_string());
        }
    }
    let html = render(&page);
    let svg = svg_of(&html);
    let (boxes, _) = cards(svg);
    assert!(rows_of(&boxes, &ids(40, "m")).len() >= 2, "the frame wraps");
    let crossed = crossings(svg);
    assert!(crossed.is_empty(), "{}", crossed.join("\n"));
}

fn bands_109() -> Page {
    let text = include_str!("../fixtures/bands-109.json");
    serde_json::from_str(text).expect("the fixture is a page")
}

/// The opening zoom of the two large fixtures in the reference view. Before rows wrapped,
/// `bands-109` was 12408 × 5492 and opened at 0.097 (its band of 65 nodes in one row); the
/// 37-part fixture was 2761 × 1354 at 0.434, and keeps that: none of its rows of cards is wider
/// than the wrap width.
#[test]
fn large_graphs_open_at_a_larger_fit() {
    let fixture = bands_109();
    assert_eq!((fixture.nodes.len(), fixture.edges.len()), (109, 209));
    let html = render(&fixture);
    let svg = svg_of(&html);
    let (w, h, z) = fit(svg);
    println!("bands-109: {w} x {h}, fit {z:.3}");
    assert!(
        w <= WRAP_WIDTH + 2.0 * 18.0,
        "bands-109 is {w} wide, the wrap width {WRAP_WIDTH}"
    );
    assert!(z > 0.097, "bands-109 opens at {z:.3} ({w} x {h})");
    let crossed = crossings(svg);
    assert!(crossed.is_empty(), "{}", crossed.join("\n"));

    let html = page(&large(), None);
    let svg = svg_of(&html);
    let (w, h, z) = fit(svg);
    println!("large-37: {w} x {h}, fit {z:.3}");
    assert!(z >= 0.434, "large-37 opens at {z:.3} ({w} x {h})");
}
