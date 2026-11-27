//! The views that explain the optimization: objective, landscape, trade-offs, sensitivity, and
//! the variants table, on synthetic results in the `agenticspec-sim-result/1` format.

use agenticspec_render::page;
use serde_json::{Map, Value, json};

const WEIGHTS: [(&str, f64); 3] = [
    ("tasks_correct", 10.0),
    ("cost_usd", -0.05),
    ("owner_wait_share", -200.0),
];
const GATES: [&str; 2] = ["unreachable_parts", "swarm_needs_agent"];

struct Synth {
    mode: &'static str,
    places: Vec<(&'static str, Vec<Value>)>,
    points: Vec<Vec<usize>>,
    infeasible: Vec<Vec<usize>>,
    baseline: Vec<usize>,
}

fn kpis_of(p: &[usize], out: bool) -> Map<String, Value> {
    let s: usize = p.iter().enumerate().map(|(i, v)| v * (i + 2)).sum();
    let tc = 100.0 + 7.0 * s as f64 + if p[0] == 1 { 5.0 } else { 0.0 };
    let cost = 200.0 + 60.0 * p[0] as f64 + 10.0 * s as f64;
    let ows = 0.1 - 0.01 * p[p.len() - 1] as f64;
    let mut m = Map::new();
    let mut put = |k: &str, v: f64| {
        m.insert(k.to_string(), json!({"value": v, "kind": "Quality"}));
    };
    put("tasks_correct", tc);
    put("cost_usd", cost);
    put("owner_wait_share", ows);
    put("unreachable_parts", if out { 1.0 } else { 0.0 });
    put("swarm_needs_agent", 0.0);
    put("tokens_cache_read", 1_716_652_500.0);
    put("acts_for_without_part", 0.0);
    m
}

fn fitness(k: &Map<String, Value>) -> f64 {
    WEIGHTS
        .iter()
        .map(|(n, w)| w * k[*n]["value"].as_f64().unwrap())
        .sum()
}

impl Synth {
    fn params(&self, p: &[usize]) -> Value {
        Value::Object(
            self.places
                .iter()
                .zip(p)
                .map(|((path, r), i)| (path.to_string(), r[*i].clone()))
                .collect(),
        )
    }

    fn run(&self, p: &[usize], idx: usize) -> Value {
        let out = self.infeasible.iter().any(|q| q == p);
        let kpis = kpis_of(p, out);
        json!({
            "id": idx,
            "params": self.params(p),
            "fitness": fitness(&kpis),
            "feasible": !out,
            "kpis": kpis,
        })
    }

