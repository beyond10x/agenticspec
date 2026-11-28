//! The block panels of `agenticspec ui panels`: from the ESS system `blocks/`, one `ess-ui/1`
//! document, `ui/views/blocks.ui.yaml`, with one widget per view and a gallery page per block,
//! the fixtures the panels render against (`ui/views/fixtures/`), the part of the compiled
//! blocks a derivation reads (`ui/views/blocks.model.json`), and the digest of every input read
//! (`ui/views/blocks.sources.sha256`). The bytes are a function of those inputs only.
//!
//! A generation compiles `blocks/` with `ess specify compile` (well under a second), so
//! `--check` regenerates in memory and compares every byte and the digest; no slow lane is
//! needed.

pub mod panels;
pub mod values;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub use panels::Block;

/// The generated document, relative to the repository root.
pub const OUT: &str = "ui/views/blocks.ui.yaml";
/// The digest of the inputs, beside it.
pub const DIGEST: &str = "ui/views/blocks.sources.sha256";
/// The part of the compiled blocks `agenticspec ui derive` reads, beside it: what the binary
/// embeds so that `agenticspec ui build` derives with neither `ess` nor `blocks/`.
pub const MODEL: &str = "ui/views/blocks.model.json";
/// The directory every generated file is in.
pub const DIR: &str = "ui/views";
/// The directory of the fixtures, which holds nothing but generated files.
pub const FIXTURES: &str = "ui/views/fixtures";
/// The sources of this generator, whose digest the digest file holds.
pub const GENERATOR: &str = "crates/agenticspec-views/src";

/// Why `agenticspec ui panels` refused: the code of the command line's error, and the detail.
#[derive(Debug)]
pub struct Refusal {
    pub code: &'static str,
    pub detail: String,
}

fn refused(code: &'static str, detail: impl Into<String>) -> Refusal {
    Refusal {
        code,
        detail: detail.into(),
    }
}

