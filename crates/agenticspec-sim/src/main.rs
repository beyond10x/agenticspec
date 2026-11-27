//! `agenticspec-sim check | run | render | ladder`.

use agenticspec_sim::check::check;
use agenticspec_sim::doc::load_yaml;
use agenticspec_sim::ladder;
use agenticspec_sim::{Experiment, run};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage:
  agenticspec-sim check <architecture.yaml>
  agenticspec-sim run <experiment.yaml> --out <dir> [--threads N]
  agenticspec-sim render <architecture.yaml> [--result result.json] --out <page.html>
  agenticspec-sim ladder <ladder.yaml> --out <ladder.json> [--check]";

struct Args {
    positional: Vec<String>,
    out: Option<PathBuf>,
    result: Option<PathBuf>,
    threads: Option<usize>,
    check: bool,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args {
        positional: Vec::new(),
        out: None,
        result: None,
        threads: None,
        check: false,
    };
    let mut it = args.iter();
    while let Some(x) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{x} needs a value"));
        match x.as_str() {
            "--out" => a.out = Some(PathBuf::from(value()?)),
            "--result" => a.result = Some(PathBuf::from(value()?)),
            "--check" => a.check = true,
            "--threads" => {
                a.threads = Some(
                    value()?
                        .parse()
                        .map_err(|_| "--threads needs a number".to_string())?,
                )
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ => a.positional.push(x.clone()),
        }
    }
    Ok(a)
}

fn one(a: &Args) -> Result<&Path, String> {
    match a.positional.as_slice() {
        [p] => Ok(Path::new(p)),
        _ => Err("expected one file".into()),
    }
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

fn cmd_check(a: &Args) -> Result<ExitCode, String> {
    let arch = load_yaml(one(a)?)?;
    let refusals = check(&arch);
    for r in &refusals {
        println!("refused: {r}");
    }
    if refusals.is_empty() {
        let parts = arch["parts"].as_object().map_or(0, |p| p.len());
        let links = arch["links"].as_array().map_or(0, |l| l.len());
        println!(
            "accepted: {} ({parts} parts, {links} links)",
            arch["architecture"].as_str().unwrap_or("architecture")
        );
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
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

fn kpi(r: &Value, which: &str, name: &str) -> String {
    r[which]["kpis"][name]["value"]
        .as_f64()
        .map_or("-".into(), |v| format!("{v:.4}"))
}

fn cmd_run(a: &Args) -> Result<ExitCode, String> {
    let path = one(a)?;
    let out = a.out.as_ref().ok_or("run needs --out <dir>")?;
    let threads = a
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get().min(8)));
    let started = Instant::now();
    let e = Experiment::load(path)?;
    for r in check(&e.architecture) {
        eprintln!("agenticspec-sim: refused: {r}");
    }
    let result = run(&e, threads)?;
    let mut json = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?;
    json.push('\n');
    write(&out.join("result.json"), &json)?;
    write(
        &out.join("index.html"),
        &agenticspec_render::page(&result["architecture"], Some(&result)),
    )?;
    let n = result["variants"].as_array().map_or(0, |v| v.len());
    eprintln!(
        "agenticspec-sim: {n} variants x {} seeds in {:.1} s on {threads} threads; fitness baseline {} best {}",
        e.seeds,
        started.elapsed().as_secs_f64(),
        result["baseline"]["fitness"],
        result["best"]["fitness"],
    );
    for name in [
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
    ] {
        let zero = |which| kpi(&result, which, name) == "0.0000";
        if TRIGGER_KPIS.contains(&name) && zero("baseline") && zero("best") {
            continue;
        }
        eprintln!(
            "  {name:<26} baseline {:>12} best {:>12}",
            kpi(&result, "baseline", name),
            kpi(&result, "best", name)
        );
    }
    eprintln!("  best params {}", result["best"]["params"]);
    Ok(ExitCode::SUCCESS)
}

fn cmd_render(a: &Args) -> Result<ExitCode, String> {
    let arch = load_yaml(one(a)?)?;
    let out = a.out.as_ref().ok_or("render needs --out <page.html>")?;
    let result: Option<Value> = match &a.result {
        Some(p) => {
            let text = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
            Some(serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?)
        }
        None => None,
    };
    write(out, &agenticspec_render::page(&arch, result.as_ref()))?;
    Ok(ExitCode::SUCCESS)
}

fn cmd_ladder(a: &Args) -> Result<ExitCode, String> {
    let path = one(a)?;
    let out = a.out.as_ref().ok_or("ladder needs --out <ladder.json>")?;
    let (doc, dir) = ladder::load(path)?;
    let refusals = ladder::check(&doc, &dir);
    for r in &refusals {
        println!("refused: {r}");
    }
    if !refusals.is_empty() {
        return Ok(ExitCode::from(1));
    }
    let text = ladder::render(path)?;
    if a.check {
        if std::fs::read_to_string(out).ok().as_deref() == Some(text.as_str()) {
            println!("ladder: {} is current", out.display());
            return Ok(ExitCode::SUCCESS);
        }
        eprintln!(
            "agenticspec-sim: {} differs from a fresh generation of {}: run task ladder:generate",
            out.display(),
            path.display()
        );
        return Ok(ExitCode::from(1));
    }
    write(out, &text)?;
    let rungs = doc["rungs"].as_array().map_or(0, |r| r.len());
    println!("ladder: wrote {} ({rungs} rungs)", out.display());
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let parsed = match parse(rest) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("agenticspec-sim: {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let outcome = match command.as_str() {
        "check" => cmd_check(&parsed),
        "run" => cmd_run(&parsed),
        "render" => cmd_render(&parsed),
        "ladder" => cmd_ladder(&parsed),
        "--help" | "-h" | "help" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        other => Err(format!("unknown command `{other}`\n{USAGE}")),
    };
    match outcome {
        Ok(code) => code,
        Err(e) => {
            eprintln!("agenticspec-sim: {e}");
            ExitCode::from(2)
        }
    }
}
