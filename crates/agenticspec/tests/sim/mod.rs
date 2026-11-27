//! `agenticspec sim`: the simulator's commands on the generated command line. What each command
//! computes is tested in the `agenticspec-sim` crate.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("sim")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sim(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .arg("sim")
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
fn check_prints_the_accepted_architecture() {
    let arch = repo().join("architectures/workstation.yaml");
    let accepted = result(&sim(&["check".as_ref(), arch.as_os_str()]));
    assert_eq!(accepted["architecture"], "workstation");
}

#[test]
fn check_exits_1_and_names_the_refusal() {
    let dir = scratch("check-refused");
    let arch = dir.join("broken.yaml");
    std::fs::write(
        &arch,
        "format: agenticspec-architecture/1\narchitecture: broken\nparts:\n  you: { block: principal, kind: human }\nlinks:\n  - { from: you, to: nobody, carries: prompt }\n",
    )
    .unwrap();
    let out = sim(&["check".as_ref(), arch.as_os_str()]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("architecture_refused: "), "{stderr}");
    assert!(stderr.contains("nobody"), "{stderr}");
}

#[test]
fn run_writes_the_result_with_the_threads_given() {
    let dir = scratch("run");
    let exp = dir.join("exp.yaml");
    std::fs::write(
        &exp,
        format!(
            "format: agenticspec-experiment/1\nexperiment: cli-run\narchitecture: {}\nprofile: {}\nhorizon: 1h\nseeds: 1\nseed: 1\noptimize: marked\nsearch:\n  mode: fit\n  budget: 2\n  restarts: 1\n  weights:\n    tasks_done: 1\n",
            repo().join("architectures/workstation.yaml").display(),
            repo().join("profiles/coding.yaml").display(),
        ),
    )
    .unwrap();
    let out_dir = dir.join("out");
    let ran = result(&sim(&[
        "run".as_ref(),
        exp.as_os_str(),
        "--out".as_ref(),
        out_dir.as_os_str(),
        "--threads".as_ref(),
        "1".as_ref(),
    ]));
    assert_eq!(ran["threads"], 1);
    assert_eq!(ran["seeds"], 1);
    assert!(out_dir.join("result.json").is_file());
    assert!(out_dir.join("index.html").is_file());
}

#[test]
fn render_writes_the_page() {
    let page = scratch("render").join("page.html");
    let arch = repo().join("architectures/workstation.yaml");
    let rendered = result(&sim(&[
        "render".as_ref(),
        arch.as_os_str(),
        "--out".as_ref(),
        page.as_os_str(),
    ]));
    assert_eq!(rendered["out"], page.display().to_string());
    assert!(page.is_file());
}

#[test]
fn ladder_check_accepts_the_committed_data_file_and_refuses_a_missing_one() {
    let current = result(&sim(&[
        "ladder".as_ref(),
        "ladder/ladder.yaml".as_ref(),
        "--out".as_ref(),
        "website/static/ladder/ladder.json".as_ref(),
        "--check".as_ref(),
    ]));
    assert_eq!(current["written"], false);
    let missing = scratch("ladder").join("ladder.json");
    let out = sim(&[
        "ladder".as_ref(),
        "ladder/ladder.yaml".as_ref(),
        "--out".as_ref(),
        missing.as_os_str(),
        "--check".as_ref(),
    ]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ladder_stale"), "{stderr}");
    assert!(stderr.contains("ladder:generate"), "{stderr}");
}
