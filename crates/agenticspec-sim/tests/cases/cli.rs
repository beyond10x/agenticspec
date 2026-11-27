//! The command line: check, run and render.

use super::common::{ARCH, PROFILE, experiment_yaml, repo_root, scratch};
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_agenticspec-sim"))
}

#[test]
fn check_accepts_the_workstation() {
    let out = bin()
        .arg("check")
        .arg(repo_root().join("architectures/workstation.yaml"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn check_exits_1_and_names_each_refusal() {
    let dir = scratch("check-refused");
    let broken = ARCH.replace(
        "to: builds,   carries: claim",
        "to: build,   carries: claim",
    );
    std::fs::write(dir.join("broken.yaml"), broken).unwrap();
    let out = bin()
        .arg("check")
        .arg(dir.join("broken.yaml"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("unknown_link_parts"), "{stdout}");
    assert!(stdout.contains("build"), "{stdout}");
}

#[test]
fn run_writes_the_result_and_the_page() {
    let dir = scratch("run");
    std::fs::write(dir.join("small.yaml"), ARCH).unwrap();
    std::fs::write(dir.join("small-profile.yaml"), PROFILE).unwrap();
    std::fs::write(dir.join("exp.yaml"), experiment_yaml("fit", 4, 256)).unwrap();
    let out_dir = dir.join("out");
    let out = bin()
        .args(["run"])
        .arg(dir.join("exp.yaml"))
        .arg("--out")
        .arg(&out_dir)
        .args(["--threads", "2"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let result: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out_dir.join("result.json")).unwrap())
            .unwrap();
    assert_eq!(result["format"], "agenticspec-sim-result/1");
    let page = std::fs::read_to_string(out_dir.join("index.html")).unwrap();
    assert!(page.contains("<html"), "{page}");
}

#[test]
fn render_writes_a_page_for_an_architecture() {
    let dir = scratch("render");
    let page = dir.join("page.html");
    let out = bin()
        .arg("render")
        .arg(repo_root().join("architectures/workstation.yaml"))
        .arg("--out")
        .arg(&page)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(std::fs::read_to_string(&page).unwrap().contains("<html"));
}

#[test]
fn an_unknown_command_exits_2() {
    let out = bin().arg("fly").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
