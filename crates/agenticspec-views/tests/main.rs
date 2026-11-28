//! The block panels: the rules of a panel on a small system (`tests/fixtures/mini/`), writing and
//! checking the generated files, and the fixture rows of the real blocks against their model.
//! Every test compiles its system with `ess` (well under a second) and reads no network.

mod files;
mod rows;
mod rules;

use std::path::{Path, PathBuf};

use serde_json::Value;

/// The repository root.
pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The small system's `blocks/`.
pub fn mini_blocks() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini/blocks")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create a directory");
    for e in std::fs::read_dir(from)
        .expect("read a fixture directory")
        .flatten()
    {
        let p = e.path();
        let target = to.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &target);
        } else {
            std::fs::copy(&p, &target).expect("copy a fixture file");
        }
    }
}

/// A fresh repository root for one test: the small system as `blocks/` and this generator's
/// sources, which the digest file names.
pub fn root(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("views-tests")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("remove the root of an earlier run");
    }
    copy_dir(&mini_blocks(), &dir.join("blocks"));
    copy_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        &dir.join(agenticspec_views::GENERATOR),
    );
    dir
}

/// The compiled IR and the blocks of a `blocks/` directory.
pub fn model(blocks: &Path) -> (Value, Vec<agenticspec_views::Block>) {
    let ir = agenticspec_views::compile(blocks).expect("the system compiles");
    let list = agenticspec_views::blocks(blocks, &ir).expect("the blocks are in layer directories");
    (ir, list)
}

/// The generated files of a model, by path below `ui/views/`.
pub fn generated(ir: &Value, blocks: &[agenticspec_views::Block]) -> Vec<(String, String)> {
    agenticspec_views::panels::generate(ir, blocks).expect("the panels generate")
}

/// One generated file, parsed: the document as YAML, a fixture as JSON.
pub fn parsed(files: &[(String, String)], path: &str) -> Value {
    let text = &files
        .iter()
        .find(|(p, _)| p == path)
        .unwrap_or_else(|| panic!("{path} is generated"))
        .1;
    if path.ends_with(".json") {
        serde_json::from_str(text).expect("a fixture is JSON")
    } else {
        serde_yaml_ng::from_str(text).expect("the document is YAML")
    }
}
