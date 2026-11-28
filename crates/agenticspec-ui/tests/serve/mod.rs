//! `agenticspec ui serve` as a library: the default control plane read from the operator UI of
//! the workstation, and the server checked by `ui conform --architecture` with one fault per
//! obligation. The healthy server and the story's acceptance run on the command line, against
//! the one server that test starts (`crates/agenticspec/tests/ui`).

use std::path::{Path, PathBuf};
use std::time::Duration;

use agenticspec_ui::conform::{self, Status};
use agenticspec_ui::operator;
use agenticspec_ui::plane::Plane;
use agenticspec_ui::serve::{self, Faults, Fixtures, Served};

mod adapter;
pub mod partial;

const TOKEN: &str = "serve-test";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The files of the workstation's operator UI with the API base of `ui serve`.
fn files() -> Vec<(String, Vec<u8>)> {
    let root = root();
    let contract =
        operator::read_doc(&root.join("ui/views/examples/workstation.contract.yaml")).unwrap();
    operator::build_with(
        &root,
        &root.join("architectures/workstation.yaml"),
        &contract,
        "workstation.contract.yaml",
        serve::ROOT,
    )
    .unwrap()
}

/// The plane holds every part of the layout with the reads of its page, the level's channel,
/// the commands of the operator ingress and of the layout's actions, and each view's shape.
#[test]
fn the_plane_of_the_workstation_is_read_from_its_operator_ui() {
    let plane = Plane::from_files(&files()).unwrap();
    assert_eq!(plane.architecture, "workstation");
    let parts: Vec<&str> = plane.parts.iter().map(|p| p.path.as_str()).collect();
    for p in ["owner", "operator", "worker", "gateway", "builds"] {
        assert!(parts.contains(&p), "{parts:?}");
    }
    assert_eq!(plane.part("worker").unwrap().count, 12);
    let gateway = plane.part("gateway").unwrap();
    assert!(
        gateway
            .reads
            .iter()
            .any(|r| r.view == "agentic.gateway.Gateways"),
        "{gateway:?}"
    );
    let queue = plane
        .part("builds")
        .unwrap()
        .reads
        .iter()
        .find(|r| r.view == "agentic.resource.Queue")
        .unwrap();
    assert!(queue.params.contains_key("pool_id"), "{queue:?}");
    assert!(!queue.params.contains_key("channel"), "{queue:?}");
    assert_eq!(plane.channels, ["level"]);
    for c in [
        "agentic.gateway.DrainGateway",
        "agentic.question.Answer",
        "agentic.channel.SendMessage",
    ] {
        assert!(plane.commands.contains(c), "{:?}", plane.commands);
    }
    assert!(plane.feeds.contains("agentic.agent.AgentDefined"));
    assert!(plane.views.contains("agentic.architecture.Placements"));
    let shape = &plane.shapes["agentic.agent.Agents"];
    assert_eq!(shape.key, "agent_id");
    assert!(shape.fields.contains("state"), "{shape:?}");
}

/// A server that adds an undeclared field to one view, resends the event of the cursor on a
/// reconnect and withholds a command in R5 that R4 still carries out fails exactly those checks,
/// each naming what broke; the rest still pass.
#[test]
fn a_server_with_one_fault_per_obligation_fails_the_check_that_names_it() {
    let files = files();
    let plane = Plane::from_files(&files).unwrap();
    let served = Served::start(
        plane.clone(),
        files,
        Fixtures::load(&root()).unwrap(),
        serve::Options {
            port: 0,
            token: TOKEN.into(),
            faults: Faults {
                undeclared_field: Some("agentic.gateway.Gateways".into()),
                resend_on_resume: true,
                withhold: Some("agentic.gateway.DrainGateway".into()),
            },
        },
    )
    .unwrap();
    let report = conform::run_plane(
        &served.interface_root(),
        &plane,
        Some(TOKEN),
        conform::Options {
            timeout: Duration::from_secs(5),
            idle: Duration::from_millis(200),
            ..conform::Options::default()
        },
    );
    let failed = report.failed();
    let names: Vec<&str> = failed.iter().map(|c| c.name.as_str()).collect();
    let read = failed
        .iter()
        .find(|c| c.name == "R1 agentic.gateway.Gateways at workstation/gateway")
        .unwrap_or_else(|| panic!("{names:?}"));
    assert!(
        read.detail
            .contains("gateway_id=00000000-0000-4000-8000-9121f2990000 with the field undeclared"),
        "{}",
        read.detail
    );
    let resume = failed
        .iter()
        .find(|c| c.name == "R2 resume level at workstation")
        .unwrap_or_else(|| panic!("{names:?}"));
    assert!(resume.detail.contains("again"), "{}", resume.detail);
    let withheld = failed
        .iter()
        .find(|c| c.name == "R4 withheld agentic.gateway.DrainGateway at workstation")
        .unwrap_or_else(|| panic!("{names:?}"));
    assert!(
        withheld.detail.starts_with(
            "R5 withholds agentic.gateway.DrainGateway at workstation, and R4 answered 200"
        ),
        "{}",
        withheld.detail
    );
    assert!(
        failed
            .iter()
            .all(|c| c.name.contains("agentic.gateway.Gateways")
                || c.name.starts_with("R2 resume")
                || c.name
                    .starts_with("R4 withheld agentic.gateway.DrainGateway")),
        "{names:?}"
    );
    assert!(report.count(Status::Passed) > 50, "{names:?}");
}

