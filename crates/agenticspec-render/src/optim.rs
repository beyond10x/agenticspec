//! The views that explain an optimization: the objective and where the fitness change came from,
//! the landscape of the fitness over marked places, the trade-offs between weighted KPIs and the
//! sensitivity of the fitness to each place. Every view draws only variants the search evaluated.

use crate::fmt::{self, esc, px, text_w};
use crate::model::{Architecture, Axis, Outcome, Run, norm};
use crate::result::ticks;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A weighted KPI: its weight, baseline, best and its share of the fitness change.
pub struct Term {
    pub name: String,
    pub weight: f64,
    pub base: Option<f64>,
    pub best: Option<f64>,
    pub contribution: Option<f64>,
}

/// The weighted KPIs, the largest contribution to the fitness change first.
pub fn terms(o: &Outcome) -> Vec<Term> {
    let mut out: Vec<Term> = o
        .weights
        .iter()
        .map(|(k, w)| {
            let base = o.baseline.as_ref().and_then(|r| r.kpi(k));
            let best = o.best.as_ref().and_then(|r| r.kpi(k));
            Term {
                name: k.clone(),
                weight: *w,
                base,
                best,
                contribution: base.zip(best).map(|(a, b)| w * (b - a)),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        let size = |t: &Term| t.contribution.map(f64::abs).unwrap_or(-1.0);
        size(b).total_cmp(&size(a))
    });
    out
}

/// A KPI name as an attribute name and option value: `[a-z0-9_]` only.
fn key(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn weight_text(w: f64) -> String {
    fmt::num(w.abs())
}

/// A name as HTML that may wrap after a `.` or `_`.
fn breakable(name: &str) -> String {
    esc(name).replace('.', ".<wbr>").replace('_', "_<wbr>")
}

/// The objective: the formula, each weighted KPI with its contribution, and the gates.
pub fn objective(h: &mut String, o: &Outcome) {
    let terms = terms(o);
    if terms.is_empty() {
        return;
    }
    h.push_str("<section class=\"card objective\" aria-labelledby=\"objective-h\"><div class=\"card-head\"><h2 id=\"objective-h\">Objective</h2><span class=\"hint\">what the search maximised and where the fitness change came from</span></div><div class=\"card-body\"><p class=\"formula\"><span class=\"muted\">fitness =</span>");
    let mut by_weight: Vec<&(String, f64)> = o.weights.iter().collect();
    by_weight.sort_by(|a, b| {
        (a.1 < 0.0)
            .cmp(&(b.1 < 0.0))
            .then(b.1.abs().total_cmp(&a.1.abs()))
            .then(a.0.cmp(&b.0))
    });
    for (i, (k, w)) in by_weight.iter().enumerate() {
        let sign = match (i, *w < 0.0) {
            (0, false) => "",
            (0, true) => "− ",
            (_, false) => "+ ",
            (_, true) => "− ",
        };
        let _ = write!(
            h,
            " <span class=\"term\">{sign}{} × {}</span>",
            esc(&weight_text(*w)),
            esc(k)
        );
    }
    if !o.gates.is_empty() {
        h.push_str(" <span class=\"muted\">· feasible only when every gate KPI is 0</span>");
    }
    h.push_str("</p>");
    let max = terms
        .iter()
        .filter_map(|t| t.contribution)
        .fold(0.0f64, |m, c| m.max(c.abs()));
    h.push_str("<div class=\"table-wrap\"><table class=\"data terms\"><thead><tr><th scope=\"col\">weighted KPI</th><th scope=\"col\" class=\"r\">weight</th><th scope=\"col\" class=\"r\">baseline</th><th scope=\"col\" class=\"r\">best</th><th scope=\"col\" class=\"r\">change</th><th scope=\"col\" class=\"bar-h\">contribution to the fitness change</th><th scope=\"col\" class=\"r\">points</th></tr></thead><tbody>");
    for t in &terms {
        let (text, tone) = crate::result::change(t.base, t.best, Some(t.weight.signum()));
        let _ = write!(
            h,
            "<tr data-term=\"{}\" data-contribution=\"{}\"><th scope=\"row\"><span class=\"kpi-name\">{}</span></th><td class=\"r mono\">{}</td><td class=\"r muted\">{}</td><td class=\"r best\">{}</td><td class=\"r delta {tone}\">{}</td><td class=\"bar-cell\">",
            esc(&t.name),
            t.contribution.unwrap_or(0.0),
            esc(&t.name),
            esc(&fmt::num(t.weight)),
            fmt::opt_html(t.base),
            fmt::opt_html(t.best),
            esc(&text)
        );
        bar(h, t.contribution.unwrap_or(0.0), max);
        let c = t.contribution.unwrap_or(0.0);
        let _ = write!(
            h,
            "</td><td class=\"r contrib {}\">{}</td></tr>",
            if c > 0.0 {
                "pos"
            } else if c < 0.0 {
                "neg"
            } else {
                "zero"
            },
            if t.contribution.is_some() {
                esc(&fmt::signed(round2(c)))
            } else {
                "–".into()
            }
        );
    }
    let fb = o.baseline.as_ref().and_then(|r| r.fitness);
    let ft = o.best.as_ref().and_then(|r| r.fitness);
    if let (Some(a), Some(b)) = (fb, ft) {
        let _ = write!(
            h,
            "<tr class=\"total\"><th scope=\"row\">fitness</th><td></td><td class=\"r muted\">{}</td><td class=\"r best\">{}</td><td class=\"r delta {}\">{}</td><td class=\"bar-cell\"></td><td class=\"r contrib\">{}</td></tr>",
            fmt::num_html(round2(a)),
            fmt::num_html(round2(b)),
            crate::result::tone(b - a, Some(1.0)),
            esc(&fmt::signed(round2(b - a))),
            esc(&fmt::signed(round2(b - a)))
        );
    }
    h.push_str("</tbody></table></div>");
    gates(h, o);
    h.push_str("</div></section>\n");
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// A diverging bar around 0: right and blue raises the fitness, left and orange lowers it.
fn bar(h: &mut String, c: f64, max: f64) {
    let (w, half) = (200.0, 96.0);
    let len = if max > 0.0 { c.abs() / max * half } else { 0.0 };
    let len = if c != 0.0 { len.max(1.5) } else { 0.0 };
    let x = if c >= 0.0 { w / 2.0 } else { w / 2.0 - len };
    let _ = write!(
        h,
        "<svg class=\"contrib-bar\" viewBox=\"0 0 {w} 16\" width=\"{w}\" height=\"16\" aria-hidden=\"true\" focusable=\"false\"><line class=\"zero\" x1=\"100\" x2=\"100\" y1=\"0\" y2=\"16\"/>"
    );
    if len > 0.0 {
        let _ = write!(
            h,
            "<rect class=\"{}\" x=\"{}\" y=\"3\" width=\"{}\" height=\"10\" rx=\"2\"/>",
            if c > 0.0 { "pos" } else { "neg" },
            px(x),
            px(len)
        );
    }
    h.push_str("</svg>");
}

fn gates(h: &mut String, o: &Outcome) {
    if o.gates.is_empty() {
        return;
    }
    let best = o.best.as_ref().or(o.baseline.as_ref());
    let zero = o
        .gates
        .iter()
        .filter(|g| best.and_then(|r| r.kpi(g)).unwrap_or(0.0) == 0.0)
        .count();
    let n = o.gates.len();
    let ok = zero == n;
    let _ = write!(
        h,
        "<details class=\"gates\"><summary><span class=\"status {}\"><span class=\"dot\"></span>{zero} of {n} gate KPIs at 0</span> <span class=\"muted\">for the best variant; a variant with a gate KPI above 0 is infeasible</span></summary><ul class=\"gate-list\">",
        if ok { "good" } else { "out" }
    );
    for g in &o.gates {
        let base = o.baseline.as_ref().and_then(|r| r.kpi(g));
        let b = best.and_then(|r| r.kpi(g));
        let _ = write!(
            h,
            "<li><span class=\"kpi-name\">{}</span><span class=\"muted\">baseline {}</span><span class=\"{}\">best {}</span></li>",
            esc(g),
            fmt::opt_html(base),
            if b.unwrap_or(0.0) == 0.0 { "" } else { "bad" },
            fmt::opt_html(b)
        );
    }
    h.push_str("</ul></details>");
}

/// The evaluated variants indexed by their values at the marked places, and the point every
/// slice holds the other places at: the best variant, else the baseline.
pub struct Space<'a> {
    pub axes: Vec<Axis>,
    anchor: Vec<usize>,
    index: BTreeMap<Vec<String>, &'a Run>,
    keys: Vec<Vec<String>>,
    pub best: Option<&'a Run>,
    pub base: Option<&'a Run>,
    pub exhaustive: bool,
}

fn same(a: Option<&Run>, b: Option<&Run>) -> bool {
    matches!((a, b), (Some(a), Some(b)) if std::ptr::eq(a, b))
}

impl<'a> Space<'a> {
    pub fn new(arch: &Architecture, o: &'a Outcome) -> Self {
        let axes = o.axes(arch);
        let keys: Vec<Vec<String>> = axes
            .iter()
            .map(|a| a.values.iter().map(fmt::value).collect())
            .collect();
        let mut index = BTreeMap::new();
        for v in &o.variants {
            let k: Option<Vec<String>> = axes
                .iter()
                .map(|a| v.param(&a.path).map(fmt::value))
                .collect();
            if let Some(k) = k {
                index.entry(k).or_insert(v);
            }
        }
        let find = |r: Option<&Run>| -> Option<&'a Run> {
            let r = r?;
            let k: Option<Vec<String>> = axes
                .iter()
                .map(|a| r.param(&a.path).map(fmt::value))
                .collect();
            index.get(&k?).copied()
        };
        let best = find(o.best.as_ref()).or_else(|| {
            o.variants
                .iter()
                .filter(|v| v.feasible != Some(false) && v.fitness.is_some())
                .max_by(|a, b| {
                    a.fitness
                        .unwrap_or(0.0)
                        .total_cmp(&b.fitness.unwrap_or(0.0))
                })
        });
        let base = find(o.baseline.as_ref());
        let anchor_run = o.best.as_ref().or(o.baseline.as_ref());
        let anchor = axes
            .iter()
            .zip(&keys)
            .map(|(a, ks)| {
                let v = anchor_run
                    .and_then(|r| r.param(&a.path))
                    .or_else(|| o.baseline.as_ref().and_then(|r| r.param(&a.path)))
                    .or_else(|| {
                        o.marked
                            .iter()
                            .find(|m| norm(&m.path) == norm(&a.path))
                            .map(|m| &m.value)
                    });
                v.and_then(|v| ks.iter().position(|k| *k == fmt::value(v)))
                    .unwrap_or(0)
            })
            .collect();
        let exhaustive = o.mode.as_deref() == Some("exhaustive");
        Space {
            axes,
            anchor,
            index,
            keys,
            best,
            base,
            exhaustive,
        }
    }

    /// The variant at the anchor with the given places set to the given value indices.
    fn at(&self, set: &[(usize, usize)]) -> Option<&'a Run> {
        let mut k: Vec<String> = self
            .anchor
            .iter()
            .enumerate()
            .map(|(i, v)| self.keys[i][*v].clone())
            .collect();
        for (i, v) in set {
            k[*i] = self.keys[*i][*v].clone();
        }
        self.index.get(&k).copied()
    }

    fn name(&self, i: usize) -> String {
        norm(&self.axes[i].path)
    }

    fn value(&self, i: usize, k: usize) -> &str {
        &self.keys[i][k]
    }

    /// The other places and the values they are held at, as text.
    fn held(&self, except: &[usize]) -> String {
        let parts: Vec<String> = (0..self.axes.len())
            .filter(|i| !except.contains(i))
            .map(|i| format!("{} {}", self.name(i), self.value(i, self.anchor[i])))
            .collect();
        parts.join(", ")
    }
}

