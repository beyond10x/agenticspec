//! Rows of the building blocks in a scenario's `given`: the declarations built in, and the check
//! of each row against them.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli;
use agenticspec_sim::reader::sha256_hex;
use agenticspec_sim::rows::{SOURCES, blocks};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn doc_ref(path: &Path) -> String {
    format!(
        "{{ ref: {}, digest: {} }}",
        path.display(),
        sha256_hex(&std::fs::read(path).unwrap())
    )
}

/// A scenario of the workstation with one prompt and the given rows, written into `dir`.
fn with_rows(dir: &Path, name: &str, rows: &str) -> PathBuf {
    let text = format!(
        "format: agenticspec-scenario/1\nscenario: {name}\narchitecture: {}\nprofile: {}\ngiven:\n  rows:\n{rows}when:\n  - {{ at: operator, kind: prompt, author: owner }}\n",
        doc_ref(&repo_root().join("architectures/workstation.yaml")),
        doc_ref(&repo_root().join("profiles/coding.yaml")),
    );
    let path = dir.join(format!("{name}.scenario.yaml"));
    std::fs::write(&path, text).unwrap();
    path
}

const POOL: &str = "00000000-0000-4000-8000-000000000001";
const HOLDER: &str = "00000000-0000-4000-8000-000000000002";
const CLAIM: &str = "00000000-0000-4000-8000-000000000003";

#[test]
fn the_block_files_built_in_are_every_block_file_as_it_is() {
    let mut on_disk = BTreeSet::new();
    for layer in std::fs::read_dir(repo_root().join("blocks")).unwrap() {
        let layer = layer.unwrap().path();
        if !layer.is_dir() || layer.ends_with("scenarios") {
            continue;
        }
        for f in std::fs::read_dir(&layer).unwrap() {
            let f = f.unwrap().path();
            let rel = f.strip_prefix(repo_root().join("blocks")).unwrap();
            on_disk.insert(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    let built: BTreeSet<String> = SOURCES.iter().map(|(p, _)| p.to_string()).collect();
    assert_eq!(built, on_disk, "rows::SOURCES lists every file of blocks/");
    for (p, text) in SOURCES {
        let disk = std::fs::read_to_string(repo_root().join("blocks").join(p)).unwrap();
        assert_eq!(*text, disk, "{p} as built in");
    }
    let b = blocks();
    for e in [
        "agentic.work.Task",
        "agentic.work.Run",
        "agentic.work.Step",
        "agentic.question.Question",
        "agentic.channel.Message",
        "agentic.resource.Claim",
    ] {
        assert!(b.entities.contains_key(e), "{e}");
    }
    let message = &b.entities["agentic.channel.Message"];
    assert!(
        message
            .fields
            .iter()
            .any(|(n, t)| n == "received_in_turn" && t == "Optional<agentic.shared.TurnId>"),
        "a message names the turn that received it"
    );
}

#[test]
fn a_given_row_with_its_fields_or_their_absence_named_passes_the_check() {
    let dir = scratch("rows-ok");
    let rows = format!(
        "    - {{ entity: agentic.resource.Claim, id: {CLAIM}, state: Held, evidence: \"s#4\",\n        fields: {{ holder: {HOLDER}, time_limit_seconds: 600, queued_at: \"2026-11-25T09:00:00.000Z\" }},\n        not_recorded: [pool_id] }}\n    - {{ entity: agentic.resource.Pool, id: {POOL}, not_recorded: [name, kind, capacity, held] }}\n"
    );
    let path = with_rows(&dir, "rows-ok", &rows);
    let r = cli::scenario("check", &path, None).unwrap();
    assert_eq!(r["inputs"], 1);
}

#[test]
fn a_given_row_is_refused_by_the_path_of_each_problem() {
    let dir = scratch("rows-refused");
    let rows = format!(
        "    - {{ entity: agentic.resource.Claim, id: {CLAIM}, state: Waiting,\n        fields: {{ holder: {HOLDER}, time_limit_seconds: \"long\", queued_at: \"2026-11-25T09:00:00.000Z\", colour: red }} }}\n    - {{ entity: agentic.resource.Claimm, id: {POOL} }}\n    - {{ entity: agentic.work.Step, id: not-a-uuid, not_recorded: [run_id, action, harness_session_id, result, recorded_at, wish], fields: {{ kind: Robot }} }}\n    - {{ entity: agentic.resource.Claim, id: {CLAIM}, fields: {{ pool_id: {POOL}, holder: {HOLDER}, time_limit_seconds: 1, queued_at: x }}, not_recorded: [pool_id] }}\n"
    );
    let path = with_rows(&dir, "rows-refused", &rows);
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "scenario_refused");
    for p in [
        "given.rows.0.state: `Waiting` is no state of agentic.resource.Claim",
        "given.rows.0.fields.time_limit_seconds: \"long\" is not of type Integer",
        "given.rows.0.fields.colour: no field of agentic.resource.Claim",
        "given.rows.0.fields.pool_id: missing, agentic.resource.Claim requires it",
        "given.rows.1.entity: `agentic.resource.Claimm` is no entity of the building blocks",
        "given.rows.2.id: \"not-a-uuid\" is not of type Uuid",
        "given.rows.2.fields.kind: \"Robot\" is not one of agentic.work.StepKind",
        "given.rows.2.not_recorded.5: `wish` is no field of agentic.work.Step",
        "given.rows.3.id: agentic.resource.Claim `00000000-0000-4000-8000-000000000003` is given twice",
        "given.rows.3.not_recorded.0: `pool_id` is given in fields as well",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }
}

#[test]
fn an_unknown_key_of_a_row_is_refused_by_its_path() {
    let dir = scratch("rows-key");
    let rows = format!("    - {{ entity: agentic.resource.Pool, id: {POOL}, colour: red }}\n");
    let path = with_rows(&dir, "rows-key", &rows);
    let err = cli::scenario("check", &path, None).unwrap_err();
    assert_eq!(err.code, "scenario_invalid");
    assert!(
        err.detail.contains("given.rows.0.colour: unknown key"),
        "{}",
        err.detail
    );
}
