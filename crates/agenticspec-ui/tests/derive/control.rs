//! `agenticspec ui derive --control-plane`: the pages across parts of the default control plane,
//! on the committed examples of `ui/views/examples/` (workstation and team-with-console), and the
//! rules that keep a command off every page whatever the contract's operator ingress lists.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value as Json;

use super::{Derivation, Inputs, derive, derived, inputs, refused, root, scratch};

const EXAMPLES: &str = "ui/views/examples";

fn control(arch: &str, contract: &str, ports: Option<PathBuf>, out: PathBuf) -> Inputs {
    let r = root();
    let mut i = inputs(
        &r.join(format!("architectures/{arch}.yaml")),
        Some(contract),
        Some(ports.unwrap_or_else(|| r.join(EXAMPLES).join("ports.json"))),
        out,
    );
    i.control_plane = true;
    i
}

fn example(arch: &str) -> Inputs {
    control(
        arch,
        &format!("{EXAMPLES}/{arch}.contract.yaml"),
        None,
        root()
            .join(EXAMPLES)
            .join(format!("{arch}.control-plane.ui.yaml")),
    )
}

/// The `control` key of the layout among the derived files.
fn layout(d: &Derivation) -> Json {
    let text = d
        .files
        .iter()
        .find(|(p, _)| p.to_string_lossy().ends_with(".layout.json"))
        .map(|(_, t)| t.clone())
        .expect("the layout is among the files");
    let doc: Json = serde_json::from_str(&text).unwrap();
    doc["control"].clone()
}

fn page<'a>(control: &'a Json, name: &str) -> &'a Json {
    control["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["page"] == name)
        .unwrap_or_else(|| panic!("page {name}"))
}

fn sections(page: &Json) -> Vec<&Json> {
    page["frames"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|f| f["sections"].as_array().unwrap())
        .collect()
}