/// What a cell of the landscape holds.
#[derive(Clone, Copy)]
enum Cell<'a> {
    Fit(&'a Run, f64),
    Out(&'a Run),
    None,
}

fn cell(r: Option<&Run>) -> Cell<'_> {
    match r {
        Some(r) if r.feasible == Some(false) => Cell::Out(r),
        Some(r) => match r.fitness {
            Some(f) => Cell::Fit(r, f),
            None => Cell::None,
        },
        None => Cell::None,
    }
}

/// Seven bins of a sequential scale.
fn bin(f: f64, lo: f64, hi: f64) -> usize {
    if hi <= lo {
        return 3;
    }
    (((f - lo) / (hi - lo)) * 7.0).floor().clamp(0.0, 6.0) as usize
}

/// A fitness that fits in `room` pixels: one decimal when it fits, else whole.
fn fit_text(f: f64, room: f64) -> String {
    if f.abs() >= 10_000.0 {
        return fmt::compact(f);
    }
    let one = fmt::num((f * 10.0).round() / 10.0);
    if text_w(&one, 12.0, false) <= room {
        one
    } else {
        fmt::num(f.round())
    }
}

fn state_text(c: Cell<'_>) -> String {
    match c {
        Cell::Fit(r, f) => format!(
            "fitness {}{}",
            fmt::num(round2(f)),
            r.id.as_ref()
                .map(|id| format!(" (variant {id})"))
                .unwrap_or_default()
        ),
        Cell::Out(r) => format!(
            "infeasible: a gate KPI is above 0{}",
            r.fitness
                .map(|f| format!(" (fitness {})", fmt::num(round2(f))))
                .unwrap_or_default()
        ),
        Cell::None => "not evaluated".to_string(),
    }
}

