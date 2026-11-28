//! The `ingress:` key of an architecture document: one refusing fixture per rule of its check
//! (`tests/cases/ingress/<rule>.yaml`), the resolution of the worked example, the contract's
//! `ingress` section, and the surface of the ports file it is resolved against.

use super::common::{repo_root, yaml};
use agenticspec_sim::check::check;
use agenticspec_sim::cli::{self, CheckOptions, ContractOptions};
use agenticspec_sim::contract::derive::{self, Contract};
use agenticspec_sim::contract::ingress::{self, ContractIngress, Kpis, OpKind, RULES};
use agenticspec_sim::contract::ports::{PortsFile, Role};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn ports() -> PortsFile {
    PortsFile::load(&repo_root().join("blocks/ports.json")).unwrap()
}

fn kpis() -> Kpis {
    Kpis::load(&repo_root().join("kpis/catalogue.yaml")).unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/cases/ingress/{name}.yaml"))
}

fn load(path: &Path) -> Value {
    agenticspec_sim::doc::load_yaml(path).unwrap()
}

fn resolve(doc: &Value) -> (Vec<ContractIngress>, Vec<ingress::Refusal>) {
    let ports = ports();
    let contract = derive::derive(doc, &ports).unwrap();
    ingress::resolve(doc, &contract, &ports, &kpis())
}

fn full_contract(doc: &Value) -> Contract {
    let ports = ports();
    let mut contract = derive::derive(doc, &ports).unwrap();
    let (resolved, refusals) = ingress::resolve(doc, &contract, &ports, &kpis());
    assert_eq!(refusals, Vec::new());
    contract.ingress = resolved;
    contract
}

fn sim_check(path: &Path) -> Result<Value, cli::Refusal> {
    let ports = repo_root().join("blocks/ports.json");
    let kpis = repo_root().join("kpis/catalogue.yaml");
    cli::check_with(&CheckOptions {
        architecture: path,
        ports: Some(&ports),
        kpis: Some(&kpis),
    })
}

fn ingress<'a>(resolved: &'a [ContractIngress], name: &str) -> &'a ContractIngress {
    resolved.iter().find(|i| i.name == name).unwrap()
}

fn names(i: &ContractIngress, kind: OpKind) -> Vec<&str> {
    i.operations
        .iter()
        .filter(|o| o.kind == kind)
        .map(|o| o.name.as_str())
        .collect()
}

#[test]
fn every_rule_has_a_fixture_that_sim_check_refuses_by_that_rule_alone() {
    for rule in RULES {
        let path = fixture(rule);
        let doc = load(&path);
        assert_eq!(
            check(&doc),
            Vec::new(),
            "{rule}: the structural check refuses"
        );
        let (_, refusals) = resolve(&doc);
        let rules: Vec<&str> = refusals.iter().map(|r| r.rule).collect();
        assert_eq!(rules, vec![*rule], "{rule}: {refusals:?}");
        let err = sim_check(&path).unwrap_err();
        assert_eq!(err.code, "architecture_refused", "{rule}");
        assert!(
            err.detail.starts_with(&format!("{rule}: ")),
            "{}",
            err.detail
        );
    }
}

#[test]
fn the_small_document_with_two_ingresses_is_accepted() {
    let ok = sim_check(&fixture("accepted")).unwrap();
    assert_eq!(ok["ingresses"], 2);
    let (resolved, refusals) = resolve(&load(&fixture("accepted")));
    assert_eq!(refusals, Vec::new());
    let ops = ingress(&resolved, "ops");
    assert!(!ops.authenticated);
    assert_eq!(names(ops, OpKind::Metric), ["cost_usd"]);
    assert_eq!(names(ops, OpKind::Command), Vec::<&str>::new());
}

