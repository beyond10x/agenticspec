//! `agenticspec sim trace` and `agenticspec sim contract --trace`: the port uses of one simulated
//! run as a trace (`agenticspec-trace/1`), and the check of a contract's routing invariants on a
//! trace, on hand-written traces and on the traces of the catalog's architectures.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, ContractOptions, Refusal, TraceOptions};
use agenticspec_sim::contract::derive;
use agenticspec_sim::contract::ports::PortsFile;
use agenticspec_sim::contract::trace::{self, Phase, Status, Trace};
use agenticspec_sim::marked::resolve;
use agenticspec_sim::profile::Profile;
use agenticspec_sim::sim::{Model, PortUse, RECORDED, simulate};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn case(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases/trace")
        .join(name)
}

/// `sim contract <architecture> --trace <trace>`: the checks, or the refusal.
fn check(architecture: &Path, trace: &Path) -> Result<Value, Refusal> {
    let root = repo_root();
    cli::contract(
        &ContractOptions {
            architecture,
            ports: Some(&root.join("blocks/ports.json")),
            out: None,
            check: None,
            suite: None,
            trace: Some(trace),
            kpis: None,
        },
        &mut std::io::sink(),
    )
}

fn workstation() -> PathBuf {
    repo_root().join("architectures/workstation.yaml")
}

fn status_of<'a>(checks: &'a Value, link: &str) -> &'a Value {
    checks["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["link"] == link)
        .unwrap_or_else(|| panic!("no check of {link}"))
}

/// Write the trace of one run of `architecture` (a path under the repository) and return it.
fn simulated(architecture: &str, dir: &Path) -> PathBuf {
    let root = repo_root();
    let out = dir.join("trace.json");
    let answer = cli::trace(&TraceOptions {
        architecture: &root.join(architecture),
        profile: &root.join("fields/coding/profiles/coding.yaml"),
        seed: Some(1),
        horizon: None,
        out: Some(&out),
    })
    .unwrap();
    assert_eq!(answer["written"], true);
    out
}

#[test]
fn a_hand_written_trace_that_keeps_every_route_holds_every_invariant() {
    let answer = check(&workstation(), &case("holds.json")).unwrap();
    for c in answer["checks"].as_array().unwrap() {
        assert_eq!(c["status"], "held", "{c}");
    }
    let inside = status_of(&answer, "worker -> gateway carries model_call");
    assert_eq!(inside["uses"], 1);
    assert_eq!(
        status_of(&answer, "operator -> owner carries question over chat")["uses"],
        1
    );
    assert_eq!(
        status_of(&answer, "owner -> operator carries prompt over chat")["uses"],
        1
    );
}

#[test]
fn each_kind_of_invariant_is_broken_by_its_own_trace_naming_link_and_event() {
    for (file, link, event) in [
        (
            "breaks-inside.json",
            "worker -> gateway carries model_call",
            3,
        ),
        ("breaks-to.json", "worker -> builds carries claim", 4),
        (
            "breaks-over.json",
            "operator -> owner carries question over chat",
            7,
        ),
    ] {
        let err = check(&workstation(), &case(file)).unwrap_err();
        assert_eq!(err.code, "invariant_violated", "{file}");
        assert!(
            err.detail.contains(&format!("link `{link}`")),
            "{file}: {}",
            err.detail
        );
        assert!(
            err.detail.contains(&format!("event {event} ")),
            "{file}: {}",
            err.detail
        );
        assert_eq!(
            err.detail.matches("link `").count(),
            1,
            "{file}: {}",
            err.detail
        );
    }
}

#[test]
fn an_invariant_over_a_port_the_trace_does_not_record_is_not_checked_never_held() {
    let answer = check(&case("tool-call.yaml"), &case("tool-call.json")).unwrap();
    let call = status_of(&answer, "agent -> tests carries call");
    assert_eq!(call["status"], "not_checked");
    assert_eq!(call["uses"], 0);
    assert!(
        call["detail"]
            .as_str()
            .unwrap()
            .contains("harness.tool-call"),
        "{call}"
    );
    assert_eq!(
        status_of(&answer, "agent -> model carries model_call")["status"],
        "held"
    );
}

#[test]
fn a_trace_of_another_architecture_or_of_a_broken_shape_is_refused() {
    let err = check(
        &repo_root().join("architectures/solo.yaml"),
        &case("holds.json"),
    )
    .unwrap_err();
    assert_eq!(err.code, "trace_unreadable");
    assert!(err.detail.contains("workstation"), "{}", err.detail);
    let text = std::fs::read_to_string(case("holds.json")).unwrap();
    for (from, to, why) in [
        ("\"index\": 3,", "\"index\": 4,", "index is"),
        (
            "\"phase\": \"end\", \"begin\": 1}",
            "\"phase\": \"end\", \"begin\": 0}",
            "another use",
        ),
        ("\"within\": 6,", "\"within\": 9,", "no earlier begin"),
    ] {
        let broken = text.replacen(from, to, 1);
        assert_ne!(broken, text);
        let err = Trace::parse(&broken).unwrap_err();
        assert!(err.contains(why), "{why}: {err}");
    }
}

