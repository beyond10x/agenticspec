//! agenticspec-docs: the generated pages of the documentation site.
//!
//! From the compiled `agentic` system (`blocks/`), the KPI catalogue (`kpis/catalogue.yaml`), the
//! architectures and the experiments that name them, it writes the layer map, one page per block,
//! one page per architecture with its rendered graph, the KPI catalogue page, and the data the
//! site's components draw. The output is a function of those inputs only.

mod arch;
mod blocks;
mod kpis;
mod layout;
mod md;
mod model;

pub use kpis::{Catalogue, Kpi};
pub use model::layer_edges;

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where a repository file is read on the forge.
pub const SOURCE_BASE: &str = "https://github.com/beyond10x/agenticspec/blob/main/";

/// Directories whose every file is generated: a file there that the generator does not write is
/// stale.
pub const OWNED_DIRS: &[&str] = &[
    "docs/blocks",
    "docs/architectures",
    "website/static/architectures",
    "website/src/generated",
];

/// Generated files outside those directories, and the ones every run writes.
pub const OWNED_FILES: &[&str] = &[
    "docs/kpis-catalogue.md",
    "docs/blocks/index.md",
    "docs/blocks/planned.md",
    "docs/architectures/index.md",
    "website/src/generated/blocks.json",
    "website/src/generated/kpis.json",
    "website/src/generated/sidebar.json",
];

#[derive(Debug)]
pub struct Generated {
    /// The compiled IR of `blocks/`.
    pub ir: Value,
    /// Every generated file: path relative to the repository root, and its text.
    pub files: BTreeMap<String, String>,
}

/// `ess specify compile --path <root>/blocks --format json`.
pub fn compile(root: &Path) -> Result<Value, String> {
    let out = Command::new("ess")
        .args(["specify", "compile", "--format", "json", "--path"])
        .arg(root.join("blocks"))
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

/// The catalogue at `kpis/catalogue.yaml`, or an empty one when the file does not exist.
pub fn read_catalogue(root: &Path) -> Result<Catalogue, String> {
    let path = root.join("kpis/catalogue.yaml");
    if !path.exists() {
        return Ok(Catalogue::default());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("kpis/catalogue.yaml: {e}"))?;
    Catalogue::parse(&text).map_err(|e| format!("kpis/catalogue.yaml: {e}"))
}

fn json_text(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).unwrap_or_default();
    s.push('\n');
    s
}

/// Every generated file for the repository at `root`. `catalogue` replaces `kpis/catalogue.yaml`
/// when given.
pub fn generate(root: &Path, catalogue: Option<Catalogue>) -> Result<Generated, String> {
    let ir = compile(root)?;
    let catalogue = match catalogue {
        Some(c) => c,
        None => read_catalogue(root)?,
    };
    let blocks = model::blocks(&root.join("blocks"), &ir)?;
    // A block the catalogue names before blocks/ declares it is planned: its KPIs go on the page
    // of planned blocks. scripts/check-kpis.sh decides which names are allowed.
    let planned: Vec<String> = catalogue
        .kpis
        .iter()
        .map(|k| k.block.clone())
        .filter(|n| !blocks.iter().any(|b| &b.name == n))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let edges = layer_edges(&ir);
    let ctx = blocks::Ctx {
        ir: &ir,
        blocks: &blocks,
        edges: &edges,
        catalogue: &catalogue,
        planned: &planned,
    };
    let index_of = |d: &str| blocks.iter().position(|b| b.domain == d);
    let pairs: Vec<(usize, usize, usize)> = edges
        .iter()
        .filter_map(|((f, t), n)| Some((index_of(f)?, index_of(t)?, *n)))
        .collect();
    let map = layout::layout(&blocks, &pairs);

    let mut files = BTreeMap::new();
    files.insert("docs/blocks/index.md".to_string(), blocks::index(&ctx));
    for b in &blocks {
        files.insert(format!("{}.md", b.doc_id()), blocks::page(&ctx, b));
    }
    files.insert(
        "docs/blocks/planned.md".to_string(),
        blocks::planned_page(&ctx),
    );
    files.insert(
        "docs/kpis-catalogue.md".to_string(),
        blocks::catalogue_page(&ctx),
    );
    files.insert(
        "website/src/generated/blocks.json".to_string(),
        json_text(&blocks::map_json(&ctx, &map)),
    );
    files.insert(
        "website/src/generated/kpis.json".to_string(),
        json_text(&blocks::kpi_json(&ctx)),
    );

    let archs = arch::load(root)?;
    files.insert(
        "docs/architectures/index.md".to_string(),
        arch::index(&archs),
    );
    for a in &archs {
        files.insert(
            format!("docs/architectures/{}.md", a.name),
            arch::page(a, &blocks),
        );
        files.insert(
            format!("website/static/architectures/{}.html", a.name),
            arch::html(a),
        );
    }
    let names: Vec<String> = archs.iter().map(|a| a.name.clone()).collect();
    files.insert(
        "website/src/generated/sidebar.json".to_string(),
        json_text(&blocks::sidebar_json(&blocks, &names)),
    );
    Ok(Generated { ir, files })
}

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

/// The files on disk under the owned directories, relative to `root`.
fn on_disk(root: &Path) -> BTreeSet<String> {
    let mut found = Vec::new();
    for d in OWNED_DIRS {
        walk(&root.join(d), &mut found);
    }
    found
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(root)
                .ok()
                .map(|r| r.to_string_lossy().replace('\\', "/"))
        })
        .collect()
}

/// One line per file that differs from the generation: changed, missing, or stale.
pub fn check(root: &Path, files: &BTreeMap<String, String>) -> Vec<String> {
    let mut out = Vec::new();
    for (path, text) in files {
        match std::fs::read(root.join(path)) {
            Ok(bytes) if bytes == text.as_bytes() => {}
            Ok(_) => out.push(format!("changed: {path}")),
            Err(_) => out.push(format!("missing: {path}")),
        }
    }
    for path in on_disk(root) {
        if !files.contains_key(&path) {
            out.push(format!("not generated: {path}"));
        }
    }
    out
}

/// Writes every file that differs and removes the stale ones; returns what it changed.
pub fn write(root: &Path, files: &BTreeMap<String, String>) -> Result<Vec<String>, String> {
    let changed = check(root, files);
    for (path, text) in files {
        let target = root.join(path);
        if std::fs::read(&target).is_ok_and(|b| b == text.as_bytes()) {
            continue;
        }
        if let Some(dir) = target.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(&target, text).map_err(|e| format!("{path}: {e}"))?;
    }
    for path in on_disk(root) {
        if !files.contains_key(&path) {
            std::fs::remove_file(root.join(&path)).map_err(|e| format!("{path}: {e}"))?;
        }
    }
    Ok(changed)
}
