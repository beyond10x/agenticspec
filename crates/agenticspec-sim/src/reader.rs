//! A typed reader over a parsed YAML document: each value is read at its path (`when.0.at`), an
//! unknown key is refused by its path, and every problem of a document is collected, not only
//! the first.

use crate::doc::{is_duration, num};
use serde_json::{Map, Value};

/// A reference to another document: its path, relative to the referring file, and the sha256 of
/// its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRef {
    pub reference: String,
    pub digest: String,
}

#[derive(Debug, Default)]
pub struct Reader {
    pub errors: Vec<String>,
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

impl Reader {
    pub fn err(&mut self, path: &str, what: impl AsRef<str>) {
        let name = if path.is_empty() { "document" } else { path };
        self.errors.push(format!("{name}: {}", what.as_ref()));
    }

    /// The mapping at `path` with only `keys`, each of `required` present.
    pub fn obj<'v>(
        &mut self,
        v: &'v Value,
        path: &str,
        keys: &[&str],
        required: &[&str],
    ) -> Option<&'v Map<String, Value>> {
        let Some(m) = v.as_object() else {
            self.err(path, "expected a mapping");
            return None;
        };
        for k in m.keys() {
            if !keys.contains(&k.as_str()) {
                self.err(
                    &join(path, k),
                    format!("unknown key; expected one of {}", keys.join(", ")),
                );
            }
        }
        for k in required {
            if !m.contains_key(*k) {
                self.err(&join(path, k), "missing");
            }
        }
        Some(m)
    }

    pub fn text(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<String> {
        let v = m.get(key)?;
        match v {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => {
                self.err(&join(path, key), "expected a non-empty string");
                None
            }
        }
    }

    pub fn duration(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<f64> {
        let v = m.get(key)?;
        match num(v).filter(|s| *s >= 0.0) {
            Some(s) if v.is_number() || is_duration(v) => Some(s),
            _ => {
                self.err(
                    &join(path, key),
                    "expected a duration such as 90s, 15m or 2h",
                );
                None
            }
        }
    }

    pub fn uint(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<u64> {
        let v = m.get(key)?;
        let n = v.as_u64();
        if n.is_none() {
            self.err(&join(path, key), "expected a whole number of at least 0");
        }
        n
    }

    pub fn number(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<f64> {
        let v = m.get(key)?;
        let n = v.as_f64();
        if n.is_none() {
            self.err(&join(path, key), "expected a number");
        }
        n
    }

    pub fn boolean(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<bool> {
        let v = m.get(key)?;
        let b = v.as_bool();
        if b.is_none() {
            self.err(&join(path, key), "expected true or false");
        }
        b
    }

    /// The list at `key`; absent or null is empty.
    pub fn list<'v>(&mut self, m: &'v Map<String, Value>, key: &str, path: &str) -> &'v [Value] {
        match m.get(key) {
            None | Some(Value::Null) => &[],
            Some(Value::Array(xs)) => xs,
            Some(_) => {
                self.err(&join(path, key), "expected a list");
                &[]
            }
        }
    }

    /// A list of non-empty strings.
    pub fn texts(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<Vec<String>> {
        m.get(key)?;
        let mut out = Vec::new();
        for (i, v) in self.list(m, key, path).iter().enumerate() {
            match v.as_str() {
                Some(s) if !s.is_empty() => out.push(s.to_string()),
                _ => self.err(
                    &format!("{}.{i}", join(path, key)),
                    "expected a non-empty string",
                ),
            }
        }
        Some(out)
    }

    /// One of the names of `table`.
    pub fn one_of<T: Copy>(
        &mut self,
        m: &Map<String, Value>,
        key: &str,
        path: &str,
        table: &[(&str, T)],
    ) -> Option<T> {
        let v = m.get(key)?;
        let found = v
            .as_str()
            .and_then(|s| table.iter().find(|(n, _)| *n == s).map(|(_, t)| *t));
        if found.is_none() {
            let names: Vec<&str> = table.iter().map(|(n, _)| *n).collect();
            self.err(
                &join(path, key),
                format!("expected one of {}", names.join(", ")),
            );
        }
        found
    }

    /// A sha256 digest: 64 lowercase hexadecimal characters.
    pub fn digest(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<String> {
        let s = self.text(m, key, path)?;
        if s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            Some(s)
        } else {
            self.err(
                &join(path, key),
                "expected a sha256 digest, 64 lowercase hexadecimal characters",
            );
            None
        }
    }

    /// `{ ref, digest }` at `key`.
    pub fn doc_ref(&mut self, m: &Map<String, Value>, key: &str, path: &str) -> Option<DocRef> {
        let v = m.get(key)?;
        let p = join(path, key);
        let o = self.obj(v, &p, &["ref", "digest"], &["ref", "digest"])?;
        Some(DocRef {
            reference: self.text(o, "ref", &p)?,
            digest: self.digest(o, "digest", &p)?,
        })
    }

    /// The format line: `format` must be `want`.
    pub fn format(&mut self, m: &Map<String, Value>, want: &str) {
        match m.get("format").and_then(Value::as_str) {
            Some(f) if f == want => {}
            Some(f) => self.err("format", format!("`{f}`, expected {want}")),
            None => self.err("format", format!("missing; expected {want}")),
        }
    }
}

/// The sha256 of `bytes` as 64 lowercase hexadecimal characters.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
