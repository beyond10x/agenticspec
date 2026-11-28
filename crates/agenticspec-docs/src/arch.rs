//! One page per architecture document, with its rendered graph and, when an experiment names it,
//! the result of a short optimization run.

use crate::SOURCE_BASE;
use crate::blocks::GENERATED_NOTE;
use crate::md::{attr, code, text, yaml};
use crate::model::Block;
use agenticspec_sim::check::check;
use agenticspec_sim::marked::collect;
use agenticspec_sim::whole::{measure, measure_of, single_point_parts};
use agenticspec_sim::{Experiment, run};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The search budget of the run shown on the docs site: a few evaluations of few seeds, so the
/// page regenerates in well under a second. The experiment file keeps its own budget.
pub const DOCS_BUDGET: usize = 40;
pub const DOCS_SEEDS: u64 = 2;

pub struct Architecture {
    /// The file name without `.yaml`.
    pub name: String,
    pub file: String,
    pub doc: Value,
    pub experiment: Option<Run>,
    /// The page and its graph, below the site root: `architectures/solo`,
    /// `fields/coding/auto-merger`.
    pub slug: String,
    /// The way up from the page to `docs/`: `../` or `../../`.
    pub up: String,
    /// What the page is generated from, for its note.
    pub sources: String,
}

pub struct Run {
    pub file: String,
    pub name: String,
    pub budget: usize,
    pub seeds: u64,
    /// The weight of each KPI in the search's fitness: above 0 more is better, below 0 less.
    pub weights: BTreeMap<String, f64>,
    pub result: Value,
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "yaml" || x == "yml"))
        .collect();
    out.sort();
    out
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Every architecture under `architectures/`, by file name, each with the first experiment under
/// `experiments/` that names it, run with the docs budget.
pub fn load(root: &Path) -> Result<Vec<Architecture>, String> {
    let experiments = yaml_files(&root.join("experiments"));
    let mut out = Vec::new();
    for path in yaml_files(&root.join("architectures")) {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let doc = agenticspec_sim::doc::load_yaml(&path)
            .map_err(|e| format!("architectures/{name}.yaml: {e}"))?;
        let mut experiment = None;
        for ep in &experiments {
            let Ok(edoc) = agenticspec_sim::doc::load_yaml(ep) else {
                continue;
            };
            let Some(arch) = edoc["architecture"].as_str() else {
                continue;
            };
            let dir = ep.parent().unwrap_or(Path::new("."));
            if !same_file(&dir.join(arch), &path) {
                continue;
            }
            let file = format!(
                "experiments/{}",
                ep.file_name().unwrap_or_default().to_string_lossy()
            );
            let mut e = Experiment::load(ep).map_err(|err| format!("{file}: {err}"))?;
            let budget = e.search.budget.min(DOCS_BUDGET);
            let seeds = e.seeds.min(DOCS_SEEDS);
            e.search.budget = budget;
            e.search.restarts = e.search.restarts.min(1);
            e.seeds = seeds;
            if let Some(o) = e.doc.as_object_mut() {
                o.insert("seeds".into(), Value::from(seeds));
                if let Some(s) = o.get_mut("search").and_then(Value::as_object_mut) {
                    s.insert("budget".into(), Value::from(budget));
                    s.insert("restarts".into(), Value::from(e.search.restarts));
                }
            }
            let result = run(&e, 4).map_err(|err| format!("{file}: {err}"))?;
            experiment = Some(Run {
                name: edoc["experiment"]
                    .as_str()
                    .unwrap_or("experiment")
                    .to_string(),
                file,
                budget,
                seeds,
                weights: e.search.weights.clone(),
                result,
            });
            break;
        }
        out.push(Architecture {
            file: format!("architectures/{name}.yaml"),
            slug: format!("architectures/{name}"),
            up: "../".into(),
            sources: "architectures/ and experiments/".into(),
            name,
            doc,
            experiment,
        });
    }
    Ok(out)
}

