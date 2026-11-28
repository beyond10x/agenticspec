//! The runtime image of each core and service unit of a bundle: a listed unit's `image:`, else
//! the one its parts name (`parts.<p>.image`), else for the core unit only `runtimes.core`; the
//! refusals `unit_image_conflict` and `unit_image_missing`, the registry and local forms of an
//! image, and `deploy check` applying the same rules to a bundle read back.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, BundleOptions};
use agenticspec_sim::deploy::bundle;
use agenticspec_sim::doc::parse_yaml;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The architecture with two runtime parts, `flow` and `notifier`, and a gateway.
fn architecture() -> PathBuf {
    repo_root()
        .join("crates/agenticspec-sim/tests/cases/deploy/unit_image_missing.architecture.yaml")
}

fn digest(n: u32) -> String {
    format!("sha256:{n:064x}")
}

fn image(name: &str, n: u32) -> Value {
    json!({ "image": format!("registry.example/{name}"), "digest": digest(n) })
}

fn reference(name: &str, n: u32) -> String {
    format!("registry.example/{name}@{}", digest(n))
}

/// A binding of the architecture with one unit per part but the lead, which is in the core, and
/// the core runtime pinned; every destination declared.
fn base() -> Value {
    json!({
        "format": "agenticspec-binding/1",
        "binding": "images",
        "architecture": "runtimes",
        "transports": { "run": "http", "model_call": "http" },
        "runtimes": { "core": image("core", 1) },
        "parts": {
            "lead": { "egress": ["flow", "gateway", "notifier"] },
            "flow": { "egress": ["gateway"] },
            "gateway": { "egress": ["model"] },
        },
        "packing": {
            "units": [
                { "name": "core", "kind": "core", "parts": ["lead"] },
                { "name": "gateway", "kind": "service", "parts": ["gateway"] },
                { "name": "flow", "kind": "service", "parts": ["flow"] },
                { "name": "notifier", "kind": "service", "parts": ["notifier"] },
            ],
            "external": ["owner", "model"],
        },
    })
}

/// `deploy bundle` of the architecture under `binding`, written as JSON (which is YAML) to `dir`.
fn bundle_of(dir: &Path, binding: &Value) -> Result<Value, cli::Refusal> {
    let root = repo_root();
    let file = dir.join("b.yaml");
    std::fs::write(&file, binding.to_string()).unwrap();
    let out = cli::deploy_bundle(&BundleOptions {
        architecture: &architecture(),
        binding: &file,
        out: None,
        ports: Some(&root.join("blocks/ports.json")),
        accepts: Some(&root.join("bundles/cloudspec-accepts.yaml")),
        kpis: Some(&root.join("kpis/catalogue.yaml")),
    })?;
    Ok(parse_yaml(out["bundle"].as_str().unwrap()).unwrap())
}

fn admitted(dir: &Path, binding: &Value) -> Value {
    bundle_of(dir, binding).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail))
}

fn unit<'a>(v: &'a Value, name: &str) -> &'a Value {
    v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == name)
        .unwrap_or_else(|| panic!("no unit {name}"))
}

/// The refusal names of a `not_admitted` detail.
fn names(detail: &str) -> BTreeSet<String> {
    detail
        .split("; ")
        .map(|f| f.split(':').next().unwrap_or("").to_owned())
        .collect()
}

fn refused_by(r: &cli::Refusal, name: &str) {
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    assert_eq!(
        names(&r.detail),
        BTreeSet::from([name.to_owned()]),
        "{}",
        r.detail
    );
}

