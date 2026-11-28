//! One self-contained HTML page for an agenticspec architecture (`agenticspec-architecture/1`)
//! and, when given, the result of its optimization (`agenticspec-sim-result/1`).
//!
//! The page holds its CSS, its script and its SVG inline, embeds both documents as JSON, and
//! loads nothing from the network. The output is a function of the two inputs only.

mod fmt;
mod graph;
mod model;
mod optim;
mod result;

use fmt::esc;
use graph::{Chip, FAMILIES, family, subtitle};
use model::{Architecture, Outcome};
use serde_json::Value;
use std::fmt::Write as _;

const CSS: &str = include_str!("../assets/page.css");
const JS: &str = include_str!("../assets/page.js");

/// The page for this architecture document and optional simulation result.
pub fn page(architecture: &Value, result: Option<&Value>) -> String {
    let arch = Architecture::read(architecture);
    let outcome = result.map(Outcome::read);
    let o = outcome.as_ref();
    let graph = graph::render(&arch, o);

    let mut h = String::with_capacity(64 * 1024);
    let title = match o {
        Some(_) => format!("{} — architecture and optimization", arch.name),
        None => format!("{} — architecture", arch.name),
    };
    let _ = write!(
        h,
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta name=\"color-scheme\" content=\"light dark\">\n<title>{}</title>\n<style>\n{CSS}</style>\n</head>\n<body>\n",
        esc(&title)
    );
    header(&mut h, &arch, o);
    h.push_str("<main>\n");
    architecture_section(&mut h, &arch, o, &graph);
    if let Some(o) = o {
        result::sections(&mut h, &arch, o);
    }
    h.push_str("</main>\n");
    let formats = [
        Some(if arch.format.is_empty() {
            "agenticspec-architecture/1"
        } else {
            arch.format.as_str()
        }),
        o.map(|_| "agenticspec-sim-result/1"),
    ];
    let _ = writeln!(
        h,
        "<footer>{} · rendered by agenticspec, self-contained, no network</footer>",
        formats
            .iter()
            .flatten()
            .map(|f| esc(f))
            .collect::<Vec<_>>()
            .join(" · ")
    );
    let data = serde_json::json!({ "architecture": architecture, "result": result });
    let json = serde_json::to_string(&data)
        .unwrap_or_else(|_| "{}".to_string())
        .replace('<', "\\u003c");
    let _ = write!(
        h,
        "<script id=\"agenticspec-data\" type=\"application/json\">{json}</script>\n<script>\n{JS}</script>\n</body>\n</html>\n"
    );
    h
}

fn header(h: &mut String, arch: &Architecture, o: Option<&Outcome>) {
    let eyebrow = if o.is_some() {
        "Architecture · optimization result"
    } else {
        "Architecture"
    };
    let _ = write!(
        h,
        "<header class=\"top\"><div class=\"title\"><div class=\"eyebrow\">{eyebrow}</div><h1>{}</h1>",
        esc(&arch.name)
    );
    if !arch.summary.is_empty() {
        let _ = write!(h, "<p class=\"summary\">{}</p>", esc(&arch.summary));
    }
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
    h.push_str("<ul class=\"facts\">");
    for f in facts {
        let _ = write!(h, "<li>{}</li>", esc(&f));
    }
    h.push_str("</ul></div>");
    h.push_str("<div class=\"segmented\" role=\"group\" aria-label=\"Theme\"><button type=\"button\" data-theme-choice=\"system\" aria-pressed=\"true\">System</button><button type=\"button\" data-theme-choice=\"light\" aria-pressed=\"false\">Light</button><button type=\"button\" data-theme-choice=\"dark\" aria-pressed=\"false\">Dark</button></div></header>\n");
}

fn legend(h: &mut String, arch: &Architecture) {
    h.push_str("<ul class=\"legend\" aria-label=\"Legend\">");
    for (fam, label) in FAMILIES {
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
            "<li><span class=\"swatch per-sample\"></span>one per owner, drawn inside it</li>",
        );
    }
    if arch.places().next().is_some() {
        h.push_str(
            "<li><span class=\"swatch chip-sample\"></span>marked place [search range]</li>",
        );
    }
    h.push_str("</ul>");
}

fn architecture_section(
    h: &mut String,
    arch: &Architecture,
    o: Option<&Outcome>,
    g: &graph::Graph,
) {
    h.push_str(
        "<section class=\"top-grid\" aria-label=\"Architecture\">\n<div class=\"card graph-card\">",
    );
    let _ = write!(
        h,
        "<div class=\"card-head\"><h2>Parts and links</h2><span class=\"hint\">Select a part for its properties{}</span></div><div class=\"card-body\">",
        if o.is_some() { " and KPIs" } else { "" }
    );
    legend(h, arch);
    // The graph is drawn at its natural size; the script shows the zoom tools when it is wider
    // than its card, and the card then pans by drag, wheel and arrow keys.
    h.push_str(
        "<div class=\"graph-tools\" hidden role=\"toolbar\" aria-label=\"Zoom\">\
<button type=\"button\" data-zoom=\"out\" aria-label=\"Zoom out\">−</button>\
<output class=\"zoom-level\" aria-live=\"polite\">100 %</output>\
<button type=\"button\" data-zoom=\"in\" aria-label=\"Zoom in\">+</button>\
<button type=\"button\" data-zoom=\"fit\" aria-label=\"Fit to width\">Fit</button>\
<button type=\"button\" data-zoom=\"actual\" aria-label=\"Actual size\">Actual size</button>\
<span class=\"hint\">drag, scroll or use the arrow keys to move around</span></div>",
    );
    let _ = write!(
        h,
        "<div class=\"graph-view\" style=\"--graph-w:{}px\">{}</div>",
        fmt::px(g.width),
        g.svg
    );
    h.push_str("</div></div>\n<aside class=\"side\">");
    parts_table(h, arch);
    details(h, arch, o);
    links_table(h, arch);
    policies(h, arch, o);
    h.push_str("</aside>\n</section>\n");
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

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
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
                h.push_str(&Chip::new(pl, o).html(true));
            }
            h.push_str("</div>");
        }
        let links: Vec<_> = arch
            .links
            .iter()
            .filter(|l| l.from == p.name || l.to == p.name || l.over.as_deref() == Some(&p.name))
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
            Chip::new(pl, o).html(false)
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
