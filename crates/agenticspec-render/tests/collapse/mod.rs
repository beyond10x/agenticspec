//! Regions drawn collapsed: a region with `collapsed: true` is one card (its label and the count
//! of nodes inside) in the page as served; edges into it are merged per pair of visible ends with
//! their count; each region's open state is in the page as a template the script swaps in, its
//! nodes and edges drawn one by one and a control that closes it; nested regions open on their
//! own; `Page::check` refuses a `collapsed` that is no boolean and a collapsed region that holds
//! no node.

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

fn region(id: &str, collapsed: bool, regions: Vec<Region>) -> Region {
    Region {
        id: id.to_string(),
        label: format!("{id} area"),
        collapsed,
        regions,
        ..Region::default()
    }
}

/// Two collapsed regions `a` (three nodes) and `c` (two nodes), an open region `b` (two nodes)
/// and a node `x` in none. Edges: three from `x` into `a`, two from `a` to `c`, one from `x` to
/// `c`, one inside `a`, one from `b` to `x`.
fn subsystems() -> Page {
    let mut p = Page::new("subsystems");
    p.regions = vec![
        region("a", true, Vec::new()),
        region("b", false, Vec::new()),
        region("c", true, Vec::new()),
    ];
    p.nodes = vec![
        node("x", None),
        node("a1", Some("a")),
        node("a2", Some("a")),
        node("a3", Some("a")),
        node("b1", Some("b")),
        node("b2", Some("b")),
        node("c1", Some("c")),
        node("c2", Some("c")),
    ];
    p.edges = vec![
        edge("x", "a1", "asks"),
        edge("x", "a2", "asks"),
        edge("x", "a3", "tells"),
        edge("a1", "c1", "writes"),
        edge("a2", "c2", "reads"),
        edge("x", "c1", "audits"),
        edge("a1", "a2", "inner"),
        edge("b1", "x", "feeds"),
    ];
    p
}

/// A collapsed region `outer` holding a node and a collapsed region `inner` of two nodes, and a
/// node `x` outside with an edge to each of the three.
fn nested() -> Page {
    let mut p = Page::new("nested");
    p.regions = vec![region(
        "outer",
        true,
        vec![region("inner", true, Vec::new())],
    )];
    p.nodes = vec![
        node("x", None),
        node("o1", Some("outer")),
        node("i1", Some("inner")),
        node("i2", Some("inner")),
    ];
    p.edges = vec![
        edge("x", "o1", "one"),
        edge("x", "i1", "two"),
        edge("x", "i2", "three"),
        edge("i1", "i2", "within"),
    ];
    p
}

/// The SVG of the page as served, without the templates of the other states.
fn served(html: &str) -> &str {
    let start = html.find("<svg class=\"graph\"").expect("the graph");
    let end = start + html[start..].find("</svg>").expect("the graph ends") + 6;
    &html[start..end]
}

/// The template of the state in which region `id` is open: its opening tag and its SVG.
fn state<'a>(html: &'a str, id: &str) -> (&'a str, &'a str) {
    let key = format!("<template data-state=\"{id}\"");
    let start = html
        .find(&key)
        .unwrap_or_else(|| panic!("a template for {id}"));
    let open_end = start + html[start..].find('>').expect("tag ends") + 1;
    let end = open_end + html[open_end..].find("</template>").expect("template ends");
    (&html[start..open_end], &html[open_end..end])
}

/// The edge paths of an SVG: from, to and the text of its title.
fn edges(svg: &str) -> Vec<(String, String, String)> {
    let attr = |tag: &str, name: &str| -> String {
        let at = tag.find(&format!("{name}=\"")).expect("the attribute") + name.len() + 2;
        tag[at..at + tag[at..].find('"').expect("closed")].to_string()
    };
    svg.match_indices("<path class=\"edge")
        .map(|(i, _)| {
            let tag = &svg[i..i + svg[i..].find('>').expect("tag ends")];
            let t = i + svg[i..].find("<title>").expect("a title") + 7;
            let title = &svg[t..t + svg[t..].find("</title>").expect("title ends")];
            (
                attr(tag, "data-from"),
                attr(tag, "data-to"),
                title.to_string(),
            )
        })
        .collect()
}

/// The text of the card of node `id`, from its group's opening tag to the next card.
fn card<'a>(svg: &'a str, id: &str) -> &'a str {
    let at = svg
        .find(&format!("data-part=\"{id}\""))
        .unwrap_or_else(|| panic!("{id} is drawn"));
    let open = svg[..at].rfind("<g ").expect("the card's group");
    let rest = &svg[at..];
    let end = rest.find("<g class=\"node").unwrap_or(rest.len());
    &svg[open..at + end]
}

