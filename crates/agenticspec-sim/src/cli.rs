//! The commands of `agenticspec sim`: check, digest, run, render and ladder. Each returns the result
//! the command line prints, a JSON value of the shape `spec/domains/cli.yaml` declares
//! (`agenticspec.cli.Sim*Result`), or a [`Refusal`] with one of the error codes `spec/cli.yaml`
//! binds to the command. The parser and the process adapter are the generated
//! `agenticspec-cli-contract` of the `agenticspec` crate.

use crate::check::check as check_architecture;
use crate::doc::load_yaml;
use crate::{Experiment, ladder as ladder_doc, nest, run as run_experiment};
use serde_json::{Value, json};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

/// A refused command: the error code of the binding and its detail.
#[derive(Debug, Clone, PartialEq, Eq)]
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

fn write(path: &Path, text: &str) -> Result<(), Refusal> {
    let unwritable = |e: String| refused("out_unwritable", e);
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| unwritable(format!("{}: {e}", dir.display())))?;
    }
    std::fs::write(path, text).map_err(|e| unwritable(format!("{}: {e}", path.display())))
}

/// The document at `path` with its nested architectures resolved, or the refusal of `sim check`.
fn load_nested(path: &Path) -> Result<Value, Refusal> {
    nest::load(path).map_err(|e| match e {
        nest::Error::Unreadable(e) => refused("document_unreadable", e),
        refusals @ nest::Error::Refused(_) => refused("architecture_refused", refusals.to_string()),
    })
}

/// `sim check <architecture>`: the architecture with its counts and its nested architectures, or
/// every refusal of the check. The references are resolved first; the check then reads the
/// document flattened.
pub fn check(path: &Path) -> Result<Value, Refusal> {
    let arch = load_nested(path)?;
    let refusals = check_architecture(&nest::flatten(&arch));
    if !refusals.is_empty() {
        let names: Vec<String> = refusals.iter().map(ToString::to_string).collect();
        return Err(refused("architecture_refused", names.join("; ")));
    }
    Ok(json!({
        "architecture": arch["architecture"].as_str().unwrap_or("architecture"),
        "parts": arch["parts"].as_object().map_or(0, |p| p.len()),
        "links": arch["links"].as_array().map_or(0, |l| l.len()),
        "nested": nest::listed(&arch),
    }))
}

/// `sim digest <architecture>`: the digest a referring document pins it with.
pub fn digest(path: &Path) -> Result<Value, Refusal> {
    let unreadable = |e: String| refused("document_unreadable", format!("{}: {e}", path.display()));
    let text = std::fs::read_to_string(path).map_err(|e| unreadable(e.to_string()))?;
    let digest = nest::digest_text(&text).map_err(unreadable)?;
    Ok(json!({ "architecture": path.display().to_string(), "digest": digest }))
}

/// KPIs of triggers and swarms, printed only when a run has them.
const TRIGGER_KPIS: &[&str] = &[
    "firings",
    "firings_skipped",
    "firing_start_wait_p90_seconds",
    "firing_failure_share",
    "claim_conflicts",
    "coordination_calls",
    "merge_conflicts",
];

/// The KPIs `sim run` prints for the baseline and the best variant.
const RUN_KPIS: &[&str] = &[
    "tasks_arrived",
    "tasks_done",
    "tasks_correct",
    "false_completions_shipped",
    "rework_turns",
    "review_calls",
    "backlog_wait_p90_seconds",
    "worker_busy_share",
    "owner_wait_share",
    "questions_defaulted",
    "queue_wait_p90_seconds",
    "cost_usd",
    "firings",
    "firings_skipped",
    "firing_start_wait_p90_seconds",
    "firing_failure_share",
    "claim_conflicts",
    "coordination_calls",
    "merge_conflicts",
];

fn kpi(r: &Value, which: &str, name: &str) -> String {
    r[which]["kpis"][name]["value"]
        .as_f64()
        .map_or("-".into(), |v| format!("{v:.4}"))
}

