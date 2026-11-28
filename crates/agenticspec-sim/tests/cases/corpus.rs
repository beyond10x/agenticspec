//! Task corpora: run entry by entry, and drawn from by a profile.

use super::common::{ARCH, PROFILE, repo_root, scratch};
use agenticspec_sim::reader::sha256_hex;
use agenticspec_sim::{Experiment, cli, run};
use std::path::Path;

fn doc_ref(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    format!(
        "{{ ref: {}, digest: {} }}",
        path.display(),
        sha256_hex(&bytes)
    )
}

fn corpus_run(path: &Path) -> (Result<serde_json::Value, cli::Refusal>, String) {
    let mut log = Vec::new();
    let r = cli::corpus("run", path, &mut log);
    (r, String::from_utf8(log).unwrap())
}

#[test]
fn the_example_corpus_runs_every_entry_and_predicts_its_class() {
    let (r, table) = corpus_run(&repo_root().join("scenarios/workstation/workstation.corpus.yaml"));
    let r = r.unwrap_or_else(|e| panic!("{e:?}\n{table}"));
    assert_eq!(r["entries"], 3);
    let rows = r["rows"].as_array().unwrap();
    let got: Vec<(&str, &str, &str)> = rows
        .iter()
        .map(|row| {
            (
                row["entry"].as_str().unwrap(),
                row["expected"].as_str().unwrap(),
                row["predicted"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            ("prompt-starts-run", "correct", "correct"),
            ("question-answered", "question", "question"),
            ("control-refused", "refused", "refused"),
        ]
    );
    assert!(table.starts_with("entry"), "{table}");
    assert!(table.contains("question-answered"), "{table}");
}

#[test]
fn an_inline_entry_runs_its_one_input_on_the_architecture_of_the_corpus() {
    let dir = scratch("corpus-inline");
    let path = dir.join("inline.corpus.yaml");
    std::fs::write(
        &path,
        format!(
            "format: agenticspec-corpus/1\ncorpus: inline\narchitecture: {}\nprofile: {}\nseed: 2\nhorizon: 1h\nentries:\n  - {{ id: s-1, kind: control, input_digest: {}, expected: refused }}\n  - {{ id: s-2, kind: order, input_digest: {}, expected: refused }}\n",
            doc_ref(&repo_root().join("architectures/workstation.yaml")),
            doc_ref(&repo_root().join("fields/coding/profiles/coding.yaml")),
            sha256_hex(b"run the release"),
            sha256_hex(b"ship it"),
        ),
    )
    .unwrap();
    let (r, table) = corpus_run(&path);
    let r = r.unwrap_or_else(|e| panic!("{e:?}\n{table}"));
    assert_eq!(r["rows"][0]["predicted"], "refused");
    assert_eq!(r["rows"][1]["kind"], "order");
}

#[test]
fn a_corpus_refuses_an_unknown_key_a_wrong_digest_and_an_inline_entry_it_cannot_run() {
    let dir = scratch("corpus-refused");
    let unknown = dir.join("unknown.corpus.yaml");
    std::fs::write(
        &unknown,
        "format: agenticspec-corpus/1\ncorpus: unknown\nentries:\n  - { id: a, kind: prompt, input_digest: 00, expected: correctly, colour: red }\n",
    )
    .unwrap();
    let err = corpus_run(&unknown).0.unwrap_err();
    assert_eq!(err.code, "corpus_invalid");
    for p in [
        "entries.0.colour: unknown key",
        "entries.0.input_digest: expected a sha256 digest",
        "entries.0.expected: expected one of",
    ] {
        assert!(err.detail.contains(p), "{p} not in {}", err.detail);
    }

    let scenario = repo_root().join("scenarios/workstation/control-refused.scenario.yaml");
    let wrong = dir.join("wrong.corpus.yaml");
    std::fs::write(
        &wrong,
        format!(
            "format: agenticspec-corpus/1\ncorpus: wrong\nentries:\n  - {{ scenario: {{ ref: {}, digest: {} }}, expected: refused }}\n",
            scenario.display(),
            sha256_hex(b"another file")
        ),
    )
    .unwrap();
    let err = corpus_run(&wrong).0.unwrap_err();
    assert_eq!(err.code, "corpus_invalid");
    assert!(err.detail.contains("scenario.digest"), "{}", err.detail);

    let alone = dir.join("alone.corpus.yaml");
    std::fs::write(
        &alone,
        format!(
            "format: agenticspec-corpus/1\ncorpus: alone\nentries:\n  - {{ id: s-1, kind: prompt, input_digest: {}, expected: correct }}\n",
            sha256_hex(b"x")
        ),
    )
    .unwrap();
    let err = corpus_run(&alone).0.unwrap_err();
    assert_eq!(err.code, "corpus_invalid");
    assert!(
        err.detail.contains("architecture and profile"),
        "{}",
        err.detail
    );
}

#[test]
fn a_corpus_entry_that_predicts_another_class_fails_the_run() {
    let dir = scratch("corpus-mismatch");
    let scenario = repo_root().join("scenarios/workstation/control-refused.scenario.yaml");
    let path = dir.join("mismatch.corpus.yaml");
    std::fs::write(
        &path,
        format!(
            "format: agenticspec-corpus/1\ncorpus: mismatch\nentries:\n  - {{ scenario: {}, expected: correct }}\n",
            doc_ref(&scenario)
        ),
    )
    .unwrap();
    let (r, table) = corpus_run(&path);
    let err = r.unwrap_err();
    assert_eq!(err.code, "corpus_failed");
    assert!(err.detail.contains("control-refused"), "{}", err.detail);
    assert!(table.contains("failed"), "{table}");
}

/// The small architecture and a profile that draws its tasks from a corpus of one class.
fn drawn(dir: &Path, class: &str) -> serde_json::Value {
    std::fs::write(dir.join("small.yaml"), ARCH).unwrap();
    std::fs::write(
        dir.join("corpus.yaml"),
        format!(
            "format: agenticspec-corpus/1\ncorpus: one-class\nentries:\n  - {{ id: a, kind: prompt, input_digest: {}, expected: {class} }}\n",
            sha256_hex(b"a")
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("profile.yaml"),
        PROFILE.replace(
            "load:\n  tasks_per_hour: 4",
            "load:\n  tasks_per_hour: 12\n  corpus: corpus.yaml",
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("exp.yaml"),
        "format: agenticspec-experiment/1\nexperiment: drawn\narchitecture: small.yaml\nprofile: profile.yaml\nhorizon: 4h\nseeds: 1\nseed: 5\nsearch:\n  mode: exhaustive\n  budget: 1\n  weights:\n    tasks_done: 1\n",
    )
    .unwrap();
    let e = Experiment::load(&dir.join("exp.yaml")).unwrap_or_else(|e| panic!("{e}"));
    run(&e, 1).unwrap()["baseline"]["kpis"].clone()
}

fn k(kpis: &serde_json::Value, name: &str) -> f64 {
    kpis[name]["value"].as_f64().unwrap()
}

#[test]
fn a_profile_draws_its_tasks_from_a_corpus() {
    let shipped_wrong = drawn(&scratch("corpus-drawn-false"), "false_completion");
    assert!(k(&shipped_wrong, "tasks_done") > 0.0);
    assert_eq!(
        k(&shipped_wrong, "false_completions_shipped"),
        k(&shipped_wrong, "tasks_done")
    );
    assert_eq!(k(&shipped_wrong, "questions_asked"), 0.0);

    let correct = drawn(&scratch("corpus-drawn-correct"), "correct");
    assert!(k(&correct, "tasks_done") > 0.0);
    assert_eq!(k(&correct, "tasks_correct"), k(&correct, "tasks_done"));

    let refused = drawn(&scratch("corpus-drawn-refused"), "refused");
    assert_eq!(k(&refused, "tasks_arrived"), 0.0);
}

#[test]
fn a_profile_naming_a_corpus_needs_the_file_it_is_read_from() {
    let profile = PROFILE.replace(
        "load:\n  tasks_per_hour: 4",
        "load:\n  tasks_per_hour: 4\n  corpus: corpus.yaml",
    );
    let err = match Experiment::from_docs(
        super::common::yaml(&super::common::experiment_yaml("fit", 1, 16)),
        super::common::yaml(ARCH),
        super::common::yaml(&profile),
    ) {
        Err(e) => e,
        Ok(e) => run(&e, 1).unwrap_err(),
    };
    assert!(err.contains("load.corpus"), "{err}");
}
