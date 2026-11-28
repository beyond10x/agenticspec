//! The sections of a simulation result: figures, KPIs, marked places, the search trace, the
//! ranked variants and the timeline.

use super::map;
use super::model::{Architecture, Outcome, Run, norm};
use super::optim;
use crate::fmt::{self, esc, px};
use std::fmt::Write as _;

pub fn sections(h: &mut String, arch: &Architecture, o: &Outcome) {
    h.push_str("<section class=\"result\" aria-label=\"Optimization result\">\n");
    tiles(h, o);
    optim::objective(h, o);
    kpi_table(h, o);
    let sp = optim::Space::new(arch, o);
    h.push_str("<div class=\"two\">");
    optim::landscape(h, &sp);
    optim::tradeoffs(h, o, &sp);
    h.push_str("</div><div class=\"two\">");
    optim::sensitivity(h, &sp);
    places_table(h, arch, o);
    h.push_str("</div>");
    trace(h, o);
    variants(h, o, sp.best);
    timeline(h, o);
    h.push_str("</section>\n");
}

fn tiles(h: &mut String, o: &Outcome) {
    let mut tiles: Vec<(String, String, String, &str)> = Vec::new();
    let base_fit = o.baseline.as_ref().and_then(|b| b.fitness);
    let best_fit = o.best.as_ref().and_then(|b| b.fitness);
    if base_fit.is_some() || best_fit.is_some() {
        let (meta, tone) = match (base_fit, best_fit) {
            (Some(a), Some(b)) => (
                format!("{} from baseline {}", fmt::signed(b - a), fmt::num(a)),
                tone(b - a, Some(1.0)),
            ),
            (Some(a), None) => (format!("baseline {}", fmt::num(a)), "flat"),
            _ => (String::new(), "flat"),
        };
        tiles.push((
            "Fitness of the best variant".into(),
            best_fit.or(base_fit).map(fmt::num).unwrap_or_default(),
            meta,
            tone,
        ));
    }
    if let Some(f) = o.best.as_ref().and_then(|b| b.feasible) {
        tiles.push((
            "Best variant".into(),
            if f { "feasible" } else { "infeasible" }.into(),
            if f {
                "every gate KPI is 0"
            } else {
                "a gate KPI is not 0"
            }
            .into(),
            if f { "good" } else { "bad" },
        ));
    }
    if !o.trace.is_empty() {
        tiles.push((
            "Evaluations".into(),
            fmt::num(o.trace.len() as f64),
            "variants scored by the search".into(),
            "flat",
        ));
    }
    if !o.variants.is_empty() {
        let out = o
            .variants
            .iter()
            .filter(|v| v.feasible == Some(false))
            .count();
        tiles.push((
            "Distinct variants".into(),
            fmt::num(o.variants.len() as f64),
            format!("{out} infeasible"),
            "flat",
        ));
    }
    let changed = o
        .best
        .as_ref()
        .map(|b| {
            b.params
                .iter()
                .filter(|(k, v)| o.baseline.as_ref().and_then(|a| a.param(k)) != Some(v))
                .count()
        })
        .unwrap_or(0);
    if o.best.is_some() && o.baseline.is_some() {
        tiles.push((
            "Marked places changed".into(),
            fmt::num(changed as f64),
            format!(
                "of {}",
                o.best.as_ref().map(|b| b.params.len()).unwrap_or(0)
            ),
            "flat",
        ));
    }
    if tiles.is_empty() {
        return;
    }
    h.push_str("<div class=\"tiles\">");
    for (label, value, meta, tone) in tiles {
        let _ = write!(
            h,
            "<div class=\"tile card\"><div class=\"label\">{}</div><div class=\"value\">{}</div><div class=\"meta {tone}\">{}</div></div>",
            esc(&label),
            esc(&value),
            esc(&meta)
        );
    }
    h.push_str("</div>");
}

pub(crate) fn tone(diff: f64, direction: Option<f64>) -> &'static str {
    match direction {
        _ if diff == 0.0 => "flat",
        Some(d) if d * diff > 0.0 => "good",
        Some(d) if d * diff < 0.0 => "bad",
        _ => "changed",
    }
}

