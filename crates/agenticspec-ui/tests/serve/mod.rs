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