#[test]
fn a_unit_image_wins_over_its_parts_images_and_each_image_is_carried_once() {
    let dir = scratch("deploy-images-unit");
    let mut b = base();
    b["packing"]["units"][2]["image"] = image("flow", 2);
    b["parts"]["flow"]["image"] = image("flow-old", 3);
    b["parts"]["notifier"] = json!({ "image": image("notifier", 4) });
    b["parts"]["gateway"]["image"] = image("core", 1);
    let v = admitted(&dir, &b);
    assert_eq!(
        unit(&v, "flow")["code"],
        json!({ "runtime": reference("flow", 2) })
    );
    assert_eq!(
        unit(&v, "notifier")["code"],
        json!({ "runtime": reference("notifier", 4) })
    );
    assert_eq!(
        unit(&v, "core")["code"],
        json!({ "runtime": reference("core", 1) })
    );
    assert_eq!(
        unit(&v, "gateway")["code"],
        json!({ "runtime": reference("core", 1) })
    );
    for u in ["core", "gateway", "flow", "notifier"] {
        let roles = unit(&v, u)["roles"].as_array().unwrap();
        assert!(!roles.contains(&json!("core-runtime")), "{u}: {roles:?}");
    }
    assert_eq!(
        v["runtimes"],
        json!({
            digest(1): { "name": "core", "image": reference("core", 1) },
            digest(2): { "name": "flow", "image": reference("flow", 2) },
            digest(4): { "name": "notifier", "image": reference("notifier", 4) },
        }),
        "the image the unit's own image overrides is not carried, and one the core runtime pins is carried once"
    );
}

#[test]
fn a_part_image_names_its_units_image_and_parts_alike_do_not_conflict() {
    let dir = scratch("deploy-images-part");
    let mut b = base();
    b["packing"]["units"] = json!([
        { "name": "core", "kind": "core", "parts": ["lead"] },
        { "name": "gateway", "kind": "service", "parts": ["gateway"] },
        { "name": "runtimes", "kind": "service", "parts": ["flow", "notifier"] },
    ]);
    b["parts"]["lead"]["egress"] = json!(["flow", "gateway", "notifier"]);
    b["parts"]["flow"]["image"] = image("runner", 5);
    b["parts"]["notifier"] = json!({ "image": image("runner", 5) });
    let v = admitted(&dir, &b);
    assert_eq!(
        unit(&v, "runtimes")["code"],
        json!({ "runtime": reference("runner", 5) })
    );
    assert_eq!(v["runtimes"][digest(5)]["name"], "runtimes");
}

#[test]
fn parts_of_one_unit_naming_different_images_are_refused_unless_the_unit_names_one() {
    let dir = scratch("deploy-images-conflict");
    let mut b = base();
    b["packing"]["units"] = json!([
        { "name": "core", "kind": "core", "parts": ["lead"] },
        { "name": "gateway", "kind": "service", "parts": ["gateway"] },
        { "name": "runtimes", "kind": "service", "parts": ["flow", "notifier"] },
    ]);
    b["parts"]["flow"]["image"] = image("flow", 2);
    b["parts"]["notifier"] = json!({ "image": image("notifier", 4) });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, "unit_image_conflict");
    assert!(
        r.detail
            .contains("unit runtimes: its parts flow, notifier name different images"),
        "{}",
        r.detail
    );
    assert!(
        !r.detail.contains("registry.example"),
        "names parts, not images: {}",
        r.detail
    );
    b["packing"]["units"][2]["image"] = image("runner", 5);
    let v = admitted(&dir, &b);
    assert_eq!(
        unit(&v, "runtimes")["code"],
        json!({ "runtime": reference("runner", 5) })
    );
}

#[test]
fn a_service_unit_with_a_runtime_part_and_no_image_is_refused_and_the_core_runtime_is_the_cores_only()
 {
    let dir = scratch("deploy-images-missing");
    let mut b = base();
    b["parts"]["flow"]["image"] = image("flow", 2);
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, "unit_image_missing");
    assert!(
        r.detail
            .contains("service unit notifier holds the runtime part notifier and has no image"),
        "{}",
        r.detail
    );
    assert!(
        r.detail.contains("`runtimes.core` is the core unit's only"),
        "{}",
        r.detail
    );
    assert!(!r.detail.contains("unit flow"), "{}", r.detail);
    // A service unit without a runtime part and without an image names the role the target
    // binds; `runtimes.core` stays the core unit's.
    b["parts"]["notifier"] = json!({ "image": image("notifier", 4) });
    let v = admitted(&dir, &b);
    let gateway = unit(&v, "gateway");
    assert_eq!(gateway["code"], json!({ "role": "core-runtime" }));
    assert!(
        gateway["roles"]
            .as_array()
            .unwrap()
            .contains(&json!("core-runtime"))
    );
    assert_eq!(
        unit(&v, "core")["code"],
        json!({ "runtime": reference("core", 1) })
    );
    // The core unit holding a runtime part takes `runtimes.core`, and is refused without it.
    let mut single = json!({
        "format": "agenticspec-binding/1",
        "binding": "single",
        "architecture": "runtimes",
        "runtimes": { "core": image("core", 1) },
        "parts": { "gateway": { "egress": ["model"] } },
    });
    let v = admitted(&dir, &single);
    assert_eq!(
        unit(&v, "core")["code"],
        json!({ "runtime": reference("core", 1) })
    );
    single.as_object_mut().unwrap().remove("runtimes");
    let r = bundle_of(&dir, &single).unwrap_err();
    refused_by(&r, "unit_image_missing");
    assert!(
        r.detail
            .contains("core unit core holds the runtime part flow, notifier and has no image"),
        "{}",
        r.detail
    );
}

