//! The non-secret configuration a binding gives a unit (R87): `env` and `args` on a part and on a
//! unit `packing.units` lists, merged per unit into the bundle's `config` (the unit first, then
//! its parts by name), a value `{secret: <name>}` carried as a reference through the unit's
//! secrets, the refusals `unit_config_invalid` and `unit_config_conflict`, and `deploy check`
//! reading `config` back.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, BundleOptions};
use agenticspec_sim::deploy::bundle;
use agenticspec_sim::deploy::config::is_env_name;
use agenticspec_sim::doc::parse_yaml;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The architecture with two runtime parts, `flow` and `notifier`, and a gateway.
fn architecture() -> PathBuf {
    repo_root()
        .join("crates/agenticspec-sim/tests/cases/deploy/unit_image_missing.architecture.yaml")
}

fn image(name: &str, n: u32) -> Value {
    json!({ "image": format!("registry.example/{name}"), "digest": format!("sha256:{n:064x}") })
}

/// A binding of the architecture: the lead in the core, the gateway a service unit of its own,
/// both runtimes in one service unit with one image; every destination declared.
fn base() -> Value {
    json!({
        "format": "agenticspec-binding/1",
        "binding": "config",
        "architecture": "runtimes",
        "transports": { "run": "http", "model_call": "http" },
        "runtimes": { "core": image("core", 1) },
        "parts": {
            "lead": { "egress": ["flow", "gateway", "notifier"] },
            "flow": { "egress": ["gateway"], "image": image("runner", 2) },
            "notifier": { "image": image("runner", 2) },
            "gateway": { "egress": ["model"] },
        },
        "packing": {
            "units": [
                { "name": "core", "kind": "core", "parts": ["lead"] },
                { "name": "gateway", "kind": "service", "parts": ["gateway"] },
                { "name": "runtimes", "kind": "service", "parts": ["flow", "notifier"] },
            ],
            "external": ["owner", "model"],
        },
    })
}

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

fn unit_index(v: &Value, name: &str) -> usize {
    v["units"]
        .as_array()
        .unwrap()
        .iter()
        .position(|u| u["name"] == name)
        .unwrap_or_else(|| panic!("no unit {name}"))
}

fn names(detail: &str) -> BTreeSet<String> {
    detail
        .split("; ")
        .map(|f| f.split(':').next().unwrap_or("").to_owned())
        .collect()
}

fn refused_by(r: &cli::Refusal, want: &[&str]) {
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    let want: BTreeSet<String> = want.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(names(&r.detail), want, "{}", r.detail);
}

#[test]
fn a_server_unit_is_told_the_host_names_to_allow() {
    let dir = scratch("deploy-config-hosts");
    let mut b = base();
    b["parts"]["gateway"]["env"] = json!({ "ALLOWED_HOSTS": "svc.ns.svc,localhost" });
    let v = admitted(&dir, &b);
    assert_eq!(
        unit(&v, "gateway")["config"],
        json!({ "env": { "ALLOWED_HOSTS": "svc.ns.svc,localhost" } })
    );
    for u in ["core", "runtimes"] {
        assert!(
            unit(&v, u).get("config").is_none(),
            "a unit given nothing carries no config: {u}"
        );
    }
    let text = bundle::write(&v);
    assert!(
        text.contains("\n  config:\n    env:\n      ALLOWED_HOSTS: svc.ns.svc,localhost\n"),
        "{text}"
    );
    check(&dir, &text).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
}

