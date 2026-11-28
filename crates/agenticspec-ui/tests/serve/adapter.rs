//! `ui conform` against a runtime's adapter that is not `ui serve` ([`super::partial`]): a view
//! it does not serve is skipped by name and any other 404 fails, a source that accepts only
//! same-origin writes gets `Origin` and the proof header, and a row's `scope` field decides its
//! scope. One source per test; the checks that need no server first.

use std::collections::BTreeSet;
use std::time::Duration;

use agenticspec_ui::conform::{self, Check, Options, Plan, Report, Status};
use agenticspec_ui::interface::{self, Ordered};
use agenticspec_ui::plane::Plane;
use serde_json::{Value, json};

use super::partial::{Config, Partial};

const TOKEN: &str = "adapter-test";
const PROOF: (&str, &str) = ("X-CSRF-Token", "csrf-test");
const NOT_SERVED: [&str; 3] = [
    "agentic.architecture.Reach",
    "agentic.architecture.Revisions",
    "agentic.architecture.Contracts",
];

fn options() -> Options {
    Options {
        timeout: Duration::from_secs(5),
        idle: Duration::from_millis(100),
        ..Options::default()
    }
}

fn check<'a>(report: &'a Report, name: &str) -> &'a Check {
    report
        .checks
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no check {name}: {:#?}", report.lines()))
}

fn plane() -> Plane {
    Plane::from_files(&super::files()).unwrap()
}

#[test]
fn an_origin_and_a_header_are_read_as_a_page_of_the_source_sends_them() {
    assert_eq!(
        conform::origin_of("http://127.0.0.1:7430/ingress/operator").unwrap(),
        "http://127.0.0.1:7430"
    );
    assert_eq!(
        conform::origin_of("http://Localhost/scopes").unwrap(),
        "http://localhost"
    );
    assert!(conform::origin_of("https://example.test").is_err());
    assert_eq!(
        conform::parse_origin("https://ui.example.test/").unwrap(),
        "https://ui.example.test"
    );
    for bad in ["127.0.0.1:7430", "http://", "http://h/path", "http://h x"] {
        assert!(conform::parse_origin(bad).is_err(), "{bad}");
    }
    assert_eq!(
        conform::parse_header("X-CSRF-Token:  abc def ").unwrap(),
        ("X-CSRF-Token".to_string(), "abc def".to_string())
    );
    for bad in [
        "no colon",
        ": empty",
        "Bad Name: x",
        "Origin: http://h",
        "authorization: Bearer x",
        "X-A: a\nb",
    ] {
        assert!(conform::parse_header(bad).is_err(), "{bad}");
    }
}

/// A `scope` field that holds a scope of the architecture decides; any other value of a field of
/// that name leaves the entity-id rule.
#[test]
fn a_row_that_names_its_scope_is_in_scope_there_and_above() {
    let placed: BTreeSet<String> = ["e1".to_string()].into();
    let row = |v: Value| v.as_object().unwrap().clone();
    let at = |r: &serde_json::Map<String, Value>, scope: &str| {
        interface::in_scope_at(r, &placed, "ws", scope)
    };
    let lead = row(json!({"id": "x", "scope": "ws/team.lead"}));
    assert!(at(&lead, "ws/team.lead"));
    assert!(at(&lead, "ws"));
    assert!(!at(&lead, "ws/team.worker"));
    assert!(!at(&lead, "ws/team.lead/2"));
    assert!(!at(&lead, "ws/team.leader"));
    let copy = row(json!({"id": "x", "scope": "ws/team.lead/2"}));
    assert!(at(&copy, "ws/team.lead") && at(&copy, "ws/team.lead/2"));
    assert!(!at(&copy, "ws/team.lead/1"));
    // The scope decides over an entity id the placements reach.
    let both = row(json!({"id": "e1", "scope": "ws/team.worker"}));
    assert!(!at(&both, "ws/team.lead"));
    // A view's own `scope` such as `Part` is no scope: the entity id decides.
    let own = row(json!({"id": "e1", "scope": "Part"}));
    assert!(at(&own, "ws/team.lead"));
    let other = row(json!({"id": "e2", "scope": "other/team.lead"}));
    assert!(!at(&other, "ws/team.lead") && !at(&other, "ws"));
}

#[test]
fn the_report_prints_every_check_on_one_line_with_its_status() {
    let report = Report {
        source: "http://127.0.0.1:1".into(),
        checks: vec![
            Check {
                name: "R1 a at ws".into(),
                status: Status::Passed,
                detail: "1 row".into(),
            },
            Check {
                name: "R3 b at ws".into(),
                status: Status::Skipped,
                detail: "not served".into(),
            },
            Check {
                name: "R4 c at ws".into(),
                status: Status::Failed,
                detail: "answered 403: {\n\"error\"}".into(),
            },
        ],
    };
    assert_eq!(
        report.lines(),
        [
            "passed R1 a at ws: 1 row",
            "skipped R3 b at ws: not served",
            "failed R4 c at ws: answered 403: { \"error\"}",
        ]
    );
}