#[test]
fn a_unit_or_part_image_takes_the_registry_or_local_form_of_the_target() {
    let dir = scratch("deploy-images-forms");
    // From the target's local store: by tag, by config id, or refused with neither.
    let mut b = base();
    b["target"] = json!({ "images": "local" });
    b["runtimes"]["core"] = json!({ "image": "core", "tag": "dev" });
    b["parts"]["flow"]["image"] = json!({ "image": "flow", "tag": "dev" });
    b["packing"]["units"][3]["image"] = json!({ "image": "notifier", "digest": digest(9) });
    let v = admitted(&dir, &b);
    assert_eq!(unit(&v, "flow")["code"]["runtime"], "flow:dev");
    assert_eq!(unit(&v, "notifier")["code"]["runtime"], digest(9));
    assert_eq!(
        v["runtimes"]["flow:dev"],
        json!({ "name": "flow", "image": "flow", "tag": "dev" })
    );
    assert_eq!(
        v["runtimes"][digest(9)],
        json!({ "name": "notifier", "image": "notifier", "digest": digest(9) })
    );
    b["parts"]["flow"]["image"] = json!({ "image": "flow" });
    b["packing"]["units"][3]["image"] = json!({ "image": "notifier" });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, "image_unpinned");
    for at in [
        "parts: flow: image: an image",
        "packing: unit notifier: image: an image",
    ] {
        assert!(r.detail.contains(at), "{at} in {}", r.detail);
    }
    // From a registry: pinned by its repository digest, and a tag is refused, as the form of the
    // binding.
    let mut b = base();
    b["parts"]["flow"]["image"] = json!({ "image": "registry.example/flow", "tag": "dev" });
    b["packing"]["units"][3]["image"] = json!({ "image": "registry.example/notifier:1" });
    let r = bundle_of(&dir, &b).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    for needle in [
        "parts: flow: image: `digest:`",
        "parts: flow: image: `tag:` goes with",
        "packing: unit notifier: image: `digest:`",
        "packing: unit notifier: image: `image:` names the image without tag",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
    // An image is for a core or service unit; the lead's unit is the core, an agent unit is not.
    let mut b = base();
    b["packing"]["units"][0] = json!({ "name": "core", "kind": "core", "parts": [] });
    b["packing"]["units"].as_array_mut().unwrap().push(
        json!({ "name": "lead", "kind": "agent", "parts": ["lead"], "image": image("lead", 6) }),
    );
    b["parts"]["lead"]["image"] = image("lead", 6);
    let r = bundle_of(&dir, &b).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    for needle in [
        "parts: lead: image is for a part of a core or service unit, and lead is in the agent unit lead",
        "packing: unit lead: image is for a core or service unit, and it is a agent unit",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
}

/// `bundle` with its binding changed by `edit`, the binding's and the bundle's digests recomputed.
fn tampered(bundle: &Value, edit: impl FnOnce(&mut Value)) -> String {
    let mut v = bundle.clone();
    edit(&mut v["binding"]["document"]);
    v["binding"]["digest"] = json!(agenticspec_sim::deploy::digest(&v["binding"]["document"]));
    v["digest"] = json!(bundle::recompute(&v));
    bundle::write(&v)
}

/// A change of the binding a bundle carries.
type Edit = Box<dyn FnOnce(&mut Value)>;

fn check(dir: &Path, text: &str) -> Result<Value, cli::Refusal> {
    let file = dir.join("bundle.yaml");
    std::fs::write(&file, text).unwrap();
    cli::deploy_check(
        &file,
        Some(&repo_root().join("bundles/cloudspec-accepts.yaml")),
    )
}

#[test]
fn deploy_check_applies_the_image_rules_to_a_bundle_read_back() {
    let dir = scratch("deploy-images-check");
    let mut b = base();
    b["packing"]["units"] = json!([
        { "name": "core", "kind": "core", "parts": ["lead"] },
        { "name": "gateway", "kind": "service", "parts": ["gateway"] },
        { "name": "runtimes", "kind": "service", "parts": ["flow", "notifier"], "image": image("runner", 5) },
    ]);
    b["parts"]["flow"]["image"] = image("flow", 2);
    b["parts"]["notifier"] = json!({ "image": image("notifier", 4) });
    let v = admitted(&dir, &b);
    check(&dir, &bundle::write(&v)).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let cases: [(&[&str], Edit); 3] = [
        (
            &["unit_image_conflict"],
            Box::new(|d| {
                d["packing"]["units"][2]
                    .as_object_mut()
                    .unwrap()
                    .remove("image");
            }),
        ),
        (
            &["unit_image_missing"],
            Box::new(|d| {
                d["packing"]["units"][2]
                    .as_object_mut()
                    .unwrap()
                    .remove("image");
                for p in ["flow", "notifier"] {
                    d["parts"][p].as_object_mut().unwrap().remove("image");
                }
            }),
        ),
        // The local store also turns the core runtime into its config id, which the core unit's
        // registry reference no longer is.
        (
            &["image_unpinned", "unit_runtime_mismatch"],
            Box::new(|d| {
                d["target"] = json!({ "images": "local" });
                d["packing"]["units"][2]["image"] = json!({ "image": "runner" });
            }),
        ),
    ];
    for (want, edit) in cases {
        let r = check(&dir, &tampered(&v, edit)).unwrap_err();
        assert_eq!(r.code, "not_admitted", "{}", r.detail);
        let want: BTreeSet<String> = want.iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(names(&r.detail), want, "{}", r.detail);
    }
}

/// `bundle` changed by `edit`, its digest recomputed.
fn rewritten(bundle: &Value, edit: impl FnOnce(&mut Value)) -> String {
    let mut v = bundle.clone();
    edit(&mut v);
    v["digest"] = json!(bundle::recompute(&v));
    bundle::write(&v)
}

fn unit_index(v: &Value, name: &str) -> usize {
    v["units"]
        .as_array()
        .unwrap()
        .iter()
        .position(|u| u["name"] == name)
        .unwrap_or_else(|| panic!("no unit {name}"))
}

#[test]
fn two_images_under_one_digest_leave_a_units_runtime_uncarried_and_are_refused() {
    let dir = scratch("deploy-images-one-digest");
    let mut b = base();
    b["parts"]["flow"]["image"] = image("flow", 2);
    b["parts"]["notifier"] = json!({ "image": image("notifier", 2) });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, "unit_runtime_mismatch");
    assert!(
        r.detail.contains(&format!(
            "unit notifier: its code.runtime {} is no image the bundle's runtimes carry, and the binding resolves {}",
            reference("notifier", 2),
            reference("notifier", 2)
        )),
        "{}",
        r.detail
    );
    assert!(!r.detail.contains("unit flow"), "{}", r.detail);
    // The core runtime is carried first: a unit image of its digest under another name is not.
    let mut b = base();
    b["parts"]["flow"]["image"] = image("flow", 1);
    b["parts"]["notifier"] = json!({ "image": image("notifier", 4) });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, "unit_runtime_mismatch");
    assert!(r.detail.contains("unit flow:"), "{}", r.detail);
    // One image under one digest for two units is carried once and admitted.
    b["parts"]["flow"]["image"] = image("core", 1);
    admitted(&dir, &b);
}

