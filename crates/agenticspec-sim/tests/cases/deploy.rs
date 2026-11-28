//! The deploy side: the `packing:` section of a binding and its strategies, `agenticspec
//! packing default`, admission and its refusals (one refusing binding per refusal under
//! `deploy/`), `agenticspec deploy bundle` and the fixture bundles of `bundles/`.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, BundleOptions};
use agenticspec_sim::deploy::admit::{self, Accepts, REFUSALS};
use agenticspec_sim::deploy::binding;
use agenticspec_sim::deploy::bundle::{self, MEMBERS};
use agenticspec_sim::deploy::fixtures;
use agenticspec_sim::deploy::grants;
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
        kpis: Some(&root.join("kpis/catalogue.yaml")),
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
        // A refusal of the architecture itself has its document beside the binding.
        let own = cases().join(format!("{name}.architecture.yaml"));
        let architecture = if own.is_file() {
            own
        } else {
            arch(b["architecture"].as_str().unwrap())
        };
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
    // document; seven links leave their unit, the five and two to external parts, and no part
    // declares its egress; the coding agents' tree is claimed by their unit and mounted nowhere.
    let findings: Vec<&str> = r.detail.split("; ").collect();
    assert_eq!(findings.len(), 16, "{}", r.detail);
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.starts_with("volume_mount_missing"))
            .count(),
        1
    );
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.starts_with("egress_undeclared"))
            .count(),
        7
    );
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
    assert_eq!(
        v["ingress"].as_array().unwrap().len(),
        5,
        "one entry per export of the default ingresses"
    );
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
        json!([{ "name": "tree", "access": "read_write_once", "per_instance": true, "size": "20Gi", "mount": "/workspace" }])
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
        [
            ("docs-mirror", ""),
            ("builds", "http"),
            ("gateway", "http"),
            ("operator", "a2a")
        ],
        "a named destination first, then the links"
    );
    let gateway = units["gateway"];
    assert_eq!(gateway["kind"], "service");
    assert_eq!(gateway["roles"], json!(["model-route:frontier"]));
    assert_eq!(gateway["secrets"], json!(["model-api-key"]));
    assert_eq!(gateway["egress"][0]["external"], true);
    let core = units["core"];
    let pinned = "sha256:0000000000000000000000000000000000000000000000000000000000000001";
    let image = format!("registry.example/agenticspec-core@{pinned}");
    assert_eq!(core["code"], json!({ "runtime": image }));
    assert_eq!(
        v["runtimes"][pinned],
        json!({ "name": "core", "image": image })
    );
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
    assert_eq!(sweeper["kind"], "agent");
    assert_eq!(sweeper["shape"], "scheduled");
    assert!(sweeper.get("replicas").is_none());
    assert_eq!(sweeper["schedule"], "0 2 * * *");
    assert_eq!(
        sweeper["clock"],
        json!({ "trigger": "sweep", "schedule": "nightly", "every": "24h", "next_fire_at": "2h", "policy": "skip", "max_parallel": 1 })
    );
    let packed = v["packing"]["document"]["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == "sweeper")
        .unwrap();
    assert_eq!(
        packed["kind"], "scheduled",
        "the packing keeps its own kinds"
    );
    assert_eq!(packed["schedule"], "0 2 * * *");
}

#[test]
fn an_agent_unit_carries_its_shape_service_job_or_scheduled() {
    let v = fixture("workstation.split.yaml");
    let worker = unit_of(&v, "worker");
    assert_eq!(
        (&worker["kind"], &worker["shape"]),
        (&json!("agent"), &json!("service"))
    );
    assert!(worker.get("schedule").is_none() && worker.get("clock").is_none());
    assert!(unit_of(&v, "core").get("shape").is_none());
    let dir = scratch("deploy-agent-shape");
    let doc = load_yaml(&arch("workstation")).unwrap();
    // `shape: job` is `schedule: job`: a part no clock wakes runs as a job.
    let p = packing::resolve(&doc, &json!({ "parts": { "worker": { "shape": "job" } } })).unwrap();
    let w = p.units.iter().find(|u| u.name == "worker").unwrap();
    assert_eq!(
        (w.kind.as_str(), w.replicas, &w.schedule),
        ("job", None, &None)
    );
    let file = workstation_binding(
        &dir,
        &format!(
            "parts:\n  worker: {{agent: {}}}\n  gateway: {{state: service}}\n  builds: {{state: service}}\ntransports: {{question: a2a, order: a2a, model_call: http, claim: http}}\npacking:\n  parts:\n    worker: {{shape: job}}\n",
            repo_root().join("bundles/agents/coder.yaml").display()
        ),
    );
    let out = bundle_with(&arch("workstation"), &file, None, None)
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    let worker = unit_of(&v, "worker");
    assert_eq!(
        (&worker["kind"], &worker["shape"]),
        (&json!("agent"), &json!("job"))
    );
    assert!(worker.get("schedule").is_none());
    // A cron rule makes a part a scheduled unit with it; one of the wrong form is refused.
    let p = packing::resolve(
        &doc,
        &json!({ "parts": { "worker": { "schedule": "*/15 * * * MON-FRI" } } }),
    )
    .unwrap();
    let w = p.units.iter().find(|u| u.name == "worker").unwrap();
    assert_eq!(w.kind, "scheduled");
    assert_eq!(w.schedule.as_deref(), Some("*/15 * * * MON-FRI"));
    let back = packing::resolve(&doc, &parse_yaml(&p.text()).unwrap()["packing"]).unwrap();
    assert_eq!(
        back.units, p.units,
        "the cron reads back from the resolved form"
    );
    let e = packing::resolve(
        &doc,
        &json!({ "parts": { "worker": { "schedule": "nightly" }, "gateway": { "shape": "job" } } }),
    )
    .unwrap_err()
    .join("\n");
    for needle in [
        "worker: `schedule` is not",
        "`shape: job` is for an agent part",
    ] {
        assert!(e.contains(needle), "{needle} in {e}");
    }
    let e = packing::resolve(
        &doc,
        &json!({ "units": [{ "name": "core", "kind": "core", "parts": [], "schedule": "0 2 * * *" }] }),
    )
    .unwrap_err()
    .join("\n");
    assert!(e.contains("schedule is a cron rule"), "{e}");
    assert!(packing::is_cron("0 2 * * *") && !packing::is_cron("0 2 * *"));
}

