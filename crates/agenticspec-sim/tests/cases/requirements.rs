//! Requirements (`requirements:`): the form and the unknown-KPI refusal of `sim check`, the
//! state of each requirement at check (met, missed, undecided), the requirements a bundle carries
//! with their state, and the search, which treats them as constraints before weights and stops
//! when all are met.

use super::common::{ARCH, PROFILE, experiment_yaml, repo_root, scratch, yaml};
use agenticspec_sim::cli::{self, BundleOptions, CheckOptions};
use agenticspec_sim::deploy::bundle;
use agenticspec_sim::doc::{load_yaml, parse_yaml};
use agenticspec_sim::requirements::{RULES, Stage, stage};
use agenticspec_sim::{Experiment, nest, run};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The workstation (13 agent copies) with `requirements` appended, written to `dir`.
fn workstation_with(dir: &Path, requirements: &str) -> PathBuf {
    let text = std::fs::read_to_string(repo_root().join("architectures/workstation.yaml")).unwrap();
    let file = dir.join("workstation.yaml");
    std::fs::write(&file, format!("{text}\nrequirements:\n{requirements}")).unwrap();
    file
}

fn check(path: &Path) -> Result<Value, cli::Refusal> {
    let root = repo_root();
    cli::check_with(&CheckOptions {
        architecture: path,
        ports: Some(&root.join("blocks/ports.json")),
        kpis: Some(&root.join("kpis/catalogue.yaml")),
    })
}

fn by_kpi<'a>(list: &'a Value, kpi: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|r| r["kpi"] == kpi)
        .unwrap_or_else(|| panic!("no requirement {kpi} in {list}"))
}

#[test]
fn sim_check_refuses_an_unknown_kpi_id_naming_it() {
    let dir = scratch("requirements-unknown");
    let file = workstation_with(&dir, "  - { kpi: no_such_kpi, at_least: 1 }\n");
    let r = check(&file).unwrap_err();
    assert_eq!(r.code, "architecture_refused", "{}", r.detail);
    assert!(
        r.detail
            .starts_with("requirement_kpi_unknown: requirements[0]:"),
        "{}",
        r.detail
    );
    assert!(r.detail.contains("no_such_kpi"), "{}", r.detail);
}

#[test]
fn sim_check_refuses_a_requirement_of_the_wrong_form_each_by_name() {
    let dir = scratch("requirements-form");
    for (entry, needle) in [
        ("  - { kpi: agents_total }\n", "sets no target"),
        (
            "  - { kpi: agents_total, at_least: 1, at_most: 4 }\n",
            "sets 2 targets",
        ),
        ("  - { kpi: agents_total, at_most: four }\n", "not a number"),
        ("  - { at_most: 4 }\n", "names no `kpi`"),
        (
            "  - { kpi: tasks_done, at_least: 1, part: nobody }\n",
            "`nobody`, which is no part",
        ),
        (
            "  - { kpi: tasks_done, at_least: 1, window: soon }\n",
            "not a duration",
        ),
        (
            "  - { kpi: agents_total, at_most: 4, part: worker }\n",
            "has no value per part",
        ),
        (
            "  - { kpi: agents_total, at_most: 4, over: 1h }\n",
            "unknown key `over`",
        ),
        ("  - agents_total\n", "is a map"),
    ] {
        let file = workstation_with(&dir, entry);
        let r = check(&file).unwrap_err();
        assert_eq!(r.code, "architecture_refused", "{entry}: {}", r.detail);
        assert!(
            r.detail
                .starts_with("requirement_invalid: requirements[0]:"),
            "{entry}: {}",
            r.detail
        );
        assert!(
            r.detail.contains(needle),
            "{entry}: {needle} in {}",
            r.detail
        );
    }
    assert_eq!(RULES, ["requirement_invalid", "requirement_kpi_unknown"]);
}