/// The landscape: a heatmap of the fitness over two places, a line for one, a choice of pair for
/// more than two.
pub fn landscape(h: &mut String, sp: &Space<'_>) {
    let n = sp.axes.len();
    if n == 0 || sp.index.is_empty() {
        return;
    }
    h.push_str("<section class=\"card landscape\" aria-labelledby=\"landscape-h\"><div class=\"card-head\"><h2 id=\"landscape-h\">Landscape</h2><span class=\"hint\">fitness over the marked places, the others at the best variant's values</span></div><div class=\"card-body\">");
    if n == 1 {
        let (lo, hi) = fitness_range(sp, &[0]);
        h.push_str("<figure class=\"pair\">");
        let _ = write!(
            h,
            "<figcaption class=\"muted\">Fitness across <span class=\"mono\">{}</span>.</figcaption>",
            esc(&sp.name(0))
        );
        sens_chart(h, sp, 0, lo, hi, "land", true);
        coverage_note(h, sp, &[0]);
        h.push_str("</figure></div></section>\n");
        return;
    }
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in i + 1..n {
            pairs.push((i, j));
        }
    }
    let score = |&(i, j): &(usize, usize)| {
        let mut count = 0usize;
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for a in 0..sp.keys[i].len() {
            for b in 0..sp.keys[j].len() {
                match cell(sp.at(&[(i, a), (j, b)])) {
                    Cell::Fit(_, f) => {
                        count += 1;
                        lo = lo.min(f);
                        hi = hi.max(f);
                    }
                    Cell::Out(_) => count += 1,
                    Cell::None => {}
                }
            }
        }
        (count, if hi > lo { hi - lo } else { 0.0 })
    };
    let mut default = 0;
    let mut top = (0usize, -1.0f64);
    for (k, p) in pairs.iter().enumerate() {
        let s = score(p);
        if s.0 > top.0 || (s.0 == top.0 && s.1 > top.1) {
            top = s;
            default = k;
        }
    }
    if pairs.len() > 1 {
        h.push_str("<div class=\"picks\"><label>Places <select data-pick=\"pair\">");
        for (k, (i, j)) in pairs.iter().enumerate() {
            let _ = write!(
                h,
                "<option value=\"{i}-{j}\"{}>{} × {}</option>",
                if k == default { " selected" } else { "" },
                esc(&sp.name(*i)),
                esc(&sp.name(*j))
            );
        }
        h.push_str("</select></label></div>");
    }
    for (k, &(i, j)) in pairs.iter().enumerate() {
        let (cx, cy) = if sp.keys[j].len() > sp.keys[i].len() {
            (j, i)
        } else {
            (i, j)
        };
        let _ = write!(
            h,
            "<figure class=\"pair\" data-pair=\"{i}-{j}\"{}>",
            if k == default { "" } else { " hidden" }
        );
        heatmap(h, sp, cx, cy, &format!("hm-{i}-{j}"));
        coverage_note(h, sp, &[cx, cy]);
        h.push_str("</figure>");
    }
    h.push_str("</div></section>\n");
}

fn coverage_note(h: &mut String, sp: &Space<'_>, on: &[usize]) {
    let total: usize = on.iter().map(|i| sp.keys[*i].len()).product();
    let mut done = 0;
    let mut idx = vec![0usize; on.len()];
    for _ in 0..total {
        let set: Vec<(usize, usize)> = on.iter().copied().zip(idx.iter().copied()).collect();
        if sp.at(&set).is_some() {
            done += 1;
        }
        for (d, i) in idx.iter_mut().enumerate() {
            *i += 1;
            if *i < sp.keys[on[d]].len() {
                break;
            }
            *i = 0;
        }
    }
    let held = sp.held(on);
    let held = if held.is_empty() {
        String::new()
    } else {
        format!(" Held: {held}.")
    };
    if done == total {
        let _ = write!(
            h,
            "<p class=\"note muted\">Every combination was evaluated.{}</p>",
            esc(&held)
        );
    } else {
        let _ = write!(
            h,
            "<p class=\"note muted\">{done} of {total} {} evaluated. {} A cell marked – was not evaluated: nothing is interpolated.{}</p>",
            if on.len() == 1 { "values" } else { "cells" },
            if sp.exhaustive {
                "The search did not reach the others."
            } else {
                "A fit search runs only the variants on its climb, so the slice through the best variant is partly empty."
            },
            esc(&held)
        );
    }
}