    fn result(&self) -> Value {
        let mut points = self.points.clone();
        if !points.contains(&self.baseline) {
            points.insert(0, self.baseline.clone());
        }
        let variants: Vec<Value> = points
            .iter()
            .enumerate()
            .map(|(i, p)| self.run(p, i))
            .collect();
        let base = variants
            .iter()
            .find(|v| v["params"] == self.params(&self.baseline))
            .unwrap()
            .clone();
        let best = variants
            .iter()
            .filter(|v| v["feasible"] == json!(true))
            .max_by(|a, b| {
                a["fitness"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&b["fitness"].as_f64().unwrap())
            })
            .unwrap()
            .clone();
        let trace: Vec<Value> = variants
            .iter()
            .enumerate()
            .map(|(i, v)| json!({"eval": i, "variant": i, "fitness": v["fitness"]}))
            .collect();
        let weights: Map<String, Value> = WEIGHTS
            .iter()
            .map(|(k, w)| (k.to_string(), json!(w)))
            .collect();
        json!({
            "format": "agenticspec-sim-result/1",
            "experiment": {"search": {"mode": self.mode, "weights": weights, "gates": GATES}},
            "marked": self.places.iter().zip(&self.baseline).map(|((p, r), b)| json!({"path": p, "value": r[*b], "range": r})).collect::<Vec<_>>(),
            "baseline": base,
            "best": best,
            "variants": variants,
            "trace": trace,
        })
    }
}

/// Two places, every combination evaluated, one of them infeasible.
fn exhaustive() -> Value {
    let places = vec![
        ("parts.team.count", vec![json!(4), json!(8), json!(12)]),
        ("parts.team.start_stagger", vec![json!("0s"), json!("3s")]),
    ];
    let mut points = Vec::new();
    for b in 0..2 {
        for a in 0..3 {
            points.push(vec![a, b]);
        }
    }
    Synth {
        mode: "exhaustive",
        places,
        points,
        infeasible: vec![vec![2, 0]],
        baseline: vec![1, 1],
    }
    .result()
}

/// Three places, a fit search that evaluated 6 of 12 combinations.
fn fit() -> Value {
    Synth {
        mode: "fit",
        places: vec![
            ("parts.a.count", vec![json!(1), json!(2), json!(3)]),
            ("parts.b.mode", vec![json!("x"), json!("y")]),
            ("policies.c", vec![json!(10), json!(20)]),
        ],
        points: vec![
            vec![0, 0, 0],
            vec![1, 0, 0],
            vec![2, 0, 0],
            vec![1, 1, 0],
            vec![1, 0, 1],
            vec![2, 1, 1],
        ],
        infeasible: vec![vec![2, 0, 0]],
        baseline: vec![0, 0, 0],
    }
    .result()
}

fn one_place() -> Value {
    Synth {
        mode: "exhaustive",
        places: vec![(
            "parts.w.count",
            vec![json!(2), json!(4), json!(6), json!(8)],
        )],
        points: vec![vec![0], vec![1], vec![2], vec![3]],
        infeasible: vec![],
        baseline: vec![1],
    }
    .result()
}

fn render(result: &Value) -> String {
    page(&json!({"architecture": "synthetic"}), Some(result))
}

/// The element of `tag` that starts at the first `marker`, up to its closing tag (not nested).
fn element<'a>(html: &'a str, marker: &str, tag: &str) -> &'a str {
    let start = html
        .find(marker)
        .unwrap_or_else(|| panic!("{marker} is on the page"));
    let open = html[..(start + marker.len()).min(html.len())]
        .rfind(&format!("<{tag}"))
        .unwrap_or_else(|| panic!("<{tag} before {marker}"));
    let mut depth = 0;
    let mut i = open;
    let (o, c) = (format!("<{tag}"), format!("</{tag}>"));
    loop {
        let next_o = html[i..].find(&o).map(|k| k + i);
        let next_c = html[i..].find(&c).map(|k| k + i).expect("closed");
        match next_o {
            Some(k) if k < next_c => {
                depth += 1;
                i = k + o.len();
            }
            _ => {
                depth -= 1;
                i = next_c + c.len();
                if depth == 0 {
                    return &html[open..i];
                }
            }
        }
    }
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let s = tag.find(&key)? + key.len();
    Some(&tag[s..s + tag[s..].find('"')?])
}

/// Every opening tag in `html` that contains `needle`.
fn tags<'a>(html: &'a str, needle: &str) -> Vec<&'a str> {
    html.split('<')
        .filter(|t| !t.starts_with('/') && t.contains(needle))
        .map(|t| &t[..t.find('>').unwrap_or(t.len())])
        .collect()
}

fn f(s: &str) -> f64 {
    s.parse().unwrap_or_else(|_| panic!("a number: {s}"))
}

