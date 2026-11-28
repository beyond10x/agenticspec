//! The UI units of a bundle: the operator UI the binding asks for under `ui:` as a `service`
//! unit with static code (the archive `agenticspec ui build --archive` writes, carried in
//! `files`), the ingress export it serves, its upstream, and the refusals of `deploy bundle` and
//! `deploy check` about it.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, BundleOptions, ContractOptions};
use agenticspec_sim::deploy::bundle;
use agenticspec_sim::deploy::ui;
use agenticspec_sim::doc::{load_yaml, parse_yaml};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const FIXTURE: &str = "bundles/workstation.operator-ui.yaml";

fn fixture() -> Value {
    load_yaml(&repo_root().join(FIXTURE)).unwrap()
}

fn unit<'a>(v: &'a Value, name: &str) -> &'a Value {
    v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == name)
        .unwrap_or_else(|| panic!("no unit {name}"))
}

fn entry<'a>(v: &'a Value, ingress: &str, protocol: &str) -> &'a Value {
    v["ingress"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["ingress"] == ingress && e["protocol"] == protocol)
        .unwrap_or_else(|| panic!("no {ingress} {protocol}"))
}

/// `deploy bundle` of the workstation under a binding written to `dir` with `text`.
fn bundle_of(dir: &Path, text: &str) -> Result<Value, cli::Refusal> {
    let root = repo_root();
    let file = dir.join("b.yaml");
    std::fs::write(&file, text).unwrap();
    let answer = cli::deploy_bundle(&BundleOptions {
        architecture: &root.join("architectures/workstation.yaml"),
        binding: &file,
        out: None,
        ports: Some(&root.join("blocks/ports.json")),
        accepts: Some(&root.join("bundles/cloudspec-accepts.yaml")),
        kpis: Some(&root.join("kpis/catalogue.yaml")),
    })?;
    Ok(parse_yaml(answer["bundle"].as_str().unwrap()).unwrap())
}

/// A binding that refines `base` (relative to the repository root) with `extra`.
fn refining(base: &str, extra: &str) -> String {
    format!(
        "format: agenticspec-binding/1\nbinding: odd\narchitecture: workstation\nrefines: {}\n{extra}",
        repo_root().join(base).display()
    )
}

/// The operator UI fixture changed by `edit`, its digests recomputed, through `deploy check`.
fn check_tampered(dir: &Path, edit: impl FnOnce(&mut Value)) -> Result<Value, cli::Refusal> {
    let mut v = fixture();
    edit(&mut v);
    v["digest"] = json!(bundle::recompute(&v));
    let file: PathBuf = dir.join("bundle.yaml");
    std::fs::write(&file, bundle::write(&v)).unwrap();
    cli::deploy_check(
        &file,
        Some(&repo_root().join("bundles/cloudspec-accepts.yaml")),
    )
}

fn names(detail: &str) -> Vec<String> {
    detail
        .split("; ")
        .map(|f| f.split(':').next().unwrap_or("").to_owned())
        .collect()
}

#[test]
fn the_operator_ui_is_a_service_unit_with_static_code_carried_in_files() {
    let v = fixture();
    let u = unit(&v, "operator-ui");
    assert_eq!(u["kind"], "service");
    assert_eq!(u["roles"], json!(["operator-ui"]));
    assert_eq!(u["parts"], json!([]));
    assert_eq!(u["egress"], json!([]));
    assert_eq!(u["secrets"], json!([]));
    assert_eq!(u["code"]["kind"], "static");
    assert_eq!(u["code"]["archive"], "operator-ui.tar");
    let digest = u["code"]["digest"].as_str().unwrap();
    assert!(
        agenticspec_sim::deploy::binding::is_digest(digest),
        "{digest}"
    );
    assert_eq!(
        u["checks"]["ready"],
        json!({ "kind": "http", "path": "/", "port": 8080 })
    );
    let file = &v["files"]["operator-ui.tar"];
    assert_eq!(file["digest"], digest);
    let bytes = ui::unbase64(file["base64"].as_str().unwrap()).unwrap();
    assert_eq!(file["size"], bytes.len());
    assert_eq!(agenticspec_sim::deploy::sha256(&bytes), digest);
    // The members are written in their order, `files` before `digest`.
    let text = std::fs::read_to_string(repo_root().join(FIXTURE)).unwrap();
    let keys: Vec<&str> = text
        .lines()
        .filter(|l| !l.starts_with([' ', '-']))
        .map(|l| l.split(':').next().unwrap())
        .collect();
    assert_eq!(&keys[keys.len() - 3..], ["runtimes", "files", "digest"]);
    // Without `ui:` the bundle has no UI unit and no `files`.
    let split = load_yaml(&repo_root().join("bundles/workstation.split.yaml")).unwrap();
    assert!(split.get("files").is_none());
    assert!(
        split["units"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["code"]["kind"] != "static")
    );
}