fn heatmap(h: &mut String, sp: &Space<'_>, cx: usize, cy: usize, uid: &str) {
    let (cols, rows) = (sp.keys[cx].len(), sp.keys[cy].len());
    let lbl_w = (0..rows)
        .map(|b| text_w(sp.value(cy, b), 12.5, false))
        .fold(0.0, f64::max);
    let ml = (lbl_w + 16.0).max(40.0);
    let cw = ((620.0 - ml) / cols as f64).clamp(44.0, 140.0);
    let ch = if rows > 8 { 28.0 } else { 34.0 };
    let mt = 26.0;
    let grid_w = cols as f64 * cw;
    let x_title = sp.name(cx);
    let y_title = sp.name(cy);
    let title_w = text_w(&x_title, 12.5, false).max(text_w(&y_title, 12.5, false));
    let w = (ml + grid_w + 4.0).max(title_w + 8.0);
    let hh = mt + rows as f64 * ch + 46.0;
    let mut cells: Vec<Vec<Cell<'_>>> = Vec::new();
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut done = 0;
    let mut top: Option<(f64, usize, usize)> = None;
    for b in 0..rows {
        let mut row = Vec::new();
        for a in 0..cols {
            let c = cell(sp.at(&[(cx, a), (cy, b)]));
            if let Cell::Fit(_, f) = c {
                lo = lo.min(f);
                hi = hi.max(f);
                if top.is_none_or(|t| f > t.0) {
                    top = Some((f, a, b));
                }
            }
            if !matches!(c, Cell::None) {
                done += 1;
            }
            row.push(c);
        }
        cells.push(row);
    }
    let top_text = top
        .map(|(f, a, b)| {
            format!(
                ", highest {} at {} {}, {} {}",
                fmt::num(round2(f)),
                x_title,
                sp.value(cx, a),
                y_title,
                sp.value(cy, b)
            )
        })
        .unwrap_or_default();
    let _ = write!(
        h,
        "<figcaption class=\"muted\">Columns <span class=\"mono\">{}</span>, rows <span class=\"mono\">{}</span>.</figcaption><svg class=\"heatmap\" viewBox=\"0 0 {} {}\" role=\"img\" aria-labelledby=\"{uid}-t\" style=\"max-width:{}px\"><title id=\"{uid}-t\">Fitness over {} (columns) and {} (rows), the other places at the best variant's values: {done} of {} cells evaluated{}.</title>",
        esc(&x_title),
        esc(&y_title),
        px(w),
        px(hh),
        px(w),
        esc(&x_title),
        esc(&y_title),
        rows * cols,
        esc(&top_text)
    );
    let _ = write!(
        h,
        "<text class=\"axis-name\" x=\"0\" y=\"14\">{}</text>",
        esc(&y_title)
    );
    for (b, row) in cells.iter().enumerate() {
        let y = mt + b as f64 * ch;
        let _ = write!(
            h,
            "<text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",
            px(ml - 8.0),
            px(y + ch / 2.0 + 4.0),
            esc(sp.value(cy, b))
        );
        for (a, c) in row.iter().enumerate() {
            let x = ml + a as f64 * cw;
            let tip = format!(
                "{} {}, {} {}: {}",
                x_title,
                sp.value(cx, a),
                y_title,
                sp.value(cy, b),
                state_text(*c)
            );
            let (class, text, state, run) = match c {
                Cell::Fit(r, f) => {
                    let q = bin(*f, lo, hi);
                    (format!("q{q}"), fit_text(*f, cw - 8.0), "fit", Some(*r))
                }
                Cell::Out(r) => ("out".to_string(), "×".to_string(), "out", Some(*r)),
                Cell::None => ("none".to_string(), "–".to_string(), "none", None),
            };
            let _ = write!(
                h,
                "<g><title>{}</title><rect class=\"cell {class}\" data-cell=\"{state}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\"/>",
                esc(&tip),
                px(x + 1.0),
                px(y + 1.0),
                px(cw - 2.0),
                px(ch - 2.0)
            );
            if same(run, sp.base) {
                let _ = write!(
                    h,
                    "<rect class=\"base-ring\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\"/>",
                    px(x + 3.0),
                    px(y + 3.0),
                    px(cw - 6.0),
                    px(ch - 6.0)
                );
            }
            if same(run, sp.best) {
                let _ = write!(
                    h,
                    "<rect class=\"best-ring\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\"/>",
                    px(x + 1.0),
                    px(y + 1.0),
                    px(cw - 2.0),
                    px(ch - 2.0)
                );
            }
            let _ = write!(
                h,
                "<text class=\"cell-t {class}{}\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text></g>",
                if same(run, sp.best) { " top" } else { "" },
                px(x + cw / 2.0),
                px(y + ch / 2.0 + 4.0),
                esc(&text)
            );
        }
    }
    let base_y = mt + rows as f64 * ch;
    for a in 0..cols {
        let _ = write!(
            h,
            "<text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            px(ml + a as f64 * cw + cw / 2.0),
            px(base_y + 16.0),
            esc(sp.value(cx, a))
        );
    }
    let tw = text_w(&x_title, 12.5, false);
    let center = (ml + grid_w / 2.0).clamp(tw / 2.0 + 2.0, w - tw / 2.0 - 2.0);
    let _ = write!(
        h,
        "<text class=\"axis-name\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text></svg>",
        px(center),
        px(base_y + 38.0),
        esc(&x_title)
    );
    h.push_str("<ul class=\"legend scale\">");
    if lo.is_finite() {
        let _ = write!(
            h,
            "<li><span class=\"muted\">fitness</span> {}<span class=\"ramp\" aria-hidden=\"true\">",
            fmt::num_html(round2(lo))
        );
        for q in 0..7 {
            let _ = write!(h, "<span class=\"q q{q}\"></span>");
        }
        let _ = write!(h, "</span>{}</li>", fmt::num_html(round2(hi)));
    }
    h.push_str("<li><span class=\"swatch ring-best\"></span>best variant</li>");
    if sp.base.is_some() {
        h.push_str("<li><span class=\"swatch ring-base\"></span>baseline</li>");
    }
    let has = |want: fn(&Cell<'_>) -> bool| cells.iter().flatten().any(want);
    if has(|c| matches!(c, Cell::Out(_))) {
        h.push_str("<li><span class=\"swatch cell-out\"></span>× infeasible</li>");
    }
    if has(|c| matches!(c, Cell::None)) {
        h.push_str("<li><span class=\"swatch cell-none\"></span>– not evaluated</li>");
    }
    h.push_str("</ul>");
    let _ = write!(
        h,
        "<details class=\"as-table\"><summary>Table</summary><div class=\"table-wrap\"><table class=\"data mini\"><thead><tr><th scope=\"col\">{} \\ {}</th>",
        esc(&y_title),
        esc(&x_title)
    );
    for a in 0..cols {
        let _ = write!(
            h,
            "<th scope=\"col\" class=\"r\">{}</th>",
            esc(sp.value(cx, a))
        );
    }
    h.push_str("</tr></thead><tbody>");
    for (b, row) in cells.iter().enumerate() {
        let _ = write!(h, "<tr><th scope=\"row\">{}</th>", esc(sp.value(cy, b)));
        for c in row {
            let t = match c {
                Cell::Fit(_, f) => fmt::num(round2(*f)),
                Cell::Out(_) => "infeasible".to_string(),
                Cell::None => "not evaluated".to_string(),
            };
            let _ = write!(h, "<td class=\"r\">{}</td>", esc(&t));
        }
        h.push_str("</tr>");
    }
    h.push_str("</tbody></table></div></details>");
}

/// The lowest and highest fitness over the slices through each of `on`, infeasible included.
fn fitness_range(sp: &Space<'_>, on: &[usize]) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &i in on {
        for k in 0..sp.keys[i].len() {
            if let Some(f) = sp.at(&[(i, k)]).and_then(|r| r.fitness) {
                lo = lo.min(f);
                hi = hi.max(f);
            }
        }
    }
    (lo, hi)
}