/// Every command a page across parts offers as a row action.
fn offered(control: &Json) -> BTreeSet<String> {
    control["pages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(sections)
        .flat_map(|s| s["row_actions"].as_array().unwrap())
        .map(|a| a["command"].as_str().unwrap().to_string())
        .collect()
}

fn read_json(path: &Path) -> Json {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// team-with-console derived with a copy of the port catalogue that `edit` changed.
fn with_ports(name: &str, edit: impl FnOnce(&mut Json)) -> Json {
    let dir = scratch(name);
    let mut ports = read_json(&root().join(EXAMPLES).join("ports.json"));
    edit(&mut ports);
    let path = dir.join("ports.json");
    std::fs::write(&path, serde_json::to_string(&ports).unwrap()).unwrap();
    let i = control(
        "team-with-console",
        &format!("{EXAMPLES}/team-with-console.contract.yaml"),
        Some(path),
        dir.join("team.control-plane.ui.yaml"),
    );
    layout(&derived(&i))
}

const ANSWER: &str = "agentic.question.Answer";

#[test]
fn the_control_plane_examples_are_current_and_derived_byte_for_byte_again() {
    for arch in ["workstation", "team-with-console"] {
        let i = example(arch);
        let a = derived(&i);
        let b = derived(&i);
        assert_eq!(a.files, b.files, "{arch}: deterministic");
        let stale = derive::differences(&a);
        assert!(
            stale.is_empty(),
            "{arch} stale: {stale:?}; run task ui:derive"
        );
    }
}

/// The acceptance of the control plane on team-with-console: the owner's questions with the
/// answer in the Inbox, exactly drain, stop and detach on the gateways in Operations, and every
/// KPI of the metrics ingress on the Metrics page.
#[test]
fn team_with_console_shows_its_inbox_operations_and_metrics() {
    let c = layout(&derived(&example("team-with-console")));
    let labels: Vec<&str> = c["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["label"].as_str().unwrap())
        .collect();
    assert_eq!(
        labels,
        [
            "Inbox",
            "Activity",
            "Operations",
            "Metrics",
            "Architecture",
            "Events"
        ]
    );
    let inbox = sections(page(&c, "control.inbox"));
    let answers: Vec<(&str, &str)> = inbox
        .iter()
        .flat_map(|s| {
            s["row_actions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| (s["part"].as_str().unwrap(), a["command"].as_str().unwrap()))
        })
        .collect();
    assert_eq!(answers, [("owner", ANSWER)]);

    let ops = sections(page(&c, "control.operations"));
    for part in ["gateway", "console.gateway"] {
        let got: BTreeSet<&str> = ops
            .iter()
            .filter(|s| s["part"] == part)
            .flat_map(|s| s["row_actions"].as_array().unwrap())
            .map(|a| a["command"].as_str().unwrap())
            .collect();
        assert_eq!(
            got,
            BTreeSet::from([
                "agentic.gateway.DetachUpstream",
                "agentic.gateway.DrainGateway",
                "agentic.gateway.StopGateway"
            ]),
            "{part}"
        );
    }
    let elsewhere: Vec<&str> = ops
        .iter()
        .filter(|s| !["gateway", "console.gateway"].contains(&s["part"].as_str().unwrap_or("")))
        .flat_map(|s| s["row_actions"].as_array().unwrap())
        .map(|a| a["command"].as_str().unwrap())
        .collect();
    assert!(elsewhere.is_empty(), "{elsewhere:?}");
    assert_eq!(c["metrics"].as_array().unwrap().len(), 9);
    for s in sections(page(&c, "control.activity")) {
        assert!(s["part"].is_null(), "activity is across parts: {s}");
    }
}

/// The selection names no block other than the architecture block, no view, no command and no
/// KPI: what a page holds comes from the contract and the port catalogue alone.
#[test]
fn the_selection_names_no_block_view_command_or_kpi() {
    let r = root();
    let source =
        std::fs::read_to_string(r.join("crates/agenticspec-ui/src/derive/control.rs")).unwrap();
    let code: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let literals: BTreeSet<&str> = code.split('"').skip(1).step_by(2).collect();
    let ports = read_json(&r.join(EXAMPLES).join("ports.json"));
    let blocks: BTreeSet<String> = ports["domains"]
        .as_object()
        .unwrap()
        .keys()
        .filter_map(|d| d.rsplit_once('.').map(|x| x.1.to_string()))
        .chain(
            ports["ports"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|p| p["block"].as_str().map(str::to_string)),
        )
        .filter(|b| b != "architecture")
        .collect();
    let result = read_json(&r.join("website/static/catalog/workstation/result.json"));
    let kpis: Vec<&String> = result["baseline"]["kpis"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    for l in &literals {
        assert!(!l.contains("agentic."), "a qualified name: {l:?}");
        assert!(!blocks.contains(*l), "a block name: {l:?}");
        assert!(!kpis.iter().any(|k| k == l), "a KPI: {l:?}");
    }
}

#[test]
fn an_offered_administration_command_is_a_row_action() {
    let c = layout(&derived(&example("team-with-console")));
    assert!(offered(&c).contains("agentic.gateway.DrainGateway"));
    assert!(offered(&c).contains(ANSWER));
}

/// Rule 1: a configuration command is never exposed. The answer of the principal port, made a
/// configuration command in a copy of the catalogue, leaves the Inbox.
#[test]
fn a_configuration_command_is_never_exposed() {
    let c = with_ports("rule-configuration", |p| {
        let commands = p["domains"]["agentic.question"]["commands"]
            .as_object_mut()
            .unwrap();
        commands.insert(ANSWER.into(), "configuration".into());
    });
    assert!(!offered(&c).contains(ANSWER), "{:?}", offered(&c));
    assert!(offered(&c).contains("agentic.gateway.DrainGateway"));
}

/// Rule 2: a command only actors holding the report of a port that is no principal port may
/// issue is the implementation's. Every actor that may answer, given a report of an outbound
/// port, takes the answer off the Inbox.
#[test]
fn a_command_only_report_issuing_actors_may_issue_is_never_exposed() {
    let c = with_ports("rule-report-actor", |p| {
        for (_, may) in p["actors"].as_object_mut().unwrap() {
            let list = may.as_array_mut().unwrap();
            if list.iter().any(|c| c == ANSWER) {
                list.push("agentic.gateway.CompleteCall".into());
            }
        }
    });
    assert!(!offered(&c).contains(ANSWER), "{:?}", offered(&c));
    assert!(offered(&c).contains("agentic.gateway.DrainGateway"));
}

/// Rule 3: machine traffic is never exposed. The answer, named as the intent command of the port
/// a `model_call` link stands for, is a command of machine traffic and leaves the Inbox.
#[test]
fn machine_traffic_is_never_exposed() {
    let c = with_ports("rule-machine", |p| {
        for port in p["ports"].as_array_mut().unwrap() {
            if port["name"] == "gateway.call" {
                port["intent_command"] = ANSWER.into();
            }
        }
    });
    assert!(!offered(&c).contains(ANSWER), "{:?}", offered(&c));
    assert!(offered(&c).contains("agentic.gateway.DrainGateway"));
}

/// The Metrics page shows the simulator's prediction of the same revision; a result of another
/// architecture is refused.
#[test]
fn the_metrics_take_the_prediction_of_the_same_revision_only() {
    let r = root();
    let result = r.join("website/static/catalog/workstation/result.json");
    let mut i = example("workstation");
    i.out = scratch("prediction").join("workstation.control-plane.ui.yaml");
    i.prediction = Some(result.clone());
    let c = layout(&derived(&i));
    let metric = |k: &str| {
        c["metrics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["kpi"] == k)
            .unwrap_or_else(|| panic!("{k}"))
            .clone()
    };
    let cost = metric("cost_usd")["prediction"]
        .as_f64()
        .expect("predicted");
    let want = read_json(&result)["baseline"]["kpis"]["cost_usd"]["value"]
        .as_f64()
        .unwrap();
    assert_eq!(cost, want);
    assert!(metric("cache_bust_count")["prediction"].is_null());

    let mut other = example("team-with-console");
    other.out = scratch("prediction-other").join("team.control-plane.ui.yaml");
    other.prediction = Some(result);
    assert_eq!(refused(&other), ["prediction"]);
}

#[test]
fn the_control_plane_without_a_contract_is_refused() {
    let r = root();
    let mut i = inputs(
        &r.join("architectures/workstation.yaml"),
        None,
        None,
        scratch("no-contract").join("workstation.control-plane.ui.yaml"),
    );
    i.control_plane = true;
    assert_eq!(refused(&i), ["contract"]);
}
