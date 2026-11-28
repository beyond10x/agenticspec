//! The deploy side: the `packing:` section of a binding and its strategies, `agenticspec
//! packing default`, admission and its refusals (one refusing binding per refusal under
//! `deploy/`), `agenticspec deploy bundle` and the fixture bundles of `bundles/`.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, BundleOptions};
use agenticspec_sim::deploy::admit::{self, Accepts, REFUSALS};
use agenticspec_sim::deploy::binding;
use agenticspec_sim::deploy::bundle::{self, MEMBERS};
use agenticspec_sim::deploy::fixtures;
use agenticspec_sim::deploy::packing::{self, Packing, STRATEGIES};
use agenticspec_sim::doc::{load_yaml, parse_yaml};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn cases() -> PathBuf {
    repo_root().join("crates/agenticspec-sim/tests/cases/deploy")
}

fn accepts() -> Accepts {
    Accepts::load(&repo_root().join("bundles/cloudspec-accepts.yaml")).unwrap()
}

/// The architectures `agenticspec sim check` accepts: the catalog's and the ladder's.
fn architectures() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    for dir in ["architectures", "architectures/ladder"] {
        let mut files: Vec<PathBuf> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
            .collect();
        files.sort();
        out.extend(files);
    }
    out
}

fn arch(name: &str) -> PathBuf {
    repo_root().join(format!("architectures/{name}.yaml"))
}

/// `deploy bundle` of an architecture under a binding, with the repository's ports file.
fn bundle_with(
    architecture: &Path,
    binding: &Path,
    accepts: Option<&Path>,
    out: Option<&Path>,
) -> Result<Value, cli::Refusal> {
    let root = repo_root();
    let ports = root.join("blocks/ports.json");
    let default_accepts = root.join("bundles/cloudspec-accepts.yaml");
    cli::deploy_bundle(&BundleOptions {
        architecture,
        binding,
        out,
        ports: Some(&ports),
        accepts: Some(accepts.unwrap_or(&default_accepts)),
    })
}

/// The names of the findings of a `not_admitted` refusal.
fn refusal_names(detail: &str) -> BTreeSet<String> {
    detail
        .split("; ")
        .map(|f| f.split(':').next().unwrap_or("").to_owned())
        .collect()
}

fn every_part_once(doc: &Value, p: &Packing) {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for u in &p.units {
        for part in &u.parts {
            *seen.entry(part).or_default() += 1;
        }
    }
    for e in &p.external {
        *seen.entry(e).or_default() += 1;
    }
    for name in doc["parts"].as_object().unwrap().keys() {
        assert_eq!(
            seen.get(name.as_str()),
            Some(&1),
            "{}: part {name} under {}",
            doc["architecture"],
            p.strategy
        );
    }
}

#[test]
fn the_default_packing_is_single_and_every_accepted_architecture_is_admitted_under_it() {
    for path in architectures() {
        let doc = load_yaml(&path).unwrap();
        let p = packing::resolve(&doc, &Value::Null).unwrap();
        assert_eq!(p.strategy, "single");
        assert_eq!(p.units.len(), 1, "{}", path.display());
        assert_eq!(p.units[0].name, "core");
        every_part_once(&doc, &p);
        for e in &p.external {
            let block = doc["parts"][e]["block"].as_str().unwrap();
            assert!(matches!(block, "principal" | "model"), "{e}: {block}");
        }
        let plan = admit::admit(&doc, &json!({}), &p, &accepts())
            .unwrap_or_else(|f| panic!("{}: {f:?}", path.display()));
        assert_eq!(plan.units.len(), 1);
        assert!(plan.crossings.is_empty());
    }
}