#[test]
fn the_config_merges_the_unit_first_then_its_parts_by_name_and_takes_scalars_as_text() {
    let dir = scratch("deploy-config-merge");
    let mut b = base();
    b["packing"]["units"][2]["env"] = json!({ "MODE": "batch", "WORKERS": 4 });
    b["packing"]["units"][2]["args"] = json!(["--log-format", "json"]);
    b["parts"]["notifier"]["env"] = json!({ "DRY_RUN": false, "MODE": "batch", "RATIO": 0.5 });
    b["parts"]["notifier"]["args"] = json!(["--channel", "ops"]);
    b["parts"]["flow"]["env"] = json!({ "_TRACE": true, "A1": "x" });
    let v = admitted(&dir, &b);
    let config = &unit(&v, "runtimes")["config"];
    assert_eq!(
        *config,
        json!({
            "env": {
                "A1": "x", "DRY_RUN": "false", "MODE": "batch", "RATIO": "0.5",
                "WORKERS": "4", "_TRACE": "true",
            },
            "args": ["--log-format", "json", "--channel", "ops"],
        })
    );
    let keys: Vec<&String> = config["env"].as_object().unwrap().keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "env keys sorted");
    // The same inputs give the same bytes.
    assert_eq!(bundle::write(&admitted(&dir, &b)), bundle::write(&v));
    check(&dir, &bundle::write(&v)).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
}

#[test]
fn two_sources_of_one_unit_that_disagree_are_refused_as_unit_config_conflict() {
    let dir = scratch("deploy-config-conflict");
    let mut b = base();
    b["packing"]["units"][2]["env"] = json!({ "MODE": "batch" });
    b["parts"]["flow"]["env"] = json!({ "MODE": "stream" });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, &["unit_config_conflict"]);
    assert!(
        r.detail.contains(
            "unit runtimes: env MODE is given two different values, by packing: unit runtimes and by parts: flow"
        ),
        "{}",
        r.detail
    );
    assert!(
        !r.detail.contains("batch") && !r.detail.contains("stream"),
        "names no value: {}",
        r.detail
    );
    // A number and its text are one value.
    b["packing"]["units"][2]["env"] = json!({ "PORT": 8080 });
    b["parts"]["flow"]["env"] = json!({ "PORT": "8080" });
    admitted(&dir, &b);
    // Two parts that give args, even the same ones, conflict; a unit's args and one part's do not.
    b["parts"]["flow"]["args"] = json!(["--x"]);
    b["parts"]["notifier"]["args"] = json!(["--x"]);
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, &["unit_config_conflict"]);
    assert!(
        r.detail
            .contains("unit runtimes: parts: flow and parts: notifier both give args"),
        "{}",
        r.detail
    );
    b["parts"]["notifier"]
        .as_object_mut()
        .unwrap()
        .remove("args");
    b["packing"]["units"][2]["args"] = json!(["--y"]);
    let v = admitted(&dir, &b);
    assert_eq!(
        unit(&v, "runtimes")["config"]["args"],
        json!(["--y", "--x"])
    );
}

#[test]
fn a_configuration_of_the_wrong_form_is_refused_as_unit_config_invalid() {
    let dir = scratch("deploy-config-invalid");
    let cases: [(&str, Value, &str); 9] = [
        (
            "env",
            json!({ "allowed_hosts": "a" }),
            "env: `allowed_hosts` is not an environment variable name",
        ),
        (
            "env",
            json!({ "9LIVES": "a" }),
            "env: `9LIVES` is not an environment variable name",
        ),
        (
            "env",
            json!({ "HOSTS": ["a", "b"] }),
            "env: HOSTS is text, a number, a boolean or {secret: <name>}",
        ),
        ("env", json!({ "HOSTS": null }), "env: HOSTS is text"),
        (
            "env",
            json!({ "KEY": { "secret": "api-key", "from": "vault" } }),
            "env: KEY is text",
        ),
        ("env", json!(["HOSTS=a"]), "env is a mapping"),
        ("args", json!("--verbose"), "args is a list of strings"),
        ("args", json!(["--port", 8080]), "args is a list of strings"),
        (
            "args",
            json!([{ "flag": "x" }]),
            "args is a list of strings",
        ),
    ];
    for (key, value, needle) in cases {
        let mut b = base();
        b["parts"]["gateway"][key] = value.clone();
        let r = bundle_of(&dir, &b).unwrap_err();
        refused_by(&r, &["unit_config_invalid"]);
        assert!(
            r.detail.contains(&format!("parts: gateway: {needle}")),
            "{key}: {value}: {}",
            r.detail
        );
    }
    let mut b = base();
    b["packing"]["units"][1]["env"] = json!({ "bad-name": "x" });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, &["unit_config_invalid"]);
    assert!(
        r.detail.contains("packing: unit gateway: env: `bad-name`"),
        "{}",
        r.detail
    );
    // A part outside every unit has no unit to configure.
    let mut b = base();
    b["parts"]["model"] = json!({ "env": { "A": "b" } });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, &["unit_config_invalid"]);
    assert!(
        r.detail
            .contains("parts: model: env is for a part of a unit, and model is in no unit"),
        "{}",
        r.detail
    );
    for (name, ok) in [
        ("ALLOWED_HOSTS", true),
        ("_X", true),
        ("A1", true),
        ("", false),
        ("1A", false),
        ("a", false),
        ("A-B", false),
        ("A B", false),
    ] {
        assert_eq!(is_env_name(name), ok, "{name:?}");
    }
}

