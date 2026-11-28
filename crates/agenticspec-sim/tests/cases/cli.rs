//! The commands of `agenticspec sim`: check, run and render. The command line itself is
//! tested in the `agenticspec` crate.

use super::common::{ARCH, PROFILE, experiment_yaml, repo_root, scratch};
use agenticspec_sim::cli;

#[test]
fn check_accepts_the_workstation() {
    let accepted = cli::check(&repo_root().join("architectures/workstation.yaml")).unwrap();
    assert_eq!(accepted["architecture"], "workstation");
    assert!(accepted["parts"].as_u64().unwrap() > 0, "{accepted}");
    assert!(accepted["links"].as_u64().unwrap() > 0, "{accepted}");
}

#[test]
fn check_names_each_refusal() {
    let dir = scratch("check-refused");
    let broken = ARCH.replace(
        "to: builds,   carries: claim",
        "to: build,   carries: claim",
    );
    std::fs::write(dir.join("broken.yaml"), broken).unwrap();
    let err = cli::check(&dir.join("broken.yaml")).unwrap_err();
    assert_eq!(err.code, "architecture_refused");
    assert!(err.detail.contains("unknown_link_parts"), "{}", err.detail);
    assert!(err.detail.contains("build"), "{}", err.detail);
}

#[test]
fn check_refuses_a_file_it_cannot_read() {
    let dir = scratch("check-missing");
    let err = cli::check(&dir.join("missing.yaml")).unwrap_err();
    assert_eq!(err.code, "document_unreadable");
}

#[test]
fn run_writes_the_result_and_the_page() {
    let dir = scratch("run");
    std::fs::write(dir.join("small.yaml"), ARCH).unwrap();
    std::fs::write(dir.join("small-profile.yaml"), PROFILE).unwrap();
    std::fs::write(dir.join("exp.yaml"), experiment_yaml("fit", 4, 256)).unwrap();
    let out_dir = dir.join("out");
    let mut log = Vec::new();
    let ran = cli::run(&dir.join("exp.yaml"), &out_dir, Some(2), false, &mut log)
        .unwrap_or_else(|e| panic!("{e:?}\n{}", String::from_utf8_lossy(&log)));
    assert_eq!(ran["threads"], 2);
    assert_eq!(ran["seeds"], 2);
    let log = String::from_utf8_lossy(&log);
    assert!(log.contains("tasks_done"), "{log}");
    let result: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out_dir.join("result.json")).unwrap())
            .unwrap();
    assert_eq!(result["format"], "agenticspec-sim-result/1");
    assert_eq!(
        ran["variants"].as_u64(),
        result["variants"].as_array().map(|v| v.len() as u64)
    );
    let page = std::fs::read_to_string(out_dir.join("index.html")).unwrap();
    assert!(page.contains("<html"), "{page}");
}

#[test]
fn run_refuses_an_experiment_it_cannot_load() {
    let dir = scratch("run-missing");
    let err = cli::run(
        &dir.join("missing.yaml"),
        &dir.join("out"),
        None,
        false,
        &mut Vec::new(),
    )
    .unwrap_err();
    assert_eq!(err.code, "experiment_invalid");
}

#[test]
fn render_writes_a_page_for_an_architecture() {
    let dir = scratch("render");
    let page = dir.join("page.html");
    let rendered = cli::render(
        &repo_root().join("architectures/workstation.yaml"),
        None,
        &page,
    )
    .unwrap();
    assert_eq!(rendered["out"], page.display().to_string());
    assert!(std::fs::read_to_string(&page).unwrap().contains("<html"));
}