#[test]
fn sim_check_decides_a_design_requirement_and_carries_every_other_undecided() {
    let dir = scratch("requirements-check");
    let file = workstation_with(
        &dir,
        "  - { kpi: agents_total, at_most: 4 }\n  - { kpi: capacity_ratio, at_most: 1 }\n  - { kpi: unknown_link_parts, equals: 0 }\n  - { kpi: tasks_done, at_least: 10, part: worker }\n  - { kpi: cache_hit_share, at_least: 0.7, window: 1h }\n  - { kpi: \"cloudspec:node_count\", at_most: 3 }\n",
    );
    let out = check(&file).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let reqs = &out["requirements"];
    assert_eq!(reqs.as_array().unwrap().len(), 6);
    assert_eq!(
        by_kpi(reqs, "agents_total"),
        &json!({"kpi": "agents_total", "at_most": 4, "stage": "check", "state": "missed", "value": 13})
    );
    let capacity = by_kpi(reqs, "capacity_ratio");
    assert_eq!(
        (&capacity["state"], &capacity["value"]),
        (&json!("met"), &json!(0.75))
    );
    assert_eq!(by_kpi(reqs, "unknown_link_parts")["state"], "met");
    for (kpi, stage) in [
        ("tasks_done", "evaluation"),
        ("cache_hit_share", "operation"),
        ("cloudspec:node_count", "cloud"),
    ] {
        let r = by_kpi(reqs, kpi);
        assert_eq!(
            (&r["stage"], &r["state"]),
            (&json!(stage), &json!("undecided")),
            "{r}"
        );
        assert!(r["value"].is_null(), "{r}");
    }
    assert_eq!(by_kpi(reqs, "cache_hit_share")["window"], "1h");
    let plain = check(&repo_root().join("architectures/workstation.yaml")).unwrap();
    assert_eq!(plain["requirements"], json!([]));
}

#[test]
fn a_kpi_is_decided_at_the_stage_it_can_be_read() {
    let kpis =
        agenticspec_sim::contract::ingress::Kpis::load(&repo_root().join("kpis/catalogue.yaml"))
            .unwrap();
    for (kpi, want) in [
        ("agents_total", Some(Stage::Check)),
        ("unreachable_parts", Some(Stage::Check)),
        ("cost_usd", Some(Stage::Evaluation)),
        ("cache_hit_share", Some(Stage::Operation)),
        ("cloudspec:cost_per_hour", Some(Stage::Cloud)),
        ("cloudspec:", None),
        ("no_such_kpi", None),
    ] {
        assert_eq!(stage(kpi, &kpis), want, "{kpi}");
    }
}

#[test]
fn requirements_stay_outside_the_nesting_digest_and_the_revision_digest() {
    let doc = load_yaml(&repo_root().join("architectures/console.yaml")).unwrap();
    let mut with = doc.clone();
    with["requirements"] = json!([{ "kpi": "agents_total", "at_most": 2 }]);
    assert_eq!(nest::digest(&with), nest::digest(&doc));
    assert_eq!(
        bundle::revision_digest(&with),
        bundle::revision_digest(&doc)
    );
}

fn bundle_of(architecture: &Path) -> Result<Value, cli::Refusal> {
    let root = repo_root();
    cli::deploy_bundle(&BundleOptions {
        architecture,
        binding: &root.join("bundles/bindings/local-workstation.yaml"),
        out: None,
        ports: Some(&root.join("blocks/ports.json")),
        accepts: Some(&root.join("bundles/cloudspec-accepts.yaml")),
        kpis: Some(&root.join("kpis/catalogue.yaml")),
    })
}

#[test]
fn deploy_bundle_writes_each_requirement_with_its_state_and_deploy_check_admits_it() {
    let dir = scratch("requirements-bundle");
    let file = workstation_with(
        &dir,
        "  - { kpi: agents_total, at_least: 10 }\n  - { kpi: agents_total, at_most: 4 }\n  - { kpi: cost_usd, at_most: 20 }\n",
    );
    let out = bundle_of(&file).unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    assert_eq!(
        v["requirements"],
        json!([
            {"kpi": "agents_total", "at_least": 10, "stage": "check", "state": "met", "value": 13},
            {"kpi": "agents_total", "at_most": 4, "stage": "check", "state": "missed", "value": 13},
            {"kpi": "cost_usd", "at_most": 20, "stage": "evaluation", "state": "undecided"},
        ])
    );
    let written = dir.join("bundle.yaml");
    std::fs::write(&written, out["bundle"].as_str().unwrap()).unwrap();
    cli::deploy_check(
        &written,
        Some(&repo_root().join("bundles/cloudspec-accepts.yaml")),
    )
    .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
}