#[test]
fn a_secret_reference_goes_through_the_secrets_of_its_unit_and_is_never_plain() {
    let dir = scratch("deploy-config-secret");
    let mut b = base();
    b["parts"]["gateway"]["env"] = json!({ "MODEL_KEY": { "secret": "model-api-key" } });
    let r = bundle_of(&dir, &b).unwrap_err();
    refused_by(&r, &["secret_undeclared"]);
    assert!(
        r.detail.contains(
            "parts: gateway: env MODEL_KEY refers to secret model-api-key, and no part of unit gateway declares it"
        ),
        "{}",
        r.detail
    );
    // Declared by a part of another unit is not enough.
    b["parts"]["lead"]["secrets"] = json!(["model-api-key"]);
    refused_by(&bundle_of(&dir, &b).unwrap_err(), &["secret_undeclared"]);
    b["parts"]["gateway"]["secrets"] = json!(["model-api-key"]);
    let v = admitted(&dir, &b);
    let g = unit(&v, "gateway");
    assert_eq!(
        g["config"],
        json!({ "env": { "MODEL_KEY": { "secret": "model-api-key" } } })
    );
    assert_eq!(g["secrets"], json!(["model-api-key"]));
    check(&dir, &bundle::write(&v)).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    // A reference on a listed unit is checked against the unit's parts the same way.
    let mut b = base();
    b["packing"]["units"][2]["env"] = json!({ "TOKEN": { "secret": "runner-token" } });
    refused_by(&bundle_of(&dir, &b).unwrap_err(), &["secret_undeclared"]);
    b["parts"]["notifier"]["secrets"] = json!(["runner-token"]);
    admitted(&dir, &b);
    // What looks like a value where the secret's name belongs is refused as one, never shown.
    for value in [
        json!({ "secret": "Not A Name: 0123 4567" }),
        json!({ "secret": 42 }),
    ] {
        let mut b = base();
        b["parts"]["gateway"]["env"] = json!({ "MODEL_KEY": value });
        let r = bundle_of(&dir, &b).unwrap_err();
        refused_by(&r, &["secret_value"]);
        assert!(
            r.detail
                .contains("parts: gateway: env: MODEL_KEY: secret is not a secret name"),
            "{}",
            r.detail
        );
        assert!(!r.detail.contains("sk-ABC"), "{}", r.detail);
    }
}

/// `bundle` changed by `edit`, its digest recomputed.
fn rewritten(bundle: &Value, edit: impl FnOnce(&mut Value)) -> String {
    let mut v = bundle.clone();
    edit(&mut v);
    v["binding"]["digest"] = json!(agenticspec_sim::deploy::digest(&v["binding"]["document"]));
    v["digest"] = json!(bundle::recompute(&v));
    bundle::write(&v)
}

fn check(dir: &Path, text: &str) -> Result<Value, cli::Refusal> {
    let file = dir.join("bundle.yaml");
    std::fs::write(&file, text).unwrap();
    cli::deploy_check(
        &file,
        Some(&repo_root().join("bundles/cloudspec-accepts.yaml")),
    )
}