#[test]
fn objective_shows_the_formula_each_weighted_kpi_and_its_contribution() {
    let r = exhaustive();
    let html = render(&r);
    let obj = element(&html, "class=\"card objective\"", "section");
    for term in [
        "10 × tasks_correct",
        "− 0.05 × cost_usd",
        "− 200 × owner_wait_share",
    ] {
        assert!(obj.contains(term), "the formula has {term}");
    }
    let rows = tags(obj, "data-term=\"");
    let names: Vec<&str> = rows.iter().map(|t| attr(t, "data-term").unwrap()).collect();
    assert_eq!(names.len(), 3, "one row per weighted KPI: {names:?}");
    let contribs: Vec<f64> = rows
        .iter()
        .map(|t| f(attr(t, "data-contribution").expect("contribution")))
        .collect();
    for w in contribs.windows(2) {
        assert!(w[0].abs() >= w[1].abs(), "ordered by size: {contribs:?}");
    }
    let change =
        r["best"]["fitness"].as_f64().unwrap() - r["baseline"]["fitness"].as_f64().unwrap();
    let sum: f64 = contribs.iter().sum();
    assert!(
        (sum - change).abs() < 1e-6,
        "contributions {sum} add up to the fitness change {change}"
    );
    assert_eq!(
        obj.matches("<svg class=\"contrib-bar\"").count(),
        3,
        "a bar per KPI"
    );
    for t in tags(obj, "svg class=\"contrib-bar\"") {
        assert_eq!(
            attr(t, "aria-hidden"),
            Some("true"),
            "the number beside the bar is the text"
        );
    }
    let gates = element(obj, "class=\"gates\"", "details");
    assert!(!gates.starts_with("<details class=\"gates\" open"));
    assert!(gates.contains("2 of 2 gate KPIs at 0"), "{gates}");
    for g in GATES {
        assert!(gates.contains(&format!(">{g}<")), "gate {g} listed");
    }
}

#[test]
fn the_full_kpi_table_is_collapsed_below_the_objective_weighted_and_gates_first() {
    let html = render(&exhaustive());
    let obj = html.find("class=\"card objective\"").expect("objective");
    let all = element(&html, "class=\"card all-kpis\"", "details");
    assert!(html.find("class=\"card all-kpis\"").unwrap() > obj);
    assert!(
        all.starts_with("<details class=\"card all-kpis\">"),
        "collapsed: {}",
        &all[..80]
    );
    assert!(all.contains("All 7 KPIs"));
    let order: Vec<&str> = tags(all, "data-kpi=\"")
        .iter()
        .map(|t| attr(t, "data-kpi").unwrap())
        .collect();
    assert_eq!(
        &order[3..5],
        &GATES[..],
        "gates after the weighted KPIs: {order:?}"
    );
    assert_eq!(&order[5..], &["acts_for_without_part", "tokens_cache_read"]);
    let tr = &all[all.find("data-kpi=\"unreachable_parts\"").unwrap()..];
    assert!(tr[..tr.find("</tr>").unwrap()].contains(">gate<"));
    assert!(
        !html.contains("<div class=\"two\"><details"),
        "the table spans the page"
    );
}

#[test]
fn landscape_of_two_places_is_a_heatmap_with_infeasible_and_best_marked() {
    let html = render(&exhaustive());
    let land = element(&html, "class=\"card landscape\"", "section");
    let svg = element(land, "class=\"heatmap\"", "svg");
    let open = &svg[..svg.find('>').unwrap()];
    assert_eq!(attr(open, "role"), Some("img"));
    let label = attr(open, "aria-labelledby").expect("labelled");
    let title = element(svg, &format!("id=\"{label}\""), "title");
    assert!(
        title.contains("team.count") && title.contains("team.start_stagger"),
        "{title}"
    );
    let cells: Vec<&str> = tags(svg, "data-cell=\"")
        .iter()
        .map(|t| attr(t, "data-cell").unwrap())
        .collect();
    assert_eq!(cells.len(), 6, "3 × 2 cells");
    assert_eq!(cells.iter().filter(|c| **c == "fit").count(), 5);
    assert_eq!(cells.iter().filter(|c| **c == "out").count(), 1);
    assert_eq!(
        svg.matches("class=\"best-ring\"").count(),
        1,
        "the best cell"
    );
    assert!(land.contains("Every combination was evaluated"));
    let table = element(land, "class=\"as-table\"", "details");
    assert_eq!(
        table.matches("<tr>").count() + table.matches("<tr ").count(),
        3,
        "header and two rows"
    );
    assert!(table.contains("infeasible"));
    assert!(
        !land.contains("<select"),
        "no pair to choose with two places"
    );
}

