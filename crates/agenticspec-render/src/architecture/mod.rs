//! The page of an agenticspec architecture (`agenticspec-architecture/1`) and, when given, the
//! result of its optimization (`agenticspec-sim-result/1`), drawn through [`crate::page`].
//!
//! [`map::to_page`] turns the architecture into a render model page; this module writes what an
//! architecture page shows around its graph: the tables of parts and links, the details of each
//! part, the policies and the sections of a result.

mod map;
mod model;
mod optim;
mod result;

use crate::fmt::esc;
use crate::graph;
use crate::page::{self, Header, Shell, plural};
use map::{FAMILIES, family, subtitle};
use model::{Architecture, Outcome, lands_on};
use serde_json::Value;
use std::fmt::Write as _;

/// The architecture document as a render model page, with the marks of a result when given.
pub fn render_model(architecture: &Value, result: Option<&Value>) -> page::Page {
    let arch = Architecture::read(architecture);
    let outcome = result.map(Outcome::read);
    map::to_page(&arch, outcome.as_ref()).0
}

fn words(arch: &Architecture) -> graph::Words {
    graph::Words {
        title: format!("Architecture {}", arch.name),
        desc: format!(
            "{} parts in rows by block layer, {} links as arrows labelled by what they carry. The tables of parts and links beside the graph list every part and every link as text.",
            arch.parts.len(),
            arch.links.len()
        ),
    }
}

/// The graph of an architecture for a host that draws it inline (agenticstudio): its SVG alone,
/// without the page around it, its size in px, and the parts and links it shows, each part with
/// its properties. The SVG carries the classes of the page's stylesheet; the host styles them.
pub fn graph(architecture: &Value) -> Value {
    let arch = Architecture::read(architecture);
    let (model, mut decor) = map::to_page(&arch, None);
    decor.words = words(&arch);
    let g = graph::render(&model, &decor);
    let parts: Vec<Value> = arch
        .parts
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.name,
                "block": p.block,
                "family": family(&p.block).0,
                "kind": p.kind,
                "count": p.count,
                "per": p.per,
                "acts_for": p.acts_for,
                "border": arch.border.contains(&p.name),
                "subtitle": subtitle(p),
                "properties": p.props.iter().map(|(k, v)| [k.replace('_', " "), v.clone()]).collect::<Vec<_>>(),
            })
        })
        .collect();
    let links: Vec<Value> = arch
        .links
        .iter()
        .map(|l| serde_json::json!({"from": l.from, "to": l.to, "carries": l.carries, "over": l.over}))
        .collect();
    serde_json::json!({
        "name": arch.name,
        "summary": arch.summary,
        "svg": g.svg,
        "width": g.width,
        "height": g.height,
        "parts": parts,
        "links": links,
    })
}

/// The page for this architecture document and optional simulation result.
pub fn page(architecture: &Value, result: Option<&Value>) -> String {
    let arch = Architecture::read(architecture);
    let outcome = result.map(Outcome::read);
    let o = outcome.as_ref();
    let (model, mut decor) = map::to_page(&arch, o);
    decor.words = words(&arch);

    let mut side = String::new();
    parts_table(&mut side, &arch);
    details(&mut side, &arch, o);
    links_table(&mut side, &arch);
    policies(&mut side, &arch, o);
    let mut after = String::new();
    if let Some(o) = o {
        result::sections(&mut after, &arch, o);
    }
    let formats = [
        Some(if arch.format.is_empty() {
            "agenticspec-architecture/1"
        } else {
            arch.format.as_str()
        }),
        o.map(|_| "agenticspec-sim-result/1"),
    ];
    let footer = format!(
        "{} · rendered by agenticspec, self-contained, no network",
        formats
            .iter()
            .flatten()
            .map(|f| esc(f))
            .collect::<Vec<_>>()
            .join(" · ")
    );
    let shell = Shell {
        title: match o {
            Some(_) => format!("{} — architecture and optimization", arch.name),
            None => format!("{} — architecture", arch.name),
        },
        header: header(&arch, o),
        section: "Architecture".to_string(),
        graph_heading: "Parts and links".to_string(),
        graph_hint: format!(
            "Select a part for its properties{}",
            if o.is_some() { " and KPIs" } else { "" }
        ),
        legend: legend(&arch),
        side,
        after,
        footer,
        data: serde_json::json!({ "architecture": architecture, "result": result }),
        decor,
    };
    page::render_with(&model, &shell)
}

fn header(arch: &Architecture, o: Option<&Outcome>) -> Header {
    let places = arch.places().count();
    let mut facts = vec![
        format!("{} parts", arch.parts.len()),
        format!("{} links", arch.links.len()),
    ];
    if !arch.border.is_empty() {
        facts.push(format!("border: {}", arch.border.join(", ")));
    }
    if places > 0 {
        facts.push(format!("{places} marked places"));
    }
    Header {
        eyebrow: Some(
            if o.is_some() {
                "Architecture · optimization result"
            } else {
                "Architecture"
            }
            .to_string(),
        ),
        heading: arch.name.clone(),
        summary: (!arch.summary.is_empty()).then(|| arch.summary.clone()),
        facts,
    }
}

