//! `agenticspec ui conform`: the conformance suite of the runtime interface on the generated
//! command line. What each check judges is tested in the `agenticspec-ui` crate.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

mod conform;
mod serve;

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

/// `agenticspec ui build` of the workstation into `dir`, from the repository root.
fn ui_build(dir: &std::path::Path, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .current_dir(repo())
        .args([
            "--output",
            "json",
            "ui",
            "build",
            "architectures/workstation.yaml",
            "--contract",
            "ui/views/examples/workstation.contract.yaml",
            "--out",
        ])
        .arg(dir)
        .args(extra)
        .output()
        .unwrap()
}

fn result(out: &std::process::Output) -> Value {
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice::<Value>(&out.stdout).unwrap()["result"].clone()
}

/// The directory holds the embedded app and the data files, `views.json` is the layout
/// `ui derive` writes for the same inputs, and two runs give the same archive, whose digest is
/// printed.
#[test]
fn ui_build_writes_the_app_and_its_data_and_the_same_archive_each_time() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ui-build");
    let _ = std::fs::remove_dir_all(&tmp);
    let (a, b) = (tmp.join("a/ui"), tmp.join("b/ui"));
    std::fs::create_dir_all(a.parent().unwrap()).unwrap();
    std::fs::create_dir_all(b.parent().unwrap()).unwrap();
    let ra = result(&ui_build(&a, &["--archive"]));
    let rb = result(&ui_build(&b, &["--archive"]));

    let mut names: Vec<String> = std::fs::read_dir(&a)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "app.css",
            "app.js",
            "config.json",
            "contract.json",
            "index.html",
            "ports.json",
            "views.json"
        ]
    );
    assert_eq!(ra["files"], 7, "{ra}");
    let config: Value =
        serde_json::from_str(&std::fs::read_to_string(a.join("config.json")).unwrap()).unwrap();
    assert_eq!(
        config,
        serde_json::json!({
            "format": "agenticspec-operator-config/1",
            "architecture": "workstation",
            "api_base": "/api/",
            "title": "workstation",
        })
    );
    for (name, committed) in [
        ("views.json", "ui/views/examples/workstation.layout.json"),
        ("ports.json", "blocks/ports.json"),
        ("index.html", "ui/operator/built/index.html"),
        ("app.js", "ui/operator/built/app.js"),
        ("app.css", "ui/operator/built/app.css"),
    ] {
        assert_eq!(
            std::fs::read(a.join(name)).unwrap(),
            std::fs::read(repo().join(committed)).unwrap(),
            "{name} is not {committed}"
        );
    }

    let digest = ra["archive_sha256"].as_str().unwrap();
    assert_eq!(digest.len(), 64);
    assert_eq!(rb["archive_sha256"].as_str(), Some(digest));
    let (ta, tb) = (
        std::fs::read(tmp.join("a/ui.tar")).unwrap(),
        std::fs::read(tmp.join("b/ui.tar")).unwrap(),
    );
    assert_eq!(ta, tb);
    assert_eq!(agenticspec_ui::operator::sha256(&ta), digest);
    assert_eq!(ta.len() % 512, 0);
    assert_eq!(&ta[257..263], b"ustar\0");
    assert_eq!(&ta[..7], b"app.css");

    // A second build into the same directory replaces it; another base path lands in config.json
    // and changes the digest.
    let rc = result(&ui_build(
        &a,
        &["--archive", "--api-base", "https://adapter.test/root"],
    ));
    let config: Value =
        serde_json::from_str(&std::fs::read_to_string(a.join("config.json")).unwrap()).unwrap();
    assert_eq!(config["api_base"], "https://adapter.test/root");
    assert_ne!(rc["archive_sha256"].as_str(), Some(digest));
}

/// `ui build --control-plane` writes the control-plane UI from the binary alone: the same app,
/// the layout `ui derive --control-plane` writes, and a config in control-plane mode.
#[test]
fn ui_build_control_plane_writes_the_pages_across_parts() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ui-build-control");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let dir = tmp.join("control-plane-ui");
    let r = result(&ui_build(&dir, &["--control-plane", "--archive"]));
    assert_eq!(
        std::fs::read(dir.join("views.json")).unwrap(),
        std::fs::read(repo().join("ui/views/examples/workstation.control-plane.layout.json"))
            .unwrap(),
        "views.json is not the derived control-plane layout"
    );
    assert_eq!(
        std::fs::read(dir.join("app.js")).unwrap(),
        std::fs::read(repo().join("ui/operator/built/app.js")).unwrap()
    );
    let config: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("config.json")).unwrap()).unwrap();
    assert_eq!(config["mode"], "control-plane");
    assert_eq!(config["title"], "workstation control plane");
    let tar = std::fs::read(tmp.join("control-plane-ui.tar")).unwrap();
    assert_eq!(
        r["archive_sha256"].as_str(),
        Some(agenticspec_ui::operator::sha256(&tar).as_str())
    );
}

/// Every file of `ui/operator/built` is embedded, and nothing else.
#[test]
fn the_binary_embeds_every_built_file_of_the_operator_app() {
    let dir = repo().join("ui/operator/built");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let embedded: Vec<&str> = agenticspec_ui::operator::APP
        .iter()
        .map(|(n, _)| *n)
        .collect();
    assert_eq!(names, embedded);
}

/// A directory that holds other files is refused and left as it was; so is a contract derived
/// from other ports.
#[test]
fn ui_build_refuses_a_foreign_directory_and_a_contract_of_other_ports() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ui-build-refused");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("notes.txt"), "mine").unwrap();
    let out = ui_build(&tmp, &[]);
    assert!(!out.status.success());
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    assert!(text.contains("out_unwritable"), "{text}");
    assert_eq!(
        std::fs::read_to_string(tmp.join("notes.txt")).unwrap(),
        "mine"
    );

    let contract =
        std::fs::read_to_string(repo().join("ui/views/examples/workstation.contract.yaml"))
            .unwrap();
    let digest = contract
        .lines()
        .find_map(|l| l.strip_prefix("ports_digest: "))
        .unwrap()
        .to_string();
    let other = tmp.join("other.contract.yaml");
    std::fs::write(&other, contract.replace(&digest, &"0".repeat(64))).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_agenticspec"))
        .current_dir(repo())
        .args([
            "ui",
            "build",
            "architectures/workstation.yaml",
            "--contract",
        ])
        .arg(&other)
        .arg("--out")
        .arg(tmp.join("ui"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    assert!(text.contains("document_refused"), "{text}");
    assert!(text.contains("ports_digest"), "{text}");
    assert!(!tmp.join("ui").exists());
}