/// Fitness across the values of one place, the others at the best variant's values; a value
/// with no evaluated variant is a gap.
fn sens_chart(h: &mut String, sp: &Space<'_>, i: usize, lo: f64, hi: f64, uid: &str, big: bool) {
    let (lo, hi) = if !lo.is_finite() {
        (0.0, 1.0)
    } else if hi - lo < 1e-9 {
        (lo - 1.0, hi + 1.0)
    } else {
        (lo, hi)
    };
    let yt = ticks(lo, hi);
    let (lo, hi) = (lo.min(yt[0]), hi.max(*yt.last().unwrap_or(&hi)));
    let labels: Vec<String> = yt.iter().map(|t| fmt::compact(*t)).collect();
    let ml = labels
        .iter()
        .map(|l| text_w(l, 12.5, false))
        .fold(0.0, f64::max)
        + 14.0;
    let (w, hh) = if big { (640.0, 240.0) } else { (320.0, 170.0) };
    let (mt, mr, mb) = (10.0, 12.0, 30.0);
    let n = sp.keys[i].len();
    let slot = (w - ml - mr) / n as f64;
    let sx = |k: usize| ml + (k as f64 + 0.5) * slot;
    let sy = |y: f64| mt + (hi - y) / (hi - lo) * (hh - mt - mb);
    let pts: Vec<Cell<'_>> = (0..n).map(|k| cell(sp.at(&[(i, k)]))).collect();
    let fits: Vec<(usize, f64)> = pts
        .iter()
        .enumerate()
        .filter_map(|(k, c)| match c {
            Cell::Fit(_, f) => Some((k, *f)),
            _ => None,
        })
        .collect();
    let summary = match (
        fits.iter().min_by(|a, b| a.1.total_cmp(&b.1)),
        fits.iter().max_by(|a, b| a.1.total_cmp(&b.1)),
    ) {
        (Some(a), Some(b)) => format!(
            ": lowest {} at {}, highest {} at {}",
            fmt::num(round2(a.1)),
            sp.value(i, a.0),
            fmt::num(round2(b.1)),
            sp.value(i, b.0)
        ),
        _ => String::new(),
    };
    let done = pts.iter().filter(|c| !matches!(c, Cell::None)).count();
    let _ = write!(
        h,
        "<svg class=\"sens{}\" viewBox=\"0 0 {w} {hh}\" role=\"img\" aria-labelledby=\"{uid}-t\"><title id=\"{uid}-t\">Fitness across {}, the other places at the best variant's values, {done} of {n} values evaluated{}.</title>",
        if big { " line" } else { "" },
        esc(&sp.name(i)),
        esc(&summary)
    );
    for (t, l) in yt.iter().zip(&labels) {
        let y = sy(*t);
        let _ = write!(
            h,
            "<line class=\"gridline\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/><text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",
            px(ml),
            px(w - mr),
            px(y),
            px(y),
            px(ml - 6.0),
            px(y + 4.0),
            esc(l)
        );
    }
    let every = ((text_w("00000", 12.0, false) / slot).ceil() as usize).max(1);
    for (k, c) in pts.iter().enumerate() {
        let gap = matches!(c, Cell::None);
        if gap {
            let _ = write!(
                h,
                "<g data-gap=\"{k}\"><title>{} {}: not evaluated</title><line class=\"gap\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/></g>",
                esc(&sp.name(i)),
                esc(sp.value(i, k)),
                px(sx(k)),
                px(sx(k)),
                px(mt),
                px(hh - mb)
            );
        }
        let label = sp.value(i, k);
        if k % every == 0 || text_w(label, 12.0, false) < slot - 2.0 {
            let _ = write!(
                h,
                "<text class=\"axis-t{}\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
                if gap { " gap-t" } else { "" },
                px(sx(k)),
                px(hh - mb + 16.0),
                esc(label)
            );
        }
    }
    let mut d = String::new();
    let mut prev: Option<usize> = None;
    for (k, f) in &fits {
        let cmd = if prev == Some(k.wrapping_sub(1)) {
            "L"
        } else {
            "M"
        };
        let _ = write!(d, "{cmd}{} {}", px(sx(*k)), px(sy(*f)));
        prev = Some(*k);
    }
    if !d.is_empty() {
        let _ = write!(h, "<path class=\"sens-line\" d=\"{d}\"/>");
    }
    for (k, c) in pts.iter().enumerate() {
        let tip = format!("{} {}: {}", sp.name(i), sp.value(i, k), state_text(*c));
        match c {
            Cell::Fit(r, f) => {
                let best = same(Some(*r), sp.best);
                let _ = write!(
                    h,
                    "<g class=\"sens-pt{}\"><title>{}</title>{}<circle cx=\"{}\" cy=\"{}\" r=\"{}\"/></g>",
                    if best { " best" } else { "" },
                    esc(&tip),
                    if best {
                        format!(
                            "<circle class=\"ring\" cx=\"{}\" cy=\"{}\" r=\"7.5\"/>",
                            px(sx(k)),
                            px(sy(*f))
                        )
                    } else {
                        String::new()
                    },
                    px(sx(k)),
                    px(sy(*f)),
                    if best { 4.5 } else { 3.5 }
                );
            }
            Cell::Out(r) => {
                let y = sy(r.fitness.unwrap_or(lo).clamp(lo, hi));
                let _ = write!(
                    h,
                    "<g class=\"sens-pt out\"><title>{}</title><path d=\"M{} {}l7 7m0-7l-7 7\"/></g>",
                    esc(&tip),
                    px(sx(k) - 3.5),
                    px(y - 3.5)
                );
            }
            Cell::None => {}
        }
    }
    h.push_str("</svg>");
}

