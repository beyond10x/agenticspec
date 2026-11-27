//! Documents: YAML into JSON values, durations and numbers inside them.

use serde_json::Value;
use std::path::Path;

pub fn parse_yaml(text: &str) -> Result<Value, String> {
    serde_yaml_ng::from_str(text).map_err(|e| e.to_string())
}

pub fn load_yaml(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_yaml(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// A duration written as `90s`, `15m`, `24h`, `1d` or `250ms`, in seconds.
pub fn parse_duration(text: &str) -> Option<f64> {
    let t = text.trim();
    let (digits, unit) = match t.find(|c: char| !(c.is_ascii_digit() || c == '.')) {
        Some(i) => t.split_at(i),
        None => (t, "s"),
    };
    let n: f64 = digits.parse().ok()?;
    let scale = match unit {
        "ms" => 0.001,
        "s" => 1.0,
        "m" => 60.0,
        "h" => 3600.0,
        "d" => 86400.0,
        _ => return None,
    };
    Some(n * scale)
}

/// Seconds written with the largest unit that divides them: `300` is `5m`.
pub fn format_duration(seconds: f64) -> String {
    let s = seconds.round().max(0.0) as u64;
    if s == 0 {
        return "0s".into();
    }
    for (unit, size) in [("d", 86400), ("h", 3600), ("m", 60)] {
        if s.is_multiple_of(size) {
            return format!("{}{unit}", s / size);
        }
    }
    format!("{s}s")
}

pub fn is_duration(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        parse_duration(s).is_some() && s.trim().ends_with(|c: char| c.is_ascii_alphabetic())
    })
}

/// A number, or a duration string in seconds.
pub fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => parse_duration(s),
        _ => None,
    }
}

/// A JSON number: an integer when the value is whole.
pub fn number(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 9.0e15 {
        Value::from(x as i64)
    } else {
        Value::from(x)
    }
}