#[test]
fn the_ui_export_of_the_operator_ingress_names_the_ui_unit_and_the_rest_stay_in_core() {
    let v = fixture();
    assert_eq!(
        entry(&v, "operator", "ui"),
        &json!({ "ingress": "operator", "protocol": "ui", "audience": "control.operator", "authenticated": true, "unit": "operator-ui", "part": "control", "port": 8080, "bind": "all" })
    );
    for protocol in ["http", "event_stream", "tool"] {
        assert_eq!(
            entry(&v, "operator", protocol)["unit"],
            "core",
            "{protocol}"
        );
    }
}

#[test]
fn the_upstream_is_the_unit_serving_the_api_unless_the_binding_names_one() {
    let v = fixture();
    assert_eq!(
        unit(&v, "operator-ui")["upstream"],
        json!({ "unit": "core", "port": 8080, "path": "/api/" })
    );
    let dir = scratch("deploy-ui-upstream");
    let named = bundle_of(
        &dir,
        &refining(
            "bundles/bindings/operator-ui.yaml",
            "ui:\n  operator:\n    upstream: {unit: gateway, port: 8081}\n",
        ),
    )
    .unwrap();
    assert_eq!(
        unit(&named, "operator-ui")["upstream"],
        json!({ "unit": "gateway", "port": 8081, "path": "/api/" })
    );
    // The gateway serves no export of the operator ingress, so it has no port unless named.
    let unported = bundle_of(
        &dir,
        &refining(
            "bundles/bindings/operator-ui.yaml",
            "ui:\n  operator:\n    upstream: {unit: gateway}\n",
        ),
    )
    .unwrap();
    assert_eq!(
        unit(&unported, "operator-ui")["upstream"],
        json!({ "unit": "gateway", "path": "/api/" })
    );
    // The single packing gives no export a port: `ui.operator` gives the UI's.
    let single = bundle_of(
        &dir,
        &refining(
            "crates/agenticspec-sim/tests/cases/deploy/egress/workstation.yaml",
            "ui:\n  operator: {port: 3000, bind: all}\n",
        ),
    )
    .unwrap();
    let e = entry(&single, "operator", "ui");
    assert_eq!(
        (&e["unit"], &e["port"], &e["bind"]),
        (&json!("operator-ui"), &json!(3000), &json!("all"))
    );
    assert_eq!(
        unit(&single, "operator-ui")["upstream"],
        json!({ "unit": "core", "path": "/api/" })
    );
}

#[test]
fn the_archive_digest_is_the_one_ui_build_archive_writes_for_the_same_inputs() {
    let root = repo_root();
    let dir = scratch("deploy-ui-archive");
    let architecture = root.join("architectures/workstation.yaml");
    let contract = dir.join("workstation.contract.yaml");
    cli::contract(
        &ContractOptions {
            architecture: &architecture,
            ports: Some(&root.join("blocks/ports.json")),
            out: Some(&contract),
            check: None,
            suite: None,
            trace: None,
            kpis: Some(&root.join("kpis/catalogue.yaml")),
        },
        &mut std::io::sink(),
    )
    .unwrap();
    let files = agenticspec_ui::operator::build(&agenticspec_ui::operator::Inputs {
        root: root.clone(),
        architecture,
        contract,
        api_base: agenticspec_ui::operator::API_BASE.to_string(),
    })
    .unwrap_or_else(|e| panic!("{e:?}"));
    let tar = agenticspec_ui::operator::archive(&files).unwrap();
    let v = fixture();
    assert_eq!(
        unit(&v, "operator-ui")["code"]["digest"],
        format!("sha256:{}", agenticspec_ui::operator::sha256(&tar))
    );
    assert_eq!(
        ui::unbase64(v["files"]["operator-ui.tar"]["base64"].as_str().unwrap()).unwrap(),
        tar
    );
}

