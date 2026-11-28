//! The page of the workstation architecture, with and without a simulation result.

mod graph;
mod optim;

use agenticspec_render::page;
use serde_json::{Value, json};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workstation() -> Value {
    let path = root().join("architectures/workstation.yaml");
    let text = std::fs::read_to_string(&path).expect("architectures/workstation.yaml");
    serde_yaml_ng::from_str(&text).expect("the workstation architecture is YAML")
}

fn result() -> Value {
    serde_json::from_str(include_str!("fixtures/workstation-result.json")).expect("fixture JSON")
}

/// The text of the element that starts at the first `marker`, up to the next `</tr>`.
fn row<'a>(html: &'a str, marker: &str) -> &'a str {
    let start = html
        .find(marker)
        .unwrap_or_else(|| panic!("{marker} is on the page"));
    let rest = &html[start..];
    &rest[..rest.find("</tr>").expect("the row ends")]
}

#[test]
fn every_part_and_every_link_label_is_drawn() {
    let arch = workstation();
    let html = page(&arch, None);
    let parts = arch["parts"].as_object().expect("parts");
    assert_eq!(parts.len(), 9);
    for name in parts.keys() {
        assert!(
            html.contains(&format!(">{name}<")),
            "part {name} is drawn as text"
        );
        assert!(
            html.contains(&format!("data-part=\"{name}\"")),
            "part {name} is a node"
        );
    }
    for link in arch["links"].as_array().expect("links") {
        let carries = link["carries"].as_str().expect("carries");
        assert!(
            html.contains(&format!(">{carries}<")),
            "link label {carries} is drawn"
        );
    }
    assert!(
        html.contains("class=\"border-mark\""),
        "the border is marked"
    );
    assert!(html.contains(">×12<"), "the worker count is a badge");
}

#[test]
fn marked_places_are_chips_with_their_range() {
    let html = page(&workstation(), None);
    assert!(html.contains("data-place=\"parts.builds.capacity\""));
    assert!(html.contains("8…32"), "the list range of capacity");
    assert!(
        html.contains("2…24"),
        "the min/max range of the worker count"
    );
    assert!(html.contains("data-place=\"policies.question_default_after\""));
}

#[test]
fn a_result_shows_kpis_best_values_and_the_change_at_marked_places() {
    let html = page(&workstation(), Some(&result()));
    let best = &result()["best"]["kpis"];
    let expected = [
        ("tasks_done", "188"),
        ("owner_wait_share", "0.12"),
        ("cost_usd", "276.9"),
        ("queue_wait_p90_seconds", "240"),
        ("questions_defaulted", "21"),
        ("unreachable_parts", "0"),
    ];
    for (name, value) in expected {
        assert!(best.get(name).is_some(), "fixture has {name}");
        let tr = row(&html, &format!("data-kpi=\"{name}\""));
        assert!(tr.contains(&format!(">{name}<")), "KPI {name} is named");
        assert!(
            tr.contains(&format!(">{value}<")),
            "KPI {name} shows best {value}: {tr}"
        );
    }
    assert!(
        html.contains("data-place=\"parts.builds.capacity\" data-baseline=\"16\" data-best=\"24\"")
    );
    assert!(
        html.contains("class=\"trace\""),
        "the search trace is drawn"
    );
    assert!(
        html.contains("data-variant=\"v007\""),
        "the best variant is ranked"
    );
    assert!(
        row(&html, "data-variant=\"v006\"").contains("infeasible"),
        "an infeasible variant is marked"
    );
    assert!(
        html.contains("data-per-part=\"builds\""),
        "per-part KPIs of builds"
    );
}

#[test]
fn the_page_loads_nothing_from_outside() {
    for html in [
        page(&workstation(), None),
        page(&workstation(), Some(&result())),
    ] {
        assert!(html.starts_with("<!doctype html>"));
        assert!(!html.contains("http://"), "no http:// reference");
        assert!(!html.contains("https://"), "no https:// reference");
        assert!(!html.contains(" src="), "no loaded script or image");
    }
}

#[test]
fn the_output_is_deterministic() {
    assert_eq!(page(&workstation(), None), page(&workstation(), None));
    assert_eq!(
        page(&workstation(), Some(&result())),
        page(&workstation(), Some(&result()))
    );
}

#[test]
fn no_two_boxes_of_the_graph_overlap() {
    for html in [
        page(&workstation(), None),
        page(&workstation(), Some(&result())),
    ] {
        let mut boxes: Vec<(String, [f64; 4])> = Vec::new();
        for piece in html.split("data-box=\"").skip(1) {
            let text = &piece[..piece.find('"').expect("closed")];
            let v: Vec<f64> = text
                .split(' ')
                .map(|n| n.parse().expect("a number"))
                .collect();
            boxes.push((text.to_string(), [v[0], v[1], v[2], v[3]]));
        }
        assert!(
            boxes.len() >= 9 + 4,
            "nodes and edge labels carry a box (links with one label into one part share one): {}",
            boxes.len()
        );
        for (i, (a, ra)) in boxes.iter().enumerate() {
            for (b, rb) in &boxes[i + 1..] {
                let apart = ra[0] + ra[2] <= rb[0]
                    || rb[0] + rb[2] <= ra[0]
                    || ra[1] + ra[3] <= rb[1]
                    || rb[1] + rb[3] <= ra[1];
                assert!(apart, "boxes {a} and {b} overlap");
            }
        }
    }
}

#[test]
fn absent_and_partial_inputs_render() {
    let empty = page(&json!({}), None);
    assert!(empty.contains("</html>"));
    let partial = page(
        &workstation(),
        Some(&json!({ "best": { "kpis": { "tasks_done": 3 } } })),
    );
    assert!(row(&partial, "data-kpi=\"tasks_done\"").contains(">3<"));
    let hostile = page(
        &json!({ "architecture": "</script><b>x", "parts": { "a<b": { "block": "agent" } } }),
        None,
    );
    assert!(!hostile.contains("</script><b>"));
    assert!(!hostile.contains(">a<b<"));
}