type Edit = Box<dyn FnOnce(&mut Value)>;

#[test]
fn deploy_check_reads_config_back_and_refuses_a_malformed_one() {
    let dir = scratch("deploy-config-check");
    let mut b = base();
    b["parts"]["gateway"]["env"] = json!({
        "ALLOWED_HOSTS": "svc.ns.svc,localhost",
        "MODEL_KEY": { "secret": "model-api-key" },
    });
    b["parts"]["gateway"]["secrets"] = json!(["model-api-key"]);
    b["parts"]["gateway"]["args"] = json!(["--serve"]);
    let v = admitted(&dir, &b);
    check(&dir, &bundle::write(&v)).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let g = unit_index(&v, "gateway");
    let c = unit_index(&v, "core");
    let cases: Vec<(&[&str], &str, Edit)> = vec![
        (
            &["unit_config_invalid"],
            "unit gateway: config: env: ALLOWED_HOSTS is text or {secret: <name>}",
            Box::new(move |v| v["units"][g]["config"]["env"]["ALLOWED_HOSTS"] = json!(["a"])),
        ),
        (
            &["unit_config_invalid"],
            "unit gateway: config: env: `allowed_hosts` is not an environment variable name",
            Box::new(move |v| v["units"][g]["config"]["env"]["allowed_hosts"] = json!("a")),
        ),
        (
            &["unit_config_invalid"],
            "unit gateway: config: args is a list of strings that is not empty",
            Box::new(move |v| v["units"][g]["config"]["args"] = json!([])),
        ),
        (
            &["unit_config_invalid"],
            "unit gateway: config: unknown key `command`",
            Box::new(move |v| v["units"][g]["config"]["command"] = json!("sh")),
        ),
        (
            &["unit_config_invalid"],
            "unit core: config is empty",
            Box::new(move |v| v["units"][c]["config"] = json!({})),
        ),
        (
            &["unit_config_invalid"],
            "unit gateway: carries a config, and its binding resolves another",
            Box::new(move |v| {
                v["units"][g]["config"]["env"]["ALLOWED_HOSTS"] = json!("evil.example")
            }),
        ),
        (
            &["unit_config_invalid"],
            "unit gateway: carries no config, and its binding resolves one",
            Box::new(move |v| {
                v["units"][g].as_object_mut().unwrap().remove("config");
            }),
        ),
        (
            &["unit_config_invalid"],
            "unit core: carries a config, and its binding resolves none",
            Box::new(move |v| v["units"][c]["config"] = json!({ "args": ["--debug"] })),
        ),
        (
            &["unit_config_invalid"],
            "unit gateway: carries a config, and its binding resolves another",
            Box::new(move |v| {
                v["binding"]["document"]["parts"]["gateway"]["env"]["ALLOWED_HOSTS"] =
                    json!("localhost")
            }),
        ),
        (
            &["unit_config_conflict"],
            "unit gateway: env ALLOWED_HOSTS is given two different values, by packing: unit gateway and by parts: gateway",
            Box::new(move |v| {
                v["binding"]["document"]["packing"]["units"][1]["env"] =
                    json!({ "ALLOWED_HOSTS": "localhost" })
            }),
        ),
        (
            &["secret_undeclared", "unit_config_invalid"],
            "unit gateway: config: env: MODEL_KEY: secret is secret other-key, and no part of unit gateway declares it",
            Box::new(move |v| {
                v["units"][g]["config"]["env"]["MODEL_KEY"] = json!({ "secret": "other-key" })
            }),
        ),
        (
            &["secret_value", "unit_config_invalid"],
            "unit gateway: config: env: MODEL_KEY: secret is not a secret name",
            Box::new(move |v| {
                v["units"][g]["config"]["env"]["MODEL_KEY"] = json!({ "secret": "Hunter2Hunter2" })
            }),
        ),
    ];
    for (want, needle, edit) in cases {
        let r = check(&dir, &rewritten(&v, edit)).unwrap_err();
        refused_by(&r, want);
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
}