#[test]
fn the_workstation_example_resolves_as_the_design_says() {
    let doc = load(&repo_root().join("architectures/workstation-ingress.yaml"));
    let (resolved, refusals) = resolve(&doc);
    assert_eq!(refusals, Vec::new());
    let names_of: Vec<&str> = resolved.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names_of, ["metrics", "operator", "user"]);

    let user = ingress(&resolved, "user");
    assert_eq!(user.audience, "owner");
    assert_eq!(user.exports, ["http", "event_stream", "ui"]);
    assert_eq!(
        names(user, OpKind::Command),
        ["agentic.channel.SendMessage", "agentic.question.Answer"]
    );
    for view in [
        "agentic.question.OpenQuestions",
        "agentic.channel.ChannelMessages",
        "agentic.harness.HarnessSessions",
        "agentic.harness.Turns",
    ] {
        assert!(names(user, OpKind::View).contains(&view), "{view}");
    }
    assert!(!names(user, OpKind::View).contains(&"agentic.gateway.Gateways"));
    assert!(names(user, OpKind::Feed).contains(&"agentic.question.QuestionAsked"));

    let operator = ingress(&resolved, "operator");
    assert_eq!(
        names(operator, OpKind::Command),
        [
            "agentic.gateway.DetachUpstream",
            "agentic.gateway.DrainGateway",
            "agentic.gateway.StopGateway"
        ]
    );
    assert!(names(operator, OpKind::View).contains(&"agentic.gateway.Gateways"));
    for v in names(user, OpKind::View) {
        assert!(names(operator, OpKind::View).contains(&v), "{v}");
    }

    let metrics = ingress(&resolved, "metrics");
    assert!(!metrics.authenticated);
    assert_eq!(
        names(metrics, OpKind::Metric),
        [
            "cost_usd",
            "owner_wait_share",
            "pool_utilization",
            "queue_wait_p90_seconds",
            "rate_limited_seconds"
        ]
    );
    assert_eq!(metrics.operations.len(), 5);
}

#[test]
fn the_contract_writes_its_ingresses_with_the_same_bytes_and_reads_them_back() {
    let doc = load(&repo_root().join("architectures/workstation-ingress.yaml"));
    let first = full_contract(&doc);
    let second = full_contract(&doc);
    assert_eq!(first.text(), second.text());
    assert!(first.text().contains("\ningress:\n  - name: metrics\n    audience: monitor\n    authenticated: false\n    exports: [metrics_scrape]\n"));
    let read = Contract::parse(&first.text()).unwrap();
    assert_eq!(read, first);
    assert_eq!(derive::check(&read, &doc, &ports()), Vec::<String>::new());

    let mut wrong = first.clone();
    wrong.ingress[0].audience = "owner".into();
    wrong.ingress[1].exports.pop();
    let problems = derive::check(&wrong, &doc, &ports());
    assert_eq!(problems.len(), 2, "{problems:?}");
    let mut missing = first.clone();
    missing.ingress.clear();
    assert_eq!(derive::check(&missing, &doc, &ports()).len(), 1);
}

#[test]
fn sim_contract_resolves_the_ingresses_and_a_document_without_ingress_writes_none() {
    let root = repo_root();
    let (ports, kpis) = (
        root.join("blocks/ports.json"),
        root.join("kpis/catalogue.yaml"),
    );
    let printed = cli::contract(
        &ContractOptions {
            architecture: &root.join("architectures/workstation-ingress.yaml"),
            ports: Some(&ports),
            out: None,
            check: None,
            suite: None,
            trace: None,
            kpis: Some(&kpis),
        },
        &mut std::io::sink(),
    )
    .unwrap();
    assert_eq!(printed["ingresses"], 3);
    assert!(
        printed["contract"]
            .as_str()
            .unwrap()
            .contains("\ningress:\n")
    );
    let plain = cli::contract(
        &ContractOptions {
            architecture: &root.join("architectures/workstation.yaml"),
            ports: Some(&ports),
            out: None,
            check: None,
            suite: None,
            trace: None,
            kpis: Some(&root.join("no-such-catalogue.yaml")),
        },
        &mut std::io::sink(),
    )
    .unwrap();
    assert_eq!(plain["ingresses"], 0);
    assert!(!plain["contract"].as_str().unwrap().contains("ingress"));
    let err = cli::contract(
        &ContractOptions {
            architecture: &fixture("ingress_without_export"),
            ports: Some(&ports),
            out: None,
            check: None,
            suite: None,
            trace: None,
            kpis: Some(&kpis),
        },
        &mut std::io::sink(),
    )
    .unwrap_err();
    assert_eq!(err.code, "architecture_refused");
}