/// Sensitivity: one small chart per marked place on a shared fitness scale.
pub fn sensitivity(h: &mut String, sp: &Space<'_>) {
    let n = sp.axes.len();
    if n == 0 || sp.index.is_empty() {
        return;
    }
    let all: Vec<usize> = (0..n).collect();
    let (lo, hi) = fitness_range(sp, &all);
    if !lo.is_finite() {
        return;
    }
    h.push_str("<section class=\"card sensitivity\" aria-labelledby=\"sensitivity-h\"><div class=\"card-head\"><h2 id=\"sensitivity-h\">Sensitivity</h2><span class=\"hint\">fitness across each place, the others at the best variant's values; one scale</span></div><div class=\"card-body\"><ul class=\"legend\"><li><span class=\"mark dot\"></span>evaluated</li><li><span class=\"swatch ring-best round\"></span>best variant</li><li><span class=\"mark cross\"></span>infeasible</li><li><span class=\"mark gap\"></span>not evaluated</li></ul><div class=\"sens-grid\">");
    let mut rows: Vec<(usize, usize, Cell<'_>)> = Vec::new();
    for i in 0..n {
        let (a, b) = fitness_range(sp, &[i]);
        let spread = if b > a { b - a } else { 0.0 };
        let _ = write!(
            h,
            "<figure class=\"sens-fig\"><figcaption><span class=\"mono\">{}</span><span class=\"muted\">spread {}</span></figcaption>",
            breakable(&sp.name(i)),
            fmt::num_html(round2(spread))
        );
        sens_chart(h, sp, i, lo, hi, &format!("sens-{i}"), false);
        h.push_str("</figure>");
        for k in 0..sp.keys[i].len() {
            rows.push((i, k, cell(sp.at(&[(i, k)]))));
        }
    }
    h.push_str("</div><details class=\"as-table\"><summary>Table</summary><div class=\"table-wrap\"><table class=\"data mini\"><thead><tr><th scope=\"col\">place</th><th scope=\"col\">value</th><th scope=\"col\" class=\"r\">fitness</th></tr></thead><tbody>");
    for (i, k, c) in rows {
        let t = match c {
            Cell::Fit(_, f) => fmt::num(round2(f)),
            Cell::Out(_) => "infeasible".into(),
            Cell::None => "not evaluated".into(),
        };
        let _ = write!(
            h,
            "<tr><th scope=\"row\" class=\"mono\">{}</th><td class=\"mono\">{}</td><td class=\"r\">{}</td></tr>",
            esc(&sp.name(i)),
            esc(sp.value(i, k)),
            esc(&t)
        );
    }
    h.push_str("</tbody></table></div></details></div></section>\n");
}

/// One axis of the trade-off scatter.
struct Scale {
    name: String,
    lo: f64,
    hi: f64,
    ticks: Vec<f64>,
    dir: f64,
}

impl Scale {
    fn new(name: &str, vals: &[f64], dir: f64) -> Self {
        let mut lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
        let mut hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if hi - lo < 1e-9 {
            lo -= 1.0;
            hi += 1.0;
        }
        let pad = (hi - lo) * 0.04;
        let t = ticks(lo - pad, hi + pad);
        let t: Vec<f64> = t
            .into_iter()
            .filter(|v| *v >= lo - pad - 1e-9 && *v <= hi + pad + 1e-9)
            .collect();
        Scale {
            name: name.to_string(),
            lo: lo - pad,
            hi: hi + pad,
            ticks: t,
            dir,
        }
    }

    fn at(&self, v: f64, a: f64, b: f64) -> f64 {
        a + (v - self.lo) / (self.hi - self.lo) * (b - a)
    }

    fn title(&self) -> String {
        format!(
            "{}{}",
            self.name,
            if self.dir > 0.0 {
                " · higher is better"
            } else if self.dir < 0.0 {
                " · lower is better"
            } else {
                ""
            }
        )
    }
}

const SW: f64 = 640.0;
const SH: f64 = 380.0;

/// Trade-offs: every evaluated variant on two weighted KPIs with the Pareto front of each pair.
pub fn tradeoffs(h: &mut String, o: &Outcome, sp: &Space<'_>) {
    let names: Vec<String> = terms(o)
        .into_iter()
        .map(|t| t.name)
        .filter(|k| o.variants.iter().filter(|v| v.kpi(k).is_some()).count() >= 2)
        .collect();
    if names.len() < 2 || o.variants.len() < 2 {
        return;
    }
    let pick = |want: &str, other: Option<&str>| {
        names
            .iter()
            .find(|n| *n == want)
            .or_else(|| names.iter().find(|n| Some(n.as_str()) != other))
            .cloned()
            .unwrap_or_default()
    };
    let dx = pick("cost_usd", Some("tasks_correct"));
    let dy = pick("tasks_correct", Some(&dx));
    let dy = if dy == dx {
        names
            .iter()
            .find(|n| **n != dx)
            .cloned()
            .unwrap_or_default()
    } else {
        dy
    };
    let scales: Vec<Scale> = names
        .iter()
        .map(|k| {
            let vals: Vec<f64> = o.variants.iter().filter_map(|v| v.kpi(k)).collect();
            Scale::new(k, &vals, o.direction(k).unwrap_or(0.0))
        })
        .collect();
    let label_w = scales
        .iter()
        .flat_map(|s| {
            s.ticks
                .iter()
                .map(|t| text_w(&fmt::compact(*t), 12.5, false))
        })
        .fold(0.0, f64::max);
    let (ml, mr, mt, mb) = (label_w + 14.0, 16.0, 26.0, 44.0);
    let (x0, x1, y0, y1) = (ml, SW - mr, SH - mb, mt);
    let sc = |k: &str| scales.iter().find(|s| s.name == k);
    let id = |v: &Run| v.id.clone().unwrap_or_default();
    // The Pareto front of each ordered pair, among feasible variants.
    let mut fronts: Vec<(String, String, Vec<&Run>)> = Vec::new();
    for a in &names {
        for b in &names {
            if a == b {
                continue;
            }
            let (da, db) = (o.direction(a).unwrap_or(1.0), o.direction(b).unwrap_or(1.0));
            let pts: Vec<(&Run, f64, f64)> = o
                .variants
                .iter()
                .filter(|v| v.feasible != Some(false))
                .filter_map(|v| Some((v, v.kpi(a)? * da, v.kpi(b)? * db)))
                .collect();
            let mut front: Vec<&Run> = pts
                .iter()
                .filter(|(_, x, y)| {
                    !pts.iter()
                        .any(|(_, x2, y2)| x2 >= x && y2 >= y && (x2 > x || y2 > y))
                })
                .map(|(v, _, _)| *v)
                .collect();
            front.sort_by(|p, q| {
                p.kpi(a)
                    .unwrap_or(0.0)
                    .total_cmp(&q.kpi(a).unwrap_or(0.0))
                    .then(id(p).cmp(&id(q)))
            });
            fronts.push((a.clone(), b.clone(), front));
        }
    }
    let default_front: Vec<String> = fronts
        .iter()
        .find(|(a, b, _)| *a == dx && *b == dy)
        .map(|(_, _, f)| f.iter().map(|v| id(v)).collect())
        .unwrap_or_default();
    let best_id = sp.best.map(id);
    h.push_str("<section class=\"card tradeoffs\" aria-labelledby=\"tradeoffs-h\"><div class=\"card-head\"><h2 id=\"tradeoffs-h\">Trade-offs</h2><span class=\"hint\">every evaluated variant on two weighted KPIs</span></div><div class=\"card-body\"><div class=\"picks\">");
    for (axis, current, label) in [("x", &dx, "Across"), ("y", &dy, "Up")] {
        let _ = write!(h, "<label>{label} <select data-axis=\"{axis}\">");
        for n in &names {
            let _ = write!(
                h,
                "<option value=\"{}\"{}>{}</option>",
                esc(&key(n)),
                if n == current { " selected" } else { "" },
                esc(n)
            );
        }
        h.push_str("</select></label>");
    }
    h.push_str("</div><ul class=\"legend\"><li><span class=\"mark dot front-dot\"></span>Pareto front</li><li><span class=\"mark dot other-dot\"></span>other feasible variant</li>");
    if o.variants.iter().any(|v| v.feasible == Some(false)) {
        h.push_str("<li><span class=\"mark cross\"></span>infeasible</li>");
    }
    if sp.best.is_some() {
        h.push_str("<li><span class=\"swatch ring-best round\"></span>best by fitness</li>");
    }
    if sp.base.is_some() {
        h.push_str("<li><span class=\"mark square\"></span>baseline</li>");
    }
    h.push_str("</ul>");
    let label_for = |a: &str, b: &str, front: &[String]| {
        format!(
            "Trade-offs between {a} (across) and {b} (up) over {} evaluated variants; Pareto front: {}{}.",
            o.variants.len(),
            if front.is_empty() {
                "none".to_string()
            } else {
                format!("variants {}", front.join(", "))
            },
            best_id
                .as_ref()
                .map(|b| format!("; best by fitness: variant {b}"))
                .unwrap_or_default()
        )
    };
    let _ = write!(
        h,
        "<svg class=\"scatter\" viewBox=\"0 0 {SW} {SH}\" role=\"img\" aria-labelledby=\"to-t\"><title id=\"to-t\">{}</title>",
        esc(&label_for(&dx, &dy, &default_front))
    );
    for s in &scales {
        let off = |want: &str| if s.name == want { "" } else { " off" };
        let _ = write!(
            h,
            "<g class=\"ax ax-x{}\" data-kpi=\"{}\">",
            off(&dx),
            esc(&key(&s.name))
        );
        for t in &s.ticks {
            let x = s.at(*t, x0, x1);
            let _ = write!(
                h,
                "<line class=\"gridline\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/><text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
                px(x),
                px(x),
                px(y1),
                px(y0),
                px(x.clamp(20.0, SW - 20.0)),
                px(y0 + 16.0),
                esc(&fmt::compact(*t))
            );
        }
        let _ = write!(
            h,
            "<text class=\"axis-name\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{} →</text></g>",
            px(x1),
            px(SH - 6.0),
            esc(&s.title())
        );
        let _ = write!(
            h,
            "<g class=\"ax ax-y{}\" data-kpi=\"{}\">",
            off(&dy),
            esc(&key(&s.name))
        );
        for t in &s.ticks {
            let y = s.at(*t, y0, y1);
            let _ = write!(
                h,
                "<line class=\"gridline\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/><text class=\"axis-t\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",
                px(x0),
                px(x1),
                px(y),
                px(y),
                px(x0 - 6.0),
                px(y + 4.0),
                esc(&fmt::compact(*t))
            );
        }
        let _ = write!(
            h,
            "<text class=\"axis-name\" x=\"{}\" y=\"14\">↑ {}</text></g>",
            px(x0),
            esc(&s.title())
        );
    }
    for (a, b, front) in &fronts {
        let (Some(sa), Some(sb)) = (sc(a), sc(b)) else {
            continue;
        };
        let mut d = String::new();
        for (k, v) in front.iter().enumerate() {
            let (x, y) = (
                sa.at(v.kpi(a).unwrap_or(0.0), x0, x1),
                sb.at(v.kpi(b).unwrap_or(0.0), y0, y1),
            );
            let _ = write!(d, "{}{} {}", if k == 0 { "M" } else { "L" }, px(x), px(y));
        }
        let ids: Vec<String> = front.iter().map(|v| id(v)).collect();
        let _ = write!(
            h,
            "<path class=\"front{}\" data-pair=\"{}|{}\" data-front=\"{}\" data-label=\"{}\" d=\"{}\"/>",
            if *a == dx && *b == dy { "" } else { " off" },
            esc(&key(a)),
            esc(&key(b)),
            esc(&ids.join(" ")),
            esc(&label_for(a, b, &ids)),
            d
        );
    }
    let mut order: Vec<&Run> = o.variants.iter().collect();
    // Best and baseline last, so they are drawn on top.
    order.sort_by_key(|v| {
        (
            same(Some(*v), sp.best) as u8 * 2 + same(Some(*v), sp.base) as u8,
            v.feasible != Some(false),
        )
    });
    for v in order {
        let vid = id(v);
        let ok = v.feasible != Some(false);
        let is_best = same(Some(v), sp.best);
        let is_base = same(Some(v), sp.base);
        let on_front = default_front.contains(&vid);
        let pos = |k: &str, a: f64, b: f64| Some(sc(k)?.at(v.kpi(k)?, a, b));
        let (tx, ty) = (pos(&dx, x0, x1), pos(&dy, y0, y1));
        let mut class = String::from("sp");
        if !ok {
            class.push_str(" out");
        }
        if on_front {
            class.push_str(" front-pt");
        }
        if is_best {
            class.push_str(" best");
        }
        if is_base {
            class.push_str(" base");
        }
        if tx.is_none() || ty.is_none() {
            class.push_str(" off");
        }
        let mut data = String::new();
        for k in &names {
            if let Some(x) = pos(k, x0, x1) {
                let _ = write!(data, " data-x-{}=\"{}\"", key(k), px(x));
            }
            if let Some(y) = pos(k, y0, y1) {
                let _ = write!(data, " data-y-{}=\"{}\"", key(k), px(y));
            }
        }
        let mut tip = format!("variant {vid}");
        if let Some(f) = v.fitness {
            let _ = write!(tip, ", fitness {}", fmt::num(round2(f)));
        }
        if !ok {
            tip.push_str(", infeasible");
        }
        if is_best {
            tip.push_str(", best");
        }
        if is_base {
            tip.push_str(", baseline");
        }
        for k in &names {
            if let Some(x) = v.kpi(k) {
                let _ = write!(tip, "; {k} {}", fmt::compact(round2(x)));
            }
        }
        let _ = write!(
            h,
            "<g class=\"{class}\" data-v=\"{}\" transform=\"translate({},{})\"{data}><title>{}</title>",
            esc(&vid),
            px(tx.unwrap_or(0.0)),
            px(ty.unwrap_or(0.0)),
            esc(&tip)
        );
        if is_best {
            h.push_str("<circle class=\"ring\" r=\"8\"/>");
        }
        if !ok {
            h.push_str("<path d=\"M-3.5 -3.5l7 7m0-7l-7 7\"/>");
        } else if is_base {
            h.push_str("<rect x=\"-4.5\" y=\"-4.5\" width=\"9\" height=\"9\"/>");
        } else {
            h.push_str("<circle r=\"4\"/>");
        }
        h.push_str("</g>");
    }
    h.push_str("</svg><p class=\"note muted\">The Pareto front joins the feasible variants that no other feasible variant beats on both KPIs at once. The best variant by fitness need not lie on the front of every pair: the fitness weighs all KPIs.</p></div></section>\n");
}