/// Reach, Revisions and Contracts answered with the declared "not served" 404 are skipped, each
/// by its name; a part's view answered with a plain 404 still fails.
#[test]
fn a_view_the_source_does_not_serve_is_skipped_by_name_and_another_404_fails() {
    let plane = plane();
    let src = Partial::start(Config {
        architecture: plane.architecture.clone(),
        token: TOKEN.into(),
        not_served: NOT_SERVED.iter().map(ToString::to_string).collect(),
        missing: vec!["agentic.gateway.Gateways".into()],
        commands: plane.commands.iter().take(1).cloned().collect(),
        ..Config::default()
    });
    let report = conform::run_plane(src.url(), &plane, Some(TOKEN), options());
    for view in NOT_SERVED {
        let c = check(&report, &format!("R3 {view} at workstation"));
        assert_eq!(c.status, Status::Skipped, "{c}");
        assert!(
            c.detail.contains(view) && c.detail.contains("view_not_served"),
            "{c}"
        );
    }
    let gone = check(
        &report,
        "R1 agentic.gateway.Gateways at workstation/gateway",
    );
    assert_eq!(gone.status, Status::Failed, "{gone}");
    assert!(gone.detail.contains("answered 404"), "{gone}");
    for view in [
        "agentic.architecture.Placements",
        "agentic.architecture.Parts",
    ] {
        let c = check(&report, &format!("R3 {view} at workstation"));
        assert_eq!(c.status, Status::Passed, "{c}");
    }
    let failed: Vec<&str> = report.failed().iter().map(|c| c.name.as_str()).collect();
    assert!(
        failed
            .iter()
            .all(|n| n.contains("agentic.gateway.Gateways")),
        "{failed:?}"
    );
    for c in report.checks.iter().filter(|c| c.status == Status::Skipped) {
        assert!(
            NOT_SERVED.iter().any(|v| c.name.contains(v)) || c.name.starts_with("R4 withheld"),
            "{c}"
        );
    }
}

/// A source that accepts a write only from its own origin with a CSRF header: with both the
/// command R5 allows is carried out (200); without the header, or with another origin, R4
/// answers 403 and the check fails.
#[test]
fn a_source_that_accepts_only_same_origin_writes_carries_out_a_command_with_its_proof() {
    let plane = plane();
    let command = plane.commands.iter().next().unwrap().clone();
    let src = Partial::start(Config {
        architecture: plane.architecture.clone(),
        token: TOKEN.into(),
        proof: Some((PROOF.0.into(), PROOF.1.into())),
        commands: vec![command.clone()],
        ..Config::default()
    });
    let name = format!("R4 allowed {command} at workstation");
    let with_proof = Options {
        headers: vec![(PROOF.0.into(), PROOF.1.into())],
        ..options()
    };
    let report = conform::run_plane(src.url(), &plane, Some(TOKEN), with_proof.clone());
    let accepted = check(&report, &name);
    assert_eq!(accepted.status, Status::Passed, "{accepted}");
    assert_eq!(accepted.detail, "200 {\"row\"}");
    assert_eq!(report.failed().len(), 0, "{:#?}", report.lines());
    let unauthenticated = report
        .checks
        .iter()
        .find(|c| c.name.starts_with("unauthenticated R4 "))
        .unwrap();
    assert_eq!(unauthenticated.status, Status::Passed, "{unauthenticated}");

    let report = conform::run_plane(src.url(), &plane, Some(TOKEN), options());
    let refused = check(&report, &name);
    assert_eq!(refused.status, Status::Failed, "{refused}");
    assert!(refused.detail.contains("answered 403"), "{refused}");

    let elsewhere = Options {
        origin: Some("http://127.0.0.1:1".into()),
        ..with_proof
    };
    let report = conform::run_plane(src.url(), &plane, Some(TOKEN), elsewhere);
    assert_eq!(check(&report, &name).status, Status::Failed);
}

