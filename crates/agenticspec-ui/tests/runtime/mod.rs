//! The runtime interface (`ui/runtime.openapi.yaml`): the fixture source answers it, the
//! conformance suite passes against it and fails against a source that breaks one obligation,
//! and the generated client calls exactly the operations the document declares.
//!
//! Each test starts the one fixture source it judges (a source with a fault is another source);
//! none reaches past 127.0.0.1.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use agenticspec_ui::conform::{self, Options, Plan, Status};
use agenticspec_ui::interface;
use agenticspec_ui::source::{Data, Faults, FixtureSource, PLAN_FILE};
use agenticspec_ui::wire;
use serde_json::Value;

const ARCH: &str = "00000000-0000-4000-8000-463ec9280000";
const TOKEN: &str = "fixture-operator";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn plan() -> Plan {
    Plan::parse(&std::fs::read_to_string(root().join(PLAN_FILE)).unwrap()).unwrap()
}

fn source(faults: Faults) -> FixtureSource {
    FixtureSource::start(Data::load(&root()).unwrap(), faults).unwrap()
}

fn options() -> Options {
    Options {
        timeout: Duration::from_secs(5),
        idle: Duration::from_millis(200),
    }
}

fn run(src: &FixtureSource) -> conform::Report {
    conform::run(src.url(), &plan(), Some(TOKEN), options())
}

fn bearer() -> String {
    format!("Bearer {TOKEN}")
}

fn get(url: &str) -> wire::Response {
    let auth = bearer();
    wire::request(
        "GET",
        url,
        &[("Authorization", &auth)],
        None,
        Duration::from_secs(5),
    )
    .unwrap()
}

fn keys(body: &[u8], field: &str) -> Vec<String> {
    let v: Value = serde_json::from_slice(body).unwrap();
    v["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[field].as_str().unwrap().to_string())
        .collect()
}

fn failed_with<'a>(report: &'a conform::Report, prefix: &str) -> &'a conform::Check {
    report
        .named(prefix)
        .into_iter()
        .find(|c| c.status == Status::Failed)
        .unwrap_or_else(|| panic!("no failed check {prefix}: {:#?}", report.checks))
}

/// Every check of the fixture source's plan passes, R1 answers exactly the rows of the part and
/// of its copy, R3 answers rows at every scope, R4 answers a row or a refusal, and R5 allows
/// `Answer` at `team.lead` by its grant and not at `team.worker`.
#[test]
fn the_fixture_source_conforms_and_answers_each_scope() {
    let src = source(Faults::default());
    let report = run(&src);
    assert!(report.failed().is_empty(), "{:#?}", report.failed());
    assert_eq!(report.count(Status::Skipped), 0, "{:#?}", report.checks);
    for name in [
        "unauthenticated R1",
        "unauthenticated R2",
        "unauthenticated R4",
        "unauthenticated R5",
        "R1 agentic.harness.Turns at",
        "R3 agentic.architecture.Placements at",
        "R3 agentic.architecture.Contracts at",
        "R2 resume harness at",
        "R2 scope harness at",
        "R4 agentic.question.Answer at",
        "R5 at",
        "R5 hidden agentic.question.Answer at",
    ] {
        assert!(
            !report.named(name).is_empty(),
            "no check {name}: {:#?}",
            report.checks
        );
    }

    let lead = interface::scope_url(src.url(), ARCH, Some("team.lead"), None);
    let worker = interface::scope_url(src.url(), ARCH, Some("team.worker"), Some(2));
    let all = interface::scope_url(src.url(), ARCH, None, None);
    assert_eq!(lead, format!("{}/scopes/{ARCH}/team.lead", src.url()));
    let turns = |base: &str| {
        keys(
            &get(&format!("{base}/api/views/agentic.harness.Turns")).body,
            "turn_id",
        )
    };
    assert_eq!(turns(&lead), ["00000000-0000-4000-8000-391e34cb0000"]);
    assert_eq!(turns(&worker), ["00000000-0000-4000-8000-391e34cb0001"]);
    assert_eq!(
        turns(&all),
        [
            "00000000-0000-4000-8000-391e34cb0000",
            "00000000-0000-4000-8000-391e34cb0001"
        ]
    );
    for view in interface::r3_views() {
        let r = get(&format!("{lead}/api/views/{view}"));
        assert_eq!(r.status, 200, "{view}");
        let v: Value = serde_json::from_slice(&r.body).unwrap();
        assert!(v["rows"].is_array(), "{view}: {v}");
    }
    let parts = keys(
        &get(&format!("{lead}/api/views/agentic.architecture.Parts")).body,
        "name",
    );
    assert_eq!(parts, ["team.lead"]);

    let actions = |base: &str| -> Value {
        serde_json::from_slice(&get(&format!("{base}/api/actions")).body).unwrap()
    };
    let answer = |v: &Value| {
        v.as_array()
            .unwrap()
            .iter()
            .find(|e| e["command"] == "agentic.question.Answer")
            .unwrap()
            .clone()
    };
    let at_lead = answer(&actions(&lead));
    assert_eq!(at_lead["allowed"], true);
    assert_eq!(at_lead["grant_id"], "00000000-0000-4000-8000-69f9b46e0100");
    let at_worker = answer(&actions(&worker));
    assert_eq!(at_worker["allowed"], false);
    assert!(at_worker.get("grant_id").is_none());
}

