//! Lanes between bands: edges whose runs through a gap do not overlap share a lane, edges that do
//! overlap take lanes of their own, and edges with one label out of one card or into one card run
//! as one bundle on one lane with one label. Two edges are never drawn over one another on a lane
//! unless they carry one label. Inputs: pages built here and `tests/fixtures/bands-109.json`.

use super::graph::{attr, numbers, svg_of};
use agenticspec_render::page::{Edge, Layout, Node, Page, Region, render};

fn node(id: &str, region: &str) -> Node {
    Node {
        id: id.to_string(),
        label: format!("{id} label"),
        group: "service".to_string(),
        region: Some(region.to_string()),
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

/// Two bands, `up` with the cards `u0`…`u5` in one row and `down` with `d0`…`d5`, and `edges`
/// from `up` to `down`.
fn two_bands(edges: &[(&str, &str, &str)]) -> Page {
    let mut page = Page::new("lanes");
    page.layout = Layout::Bands;
    page.regions = ["up", "down"]
        .iter()
        .map(|id| Region {
            id: id.to_string(),
            label: format!("{id} band"),
            ..Region::default()
        })
        .collect();
    for i in 0..6 {
        page.nodes.push(node(&format!("u{i}"), "up"));
    }
    for i in 0..6 {
        page.nodes.push(node(&format!("d{i}"), "down"));
    }
    page.edges = edges.iter().map(|(f, t, l)| edge(f, t, l)).collect();
    page
}

/// Every label pill: its text and its box.
fn pills(svg: &str) -> Vec<(String, [f64; 4])> {
    let mut out = Vec::new();
    for piece in svg.split("<g class=\"pill\">").skip(1) {
        let b = numbers(attr(piece, "data-box").expect("a pill box"));
        let start = piece.find("<text").expect("a text");
        let text = &piece[start..];
        let text =
            &text[text.find('>').expect("tag ends") + 1..text.find("</text>").expect("ends")];
        out.push((text.to_string(), [b[0], b[1], b[2], b[3]]));
    }
    out
}

fn pills_of(svg: &str, label: &str) -> Vec<[f64; 4]> {
    pills(svg)
        .into_iter()
        .filter(|p| p.0 == label)
        .map(|p| p.1)
        .collect()
}

/// The centre height of the one pill `label`.
fn lane_of(svg: &str, label: &str) -> f64 {
    let found = pills_of(svg, label);
    assert_eq!(found.len(), 1, "one pill {label}: {found:?}");
    found[0][1] + found[0][3] / 2.0
}

/// An edge path: from, to, label and its corners.
type Drawn = (String, String, String, Vec<(f64, f64)>);

/// Every edge path.
fn paths(svg: &str) -> Vec<Drawn> {
    let mut out = Vec::new();
    for piece in svg.split("<path class=\"edge").skip(1) {
        let head = &piece[..piece.find('>').expect("tag ends")];
        let title = &piece
            [piece.find("<title>").expect("a title") + 7..piece.find("</title>").expect("ends")];
        let label = title.split_once(": ").map_or("", |t| t.1).to_string();
        let d = attr(head, "d").expect("a path");
        let mut corners = Vec::new();
        // A path of curves runs inside a band; only straight runs are compared.
        if d.contains(|c: char| c.is_ascii_alphabetic() && c != 'M' && c != 'L') {
            out.push((
                attr(head, "data-from").expect("from").to_string(),
                attr(head, "data-to").expect("to").to_string(),
                label,
                corners,
            ));
            continue;
        }
        for p in d.split(['M', 'L']) {
            if !p.is_empty() {
                let v = numbers(p.trim());
                corners.push((v[0], v[1]));
            }
        }
        out.push((
            attr(head, "data-from").expect("from").to_string(),
            attr(head, "data-to").expect("to").to_string(),
            label,
            corners,
        ));
    }
    out
}

fn path_of(svg: &str, from: &str, to: &str) -> Vec<(f64, f64)> {
    paths(svg)
        .into_iter()
        .find(|p| p.0 == from && p.1 == to)
        .unwrap_or_else(|| panic!("edge {from} -> {to}"))
        .3
}

/// The bottom of band `up` and the top of band `down`.
fn gap(svg: &str) -> (f64, f64) {
    let band = |id: &str| {
        let at = svg
            .find(&format!("data-band=\"{id}\""))
            .unwrap_or_else(|| panic!("band {id}"));
        let b = numbers(attr(&svg[at..], "data-band-box").expect("a band box"));
        [b[0], b[1], b[2], b[3]]
    };
    let (u, d) = (band("up"), band("down"));
    (u[1] + u[3], d[1])
}

/// The horizontal runs of every edge path, with the edge's label: (y, x from, x to, label, edge).
fn runs(svg: &str) -> Vec<(f64, f64, f64, String, String)> {
    let mut out = Vec::new();
    for (from, to, label, corners) in paths(svg) {
        for w in corners.windows(2) {
            if (w[0].1 - w[1].1).abs() < 0.01 && (w[0].0 - w[1].0).abs() > 0.01 {
                out.push((
                    w[0].1,
                    w[0].0.min(w[1].0),
                    w[0].0.max(w[1].0),
                    label.clone(),
                    format!("{from} -> {to}"),
                ));
            }
        }
    }
    out
}

/// The box of every band.
fn band_boxes(svg: &str) -> Vec<[f64; 4]> {
    svg.split("data-band-box=\"")
        .skip(1)
        .map(|piece| {
            let b = numbers(&piece[..piece.find('"').expect("closed")]);
            [b[0], b[1], b[2], b[3]]
        })
        .collect()
}

/// No two pills overlap, and two edges that run over one stretch of one height in a gap between
/// bands carry one label.
fn distinguishable(svg: &str) -> Vec<String> {
    let bands = band_boxes(svg);
    let in_gap = |y: f64| !bands.iter().any(|b| y >= b[1] && y <= b[1] + b[3]);
    let mut problems = Vec::new();
    let all = pills(svg);
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            let (p, q) = (a.1, b.1);
            if p[0] < q[0] + q[2] && q[0] < p[0] + p[2] && p[1] < q[1] + q[3] && q[1] < p[1] + p[3]
            {
                problems.push(format!("pills {} {p:?} and {} {q:?} overlap", a.0, b.0));
            }
        }
    }
    let rs: Vec<_> = runs(svg).into_iter().filter(|r| in_gap(r.0)).collect();
    for (i, a) in rs.iter().enumerate() {
        for b in &rs[i + 1..] {
            if (a.0 - b.0).abs() < 0.01 && a.1 < b.2 - 0.01 && b.1 < a.2 - 0.01 && a.3 != b.3 {
                problems.push(format!(
                    "{} ({}) and {} ({}) run over one another at y {}",
                    a.4, a.3, b.4, b.3, a.0
                ));
            }
        }
    }
    problems
}