/// The plane mode with rows that carry `scope`: a row named for the part read, or a copy of it,
/// passes, also where the view does not declare the field; a row named for another part fails,
/// and so does an undeclared `scope` that is no scope of the architecture.
#[test]
fn the_plane_mode_judges_the_scope_a_row_names() {
    let plane = plane();
    // Three parts, each with a read without parameters of a view that does not declare `scope`;
    // the first runs several copies.
    let mut picks: Vec<(String, String)> = plane
        .parts
        .iter()
        .filter_map(|p| {
            p.reads
                .iter()
                .find(|r| {
                    r.params.is_empty()
                        && plane
                            .shapes
                            .get(&r.view)
                            .is_some_and(|s| !s.fields.contains("scope"))
                })
                .map(|r| (p.path.clone(), r.view.clone()))
        })
        .collect();
    let first = picks
        .iter()
        .position(|(p, _)| plane.part(p).unwrap().count > 1)
        .expect("a part of several copies");
    picks.swap(0, first);
    assert!(picks.len() >= 3, "{picks:?}");
    let at = |i: usize| format!("workstation/{}", picks[i].0);
    let row = |i: usize, key: &str, scope: &str| {
        let shape = &plane.shapes[&picks[i].1];
        (
            picks[i].1.clone(),
            at(i),
            Ordered(vec![
                (shape.key.clone(), json!(key)),
                ("scope".into(), json!(scope)),
            ]),
        )
    };
    let src = Partial::start(Config {
        architecture: plane.architecture.clone(),
        token: TOKEN.into(),
        rows: vec![
            row(0, "own", &at(0)),
            row(0, "copy", &format!("{}/2", at(0))),
            row(1, "stray", &at(0)),
            row(2, "own", "Part"),
        ],
        ..Config::default()
    });
    let report = conform::run_plane(src.url(), &plane, Some(TOKEN), options());
    let name = |i: usize| format!("R1 {} at {}", picks[i].1, at(i));
    let c = check(&report, &name(0));
    assert_eq!(c.status, Status::Passed, "{c}");
    let c = check(&report, &name(1));
    assert_eq!(c.status, Status::Failed, "{c}");
    let key = &plane.shapes[&picks[1].1].key;
    assert!(
        c.detail.contains(&format!(
            "row {key}=stray of scope {}, which is not in scope at {}",
            at(0),
            at(1)
        )),
        "{c}"
    );
    let c = check(&report, &name(2));
    assert_eq!(c.status, Status::Failed, "{c}");
    assert!(
        c.detail
            .contains("with scope \"Part\", which is no scope of workstation"),
        "{c}"
    );
}

/// The plan mode honours `scope` in R1 (a row out of its scope or left out fails, named by its
/// scope) and skips a view the source does not serve, while a plain 404 fails.
#[test]
fn the_plan_mode_honours_scope_and_skips_a_view_not_served() {
    let thing = |id: &str, scope: &str| {
        Ordered(vec![
            ("thing_id".into(), json!(id)),
            ("scope".into(), json!(scope)),
        ])
    };
    let rows = |view: &str, list: &[(&str, &str, &str)]| -> Vec<(String, String, Ordered)> {
        list.iter()
            .map(|(at, id, scope)| (view.to_string(), at.to_string(), thing(id, scope)))
            .collect()
    };
    let mut all = rows(
        "agentic.test.Things",
        &[
            ("ws", "t1", "ws/worker"),
            ("ws", "t2", "ws/gateway"),
            ("ws/worker", "t1", "ws/worker"),
        ],
    );
    all.extend(rows(
        "agentic.test.Leaky",
        &[
            ("ws", "t2", "ws/gateway"),
            ("ws/worker", "t2", "ws/gateway"),
        ],
    ));
    all.extend(rows("agentic.test.Shy", &[("ws", "t1", "ws/worker")]));
    let src = Partial::start(Config {
        architecture: "ws".into(),
        token: TOKEN.into(),
        not_served: NOT_SERVED.iter().map(ToString::to_string).collect(),
        missing: vec!["agentic.test.Gone".into()],
        rows: all,
        ..Config::default()
    });
    let plan = Plan::parse(
        &json!({
            "format": "agenticspec-runtime-conformance/1",
            "architecture_id": "ws",
            "token": TOKEN,
            "scopes": [{"part": "worker", "views": [
                "agentic.test.Things", "agentic.test.Leaky", "agentic.test.Shy",
                "agentic.architecture.Reach", "agentic.test.Gone"
            ]}]
        })
        .to_string(),
    )
    .unwrap();
    let report = conform::run(src.url(), &plan, Some(TOKEN), options());
    let status = |name: &str| check(&report, name).status;
    assert_eq!(status("R1 agentic.test.Things at ws"), Status::Passed);
    assert_eq!(
        status("R1 agentic.test.Things at ws/worker"),
        Status::Passed
    );
    let leak = check(&report, "R1 agentic.test.Leaky at ws/worker");
    assert_eq!(leak.status, Status::Failed);
    assert!(
        leak.detail
            .contains("row thing_id=t2 of scope ws/gateway, which is not in scope at ws/worker"),
        "{leak}"
    );
    let shy = check(&report, "R1 agentic.test.Shy at ws/worker");
    assert_eq!(shy.status, Status::Failed);
    assert!(
        shy.detail
            .contains("left out the row thing_id=t1 of scope ws/worker"),
        "{shy}"
    );
    for name in [
        "R1 agentic.architecture.Reach at ws",
        "R1 agentic.architecture.Reach at ws/worker",
        "R3 agentic.architecture.Revisions at ws/worker",
    ] {
        let c = check(&report, name);
        assert_eq!(c.status, Status::Skipped, "{c}");
        assert!(c.detail.contains("agentic.architecture."), "{c}");
    }
    let gone = check(&report, "R1 agentic.test.Gone at ws/worker");
    assert_eq!(gone.status, Status::Failed, "{gone}");
    assert!(gone.detail.contains("answered 404"), "{gone}");
}
