//! The default control plane: the ingresses `operator` and `metrics` every architecture has by
//! name, `none` to opt out, the reserved path `control` of their implied audiences, the rule
//! values `principal: all`, `plays: administration` and `metrics: computed`, and
//! `agenticspec ingress default`.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, CheckOptions};
use agenticspec_sim::contract::derive::{self, Contract};
use agenticspec_sim::contract::ingress::{self, ContractIngress, Kpis, OpKind};
use agenticspec_sim::contract::ports::{PortsFile, Role};
use serde_json::Value;
use std::path::Path;

fn ports() -> PortsFile {
    PortsFile::load(&repo_root().join("blocks/ports.json")).unwrap()
}

fn kpis() -> Kpis {
    Kpis::load(&repo_root().join("kpis/catalogue.yaml")).unwrap()
}

fn load(path: &Path) -> Value {
    agenticspec_sim::doc::load_yaml(path).unwrap()
}

fn workstation() -> Value {
    load(&repo_root().join("architectures/workstation.yaml"))
}

fn resolve(doc: &Value) -> (Vec<ContractIngress>, Vec<ingress::Refusal>) {
    let ports = ports();
    let contract = derive::derive(doc, &ports).unwrap();
    ingress::resolve(doc, &contract, &ports, &kpis())
}

fn contract(doc: &Value) -> Contract {
    let ports = ports();
    let mut c = derive::derive(doc, &ports).unwrap();
    let (resolved, refusals) = ingress::resolve(doc, &c, &ports, &kpis());
    assert_eq!(refusals, Vec::new());
    c.ingress = resolved;
    c
}

fn names(i: &ContractIngress, kind: OpKind) -> Vec<&str> {
    i.operations
        .iter()
        .filter(|o| o.kind == kind)
        .map(|o| o.name.as_str())
        .collect()
}

fn with_ingress(doc: &Value, ingress: Value) -> Value {
    let mut d = doc.clone();
    d["ingress"] = ingress;
    d
}

#[test]
fn a_document_without_ingress_has_the_default_operator_and_metrics_ingresses() {
    let (resolved, refusals) = resolve(&workstation());
    assert_eq!(refusals, Vec::new());
    let names_of: Vec<&str> = resolved.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names_of, ["metrics", "operator"]);

    let operator = &resolved[1];
    assert_eq!(operator.audience, "control.operator");
    assert!(operator.authenticated);
    assert_eq!(operator.exports, ["http", "event_stream", "ui", "tool"]);
    let commands = names(operator, OpKind::Command);
    // principal: all answers the owner's question port; plays: administration plays the
    // gateway's operator, whose configuration commands drop out.
    for c in [
        "agentic.question.Answer",
        "agentic.gateway.DrainGateway",
        "agentic.gateway.StopGateway",
        "agentic.gateway.DetachUpstream",
    ] {
        assert!(commands.contains(&c), "{c}: {commands:?}");
    }
    let p = ports();
    for c in &commands {
        let role = p.surface.role(c);
        assert!(
            matches!(role, Some(Role::Administration | Role::Report)),
            "{c} is {role:?}"
        );
    }
    assert!(!commands.contains(&"agentic.gateway.OpenGateway"));
    assert!(names(operator, OpKind::View).contains(&"agentic.gateway.Gateways"));
    assert!(names(operator, OpKind::Feed).contains(&"agentic.question.QuestionAsked"));

    let metrics = &resolved[0];
    assert_eq!(metrics.audience, "control.monitor");
    assert!(!metrics.authenticated);
    let k = kpis();
    let ids = names(metrics, OpKind::Metric);
    assert!(!ids.is_empty());
    for id in &ids {
        assert!(
            matches!(k.0[*id].status.as_str(), "computed" | "both"),
            "{id}"
        );
    }
    assert!(ids.contains(&"cost_usd"), "{ids:?}");
    assert_eq!(metrics.operations.len(), ids.len());
}

#[test]
fn a_default_is_replaced_by_name_and_removed_by_none() {
    let doc = workstation();
    let (only_metrics, refusals) =
        resolve(&with_ingress(&doc, serde_json::json!({"operator": "none"})));
    // The owner answers the operator's questions, and no ingress carries the answer now.
    let rules: Vec<&str> = refusals.iter().map(|r| r.rule).collect();
    assert_eq!(rules, ["principal_port_without_ingress"]);
    assert_eq!(only_metrics.len(), 1);
    assert_eq!(only_metrics[0].name, "metrics");

    let (none, refusals) = resolve(&with_ingress(&doc, Value::String("none".into())));
    assert!(none.is_empty());
    assert_eq!(refusals.len(), 1, "{refusals:?}");

    let replaced = with_ingress(
        &doc,
        serde_json::json!({"operator": {
            "audience": "control.operator",
            "interface": {"scope": "all", "views": true},
            "exports": [{"protocol": "http"}],
        }, "metrics": "none"}),
    );
    let (resolved, refusals) = resolve(&replaced);
    assert_eq!(
        refusals.iter().map(|r| r.rule).collect::<Vec<_>>(),
        ["principal_port_without_ingress"]
    );
    assert_eq!(resolved.len(), 1);
    assert!(names(&resolved[0], OpKind::Command).is_empty());
}

