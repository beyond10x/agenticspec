//! `agenticspec ui conform` against a runtime's adapter that is not `ui serve`: the test source
//! of the `agenticspec-ui` crate, which does not serve Reach, Revisions and Contracts and accepts a
//! write only from its own origin with a CSRF header. One source for the test.

use std::process::{Command, Output};

use agenticspec_ui::{operator, plane::Plane, serve};
use serde_json::Value;

use super::repo;

#[path = "../../../agenticspec-ui/tests/serve/partial.rs"]
mod partial;

const TOKEN: &str = "ui-conform-test";
const CONTRACT: &str = "ui/views/examples/workstation.contract.yaml";
const ARCH: &str = "architectures/workstation.yaml";

fn plane() -> Plane {
    let root = repo();
    let contract = operator::read_doc(&root.join(CONTRACT)).unwrap();
    let files = operator::build_with(
        &root,
        &root.join(ARCH),
        &contract,
        "workstation.contract.yaml",
        serve::ROOT,
    )
    .unwrap();
    Plane::from_files(&files).unwrap()
}

fn conform(url: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .current_dir(repo())
        .args([
            "ui",
            "conform",
            "--architecture",
            ARCH,
            "--contract",
            CONTRACT,
        ])
        .args(["--source", url])
        .args(extra)
        .env("AGENTICSPEC_UI_TOKEN", TOKEN)
        .output()
        .unwrap()
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Without the proof the write is refused (403) and the text output lists every check on a line
/// of its own, passed, failed and skipped; with `--header` the command R5 allows is carried out,
/// the views not served are skipped by name and the run passes; `--json` prints the report also
/// when a check fails (here by another `--origin`); a malformed header or origin is refused.
#[test]
fn ui_conform_sends_origin_and_headers_on_writes_and_reports_every_check() {
    let plane = plane();
    let command = plane.commands.iter().next().unwrap().clone();
    let src = partial::Partial::start(partial::Config {
        architecture: plane.architecture.clone(),
        token: TOKEN.into(),
        not_served: [
            "agentic.architecture.Reach",
            "agentic.architecture.Revisions",
            "agentic.architecture.Contracts",
        ]
        .iter()
        .map(ToString::to_string)
        .collect(),
        proof: Some(("X-CSRF-Token".into(), "csrf-test".into())),
        commands: vec![command.clone()],
        ..partial::Config::default()
    });

    let out = conform(src.url(), &[]);
    assert!(!out.status.success());
    let stdout = text(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() > 20, "{stdout}");
    assert!(
        lines.iter().all(|l| ["passed ", "failed ", "skipped "]
            .iter()
            .any(|w| l.starts_with(w))),
        "{stdout}"
    );
    assert!(
        lines.contains(
            &"skipped R3 agentic.architecture.Reach at workstation: the source does not serve agentic.architecture.Reach (404 view_not_served)"
        ),
        "{stdout}"
    );
    let refused = format!("failed R4 allowed {command} at workstation: ");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with(&refused) && l.contains("answered 403")),
        "{stdout}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("passed R2 resume level")),
        "{stdout}"
    );
    let stderr = text(&out.stderr);
    assert!(stderr.starts_with("conformance_failed"), "{stderr}");
    assert!(stderr.contains(" passed, "), "{stderr}");

    let out = conform(
        src.url(),
        &["--header", "X-CSRF-Token: csrf-test", "--json"],
    );
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["failed"], 0, "{report}");
    let checks = report["checks"].as_array().unwrap();
    let status = |name: &str| {
        checks
            .iter()
            .find(|c| c["name"] == name)
            .map(|c| c["status"].as_str().unwrap().to_string())
            .unwrap_or_else(|| panic!("no {name} in {report}"))
    };
    assert_eq!(
        status(&format!("R4 allowed {command} at workstation")),
        "passed"
    );
    assert_eq!(
        status("R3 agentic.architecture.Contracts at workstation"),
        "skipped"
    );

    let out = conform(
        src.url(),
        &[
            "--header",
            "X-CSRF-Token: csrf-test",
            "--origin",
            "http://elsewhere.test",
            "--json",
        ],
    );
    assert!(!out.status.success());
    let report: Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", text(&out.stdout)));
    assert!(report["failed"].as_u64().unwrap() > 0, "{report}");
    assert!(report["passed"].as_u64().unwrap() > 20, "{report}");
    assert!(report["skipped"].as_u64().unwrap() >= 3, "{report}");
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["detail"].is_string()
                && ["passed", "failed", "skipped"].contains(&c["status"].as_str().unwrap())),
        "{report}"
    );
    assert!(text(&out.stderr).contains("conformance_failed"));

    let out = conform(src.url(), &["--header", "no colon"]);
    assert!(
        text(&out.stderr).starts_with("header_invalid"),
        "{}",
        text(&out.stderr)
    );
    let out = conform(src.url(), &["--origin", "elsewhere.test"]);
    assert!(
        text(&out.stderr).starts_with("origin_invalid"),
        "{}",
        text(&out.stderr)
    );
}
