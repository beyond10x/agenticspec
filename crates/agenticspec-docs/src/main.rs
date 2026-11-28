//! `agenticspec-docs [--root DIR] [--check]`: write the generated pages of the documentation site,
//! or compare them byte for byte with a fresh generation; with `--results`, run the experiments of
//! the catalog and write (or compare) their committed results; with `--bundles`, write the
//! fixture bundles of `bundles/` (with `--check`: compare the digests of their sources; with
//! `--slow`: build them afresh and compare bytes).

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str =
    "usage: agenticspec-docs [--root DIR] [--results | --bundles] [--check | --slow]
  writes docs/blocks/, docs/architectures/, docs/catalog/, docs/kpis-catalogue.md,
  website/static/architectures/, website/static/catalog/
  and website/src/generated/ from blocks/, kpis/catalogue.yaml, architectures/, experiments/
  and fields/ (with packs/ and the root ess-inputs.yaml: docs/packs/, docs/fields/);
  --check writes nothing and exits 1 when a file differs, is missing or is not generated;
  --results runs every experiment at its own budget and seeds and writes
  website/static/catalog/<entry>/result.json and sources.sha256 (with --check: compares them);
  --bundles writes the fixture bundles bundles/<architecture>.<binding>.yaml and
  bundles/sources.sha256 (with --check: compares the digests of their sources; with --slow:
  builds each afresh and compares bytes)";

fn catalog_results(root: &std::path::Path, check: bool, started: Instant) -> ExitCode {
    match agenticspec_docs::catalog_results(root, check) {
        Ok(diff) => {
            for d in &diff {
                println!("catalog: {d}");
            }
            let secs = started.elapsed().as_secs_f64();
            if !check {
                println!("catalog: {} results written ({secs:.1} s)", diff.len());
                ExitCode::SUCCESS
            } else if diff.is_empty() {
                println!(
                    "catalog: every committed result is the bytes of a fresh run ({secs:.1} s)"
                );
                ExitCode::SUCCESS
            } else {
                println!(
                    "catalog: {} committed files differ from a fresh run; run `task catalog:generate`",
                    diff.len()
                );
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("agenticspec-docs: {e}");
            ExitCode::from(2)
        }
    }
}

/// `--bundles`: write the fixture bundles of `bundles/` and their digest file; with `--check`,
/// compare the digests of their sources only; with `--slow`, build each afresh and compare bytes.
fn fixture_bundles(root: &std::path::Path, check: bool, slow: bool, started: Instant) -> ExitCode {
    use agenticspec_sim::deploy::fixtures;
    let outcome = if slow {
        fixtures::stale(root).and_then(|mut d| {
            d.extend(fixtures::check(root)?);
            Ok(d)
        })
    } else if check {
        fixtures::check(root)
    } else {
        fixtures::write(root)
    };
    let secs = started.elapsed().as_secs_f64();
    match outcome {
        Ok(lines) if !check && !slow => {
            println!("bundles: {} files written ({secs:.1} s)", lines.len());
            ExitCode::SUCCESS
        }
        Ok(lines) if lines.is_empty() => {
            let what = if slow {
                "is the bytes of a fresh build"
            } else {
                "was written from its current sources"
            };
            println!("bundles: every fixture bundle {what} ({secs:.1} s)");
            ExitCode::SUCCESS
        }
        Ok(lines) => {
            for l in &lines {
                println!("bundles: {l}");
            }
            println!(
                "bundles: {} difference(s); run `{}`",
                lines.len(),
                fixtures::REGENERATE
            );
            ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("agenticspec-docs: {e}");
            ExitCode::from(2)
        }
    }
}

fn main() -> ExitCode {
    let mut root = PathBuf::from(".");
    let mut check = false;
    let mut results = false;
    let mut bundles = false;
    let mut slow = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--check" => check = true,
            "--results" => results = true,
            "--bundles" => bundles = true,
            "--slow" => slow = true,
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
    if bundles {
        return fixture_bundles(&root, check, slow, started);
    }
    if results {
        return catalog_results(&root, check, started);
    }
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