#[test]
fn the_printed_defaults_pasted_into_the_document_give_the_same_contract() {
    for arch in ["workstation.yaml", "solo.yaml", "shared-board.yaml"] {
        let doc = load(&repo_root().join("architectures").join(arch));
        let text = ingress::default_text(&doc);
        let pasted = with_ingress(
            &doc,
            agenticspec_sim::doc::parse_yaml(&text).unwrap()["ingress"].clone(),
        );
        assert_eq!(contract(&doc).text(), contract(&pasted).text(), "{arch}");
        assert_eq!(ingress::defaults_in_effect(&pasted), Vec::<&str>::new());
        assert!(ingress::default_text(&pasted).starts_with('#'));
    }
    // A document that declares one default gets the other printed only.
    let doc = with_ingress(&workstation(), serde_json::json!({"metrics": "none"}));
    assert_eq!(ingress::defaults_in_effect(&doc), ["operator"]);
    assert!(!ingress::default_text(&doc).contains("metrics"));
}

#[test]
fn ingress_default_prints_the_section_and_refuses_a_reserved_part() {
    let printed =
        cli::ingress_default(&repo_root().join("architectures/workstation.yaml")).unwrap();
    assert_eq!(
        printed["defaults"],
        serde_json::json!(["operator", "metrics"])
    );
    let text = printed["ingress"].as_str().unwrap();
    assert!(text.starts_with("ingress:\n  operator:\n    audience: control.operator\n"));
    assert!(text.contains("commands: { principal: all, plays: administration }"));
    assert!(text.contains("interface: { scope: all, metrics: computed }"));

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases/ingress/ingress_reserved_part.yaml");
    let err = cli::ingress_default(&fixture).unwrap_err();
    assert_eq!(err.code, "architecture_refused");
    assert!(
        err.detail
            .starts_with("ingress_reserved_part: part `control`")
    );
}

#[test]
fn sim_check_counts_the_defaults_and_reads_no_ports_without_ingress() {
    let dir = scratch("control-plane-check");
    let missing = dir.join("missing.json");
    let ok = cli::check_with(&CheckOptions {
        architecture: &repo_root().join("architectures/workstation.yaml"),
        ports: Some(&missing),
        kpis: Some(&missing),
    })
    .unwrap();
    assert_eq!(ok["ingresses"], 2);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases/ingress/ingress_reserved_part.yaml");
    let err = cli::check_with(&CheckOptions {
        architecture: &fixture,
        ports: Some(&missing),
        kpis: Some(&missing),
    })
    .unwrap_err();
    assert_eq!(err.code, "architecture_refused");
}

#[test]
fn every_architecture_has_a_contract_with_the_defaults_and_no_part_on_the_reserved_path() {
    let dir = repo_root().join("architectures");
    let mut n = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "yaml") {
            continue;
        }
        let doc = agenticspec_sim::nest::flatten(&agenticspec_sim::nest::load(&path).unwrap());
        assert_eq!(ingress::reserved(&doc), Vec::new(), "{}", path.display());
        let c = contract(&doc);
        assert!(
            c.ingress.iter().any(|i| i.name == "operator"),
            "{}",
            path.display()
        );
        n += 1;
    }
    assert!(n >= 10, "{n}");
}

#[test]
fn the_network_component_accepts_every_entry_command_an_export_may_serve() {
    let p = ports();
    let may: std::collections::BTreeSet<&String> = p.surface.actors.values().flatten().collect();
    let answers = &p.surface.actors[agenticspec_sim::contract::ports::PRINCIPAL];
    let mut expected: Vec<String> = p
        .surface
        .domains
        .values()
        .flat_map(|d| d.commands.iter())
        .filter(|(c, r)| matches!(r, Role::Intent | Role::Administration) || answers.contains(c))
        .filter(|(c, _)| may.contains(c))
        .map(|(c, _)| c.clone())
        .collect();
    expected.sort();
    let components = load(&repo_root().join("blocks/components.yaml"));
    let component = &components["components"][0];
    assert_eq!(component["reached_by"], "network");
    let mut accepted: Vec<String> = component["accepts"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_owned())
        .collect();
    accepted.sort();
    assert_eq!(accepted, expected);
}