#[test]
fn an_agent_unit_names_the_agent_runtime_the_binding_pins() {
    let dir = scratch("deploy-agent-runtime");
    let d = "sha256:00000000000000000000000000000000000000000000000000000000000000a2";
    let agents = format!(
        "parts:\n  operator: {{agent: {0}/operator.yaml}}\n  worker: {{agent: {0}/coder.yaml}}\n  gateway: {{state: service}}\n  builds: {{state: service}}\npacking: {{strategy: by_part}}\ntransports: {{question: a2a, order: a2a, model_call: http, claim: http}}\n",
        repo_root().join("bundles/agents").display()
    );
    let file = workstation_binding(
        &dir,
        &format!(
            "{agents}runtimes:\n  agent-runtime: {{image: registry.example/agent-runtime, digest: \"{d}\"}}\n"
        ),
    );
    let out = bundle_with(&arch("workstation"), &file, None, None)
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    let image = format!("registry.example/agent-runtime@{d}");
    assert_eq!(unit_of(&v, "worker")["code"]["runtime"], json!(image));
    assert_eq!(unit_of(&v, "worker")["code"]["role"], "agent-runtime");
    assert_eq!(
        v["runtimes"][d],
        json!({ "name": "agent-runtime", "image": image })
    );
    assert!(
        unit_of(&v, "core")["code"].get("runtime").is_none(),
        "the core runtime is not the agents'"
    );
    // From the target's local store: a tag is enough, neither tag nor digest is refused.
    let file = workstation_binding(
        &dir,
        &format!(
            "{agents}target: {{images: local}}\nruntimes:\n  agent-runtime: {{image: agent-runtime, tag: dev}}\n"
        ),
    );
    let out = bundle_with(&arch("workstation"), &file, None, None)
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    assert_eq!(
        unit_of(&v, "operator")["code"]["runtime"],
        "agent-runtime:dev"
    );
    let file = workstation_binding(
        &dir,
        &format!(
            "{agents}target: {{images: local}}\nruntimes:\n  agent-runtime: {{image: agent-runtime}}\n"
        ),
    );
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(
        refusal_names(&r.detail),
        BTreeSet::from(["image_unpinned".to_owned()]),
        "{}",
        r.detail
    );
    // Without one the target binds the role; the older name `agent` is refused by its new one.
    let file = workstation_binding(&dir, &agents);
    let out = bundle_with(&arch("workstation"), &file, None, None).unwrap();
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    assert!(unit_of(&v, "worker")["code"].get("runtime").is_none());
    let file = workstation_binding(
        &dir,
        &format!(
            "{agents}runtimes:\n  agent: {{image: registry.example/agent-runtime, digest: \"{d}\"}}\n"
        ),
    );
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    assert!(
        r.detail.contains("`agent` is named `agent-runtime`"),
        "{}",
        r.detail
    );
}

#[test]
fn an_agent_document_travels_as_its_exact_bytes_keyed_by_their_digest() {
    let v = fixture("workstation.split.yaml");
    let bytes = std::fs::read(repo_root().join("bundles/agents/coder.yaml")).unwrap();
    let d = agenticspec_sim::deploy::sha256(&bytes);
    let entry = &v["agents"][&d];
    assert_eq!(entry["parts"], json!(["worker"]));
    let carried =
        agenticspec_sim::deploy::ui::unbase64(entry["content_base64"].as_str().unwrap()).unwrap();
    assert_eq!(carried, bytes);
    assert_eq!(
        entry["document"],
        parse_yaml(std::str::from_utf8(&bytes).unwrap()).unwrap()
    );
    assert_eq!(
        unit_of(&v, "worker")["code"]["agents"][0]["agent"],
        json!(d)
    );
}

