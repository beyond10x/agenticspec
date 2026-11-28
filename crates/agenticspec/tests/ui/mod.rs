//! `agenticspec ui conform`: the conformance suite of the runtime interface on the generated
//! command line. What each check judges is tested in the `agenticspec-ui` crate.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Without `--source` the command starts the fixture source and every check passes; a source
/// that does not answer fails each check, by name, as a refusal.
#[test]
fn ui_conform_checks_the_fixture_source_and_refuses_a_source_that_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .args(["--output", "json", "ui", "conform", "--root"])
        .arg(repo())
        .env_remove("AGENTICSPEC_UI_TOKEN")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice::<Value>(&out.stdout).unwrap()["result"].clone();
    assert_eq!(v["failed"], 0, "{v}");
    assert!(v["passed"].as_u64().unwrap() > 30, "{v}");

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let out = Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .args([
            "ui",
            "conform",
            "--source",
            &format!("http://127.0.0.1:{port}"),
            "--root",
        ])
        .arg(repo())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("conformance_failed"), "{text}");
    assert!(text.contains("unauthenticated R1"), "{text}");
}