#[test]
fn every_strategy_places_each_part_once_and_per_parts_with_their_part() {
    for path in architectures() {
        let doc = load_yaml(&path).unwrap();
        for s in ["single", "by_kind", "by_part"] {
            let p = packing::resolve(&doc, &json!({ "strategy": s })).unwrap();
            every_part_once(&doc, &p);
            assert_eq!(p.units.iter().filter(|u| u.kind == "core").count(), 1);
            for (name, part) in doc["parts"].as_object().unwrap() {
                if let Some(x) = part["per"].as_str() {
                    assert_eq!(
                        p.unit_of(name).map(|u| &u.name),
                        p.unit_of(x).map(|u| &u.name),
                        "{}: {name} per {x} under {s}",
                        path.display()
                    );
                }
            }
        }
    }
    let doc = load_yaml(&arch("workstation")).unwrap();
    let p = packing::resolve(&doc, &json!({ "strategy": "by_part" })).unwrap();
    let worker = p.units.iter().find(|u| u.name == "worker").unwrap();
    assert_eq!(worker.kind, "agent");
    assert_eq!(worker.parts, ["box", "tree", "worker"]);
    assert_eq!(worker.replicas, Some(12), "count 12 is 12 replicas");
    let p = packing::resolve(&doc, &json!({ "strategy": "by_kind" })).unwrap();
    let agents = p.units.iter().find(|u| u.name == "agent").unwrap();
    assert_eq!(agents.parts, ["box", "operator", "tree", "worker"]);
    assert_eq!(agents.replicas, Some(1));
    let names: Vec<&str> = p.units.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names, ["core", "agent", "gateway", "pool"]);
}

#[test]
fn groups_put_the_rest_in_core_and_overrides_win() {
    let doc = load_yaml(&arch("scheduled-loops")).unwrap();
    let p = packing::resolve(
        &doc,
        &json!({
            "strategy": "groups",
            "groups": { "edge": ["gateway", "builds"] },
            "parts": { "sweeper": { "schedule": "job" }, "model": { "external": false } },
        }),
    )
    .unwrap();
    let kinds: BTreeMap<&str, (&str, &[String])> = p
        .units
        .iter()
        .map(|u| (u.name.as_str(), (u.kind.as_str(), u.parts.as_slice())))
        .collect();
    assert_eq!(kinds["edge"].0, "service");
    assert_eq!(kinds["edge"].1, ["builds", "gateway"]);
    assert_eq!(kinds["sweeper"].0, "scheduled");
    assert!(kinds["core"].1.contains(&"triager".to_owned()));
    assert!(
        kinds["core"].1.contains(&"model".to_owned()),
        "a served model in groups is in core"
    );
    assert_eq!(p.external, ["owner"]);
    assert_eq!(
        p.units
            .iter()
            .find(|u| u.name == "sweeper")
            .unwrap()
            .replicas,
        None
    );
}

#[test]
fn the_resolved_packing_reads_back_as_the_same_units() {
    for path in architectures() {
        let doc = load_yaml(&path).unwrap();
        for s in STRATEGIES.iter().filter(|s| **s != "groups") {
            let p = packing::resolve(&doc, &json!({ "strategy": s })).unwrap();
            let text = p.text();
            let section = &parse_yaml(&text).unwrap()["packing"];
            let back = packing::resolve(&doc, section).unwrap();
            assert_eq!(back.units, p.units, "{} {s}", path.display());
            assert_eq!(back.external, p.external);
            assert_eq!(back.strategy, "units");
            assert_eq!(back.text(), text);
        }
    }
}

#[test]
fn packing_default_prints_the_single_packing_and_refuses_an_unknown_strategy() {
    let path = arch("workstation");
    let out = cli::packing_default(&path, None).unwrap();
    assert_eq!(out["strategy"], "single");
    assert_eq!(out["units"], 1);
    assert_eq!(
        out["packing"],
        "packing:\n  units:\n    - name: core\n      kind: core\n      parts: [box, builds, chat, gateway, operator, tree, worker]\n      replicas: 1\n  external: [model, owner]\n"
    );
    let out = cli::packing_default(&path, Some("by_part")).unwrap();
    assert_eq!(out["units"], 5, "core, operator, worker, gateway, builds");
    let r = cli::packing_default(&path, Some("by_host")).unwrap_err();
    assert_eq!(r.code, "packing_refused");
    assert!(r.detail.contains("by_host"), "{}", r.detail);
}

