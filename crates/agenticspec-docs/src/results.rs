//! The committed results of the catalog's experiments: `website/static/catalog/<entry>/result.json`
//! and beside it `sources.sha256`, the SHA-256 of every source the result depends on (the
//! experiment, its architecture and profile, the simulator's source files, the versions of the
//! simulator's dependencies in `Cargo.lock`, and `rust-toolchain.toml`).
//!
//! `task catalog:generate` runs every experiment at its own budget and seeds and writes both files;
//! `task slow:catalog` runs them again and compares bytes. The docs generator never runs a full
//! experiment: it reads the committed result and refuses one whose sources changed.

use agenticspec_sim::best::{fitness_as_written, yaml_of_result};
use agenticspec_sim::doc::parse_yaml;
use agenticspec_sim::{Experiment, run};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const RESULT: &str = "result.json";
pub const DIGEST: &str = "sources.sha256";

/// The command that writes the results.
pub const REGENERATE: &str = "task catalog:generate";

/// Files of the simulator that do not change a result: the commands of `agenticspec sim`, the
/// module list, the ladder, the best variant as a document (derived from the result by the docs
/// generator), the scenario runner and the rows of its `given`. `events.rs` counts: its table
/// names the events an `on_event` trigger fires on.
const SIM_EXCLUDED: &[&str] = &[
    "cli.rs",
    "lib.rs",
    "ladder.rs",
    "best.rs",
    "scenario.rs",
    "rows.rs",
];

/// The simulator's dependencies from crates.io whose versions a result depends on.
const SIM_DEPS: &[&str] = &["rayon", "serde_json", "serde_yaml_ng"];

/// One experiment and the files it reads.
pub struct Source {
    pub ep: PathBuf,
    /// `experiments/<file>`.
    pub exp_file: String,
    pub exp_text: String,
    pub edoc: Value,
    pub arch_path: PathBuf,
    pub arch_text: String,
    pub profile_path: PathBuf,
    pub profile_text: String,
    /// The entry: the architecture's file stem, or the experiment's when another entry has it.
    pub name: String,
}

impl Source {
    pub fn dir(&self) -> String {
        format!("website/static/catalog/{}", self.name)
    }
}