/// Rows and events whose `scope` field names a scope of the architecture are answered at that
/// scope and the scopes above it only; a `scope` that is no scope of the architecture, or none,
/// leaves a row at every scope; an event keeps its cursor at every scope.
#[test]
fn ui_serve_answers_a_row_or_event_that_names_its_scope_only_there_and_above() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("serve-scope-field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    let view = "agentic.gateway.Gateways";
    let rows = serde_json::json!({"view": view, "rows": [
        {"gateway_id": "g1", "scope": "workstation/gateway"},
        {"gateway_id": "g2", "scope": "workstation/worker/3"},
        {"gateway_id": "g3", "scope": "Part"},
    ]});
    std::fs::write(dir.join(format!("{view}.json")), rows.to_string()).unwrap();
    let event = "agentic.agent.AgentDefined";
    let script = serde_json::json!({"channel": "agent", "events": [
        {"event": event, "payload": {"agent_id": "a1", "scope": "workstation/gateway"}},
        {"event": event, "payload": {"agent_id": "a2"}},
        {"event": event, "payload": {"agent_id": "a3", "scope": "workstation/worker"}},
    ]});
    std::fs::write(dir.join("scripts/agent.json"), script.to_string()).unwrap();
    let files = files();
    let plane = Plane::from_files(&files).unwrap();
    assert!(plane.feeds.contains(event) && plane.views.contains(view));
    let served = Served::start(
        plane,
        files,
        Fixtures::from_dir(&dir).unwrap(),
        serve::Options {
            port: 0,
            token: TOKEN.into(),
            faults: Faults::default(),
        },
    )
    .unwrap();
    let auth = format!("Bearer {TOKEN}");
    let base = |scope: &str| format!("{}/scopes/{scope}/api", served.interface_root());
    let ids = |scope: &str| -> Vec<String> {
        let r = agenticspec_ui::wire::request(
            "GET",
            &format!("{}/views/{view}", base(scope)),
            &[("Authorization", &auth)],
            None,
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(r.status, 200, "{}", r.text());
        let v: serde_json::Value = serde_json::from_slice(&r.body).unwrap();
        v["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["gateway_id"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(ids("workstation"), ["g1", "g2", "g3"]);
    assert_eq!(ids("workstation/gateway"), ["g1", "g3"]);
    assert_eq!(ids("workstation/worker"), ["g2", "g3"]);
    assert_eq!(ids("workstation/worker/3"), ["g2", "g3"]);
    assert_eq!(ids("workstation/worker/1"), ["g3"]);
    let events = |scope: &str| -> Vec<(String, String)> {
        let s = agenticspec_ui::wire::subscribe(
            &format!("{}/channels/level", base(scope)),
            &[("Authorization", &auth)],
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(s.status, 200);
        let mut got = Vec::new();
        while let Some(e) = s.next(Duration::from_millis(300)) {
            let data: serde_json::Value = serde_json::from_str(&e.data).unwrap();
            got.push((
                e.id.unwrap_or_default(),
                data["agent_id"].as_str().unwrap().to_string(),
            ));
        }
        got
    };
    let pair = |id: &str, agent: &str| (id.to_string(), agent.to_string());
    assert_eq!(
        events("workstation"),
        [pair("1", "a1"), pair("2", "a2"), pair("3", "a3")]
    );
    assert_eq!(
        events("workstation/gateway"),
        [pair("1", "a1"), pair("2", "a2")]
    );
    assert_eq!(events("workstation/worker/2"), [pair("2", "a2")]);
    assert_eq!(
        events("workstation/worker"),
        [pair("2", "a2"), pair("3", "a3")]
    );
}

/// The fixtures built into the binary are the fixtures of the checkout, and the committed
/// bundle is the one its sources give now.
#[test]
fn the_embedded_fixtures_are_the_fixtures_of_the_checkout() {
    assert_eq!(Fixtures::embedded(), Fixtures::load(&root()).unwrap());
    let held = std::fs::read_to_string(root().join(serve::BUNDLE_FILE)).unwrap();
    assert_eq!(
        held,
        serve::BUNDLE,
        "the binary is built from another bundle than the tree"
    );
    assert_eq!(held, serve::bundle(&root()).unwrap(), "run task ui:panels");
    serve::write_bundle(&root(), true).unwrap();
}

/// A directory of `ui serve --fixtures`: the views, `scripts/` and `outcomes.json` read as the
/// checkout's fixtures; scripts and outcomes may be absent; a directory with no view is refused.
#[test]
fn a_fixtures_directory_reads_views_scripts_and_outcomes() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("serve-fixtures");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    let from = root().join(serve::FIXTURES);
    for sub in ["", "scripts"] {
        for e in std::fs::read_dir(from.join(sub)).unwrap() {
            let p = e.unwrap().path();
            if p.is_file() {
                std::fs::copy(&p, dir.join(sub).join(p.file_name().unwrap())).unwrap();
            }
        }
    }
    std::fs::copy(
        root().join(serve::OUTCOMES_FILE),
        dir.join(serve::DIR_OUTCOMES),
    )
    .unwrap();
    assert_eq!(Fixtures::from_dir(&dir).unwrap(), Fixtures::embedded());

    std::fs::remove_dir_all(dir.join("scripts")).unwrap();
    std::fs::remove_file(dir.join(serve::DIR_OUTCOMES)).unwrap();
    let bare = Fixtures::from_dir(&dir).unwrap();
    assert_ne!(bare, Fixtures::embedded());

    let empty = dir.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let err = Fixtures::from_dir(&empty).unwrap_err();
    assert!(err.contains("holds no view fixture"), "{err}");
    let err = Fixtures::from_dir(&dir.join("absent")).unwrap_err();
    assert!(err.contains("absent"), "{err}");
}