/// `sim run <experiment> --out <dir> [--threads N]`: writes `result.json`, `index.html` and,
/// when the result has a best variant, `best.yaml` into `out`. Refusals of the architecture and
/// the KPIs of the baseline and the best variant go to `log`.
pub fn run(
    path: &Path,
    out: &Path,
    threads: Option<usize>,
    log: &mut dyn Write,
) -> Result<Value, Refusal> {
    let invalid = |e: String| refused("experiment_invalid", e);
    let threads = threads
        .filter(|&t| t > 0)
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get().min(8)));
    let started = Instant::now();
    let e = Experiment::load(path).map_err(invalid)?;
    for r in check_architecture(&nest::flatten(&e.architecture)) {
        let _ = writeln!(log, "agenticspec sim: refused: {r}");
    }
    let result = run_experiment(&e, threads).map_err(invalid)?;
    let mut text = serde_json::to_string_pretty(&result).map_err(|e| invalid(e.to_string()))?;
    text.push('\n');
    write(&out.join("result.json"), &text)?;
    write(
        &out.join("index.html"),
        &agenticspec_render::page(&result["architecture"], Some(&result)),
    )?;
    let source = e.doc["architecture"].as_str().and_then(|p| {
        std::fs::read_to_string(path.parent().unwrap_or(Path::new(".")).join(p)).ok()
    });
    let best = crate::best::yaml_of_result(&result, source.as_deref()).map_err(invalid)?;
    if let Some(best) = &best {
        write(&out.join("best.yaml"), best)?;
    }
    let variants = result["variants"].as_array().map_or(0, |v| v.len());
    let _ = writeln!(
        log,
        "agenticspec sim: {variants} variants x {} seeds in {:.1} s on {threads} threads; fitness baseline {} best {}",
        e.seeds,
        started.elapsed().as_secs_f64(),
        result["baseline"]["fitness"],
        result["best"]["fitness"],
    );
    for name in RUN_KPIS {
        let zero = |which| kpi(&result, which, name) == "0.0000";
        if TRIGGER_KPIS.contains(name) && zero("baseline") && zero("best") {
            continue;
        }
        let _ = writeln!(
            log,
            "  {name:<26} baseline {:>12} best {:>12}",
            kpi(&result, "baseline", name),
            kpi(&result, "best", name)
        );
    }
    let _ = writeln!(log, "  best params {}", result["best"]["params"]);
    Ok(json!({
        "out": out.display().to_string(),
        "variants": variants,
        "seeds": e.seeds,
        "threads": threads,
        "best_written": best.is_some(),
    }))
}

/// `sim render <architecture> [--result result.json] --out <page.html>`. A nested architecture is
/// drawn as one box; a reference that does not resolve is drawn as written.
pub fn render(path: &Path, result: Option<&Path>, out: &Path) -> Result<Value, Refusal> {
    let unreadable = |e: String| refused("document_unreadable", e);
    let arch = match nest::load(path) {
        Ok(a) => a,
        Err(nest::Error::Unreadable(e)) => return Err(unreadable(e)),
        Err(nest::Error::Refused(_)) => load_yaml(path).map_err(unreadable)?,
    };
    let result: Option<Value> = match result {
        Some(p) => {
            let text = std::fs::read_to_string(p)
                .map_err(|e| unreadable(format!("{}: {e}", p.display())))?;
            Some(
                serde_json::from_str(&text)
                    .map_err(|e| unreadable(format!("{}: {e}", p.display())))?,
            )
        }
        None => None,
    };
    write(out, &agenticspec_render::page(&arch, result.as_ref()))?;
    Ok(json!({ "out": out.display().to_string() }))
}

/// `sim ladder <ladder.yaml> --out <ladder.json> [--check]`: writes the data file of the ladder
/// page, or with `check` refuses a file that differs from a fresh generation.
pub fn ladder(path: &Path, out: &Path, check: bool) -> Result<Value, Refusal> {
    let (doc, dir) = ladder_doc::load(path).map_err(|e| refused("document_unreadable", e))?;
    let refusals = ladder_doc::check(&doc, &dir);
    if !refusals.is_empty() {
        let names: Vec<String> = refusals.iter().map(ToString::to_string).collect();
        return Err(refused("ladder_refused", names.join("; ")));
    }
    let text = ladder_doc::render(path).map_err(|e| refused("ladder_refused", e))?;
    let rungs = doc["rungs"].as_array().map_or(0, |r| r.len());
    let answer = |written: bool| json!({ "out": out.display().to_string(), "rungs": rungs, "written": written });
    if check {
        if std::fs::read_to_string(out).ok().as_deref() == Some(text.as_str()) {
            return Ok(answer(false));
        }
        return Err(refused(
            "ladder_stale",
            format!(
                "{} differs from a fresh generation of {}: run task ladder:generate",
                out.display(),
                path.display()
            ),
        ));
    }
    write(out, &text)?;
    Ok(answer(true))
}