/// `ess specify compile --path <blocks> --format json`.
///
/// # Errors
///
/// `ess` cannot be started, refuses the system or prints no JSON.
pub fn compile(blocks: &Path) -> Result<Value, String> {
    let out = Command::new("ess")
        .args(["specify", "compile", "--format", "json", "--path"])
        .arg(blocks)
        .output()
        .map_err(|e| format!("ess: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "ess specify compile failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("ess output: {e}"))
}

/// The `ess --version` line.
fn ess_version() -> Result<String, String> {
    let out = Command::new("ess")
        .arg("--version")
        .output()
        .map_err(|e| format!("ess: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The files of `specification:` in `<blocks>/ess-inputs.yaml`, relative to `<blocks>`.
fn specification(blocks: &Path) -> Result<Vec<String>, String> {
    let path = blocks.join("ess-inputs.yaml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc: Value =
        serde_yaml_ng::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(doc["specification"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect())
}

/// The blocks, ordered by layer and then by name: each domain file of the specification list in
/// a numbered layer directory (`4-agent/agent.yaml`) and its `domain:` line.
///
/// # Errors
///
/// A file that cannot be read, or a domain outside a numbered layer directory.
pub fn blocks(blocks: &Path, ir: &Value) -> Result<Vec<Block>, String> {
    let system = ir["system"].as_str().unwrap_or("");
    let mut out = Vec::new();
    for file in specification(blocks)? {
        let Some((dir, _)) = file.split_once('/') else {
            continue;
        };
        let text =
            std::fs::read_to_string(blocks.join(&file)).map_err(|e| format!("{file}: {e}"))?;
        let Some(domain) = text
            .lines()
            .find_map(|l| l.strip_prefix("domain:"))
            .map(|d| d.trim().to_string())
        else {
            continue;
        };
        let layer = dir
            .split_once('-')
            .and_then(|(n, _)| n.parse::<u32>().ok())
            .ok_or_else(|| format!("{file}: {domain} is not in a numbered layer directory"))?;
        let short = domain
            .strip_prefix(&format!("{system}."))
            .unwrap_or(&domain)
            .to_string();
        out.push(Block {
            domain,
            short,
            layer,
            dir: dir.to_string(),
        });
    }
    out.sort_by(|a, b| (a.layer, &a.short).cmp(&(b.layer, &b.short)));
    Ok(out)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// The digest file: one `<sha256>  <input>` line per input read, the `ess` that compiled them
/// and every source file of this generator.
///
/// # Errors
///
/// An input that cannot be read.
pub fn digest(root: &Path) -> Result<String, String> {
    let blocks = root.join("blocks");
    let mut lines = Vec::new();
    let version = ess_version()?;
    lines.push(format!(
        "{}  ess --version: {version}",
        sha(version.as_bytes())
    ));
    let mut files = vec!["ess-inputs.yaml".to_string()];
    files.extend(specification(&blocks)?);
    for f in files {
        let bytes = std::fs::read(blocks.join(&f)).map_err(|e| format!("blocks/{f}: {e}"))?;
        lines.push(format!("{}  blocks/{f}", sha(&bytes)));
    }
    let mut sources: Vec<PathBuf> = std::fs::read_dir(root.join(GENERATOR))
        .map_err(|e| format!("{GENERATOR}: {e}"))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    sources.sort();
    for p in sources {
        let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        lines.push(format!("{}  {GENERATOR}/{name}", sha(&bytes)));
    }
    Ok(lines.join("\n") + "\n")
}

/// Every generated file, by its path relative to the repository root, in a fixed order: the
/// document, the fixtures and the digest file.
///
/// # Errors
///
/// `blocks/` does not compile, or the generation refuses the model.
pub fn generation(root: &Path) -> Result<Vec<(String, String)>, String> {
    let blocks_dir = root.join("blocks");
    let ir = compile(&blocks_dir)?;
    let list = blocks(&blocks_dir, &ir)?;
    let mut files: Vec<(String, String)> = panels::generate(&ir, &list)?
        .into_iter()
        .map(|(p, t)| (format!("{DIR}/{p}"), t))
        .collect();
    files.push((MODEL.to_string(), model(&ir)));
    files.push((DIGEST.to_string(), digest(root)?));
    Ok(files)
}

/// The part of the compiled blocks a derivation reads (`agenticspec_ui::derive::model::Model`):
/// the system's name, every domain and command with its display name, the names of the entities,
/// the source of each view, the input fields of each command and the variants of each enum.
pub fn model(ir: &Value) -> String {
    let entries = |key: &str| ir[key].as_object().into_iter().flatten();
    let display = |v: &Value| match v["naming"]["display"].as_str() {
        Some(d) => json!({ "naming": { "display": d } }),
        None => json!({}),
    };
    let domains: serde_json::Map<String, Value> = entries("domains")
        .map(|(k, v)| (k.clone(), display(v)))
        .collect();
    let entities: serde_json::Map<String, Value> = entries("entities")
        .map(|(k, _)| (k.clone(), json!({})))
        .collect();
    let views: serde_json::Map<String, Value> = entries("views")
        .map(|(k, v)| match v["source"].as_str() {
            Some(s) => (k.clone(), json!({ "source": s })),
            None => (k.clone(), json!({})),
        })
        .collect();
    let commands: serde_json::Map<String, Value> = entries("commands")
        .map(|(k, v)| {
            let mut c = display(v);
            let input: Vec<Value> = v["input"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|f| f["name"].as_str())
                .map(|n| json!({ "name": n }))
                .collect();
            c["input"] = Value::Array(input);
            (k.clone(), c)
        })
        .collect();
    let types: serde_json::Map<String, Value> = entries("types")
        .filter_map(|(k, v)| {
            let variants = v["body"]["variants"].as_array()?;
            Some((k.clone(), json!({ "body": { "variants": variants } })))
        })
        .collect();
    let doc = json!({
        "system": ir["system"],
        "domains": domains,
        "entities": entities,
        "views": views,
        "commands": commands,
        "types": types,
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n"
}

/// The files below `dir`, relative to `root`, sorted.
fn tree(root: &Path, dir: &str) -> Vec<String> {
    fn walk(base: &Path, d: &Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if let Ok(rel) = p.strip_prefix(base) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &root.join(dir), &mut out);
    out.sort();
    out
}

/// What differs between a fresh generation and the files under `root`: a file that differs, is
/// missing, or lies under the fixtures directory and is not generated. Empty when current.
pub fn differences(root: &Path, files: &[(String, String)]) -> Vec<String> {
    let mut out = Vec::new();
    for (path, text) in files {
        match std::fs::read_to_string(root.join(path)) {
            Ok(t) if t == *text => {}
            Ok(_) => out.push(format!("{path}: differs from a fresh generation")),
            Err(_) => out.push(format!("{path}: generated, and not in the tree")),
        }
    }
    for f in tree(root, FIXTURES) {
        if !files.iter().any(|(p, _)| *p == f) {
            out.push(format!("{f}: in the tree, and not generated"));
        }
    }
    out
}

/// `agenticspec ui panels`: write the generation under `root`, or with `check` compare it with
/// the files there.
///
/// # Errors
///
/// `blocks_unreadable` when `blocks/` does not compile or the generation refuses it,
/// `panels_stale` when `check` finds a difference (each is named), `out_unwritable` when a file
/// cannot be written.
pub fn run(root: &Path, check: bool) -> Result<Value, Refusal> {
    let files = generation(root).map_err(|e| refused("blocks_unreadable", e))?;
    let panels = files
        .iter()
        .filter(|(p, _)| {
            p.starts_with(&format!("{FIXTURES}/"))
                && p.ends_with(".json")
                && !p.contains("/scripts/")
        })
        .count();
    let answer = |written: bool| json!({ "out": OUT, "panels": panels, "files": files.len(), "written": written });
    if check {
        let diff = differences(root, &files);
        if diff.is_empty() {
            return Ok(answer(false));
        }
        let shown: Vec<&str> = diff.iter().take(20).map(String::as_str).collect();
        let more = if diff.len() > 20 {
            format!("; {} more", diff.len() - 20)
        } else {
            String::new()
        };
        return Err(refused(
            "panels_stale",
            format!(
                "{}{more}. Generated files are never edited: run task ui:panels",
                shown.join("; ")
            ),
        ));
    }
    let fixtures = root.join(FIXTURES);
    if fixtures.exists() {
        std::fs::remove_dir_all(&fixtures)
            .map_err(|e| refused("out_unwritable", format!("{FIXTURES}: {e}")))?;
    }
    for (path, text) in &files {
        let p = root.join(path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| refused("out_unwritable", format!("{}: {e}", parent.display())))?;
        }
        std::fs::write(&p, text).map_err(|e| refused("out_unwritable", format!("{path}: {e}")))?;
    }
    Ok(answer(true))
}