#[test]
fn deploy_check_refuses_an_agent_document_whose_bytes_do_not_hold() {
    let dir = scratch("deploy-check-agent-bytes");
    let coder = agenticspec_sim::deploy::sha256(
        &std::fs::read(repo_root().join("bundles/agents/coder.yaml")).unwrap(),
    );
    let (c1, c2, c3) = (coder.clone(), coder.clone(), coder);
    let cases: [Edit; 3] = [
        Box::new(move |v| {
            v["agents"][&c1]["content_base64"] = json!(agenticspec_sim::deploy::ui::base64(
                b"format: loom.agent/1\n"
            ));
        }),
        Box::new(move |v| {
            v["agents"][&c2]
                .as_object_mut()
                .unwrap()
                .remove("content_base64");
        }),
        Box::new(move |v| {
            v["agents"][&c3]["document"]["format"] = json!("loom.agent/2");
        }),
    ];
    for edit in cases {
        let r = check_text(&dir, &tampered(edit)).unwrap_err();
        assert_eq!(r.code, "not_admitted", "{}", r.detail);
        assert_eq!(
            refusal_names(&r.detail),
            BTreeSet::from(["agent_document_mismatch".to_owned()]),
            "{}",
            r.detail
        );
    }
    assert_eq!(bundle::CHECK_REFUSALS, ["agent_document_mismatch"]);
}

#[test]
fn a_part_of_a_pack_block_is_packed_bundled_and_checked_like_any_part() {
    let path = repo_root().join("fields/coding/architectures/auto-merger.yaml");
    let doc = load_yaml(&path).unwrap();
    let pack = ["changes", "checks", "forge", "repo"];
    for s in STRATEGIES.iter().filter(|s| **s != "groups") {
        let p = packing::resolve(&doc, &json!({ "strategy": s })).unwrap();
        every_part_once(&doc, &p);
        for part in pack {
            assert_eq!(
                p.unit_of(part).map(|u| u.kind.as_str()),
                Some("core"),
                "{s}: {part}"
            );
        }
    }
    assert_eq!(cli::packing_default(&path, None).unwrap()["units"], 1);
    let dir = scratch("deploy-pack-block");
    let file = dir.join("b.yaml");
    std::fs::write(
        &file,
        "format: agenticspec-binding/1\nbinding: probe\narchitecture: auto-merger\nparts:\n  gateway: {egress: [model]}\n  merger: {egress: [owner]}\n",
    )
    .unwrap();
    let out_file = dir.join("bundle.yaml");
    bundle_with(&path, &file, None, Some(&out_file))
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = load_yaml(&out_file).unwrap();
    let core = unit_of(&v, "core");
    for part in pack {
        assert!(
            core["parts"].as_array().unwrap().iter().any(|p| p == part),
            "{part}"
        );
    }
    assert!(
        core["ports"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| !pack.contains(&p["part"].as_str().unwrap())),
        "a pack block declares no port of the contract"
    );
    let accepts = repo_root().join("bundles/cloudspec-accepts.yaml");
    cli::deploy_check(&out_file, Some(&accepts))
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
}