pub(crate) fn change(
    base: Option<f64>,
    best: Option<f64>,
    direction: Option<f64>,
) -> (String, &'static str) {
    match (base, best) {
        (Some(a), Some(b)) => {
            let d = b - a;
            if d == 0.0 {
                return ("no change".into(), "flat");
            }
            let pct = if a != 0.0 {
                format!(" ({}%)", fmt::signed((d / a.abs() * 100.0).round()))
            } else {
                String::new()
            };
            let text = format!(
                "{}{}",
                if d > 0.0 { "+" } else { "−" },
                fmt::compact(d.abs())
            );
            (format!("{text}{pct}"), tone(d, direction))
        }
        _ => ("–".into(), "flat"),
    }
}

fn kpi_names(runs: &[Option<&Run>]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for r in runs.iter().flatten() {
        for (k, _) in &r.kpis {
            if !names.contains(k) {
                names.push(k.clone());
            }
        }
    }
    names
}

fn kpi_table(h: &mut String, o: &Outcome) {
    let (base, best) = (o.baseline.as_ref(), o.best.as_ref());
    let mut names = kpi_names(&[base, best]);
    if names.is_empty() {
        return;
    }
    // Weighted KPIs in the order of the objective, then the gates, then the rest by name.
    let weighted: Vec<String> = optim::terms(o).into_iter().map(|t| t.name).collect();
    let rank = |n: &String| {
        if let Some(i) = weighted.iter().position(|w| w == n) {
            (0, i)
        } else if let Some(i) = o.gates.iter().position(|g| g == n) {
            (1, i)
        } else {
            (2, 0)
        }
    };
    names.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.cmp(b)));
    let roles = !o.weights.is_empty() || !o.gates.is_empty();
    let kinds = names.iter().any(|n| o.kind(n).is_some());
    let _ = write!(
        h,
        "<details class=\"card all-kpis\"{}><summary class=\"card-head\"><h2>All {} KPIs</h2><span class=\"hint\">{}baseline is the architecture as written</span></summary><div class=\"table-wrap\"><table class=\"data kpis\"><thead><tr><th scope=\"col\">KPI</th>",
        if o.weights.is_empty() { " open" } else { "" },
        names.len(),
        if roles {
            "weighted first, then the gates; "
        } else {
            ""
        }
    );
    if roles {
        h.push_str("<th scope=\"col\">role</th>");
    }
    if kinds {
        h.push_str("<th scope=\"col\">kind</th>");
    }
    h.push_str("<th scope=\"col\" class=\"r\">baseline</th><th scope=\"col\" class=\"r\">best</th><th scope=\"col\" class=\"r\">change</th></tr></thead><tbody>");
    for n in &names {
        let a = base.and_then(|r| r.kpi(n));
        let b = best.and_then(|r| r.kpi(n));
        let (text, tone) = change(a, b, o.direction(n));
        let _ = write!(
            h,
            "<tr data-kpi=\"{}\"><th scope=\"row\"><span class=\"kpi-name\">{}</span></th>",
            esc(n),
            esc(n)
        );
        if roles {
            let role = match (o.weight(n), o.gates.contains(n)) {
                (Some(w), _) => format!(
                    "<span class=\"role weighted\">weight {}</span>",
                    esc(&fmt::num(w))
                ),
                (None, true) => "<span class=\"role gate\">gate</span>".to_string(),
                _ => "<span class=\"muted\">reported</span>".to_string(),
            };
            let _ = write!(h, "<td>{role}</td>");
        }
        if kinds {
            let _ = write!(
                h,
                "<td><span class=\"kind\">{}</span></td>",
                esc(o.kind(n).unwrap_or(""))
            );
        }
        let _ = write!(
            h,
            "<td class=\"r muted\">{}</td><td class=\"r best\">{}</td><td class=\"r delta {tone}\">{}</td></tr>",
            fmt::opt_html(a),
            fmt::opt_html(b),
            esc(&text)
        );
    }
    h.push_str("</tbody></table></div></details>");
}

