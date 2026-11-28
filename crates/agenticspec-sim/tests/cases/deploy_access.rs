//! The access of a volume claim follows from who mounts it (R93): `read_write_once` when every
//! part that mounts it is in one unit and the claim is one per copy or the unit runs one copy,
//! whatever the sharing among those parts; `read_write_many` when two units, or two copies of
//! one unit, mount the same claim; `read_only_many` for a shared read-only workspace. `deploy
//! check` admits the bundle as written.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, BundleOptions};
use agenticspec_sim::doc::parse_yaml;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The architecture: a runtime `flow` with its own `notes` (shared read-write, `per: flow`), a
/// `board` shared read-write and a `base` shared read-only.
fn architecture() -> PathBuf {
    repo_root().join("crates/agenticspec-sim/tests/cases/deploy/volume_access.architecture.yaml")
}

/// An image of the registry, pinned by digest `n`.
fn image(n: u32) -> Value {
    json!({ "image": format!("registry.example/image-{n}"), "digest": format!("sha256:{n:064x}") })
}

/// One `service` unit with one replica packs the runtime and its notes, which the runtime
/// mounts; the core holds the lead, the gateway, the board and the base, which the lead mounts.
fn base() -> Value {
    json!({
        "format": "agenticspec-binding/1",
        "binding": "access",
        "architecture": "volume-access",
        "transports": { "run": "http", "model_call": "http", "checkout": "http" },
        "runtimes": { "core": image(1) },
        "parts": {
            "lead": {
                "egress": ["flow", "gateway"],
                "volumes": [
                    { "name": "board", "mount": "/board" },
                    { "name": "base", "mount": "/base" },
                ],
            },
            "flow": {
                "egress": ["gateway"],
                "volumes": [{ "name": "notes", "mount": "/notes" }],
            },
            "gateway": { "egress": ["model"] },
        },
        "packing": {
            "units": [
                { "name": "core", "kind": "core", "parts": ["lead", "gateway", "board", "base"] },
                { "name": "flow", "kind": "service", "parts": ["flow", "notes"], "replicas": 1, "image": image(2) },
            ],
            "external": ["owner", "model"],
        },
    })
}

fn write(dir: &Path, name: &str, text: &str) -> PathBuf {
    let file = dir.join(name);
    std::fs::write(&file, text).unwrap();
    file
}

/// `deploy bundle` under `binding`, then `deploy check` of the bundle as written: the bundle.
fn admitted(dir: &Path, binding: &Value) -> Value {
    let root = repo_root();
    let accepts = root.join("bundles/cloudspec-accepts.yaml");
    let file = write(dir, "b.yaml", &binding.to_string());
    let out = cli::deploy_bundle(&BundleOptions {
        architecture: &architecture(),
        binding: &file,
        out: None,
        ports: Some(&root.join("blocks/ports.json")),
        accepts: Some(&accepts),
        kpis: Some(&root.join("kpis/catalogue.yaml")),
    })
    .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let text = out["bundle"].as_str().unwrap();
    let written = write(dir, "bundle.yaml", text);
    cli::deploy_check(&written, Some(&accepts))
        .unwrap_or_else(|r| panic!("deploy check: {}: {}", r.code, r.detail));
    parse_yaml(text).unwrap()
}

/// The access of the claim on `volume` in unit `unit`.
fn access(v: &Value, unit: &str, volume: &str) -> String {
    let u = v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == unit)
        .unwrap_or_else(|| panic!("no unit {unit}"));
    let claim = u["claims"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == volume)
        .unwrap_or_else(|| panic!("unit {unit} claims no volume {volume}"));
    claim["access"].as_str().unwrap().to_owned()
}

#[test]
fn a_workspace_shared_by_parts_of_one_unit_with_one_copy_is_read_write_once() {
    let dir = scratch("deploy-access-one-unit");
    let v = admitted(&dir, &base());
    let flow = v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == "flow")
        .unwrap();
    assert_eq!(flow["replicas"], json!(1));
    assert_eq!(
        flow["claims"]["volumes"],
        json!([{ "name": "notes", "access": "read_write_once", "per_instance": true, "mount": "/notes" }])
    );
    // Not per instance, and the core runs one copy: still one writer.
    assert_eq!(access(&v, "core", "board"), "read_write_once");
}

#[test]
fn a_workspace_two_units_mount_is_read_write_many() {
    let dir = scratch("deploy-access-two-units");
    let mut b = base();
    b["parts"]["flow"]["volumes"] = json!([
        { "name": "notes", "mount": "/notes" },
        { "name": "board", "mount": "/board" },
    ]);
    let v = admitted(&dir, &b);
    assert_eq!(access(&v, "core", "board"), "read_write_many");
    assert_eq!(access(&v, "flow", "notes"), "read_write_once");
}

#[test]
fn a_workspace_not_per_instance_in_a_unit_of_two_replicas_is_read_write_many() {
    let dir = scratch("deploy-access-replicas");
    let mut b = base();
    b["packing"]["units"] = json!([
        { "name": "core", "kind": "core", "parts": ["lead", "gateway", "base"] },
        { "name": "flow", "kind": "service", "parts": ["flow", "notes", "board"], "replicas": 2, "image": image(2) },
    ]);
    b["parts"]["lead"]["volumes"] = json!([{ "name": "base", "mount": "/base" }]);
    b["parts"]["flow"]["volumes"] = json!([
        { "name": "notes", "mount": "/notes" },
        { "name": "board", "mount": "/board" },
    ]);
    b["parts"]["board"] = json!({ "egress": ["lead"] });
    let v = admitted(&dir, &b);
    assert_eq!(access(&v, "flow", "board"), "read_write_many");
    // One claim per copy: each copy mounts its own.
    assert_eq!(access(&v, "flow", "notes"), "read_write_once");
}

#[test]
fn a_shared_read_only_workspace_is_read_only_many_however_many_mount_it() {
    let dir = scratch("deploy-access-read-only");
    // One unit, one copy.
    let v = admitted(&dir, &base());
    assert_eq!(access(&v, "core", "base"), "read_only_many");
    // Two units.
    let mut b = base();
    b["parts"]["flow"]["volumes"] = json!([
        { "name": "notes", "mount": "/notes" },
        { "name": "base", "mount": "/base" },
    ]);
    let v = admitted(&dir, &b);
    assert_eq!(access(&v, "core", "base"), "read_only_many");
}