#[test]
fn two_edges_whose_runs_do_not_overlap_share_a_lane() {
    let html = render(&two_bands(&[("u0", "d0", "left"), ("u5", "d5", "right")]));
    let svg = svg_of(&html);
    assert_eq!(
        lane_of(svg, "left"),
        lane_of(svg, "right"),
        "the two edges share one lane"
    );
    let (top, bottom) = gap(svg);
    assert!(
        bottom - top < 26.0 * 2.0,
        "the gap holds one lane: {}",
        bottom - top
    );
    let problems = distinguishable(svg);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn two_edges_whose_runs_overlap_take_two_lanes() {
    let html = render(&two_bands(&[
        ("u0", "d5", "over"),
        ("u5", "d0", "back"),
        ("u1", "d4", "inner"),
    ]));
    let svg = svg_of(&html);
    let ys = [
        lane_of(svg, "over"),
        lane_of(svg, "back"),
        lane_of(svg, "inner"),
    ];
    for (i, a) in ys.iter().enumerate() {
        for b in &ys[i + 1..] {
            assert!((a - b).abs() >= 26.0, "lanes {ys:?} are 26 px apart");
        }
    }
    let problems = distinguishable(svg);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn edges_with_one_label_into_one_card_run_as_one_bundle() {
    let html = render(&two_bands(&[
        ("u0", "d3", "join"),
        ("u2", "d3", "join"),
        ("u5", "d3", "join"),
        ("u3", "d0", "fan"),
        ("u3", "d2", "fan"),
        ("u3", "d5", "fan"),
    ]));
    let svg = svg_of(&html);
    for (label, ends) in [
        ("join", [("u0", "d3"), ("u2", "d3"), ("u5", "d3")]),
        ("fan", [("u3", "d0"), ("u3", "d2"), ("u3", "d5")]),
    ] {
        let y = lane_of(svg, label);
        let drawn: Vec<Vec<(f64, f64)>> = ends.iter().map(|(f, t)| path_of(svg, f, t)).collect();
        for (p, (f, t)) in drawn.iter().zip(&ends) {
            assert!(
                p.iter().any(|c| (c.1 - y).abs() < 0.01),
                "{f} -> {t} runs on the lane of its bundle at {y}: {p:?}"
            );
        }
        // The shared end is one point: the bundle forks toward its other ends.
        let shared = |p: &Vec<(f64, f64)>| {
            if label == "join" {
                p[p.len() - 1]
            } else {
                p[0]
            }
        };
        assert!(
            drawn.iter().all(|p| shared(p) == shared(&drawn[0])),
            "the edges of bundle {label} share their end"
        );
    }
    let (top, bottom) = gap(svg);
    assert!(
        bottom - top < 26.0 * 3.0,
        "the two bundles take at most two lanes: {}",
        bottom - top
    );
    let problems = distinguishable(svg);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_edges_of_bands_109_are_distinguishable() {
    let page: Page = serde_json::from_str(include_str!("../fixtures/bands-109.json"))
        .expect("the fixture is a page");
    let html = render(&page);
    let svg = svg_of(&html);
    let problems = distinguishable(svg);
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
    // Every edge is drawn.
    assert_eq!(paths(svg).len(), page.edges.len());
    // Inside a band, an edge on its way to the band's side shares the room between two rows with
    // the others; how often two of different labels meet there is printed, not held.
    let rs = runs(svg);
    let met = rs
        .iter()
        .enumerate()
        .flat_map(|(i, a)| rs[i + 1..].iter().map(move |b| (a, b)))
        .filter(|(a, b)| {
            (a.0 - b.0).abs() < 0.01 && a.1 < b.2 - 0.01 && b.1 < a.2 - 0.01 && a.3 != b.3
        })
        .count();
    println!("bands-109: {met} pairs of runs of different labels meet inside bands");
}