fn places_table(h: &mut String, arch: &Architecture, o: &Outcome) {
    let mut rows: Vec<crate::graph::Chip> = arch.places().map(|p| map::chip(p, Some(o))).collect();
    for m in &o.marked {
        if !rows
            .iter()
            .any(|c| norm(c.id.as_deref().unwrap_or("")) == norm(&m.path))
        {
            let place = super::model::Place {
                path: m.path.clone(),
                field: m.path.clone(),
                value: m.value.clone(),
                vary: m.range.clone(),
            };
            rows.push(map::chip(&place, Some(o)));
        }
    }
    if rows.is_empty() {
        return;
    }
    h.push_str("<div class=\"card\"><div class=\"card-head\"><h2>Marked places</h2><span class=\"hint\">the search space</span></div><div class=\"table-wrap\"><table class=\"data places\"><thead><tr><th scope=\"col\">place</th><th scope=\"col\">range</th><th scope=\"col\" class=\"r\">baseline</th><th scope=\"col\" class=\"r\">best</th></tr></thead><tbody>");
    for c in rows {
        let _ = write!(
            h,
            "<tr{}><th scope=\"row\"><span class=\"mono\">{}</span></th><td class=\"mono muted\">{}</td><td class=\"r mono\">{}</td><td class=\"r mono best\">{}</td></tr>",
            if c.changed() {
                " class=\"changed\""
            } else {
                ""
            },
            esc(&norm(c.id.as_deref().unwrap_or(""))),
            esc(&c.range),
            esc(c.value.as_deref().unwrap_or("")),
            esc(c.best.as_deref().unwrap_or("–"))
        );
    }
    h.push_str("</tbody></table></div></div>");
}

/// Ticks for an axis: about five round numbers covering `lo..hi`.
pub(crate) fn ticks(lo: f64, hi: f64) -> Vec<f64> {
    let span = (hi - lo).max(1e-9);
    let raw = span / 5.0;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| span / s <= 6.0)
        .unwrap_or(10.0 * mag);
    steps_from(lo, hi, step)
}

/// Ticks from the step at or below `lo` to the first step at or above `hi`.
fn steps_from(lo: f64, hi: f64, step: f64) -> Vec<f64> {
    let mut t = (lo / step).floor() * step;
    let mut out = vec![t];
    while t < hi - 1e-9 && out.len() < 14 {
        t += step;
        out.push(t);
    }
    out
}

/// Ticks for a time axis in seconds, at whole minutes or hours.
fn time_ticks(end: f64) -> Vec<f64> {
    let step = [
        60.0, 300.0, 900.0, 1800.0, 3600.0, 7200.0, 10800.0, 14400.0, 21600.0, 43200.0, 86400.0,
    ]
    .into_iter()
    .find(|s| end / s <= 8.0)
    .unwrap_or(86400.0 * ((end / 86400.0 / 8.0).ceil()));
    steps_from(0.0, end, step)
}