/// A reconnect with the cursor of event 3 receives exactly events 4 and 5, in order, each named
/// by its qualified event name.
#[test]
fn a_reconnect_from_the_cursor_of_event_3_receives_exactly_events_4_and_5() {
    let src = source(Faults::default());
    let url = format!(
        "{}/api/channels/harness",
        interface::scope_url(src.url(), ARCH, Some("team.lead"), None)
    );
    let auth = bearer();
    let first = wire::subscribe(&url, &[("Authorization", &auth)], Duration::from_secs(5)).unwrap();
    let mut sent = Vec::new();
    while let Some(e) = first.next(Duration::from_millis(300)) {
        sent.push(e);
    }
    drop(first);
    assert_eq!(sent.len(), 5, "{sent:#?}");
    assert!(sent.iter().all(|e| e.event.starts_with("agentic.harness.")));
    let cursor = sent[2].id.clone().unwrap();
    let again = wire::subscribe(
        &url,
        &[("Authorization", &auth), ("Last-Event-ID", &cursor)],
        Duration::from_secs(5),
    )
    .unwrap();
    let mut got = Vec::new();
    while let Some(e) = again.next(Duration::from_millis(300)) {
        got.push(e);
    }
    assert_eq!(got, sent[3..].to_vec());
}

#[test]
fn one_out_of_scope_row_fails_the_scoped_read_and_names_the_view_and_its_key() {
    let src = source(Faults {
        leak_row: Some("agentic.harness.Turns".into()),
        ..Faults::default()
    });
    let report = run(&src);
    let c = failed_with(
        &report,
        &format!("R1 agentic.harness.Turns at {ARCH}/team.lead"),
    );
    assert!(c.detail.contains("agentic.harness.Turns"), "{c}");
    assert!(
        c.detail
            .contains("turn_id=00000000-0000-4000-8000-391e34cb0001"),
        "{c}"
    );
}

#[test]
fn a_row_left_out_of_a_scope_fails_the_scoped_read() {
    let src = source(Faults {
        drop_row: Some("agentic.question.OpenQuestions".into()),
        ..Faults::default()
    });
    let report = run(&src);
    let c = failed_with(&report, "R1 agentic.question.OpenQuestions at");
    assert!(c.detail.contains("left out the row question_id="), "{c}");
}

#[test]
fn a_source_that_resends_the_event_of_the_cursor_fails_the_resume_check() {
    let src = source(Faults {
        resend_on_resume: true,
        ..Faults::default()
    });
    let report = run(&src);
    let c = failed_with(&report, "R2 resume harness");
    assert!(c.detail.contains("sent event 3 again"), "{c}");
}