#[test]
fn landscape_after_a_fit_search_shows_cells_not_evaluated_and_a_pair_selector() {
    let html = render(&fit());
    let land = element(&html, "class=\"card landscape\"", "section");
    let select = element(land, "<select", "select");
    assert_eq!(select.matches("<option").count(), 3, "3 pairs of 3 places");
    assert!(land.contains("<label"), "the selector has a label");
    let figures = tags(land, "figure class=\"pair\"");
    assert_eq!(figures.len(), 3);
    assert_eq!(
        figures.iter().filter(|t| !t.contains(" hidden")).count(),
        1,
        "one pair shown"
    );
    assert!(
        land.contains("data-cell=\"none\""),
        "a cell the search did not run"
    );
    assert!(
        land.contains("not evaluated"),
        "the page says why cells are empty"
    );
}

#[test]
fn landscape_of_one_place_is_a_line() {
    let html = render(&one_place());
    let land = element(&html, "class=\"card landscape\"", "section");
    assert!(
        land.contains("class=\"sens line\""),
        "a line over the one place"
    );
    assert!(!land.contains("class=\"heatmap\""));
}

#[test]
fn trade_offs_scatter_every_variant_with_the_pareto_front_and_the_best() {
    let r = exhaustive();
    let html = render(&r);
    let card = element(&html, "class=\"card tradeoffs\"", "section");
    let svg = element(card, "class=\"scatter\"", "svg");
    let open = &svg[..svg.find('>').unwrap()];
    assert_eq!(attr(open, "role"), Some("img"));
    assert!(attr(open, "aria-labelledby").is_some());
    let pts = tags(svg, "data-v=\"");
    assert_eq!(
        pts.len(),
        r["variants"].as_array().unwrap().len(),
        "every variant"
    );
    assert_eq!(pts.iter().filter(|t| t.contains("best")).count(), 1);
    let fronts = tags(svg, "class=\"front");
    assert_eq!(
        fronts.len(),
        6,
        "one front per ordered pair of 3 weighted KPIs"
    );
    let shown: Vec<&&str> = fronts.iter().filter(|t| !t.contains(" off")).collect();
    assert_eq!(shown.len(), 1);
    assert_eq!(attr(shown[0], "data-pair"), Some("cost_usd|tasks_correct"));
    // The front: feasible variants no other feasible variant beats on lower cost and more tasks.
    let vs = r["variants"].as_array().unwrap();
    let kv = |v: &Value, k: &str| v["kpis"][k]["value"].as_f64().unwrap();
    let mut want: Vec<String> = vs
        .iter()
        .filter(|v| v["feasible"] == json!(true))
        .filter(|v| {
            !vs.iter().any(|w| {
                w["feasible"] == json!(true)
                    && kv(w, "cost_usd") <= kv(v, "cost_usd")
                    && kv(w, "tasks_correct") >= kv(v, "tasks_correct")
                    && (kv(w, "cost_usd") < kv(v, "cost_usd")
                        || kv(w, "tasks_correct") > kv(v, "tasks_correct"))
            })
        })
        .map(|v| v["id"].to_string())
        .collect();
    want.sort();
    let mut got: Vec<String> = attr(shown[0], "data-front")
        .unwrap()
        .split(' ')
        .map(str::to_string)
        .collect();
    got.sort();
    assert_eq!(got, want);
    for (axis, default) in [("x", "cost_usd"), ("y", "tasks_correct")] {
        let sel = element(card, &format!("data-axis=\"{axis}\""), "select");
        assert_eq!(sel.matches("<option").count(), 3, "the weighted KPIs");
        assert!(
            sel.contains(&format!("value=\"{default}\" selected")),
            "{sel}"
        );
    }
}

#[test]
fn sensitivity_draws_each_place_with_gaps_where_the_search_did_not_go() {
    let html = render(&fit());
    let card = element(&html, "class=\"card sensitivity\"", "section");
    let svgs = tags(card, "svg class=\"sens\"");
    assert_eq!(svgs.len(), 3, "one chart per marked place");
    for s in &svgs {
        assert_eq!(attr(s, "role"), Some("img"));
        assert!(attr(s, "aria-labelledby").is_some());
    }
    let best = &fit()["best"]["params"];
    // policies.c at 20 with a=best, b=best: evaluated only if the best is (1, x, *).
    assert!(best.is_object());
    assert!(
        card.contains("data-gap="),
        "a value with no evaluated variant is shown as a gap"
    );
    assert!(card.contains("class=\"as-table\""), "a table equivalent");
}