fn trace(h: &mut String, o: &Outcome) {
    let pts: Vec<(f64, Option<f64>, Option<&str>)> = o
        .trace
        .iter()
        .map(|t| (t.eval, t.fitness, t.variant.as_deref()))
        .collect();
    if pts.iter().all(|p| p.1.is_none()) && o.trace.iter().all(|t| t.best_so_far.is_none()) {
        return;
    }
    let mut best = Vec::new();
    let mut acc: Option<f64> = None;
    for t in &o.trace {
        let b = t.best_so_far.or_else(|| match (acc, t.fitness) {
            (Some(a), Some(f)) => Some(a.max(f)),
            (None, f) => f,
            (a, None) => a,
        });
        acc = b.or(acc);
        best.push(b);
    }
    let feasible = |id: Option<&str>| {
        id.and_then(|id| o.variants.iter().find(|v| v.id.as_deref() == Some(id)))
            .and_then(|v| v.feasible)
    };
    let ys: Vec<f64> = pts
        .iter()
        .filter_map(|p| p.1)
        .chain(best.iter().flatten().copied())
        .collect();
    let (mut lo, mut hi) = (
        ys.iter().copied().fold(f64::INFINITY, f64::min),
        ys.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    if lo == hi {
        lo -= 1.0;
        hi += 1.0;
    }
    let yt = ticks(lo, hi);
    let (lo, hi) = (lo.min(yt[0]), hi.max(*yt.last().unwrap_or(&hi)));
    let x0 = pts.first().map(|p| p.0).unwrap_or(0.0);
    let x1 = pts.last().map(|p| p.0).unwrap_or(1.0).max(x0 + 1.0);
    let (w, hh, ml, mr, mt, mb) = (1000.0, 260.0, 64.0, 18.0, 14.0, 34.0);
    let sx = |x: f64| ml + (x - x0) / (x1 - x0) * (w - ml - mr);
    let sy = |y: f64| mt + (hi - y) / (hi - lo) * (hh - mt - mb);
    let best_final = best.iter().flatten().last().copied();
    let _ = write!(
        h,
        "<div class=\"card section-gap\"><div class=\"card-head\"><h2>Search trace</h2><span class=\"hint\">fitness of each evaluated variant and the best so far</span></div><div class=\"card-body\"><ul class=\"legend\"><li><span class=\"mark dot\"></span>feasible variant</li><li><span class=\"mark cross\"></span>infeasible variant</li><li><span class=\"mark line\"></span>best so far</li></ul>"
    );
    let _ = write!(
        h,
        "<svg class=\"trace\" viewBox=\"0 0 {w} {hh}\" role=\"img\" aria-labelledby=\"trace-title\"><title id=\"trace-title\">Search trace: {} evaluations, fitness from {} to {}{}</title>",
        pts.len(),
        fmt::num(lo),
        fmt::num(hi),
        best_final
            .map(|b| format!(", best {}", fmt::num(b)))
            .unwrap_or_default()
    );
    for t in &yt {
        let y = sy(*t);
        let _ = write!(
            h,
            "<line class=\"gridline\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/><text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",
            px(ml),
            px(w - mr),
            px(y),
            px(y),
            px(ml - 8.0),
            px(y + 4.0),
            fmt::num(*t)
        );
    }
    for t in ticks(x0, x1) {
        if t < x0 - 1e-9 || t > x1 + 1e-9 {
            continue;
        }
        let x = sx(t);
        let _ = write!(
            h,
            "<line class=\"tick\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/><text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            px(x),
            px(x),
            px(hh - mb),
            px(hh - mb + 4.0),
            px(x),
            px(hh - mb + 18.0),
            fmt::num(t)
        );
    }
    let _ = write!(
        h,
        "<text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"end\">evaluation</text>",
        px(w - mr),
        px(hh - 2.0)
    );
    let mut d = String::new();
    for (i, b) in best.iter().enumerate() {
        let Some(b) = b else { continue };
        let (x, y) = (sx(pts[i].0), sy(*b));
        if d.is_empty() {
            let _ = write!(d, "M{} {}", px(x), px(y));
        } else {
            let _ = write!(d, "H{}V{}", px(x), px(y));
        }
    }
    let _ = write!(h, "<path class=\"best-line\" d=\"{d}\"/>");
    for (i, (x, f, id)) in pts.iter().enumerate() {
        let Some(f) = f else { continue };
        let (cx, cy) = (sx(*x), sy(*f));
        let tip = format!(
            "evaluation {}{}: fitness {}",
            fmt::num(*x),
            id.map(|v| format!(", {v}")).unwrap_or_default(),
            fmt::num(*f)
        );
        if feasible(*id) == Some(false) {
            let _ = write!(
                h,
                "<g class=\"pt out\" data-eval=\"{i}\"><title>{} (infeasible)</title><path d=\"M{} {}l7 7m0-7l-7 7\"/></g>",
                esc(&tip),
                px(cx - 3.5),
                px(cy - 3.5)
            );
        } else {
            let _ = write!(
                h,
                "<g class=\"pt\" data-eval=\"{i}\"><title>{}</title><circle cx=\"{}\" cy=\"{}\" r=\"3.5\"/></g>",
                esc(&tip),
                px(cx),
                px(cy)
            );
        }
    }
    h.push_str("</svg></div></div>");
}

fn short(path: &str) -> String {
    norm(path)
}

fn variants(h: &mut String, o: &Outcome, best: Option<&Run>) {
    if o.variants.is_empty() {
        return;
    }
    let is_best = |v: &Run| best.is_some_and(|b| std::ptr::eq(b, v));
    // Ties in fitness: the best variant of the result first, then by id, numerically when it is a number.
    let id_key = |v: &Run| {
        let id = v.id.clone().unwrap_or_default();
        (id.parse::<f64>().unwrap_or(f64::INFINITY), id)
    };
    let mut vs: Vec<&Run> = o.variants.iter().collect();
    vs.sort_by(|a, b| {
        let fa = a.fitness.unwrap_or(f64::NEG_INFINITY);
        let fb = b.fitness.unwrap_or(f64::NEG_INFINITY);
        let (ka, kb) = (id_key(a), id_key(b));
        fb.total_cmp(&fa)
            .then(is_best(b).cmp(&is_best(a)))
            .then(ka.0.total_cmp(&kb.0))
            .then(ka.1.cmp(&kb.1))
    });
    let best_id = vs
        .iter()
        .find(|v| v.feasible != Some(false))
        .and_then(|v| v.id.clone());
    let weighted: Vec<String> = optim::terms(o).into_iter().map(|t| t.name).collect();
    let kpis: Vec<String> = if weighted.is_empty() {
        kpi_names(&vs.iter().map(|v| Some(*v)).collect::<Vec<_>>())
            .into_iter()
            .take(3)
            .collect()
    } else {
        weighted
    };
    let shown = vs.len().min(20);
    let _ = write!(
        h,
        "<div class=\"card section-gap\"><div class=\"card-head\"><h2>Variants</h2><span class=\"hint\">top {shown} of {} by fitness; infeasible variants are out of the ranking</span></div><div class=\"table-wrap\"><table class=\"data variants\"><thead><tr><th scope=\"col\" class=\"r\">rank</th><th scope=\"col\">variant</th><th scope=\"col\" class=\"r\">fitness</th><th scope=\"col\">status</th><th scope=\"col\">changed from baseline</th>",
        vs.len()
    );
    for k in &kpis {
        let _ = write!(
            h,
            "<th scope=\"col\" class=\"r kpi-col\" data-col=\"{}\">{}</th>",
            esc(k),
            esc(&k.replace('_', " "))
        );
    }
    h.push_str("</tr></thead><tbody>");
    let mut rank = 0;
    for v in vs.iter().take(20) {
        let ok = v.feasible != Some(false);
        let id = v.id.clone().unwrap_or_default();
        let rank_text = if ok {
            rank += 1;
            rank.to_string()
        } else {
            "–".to_string()
        };
        let is_best = ok && v.id.is_some() && v.id == best_id;
        let status = match (ok, is_best) {
            (true, true) => "<span class=\"status good\"><span class=\"dot\"></span>best</span>",
            (true, false) => "<span class=\"status ok\"><span class=\"dot\"></span>feasible</span>",
            (false, _) => "<span class=\"status out\"><span class=\"dot\"></span>infeasible</span>",
        };
        let diffs: Vec<String> = v
            .params
            .iter()
            .filter(|(k, val)| o.baseline.as_ref().and_then(|b| b.param(k)) != Some(val))
            .map(|(k, val)| {
                format!(
                    "<span class=\"param\"><span class=\"muted\">{}</span> {}</span>",
                    esc(&short(k)),
                    esc(&fmt::value(val))
                )
            })
            .collect();
        let diffs = if diffs.is_empty() {
            "<span class=\"muted\">none</span>".to_string()
        } else {
            diffs.join("")
        };
        let _ = write!(
            h,
            "<tr data-variant=\"{}\" class=\"{}\"><td class=\"r\">{rank_text}</td><td class=\"mono\">{}</td><td class=\"r\">{}</td><td>{status}</td><td class=\"params\">{diffs}</td>",
            esc(&id),
            if ok { "ok" } else { "out" },
            esc(&id),
            v.fitness.map(fmt::num).unwrap_or_else(|| "–".into())
        );
        for k in &kpis {
            let _ = write!(h, "<td class=\"r\">{}</td>", fmt::opt_html(v.kpi(k)));
        }
        h.push_str("</tr>");
    }
    h.push_str("</tbody></table></div></div>");
}

fn timeline(h: &mut String, o: &Outcome) {
    if o.timeline.is_empty() {
        return;
    }
    let mut parts: Vec<&str> = Vec::new();
    let mut kinds: Vec<&str> = Vec::new();
    for e in &o.timeline {
        if !parts.contains(&e.part.as_str()) {
            parts.push(&e.part);
        }
        if !kinds.contains(&e.kind.as_str()) {
            kinds.push(&e.kind);
        }
    }
    let last = o.timeline.iter().map(|e| e.t).fold(0.0, f64::max).max(60.0);
    let axis = time_ticks(last);
    let end = axis.last().copied().unwrap_or(last).max(last);
    let lane = 26.0;
    let (w, ml, mr, mt) = (1000.0, 110.0, 18.0, 8.0);
    let hh = mt + parts.len() as f64 * lane + 30.0;
    let sx = |t: f64| ml + t / end * (w - ml - mr);
    h.push_str("<div class=\"card section-gap\"><div class=\"card-head\"><h2>Timeline</h2><span class=\"hint\">events of the simulated run by part</span></div><div class=\"card-body\"><ul class=\"legend\">");
    for (i, k) in kinds.iter().enumerate() {
        let _ = write!(
            h,
            "<li><span class=\"swatch s{}\"></span>{}</li>",
            i % 8 + 1,
            esc(k)
        );
    }
    let _ = write!(
        h,
        "</ul><svg class=\"timeline\" viewBox=\"0 0 {w} {}\" role=\"img\" aria-labelledby=\"timeline-title\"><title id=\"timeline-title\">Timeline: {} events over {}</title>",
        px(hh),
        o.timeline.len(),
        fmt::duration(end)
    );
    for (i, p) in parts.iter().enumerate() {
        let y = mt + i as f64 * lane + lane / 2.0;
        let _ = write!(
            h,
            "<text class=\"lane-t\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text><line class=\"gridline\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
            px(ml - 10.0),
            px(y + 4.0),
            esc(p),
            px(ml),
            px(w - mr),
            px(y),
            px(y)
        );
    }
    let base = mt + parts.len() as f64 * lane;
    for &t in &axis {
        if t > end + 1e-9 {
            continue;
        }
        let _ = write!(
            h,
            "<text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            px(sx(t)),
            px(base + 18.0),
            fmt::duration(t)
        );
    }
    for e in &o.timeline {
        let i = parts.iter().position(|p| *p == e.part).unwrap_or(0);
        let k = kinds.iter().position(|x| *x == e.kind).unwrap_or(0);
        let y = mt + i as f64 * lane + lane / 2.0;
        let _ = write!(
            h,
            "<g class=\"ev s{}\"><title>{} at {}: {}</title><rect x=\"{}\" y=\"{}\" width=\"6\" height=\"14\" rx=\"2\"/></g>",
            k % 8 + 1,
            esc(&e.part),
            fmt::duration(e.t),
            esc(&e.kind),
            px(sx(e.t) - 3.0),
            px(y - 7.0)
        );
    }
    h.push_str("</svg></div></div>");
}

/// The KPIs of one part at baseline and best, for its detail panel.
pub fn per_part(h: &mut String, o: &Outcome, part: &str) {
    let base = o
        .baseline
        .as_ref()
        .map(|b| b.part_figures(part))
        .unwrap_or(&[]);
    let best = o.best.as_ref().map(|b| b.part_figures(part)).unwrap_or(&[]);
    let mut names: Vec<&str> = Vec::new();
    for (k, _) in base.iter().chain(best.iter()) {
        if !names.contains(&k.as_str()) {
            names.push(k);
        }
    }
    if names.is_empty() {
        return;
    }
    let get = |xs: &[(String, f64)], k: &str| xs.iter().find(|(n, _)| n == k).map(|(_, v)| *v);
    let _ = write!(
        h,
        "<h4>KPIs</h4><table class=\"data mini\" data-per-part=\"{}\"><thead><tr><th scope=\"col\">KPI</th><th scope=\"col\" class=\"r\">baseline</th><th scope=\"col\" class=\"r\">best</th><th scope=\"col\" class=\"r\">change</th></tr></thead><tbody>",
        esc(part)
    );
    for n in names {
        let (a, b) = (get(base, n), get(best, n));
        let (text, tone) = change(a, b, None);
        let _ = write!(
            h,
            "<tr><th scope=\"row\">{}</th><td class=\"r muted\">{}</td><td class=\"r best\">{}</td><td class=\"r delta {tone}\">{}</td></tr>",
            esc(&n.replace('_', " ")),
            fmt::opt_html(a),
            fmt::opt_html(b),
            esc(&text)
        );
    }
    h.push_str("</tbody></table>");
}
