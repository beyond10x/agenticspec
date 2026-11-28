//! The fact base of an architecture: the relations `part`, `setting`, `link` and `marker`, beside
//! the session relations and apart from them.

use agenticspec_records::sq::architecture::{self, Facts, Link, Marker, Part, Setting};
use agenticspec_records::sq::eval;
use agenticspec_records::sq::facts::{Val, vs};

fn facts() -> Facts {
    Facts {
        name: "two".into(),
        parts: vec![
            Part {
                id: "owner".into(),
                block: vs("principal"),
                count: 1,
            },
            Part {
                id: "worker".into(),
                block: vs("agent"),
                count: 12,
            },
        ],
        settings: vec![Setting {
            part: "worker".into(),
            name: "harness".into(),
            value: vs("coding"),
        }],
        links: vec![Link {
            index: 0,
            from: vs("owner"),
            to: vs("worker"),
            carries: vs("order"),
            over: Val::Null,
        }],
        markers: vec![Marker {
            path: "parts.worker.count".into(),
            value: Val::Int(12),
        }],
    }
}

fn rows(src: &str) -> Vec<Vec<Val>> {
    let program = architecture::compile(src).unwrap();
    let answers = eval::run(&program, &architecture::fact_base(&facts()));
    let mut out: Vec<Vec<Val>> = answers.queries[0]
        .rows
        .iter()
        .map(|(v, _)| v.clone())
        .collect();
    out.sort();
    out
}

#[test]
fn the_relations_and_fields_are_the_four_of_an_architecture() {
    let names: Vec<String> = architecture::SCHEMA
        .iter()
        .map(|(rel, cols)| format!("{rel}{{{}}}", cols.join(", ")))
        .collect();
    assert_eq!(
        names,
        [
            "part{id, block, count}",
            "setting{part, name, value}",
            "link{index, from, to, carries, over}",
            "marker{path, value}",
        ]
    );
}

#[test]
fn a_selector_binds_a_part_by_its_block_and_count() {
    assert_eq!(
        rows(r#"?- part{id: P, block: "agent", count: 12}."#),
        [vec![vs("worker")]]
    );
    assert!(rows(r#"?- part{id: P, block: "agent", count: 3}."#).is_empty());
}

#[test]
fn settings_links_and_markers_are_facts() {
    assert_eq!(
        rows(r#"?- setting{part: P, name: "harness", value: V}."#),
        [vec![vs("worker"), vs("coding")]]
    );
    assert_eq!(
        rows("?- link{index: I, from: F, over: O}."),
        [vec![Val::Int(0), vs("owner"), Val::Null]]
    );
    assert_eq!(
        rows("?- marker{path: P, value: V}, V > 10."),
        [vec![vs("parts.worker.count"), Val::Int(12)]]
    );
}

#[test]
fn a_session_relation_is_unknown_to_an_architecture_and_back() {
    let err = architecture::compile("?- call{node: N}.").unwrap_err();
    assert!(err.contains("unknown relation `call`"), "{err}");
    let err = agenticspec_records::sq::compile("?- part{id: P}.").unwrap_err();
    assert!(err.contains("unknown relation `part`"), "{err}");
}

#[test]
fn no_relation_of_an_architecture_has_a_name_role_or_kind_field() {
    for field in ["name", "role", "kind"] {
        let err = architecture::compile(&format!("?- part{{id: P, {field}: X}}.")).unwrap_err();
        assert!(err.contains(&format!("no field `{field}`")), "{err}");
    }
    let err = architecture::compile("?- link{id: L}.").unwrap_err();
    assert!(err.contains("no field `id`"), "{err}");
}