#[test]
fn the_audience_of_an_ingress_counts_as_reached() {
    let doc = load(&repo_root().join("architectures/workstation-ingress.yaml"));
    assert_eq!(check(&doc), Vec::new());
    let mut bare = doc.clone();
    bare.as_object_mut().unwrap().remove("ingress");
    let unreached: Vec<String> = check(&bare).iter().map(|r| r.part().to_owned()).collect();
    assert_eq!(unreached, ["admin", "monitor"]);
}

#[test]
fn the_ports_file_gives_each_command_its_role_and_each_actor_its_commands() {
    let p = ports();
    let s = &p.surface;
    assert_eq!(
        s.role("agentic.gateway.DrainGateway"),
        Some(Role::Administration)
    );
    assert_eq!(
        s.role("agentic.gateway.OpenGateway"),
        Some(Role::Configuration)
    );
    assert_eq!(s.role("agentic.gateway.AttachUpstream"), Some(Role::Intent));
    assert_eq!(s.role("agentic.gateway.CompleteCall"), Some(Role::Report));
    assert_eq!(s.role("agentic.question.Answer"), Some(Role::Report));
    assert_eq!(s.role("agentic.model.StartInference"), Some(Role::Internal));
    assert!(s.has_view("agentic.architecture.Ingresses"));
    assert!(s.has_event("agentic.architecture.IngressAdded"));
    assert!(
        s.actors["agentic.gateway.GatewayOperator"]
            .contains(&"agentic.gateway.DrainGateway".into())
    );
    // A ports file written before the surface was derived reads with an empty one.
    let old = PortsFile::parse(
        r#"{"format": "agenticspec-ports/1", "ess": "ess 0", "sources_sha256": "x", "ports": []}"#,
    )
    .unwrap();
    assert!(old.surface.actors.is_empty() && old.surface.domains.is_empty());
    // The roles of a small compiled system: a bound command is internal, a deployer's retire
    // that moves a state is administration.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases/ports/small.json");
    let ir: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let d = agenticspec_sim::contract::ports::derive(&ir).unwrap();
    assert_eq!(d.surface.role("small.job.Start"), Some(Role::Internal));
    assert_eq!(d.surface.role("small.job.Finish"), Some(Role::Report));
    assert_eq!(d.surface.role("small.ask.Answer"), Some(Role::Report));
}

#[test]
fn the_ingress_example_simulates_like_the_workstation() {
    use agenticspec_sim::{Experiment, run};
    let root = repo_root();
    let profile = load(&root.join("fields/coding/profiles/coding.yaml"));
    let experiment = yaml(
        "format: agenticspec-experiment/1\nexperiment: same\nhorizon: 4h\nseeds: 1\nseed: 3\n\
         optimize: marked\nsearch: { mode: fit, budget: 1, weights: { tasks_done: 1 } }\n",
    );
    let kpis_of = |arch: &str| {
        let e = Experiment::from_docs(experiment.clone(), load(&root.join(arch)), profile.clone())
            .unwrap();
        let mut kpis = run(&e, 1).unwrap()["baseline"]["kpis"].clone();
        // The audiences of the ingresses are principal parts of the document: the whole-architecture
        // KPI principals_total counts them; every simulated KPI is the same.
        kpis.as_object_mut().unwrap().remove("principals_total");
        kpis
    };
    assert_eq!(
        kpis_of("architectures/workstation.yaml"),
        kpis_of("architectures/workstation-ingress.yaml")
    );
}
