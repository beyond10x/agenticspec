//! Escaping, number formatting and text widths for layout.

use serde_json::Value;

/// Text for an HTML text node or a quoted attribute.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// A number as the page shows it: integers whole (grouped from 10,000), others with at most two
/// decimals (three below 1).
pub fn num(n: f64) -> String {
    if !n.is_finite() {
        return "–".to_string();
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        let i = n as i64;
        return if i.abs() >= 10_000 {
            group(i)
        } else {
            i.to_string()
        };
    }
    let a = n.abs();
    let decimals = if a >= 1000.0 {
        1
    } else if a >= 1.0 {
        2
    } else {
        3
    };
    let s = format!("{n:.decimals$}");
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s == "-0" { "0".to_string() } else { s }
}

fn group(i: i64) -> String {
    let digits = i.unsigned_abs().to_string();
    let mut out = String::new();
    for (k, c) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if i < 0 { format!("-{out}") } else { out }
}

/// A large number in short form: `1.72 B`, `57.2 M`, `383.9 k`; below 10,000 as [`num`].
pub fn compact(n: f64) -> String {
    let a = n.abs();
    if !n.is_finite() || a < 10_000.0 {
        return num(n);
    }
    let (v, unit) = if a >= 1e9 {
        (n / 1e9, "B")
    } else if a >= 1e6 {
        (n / 1e6, "M")
    } else {
        (n / 1e3, "k")
    };
    let decimals = if v.abs() < 10.0 { 2 } else { 1 };
    let s = format!("{v:.decimals$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    format!("{s} {unit}")
}

/// A number as HTML: the compact form, with the exact value in a title when they differ.
pub fn num_html(n: f64) -> String {
    let (short, exact) = (compact(n), num(n));
    if short == exact {
        esc(&short)
    } else {
        format!("<span title=\"{}\">{}</span>", esc(&exact), esc(&short))
    }
}

/// An optional number as HTML, `–` when absent.
pub fn opt_html(n: Option<f64>) -> String {
    n.map(num_html).unwrap_or_else(|| "–".to_string())
}

/// A signed difference: `+47`, `−0.19`.
pub fn signed(n: f64) -> String {
    if n > 0.0 {
        format!("+{}", num(n))
    } else if n < 0.0 {
        format!("−{}", num(-n))
    } else {
        "0".to_string()
    }
}

/// Any JSON value as short text.
pub fn value(v: &Value) -> String {
    match v {
        Value::Null => "–".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.as_f64().map(num).unwrap_or_else(|| n.to_string()),
        Value::String(s) => s.clone(),
        Value::Array(xs) => xs.iter().map(value).collect::<Vec<_>>().join(", "),
        Value::Object(_) => v.to_string(),
    }
}

/// Estimated width in pixels of `s` in the page's sans-serif face.
pub fn text_w(s: &str, size: f64, bold: bool) -> f64 {
    let em: f64 = s
        .chars()
        .map(|c| match c {
            'i' | 'j' | 'l' | '.' | ',' | '\'' | ':' | ';' | '|' | '!' | ' ' | 'I' => 0.30,
            'f' | 't' | 'r' | '(' | ')' | '[' | ']' | '-' => 0.40,
            'm' | 'w' | 'M' | 'W' | '…' | '—' => 0.88,
            '_' => 0.55,
            'A'..='Z' => 0.70,
            '0'..='9' => 0.60,
            _ => 0.58,
        })
        .sum();
    em * size * if bold { 1.07 } else { 1.0 }
}

/// Width in pixels of `s` in the monospace face.
pub fn mono_w(s: &str, size: f64) -> f64 {
    s.chars().count() as f64 * 0.62 * size
}

/// A coordinate with at most one decimal.
pub fn px(n: f64) -> String {
    let r = (n * 10.0).round() / 10.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r:.1}")
    }
}

/// Seconds as `2h 30m`, `45m`, `30s`.
pub fn duration(s: f64) -> String {
    let s = s.max(0.0).round() as i64;
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    match (h, m, sec) {
        (0, 0, sec) => format!("{sec}s"),
        (0, m, _) => format!("{m}m"),
        (h, 0, _) => format!("{h}h"),
        (h, m, _) => format!("{h}h {m}m"),
    }
}