fn legend(arch: &Architecture) -> String {
    let mut h = String::from("<ul class=\"legend\" aria-label=\"Legend\">");
    for (fam, label, _) in FAMILIES {
        if arch.parts.iter().any(|p| family(&p.block).0 == fam) {
            let _ = write!(
                h,
                "<li><span class=\"swatch f-{fam}\"></span>{}</li>",
                esc(label)
            );
        }
    }
    if !arch.border.is_empty() {
        h.push_str(
            "<li><span class=\"swatch border-sample\"></span>border: what a human reaches</li>",
        );
    }
    if arch.parts.iter().any(|p| p.per.is_some()) {
        h.push_str(
            "<li><span class=\"swatch per-sample\"></span>one per owner, framed with it</li>",
        );
    }
    if arch.places().next().is_some() {
        h.push_str(
            "<li><span class=\"swatch chip-sample\"></span>marked place [search range]</li>",
        );
    }
    h.push_str("</ul>");
    h
}

/// How many links start at, end at or pass over a part.
fn link_count(arch: &Architecture, name: &str) -> usize {
    arch.links
        .iter()
        .filter(|l| l.from == name || l.to == name || l.over.as_deref() == Some(name))
        .count()
}

/// Every part, one row each, by block layer, then block, then name; a row selects its part.
fn parts_table(h: &mut String, arch: &Architecture) {
    let mut parts: Vec<&model::Part> = arch.parts.iter().collect();
    parts.sort_by(|a, b| {
        (family(&a.block).1, &a.block, &a.name).cmp(&(family(&b.block).1, &b.block, &b.name))
    });
    let _ = write!(
        h,
        "<div class=\"card parts-card\"><div class=\"card-head\"><h2>Parts</h2><span class=\"hint\">{} · select one</span></div><div class=\"table-wrap scroll-y\"><table class=\"data mini parts\"><thead><tr><th scope=\"col\">Part</th><th scope=\"col\">Block</th><th scope=\"col\" class=\"r\">Links</th></tr></thead><tbody>",
        plural(parts.len(), "part")
    );
    for p in parts {
        let _ = write!(
            h,
            "<tr data-part=\"{n}\"><th scope=\"row\"><button type=\"button\" class=\"pick\" data-pick=\"{n}\" aria-pressed=\"false\"><span class=\"swatch f-{}\"></span>{n}</button></th><td>{}</td><td class=\"r\">{}</td></tr>",
            family(&p.block).0,
            esc(&subtitle(p)),
            link_count(arch, &p.name),
            n = esc(&p.name)
        );
    }
    h.push_str("</tbody></table></div></div>");
}

/// Every link, one row each, filtered by the part it touches.
fn links_table(h: &mut String, arch: &Architecture) {
    if arch.links.is_empty() {
        return;
    }
    let _ = write!(
        h,
        "<div class=\"card links-card\"><div class=\"card-head\"><h2>Links</h2><span class=\"hint\" data-link-count>{}</span></div><div class=\"card-body filter-row\"><label>Part <select data-link-filter><option value=\"\">all parts</option>",
        plural(arch.links.len(), "link")
    );
    let mut names: Vec<&str> = arch.parts.iter().map(|p| p.name.as_str()).collect();
    names.sort_unstable();
    for n in names {
        let _ = write!(h, "<option>{}</option>", esc(n));
    }
    h.push_str("</select></label></div><div class=\"table-wrap scroll-y\"><table class=\"data mini links-table\"><thead><tr><th scope=\"col\">From</th><th scope=\"col\">Carries</th><th scope=\"col\">To</th></tr></thead><tbody>");
    for l in &arch.links {
        let (over_attr, over) = match &l.over {
            Some(c) => (
                format!(" data-over=\"{}\"", esc(c)),
                format!(" <span class=\"muted\">over {}</span>", esc(c)),
            ),
            None => (String::new(), String::new()),
        };
        let _ = write!(
            h,
            "<tr data-from=\"{f}\" data-to=\"{t}\"{over_attr}><td>{f}</td><td><span class=\"carries\">{}</span>{over}</td><td>{t}</td></tr>",
            esc(&l.carries),
            f = esc(&l.from),
            t = esc(&l.to)
        );
    }
    h.push_str("</tbody></table></div></div>");
}