/// Every box of an SVG, as `x y w h`.
fn boxes(svg: &str) -> Vec<[f64; 4]> {
    svg.split("data-box=\"")
        .skip(1)
        .map(|piece| {
            let v: Vec<f64> = piece[..piece.find('"').expect("closed")]
                .split(' ')
                .map(|n| n.parse().expect("a number"))
                .collect();
            [v[0], v[1], v[2], v[3]]
        })
        .collect()
}

fn apart(svg: &str) {
    let b = boxes(svg);
    for (i, ra) in b.iter().enumerate() {
        for rb in &b[i + 1..] {
            let ok = ra[0] + ra[2] <= rb[0]
                || rb[0] + rb[2] <= ra[0]
                || ra[1] + ra[3] <= rb[1]
                || rb[1] + rb[3] <= ra[1];
            assert!(ok, "boxes {ra:?} and {rb:?} overlap");
        }
    }
}

#[test]
fn a_collapsed_region_is_one_card_with_its_label_and_the_count_of_its_nodes() {
    let html = render(&subsystems());
    let svg = served(&html);
    for hidden in ["a1", "a2", "a3", "c1", "c2"] {
        assert!(
            !svg.contains(&format!("data-part=\"{hidden}\"")),
            "{hidden} is inside a collapsed region"
        );
    }
    for shown in ["x", "b1", "b2"] {
        assert!(svg.contains(&format!("data-part=\"{shown}\"")), "{shown}");
    }
    let a = card(svg, "a");
    assert!(a.contains("data-open=\"a\""), "{a}");
    assert!(
        a.contains(">a area<"),
        "the card shows the region's label: {a}"
    );
    assert!(a.contains(">3 nodes<"), "the card counts the nodes: {a}");
    assert!(
        a.contains("role=\"button\"") && a.contains("aria-label=\"Open region a area, 3 nodes\""),
        "{a}"
    );
    assert!(
        a.contains("aria-expanded=\"false\"") && !a.contains("aria-pressed"),
        "{a}"
    );
    assert!(card(svg, "c").contains(">2 nodes<"));
    assert!(
        !svg.contains("data-region=\"a\""),
        "a collapsed region draws no frame"
    );
    assert!(
        svg.contains("data-region=\"b\""),
        "an open region keeps its frame"
    );
    apart(svg);
}

#[test]
fn edges_into_a_collapsed_region_merge_into_one_per_pair_of_visible_ends_with_their_count() {
    let html = render(&subsystems());
    let e = edges(served(&html));
    let between = |f: &str, t: &str| -> Vec<&(String, String, String)> {
        e.iter().filter(|x| x.0 == f && x.1 == t).collect()
    };
    let xa = between("x", "a");
    assert_eq!(xa.len(), 1, "three edges from x into a are one: {e:?}");
    assert_eq!(xa[0].2, "x → a: ×3", "labelled with its count");
    let ac = between("a", "c");
    assert_eq!(
        ac.len(),
        1,
        "two edges between collapsed a and c are one: {e:?}"
    );
    assert_eq!(ac[0].2, "a → c: ×2");
    let xc = between("x", "c");
    assert_eq!(xc.len(), 1, "{e:?}");
    assert_eq!(
        xc[0].2, "x → c: audits",
        "one edge keeps its label, no count"
    );
    assert!(
        !e.iter().any(|x| x.0 == "a" && x.1 == "a"),
        "an edge inside a collapsed region is not drawn: {e:?}"
    );
    assert_eq!(
        between("b1", "x").len(),
        1,
        "an edge between open ends stays"
    );
    assert_eq!(e.len(), 4, "{e:?}");
    assert!(
        served(&html).contains(">×3<"),
        "the count is drawn as a label"
    );
}

#[test]
fn a_region_opens_in_place_from_its_state_and_closes_with_its_control() {
    let html = render(&subsystems());
    let (tag, svg) = state(&html, "a");
    assert!(
        tag.contains("data-up=\"\""),
        "closing a returns to the page as served: {tag}"
    );
    assert!(svg.starts_with("<svg class=\"graph\""), "{}", &svg[..40]);
    for shown in ["a1", "a2", "a3", "x", "b1"] {
        assert!(
            svg.contains(&format!("data-part=\"{shown}\"")),
            "{shown} in a's state"
        );
    }
    assert!(svg.contains("data-region=\"a\""), "a is a frame again");
    assert!(
        card(svg, "c").contains("data-open=\"c\""),
        "c stays collapsed"
    );
    let e = edges(svg);
    for (f, t) in [("x", "a1"), ("x", "a2"), ("x", "a3"), ("a1", "a2")] {
        assert!(
            e.iter().any(|x| x.0 == f && x.1 == t),
            "{f} -> {t} drawn on its own: {e:?}"
        );
    }
    assert!(
        e.iter()
            .any(|x| x.0 == "a1" && x.1 == "c" && x.2 == "a1 → c: writes"),
        "an edge into a still collapsed region lands on its card: {e:?}"
    );
    let close = svg.find("data-close=\"a\"").expect("a control closes a");
    let rest = &svg[close..close + svg[close..].find("</g>").expect("the control ends")];
    assert!(
        rest.contains("role=\"button\"")
            && rest.contains("aria-label=\"Close region a area\"")
            && rest.contains("aria-expanded=\"true\"")
            && rest.contains("tabindex=\"0\""),
        "{rest}"
    );
    assert!(
        !served(&html).contains("data-close="),
        "the page as served opens no region"
    );
    apart(svg);
    let (_, c) = state(&html, "c");
    assert!(
        card(c, "a").contains("data-open=\"a\""),
        "one region opens at a time"
    );
}