#[test]
fn the_variants_table_shows_the_weighted_kpis() {
    let html = render(&exhaustive());
    let table = element(&html, "class=\"data variants\"", "table");
    let head = element(table, "<thead", "thead");
    for (k, _) in WEIGHTS {
        assert!(
            head.contains(&format!("data-col=\"{k}\"")),
            "column {k}: {head}"
        );
    }
    assert!(!head.contains("acts_for_without_part"));
    assert!(!head.contains("tokens_cache_read"));
}

#[test]
fn large_numbers_are_compact_with_the_exact_value_in_a_title() {
    let html = render(&exhaustive());
    assert!(
        html.contains("<span title=\"1,716,652,500\">1.72 B</span>"),
        "compact token count"
    );
    assert!(!html.contains(">1,716,652,500<"));
}

/// Coordinates of every mark, and the estimated extent of every text, lie inside the viewBox.
#[test]
fn chart_marks_stay_inside_their_view_box() {
    for r in [exhaustive(), fit(), one_place()] {
        let html = render(&r);
        for class in ["heatmap", "scatter", "sens", "contrib-bar"] {
            for piece in html.split(&format!("<svg class=\"{class}")).skip(1) {
                let svg = &piece[..piece.find("</svg>").expect("closed")];
                let vb: Vec<f64> = attr(&format!(" {svg}"), "viewBox")
                    .expect("viewBox")
                    .split(' ')
                    .map(f)
                    .collect();
                let (w, h) = (vb[2], vb[3]);
                // Marks inside a translated group are drawn around its origin.
                let mut origin = (0.0, 0.0);
                for t in svg.split('<').skip(1) {
                    let t = &t[..t.find('>').unwrap_or(t.len())];
                    if t.starts_with("/g") {
                        origin = (0.0, 0.0);
                        continue;
                    }
                    if let Some(tr) = attr(t, "transform")
                        && let Some(inner) = tr.strip_prefix("translate(")
                    {
                        let xy: Vec<f64> = inner.trim_end_matches(')').split(',').map(f).collect();
                        assert!(
                            (0.0..=w).contains(&xy[0]) && (0.0..=h).contains(&xy[1]),
                            "{class}: {tr}"
                        );
                        origin = (xy[0], xy[1]);
                    }
                    for (a, lim, o) in [
                        ("x", w, origin.0),
                        ("cx", w, origin.0),
                        ("x1", w, origin.0),
                        ("x2", w, origin.0),
                        ("y", h, origin.1),
                        ("cy", h, origin.1),
                        ("y1", h, origin.1),
                        ("y2", h, origin.1),
                    ] {
                        if let Some(v) = attr(t, a) {
                            let v = f(v) + o;
                            assert!(
                                (-0.5..=lim + 0.5).contains(&v),
                                "{class}: {a}={v} outside 0..{lim} in <{t}>"
                            );
                        }
                    }
                }
                for piece in svg.split("<text").skip(1) {
                    let open = &piece[..piece.find('>').unwrap()];
                    let text = &piece[piece.find('>').unwrap() + 1..piece.find("</text>").unwrap()];
                    let len = text.replace("&amp;", "&").chars().count() as f64 * 0.6 * 11.5;
                    let x = f(attr(open, "x").unwrap_or("0"));
                    let (lo, hi) = match attr(open, "text-anchor") {
                        Some("end") => (x - len, x),
                        Some("middle") => (x - len / 2.0, x + len / 2.0),
                        _ => (x, x + len),
                    };
                    assert!(
                        lo >= -2.0 && hi <= w + 2.0,
                        "{class}: text {text:?} from {lo} to {hi} leaves 0..{w}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_optimization_views_are_deterministic() {
    for r in [exhaustive(), fit(), one_place()] {
        assert_eq!(render(&r), render(&r));
    }
}
