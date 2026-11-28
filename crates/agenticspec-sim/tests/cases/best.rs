//! The best variant as an architecture document (`best.yaml`): markers set to the best values and
//! kept, accepted by the check, and reproducing the best fitness under the same experiment.

use super::common::{ARCH, PROFILE, experiment, experiment_yaml, repo_root, scratch, yaml};
use agenticspec_sim::best::{self, fitness_as_written};
use agenticspec_sim::check::check;
use agenticspec_sim::cli;
use agenticspec_sim::doc::{load_yaml, parse_yaml};
use agenticspec_sim::marked::collect;
use agenticspec_sim::{Experiment, run};
use serde_json::Value;
#[test]
fn apply_sets_each_marker_to_its_best_value_and_keeps_vary() {
    let arch = yaml(ARCH);
    let params = serde_json::json!({
        "parts.worker.count": 6,
        "parts.builds.capacity": 4,
        "policies.question_default_after": "5m",
    });
    let out = best::apply(&arch, params.as_object().unwrap()).unwrap();
    assert_eq!(out["parts"]["worker"]["count"]["value"], 6);
    assert_eq!(
        out["parts"]["worker"]["count"]["vary"],
        arch["parts"]["worker"]["count"]["vary"]
    );
    assert_eq!(out["parts"]["builds"]["capacity"]["value"], 4);
    assert_eq!(out["policies"]["question_default_after"]["value"], "5m");
    assert_eq!(out["parts"]["gateway"], arch["parts"]["gateway"]);
    assert_eq!(
        collect(&out, false).unwrap().len(),
        collect(&arch, false).unwrap().len()
    );
}

#[test]
fn apply_refuses_a_place_the_architecture_does_not_have() {
    let params = serde_json::json!({"parts.nobody.count": 2});
    let err = best::apply(&yaml(ARCH), params.as_object().unwrap()).unwrap_err();
    assert!(err.contains("parts.nobody.count"), "{err}");
}

#[test]
fn every_architecture_is_written_as_yaml_that_reads_back_unchanged() {
    for entry in std::fs::read_dir(repo_root().join("architectures")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "yaml") {
            continue;
        }
        let doc = load_yaml(&path).unwrap();
        let source = std::fs::read_to_string(&path).unwrap();
        for text in [best::yaml(&doc), best::yaml_like(&doc, &source)] {
            let text = text.unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert_eq!(parse_yaml(&text).unwrap(), doc, "{}", path.display());
            assert!(
                text.starts_with("format: agenticspec-architecture/1\n"),
                "{text}"
            );
        }
    }
}

#[test]
fn written_like_its_source_the_parts_keep_their_order() {
    let source =
        std::fs::read_to_string(repo_root().join("architectures/workstation.yaml")).unwrap();
    let text = best::yaml_like(&parse_yaml(&source).unwrap(), &source).unwrap();
    let at = |name: &str| {
        text.find(&format!("\n  {name}:"))
            .unwrap_or_else(|| panic!("{name}: {text}"))
    };
    assert!(
        at("owner") < at("operator") && at("operator") < at("worker") && at("worker") < at("box"),
        "{text}"
    );
    assert!(text.contains("{ block: agent, harness: coding, count: { value: 12, vary: { min: 2, max: 24, step: 2 } } }"), "{text}");
}

#[test]
fn written_like_its_source_the_view_key_is_kept_unchanged() {
    let source =
        std::fs::read_to_string(repo_root().join("architectures/workstation.yaml")).unwrap();
    let doc = parse_yaml(&source).unwrap();
    assert!(doc["view"].is_object(), "workstation.yaml has a view key");
    let text = best::yaml_like(&doc, &source).unwrap();
    assert_eq!(parse_yaml(&text).unwrap()["view"], doc["view"], "{text}");
    assert!(text.contains("\nview:"), "{text}");
}

#[test]
fn no_best_variant_means_no_document() {
    let e = experiment("fit", 4, 256);
    let mut result = run(&e, 2).unwrap();
    result["best"] = Value::Null;
    assert_eq!(best::yaml_of_result(&result, None).unwrap(), None);
}

/// Runs the experiment as configured, writes its best variant as YAML, reads it back, checks it,
/// and runs the experiment again on it: its fitness is the best fitness of the first run. A hill
/// climb is held to 30 evaluations here so the test stays under 5 s; `task catalog:generate` checks the
/// same at the full budget for every catalog entry.
fn reproduces(file: &str) {
    let mut e = Experiment::load(&repo_root().join("experiments").join(file)).unwrap();
    e.search.budget = e.search.budget.min(30);
    let result = run(&e, 4).unwrap();
    let text = best::yaml_of_result(&result, None)
        .unwrap()
        .expect("a best variant");
    let doc = parse_yaml(&text).unwrap();
    let refusals = check(&doc);
    assert!(refusals.is_empty(), "{file}: {refusals:?}");
    assert_eq!(
        collect(&doc, false).unwrap().len(),
        e.marked.len(),
        "{file}: the markers are kept"
    );
    let want = result["best"]["fitness"].as_f64().unwrap();
    let got = fitness_as_written(&e, doc, 4).unwrap();
    assert_eq!(got, want, "{file}");
}

#[test]
fn best_of_orchestrator_workers_tuning_reproduces_its_fitness() {
    reproduces("orchestrator-workers-tuning.yaml");
}

#[test]
fn best_of_reviewer_tuning_reproduces_its_fitness() {
    reproduces("reviewer-tuning.yaml");
}

#[test]
fn best_of_scheduled_loops_tuning_reproduces_its_fitness() {
    reproduces("scheduled-loops-tuning.yaml");
}

#[test]
fn best_of_scope_partition_tuning_reproduces_its_fitness() {
    reproduces("scope-partition-tuning.yaml");
}

#[test]
fn best_of_shared_board_tuning_reproduces_its_fitness() {
    reproduces("shared-board-tuning.yaml");
}

#[test]
fn best_of_workstation_tuning_reproduces_its_fitness() {
    reproduces("workstation-tuning.yaml");
}

#[test]
fn run_writes_best_yaml_that_check_accepts() {
    let dir = scratch("run-best");
    std::fs::write(dir.join("small.yaml"), ARCH).unwrap();
    std::fs::write(dir.join("small-profile.yaml"), PROFILE).unwrap();
    std::fs::write(dir.join("exp.yaml"), experiment_yaml("fit", 6, 256)).unwrap();
    let out_dir = dir.join("out");
    let mut log = Vec::new();
    let ran = cli::run(&dir.join("exp.yaml"), &out_dir, Some(2), false, &mut log)
        .unwrap_or_else(|e| panic!("{e:?}\n{}", String::from_utf8_lossy(&log)));
    assert_eq!(ran["best_written"], true);
    let best_file = out_dir.join("best.yaml");
    let text = std::fs::read_to_string(&best_file).unwrap();
    assert!(text.contains("vary"), "{text}");
    cli::check(&best_file).unwrap();
    let result: Value =
        serde_json::from_str(&std::fs::read_to_string(out_dir.join("result.json")).unwrap())
            .unwrap();
    let doc = parse_yaml(&text).unwrap();
    for (path, value) in result["best"]["params"].as_object().unwrap() {
        let at = path.split('.').fold(&doc, |v, k| &v[k]);
        assert_eq!(&at["value"], value, "{path}");
    }
}