#[test]
fn a_nested_region_stays_collapsed_when_its_parent_opens() {
    let html = render(&nested());
    let svg = served(&html);
    let outer = card(svg, "outer");
    assert!(
        outer.contains(">3 nodes<"),
        "the inner region's nodes count: {outer}"
    );
    assert!(
        !svg.contains("data-part=\"inner\""),
        "inside a collapsed region"
    );
    let e = edges(svg);
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].2, "x → outer: ×3");

    let (tag, open) = state(&html, "outer");
    assert!(tag.contains("data-up=\"\""), "{tag}");
    assert!(open.contains("data-part=\"o1\""));
    assert!(
        card(open, "inner").contains(">2 nodes<"),
        "inner stays collapsed"
    );
    assert!(!open.contains("data-part=\"i1\""));
    let e = edges(open);
    assert!(
        e.iter()
            .any(|x| x.0 == "x" && x.1 == "inner" && x.2 == "x → inner: ×2"),
        "{e:?}"
    );

    let (tag, both) = state(&html, "inner");
    assert!(
        tag.contains("data-up=\"outer\""),
        "closing inner returns to outer open: {tag}"
    );
    for shown in ["o1", "i1", "i2"] {
        assert!(both.contains(&format!("data-part=\"{shown}\"")), "{shown}");
    }
    assert!(both.contains("data-close=\"outer\"") && both.contains("data-close=\"inner\""));
    apart(both);
}

#[test]
fn a_collapsed_band_is_its_band_holding_the_card() {
    let mut p = subsystems();
    p.layout = Layout::Bands;
    let html = render(&p);
    let svg = served(&html);
    assert!(svg.contains("data-band=\"a\""), "a keeps its band");
    assert!(card(svg, "a").contains(">3 nodes<"));
    assert!(!svg.contains("data-part=\"a1\""));
    let (_, open) = state(&html, "a");
    assert!(open.contains("data-part=\"a1\"") && open.contains("data-close=\"a\""));
    apart(open);
}

#[test]
fn a_page_without_a_collapsed_region_has_no_states() {
    let mut p = subsystems();
    for r in &mut p.regions {
        r.collapsed = false;
    }
    let html = render(&p);
    assert!(!html.contains("<template"));
    assert!(!served(&html).contains("data-open="));
    assert_eq!(edges(served(&html)).len(), 8);
}

#[test]
fn collapsed_is_read_from_json_and_must_be_a_boolean() {
    let mut v = serde_json::to_value(subsystems()).expect("serializes");
    assert_eq!(v["regions"][0]["collapsed"], json!(true));
    assert!(
        v["regions"][1].get("collapsed").is_none(),
        "false is left out"
    );
    let back: Page = serde_json::from_value(v.clone()).expect("a page");
    assert_eq!(back, subsystems());
    for bad in [json!("yes"), json!(1), json!(null)] {
        v["regions"][0]["collapsed"] = bad.clone();
        let err = serde_json::from_value::<Page>(v.clone())
            .expect_err("a collapsed that is no boolean")
            .to_string();
        assert!(
            err.starts_with("region_collapsed_invalid: the region \"a\""),
            "{bad}: {err}"
        );
    }
}

#[test]
fn a_collapsed_region_that_holds_no_node_is_refused() {
    let mut p = subsystems();
    p.regions.push(region("empty", true, Vec::new()));
    let err = p.check().expect_err("a card for nothing");
    assert!(
        err.starts_with("region_collapsed_empty: the region \"empty\""),
        "{err}"
    );
    let v = serde_json::to_value(&p).expect("serializes");
    let err = serde_json::from_value::<Page>(v)
        .expect_err("refused when read")
        .to_string();
    assert!(err.starts_with("region_collapsed_empty"), "{err}");
    // A region that holds only a collapsed region holds that region's nodes.
    assert!(nested().check().is_ok());
    // Its own node is the card, not a node inside it.
    let mut own = subsystems();
    own.regions.push(region("solo", true, Vec::new()));
    own.nodes.push(node("solo", None));
    assert!(
        own.check()
            .unwrap_err()
            .starts_with("region_collapsed_empty")
    );
}