/// The architectures a field lists (`fields/<field>/architectures/<name>.yaml`), without an
/// experiment: their pages go under `docs/fields/<field>/`.
pub fn load_field(field: &str, files: &[String], root: &Path) -> Result<Vec<Architecture>, String> {
    let mut out = Vec::new();
    for file in files {
        let path = root.join(file);
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let doc = agenticspec_sim::doc::load_yaml(&path).map_err(|e| format!("{file}: {e}"))?;
        out.push(Architecture {
            file: file.clone(),
            slug: format!("fields/{field}/{name}"),
            up: "../../".into(),
            sources: format!("fields/{field}/"),
            name,
            doc,
            experiment: None,
        });
    }
    Ok(out)
}

pub fn html(a: &Architecture) -> String {
    match &a.experiment {
        Some(r) => agenticspec_render::page(&r.result["architecture"], Some(&r.result)),
        None => agenticspec_render::page(&a.doc, None),
    }
}

/// A value as written, markers as `value (search: range)`.
fn value(v: &Value) -> String {
    match v {
        Value::Object(o) if o.contains_key("vary") => {
            let range = match &o["vary"] {
                Value::Array(xs) => xs.iter().map(value).collect::<Vec<_>>().join(", "),
                Value::Object(r) => {
                    let mut s = format!(
                        "{} to {}",
                        value(r.get("min").unwrap_or(&Value::Null)),
                        value(r.get("max").unwrap_or(&Value::Null))
                    );
                    if let Some(step) = r.get("step") {
                        let _ = write!(s, " by {}", value(step));
                    }
                    s
                }
                other => value(other),
            };
            format!(
                "{} <span className=\"as-tag as-tag--marked\">marked: {}</span>",
                value(o.get("value").unwrap_or(&Value::Null)),
                text(&range)
            )
        }
        Value::String(s) => text(s),
        Value::Array(xs) => xs.iter().map(value).collect::<Vec<_>>().join(", "),
        Value::Object(o) => o
            .iter()
            .map(|(k, v)| format!("{} {}", text(k), value(v)))
            .collect::<Vec<_>>()
            .join(", "),
        Value::Null => "–".into(),
        other => other.to_string(),
    }
}

fn block_link(blocks: &[Block], block: &str, up: &str) -> String {
    match blocks.iter().find(|b| b.name == block) {
        Some(b) => format!("[{}]({up}{}/{}/{}.md)", b.name, b.section, b.dir, b.page),
        None => code(block),
    }
}

fn number(v: &Value) -> String {
    match v.as_f64() {
        Some(x) if x.fract() == 0.0 && x.abs() < 1e12 => format!("{}", x as i64),
        Some(x) if x.abs() >= 100.0 => format!("{x:.1}"),
        Some(x) => format!("{x:.3}"),
        None => "–".into(),
    }
}

fn kpis_table(s: &mut String, result: &Value) {
    let base = &result["baseline"]["kpis"];
    let best = &result["best"]["kpis"];
    let Some(names) = base.as_object() else {
        return;
    };
    s.push_str("| KPI | kind | as built | best variant |\n|---|---|---|---|\n");
    for (name, k) in names {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} |",
            code(name),
            text(k["kind"].as_str().unwrap_or("")).to_lowercase(),
            number(&k["value"]),
            if best.is_null() {
                "–".into()
            } else {
                number(&best[name]["value"])
            }
        );
    }
    s.push('\n');
}

/// A weight or a difference with its sign, `+10`, `−0.05`.
fn signed(x: f64) -> String {
    if x > 0.0 {
        format!("+{}", number(&Value::from(x)))
    } else if x < 0.0 {
        format!("−{}", number(&Value::from(-x)))
    } else {
        "0".into()
    }
}

/// The order of the kinds of KPI in a table: the checks first.
fn kind_rank(kind: &str) -> u8 {
    match kind.to_ascii_lowercase().as_str() {
        "correctness" => 0,
        "completeness" => 1,
        _ => 2,
    }
}