#[test]
fn a_packing_section_of_the_wrong_form_is_refused_with_every_problem() {
    let doc = load_yaml(&arch("workstation")).unwrap();
    let e = packing::resolve(
        &doc,
        &json!({
            "strategy": "groups",
            "groups": { "g": ["nobody"] },
            "parts": { "gateway": { "schedule": "job" }, "worker": { "replicas": 3 } },
            "colour": "blue",
        }),
    )
    .unwrap_err();
    let text = e.join("\n");
    for needle in [
        "`colour`",
        "`nobody`",
        "`schedule: job` is for an agent part",
        "worker: `replicas`",
    ] {
        assert!(text.contains(needle), "{needle} in {text}");
    }
    let e = packing::resolve(
        &doc,
        &json!({ "strategy": "single", "units": [{ "name": "core", "kind": "pod", "parts": [] }] }),
    )
    .unwrap_err()
    .join("\n");
    assert!(
        e.contains("do not go together") && e.contains("kind `pod`"),
        "{e}"
    );
}

#[test]
fn each_refusal_of_admission_has_a_binding_that_is_refused_by_it_alone() {
    for name in REFUSALS {
        let file = cases().join(format!("{name}.yaml"));
        assert!(file.is_file(), "no refusing binding {}", file.display());
        let b = load_yaml(&file).unwrap();
        let architecture = arch(b["architecture"].as_str().unwrap());
        let accepts_file = cases().join(format!("{name}.accepts.yaml"));
        let accepts = accepts_file.is_file().then_some(accepts_file.as_path());
        let r = bundle_with(&architecture, &file, accepts, None).unwrap_err();
        assert_eq!(r.code, "not_admitted", "{name}: {}", r.detail);
        assert_eq!(
            refusal_names(&r.detail),
            BTreeSet::from([(*name).to_owned()]),
            "{name}: {}",
            r.detail
        );
    }
}

#[test]
fn a_refusal_lists_every_finding_and_writes_nothing() {
    let dir = scratch("deploy-refused");
    let out = dir.join("bundle.yaml");
    let file = dir.join("by-part.yaml");
    std::fs::write(
        &file,
        "format: agenticspec-binding/1\nbinding: by-part\npacking: {strategy: by_part}\n",
    )
    .unwrap();
    let r = bundle_with(&arch("workstation"), &file, None, Some(&out)).unwrap_err();
    assert_eq!(r.code, "not_admitted");
    // Five links cross units: operator to gateway and to worker, worker to builds, gateway and
    // operator; the gateway's rate limit stays in memory; the two agent units name no agent
    // document.
    let findings: Vec<&str> = r.detail.split("; ").collect();
    assert_eq!(findings.len(), 8, "{}", r.detail);
    assert!(
        findings[0].starts_with("shared_state_split"),
        "sorted by refusal"
    );
    let crossing = findings
        .iter()
        .filter(|f| f.starts_with("link_without_transport"))
        .count();
    assert_eq!(crossing, 5, "{}", r.detail);
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.starts_with("agent_document_missing"))
            .count(),
        2
    );
    assert!(!out.exists(), "a refused bundle is not written");
}

