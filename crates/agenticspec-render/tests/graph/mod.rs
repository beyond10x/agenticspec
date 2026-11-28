//! The graph of a large architecture: edges stay out of the cards they do not end at, links that
//! carry the same thing into one part share one label, and the page lists every part and link in
//! tables beside the graph. Inputs: every document under `architectures/` and the 37-part fixture.

use agenticspec_render::architecture::page;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_yaml_ng::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn large() -> Value {
    load(&root().join("crates/agenticspec-render/tests/fixtures/large-37.yaml"))
}

fn factory() -> Value {
    load(&root().join("architectures/ladder/09-factory.yaml"))
}

/// Every architecture document of the repository and the fixture, by name.
fn all() -> Vec<(String, Value)> {
    let mut out = vec![("large-37".to_string(), large())];
    for dir in ["architectures", "architectures/ladder"] {
        let mut files: Vec<PathBuf> = std::fs::read_dir(root().join(dir))
            .expect("architectures/")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
            .collect();
        files.sort();
        for f in files {
            let name = f.file_name().map(|n| n.to_string_lossy().into_owned());
            out.push((name.unwrap_or_default(), load(&f)));
        }
    }
    out
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

fn numbers(s: &str) -> Vec<f64> {
    s.split([' ', ','])
        .filter(|t| !t.is_empty())
        .map(|t| t.parse().unwrap_or_else(|_| panic!("a number: {t}")))
        .collect()
}

/// The box of the card of each part, and the channel cards drawn on edges.
fn cards(svg: &str) -> (BTreeMap<String, [f64; 4]>, Vec<String>) {
    let mut boxes = BTreeMap::new();
    let mut channels = Vec::new();
    for piece in svg.split("<g class=\"node").skip(1) {
        let head = &piece[..piece.find('>').expect("tag ends")];
        let Some(part) = attr(head, "data-part") else {
            continue;
        };
        let part = part.to_string();
        let rest = &piece[piece.find(" data-box=").expect("a card carries its box")..];
        let rect = &rest[..rest.find("/>").expect("rect ends")];
        let b = numbers(attr(rect, "data-box").expect("a card carries its box"));
        let b = [b[0], b[1], b[2], b[3]];
        if head.starts_with(" channel") {
            channels.push(part.clone());
        }
        boxes.insert(part, b);
    }
    (boxes, channels)
}

/// Points along a path of `M`, `L`, `H`, `V` and `C` commands, each curve sampled.
fn samples(d: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut cur = (0.0, 0.0);
    let mut cmd = ' ';
    let mut num = String::new();
    let mut args: Vec<f64> = Vec::new();
    let flush =
        |cmd: char, args: &mut Vec<f64>, cur: &mut (f64, f64), out: &mut Vec<(f64, f64)>| {
            match (cmd, args.len()) {
                ('M', 2) | ('L', 2) => {
                    let to = (args[0], args[1]);
                    if cmd == 'L' {
                        for k in 1..=20 {
                            let t = k as f64 / 20.0;
                            out.push((cur.0 + (to.0 - cur.0) * t, cur.1 + (to.1 - cur.1) * t));
                        }
                    } else {
                        out.push(to);
                    }
                    *cur = to;
                }
                ('C', 6) => {
                    let (p0, p1, p2, p3) = (
                        *cur,
                        (args[0], args[1]),
                        (args[2], args[3]),
                        (args[4], args[5]),
                    );
                    for k in 1..=40 {
                        let t = k as f64 / 40.0;
                        let u = 1.0 - t;
                        let x = u * u * u * p0.0
                            + 3.0 * u * u * t * p1.0
                            + 3.0 * u * t * t * p2.0
                            + t * t * t * p3.0;
                        let y = u * u * u * p0.1
                            + 3.0 * u * u * t * p1.1
                            + 3.0 * u * t * t * p2.1
                            + t * t * t * p3.1;
                        out.push((x, y));
                    }
                    *cur = p3;
                }
                _ => return,
            }
            args.clear();
        };
    for c in d.chars() {
        if c.is_ascii_alphabetic() {
            if !num.is_empty() {
                args.push(num.parse().expect("a number"));
                num.clear();
            }
            flush(cmd, &mut args, &mut cur, &mut out);
            cmd = c;
        } else if c == ' ' || c == ',' {
            if !num.is_empty() {
                args.push(num.parse().expect("a number"));
                num.clear();
            }
            flush(cmd, &mut args, &mut cur, &mut out);
        } else {
            num.push(c);
        }
    }
    if !num.is_empty() {
        args.push(num.parse().expect("a number"));
    }
    flush(cmd, &mut args, &mut cur, &mut out);
    out
}

fn svg_of(html: &str) -> &str {
    let start = html.find("<svg class=\"graph\"").expect("the graph");
    &html[start..start + html[start..].find("</svg>").expect("svg ends")]
}

#[test]
fn no_edge_passes_through_a_card_it_does_not_end_at() {
    let mut problems = Vec::new();
    for (name, doc) in all() {
        let html = page(&doc, None);
        let svg = svg_of(&html);
        let (boxes, channels) = cards(svg);
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
            for (part, b) in &boxes {
                if own.contains(b) || channels.contains(part) {
                    continue;
                }
                let inside = |(x, y): (f64, f64)| {
                    x > b[0] + 2.0
                        && x < b[0] + b[2] - 2.0
                        && y > b[1] + 2.0
                        && y < b[1] + b[3] - 2.0
                };
                if samples(d).into_iter().any(inside) {
                    problems.push(format!(
                        "{name}: edge {from} -> {to} crosses the card of {part}"
                    ));
                }
            }
        }
    }
    problems.dedup();
    assert!(
        problems.is_empty(),
        "{} edges cross cards:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn no_two_boxes_overlap_in_any_architecture() {
    let mut problems = Vec::new();
    for (name, doc) in all() {
        let html = page(&doc, None);
        let boxes: Vec<[f64; 4]> = svg_of(&html)
            .split("data-box=\"")
            .skip(1)
            .map(|piece| {
                let v = numbers(&piece[..piece.find('"').expect("closed")]);
                [v[0], v[1], v[2], v[3]]
            })
            .collect();
        for (i, a) in boxes.iter().enumerate() {
            for b in &boxes[i + 1..] {
                let apart = a[0] + a[2] <= b[0]
                    || b[0] + b[2] <= a[0]
                    || a[1] + a[3] <= b[1]
                    || b[1] + b[3] <= a[1];
                if !apart {
                    problems.push(format!("{name}: {a:?} and {b:?} overlap"));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

fn pills(svg: &str, text: &str) -> usize {
    svg.matches(&format!("text-anchor=\"middle\">{text}</text>"))
        .count()
}

#[test]
fn links_that_carry_one_thing_into_one_part_share_one_label() {
    let html = page(&factory(), None);
    let svg = svg_of(&html);
    // Eleven parts send a model call to the gateway, which sends one to the model beside it.
    assert!(
        pills(svg, "model_call") <= 2,
        "model_call labels: {}",
        pills(svg, "model_call")
    );
    let html = page(&large(), None);
    let svg = svg_of(&html);
    assert!(
        pills(svg, "model_call") <= 3,
        "model_call labels: {}",
        pills(svg, "model_call")
    );
    assert_eq!(
        svg.matches("<path class=\"edge\"").count(),
        90,
        "every link is still its own path"
    );
}

#[test]
fn the_page_lists_every_part_and_every_link_in_tables() {
    let doc = large();
    let html = page(&doc, None);
    let parts = doc["parts"].as_object().expect("parts");
    let links = doc["links"].as_array().expect("links");
    let table = |marker: &str| {
        let start = html
            .find(marker)
            .unwrap_or_else(|| panic!("{marker} is on the page"));
        let rest = &html[start..];
        rest[..rest.find("</table>").expect("table ends")].to_string()
    };
    let part_table = table("<table class=\"data mini parts\"");
    assert_eq!(part_table.matches("<tr data-part=").count(), parts.len());
    for name in parts.keys() {
        assert!(
            part_table.contains(&format!("data-pick=\"{name}\"")),
            "{name} opens from the table"
        );
    }
    let link_table = table("<table class=\"data mini links-table\"");
    assert_eq!(link_table.matches("<tr data-from=").count(), links.len());
    assert!(
        html.contains("<select data-link-filter"),
        "the links filter by part"
    );
    assert!(
        html.contains("class=\"graph-tools\" hidden"),
        "the zoom tools wait for the script"
    );
    for label in ["Zoom out", "Zoom in", "Fit to width", "Actual size"] {
        assert!(
            html.contains(&format!("aria-label=\"{label}\"")),
            "{label} button"
        );
    }
}

#[test]
fn the_large_fixture_is_wider_than_any_page() {
    let html = page(&large(), None);
    let svg = svg_of(&html);
    let w = numbers(attr(svg, "viewBox").expect("viewBox"))[2];
    assert!(w > 1600.0, "natural width {w}");
}