/// The KPIs whose value differs between the architecture as built and the best variant: the ones
/// the search weighs first, each with the direction of its change.
fn moved_kpis(s: &mut String, r: &Run) {
    let base = &r.result["baseline"]["kpis"];
    let best = &r.result["best"]["kpis"];
    let Some(names) = base.as_object() else {
        return;
    };
    let mut moved: Vec<(&String, f64, f64, &str)> = names
        .iter()
        .filter_map(|(name, k)| {
            let a = k["value"].as_f64()?;
            let b = best[name]["value"].as_f64()?;
            ((a - b).abs() > 1e-9).then_some((name, a, b, k["kind"].as_str().unwrap_or("")))
        })
        .collect();
    moved.sort_by_key(|(name, _, _, kind)| {
        (
            !r.weights.contains_key(name.as_str()),
            kind_rank(kind),
            name.to_string(),
        )
    });
    let _ = writeln!(
        s,
        "{} of {} KPIs differ between the architecture as built and its best variant; the others are equal. \
A weight is what the experiment scores a KPI by: above 0 more is better, below 0 less is better.\n",
        moved.len(),
        names.len()
    );
    if moved.is_empty() {
        return;
    }
    s.push_str("| KPI | weight | as built | best variant | change |\n|---|---|---|---|---|\n");
    for (name, a, b, _) in moved {
        let d = b - a;
        let mut change = if a.abs() > 1e-9 {
            let pct = d / a.abs() * 100.0;
            format!("{}{:.1} %", if pct > 0.0 { "+" } else { "−" }, pct.abs())
        } else {
            signed(d)
        };
        let weight = r.weights.get(name.as_str()).copied();
        if let Some(w) = weight.filter(|w| *w != 0.0) {
            change.push_str(if (d > 0.0) == (w > 0.0) {
                ", better"
            } else {
                ", worse"
            });
        }
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} |",
            code(name),
            weight.map_or("–".into(), |w| format!(
                "{}{}",
                if w < 0.0 { "−" } else { "+" },
                w.abs()
            )),
            number(&Value::from(a)),
            number(&Value::from(b)),
            change
        );
    }
    s.push('\n');
}