#[test]
fn deploy_check_refuses_a_unit_runtime_that_is_not_its_bindings_image_or_not_carried() {
    let dir = scratch("deploy-images-runtime-check");
    let mut b = base();
    b["parts"]["flow"]["image"] = image("flow", 2);
    b["parts"]["notifier"] = json!({ "image": image("notifier", 4) });
    let v = admitted(&dir, &b);
    let (flow, gateway, core) = (
        unit_index(&v, "flow"),
        unit_index(&v, "gateway"),
        unit_index(&v, "core"),
    );
    check(&dir, &rewritten(&v, |_| {})).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let cases: [(String, Edit); 6] = [
        // Another image the bundle carries, not the one the binding resolves.
        (
            format!(
                "unit flow: its code.runtime is {}, and the binding resolves {}",
                reference("core", 1),
                reference("flow", 2)
            ),
            Box::new(move |v| v["units"][flow]["code"]["runtime"] = json!(reference("core", 1))),
        ),
        // The binding's image, which the bundle's runtimes no longer carry.
        (
            format!(
                "unit flow: its code.runtime {} is no image the bundle's runtimes carry, and the binding resolves {}",
                reference("flow", 2),
                reference("flow", 2)
            ),
            Box::new(|v| {
                v["runtimes"].as_object_mut().unwrap().remove(&digest(2));
            }),
        ),
        // An image for a unit the binding gives none, which names the role the target binds.
        (
            format!(
                "unit gateway: its code.runtime is {}, and the binding resolves none",
                reference("core", 1)
            ),
            Box::new(move |v| {
                v["units"][gateway]["code"] = json!({ "runtime": reference("core", 1) })
            }),
        ),
        // No image for a unit the binding gives one.
        (
            format!(
                "unit core: its code.runtime is none, and the binding resolves {}",
                reference("core", 1)
            ),
            Box::new(move |v| v["units"][core]["code"] = json!({ "role": "core-runtime" })),
        ),
        // The carried binding changed under its unit: another image of the unit's own.
        (
            format!(
                "unit flow: its code.runtime is {}, and the binding resolves {}",
                reference("flow", 2),
                reference("flow", 7)
            ),
            Box::new(|v| {
                v["binding"]["document"]["parts"]["flow"]["image"] = image("flow", 7);
                v["binding"]["digest"] =
                    json!(agenticspec_sim::deploy::digest(&v["binding"]["document"]));
            }),
        ),
        // From the local store an entry is run by `<image>:<tag>`; a unit naming its digest is
        // not that image.
        (
            "unit flow: its code.runtime is sha256:".to_owned(),
            Box::new(move |v| {
                let d = &mut v["binding"]["document"];
                d["target"] = json!({ "images": "local" });
                d["runtimes"]["core"] = json!({ "image": "core", "tag": "dev" });
                d["parts"]["flow"]["image"] = json!({ "image": "flow", "tag": "dev" });
                d["parts"]["notifier"]["image"] = json!({ "image": "notifier", "tag": "dev" });
                v["binding"]["digest"] = json!(agenticspec_sim::deploy::digest(d));
                v["runtimes"] = json!({
                    "core:dev": { "name": "core", "image": "core", "tag": "dev" },
                    "flow:dev": { "name": "flow", "image": "flow", "tag": "dev" },
                    "notifier:dev": { "name": "notifier", "image": "notifier", "tag": "dev" },
                });
                v["units"][core]["code"]["runtime"] = json!("core:dev");
                v["units"][flow]["code"]["runtime"] = json!(digest(2));
                let n = v["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(|u| u["name"] == "notifier")
                    .unwrap();
                v["units"][n]["code"]["runtime"] = json!("notifier:dev");
            }),
        ),
    ];
    for (needle, edit) in cases {
        let r = check(&dir, &rewritten(&v, edit)).unwrap_err();
        refused_by(&r, "unit_runtime_mismatch");
        assert!(r.detail.contains(&needle), "{needle} in {}", r.detail);
        assert_eq!(
            r.detail.matches("unit_runtime_mismatch").count(),
            1,
            "{}",
            r.detail
        );
    }
}