pub fn yaml_files(dir: &Path) -> Vec<PathBuf> {
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

pub fn stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Every experiment under `experiments/`, in file order.
pub fn sources(root: &Path) -> Result<Vec<Source>, String> {
    let mut out: Vec<Source> = Vec::new();
    for ep in yaml_files(&root.join("experiments")) {
        let exp_file = format!(
            "experiments/{}",
            ep.file_name().unwrap_or_default().to_string_lossy()
        );
        let exp_text = read(&ep)?;
        let edoc = parse_yaml(&exp_text).map_err(|e| format!("{exp_file}: {e}"))?;
        let dir = ep.parent().unwrap_or(Path::new("."));
        let field = |k: &str| {
            edoc[k]
                .as_str()
                .map(|p| dir.join(p))
                .ok_or(format!("{exp_file}: needs `{k}`"))
        };
        let arch_path = field("architecture")?;
        let profile_path = field("profile")?;
        let mut name = stem(&arch_path);
        if out.iter().any(|s| s.name == name) {
            name = stem(&ep);
        }
        out.push(Source {
            arch_text: read(&arch_path)?,
            profile_text: read(&profile_path)?,
            ep,
            exp_file,
            exp_text,
            edoc,
            arch_path,
            profile_path,
            name,
        });
    }
    Ok(out)
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `path` relative to `root`, with `/`.
fn relative(root: &Path, path: &Path) -> String {
    let (r, p) = (
        std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf()),
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()),
    );
    p.strip_prefix(&r)
        .unwrap_or(&p)
        .to_string_lossy()
        .replace('\\', "/")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// The versions of the simulator's dependencies in `Cargo.lock`: `rayon 1.12.0`, one per line.
fn locked_versions(lock: &str) -> String {
    let mut out = String::new();
    let mut name: Option<&str> = None;
    for line in lock.lines() {
        if let Some(n) = line.strip_prefix("name = ") {
            name = Some(n.trim_matches('"'));
        } else if let (Some(n), Some(v)) = (name, line.strip_prefix("version = ")) {
            if SIM_DEPS.contains(&n) {
                out.push_str(&format!("{n} {}\n", v.trim_matches('"')));
            }
            name = None;
        }
    }
    out
}

/// The digest file of a source: one `<sha256>  <what>` line per input of its result.
pub fn digest(root: &Path, s: &Source) -> Result<String, String> {
    let mut lines: Vec<String> = Vec::new();
    let mut add = |bytes: &[u8], what: String| lines.push(format!("{}  {what}", hex(bytes)));
    add(s.exp_text.as_bytes(), s.exp_file.clone());
    add(s.arch_text.as_bytes(), relative(root, &s.arch_path));
    add(s.profile_text.as_bytes(), relative(root, &s.profile_path));
    let sim = root.join("crates/agenticspec-sim/src");
    let mut files = Vec::new();
    walk(&sim, &mut files);
    files.sort();
    if files.is_empty() {
        return Err(format!(
            "no source of the simulator under {}",
            relative(root, &sim)
        ));
    }
    for f in files {
        let rel = relative(root, &f);
        if SIM_EXCLUDED
            .iter()
            .any(|x| rel == format!("crates/agenticspec-sim/src/{x}"))
        {
            continue;
        }
        add(&std::fs::read(&f).map_err(|e| format!("{rel}: {e}"))?, rel);
    }
    let lock = read(&root.join("Cargo.lock"))?;
    let versions = locked_versions(&lock);
    add(
        versions.as_bytes(),
        format!("Cargo.lock ({})", versions.trim_end().replace('\n', ", ")),
    );
    add(
        read(&root.join("rust-toolchain.toml"))?.as_bytes(),
        "rust-toolchain.toml".into(),
    );
    let mut text = lines.join("\n");
    text.push('\n');
    Ok(text)
}

/// The inputs whose line differs between two digest files.
fn changed(written: &str, now: &str) -> Vec<String> {
    let what = |l: &str| {
        l.split_once("  ")
            .map_or(l.to_string(), |(_, w)| w.to_string())
    };
    let mut out: Vec<String> = now
        .lines()
        .filter(|l| !written.lines().any(|w| w == *l))
        .map(what)
        .collect();
    for l in written.lines() {
        let w = what(l);
        if !now.lines().any(|n| what(n) == w) && !out.contains(&w) {
            out.push(w);
        }
    }
    out
}

/// The committed result of a source, after its digest file is checked against its sources.
pub fn committed(root: &Path, s: &Source) -> Result<Value, String> {
    let dir = s.dir();
    let now = digest(root, s)?;
    let written = std::fs::read_to_string(root.join(&dir).join(DIGEST)).unwrap_or_default();
    if written != now {
        let what = if written.is_empty() {
            format!("{dir}/{DIGEST} is missing")
        } else {
            format!(
                "{dir}/{RESULT} is not current: changed since it was written: {}",
                changed(&written, &now).join(", ")
            )
        };
        return Err(format!("{}: {what}; run `{REGENERATE}`", s.exp_file));
    }
    let text = std::fs::read_to_string(root.join(&dir).join(RESULT))
        .map_err(|e| format!("{dir}/{RESULT}: {e}; run `{REGENERATE}`"))?;
    // serde_json reads a float back to within a unit in the last place, so the pages may print the
    // last digit of a float differently from this file; the file itself is the bytes of the run.
    serde_json::from_str(&text).map_err(|e| format!("{dir}/{RESULT}: {e}"))
}

pub fn json_text(v: &Value) -> Result<String, String> {
    let mut s = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    s.push('\n');
    Ok(s)
}

/// Runs the experiment of `s` as configured, and refuses a result whose best variant, written as
/// an architecture, does not score the best fitness.
fn fresh(s: &Source, threads: usize) -> Result<Value, String> {
    let e = Experiment::load(&s.ep)?;
    let result = run(&e, threads)?;
    if let Some(text) = yaml_of_result(&result, Some(&s.arch_text))? {
        let want = result["best"]["fitness"].as_f64().unwrap_or(f64::NAN);
        let got = fitness_as_written(&e, parse_yaml(&text)?, threads)?;
        if got != want {
            return Err(format!(
                "its best variant written as an architecture scores {got}, not {want}"
            ));
        }
    }
    Ok(result)
}

/// Runs every experiment and writes, or with `check` compares, its result and digest file. Returns
/// one line per file that differs.
pub fn generate(root: &Path, check: bool) -> Result<Vec<String>, String> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get().min(8));
    let mut out = Vec::new();
    for s in sources(root)? {
        let result = fresh(&s, threads).map_err(|e| format!("{}: {e}", s.exp_file))?;
        let files = [
            (format!("{}/{RESULT}", s.dir()), json_text(&result)?),
            (format!("{}/{DIGEST}", s.dir()), digest(root, &s)?),
        ];
        for (path, text) in files {
            match std::fs::read(root.join(&path)) {
                Ok(bytes) if bytes == text.as_bytes() => continue,
                Ok(_) => out.push(format!("changed: {path}")),
                Err(_) => out.push(format!("missing: {path}")),
            }
            if !check {
                let target = root.join(&path);
                if let Some(d) = target.parent() {
                    std::fs::create_dir_all(d).map_err(|e| format!("{path}: {e}"))?;
                }
                std::fs::write(&target, text).map_err(|e| format!("{path}: {e}"))?;
            }
        }
    }
    Ok(out)
}

/// Whether a path under `website/static/catalog/` is one of these committed inputs.
pub fn is_input(path: &str) -> bool {
    path.strip_prefix("website/static/catalog/")
        .and_then(|p| p.split_once('/'))
        .is_some_and(|(_, f)| f == RESULT || f == DIGEST)
}
