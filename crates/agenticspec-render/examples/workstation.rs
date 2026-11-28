//! Writes the page of `architectures/workstation.yaml` to `target/render/workstation.html`.
//!
//! `cargo run -p agenticspec-render --example workstation [-- <result.json> [<out.html>]]`
//! With a result file (format `agenticspec-sim-result/1`) the page shows it too.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join("architectures/workstation.yaml"))?;
    let architecture: serde_json::Value = serde_yaml_ng::from_str(&text)?;
    let mut args = std::env::args().skip(1);
    let result: Option<serde_json::Value> = match args.next() {
        Some(path) => Some(serde_json::from_str(&std::fs::read_to_string(path)?)?),
        None => None,
    };
    let out = match args.next() {
        Some(path) => PathBuf::from(path),
        None => root.join("target/render/workstation.html"),
    };
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        &out,
        agenticspec_render::architecture::page(&architecture, result.as_ref()),
    )?;
    println!("{}", out.display());
    Ok(())
}
