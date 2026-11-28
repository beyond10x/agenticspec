//! `agenticspec blocks lift` and `agenticspec blocks scenario`: the rows of one session in the
//! building blocks, checked against the blocks built into the simulator, and the draft scenario
//! of its outside inputs, checked against its architecture and profile before it is written.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use agenticspec_records::blocks::{self, DraftArgs, Lift};
use agenticspec_sim::doc::{load_yaml, parse_yaml};
use agenticspec_sim::reader::{Reader, sha256_hex};
use agenticspec_sim::{rows, scenario};
use serde_json::{Value, json};

/// A refused command: its code and detail.
pub type Refused = (&'static str, String);

fn lifted(transcript: &Path) -> Result<Lift, Refused> {
    blocks::lift_file(transcript).map_err(|e| {
        (
            "transcript_unreadable",
            format!("{}: {e}", transcript.display()),
        )
    })
}

/// Every problem of the lifted rows against the blocks; none is expected.
pub fn problems(lift: &Lift) -> Vec<String> {
    let mut out = Vec::new();
    for (i, row) in lift.rows.iter().enumerate() {
        let p = format!("rows.{i}");
        let mut r = Reader::default();
        match rows::read_row(&mut r, &row.value(), &p) {
            Some(read) => out.extend(rows::check_row(rows::blocks(), &read, &p)),
            None => out.extend(r.errors),
        }
    }
    out
}

/// `blocks lift <transcript>`.
pub fn lift(transcript: &Path) -> Result<Value, Refused> {
    let lift = lifted(transcript)?;
    let found = problems(&lift);
    if !found.is_empty() {
        return Err(("rows_invalid", found.join("; ")));
    }
    let mut counts: BTreeMap<&str, i64> = BTreeMap::new();
    for row in &lift.rows {
        *counts.entry(row.entity).or_default() += 1;
    }
    Ok(json!({
        "session_id": lift.session_id,
        "given": { "rows": lift.rows.iter().map(|r| r.value()).collect::<Vec<_>>() },
        "entities": blocks::ENTITIES.iter().map(|e| json!({
            "entity": e, "rows": counts.get(e).copied().unwrap_or(0)
        })).collect::<Vec<_>>(),
        "not_observable": lift.not_observable,
    }))
}

/// The parts of `--map`: `key=part` pairs separated by commas.
pub fn parse_map(text: &str) -> Result<BTreeMap<String, String>, String> {
    const KEYS: &[&str] = &["main", "sub", "human", "delegate", "peer"];
    let mut out = BTreeMap::new();
    for pair in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let Some((k, v)) = pair.split_once('=') else {
            return Err(format!("`{pair}` is no key=part pair"));
        };
        let (k, v) = (k.trim(), v.trim());
        if !KEYS.contains(&k) {
            return Err(format!(
                "`{k}` is no key; expected one of {}",
                KEYS.join(", ")
            ));
        }
        if v.is_empty() || out.insert(k.to_string(), v.to_string()).is_some() {
            return Err(format!("`{k}` is empty or given twice"));
        }
    }
    if !out.contains_key("main") {
        return Err("`main=<part>` is required".into());
    }
    Ok(out)
}

fn absolute(p: &Path) -> PathBuf {
    let p = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// `to` relative to the directory `from`, with `/` between the parts.
pub fn relative(from: &Path, to: &Path) -> String {
    let (from, to) = (absolute(from), absolute(to));
    let (a, b): (Vec<Component>, Vec<Component>) =
        (from.components().collect(), to.components().collect());
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<String> = vec!["..".to_string(); a.len() - common];
    parts.extend(
        b[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

fn read(path: &Path) -> Result<Vec<u8>, Refused> {
    std::fs::read(path).map_err(|e| ("document_unreadable", format!("{}: {e}", path.display())))
}

/// `blocks scenario <transcript> --architecture <a> --profile <p> --map main=<part> --out <file>`.
pub fn draft(input: &Value) -> Result<Value, Refused> {
    let field = |k: &str| input.get(k).and_then(Value::as_str).unwrap_or("");
    let transcript = PathBuf::from(field("transcript"));
    let (arch_path, profile_path, out) = (
        PathBuf::from(field("architecture")),
        PathBuf::from(field("profile")),
        PathBuf::from(field("out")),
    );
    let map = parse_map(field("map")).map_err(|e| ("map_invalid", e))?;
    let arch = load_yaml(&arch_path).map_err(|e| ("document_unreadable", e))?;
    let main = map["main"].clone();
    let acts_for = arch["parts"][main.as_str()]["acts_for"]
        .as_str()
        .map(str::to_string);
    let dir = out.parent().unwrap_or(Path::new("."));
    let name = input
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            let file = out.file_name().and_then(|n| n.to_str()).unwrap_or("draft");
            file.trim_end_matches(".yaml")
                .trim_end_matches(".scenario")
                .to_string()
        });
    let args = DraftArgs {
        name,
        architecture: (relative(dir, &arch_path), sha256_hex(&read(&arch_path)?)),
        profile: (
            relative(dir, &profile_path),
            sha256_hex(&read(&profile_path)?),
        ),
        main,
        sub: map.get("sub").cloned(),
        human: map.get("human").cloned().or(acts_for),
        delegate: map.get("delegate").cloned(),
        peer: map.get("peer").cloned(),
    };
    // The references are read relative to the directory of `--out`, which must exist for that.
    if !dir.as_os_str().is_empty() {
        std::fs::create_dir_all(dir)
            .map_err(|e| ("out_unwritable", format!("{}: {e}", dir.display())))?;
    }
    let lift = lifted(&transcript)?;
    let d = blocks::draft(&lift, &args);
    let doc = parse_yaml(&d.yaml).map_err(|e| ("scenario_refused", e))?;
    let parsed = scenario::parse(&doc).map_err(|e| ("scenario_refused", e.join("; ")))?;
    let loaded = scenario::load_with(&out, parsed).map_err(|r| (r.code, r.detail))?;
    let found = scenario::check(&loaded);
    if !found.is_empty() {
        return Err(("scenario_refused", found.join("; ")));
    }
    std::fs::write(&out, &d.yaml)
        .map_err(|e| ("out_unwritable", format!("{}: {e}", out.display())))?;
    Ok(json!({
        "out": out.display().to_string(),
        "scenario": args.name,
        "inputs": d.inputs as i64,
        "expectations": d.expectations as i64,
        "unjoined": d.unjoined,
        "unmapped": d.unmapped,
    }))
}