#[test]
fn a_binding_of_the_wrong_form_is_refused_before_admission() {
    let dir = scratch("deploy-binding-form");
    let file = dir.join("b.yaml");
    std::fs::write(
        &file,
        "format: agenticspec-binding/1\nbinding: odd\narchitecture: solo\ntarget: {isolation: steel}\nparts: {nobody: {}}\nclaims: {elsewhere: {cpu_millis: 0}}\nhosts: [a]\n",
    )
    .unwrap();
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid");
    for needle in [
        "`hosts`",
        "`architecture: solo`",
        "isolation is one of",
        "`nobody` is not a part",
        "`cpu_millis` is a whole number",
        "`elsewhere` is not a unit",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
}

#[test]
fn refines_resolves_a_chain_child_over_base_and_refuses_a_cycle() {
    let dir = scratch("deploy-refines");
    std::fs::create_dir_all(dir.join("base")).unwrap();
    std::fs::write(
        dir.join("base/b.yaml"),
        "format: agenticspec-binding/1\nbinding: base\nparts: {worker: {agent: w.yaml, secrets: [a]}}\ntransports: {order: a2a, claim: http}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("c.yaml"),
        "format: agenticspec-binding/1\nbinding: child\nrefines: base/b.yaml\ntransports: {claim: grpc}\n",
    )
    .unwrap();
    let b = binding::load(&dir.join("c.yaml")).unwrap();
    assert_eq!(b.doc["binding"], "child");
    assert_eq!(
        b.doc["transports"],
        json!({ "order": "a2a", "claim": "grpc" })
    );
    assert!(b.doc.get("refines").is_none());
    assert_eq!(b.agent_files["worker"], dir.join("base").join("w.yaml"));
    std::fs::write(
        dir.join("x.yaml"),
        "format: agenticspec-binding/1\nbinding: x\nrefines: y.yaml\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("y.yaml"),
        "format: agenticspec-binding/1\nbinding: y\nrefines: x.yaml\n",
    )
    .unwrap();
    let e = binding::load(&dir.join("x.yaml")).unwrap_err();
    assert!(e.contains("comes back"), "{e}");
}

#[test]
fn a_bundle_is_the_same_bytes_on_every_run_and_its_digest_holds() {
    let root = repo_root();
    let dir = scratch("deploy-bundle");
    let (a, b) = (dir.join("a.yaml"), dir.join("b.yaml"));
    let binding = root.join("bundles/bindings/split.yaml");
    let first = bundle_with(&arch("workstation"), &binding, None, Some(&a)).unwrap();
    let second = bundle_with(&arch("workstation"), &binding, None, Some(&b)).unwrap();
    let text = std::fs::read_to_string(&a).unwrap();
    assert_eq!(text, std::fs::read_to_string(&b).unwrap());
    assert_eq!(first["digest"], second["digest"]);
    assert_eq!(first["units"], 5);
    let v = parse_yaml(&text).unwrap();
    assert_eq!(v["format"], bundle::FORMAT);
    assert_eq!(bundle::recompute(&v), v["digest"].as_str().unwrap());
    assert_eq!(v["digest"], first["digest"]);
    let keys: Vec<&str> = text
        .lines()
        .filter(|l| !l.starts_with([' ', '-']))
        .map(|l| l.split(':').next().unwrap())
        .collect();
    assert_eq!(keys, MEMBERS);
    let doc = load_yaml(&arch("workstation")).unwrap();
    assert_eq!(v["architecture"]["revision"], bundle::revision_digest(&doc));
    assert_eq!(
        v["contract"]["digest"],
        agenticspec_sim::deploy::digest(&v["contract"]["document"])
    );
    assert_eq!(
        v["packing"]["digest"],
        agenticspec_sim::deploy::digest(&v["packing"]["document"])
    );
    assert_eq!(v["nested"], json!({}));
    assert_eq!(v["ingress"], json!([]));
    assert!(
        !text.contains(root.to_string_lossy().as_ref()),
        "no host path"
    );
}

#[test]
fn each_unit_of_a_bundle_carries_its_code_claims_isolation_ports_and_roles() {
    let root = repo_root();
    let out = bundle_with(
        &arch("workstation"),
        &root.join("bundles/bindings/split.yaml"),
        None,
        None,
    )
    .unwrap();
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    let units: BTreeMap<&str, &Value> = v["units"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| (u["name"].as_str().unwrap(), u))
        .collect();
    let worker = units["worker"];
    assert_eq!(worker["kind"], "agent");
    assert_eq!(worker["replicas"], 12);
    assert_eq!(worker["code"]["role"], "agent-runtime");
    let agent = worker["code"]["agents"][0]["agent"].as_str().unwrap();
    assert_eq!(v["agents"][agent]["parts"], json!(["worker"]));
    assert_eq!(v["agents"][agent]["document"]["format"], "loom.agent/1");
    assert_eq!(worker["claims"]["cpu_millis"], 2000);
    assert_eq!(
        worker["claims"]["volumes"],
        json!([{ "name": "tree", "access": "read_write_once", "per_instance": true, "size": "20Gi" }])
    );
    assert_eq!(worker["isolation"], "process");
    assert_eq!(worker["roles"], json!(["agent-runtime"]));
    let ports: BTreeSet<&str> = worker["ports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["port"].as_str().unwrap())
        .collect();
    assert!(
        ports.contains("harness.session") && ports.contains("workspace.lease"),
        "{ports:?}"
    );
    assert_eq!(worker["checks"]["startup"]["role"], "agent-runtime");
    assert_eq!(
        worker["source"],
        json!({ "unit": "worker", "parts": ["box", "tree", "worker"] })
    );
    let egress: Vec<(&str, &str)> = worker["egress"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["to"].as_str().unwrap(),
                e["transport"].as_str().unwrap_or(""),
            )
        })
        .collect();
    assert_eq!(
        egress,
        [("builds", "http"), ("gateway", "http"), ("operator", "a2a")]
    );
    let gateway = units["gateway"];
    assert_eq!(gateway["kind"], "service");
    assert_eq!(gateway["roles"], json!(["model-route:frontier"]));
    assert_eq!(gateway["secrets"], json!(["model-api-key"]));
    assert_eq!(gateway["egress"][0]["external"], true);
    let core = units["core"];
    let pinned = "sha256:0000000000000000000000000000000000000000000000000000000000000001";
    assert_eq!(core["code"], json!({ "runtime": pinned }));
    assert_eq!(v["runtimes"][pinned]["name"], "core");
    assert!(core.get("schedule").is_none());
    for u in units.values() {
        for k in [
            "name",
            "kind",
            "parts",
            "code",
            "claims",
            "isolation",
            "checks",
            "secrets",
            "egress",
            "roles",
            "ports",
            "source",
        ] {
            assert!(u.get(k).is_some(), "unit {} has no `{k}`", u["name"]);
        }
    }
}

