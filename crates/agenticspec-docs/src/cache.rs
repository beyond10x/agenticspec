//! A disk cache of the catalog's experiment results, so a full search runs once per build of the
//! program and per text of its inputs, not once per generation: the tests generate the pages many
//! times, and each generation would otherwise run every experiment at its full budget.
//!
//! The key is a SHA-256 of the program that runs the search (path, size and modification time of
//! the running executable, which links the simulator) and the texts of the experiment, its
//! architecture and its profile. A run is deterministic for those, so a hit is the same result a
//! run would give. The files live beside the executable, under `target/`, in `catalog-cache/`.
//! One process computes a key while the others wait for its file (a lock file per key).

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Raised when the cached form or what produces it changes meaning.
const VERSION: &str = "agenticspec-docs catalog cache 1";

/// How long a lock may be held before another process takes the work over.
const STALE: Duration = Duration::from_secs(300);

fn exe_identity() -> String {
    let Ok(exe) = std::env::current_exe() else {
        return String::new();
    };
    let meta = std::fs::metadata(&exe).ok();
    let len = meta.as_ref().map_or(0, std::fs::Metadata::len);
    let modified = meta
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    format!("{}\n{len}\n{modified}", exe.display())
}

fn dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join("catalog-cache"))
}

/// The key of a result from these input texts, run by this executable.
pub fn key(texts: &[&str]) -> String {
    let mut h = Sha256::new();
    h.update(VERSION.as_bytes());
    h.update([0]);
    h.update(exe_identity().as_bytes());
    for t in texts {
        h.update([0]);
        h.update((t.len() as u64).to_le_bytes());
        h.update(t.as_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The value with every float as `{"$f64": "<bits in hex>"}`: serde_json reads a float back to
/// the nearest value only within a unit in the last place, and a cached result must be the bytes
/// a run gives.
fn encode(v: &Value) -> Value {
    match v {
        Value::Number(n) if n.is_f64() => {
            let bits = n.as_f64().unwrap_or(0.0).to_bits();
            serde_json::json!({ "$f64": format!("{bits:016x}") })
        }
        Value::Array(xs) => Value::Array(xs.iter().map(encode).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, x)| (k.clone(), encode(x))).collect()),
        other => other.clone(),
    }
}

fn decode(v: Value) -> Option<Value> {
    match v {
        Value::Object(o) if o.len() == 1 && o.contains_key("$f64") => {
            let bits = u64::from_str_radix(o["$f64"].as_str()?, 16).ok()?;
            serde_json::Number::from_f64(f64::from_bits(bits)).map(Value::Number)
        }
        Value::Array(xs) => xs
            .into_iter()
            .map(decode)
            .collect::<Option<Vec<_>>>()
            .map(Value::Array),
        Value::Object(o) => o
            .into_iter()
            .map(|(k, x)| decode(x).map(|x| (k, x)))
            .collect::<Option<serde_json::Map<_, _>>>()
            .map(Value::Object),
        other => Some(other),
    }
}

/// The cached value of `key`, or `compute`'s, which is then cached. Without a cache directory,
/// `compute` runs every time.
pub fn get_or(key: &str, compute: impl FnOnce() -> Result<Value, String>) -> Result<Value, String> {
    let Some(dir) = dir() else {
        return compute();
    };
    if std::fs::create_dir_all(&dir).is_err() {
        return compute();
    }
    let file = dir.join(format!("{key}.json"));
    let lock = dir.join(format!("{key}.lock"));
    loop {
        if let Some(v) = std::fs::read(&file)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(decode)
        {
            return Ok(v);
        }
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(_) => break,
            Err(_) => {
                let stale = std::fs::metadata(&lock)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|age| age > STALE);
                if stale {
                    let _ = std::fs::remove_file(&lock);
                } else {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
    let out = compute();
    if let Ok(v) = &out {
        let tmp = dir.join(format!("{key}.{}.tmp", std::process::id()));
        if std::fs::write(&tmp, serde_json::to_vec(&encode(v)).unwrap_or_default()).is_ok() {
            let _ = std::fs::rename(&tmp, &file);
        }
    }
    let _ = std::fs::remove_file(&lock);
    out
}