#[test]
fn the_binding_gives_the_ui_unit_its_claims_and_check_ports_by_its_unit_name() {
    let fixture = fixture();
    let u = unit(&fixture, "operator-ui");
    assert_eq!(
        u["claims"],
        json!({ "cpu_millis": 500, "memory_mib": 512, "accelerators": 0, "volumes": [] })
    );
    let dir = scratch("deploy-ui-claims");
    let v = bundle_of(
        &dir,
        &refining(
            "bundles/bindings/operator-ui.yaml",
            "claims:\n  operator-ui: {cpu_millis: 100, memory_mib: 64}\nchecks:\n  operator-ui:\n    ready: {port: 9000}\n",
        ),
    )
    .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let u = unit(&v, "operator-ui");
    assert_eq!(
        u["claims"],
        json!({ "cpu_millis": 100, "memory_mib": 64, "accelerators": 0, "volumes": [] })
    );
    assert_eq!(
        u["checks"],
        json!({
            "ready": { "kind": "http", "path": "/", "port": 9000 },
            "alive": { "kind": "http", "path": "/", "port": 8080 },
        })
    );
    // The UI unit is a service unit: it has no startup check, and without `ui:` it is no unit.
    for (base, extra, says) in [
        (
            "bundles/bindings/operator-ui.yaml",
            "checks:\n  operator-ui:\n    startup: {port: 9000}\n",
            "checks: operator-ui: a service unit has no startup check",
        ),
        (
            "bundles/bindings/split.yaml",
            "claims:\n  operator-ui: {cpu_millis: 100}\n",
            "claims: `operator-ui` is not a unit of the packing",
        ),
    ] {
        let r = bundle_of(&dir, &refining(base, extra)).unwrap_err();
        assert_eq!(r.code, "binding_invalid", "{extra}: {}", r.detail);
        assert!(r.detail.contains(says), "{extra}: {}", r.detail);
    }
}

#[test]
fn deploy_bundle_refuses_an_upstream_that_names_no_unit_and_a_ui_of_the_wrong_form() {
    let dir = scratch("deploy-ui-refused");
    let r = bundle_of(
        &dir,
        &refining(
            "bundles/bindings/operator-ui.yaml",
            "ui:\n  operator:\n    upstream: {unit: adapter}\n",
        ),
    )
    .unwrap_err();
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    assert_eq!(names(&r.detail), ["ui_upstream_unknown"], "{}", r.detail);
    for (extra, says) in [
        ("ui:\n  console: {}\n", "`console` is not one of operator"),
        (
            "ui:\n  operator: {upstream: {host: x}}\n",
            "unknown key `host`",
        ),
        ("ui:\n  operator: {port: 0}\n", "port is a whole number"),
        ("ui:\n  operator: {port: 9000}\n", "give it in one place"),
        (
            "ingress:\n  operator:\n    exports:\n      ui: {part: gateway}\n",
            "the operator UI's unit serves this export",
        ),
    ] {
        let r = bundle_of(&dir, &refining("bundles/bindings/operator-ui.yaml", extra)).unwrap_err();
        assert_eq!(r.code, "binding_invalid", "{extra}: {}", r.detail);
        assert!(r.detail.contains(says), "{extra}: {}", r.detail);
    }
}

#[test]
fn deploy_check_admits_the_ui_bundle_and_refuses_a_changed_archive_or_an_unknown_upstream() {
    let dir = scratch("deploy-ui-check");
    check_tampered(&dir, |_| {}).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let ui_unit = |v: &mut Value| -> usize {
        v["units"]
            .as_array()
            .unwrap()
            .iter()
            .position(|u| u["name"] == "operator-ui")
            .unwrap()
    };
    let r = check_tampered(&dir, |v| {
        let b = v["files"]["operator-ui.tar"]["base64"].as_str().unwrap();
        let changed = format!("AAAA{}", &b[4..]);
        v["files"]["operator-ui.tar"]["base64"] = json!(changed);
    })
    .unwrap_err();
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    assert_eq!(names(&r.detail), ["ui_archive_mismatch"], "{}", r.detail);
    let r = check_tampered(&dir, |v| {
        v["files"]
            .as_object_mut()
            .unwrap()
            .remove("operator-ui.tar");
    })
    .unwrap_err();
    assert_eq!(names(&r.detail), ["ui_archive_mismatch"], "{}", r.detail);
    let r = check_tampered(&dir, |v| {
        let i = ui_unit(v);
        v["units"][i]["upstream"]["unit"] = json!("adapter");
    })
    .unwrap_err();
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    assert_eq!(names(&r.detail), ["ui_upstream_unknown"], "{}", r.detail);
}

#[test]
fn base64_reads_back_what_it_writes() {
    for bytes in [&b""[..], b"a", b"ab", b"abc", b"abcd", &[0, 255, 128, 7, 9]] {
        let text = ui::base64(bytes);
        assert_eq!(text.len() % 4, 0);
        assert_eq!(ui::unbase64(&text).as_deref(), Some(bytes), "{text}");
    }
    assert_eq!(ui::base64(b"foobar"), "Zm9vYmFy");
    assert_eq!(ui::base64(b"fo"), "Zm8=");
    for bad in ["Zm8", "Zm=8", "Z===", "Zm8=Zm8=", "Zm!="] {
        assert_eq!(ui::unbase64(bad), None, "{bad}");
    }
}
