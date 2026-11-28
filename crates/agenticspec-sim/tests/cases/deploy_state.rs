//! The durable state of a runtime part (`durable_state: true`, R88): its unit claims an
//! exclusive volume `<part>-state`, mounted where the binding says and sized as a workspace's
//! volume is; refused without a mount (`volume_mount_missing`) and in a unit that holds no volume
//! (`durable_state_unplaceable`); `deploy check` applies the same rules to a bundle read back.
//! Programs outside the runtime that write the state too (`state_writers`, R109) are named in the
//! claim, which stays one volume.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, BundleOptions};
use agenticspec_sim::doc::parse_yaml;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The architecture whose runtime part `flow` keeps durable state.
fn architecture() -> PathBuf {
    repo_root().join(
        "crates/agenticspec-sim/tests/cases/deploy/durable_state_unplaceable.architecture.yaml",
    )
}

/// An image of the registry, pinned by digest `n`.
fn image(n: u32) -> Value {
    json!({ "image": format!("registry.example/image-{n}"), "digest": format!("sha256:{n:064x}") })
}

/// A binding with the runtime in a service unit of its own with its image, its state mounted at
/// `/var/lib/flow`; every crossing has a transport and every destination is declared.
fn base() -> Value {
    json!({
        "format": "agenticspec-binding/1",
        "binding": "state",
        "architecture": "durable-state",
        "transports": { "run": "http", "model_call": "http" },
        "runtimes": { "core": image(1) },
        "parts": {
            "lead": { "egress": ["flow", "gateway"] },
            "flow": {
                "egress": ["gateway"],
                "volumes": [{ "name": "flow-state", "mount": "/var/lib/flow" }],
            },
            "gateway": { "egress": ["model"] },
        },
        "packing": {
            "units": [
                { "name": "core", "kind": "core", "parts": ["lead", "gateway"] },
                { "name": "flow", "kind": "service", "parts": ["flow"], "image": image(2) },
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

/// `deploy bundle` of `architecture` under `binding`: the bundle's text, or the refusal.
fn bundle_text(dir: &Path, architecture: &Path, binding: &Value) -> Result<String, cli::Refusal> {
    let root = repo_root();
    let file = write(dir, "b.yaml", &binding.to_string());
    let out = cli::deploy_bundle(&BundleOptions {
        architecture,
        binding: &file,
        out: None,
        ports: Some(&root.join("blocks/ports.json")),
        accepts: Some(&root.join("bundles/cloudspec-accepts.yaml")),
        kpis: Some(&root.join("kpis/catalogue.yaml")),
    })?;
    Ok(out["bundle"].as_str().unwrap().to_owned())
}

fn admitted(dir: &Path, binding: &Value) -> Value {
    let text = bundle_text(dir, &architecture(), binding)
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    parse_yaml(&text).unwrap()
}

fn unit<'a>(v: &'a Value, name: &str) -> &'a Value {
    v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == name)
        .unwrap_or_else(|| panic!("no unit {name}"))
}

fn refused_by(r: &cli::Refusal, name: &str) {
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    let names: BTreeSet<&str> = r
        .detail
        .split("; ")
        .map(|f| f.split(':').next().unwrap_or(""))
        .collect();
    assert_eq!(names, BTreeSet::from([name]), "{}", r.detail);
}

fn refused(dir: &Path, binding: &Value) -> cli::Refusal {
    bundle_text(dir, &architecture(), binding).unwrap_err()
}

#[test]
fn a_runtime_with_durable_state_claims_an_exclusive_volume_mounted_and_sized_by_the_binding() {
    let dir = scratch("deploy-state-claim");
    let mut b = base();
    b["parts"]["flow"]["size"] = json!("2Gi");
    let v = admitted(&dir, &b);
    assert_eq!(
        unit(&v, "flow")["claims"]["volumes"],
        json!([{
            "name": "flow-state",
            "access": "read_write_once",
            "per_instance": false,
            "size": "2Gi",
            "mount": "/var/lib/flow",
        }])
    );
    assert_eq!(unit(&v, "core")["claims"]["volumes"], json!([]));
    // Without a size the claim carries none, as a workspace's does; another part of the unit
    // may name the mount.
    let mut b = base();
    b["packing"]["units"] = json!([
        { "name": "core", "kind": "core", "parts": ["lead", "gateway", "flow"] },
    ]);
    let flow = b["parts"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("volumes");
    b["parts"]["lead"]["volumes"] = flow.unwrap();
    let v = admitted(&dir, &b);
    assert_eq!(
        unit(&v, "core")["claims"]["volumes"],
        json!([{ "name": "flow-state", "access": "read_write_once", "per_instance": false, "mount": "/var/lib/flow" }])
    );
}

/// R109: durable state that programs outside the runtime write too (`state_writers`) is still one
/// volume of the runtime's unit, `read_write_once` in a unit of one copy; no workspace part, so no
/// workspace lease or proposal port; the claim and the architecture the bundle carries name the
/// writers, and `deploy check` admits the bundle read back.
#[test]
fn durable_state_with_outside_writers_is_one_volume_that_names_them_and_no_workspace_port() {
    let dir = scratch("deploy-state-writers");
    let text = std::fs::read_to_string(architecture()).unwrap().replace(
        "durable_state: true",
        "durable_state: true, state_writers: [backup, indexer]",
    );
    let file = write(&dir, "writers.yaml", &text);
    let bundle =
        bundle_text(&dir, &file, &base()).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(&bundle).unwrap();
    assert_eq!(
        unit(&v, "flow")["claims"]["volumes"],
        json!([{
            "name": "flow-state",
            "access": "read_write_once",
            "per_instance": false,
            "writers": ["backup", "indexer"],
            "mount": "/var/lib/flow",
        }])
    );
    assert_eq!(unit(&v, "core")["claims"]["volumes"], json!([]));
    assert_eq!(
        v["architecture"]["document"]["parts"]["flow"]["state_writers"],
        json!(["backup", "indexer"])
    );
    for port in ["workspace.lease", "workspace.proposal"] {
        assert!(!bundle.contains(port), "the bundle names {port}");
    }
    let accepts = repo_root().join("bundles/cloudspec-accepts.yaml");
    let checked = write(&dir, "bundle.yaml", &bundle);
    cli::deploy_check(&checked, Some(&accepts))
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    // Without writers the claim carries no `writers` key: `durable_state: true` alone is as
    // before.
    let plain = bundle_text(&dir, &architecture(), &base()).unwrap();
    assert!(!plain.contains("writers"), "{plain}");
}

#[test]
fn durable_state_is_off_by_default_and_its_volume_is_then_no_name_the_binding_may_mount() {
    let dir = scratch("deploy-state-off");
    let text = std::fs::read_to_string(architecture())
        .unwrap()
        .replace(", durable_state: true", "");
    let plain = write(&dir, "plain.yaml", &text);
    let mut b = base();
    b["parts"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("volumes");
    let v = parse_yaml(&bundle_text(&dir, &plain, &b).unwrap()).unwrap();
    assert_eq!(unit(&v, "flow")["claims"]["volumes"], json!([]));
    let r = bundle_text(&dir, &plain, &base()).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    assert!(
        r.detail.contains(
            "volumes[0]: name names a workspace part of the document, or `<part>-state` of a runtime part with `durable_state: true`"
        ),
        "{}",
        r.detail
    );
}

#[test]
fn durable_state_without_an_absolute_mount_is_refused_as_a_workspace_is() {
    let dir = scratch("deploy-state-mount");
    let mut b = base();
    b["parts"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("volumes");
    let r = refused(&dir, &b);
    refused_by(&r, "volume_mount_missing");
    assert!(
        r.detail
            .contains("unit flow: volume flow-state (durable state of flow) has no mount"),
        "{}",
        r.detail
    );
    let mut b = base();
    b["parts"]["flow"]["volumes"][0]["mount"] = json!("var/lib/flow");
    let r = refused(&dir, &b);
    refused_by(&r, "volume_mount_missing");
    assert!(r.detail.contains("not an absolute path"), "{}", r.detail);
    assert!(!r.detail.contains("var/lib/flow"), "{}", r.detail);
}

#[test]
fn durable_state_in_a_unit_that_holds_no_volume_is_refused() {
    let dir = scratch("deploy-state-unplaceable");
    for kind in ["job", "model"] {
        let mut b = base();
        b["packing"]["units"][1] = json!({ "name": "flow", "kind": kind, "parts": ["flow"] });
        let r = refused(&dir, &b);
        refused_by(&r, "durable_state_unplaceable");
        assert!(
            r.detail.contains(&format!(
                "runtime part flow keeps durable state, and its unit flow is a {kind} unit"
            )),
            "{}",
            r.detail
        );
    }
    // A part named as the state volume would be is refused too: two volumes of one name.
    let text = std::fs::read_to_string(architecture()).unwrap().replace(
        "  gateway: { block: gateway }",
        "  gateway: { block: gateway }\n  flow-state: { block: workspace, sharing: exclusive, per: lead }",
    );
    let clash = write(&dir, "clash.yaml", &text);
    let mut b = base();
    b["packing"]["units"][0]["parts"] = json!(["lead", "gateway", "flow-state"]);
    let r = bundle_text(&dir, &clash, &b).unwrap_err();
    assert!(
        r.detail.contains(
            "durable_state_unplaceable: runtime part flow keeps durable state in the volume flow-state, and a part of the document has that name"
        ),
        "{}",
        r.detail
    );
}

#[test]
fn deploy_check_refuses_a_state_volume_without_a_mount_and_one_in_a_model_unit() {
    let dir = scratch("deploy-state-check");
    let accepts = repo_root().join("bundles/cloudspec-accepts.yaml");
    let text = bundle_text(&dir, &architecture(), &base()).unwrap();
    let file = write(&dir, "bundle.yaml", &text);
    cli::deploy_check(&file, Some(&accepts)).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    type Edit = fn(&mut Value);
    let cases: [(&str, Edit); 2] = [
        ("volume_mount_missing", |v| {
            let u = v["units"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|u| u["name"] == "flow")
                .unwrap();
            u["claims"]["volumes"][0]
                .as_object_mut()
                .unwrap()
                .remove("mount");
        }),
        ("durable_state_unplaceable", |v| {
            v["binding"]["document"]["packing"]["units"][1] =
                json!({ "name": "flow", "kind": "model", "parts": ["flow"] });
            v["packing"]["document"]["units"][1] =
                json!({ "name": "flow", "kind": "model", "parts": ["flow"] });
        }),
    ];
    for (name, edit) in cases {
        let mut v = parse_yaml(&text).unwrap();
        edit(&mut v);
        v["binding"]["digest"] = json!(agenticspec_sim::deploy::digest(&v["binding"]["document"]));
        v["packing"]["digest"] = json!(agenticspec_sim::deploy::digest(&v["packing"]["document"]));
        v["digest"] = json!(agenticspec_sim::deploy::bundle::recompute(&v));
        let file = write(
            &dir,
            "tampered.yaml",
            &agenticspec_sim::deploy::bundle::write(&v),
        );
        let r = cli::deploy_check(&file, Some(&accepts)).unwrap_err();
        assert_eq!(r.code, "not_admitted", "{name}: {}", r.detail);
        assert!(
            r.detail.split("; ").any(|f| f.starts_with(name)),
            "{name}: {}",
            r.detail
        );
    }
}
