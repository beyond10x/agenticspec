//! `agenticspec-docs [--root DIR] [--check]`: write the generated pages of the documentation site,
//! or compare them byte for byte with a fresh generation.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage: agenticspec-docs [--root DIR] [--check]
  writes docs/blocks/, docs/architectures/, docs/catalog/, docs/kpis-catalogue.md,
  website/static/architectures/, website/static/catalog/
  and website/src/generated/ from blocks/, kpis/catalogue.yaml, architectures/, experiments/
  and profiles/;
  --check writes nothing and exits 1 when a file differs, is missing or is not generated";

fn main() -> ExitCode {
    let mut root = PathBuf::from(".");
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--check" => check = true,
            "--root" => match args.next() {
                Some(r) => root = PathBuf::from(r),
                None => {
                    eprintln!("agenticspec-docs: --root needs a directory\n{USAGE}");
                    return ExitCode::from(2);
                }
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("agenticspec-docs: unknown argument {other}\n{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let started = Instant::now();
    let generated = match agenticspec_docs::generate(&root, None) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("agenticspec-docs: {e}");
            return ExitCode::from(2);
        }
    };
    if check {
        let diff = agenticspec_docs::check(&root, &generated.files);
        if diff.is_empty() {
            println!(
                "docs: {} generated files match ({:.1} s)",
                generated.files.len(),
                started.elapsed().as_secs_f64()
            );
            ExitCode::SUCCESS
        } else {
            for d in &diff {
                println!("docs: {d}");
            }
            println!(
                "docs: {} of the generated files differ; run `task docs:generate`",
                diff.len()
            );
            ExitCode::from(1)
        }
    } else {
        match agenticspec_docs::write(&root, &generated.files) {
            Ok(changed) => {
                for c in &changed {
                    println!("docs: {c}");
                }
                println!(
                    "docs: {} generated files, {} written or removed ({:.1} s)",
                    generated.files.len(),
                    changed.len(),
                    started.elapsed().as_secs_f64()
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("agenticspec-docs: {e}");
                ExitCode::from(2)
            }
        }
    }
}