/// Without credentials every request is refused with 401 and no row before it is dispatched.
#[test]
fn a_request_without_credentials_is_refused_before_any_dispatch() {
    let src = source(Faults::default());
    let lead = interface::scope_url(src.url(), ARCH, Some("team.lead"), None);
    let t = Duration::from_secs(5);
    let answers = [
        wire::request(
            "GET",
            &format!("{lead}/api/views/agentic.harness.Turns"),
            &[],
            None,
            t,
        )
        .unwrap(),
        wire::request("GET", &format!("{lead}/api/actions"), &[], None, t).unwrap(),
        wire::request(
            "POST",
            &format!("{lead}/api/commands/agentic.question.Answer"),
            &[
                ("Content-Type", "application/json"),
                ("Authorization", "Bearer not-a-token"),
            ],
            Some(br#"{"question_id": "00000000-0000-4000-8000-d74009500000", "option": "A"}"#),
            t,
        )
        .unwrap(),
    ];
    for r in &answers {
        assert_eq!(r.status, 401, "{}", r.text());
        let v: Value = serde_json::from_slice(&r.body).unwrap();
        assert!(v.get("rows").is_none() && v.get("row").is_none(), "{v}");
    }
    let stream = wire::subscribe(&format!("{lead}/api/channels/harness"), &[], t).unwrap();
    assert_eq!(stream.status, 401);
    assert!(stream.next(Duration::from_millis(100)).is_none());
    assert_eq!(src.dispatched(), 0);

    let open = source(Faults {
        open: true,
        ..Faults::default()
    });
    let report = run(&open);
    let c = failed_with(&report, "unauthenticated R1");
    assert!(c.detail.contains("answered 200 without credentials"), "{c}");
}

/// R5 that allows a command no grant covers at a scope fails, naming the command and the scope;
/// R4 still refuses the command R5 hides.
#[test]
fn an_action_set_that_allows_what_no_grant_covers_fails_and_r4_still_refuses() {
    let src = source(Faults {
        allow_everything: true,
        ..Faults::default()
    });
    let report = run(&src);
    let c = failed_with(&report, &format!("R5 at {ARCH}/team.lead"));
    assert!(
        c.detail.contains("agentic.question.TakeDefault")
            && c.detail.contains(&format!("{ARCH}/team.lead")),
        "{c}"
    );
    let c = failed_with(&report, &format!("R5 at {ARCH}/team.worker/2"));
    assert!(c.detail.contains("agentic.question.Answer"), "{c}");

    let worker = interface::scope_url(src.url(), ARCH, Some("team.worker"), Some(2));
    let auth = bearer();
    let r = wire::request(
        "POST",
        &format!("{worker}/api/commands/agentic.question.Answer"),
        &[
            ("Content-Type", "application/json"),
            ("Authorization", &auth),
        ],
        Some(br#"{"question_id": "00000000-0000-4000-8000-d74009500001", "option": "A"}"#),
        Duration::from_secs(5),
    )
    .unwrap();
    assert_eq!(r.status, 422, "{}", r.text());
    let v: Value = serde_json::from_slice(&r.body).unwrap();
    assert_eq!(v["row"]["error"], "agentic.authority.NoGrantCovers");
}

/// The generated client calls exactly the operations of `ui/runtime.openapi.yaml`, by method and
/// path, handles the statuses the document gives a meaning (422 of a command, 404 of R5), sends
/// credentials, and scopes below the document's prefix.
#[test]
fn the_generated_client_calls_exactly_the_operations_of_the_interface_document() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/widgets/input.ui.yaml"),
    )
    .unwrap();
    let files = agenticspec_ui::generate(&text).unwrap();
    let client = &files["client.ts"];
    let runtime = &files["runtime.ts"];

    let call = regex::Regex::new(r"`\$\{base\}(/api/[^`]*)`").unwrap();
    let var = regex::Regex::new(r"\$\{(\w+)\}").unwrap();
    let mut called = BTreeSet::new();
    for m in call.captures_iter(client) {
        let whole = m.get(0).unwrap();
        let path = m[1].split("${query(").next().unwrap().to_string();
        let path = var.replace_all(&path, "{$1}").to_string();
        let after = &client[whole.end()..client.len().min(whole.end() + 80)];
        let method = if after.contains("method: 'POST'") {
            "POST"
        } else {
            "GET"
        };
        called.insert((method.to_string(), path));
    }
    let declared: BTreeSet<(String, String)> = interface::operations()
        .into_iter()
        .map(|o| (o.method, o.path))
        .collect();
    assert_eq!(
        called, declared,
        "the client's calls and the document's operations"
    );

    let ops = interface::operations();
    let status = |id: &str, s: u16| ops.iter().any(|o| o.id == id && o.statuses.contains(&s));
    assert!(status("command", 422) && client.contains("r.status === 422"));
    assert!(status("actions", 404) && client.contains("r.status === 404"));
    for id in ["read", "subscribe", "command", "actions"] {
        assert!(status(id, 401), "{id} declares 401");
    }
    assert!(client.contains("credentials: 'include'"));
    assert!(client.contains("withCredentials: true"));
    assert!(runtime.contains(&format!("'{}'", interface::scope_prefix())));
}