/// A KPI of the architecture as a whole for a reader: whole numbers as such, else at most three
/// decimals without trailing zeros (`0.75`, `0.375`).
pub fn whole_value(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e12 {
        return format!("{}", x as i64);
    }
    let s = format!("{x:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The section "The architecture as a whole": the KPIs `agenticspec sim check` reports for `doc`,
/// and with `best` (the KPIs of a best variant in a result) their values there. `up` is the way
/// up from the page to `docs/`.
pub fn whole_section(s: &mut String, doc: &Value, best: Option<&Value>, up: &str) {
    s.push_str("## The architecture as a whole\n\n");
    let _ = write!(
        s,
        "Read from the document alone, before any simulation: its size, autonomy, exposure, weak points and fixed limits. \
`agenticspec sim check` reports the same numbers; a marked place counts at its value as built. \
The KPIs are in the [KPI catalogue]({up}kpis-catalogue.md) under the block `architecture`.\n\n",
    );
    let points = single_point_parts(doc);
    let best = best.filter(|b| b.is_object());
    if best.is_some() {
        s.push_str("| KPI | as written | optimized | what it counts |\n|---|---|---|---|\n");
    } else {
        s.push_str("| KPI | value | what it counts |\n|---|---|---|\n");
    }
    for (id, v) in measure(doc) {
        let mut cell = whole_value(v);
        if id == "single_points" && !points.is_empty() {
            let names: Vec<String> = points.iter().map(|p| code(p)).collect();
            let _ = write!(cell, " ({})", names.join(", "));
        }
        let what = measure_of(id).map_or("", |m| m.what);
        match best {
            Some(b) => {
                let _ = writeln!(
                    s,
                    "| {} | {cell} | {} | {} |",
                    code(id),
                    b[id]["value"].as_f64().map_or("–".into(), whole_value),
                    text(what)
                );
            }
            None => {
                let _ = writeln!(s, "| {} | {cell} | {} |", code(id), text(what));
            }
        }
    }
    s.push('\n');
}

/// The table of the marked places, with the best value of the search when there is one.
fn marked_places(s: &mut String, doc: &Value, best: Option<&Value>) {
    s.push_str("## Marked places\n\n");
    match collect(doc, false) {
        Ok(marked) if !marked.is_empty() => {
            s.push_str("A marked place carries `vary`: the value as built and the values a search may try there.");
            if best.is_some() {
                s.push_str(" The best value is that of the best variant.");
            }
            s.push_str("\n\n");
            if best.is_some() {
                s.push_str(
                    "| place | as built | best value | the search may try |\n|---|---|---|---|\n",
                );
            } else {
                s.push_str("| place | as built | the search may try |\n|---|---|---|\n");
            }
            for m in marked {
                let range: Vec<String> = m.range.iter().map(value).collect();
                let best_cell = match best {
                    Some(b) => format!(" {} |", b.get(&m.path).map_or("–".to_string(), value)),
                    None => String::new(),
                };
                let _ = writeln!(
                    s,
                    "| {} | {} |{best_cell} {} |",
                    code(&m.path),
                    value(&m.value),
                    range.join(", ")
                );
            }
            s.push('\n');
        }
        Ok(_) => s.push_str("None: this architecture is not searched.\n\n"),
        Err(e) => {
            let _ = writeln!(s, "The marked places cannot be read: {}.\n", text(&e));
        }
    }
}

pub fn page(a: &Architecture, blocks: &[Block]) -> String {
    let doc = &a.doc;
    let summary = doc["summary"].as_str().unwrap_or("");
    let mut s = String::new();
    s.push_str("---\n");
    let _ = writeln!(s, "title: {}", yaml(&a.name));
    let _ = writeln!(s, "sidebar_label: {}", yaml(&a.name));
    let _ = writeln!(s, "slug: /{}", a.slug);
    if !summary.is_empty() {
        let _ = writeln!(s, "description: {}", yaml(summary));
        let _ = writeln!(s, "lede: {}", yaml(summary));
    }
    let _ = writeln!(s, "kicker: {}", yaml("Architecture"));
    let _ = writeln!(s, "source: {}", yaml(&a.file));
    let _ = writeln!(
        s,
        "source_url: {}",
        yaml(&format!("{SOURCE_BASE}{}", a.file))
    );
    s.push_str("mdx:\n  format: mdx\n---\n\n");
    s.push_str(&GENERATED_NOTE.replace("the specification in blocks/", &a.sources));

    let refusals = check(doc);
    let parts = doc["parts"].as_object().cloned().unwrap_or_default();
    let links = doc["links"].as_array().cloned().unwrap_or_default();
    // Without `.html`: `docusaurus serve` answers `<name>.html` with a redirect that drops the base
    // path, and GitHub Pages answers `<name>` with `<name>.html`.
    let html_path = format!("/{}", a.slug);
    let caption = match &a.experiment {
        Some(r) => format!(
            "The architecture as a graph, with the best variant of a short run of the experiment {} at its marked places.",
            r.name
        ),
        None => "The architecture as a graph: parts by layer, links labelled by what they carry, the border marked.".to_string(),
    };
    let frame = format!(
        "## Graph\n\n<ArchitectureFrame name={} src={} caption={} />\n\n",
        attr(&a.name),
        attr(&html_path),
        attr(&caption)
    );

    // With an experiment the result of its search comes first, the graph after it.
    let best = a
        .experiment
        .as_ref()
        .map(|r| &r.result["best"]["params"])
        .filter(|p| p.is_object());
    if let Some(r) = &a.experiment {
        let res = &r.result;
        let variants = res["variants"].as_array().map_or(0, |v| v.len());
        s.push_str("## Search result\n\n");
        if best.is_some() {
            let _ = writeln!(
                s,
                "A short run of the experiment [{}]({SOURCE_BASE}{}) tried {variants} variants of the marked places. \
Its fitness, the weighted sum of KPIs the experiment scores, is {} as built and {} for the best variant. \
How the run was made: [Optimization](#optimization).\n",
                text(&r.name),
                r.file,
                number(&res["baseline"]["fitness"]),
                number(&res["best"]["fitness"])
            );
        } else {
            let _ = writeln!(
                s,
                "A short run of the experiment [{}]({SOURCE_BASE}{}) tried {variants} variants of the marked places and found none that passes its gates. \
How the run was made: [Optimization](#optimization).\n",
                text(&r.name),
                r.file
            );
        }
        marked_places(&mut s, doc, best);
        if best.is_some() {
            s.push_str("## KPIs that moved\n\n");
            moved_kpis(&mut s, r);
        }
    }
    s.push_str(&frame);

    s.push_str("## Check\n\n");
    if refusals.is_empty() {
        let _ = writeln!(
            s,
            "<span className=\"as-tag as-tag--accepted\">accepted</span> {} parts and {} links; the structural check refuses nothing.\n",
            parts.len(),
            links.len()
        );
    } else {
        let _ = writeln!(
            s,
            "<span className=\"as-tag as-tag--refused\">refused</span> {} refusal{}:\n",
            refusals.len(),
            if refusals.len() == 1 { "" } else { "s" }
        );
        for r in &refusals {
            let _ = writeln!(s, "- {}: {}", code(r.rule()), text(&r.to_string()));
        }
        s.push('\n');
    }

    whole_section(&mut s, doc, None, &a.up);

    s.push_str("## Parts\n\n| part | block | settings |\n|---|---|---|\n");
    // Parts in layer order of their blocks, the principals first, as the graph draws them.
    let layer_of = |block: &str| {
        let name = if block == "pool" { "resource" } else { block };
        blocks
            .iter()
            .find(|b| b.name == name)
            .map_or(99, |b| b.layer)
    };
    let mut ordered: Vec<(&String, &Value)> = parts.iter().collect();
    ordered.sort_by_key(|(n, p)| (layer_of(p["block"].as_str().unwrap_or("")), n.to_string()));
    for (name, p) in ordered {
        let block = p["block"].as_str().unwrap_or("?");
        let settings: Map<String, Value> = p
            .as_object()
            .map(|o| {
                o.iter()
                    .filter(|(k, _)| k.as_str() != "block")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let set: Vec<String> = settings
            .iter()
            .map(|(k, v)| format!("{} {}", text(k), value(v)))
            .collect();
        let _ = writeln!(
            s,
            "| {} | {} | {} |",
            code(name),
            block_link(blocks, block, &a.up),
            if set.is_empty() {
                "–".into()
            } else {
                set.join("; ")
            }
        );
    }
    s.push('\n');

    s.push_str("## Links\n\nNo link, no reach: a part reaches only what a link names.\n\n");
    for l in &links {
        let mut line = format!(
            "- **{}** sends {} to **{}**",
            text(l["from"].as_str().unwrap_or("?")),
            code(l["carries"].as_str().unwrap_or("?")),
            text(l["to"].as_str().unwrap_or("?"))
        );
        if let Some(over) = l["over"].as_str() {
            let _ = write!(line, " over **{}**", text(over));
        }
        line.push('.');
        let _ = writeln!(s, "{line}");
    }
    s.push('\n');

    let border: Vec<String> = doc["border"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|b| format!("**{}**", text(b)))
        .collect();
    if !border.is_empty() {
        let _ = writeln!(
            s,
            "## Border\n\nWhat a human reaches directly: {}.\n",
            border.join(", ")
        );
    }

    if let Some(policies) = doc["policies"].as_object().filter(|p| !p.is_empty()) {
        s.push_str("## Policies\n\n| policy | value |\n|---|---|\n");
        for (k, v) in policies {
            let _ = writeln!(s, "| {} | {} |", code(k), value(v));
        }
        s.push('\n');
    }

    match &a.experiment {
        None => marked_places(&mut s, doc, None),
        Some(r) => {
            let _ = writeln!(s, "## Optimization\n");
            let _ = writeln!(
                s,
                "The experiment [{}]({SOURCE_BASE}{}) searches the marked places of this architecture under a behaviour profile. \
The figures on this page come from a short run of it: {} evaluations of {} seed{} each. The experiment file asks for more; \
`agenticspec sim run {}` runs it in full.\n",
                text(&r.name),
                r.file,
                r.budget,
                r.seeds,
                if r.seeds == 1 { "" } else { "s" },
                r.file
            );
            s.push_str("Every KPI of the architecture as built and of its best variant:\n\n");
            kpis_table(&mut s, &r.result);
        }
    }
    s
}

pub fn index(archs: &[Architecture]) -> String {
    let mut s = String::new();
    s.push_str("---\n");
    s.push_str("title: Architectures\nsidebar_label: Overview\nslug: /architectures\n");
    let _ = writeln!(
        s,
        "description: {}",
        yaml("Concrete setups composed of building blocks, checked and simulated.")
    );
    let _ = writeln!(
        s,
        "lede: {}",
        yaml(
            "An architecture composes building blocks into one concrete setup: which agents, which channels, which sandbox, how many. It is checked before anything is built and simulated to score it."
        )
    );
    let _ = writeln!(s, "source: {}", yaml("architectures/"));
    let _ = writeln!(
        s,
        "source_url: {}",
        yaml(&format!(
            "{}architectures",
            SOURCE_BASE.replace("/blob/", "/tree/")
        ))
    );
    s.push_str("mdx:\n  format: mdx\n---\n\n");
    s.push_str(&GENERATED_NOTE.replace(
        "the specification in blocks/",
        "architectures/ and experiments/",
    ));
    s.push_str(
        "An architecture document (format `agenticspec-architecture/1`) names its parts, each an instance of a block, \
the links between them (who may call or message whom, and what a link carries), the border a human reaches, and policies. \
A value can be marked with `vary`: the search of an experiment tries the values listed there.\n\n\
Each page shows the result of the structural check and the architecture drawn as a graph.\n\n",
    );
    s.push_str("| architecture | what it is | parts | links | check | experiment |\n|---|---|---|---|---|---|\n");
    for a in archs {
        let refusals = check(&a.doc).len();
        let _ = writeln!(
            s,
            "| [{}]({}.md) | {} | {} | {} | {} | {} |",
            a.name,
            a.name,
            text(a.doc["summary"].as_str().unwrap_or("")),
            a.doc["parts"].as_object().map_or(0, |p| p.len()),
            a.doc["links"].as_array().map_or(0, |l| l.len()),
            if refusals == 0 {
                "accepted".to_string()
            } else {
                format!("{refusals} refused")
            },
            a.experiment.as_ref().map_or("–".into(), |r| code(&r.name)),
        );
    }
    let events: Vec<String> = agenticspec_sim::events::triggers()
        .map(|(name, _)| code(name))
        .collect();
    let _ = write!(
        s,
        "\nWhat the simulator models:\n\n\
- **Agents.** Every agent part that does not act for a principal and is not a reviewer takes tasks. \
Its behaviour is the profile role its `role` names (`coding` when it names none); `harness` names the kind of harness and picks no role.\n\
- **Questions.** The part a `question` link ends at answers: a principal with the delay of its kind, an agent with the \
`answer_seconds` of its own role, whatever the part is called. An agent that acts for a principal passes questions that need the owner on to it.\n\
- **Models.** Each model call goes to one of the model parts the agent reaches over `model_call` links, drawn by the agent's \
`model_share` (a share per model part; equal shares when it gives none), and is priced by that model's `route` in the profile. \
A route the profile does not price is refused, by name.\n\
- **Runtimes.** A runtime part that is given runs (a `run` or `order` link, or a trigger) executes them with the profile role \
its `role` names: `task_turns` steps a run, each lasting `turn_seconds`, each a model call when the runtime reaches a model. \
A `question` link to a principal adds person steps, answered with the principal's delay. A runtime is not charged as a live agent.\n\
- **Triggers.** A trigger fires on its schedule, on a post to a shared channel, or on an event of the simulation: {}. \
An `on_event` trigger fires on that event at any part, or only at the part its `source` names; it may wake an agent or a runtime.\n",
        events.join(", ")
    );
    s
}
