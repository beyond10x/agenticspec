//! One generation of the pages per test run, not one per test: a cache on disk under the target's
//! test directory, keyed by the digest of the size and modification time of the generator's inputs
//! and of the test executable (which holds the generator and the simulator), and by
//! `ess --version`; one file per catalogue. The first
//! test process that needs an entry builds it under a file lock; the others wait and read it.
//! cargo-nextest runs each test in its own process, so an in-process cache would not do.

use agenticspec_docs::{Catalogue, Generated, generate};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What the generator reads under the repository root.
const INPUT_DIRS: &[&str] = &[
    "blocks",
    "architectures",
    "experiments",
    "kpis",
    "fields",
    "packs",
    "ladder",
    "crates/agenticspec-sim/src",
];
const INPUT_FILES: &[&str] = &[
    "ess-inputs.yaml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "docs/ladder.mdx",
];
/// The committed results of the catalog, read from `website/static/catalog/<entry>/`.
const RESULT_FILES: &[&str] = &["result.json", "sources.sha256"];

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// `<size> <modification time in ns>\n` of a file.
fn stamp(path: &Path) -> String {
    let meta = std::fs::metadata(path).expect("an input file");
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    format!("{} {mtime}\n", meta.len())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The digest of everything a generation of `root` depends on, except the catalogue: the size and
/// modification time of each file, as cargo decides what to build again; reading the files would
/// cost each test process more than the pages it reads.
fn inputs_key(root: &Path) -> String {
    let mut h = Sha256::new();
    let exe = std::env::current_exe().expect("the test executable");
    h.update(stamp(&exe));
    let ess = Command::new("ess")
        .arg("--version")
        .output()
        .expect("ess runs");
    h.update(&ess.stdout);
    let mut files = Vec::new();
    for d in INPUT_DIRS {
        walk(&root.join(d), &mut files);
    }
    for f in INPUT_FILES {
        let p = root.join(f);
        if p.exists() {
            files.push(p);
        }
    }
    let mut results = Vec::new();
    walk(&root.join("website/static/catalog"), &mut results);
    files.extend(results.into_iter().filter(|p| {
        p.file_name()
            .is_some_and(|n| RESULT_FILES.iter().any(|r| n == *r))
    }));
    files.sort();
    for f in files {
        let rel = f.strip_prefix(root).unwrap_or(&f).to_string_lossy();
        h.update(format!("{rel} "));
        h.update(stamp(&f));
    }
    hex(&h.finalize())[..32].to_string()
}

/// The pages of the repository at `root` with `catalogue` (its text; `None` reads
/// `kpis/catalogue.yaml`, as [`generate`] does), from the cache or generated into it.
pub fn generated(root: &Path, catalogue: Option<&str>) -> Generated {
    if catalogue.is_none()
        && let Ok(own) = std::fs::read_to_string(root.join("kpis/catalogue.yaml"))
    {
        return generated(root, Some(&own));
    }
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("docs-generated");
    let key = inputs_key(root);
    let dir = base.join(&key);
    std::fs::create_dir_all(&dir).expect("the cache directory");
    let label = match catalogue {
        Some(text) => hex(&Sha256::digest(text.as_bytes()))[..32].to_string(),
        None => "no-catalogue".to_string(),
    };
    let entry = dir.join(format!("{label}.pages"));
    let lock = File::create(dir.join(format!("{label}.lock"))).expect("the lock file");
    lock.lock().expect("the lock");
    if let Ok(bytes) = std::fs::read(&entry) {
        return decode(&bytes);
    }
    let parsed = catalogue.map(|t| Catalogue::parse(t).expect("the catalogue parses"));
    let g = generate(root, parsed).expect("the pages generate");
    let partial = dir.join(format!("{label}.pages.{}", std::process::id()));
    std::fs::write(&partial, encode(&g)).expect("the cache entry");
    std::fs::rename(&partial, &entry).expect("the cache entry");
    // Entries of other inputs are stale: a test process of this run never reads them.
    for d in std::fs::read_dir(&base).into_iter().flatten().flatten() {
        if d.file_name() != key.as_str() {
            let _ = std::fs::remove_dir_all(d.path());
        }
    }
    g
}

/// An entry: the IR as JSON, then each file as its path and its text, every part after a line with
/// its length in bytes. The pages are about 12 MB; as one JSON document they took a test process
/// half a second to parse.
fn encode(g: &Generated) -> Vec<u8> {
    let mut out = Vec::new();
    let mut part = |bytes: &[u8]| {
        out.extend_from_slice(format!("{}\n", bytes.len()).as_bytes());
        out.extend_from_slice(bytes);
    };
    part(serde_json::to_string(&g.ir).expect("JSON").as_bytes());
    for (path, text) in &g.files {
        part(path.as_bytes());
        part(text.as_bytes());
    }
    out
}

fn decode(bytes: &[u8]) -> Generated {
    let mut rest = bytes;
    let mut part = || -> Option<String> {
        if rest.is_empty() {
            return None;
        }
        let nl = rest.iter().position(|b| *b == b'\n').expect("a length");
        let len: usize = std::str::from_utf8(&rest[..nl])
            .ok()
            .and_then(|s| s.parse().ok())
            .expect("a length");
        let (text, tail) = rest[nl + 1..].split_at(len);
        rest = tail;
        Some(String::from_utf8(text.to_vec()).expect("UTF-8"))
    };
    let ir = serde_json::from_str(&part().expect("the IR")).expect("the IR");
    let mut files = BTreeMap::new();
    while let Some(path) = part() {
        files.insert(path, part().expect("the text of a file"));
    }
    Generated { ir, files }
}