fn details(h: &mut String, arch: &Architecture, o: Option<&Outcome>) {
    h.push_str("<div class=\"card details\"><div class=\"card-head\"><h2>Part</h2><span class=\"hint\" id=\"pin-hint\">hover or select</span></div><div class=\"card-body\" aria-live=\"polite\">");
    let _ = write!(
        h,
        "<section class=\"detail\" data-detail=\"\"><p class=\"muted\">Point at a part, or move to it with Tab and press Enter, to see its properties, its links{}.</p><dl class=\"kv\">",
        if o.is_some() {
            " and its KPIs at baseline and best"
        } else {
            ""
        }
    );
    let mut blocks: Vec<&str> = arch.parts.iter().map(|p| p.block.as_str()).collect();
    blocks.sort_unstable();
    blocks.dedup();
    let _ = write!(h, "<dt>blocks</dt><dd>{}</dd>", esc(&blocks.join(", ")));
    if !arch.border.is_empty() {
        let _ = write!(
            h,
            "<dt>border</dt><dd>{}</dd>",
            esc(&arch.border.join(", "))
        );
    }
    h.push_str("</dl></section>");
    for p in &arch.parts {
        let _ = write!(
            h,
            "<section class=\"detail\" data-detail=\"{}\" hidden><div class=\"detail-head\"><span class=\"swatch f-{}\"></span><h3>{}</h3></div><p class=\"muted\">{}</p><dl class=\"kv\">",
            esc(&p.name),
            family(&p.block).0,
            esc(&p.name),
            esc(&subtitle(p))
        );
        let mut rows: Vec<(String, String)> = vec![("block".into(), p.block.clone())];
        if let Some(k) = &p.kind {
            rows.push(("kind".into(), k.clone()));
        }
        if let Some(c) = &p.count {
            rows.push(("count".into(), c.clone()));
        }
        if let Some(per) = &p.per {
            rows.push(("one per".into(), per.clone()));
        }
        if let Some(a) = &p.acts_for {
            rows.push(("acts for".into(), a.clone()));
        }
        if arch.border.contains(&p.name) {
            rows.push(("border".into(), "a human reaches it".into()));
        }
        if let Some(n) = &p.nested {
            if let Some(name) = &n.name {
                rows.push(("architecture".into(), name.clone()));
            }
            if let Some(c) = n.parts {
                rows.push(("parts".into(), c.to_string()));
            }
            if let Some(d) = &n.digest {
                rows.push(("digest".into(), d.clone()));
            }
            if !n.ports.is_empty() {
                rows.push(("ports".into(), n.ports.join(", ")));
            }
        }
        rows.extend(
            p.props
                .iter()
                .map(|(k, v)| (k.replace('_', " "), v.clone())),
        );
        for (k, v) in rows {
            let _ = write!(h, "<dt>{}</dt><dd>{}</dd>", esc(&k), esc(&v));
        }
        h.push_str("</dl>");
        if !p.places.is_empty() {
            h.push_str("<h4>Marked places</h4><div class=\"chips\">");
            for pl in &p.places {
                h.push_str(&map::chip(pl, o).html(true));
            }
            h.push_str("</div>");
        }
        let links: Vec<_> = arch
            .links
            .iter()
            .filter(|l| {
                lands_on(&l.from, &p.name)
                    || lands_on(&l.to, &p.name)
                    || l.over.as_deref() == Some(&p.name)
            })
            .collect();
        if !links.is_empty() {
            h.push_str("<h4>Links</h4><ul class=\"links\">");
            for l in links {
                let over = l
                    .over
                    .as_ref()
                    .map(|c| format!(" <span class=\"muted\">over {}</span>", esc(c)))
                    .unwrap_or_default();
                let _ = write!(
                    h,
                    "<li>{} <span class=\"muted\">→</span> {} <span class=\"carries\">{}</span>{over}</li>",
                    esc(&l.from),
                    esc(&l.to),
                    esc(&l.carries)
                );
            }
            h.push_str("</ul>");
        }
        if let Some(o) = o {
            result::per_part(h, o, &p.name);
        }
        h.push_str("</section>");
    }
    h.push_str("</div></div>");
}

fn policies(h: &mut String, arch: &Architecture, o: Option<&Outcome>) {
    if arch.policies.is_empty() && arch.policy_places.is_empty() {
        return;
    }
    h.push_str("<div class=\"card policies\"><div class=\"card-head\"><h2>Policies</h2></div><div class=\"card-body\"><ul class=\"policy-list\">");
    for pl in &arch.policy_places {
        let _ = write!(
            h,
            "<li><span class=\"policy-name\">{}</span>{}</li>",
            esc(&pl.field.replace('_', " ")),
            map::chip(pl, o).html(false)
        );
    }
    for (k, v) in &arch.policies {
        let _ = write!(
            h,
            "<li><span class=\"policy-name\">{}</span><span class=\"policy-v\">{}</span></li>",
            esc(&k.replace('_', " ")),
            esc(v)
        );
    }
    h.push_str("</ul></div></div>");
}
