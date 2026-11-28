//! agenticspec-docs: the generated pages of the documentation site.
//!
//! From the compiled `agentic` system (`blocks/`), the KPI catalogue (`kpis/catalogue.yaml`), the
//! architectures and the experiments that name them, it writes the layer map, one page per block,
//! one page per architecture with its rendered graph, the KPI catalogue page, and the data the
//! site's components draw; and, from the whole system (the root `ess-inputs.yaml`), one page per
//! block of a pack, the index of the packs, and the pages of the fields with their architectures.
//! The output is a function of those inputs only.

mod arch;
mod blocks;
mod catalog;
mod contract;
mod kpis;
mod layout;
mod md;
mod model;
mod packs;
mod results;

pub use kpis::{Catalogue, Kpi};
pub use model::layer_edges;
pub use results::without_view;

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
    "docs/catalog",
    "website/static/catalog",
    "website/src/generated",
    "docs/packs",
    "docs/fields",
    "website/static/fields",
];

/// Generated files outside those directories, and the ones every run writes.
pub const OWNED_FILES: &[&str] = &[
    "docs/kpis-catalogue.md",
    "docs/blocks/index.md",
    "docs/blocks/planned.md",
    "docs/architectures/index.md",
    "docs/catalog/index.md",
    "website/src/generated/blocks.json",
    "website/src/generated/kpis.json",
    "website/src/generated/sidebar.json",
    "docs/packs/index.md",
    "docs/fields/index.md",
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
    compile_at(&root.join("blocks"))
}

/// `ess specify compile --path <path> --format json`: `<root>/blocks` for the core, `<root>` for
/// the whole system.
pub fn compile_at(path: &Path) -> Result<Value, String> {
    let out = Command::new("ess")
        .args(["specify", "compile", "--format", "json", "--path"])
        .arg(path)
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

/// Runs every experiment of the catalog at its own budget and seeds and writes its committed result
/// and digest file, or with `check` compares them byte for byte; one line per file that differs.
pub fn catalog_results(root: &Path, check: bool) -> Result<Vec<String>, String> {
    results::generate(root, check)
}

/// Checks the committed result of every experiment against the digest of its sources, as
/// [`generate`] does before it reads them; returns how many there are.
pub fn check_committed(root: &Path) -> Result<usize, String> {
    let sources = results::sources(root)?;
    for s in &sources {
        results::committed(root, s)?;
    }
    Ok(sources.len())
}

/// Every generated file for the repository at `root`. `catalogue` replaces `kpis/catalogue.yaml`
/// when given.
pub fn generate(root: &Path, catalogue: Option<Catalogue>) -> Result<Generated, String> {
    let ir = compile(root)?;
    // The committed results first: a stale one is refused before the architectures run.
    let cat = catalog::load(root)?;
    // The short runs of the architectures take longest; the other pages are made beside them.
    std::thread::scope(|s| {
        let archs = s.spawn(|| arch::load(root));
        pages(root, catalogue, ir, cat, || {
            archs
                .join()
                .unwrap_or_else(|p| std::panic::resume_unwind(p))
        })
    })
}

fn pages(
    root: &Path,
    catalogue: Option<Catalogue>,
    ir: Value,
    cat: catalog::Catalog,
    archs: impl FnOnce() -> Result<Vec<arch::Architecture>, String>,
) -> Result<Generated, String> {
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

    let packs = packs::generate(root, &catalogue);
    let archs = archs()?;
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
        if let Ok(shown) = &a.contract {
            files.insert(
                format!("website/static/architectures/{}.contract.yaml", a.name),
                shown.contract.text(),
            );
        }
    }
    let names: Vec<String> = archs.iter().map(|a| a.name.clone()).collect();

    files.insert("docs/catalog/index.md".to_string(), catalog::index(&cat));
    for e in &cat.entries {
        files.insert(format!("docs/catalog/{}.md", e.name), catalog::page(e));
        files.extend(catalog::files(e)?);
    }
    let entries: Vec<String> = cat.entries.iter().map(|e| e.name.clone()).collect();

    let (sidebar_packs, sidebar_fields) = match packs? {
        Some(out) => {
            if let Some(index) = files.get_mut("docs/blocks/index.md") {
                index.push_str(
                    "\n## Packs and fields\n\n\
These blocks are the core: what every agentic system has. Above them, [packs](../packs/index.md) hold the blocks more than one field of work reuses \
(source control, tracking, file tools, a shell), and [fields](../fields/index.md) name one kind of work, the packs it uses and its example architectures. \
No core block depends on a pack or a field.\n",
                );
            }
            files.extend(out.files);
            (out.sidebar_packs, out.sidebar_fields)
        }
        None => (Vec::new(), Vec::new()),
    };
    files.insert(
        "website/src/generated/sidebar.json".to_string(),
        json_text(&blocks::sidebar_json(
            &blocks,
            &names,
            &entries,
            sidebar_packs,
            sidebar_fields,
        )),
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

/// The files on disk under the owned directories, relative to `root`, without the committed
/// results of the catalog, which are inputs (`task catalog:generate`).
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
        .filter(|p| !results::is_input(p))
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
