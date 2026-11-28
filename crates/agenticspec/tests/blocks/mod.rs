//! `agenticspec blocks lift` and `blocks scenario` on the generated command line. What the lift
//! and the draft compute is tested in the `agenticspec-records` crate.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> PathBuf {
    repo().join(format!(
        "crates/agenticspec-records/tests/fixtures/blocks/{name}.jsonl"
    ))
}

fn blocks(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .arg("blocks")
        .args(args)
        .current_dir(repo())
        .output()
        .unwrap()
}

fn result(out: &Output) -> Value {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn lift_prints_the_rows_and_what_is_not_observable() {
    let r = result(&blocks(&[
        "lift".as_ref(),
        fixture("lift-demo").as_os_str(),
    ]));
    assert_eq!(r["session_id"], "lift-demo");
    assert_eq!(
        r["not_observable"],
        serde_json::json!(["agentic.resource.Claim"])
    );
    let count = |e: &str| {
        r["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["entity"] == e)
            .unwrap()["rows"]
            .as_i64()
            .unwrap()
    };
    assert_eq!(count("agentic.work.Task"), 1);
    assert_eq!(count("agentic.work.Run"), 2);
    assert!(r["given"]["rows"].as_array().unwrap().len() > 5);
}

#[test]
fn scenario_writes_a_draft_the_simulator_accepts() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("blocks-scenario");
    let _ = std::fs::remove_dir_all(&dir);
    let out = dir.join("drafts/lift-demo.scenario.yaml");
    let r = result(&blocks(&[
        "scenario".as_ref(),
        fixture("lift-demo").as_os_str(),
        "--architecture".as_ref(),
        "architectures/workstation.yaml".as_ref(),
        "--profile".as_ref(),
        "fields/coding/profiles/coding.yaml".as_ref(),
        "--map".as_ref(),
        "main=operator,sub=worker".as_ref(),
        "--out".as_ref(),
        out.as_os_str(),
    ]));
    assert_eq!(r["scenario"], "lift-demo");
    assert_eq!(r["inputs"], 2);
    assert_eq!(r["unjoined"], serde_json::json!(["when.1 (lift-demo#28)"]));
    let checked = Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .args(["sim", "scenario", "check"])
        .arg(&out)
        .output()
        .unwrap();
    assert_eq!(result(&checked)["inputs"], 2);

    let refused = blocks(&[
        "scenario".as_ref(),
        fixture("lift-demo").as_os_str(),
        "--architecture".as_ref(),
        "architectures/workstation.yaml".as_ref(),
        "--profile".as_ref(),
        "fields/coding/profiles/coding.yaml".as_ref(),
        "--map".as_ref(),
        "main=worker,human=owner".as_ref(),
        "--out".as_ref(),
        dir.join("refused.scenario.yaml").as_os_str(),
    ]);
    assert_eq!(refused.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(stderr.starts_with("scenario_refused: "), "{stderr}");
    assert!(stderr.contains("is not a border part"), "{stderr}");
    assert!(!dir.join("refused.scenario.yaml").exists());
}