#[test]
fn deploy_bundle_refuses_an_unknown_requirement_kpi_as_a_finding_and_writes_nothing() {
    let dir = scratch("requirements-bundle-unknown");
    let file = workstation_with(
        &dir,
        "  - { kpi: no_such_kpi, at_least: 1 }\n  - { kpi: agents_total }\n",
    );
    let r = bundle_of(&file).unwrap_err();
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    assert!(
        r.detail
            .contains("requirement_kpi_unknown: requirements[0]:"),
        "{}",
        r.detail
    );
    assert!(
        r.detail.contains("requirement_invalid: requirements[1]:"),
        "{}",
        r.detail
    );
}

/// The small experiment of `common` in `mode`, its architecture with `requirements`.
fn experiment_with(mode: &str, budget: usize, requirements: Value) -> Experiment {
    let mut arch = yaml(ARCH);
    arch["requirements"] = requirements;
    Experiment::from_docs(
        yaml(&experiment_yaml(mode, budget, 256)),
        arch,
        yaml(PROFILE),
    )
    .expect("experiment loads")
}

#[test]
fn the_search_stops_at_the_first_variant_that_meets_every_requirement() {
    let e = experiment_with(
        "fit",
        16,
        json!([{ "kpi": "tasks_arrived", "at_least": 0 }]),
    );
    let r = run(&e, 2).unwrap();
    assert_eq!(
        r["variants"].as_array().unwrap().len(),
        1,
        "the baseline meets it; nothing else is evaluated"
    );
    let met = &r["best"]["requirements"][0];
    assert_eq!(
        (&met["kpi"], &met["stage"], &met["state"]),
        (&json!("tasks_arrived"), &json!("evaluation"), &json!("met"))
    );
    assert_eq!(r["requirements"], r["best"]["requirements"]);
    assert_eq!(r["variants"][0]["requirements_missed"], 0);
}

#[test]
fn the_search_ranks_requirements_before_weights() {
    // Workers are 2, 4 or 6 copies beside the operator: agents_total 3, 5 or 7. The weights
    // favour more tasks done; the requirement holds the team at three copies.
    let e = experiment_with(
        "exhaustive",
        0,
        json!([{ "kpi": "agents_total", "at_most": 3 }]),
    );
    let r = run(&e, 4).unwrap();
    let best = &r["best"];
    assert_eq!(
        best["requirements"][0]["state"], "met",
        "{}",
        best["requirements"]
    );
    assert_eq!(
        best["params"]["parts.worker.count"], 2,
        "{}",
        best["params"]
    );
    let free = run(&super::common::experiment("exhaustive", 0, 100), 4).unwrap();
    assert!(
        free["best"]["fitness"].as_f64().unwrap() >= best["fitness"].as_f64().unwrap(),
        "the constraint costs fitness or nothing"
    );
    assert!(free.get("requirements").is_none());
    assert!(free["variants"][0].get("requirements_missed").is_none());
}

#[test]
fn an_experiment_refuses_a_requirement_of_the_wrong_form() {
    let mut arch = yaml(ARCH);
    arch["requirements"] = json!([{ "kpi": "tasks_done" }]);
    let err = Experiment::from_docs(yaml(&experiment_yaml("fit", 4, 256)), arch, yaml(PROFILE))
        .unwrap_err();
    assert!(
        err.starts_with("requirement_invalid: requirements[0]:"),
        "{err}"
    );
}

#[test]
fn optimize_all_leaves_a_requirements_target_out_of_the_search() {
    let arch = yaml(
        "format: agenticspec-architecture/1\narchitecture: t\nparts:\n  owner: { block: principal, kind: human }\nrequirements:\n  - { kpi: agents_total, at_most: 4 }\n",
    );
    let marked = agenticspec_sim::marked::collect(&arch, true).unwrap();
    assert!(
        marked.iter().all(|m| !m.path.starts_with("requirements")),
        "{:?}",
        marked.iter().map(|m| &m.path).collect::<Vec<_>>()
    );
}