#[test]
fn the_fixture_bundles_are_current_and_hold_only_accepted_kinds_and_roles() {
    let root = repo_root();
    assert_eq!(fixtures::check(&root).unwrap(), Vec::<String>::new());
    assert_eq!(fixtures::stale(&root).unwrap(), Vec::<String>::new());
    let all = fixtures::fixtures(&root).unwrap();
    assert_eq!(
        all.len(),
        9,
        "six catalog architectures under `local`, three named bindings"
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

#[test]
fn a_secret_name_is_lower_case_words_and_nothing_that_looks_like_a_value() {
    for ok in ["model-api-key", "chat-token", "a", "db2-password-ref"] {
        assert!(grants::is_name(ok), "{ok}");
    }
    for refused in [
        "",
        "Model-Api-Key",
        "2fa-seed",
        "API_KEY=x",
        "sk live",
        "key:abc",
        "a/b",
        "dGhpcyBpcyBub3QgYSBrZXk=",
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "abcdefghijabcdefghijk",
    ] {
        assert!(!grants::is_name(refused), "{refused}");
    }
    assert!(!grants::is_name(&"ab-".repeat(22)), "longer than 63");
}

#[test]
fn a_secret_value_is_refused_by_where_it_is_never_by_its_text() {
    let file = cases().join("secret_value.yaml");
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    let findings: Vec<&str> = r.detail.split("; ").collect();
    assert_eq!(findings.len(), 4, "{}", r.detail);
    for needle in [
        "parts: operator: secrets[0] is not a secret name",
        "parts: gateway: secrets[1] is not a secret name",
        "parts: gateway has `value:`",
        "parts: gateway: egress[0] has `data:`",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
    assert!(!r.detail.contains("placeholder"), "{}", r.detail);
    assert!(!r.detail.contains("Model Key Text"), "{}", r.detail);
}

#[test]
fn secrets_and_egress_are_carried_per_unit_by_name_with_the_part_they_start_at() {
    let root = repo_root();
    let out = bundle_with(
        &arch("workstation"),
        &root.join("bundles/bindings/split.yaml"),
        None,
        None,
    )
    .unwrap();
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    let unit = |n: &str| {
        v["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["name"] == n)
            .unwrap()
            .clone()
    };
    let gateway = unit("gateway");
    assert_eq!(
        gateway["egress"],
        json!([{ "link": "gateway -> model carries model_call", "from": "gateway", "to": "model", "carries": "model_call", "external": true, "secret": "model-api-key", "name": "models.example:443", "address": "models.example:443" }])
    );
    let operator = unit("operator");
    assert_eq!(operator["secrets"], json!(["chat-token"]));
    assert_eq!(
        operator["egress"][0],
        json!({ "from": "operator", "to": "chat-relay", "named": true, "secret": "chat-token", "name": "chat-relay", "address": "chat-relay.example:443" }),
        "a named destination comes first, it has no link"
    );
    let to: Vec<&str> = operator["egress"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["to"].as_str().unwrap())
        .collect();
    assert_eq!(to, ["chat-relay", "gateway", "owner", "worker"]);
    assert_eq!(unit("core")["egress"], json!([]));
}

#[test]
fn an_egress_entry_of_the_wrong_form_is_refused_before_admission() {
    let dir = scratch("deploy-egress-form");
    let file = dir.join("b.yaml");
    std::fs::write(
        &file,
        "format: agenticspec-binding/1\nbinding: odd\nparts:\n  gateway: {egress: model}\n  operator: {egress: [\"https://chat.example\", {to: owner, via: chat}, 3]}\n",
    )
    .unwrap();
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    for needle in [
        "gateway: egress is a list of destinations",
        "operator: egress[0] names neither a part",
        "operator: egress[1]: unknown key `via`",
        "operator: egress[2] is a destination name",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
}

/// `deploy check` of a bundle as YAML text written to a scratch file.
fn check_text(dir: &Path, text: &str) -> Result<Value, cli::Refusal> {
    let file = dir.join("bundle.yaml");
    std::fs::write(&file, text).unwrap();
    let accepts = repo_root().join("bundles/cloudspec-accepts.yaml");
    cli::deploy_check(&file, Some(&accepts))
}

/// The split fixture bundle changed by `edit`, its binding and bundle digests recomputed.
fn tampered(edit: impl FnOnce(&mut Value)) -> String {
    let root = repo_root();
    let mut v = load_yaml(&root.join("bundles/workstation.split.yaml")).unwrap();
    edit(&mut v);
    v["binding"]["digest"] = json!(agenticspec_sim::deploy::digest(&v["binding"]["document"]));
    v["digest"] = json!(bundle::recompute(&v));
    bundle::write(&v)
}

/// A change of a bundle read back.
type Edit = Box<dyn FnOnce(&mut Value)>;

fn unit_mut<'a>(v: &'a mut Value, name: &str) -> &'a mut Value {
    v["units"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|u| u["name"] == name)
        .unwrap()
}

#[test]
fn deploy_check_admits_every_fixture_bundle_again() {
    let root = repo_root();
    let accepts = root.join("bundles/cloudspec-accepts.yaml");
    for f in fixtures::fixtures(&root).unwrap() {
        let path = root.join("bundles").join(&f.file);
        let out = cli::deploy_check(&path, Some(&accepts))
            .unwrap_or_else(|r| panic!("{}: {}: {}", f.file, r.code, r.detail));
        let v = load_yaml(&path).unwrap();
        assert_eq!(out["digest"], v["digest"]);
        assert_eq!(out["units"], v["units"].as_array().unwrap().len());
    }
}

#[test]
fn deploy_check_refuses_an_undeclared_secret_or_egress_and_a_value_each_by_name() {
    let dir = scratch("deploy-check-refusals");
    let cases: [(&str, Edit); 4] = [
        (
            "secret_undeclared",
            Box::new(|v| {
                unit_mut(v, "worker")["secrets"] = json!(["model-api-key"]);
            }),
        ),
        (
            "egress_undeclared",
            Box::new(|v| {
                unit_mut(v, "worker")["egress"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({ "from": "worker", "to": "package-mirror", "named": true }));
            }),
        ),
        (
            "secret_value",
            Box::new(|v| {
                let e = &mut unit_mut(v, "gateway")["egress"][0];
                e["secret"] = json!("Bearer placeholder");
                e["token"] = json!("placeholder");
            }),
        ),
        (
            "egress_undeclared",
            Box::new(|v| {
                v["binding"]["document"]["parts"]["gateway"]
                    .as_object_mut()
                    .unwrap()
                    .remove("egress");
            }),
        ),
    ];
    for (name, edit) in cases {
        let r = check_text(&dir, &tampered(edit)).unwrap_err();
        assert_eq!(r.code, "not_admitted", "{name}: {}", r.detail);
        assert_eq!(
            refusal_names(&r.detail),
            BTreeSet::from([name.to_owned()]),
            "{name}: {}",
            r.detail
        );
        assert!(!r.detail.contains("placeholder"), "{}", r.detail);
    }
}

#[test]
fn deploy_check_refuses_a_bundle_whose_digest_does_not_hold() {
    let dir = scratch("deploy-check-digest");
    let root = repo_root();
    let text = std::fs::read_to_string(root.join("bundles/workstation.split.yaml")).unwrap();
    let r = check_text(&dir, &text.replace("chat-token", "chat-key")).unwrap_err();
    assert_eq!(r.code, "bundle_invalid", "{}", r.detail);
    assert!(
        r.detail.contains("`digest` is not the digest"),
        "{}",
        r.detail
    );
    let r = check_text(&dir, "format: agenticspec-binding/1\n").unwrap_err();
    assert_eq!(r.code, "bundle_invalid");
}

#[test]
fn a_runtime_is_pinned_by_image_and_digest_and_refused_without_either() {
    let dir = scratch("deploy-runtime-form");
    let file = dir.join("b.yaml");
    let d = "sha256:0000000000000000000000000000000000000000000000000000000000000002";
    std::fs::write(
        &file,
        format!(
            "format: agenticspec-binding/1\nbinding: odd\nruntimes:\n  core: {{image: registry.example/core:1.2}}\n  agent-runtime: {{digest: \"{d}\"}}\n  extra: {{image: \"registry.example/x@{d}\", digest: \"{d}\"}}\n"
        ),
    )
    .unwrap();
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    for needle in [
        "runtimes: core: `digest:`",
        "runtimes: core: `image:`",
        "runtimes: agent-runtime: `image:`",
        "runtimes: extra: `image:`",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
    assert!(!r.detail.contains("runtimes: extra: `digest:`"));
    assert!(binding::is_image("localhost:5000/team/core"));
    assert!(!binding::is_image("localhost:5000/team/core:latest"));
}

/// The variants of `agentic.architecture.PartKind`, in snake case.
fn declared_part_kinds() -> BTreeSet<String> {
    let doc = load_yaml(&repo_root().join("blocks/9-composition/architecture.yaml")).unwrap();
    let t = doc["types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "agentic.architecture.PartKind")
        .unwrap();
    t["variants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_lowercase())
        .collect()
}

#[test]
fn every_part_kind_derives_its_ports_and_sim_check_refuses_any_other_block() {
    use agenticspec_sim::contract::derive::{PART_KINDS, block_ports};
    let table: BTreeSet<String> = PART_KINDS.iter().map(|k| (*k).to_owned()).collect();
    assert_eq!(table, declared_part_kinds(), "the table is the part kinds");
    let ports = load_yaml(&repo_root().join("blocks/ports.json")).unwrap();
    let known: BTreeSet<&str> = ports["ports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    for kind in PART_KINDS {
        let part = json!({ "block": kind });
        let doc = json!({ "parts": { "x": part } });
        let used = block_ports(&doc, "x", &part).unwrap_or_else(|e| panic!("{kind}: {e}"));
        for p in used {
            assert!(
                known.contains(p),
                "{kind}: port {p} is not in blocks/ports.json"
            );
        }
    }
    let dir = scratch("sim-check-blocks");
    for odd in ["resource", "cache"] {
        let part = json!({ "block": odd });
        assert!(block_ports(&json!({ "parts": { "x": part } }), "x", &part).is_err());
        let file = dir.join(format!("{odd}.yaml"));
        std::fs::write(
            &file,
            format!(
                "format: agenticspec-architecture/1\narchitecture: odd\nparts:\n  owner: {{block: principal, kind: human}}\n  agent: {{block: agent, harness: coding}}\n  model: {{block: model, route: frontier}}\n  x: {{block: {odd}}}\nlinks:\n  - {{from: owner, to: agent, carries: prompt}}\n  - {{from: agent, to: model, carries: model_call}}\n  - {{from: agent, to: x, carries: message}}\n"
            ),
        )
        .unwrap();
        let r = cli::check(&file).unwrap_err();
        assert_eq!(r.code, "architecture_refused", "{odd}: {}", r.detail);
        assert!(
            r.detail
                .contains(&format!("block `{odd}` is not a part kind")),
            "{odd}: {}",
            r.detail
        );
    }
}

/// A committed fixture bundle of `bundles/`, read back.
fn fixture(file: &str) -> Value {
    load_yaml(&repo_root().join("bundles").join(file)).unwrap()
}

fn unit_of<'a>(v: &'a Value, name: &str) -> &'a Value {
    v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["name"] == name)
        .unwrap()
}

/// A binding written to a scratch file that refines the workstation's refusing base (every
/// destination declared, the tree mounted), with `extra` after the header.
fn workstation_binding(dir: &Path, extra: &str) -> PathBuf {
    let base = cases().join("egress/workstation.yaml");
    let file = dir.join("b.yaml");
    std::fs::write(
        &file,
        format!(
            "format: agenticspec-binding/1\nbinding: odd\narchitecture: workstation\nrefines: {}\n{extra}",
            base.display()
        ),
    )
    .unwrap();
    file
}

#[test]
fn each_export_of_an_ingress_carries_its_unit_part_port_and_bind() {
    let v = fixture("workstation.split.yaml");
    let entries = v["ingress"].as_array().unwrap();
    let find = |ingress: &str, protocol: &str| {
        entries
            .iter()
            .find(|e| e["ingress"] == ingress && e["protocol"] == protocol)
            .unwrap_or_else(|| panic!("no {ingress} {protocol}"))
    };
    assert_eq!(
        find("operator", "http"),
        &json!({ "ingress": "operator", "protocol": "http", "audience": "control.operator", "authenticated": true, "unit": "core", "part": "control", "port": 8080, "bind": "all" })
    );
    assert_eq!(find("metrics", "metrics_scrape")["port"], 9090);
    assert_eq!(find("metrics", "metrics_scrape")["authenticated"], false);
    let tool = find("operator", "tool");
    assert!(
        tool.get("port").is_none(),
        "no port, no entry point: {tool}"
    );
    assert_eq!(
        tool["bind"], "loopback",
        "loopback unless the binding says all"
    );
    for e in fixture("workstation.local.yaml")["ingress"]
        .as_array()
        .unwrap()
    {
        assert!(e.get("port").is_none(), "{e}");
        assert_eq!(e["unit"], "core");
    }
}

#[test]
fn an_export_served_by_a_part_is_in_that_parts_unit_and_its_address_is_carried() {
    let dir = scratch("deploy-export-part");
    let agents = repo_root().join("bundles/agents");
    let file = workstation_binding(
        &dir,
        &format!(
            "transports: {{model_call: http, order: a2a, question: a2a, claim: http}}\nparts:\n  gateway: {{state: service, egress: [model]}}\n  builds: {{state: service}}\n  operator: {{agent: {}}}\n  worker: {{agent: {}}}\ningress:\n  operator:\n    exports:\n      tool: {{part: operator, port: 7000, address: tools.example}}\npacking: {{strategy: by_part}}\n",
            agents.join("operator.yaml").display(),
            agents.join("coder.yaml").display()
        ),
    );
    let out = bundle_with(&arch("workstation"), &file, None, None)
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    let tool = v["ingress"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["protocol"] == "tool")
        .unwrap();
    assert_eq!(tool["unit"], "operator");
    assert_eq!(tool["part"], "operator");
    assert_eq!(tool["port"], 7000);
    assert_eq!(tool["address"], "tools.example");
}

#[test]
fn a_check_carries_the_port_the_binding_gives_it() {
    let v = fixture("workstation.split.yaml");
    let core = unit_of(&v, "core");
    assert_eq!(
        core["checks"]["ready"],
        json!({ "kind": "http", "path": "/ready", "port": 8080 })
    );
    assert_eq!(unit_of(&v, "gateway")["checks"]["alive"]["port"], 8081);
    let worker = unit_of(&v, "worker");
    assert!(worker["checks"]["startup"].get("port").is_none());
}

#[test]
fn a_volume_carries_its_mount_and_a_relative_mount_is_refused_without_its_text() {
    let v = fixture("workstation.local.yaml");
    assert_eq!(
        unit_of(&v, "core")["claims"]["volumes"][0]["mount"],
        "/workspace"
    );
    let dir = scratch("deploy-mount-relative");
    let file = workstation_binding(
        &dir,
        "parts:\n  tree: {volumes: [{name: tree, mount: work/tree}]}\n",
    );
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "not_admitted", "{}", r.detail);
    assert_eq!(
        refusal_names(&r.detail),
        BTreeSet::from(["volume_mount_missing".to_owned()]),
        "{}",
        r.detail
    );
    assert!(r.detail.contains("not an absolute path"), "{}", r.detail);
    assert!(!r.detail.contains("work/tree"), "{}", r.detail);
}

#[test]
fn a_local_image_is_carried_as_the_binding_gave_it() {
    let v = fixture("scheduled-loops.jobs.yaml");
    assert_eq!(
        v["runtimes"]["agenticspec-core:dev"],
        json!({ "name": "core", "image": "agenticspec-core", "tag": "dev" })
    );
    assert_eq!(
        unit_of(&v, "core")["code"],
        json!({ "runtime": "agenticspec-core:dev" })
    );
    let dir = scratch("deploy-local-image");
    let id = "sha256:00000000000000000000000000000000000000000000000000000000000000ab";
    let file = workstation_binding(
        &dir,
        &format!(
            "target: {{images: local}}\nruntimes:\n  core: {{image: agenticspec-core, digest: \"{id}\"}}\n"
        ),
    );
    let out = bundle_with(&arch("workstation"), &file, None, None)
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    assert_eq!(
        v["runtimes"][id],
        json!({ "name": "core", "image": "agenticspec-core", "digest": id })
    );
    assert_eq!(unit_of(&v, "core")["code"]["runtime"], id);
    // From a registry the image stays pinned by its digest, and a tag is refused.
    let file = workstation_binding(
        &dir,
        "runtimes:\n  core: {image: agenticspec-core, tag: dev}\n",
    );
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    for needle in [
        "runtimes: core: `digest:`",
        "runtimes: core: `tag:` goes with",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
}

#[test]
fn any_part_may_be_external_with_its_kind_and_reach_and_egress_names_what_the_target_resolves() {
    let dir = scratch("deploy-external-part");
    let file = workstation_binding(
        &dir,
        "parts:\n  operator: {egress: [gateway, owner, worker, {to: metrics-sink, address: {namespace: observability, labels: {app: sink}}}, {to: chat-relay, address: \"10.0.0.0/8\"}]}\npacking:\n  units:\n    - {name: core, kind: core, parts: [box, chat, gateway, operator, tree, worker]}\n  external: [model, owner, {part: builds, reach: {name: build-farm}}]\n",
    );
    let out = bundle_with(&arch("workstation"), &file, None, None)
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
    let v = parse_yaml(out["bundle"].as_str().unwrap()).unwrap();
    assert_eq!(
        v["packing"]["document"]["external"],
        json!([
            { "part": "model", "kind": "model", "reach": { "name": "model" } },
            { "part": "owner", "kind": "principal", "reach": { "name": "owner" } },
            { "part": "builds", "kind": "pool", "reach": { "name": "build-farm" } },
        ]),
        "in the order written"
    );
    let egress = unit_of(&v, "core")["egress"].as_array().unwrap().clone();
    let to = |t: &str| egress.iter().find(|e| e["to"] == t).unwrap().clone();
    let builds = to("builds");
    assert_eq!(builds["external"], true);
    assert_eq!(builds["name"], "build-farm");
    assert!(builds.get("address").is_none());
    assert_eq!(
        to("model")["name"],
        "model",
        "the default reach is the part's name"
    );
    assert_eq!(
        to("metrics-sink")["address"],
        json!({ "namespace": "observability", "labels": { "app": "sink" } })
    );
    assert_eq!(to("chat-relay")["address"], "10.0.0.0/8");
    assert_eq!(to("chat-relay")["name"], "chat-relay");
    assert!(to("chat-relay").get("unresolved").is_none());
    // Read back, the packing with its reach resolves to the same units.
    let accepts = repo_root().join("bundles/cloudspec-accepts.yaml");
    let written = dir.join("bundle.yaml");
    std::fs::write(&written, out["bundle"].as_str().unwrap()).unwrap();
    cli::deploy_check(&written, Some(&accepts))
        .unwrap_or_else(|r| panic!("{}: {}", r.code, r.detail));
}

#[test]
fn a_crossing_is_named_by_its_unit_and_an_unresolved_destination_says_so() {
    let v = fixture("workstation.split.yaml");
    let worker = unit_of(&v, "worker");
    let to = |t: &str| {
        worker["egress"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["to"] == t)
            .unwrap()
            .clone()
    };
    assert_eq!(to("gateway")["name"], "gateway");
    assert_eq!(to("gateway")["unit"], "gateway");
    assert_eq!(
        to("docs-mirror"),
        json!({ "from": "worker", "to": "docs-mirror", "named": true, "name": "docs-mirror", "unresolved": true })
    );
    let operator = unit_of(&v, "operator");
    let owner = operator["egress"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["to"] == "owner")
        .unwrap();
    assert_eq!(owner["name"], "owner");
    assert_eq!(owner["external"], true);
}

#[test]
fn the_new_binding_keys_of_the_wrong_form_are_refused_before_admission() {
    let dir = scratch("deploy-served-form");
    let file = workstation_binding(
        &dir,
        concat!(
            "target: {images: cache, network_policy: strict}\n",
            "parts:\n",
            "  operator: {egress: [{to: relay, address: \"relay\"}, {to: owner, address: \"10.0.0.1:80\"}, {to: sink, address: {zone: a}}]}\n",
            "  worker: {network: open, volumes: [{name: box, mount: /w}, {mount: /x}]}\n",
            "checks:\n",
            "  worker: {ready: {port: 80}, readiness: {}}\n",
            "  gateway: {alive: {port: 0}}\n",
            "  nowhere: {ready: {port: 80}}\n",
            "ingress:\n",
            "  operator:\n",
            "    exports:\n",
            "      http: {port: 70000, bind: everywhere}\n",
            "      cli: {port: 9000}\n",
            "  public: {exports: {http: {port: 80}}}\n",
            "packing:\n",
            "  strategy: by_part\n",
            "  parts:\n",
            "    gateway: {reach: {name: gw}}\n",
            "    model: {reach: {host: models}}\n",
        ),
    );
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    for needle in [
        "target: images is one of registry, local",
        "target: network_policy is one of enforced, not_enforced",
        "egress[0]: address is a CIDR block",
        "egress[1]: address is for a destination outside the document",
        "egress[2]: address: unknown key `zone`",
        "parts: worker: network is `isolated`",
        "volumes[0]: name names a workspace part",
        "volumes[1] needs a `name:`",
        "checks: worker: `readiness` is not one of",
        "checks: gateway: alive: port is a whole number",
        "exports: http: port is a whole number",
        "exports: http: bind is one of loopback, all",
        "packing: parts: model: `reach` is {address: <text>} or {name: <text>}",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
    // Those that need the packing and the contract: the form above has to pass first.
    let file = workstation_binding(
        &dir,
        concat!(
            "checks:\n",
            "  core: {startup: {port: 80}}\n",
            "  nowhere: {ready: {port: 80}}\n",
            "ingress:\n",
            "  operator: {exports: {cli: {port: 9000}, http: {part: model}}}\n",
            "  public: {exports: {http: {port: 80}}}\n",
            "packing:\n",
            "  parts:\n",
            "    gateway: {reach: {name: gw}}\n",
        ),
    );
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    assert!(
        r.detail
            .contains("packing: parts: gateway: `reach` is for an external part"),
        "{}",
        r.detail
    );
    let file = workstation_binding(
        &dir,
        concat!(
            "checks:\n",
            "  core: {startup: {port: 80}}\n",
            "  nowhere: {ready: {port: 80}}\n",
            "ingress:\n",
            "  operator: {exports: {cli: {port: 9000}, http: {part: model}}}\n",
            "  public: {exports: {http: {port: 80}}}\n",
        ),
    );
    let r = bundle_with(&arch("workstation"), &file, None, None).unwrap_err();
    assert_eq!(r.code, "binding_invalid", "{}", r.detail);
    for needle in [
        "checks: core: a core unit has no startup check",
        "checks: `nowhere` is not a unit of the packing",
        "ingress: operator: exports: `cli` is not an export of the ingress",
        "ingress: operator: exports: http: part model is in no unit",
        "ingress: `public` is not an ingress of the contract",
    ] {
        assert!(r.detail.contains(needle), "{needle} in {}", r.detail);
    }
}

#[test]
fn an_address_is_a_cidr_block_or_a_host_and_port() {
    for ok in [
        "10.0.0.0/8",
        "fd00::/8",
        "chat-relay.example:443",
        "10.0.0.1:80",
        "[fd00::1]:8443",
    ] {
        assert!(binding::is_address(ok), "{ok}");
    }
    for refused in [
        "relay",
        "10.0.0.0/33",
        "relay:0",
        "relay:99999",
        "https://relay.example",
        ":80",
        "-relay:80",
        "[relay]:80",
    ] {
        assert!(!binding::is_address(refused), "{refused}");
    }
}

#[test]
fn deploy_check_refuses_a_volume_without_a_mount_an_unpinned_local_image_and_unmet_network_isolation()
 {
    let dir = scratch("deploy-check-served");
    let cases: [(&str, Edit); 4] = [
        (
            "volume_mount_missing",
            Box::new(|v| {
                v["binding"]["document"]["parts"]["worker"]
                    .as_object_mut()
                    .unwrap()
                    .remove("volumes");
            }),
        ),
        (
            "volume_mount_missing",
            Box::new(|v| {
                unit_mut(v, "worker")["claims"]["volumes"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("mount");
            }),
        ),
        (
            "image_unpinned",
            Box::new(|v| {
                let b = &mut v["binding"]["document"];
                b["target"]["images"] = json!("local");
                b["runtimes"]["core"] = json!({ "image": "agenticspec-core" });
            }),
        ),
        (
            "network_isolation_unmet",
            Box::new(|v| {
                v["binding"]["document"]["target"]["network_policy"] = json!("not_enforced");
            }),
        ),
    ];
    for (name, edit) in cases {
        let r = check_text(&dir, &tampered(edit)).unwrap_err();
        assert_eq!(r.code, "not_admitted", "{name}: {}", r.detail);
        assert_eq!(
            refusal_names(&r.detail),
            BTreeSet::from([name.to_owned()]),
            "{name}: {}",
            r.detail
        );
    }
}