#[test]
fn a_scheduled_unit_carries_its_clock_and_needs_no_transport_from_it() {
    let root = repo_root();
    let out = bundle_with(
        &arch("scheduled-loops"),
        &root.join("bundles/bindings/jobs.yaml"),
        None,
        None,
    )
    .unwrap();
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    let sweeper = v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == "sweeper")
        .unwrap();
    assert_eq!(sweeper["kind"], "scheduled");
    assert!(sweeper.get("replicas").is_none());
    assert_eq!(
        sweeper["schedule"],
        json!({ "trigger": "sweep", "schedule": "nightly", "every": "24h", "next_fire_at": "2h", "policy": "skip", "max_parallel": 1 })
    );
}

#[test]
fn the_fixture_bundles_are_current_and_hold_only_accepted_kinds_and_roles() {
    let root = repo_root();
    assert_eq!(fixtures::check(&root).unwrap(), Vec::<String>::new());
    assert_eq!(fixtures::stale(&root).unwrap(), Vec::<String>::new());
    let all = fixtures::fixtures(&root).unwrap();
    assert_eq!(
        all.len(),
        8,
        "six catalog architectures under `local`, two named bindings"
    );
    let accepts = accepts();
    for f in all {
        let v = load_yaml(&root.join("bundles").join(&f.file)).unwrap();
        for u in v["units"].as_array().unwrap() {
            assert!(
                accepts.unit_kinds.contains(u["kind"].as_str().unwrap()),
                "{}",
                f.file
            );
            for r in u["roles"].as_array().unwrap() {
                let name = r.as_str().unwrap().split(':').next().unwrap();
                assert!(accepts.roles.contains(name), "{}: role {r}", f.file);
            }
        }
    }
}
