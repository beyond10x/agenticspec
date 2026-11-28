//! The commands of `agenticspec sim`: check, run, render and ladder. Each returns the result
//! the command line prints, a JSON value of the shape `spec/domains/cli.yaml` declares
//! (`agenticspec.cli.Sim*Result`), or a [`Refusal`] with one of the error codes `spec/cli.yaml`
//! binds to the command. The parser and the process adapter are the generated
//! `agenticspec-cli-contract` of the `agenticspec` crate.

use crate::check::check as check_architecture;
use crate::doc::load_yaml;
use crate::{Experiment, ladder as ladder_doc, run as run_experiment};
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

/// `sim check <architecture>`: the architecture with its counts and the KPIs of the architecture as
/// a whole (`whole.rs`), or every refusal of the check.
pub fn check(path: &Path) -> Result<Value, Refusal> {
    let arch = load_yaml(path).map_err(|e| refused("document_unreadable", e))?;
    let refusals = check_architecture(&arch);
    if !refusals.is_empty() {
        let names: Vec<String> = refusals.iter().map(ToString::to_string).collect();
        return Err(refused("architecture_refused", names.join("; ")));
    }
    let kpis: serde_json::Map<String, Value> = crate::whole::measure(&arch)
        .into_iter()
        .map(|(k, v)| (k.to_string(), crate::doc::number(v)))
        .collect();
    Ok(json!({
        "architecture": arch["architecture"].as_str().unwrap_or("architecture"),
        "parts": arch["parts"].as_object().map_or(0, |p| p.len()),
        "links": arch["links"].as_array().map_or(0, |l| l.len()),
        "kpis": kpis,
        "single_points": crate::whole::single_point_parts(&arch),
    }))
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
    for r in check_architecture(&e.architecture) {
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

/// `sim render <architecture> [--result result.json] --out <page.html>`.
pub fn render(path: &Path, result: Option<&Path>, out: &Path) -> Result<Value, Refusal> {
    let unreadable = |e: String| refused("document_unreadable", e);
    let arch = load_yaml(path).map_err(unreadable)?;
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

/// `sim scenario check <scenario>`: the scenario checked against its architecture and profile,
/// or every problem; `sim scenario run <scenario> --out <dir>`: runs it, writes
/// `scenario-result.json` into `out`, and refuses with `expectation_failed` when an expectation
/// or a KPI bound failed.
pub fn scenario(action: &str, path: &Path, out: Option<&Path>) -> Result<Value, Refusal> {
    let l = crate::scenario::load_checked(path)?;
    let s = &l.scenario;
    let mut answer = json!({
        "scenario": s.name,
        "architecture": l.architecture["architecture"].as_str().unwrap_or("architecture"),
        "inputs": s.when.len(),
        "expectations": s.then.events.len() + s.then.kpis.len(),
    });
    match action {
        "check" => return Ok(answer),
        "run" => {}
        other => {
            return Err(refused(
                "scenario_invalid",
                format!("action `{other}`, expected check or run"),
            ));
        }
    }
    let Some(out) = out else {
        return Err(refused(
            "out_missing",
            "sim scenario run writes scenario-result.json into the directory --out names",
        ));
    };
    let ran = crate::scenario::run(&l)?;
    let doc = crate::scenario::result_json(&l, &ran);
    let mut text =
        serde_json::to_string_pretty(&doc).map_err(|e| refused("out_unwritable", e.to_string()))?;
    text.push('\n');
    let file = out.join("scenario-result.json");
    write(&file, &text)?;
    use crate::scenario::Status;
    let (held, failed, not_checked) = (
        ran.count(Status::Held),
        ran.count(Status::Failed),
        ran.count(Status::NotChecked),
    );
    if failed > 0 {
        return Err(refused(
            "expectation_failed",
            format!(
                "{}: {failed} failed, {held} held, {not_checked} not checked; see {}",
                s.name,
                file.display()
            ),
        ));
    }
    answer["out"] = json!(out.display().to_string());
    answer["verdict"] = json!("held");
    answer["predicted"] = json!(ran.predicted.name());
    answer["held"] = json!(held);
    answer["failed"] = json!(failed);
    answer["not_checked"] = json!(not_checked);
    Ok(answer)
}

/// `sim corpus run <corpus>`: runs every entry, prints the table to `log`, and refuses with
/// `corpus_failed` when an entry's expectations failed or it predicts another class than it
/// expects.
pub fn corpus(action: &str, path: &Path, log: &mut dyn Write) -> Result<Value, Refusal> {
    if action != "run" {
        return Err(refused(
            "corpus_invalid",
            format!("action `{action}`, expected run"),
        ));
    }
    let (c, rows) = crate::corpus::run(path)?;
    let _ = write!(log, "{}", crate::corpus::table(&rows));
    let failed: Vec<&str> = rows
        .iter()
        .filter(|r| !r.holds())
        .map(|r| r.entry.as_str())
        .collect();
    if !failed.is_empty() {
        return Err(refused(
            "corpus_failed",
            format!(
                "{}: {} of {} entries failed: {}",
                c.name,
                failed.len(),
                rows.len(),
                failed.join(", ")
            ),
        ));
    }
    Ok(json!({
        "corpus": c.name,
        "entries": rows.len(),
        "held": rows.len(),
        "failed": 0,
        "rows": rows.iter().map(crate::corpus::Row::json).collect::<Vec<_>>(),
    }))
}

/// The options of `sim ports`.
#[derive(Debug, Clone, Default)]
pub struct PortsOptions<'a> {
    /// The blocks directory; `blocks` when absent.
    pub blocks: Option<&'a Path>,
    pub out: Option<&'a Path>,
    pub check: Option<&'a Path>,
    pub expect: Option<&'a Path>,
}

/// `sim ports [--blocks <dir>] [--out <file>] [--check <file>] [--expect <file>]`: derive the
/// ports of the building blocks and write them (or print the file), compare a ports file's
/// digest with its sources (`--check`, no derivation and no `ess` call), or list every difference
/// from an expected list (`--expect`: one line per difference to `log`, refused as
/// `ports_differ` when there is one).
pub fn ports(o: &PortsOptions, log: &mut dyn Write) -> Result<Value, Refusal> {
    use crate::contract::ports as p;
    let blocks = o.blocks.unwrap_or(Path::new("blocks"));
    if let Some(file) = o.check {
        let have = p::PortsFile::load(file).map_err(|e| refused("document_unreadable", e))?;
        let digest =
            p::sources_digest(blocks, &have.ess).map_err(|e| refused("blocks_unreadable", e))?;
        if digest != have.sources_sha256 {
            return Err(refused(
                "ports_stale",
                format!(
                    "{} was derived from other sources than {}: run task ports:generate",
                    file.display(),
                    blocks.display()
                ),
            ));
        }
        return Ok(json!({
            "ports": have.ports.len(),
            "sources_sha256": digest,
            "written": false,
            "differences": [],
        }));
    }
    let unreadable = |e: String| refused("blocks_unreadable", e);
    let ess = p::ess_version().map_err(unreadable)?;
    let digest = p::sources_digest(blocks, &ess).map_err(unreadable)?;
    let ir = p::compile(blocks).map_err(unreadable)?;
    let derived = p::derive(&ir).map_err(|e| refused("ports_refused", e.join("; ")))?;
    if let Some(expect) = o.expect {
        let rows = p::load_expected(expect).map_err(|e| refused("document_unreadable", e))?;
        let lines = p::compare(&derived.ports, &rows);
        for line in &lines {
            let _ = writeln!(log, "{line}");
        }
        if !lines.is_empty() {
            return Err(refused(
                "ports_differ",
                format!("{} difference(s) from {}", lines.len(), expect.display()),
            ));
        }
        return Ok(json!({
            "ports": derived.ports.len(),
            "sources_sha256": digest,
            "written": false,
            "differences": [],
        }));
    }
    let text = p::file_text(&derived.ports, &derived.bindings, &ess, &digest);
    let mut answer = json!({
        "ports": derived.ports.len(),
        "sources_sha256": digest,
        "written": o.out.is_some(),
        "differences": [],
    });
    match o.out {
        Some(out) => write(out, &text)?,
        None => answer["file"] = json!(text),
    }
    Ok(answer)
}

/// The options of `sim contract`.
#[derive(Debug, Clone)]
pub struct ContractOptions<'a> {
    pub architecture: &'a Path,
    /// The ports file; `blocks/ports.json` when absent.
    pub ports: Option<&'a Path>,
    pub out: Option<&'a Path>,
    /// A written contract to check against the document and the ports, without deriving.
    pub check: Option<&'a Path>,
}

/// `sim contract <architecture> [--ports <file>] [--out <file>] [--check <file>]`: derive the
/// contract of an architecture document and check it, then write it (or print it); refused,
/// writing nothing, when the document fails `sim check`. With `--check`, read a written contract
/// and check it against the document and the ports instead.
pub fn contract(o: &ContractOptions) -> Result<Value, Refusal> {
    use crate::contract::derive as d;
    use crate::contract::ports::PortsFile;
    let arch = load_yaml(o.architecture).map_err(|e| refused("document_unreadable", e))?;
    let refusals = check_architecture(&arch);
    if !refusals.is_empty() {
        let names: Vec<String> = refusals.iter().map(ToString::to_string).collect();
        return Err(refused("architecture_refused", names.join("; ")));
    }
    let ports_path = o.ports.unwrap_or(Path::new("blocks/ports.json"));
    let ports = PortsFile::load(ports_path).map_err(|e| refused("ports_unreadable", e))?;
    let contract = match o.check {
        Some(file) => {
            let text = std::fs::read_to_string(file)
                .map_err(|e| refused("document_unreadable", format!("{}: {e}", file.display())))?;
            d::Contract::parse(&text)
                .map_err(|e| refused("contract_refused", format!("{}: {e}", file.display())))?
        }
        None => d::derive(&arch, &ports).map_err(|r| refused("contract_refused", r.join("; ")))?,
    };
    let problems = d::check(&contract, &arch, &ports);
    if !problems.is_empty() {
        return Err(refused("contract_refused", problems.join("; ")));
    }
    let mut answer = json!({
        "architecture": contract.architecture,
        "ports": contract.ports.len(),
        "invariants": contract.invariants.len(),
        "written": false,
    });
    if o.check.is_some() {
        return Ok(answer);
    }
    let text = contract.text();
    match o.out {
        Some(out) => {
            write(out, &text)?;
            answer["written"] = json!(true);
        }
        None => answer["contract"] = json!(text),
    }
    Ok(answer)
}