#[test]
fn uses_become_events_by_time_with_enclosing_uses_first_and_their_ends_last() {
    let outer = PortUse {
        port: "gateway.call",
        part: "g".into(),
        instance: 0,
        begin: 10.0,
        end: 10.0,
        within: None,
        to: Some("m".into()),
        over: None,
    };
    let mut inner = outer.clone();
    inner.port = "model.inference";
    inner.part = "w".into();
    inner.within = Some(0);
    let mut late = outer.clone();
    late.begin = 2.0;
    late.end = f64::INFINITY;
    let t = Trace::from_uses("a", json!({}), RECORDED, &[outer, inner, late]);
    let order: Vec<(&str, Phase)> = t
        .events
        .iter()
        .map(|e| (e.part.as_str(), e.phase))
        .collect();
    assert_eq!(
        order,
        [
            ("g", Phase::Begin),
            ("g", Phase::Begin),
            ("w", Phase::Begin),
            ("w", Phase::End),
            ("g", Phase::End),
        ]
    );
    assert_eq!(t.events[2].within, Some(1));
    assert_eq!(t.events[3].begin, Some(2));
    assert_eq!(
        t.events[4].begin,
        Some(1),
        "the use without an end has no end event"
    );
    assert_eq!(Trace::parse(&t.text()).unwrap(), t);
}

#[test]
fn the_workstation_trace_holds_every_invariant_and_breaks_when_an_inference_leaves_its_call() {
    let dir = scratch("trace-workstation");
    let file = simulated("architectures/workstation.yaml", &dir);
    let answer = check(&workstation(), &file).unwrap();
    for c in answer["checks"].as_array().unwrap() {
        assert_eq!(c["status"], "held", "{c}");
    }
    let inside = status_of(&answer, "worker -> gateway carries model_call");
    assert!(inside["uses"].as_u64().unwrap() > 0, "{inside}");
    assert!(
        status_of(&answer, "worker -> builds carries claim")["uses"]
            .as_u64()
            .unwrap()
            > 0
    );

    // Move one inference of a worker to before the gateway call it lies inside.
    let mut t = Trace::parse(&std::fs::read_to_string(&file).unwrap()).unwrap();
    let moved = t
        .events
        .iter()
        .find(|e| e.port == "model.inference" && e.part == "worker" && e.phase == Phase::Begin)
        .unwrap()
        .index;
    let call = t.events[moved].within.unwrap();
    t.events[moved].time = t.events[call].time - 1.0;
    let broken = dir.join("moved.json");
    std::fs::write(&broken, t.text()).unwrap();
    let err = check(&workstation(), &broken).unwrap_err();
    assert_eq!(err.code, "invariant_violated");
    assert!(
        err.detail.contains(
            "link `worker -> gateway carries model_call`: model.inference@worker: event "
        ),
        "{}",
        err.detail
    );
    assert!(
        err.detail.contains(&format!("event {moved} ")),
        "{}",
        err.detail
    );
}

#[test]
fn every_catalog_architecture_keeps_its_routes_and_recording_changes_no_kpi() {
    let root = repo_root();
    let ports = PortsFile::load(&root.join("blocks/ports.json")).unwrap();
    let profile_doc = agenticspec_sim::doc::load_yaml(&root.join("fields/coding/profiles/coding.yaml")).unwrap();
    let profile = Profile::parse(&profile_doc).unwrap();
    let mut files: Vec<PathBuf> = std::fs::read_dir(root.join("architectures"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    files.sort();
    assert!(files.len() >= 10);
    for path in files {
        let doc = agenticspec_sim::nest::flatten(&agenticspec_sim::nest::load(&path).unwrap());
        let built = resolve(&doc, &serde_json::Map::new()).unwrap();
        let model = Model::build(&built, &profile, 86400.0, &[], &[]).unwrap();
        let recorded = simulate(&model, 1, true);
        assert_eq!(
            simulate(&model, 1, false).kpis,
            recorded.kpis,
            "{}",
            path.display()
        );
        assert!(!recorded.uses.is_empty(), "{}", path.display());
        let name = doc["architecture"].as_str().unwrap();
        let t = Trace::from_uses(name, json!({}), RECORDED, &recorded.uses);
        let contract = derive::derive(&doc, &ports).unwrap();
        for c in trace::check(&contract, &t).unwrap() {
            assert_eq!(c.status, Status::Held, "{}: {c:?}", path.display());
        }
    }
}

#[test]
fn the_same_seed_writes_the_same_trace() {
    let a = std::fs::read(simulated(
        "architectures/shared-board.yaml",
        &scratch("trace-a"),
    ))
    .unwrap();
    let b = std::fs::read(simulated(
        "architectures/shared-board.yaml",
        &scratch("trace-b"),
    ))
    .unwrap();
    assert_eq!(a, b);
    let t = Trace::parse(std::str::from_utf8(&a).unwrap()).unwrap();
    assert!(t.events.iter().any(|e| e.port == "channel.post"));
    assert!(t.events.iter().any(|e| e.port == "trigger.firing"));
}
