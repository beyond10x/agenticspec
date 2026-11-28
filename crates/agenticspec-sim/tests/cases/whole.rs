//! The KPIs of an architecture as a whole: one case per KPI over a document written for it, the
//! acceptance cases, and the numbers of every architecture under `architectures/`.

use super::common::{repo_root, yaml};
use agenticspec_sim::cli;
use agenticspec_sim::whole::{ARCHITECTURE_KPIS, measure, single_point_parts};
use serde_json::Value;

fn kpi(arch: &Value, id: &str) -> f64 {
    measure(arch)
        .into_iter()
        .find(|(k, _)| *k == id)
        .unwrap_or_else(|| panic!("{id} is measured"))
        .1
}

fn doc(parts: &str, links: &str, rest: &str) -> Value {
    yaml(&format!(
        "format: agenticspec-architecture/1\narchitecture: case\nparts:\n{parts}\nlinks:\n{links}\n{rest}"
    ))
}

/// Two agents behind one gateway to one model, the shape the weak-point cases change.
const GATED_PARTS: &str = "
  a:     { block: agent, harness: coding }
  b:     { block: agent, harness: coding }
  gw:    { block: gateway }
  model: { block: model, route: frontier }";
const GATED_LINKS: &str = "
  - { from: a,  to: gw,    carries: model_call }
  - { from: b,  to: gw,    carries: model_call }
  - { from: gw, to: model, carries: model_call }";

#[test]
fn every_kpi_is_measured_once_in_the_order_of_the_list() {
    let ids: Vec<&str> = measure(&doc(GATED_PARTS, GATED_LINKS, ""))
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    let listed: Vec<&str> = ARCHITECTURE_KPIS.iter().map(|m| m.id).collect();
    assert_eq!(ids, listed);
}

#[test]
fn agents_total_sums_the_copies_of_the_agent_parts() {
    let a = doc(
        "
  a:     { block: agent, count: 3 }
  b:     { block: agent, count: { value: 5, vary: [1, 5, 9] } }
  c:     { block: agent }
  model: { block: model }",
        "  - { from: a, to: model, carries: model_call }",
        "",
    );
    assert_eq!(kpi(&a, "agents_total"), 9.0);
}

#[test]
fn principals_total_counts_the_principal_parts() {
    let a = doc(
        "
  owner:  { block: principal, kind: human }
  second: { block: principal, kind: human }
  a:      { block: agent }",
        "  - { from: owner, to: a, carries: prompt }",
        "",
    );
    assert_eq!(kpi(&a, "principals_total"), 2.0);
}

#[test]
fn human_touchpoints_count_questions_and_proposals_to_a_principal() {
    let a = doc(
        "
  owner: { block: principal, kind: human }
  a:     { block: agent }
  b:     { block: agent }",
        "
  - { from: owner, to: a,     carries: prompt }
  - { from: a,     to: owner, carries: question }
  - { from: b,     to: owner, carries: proposal }
  - { from: b,     to: owner, carries: report }
  - { from: b,     to: a,     carries: question }",
        "",
    );
    assert_eq!(kpi(&a, "human_touchpoints"), 2.0);
}

#[test]
fn autonomy_depth_is_the_longest_chain_of_agent_links() {
    let a = doc(
        "
  owner: { block: principal, kind: human }
  lead:  { block: agent }
  mid:   { block: agent }
  leaf:  { block: agent }",
        "
  - { from: owner, to: lead,  carries: order }
  - { from: lead,  to: mid,   carries: order }
  - { from: mid,   to: leaf,  carries: order }
  - { from: leaf,  to: mid,   carries: question }
  - { from: leaf,  to: owner, carries: question }
  - { from: mid,   to: leaf,  carries: handoff }",
        "",
    );
    assert_eq!(kpi(&a, "autonomy_depth"), 2.0);
    let solo = doc("  a: { block: agent }", "  []", "");
    assert_eq!(kpi(&solo, "autonomy_depth"), 0.0);
}

#[test]
fn model_call_paths_count_every_path_from_an_agent_to_a_model() {
    let a = doc(
        "
  a:     { block: agent }
  gw1:   { block: gateway }
  gw2:   { block: gateway }
  m1:    { block: model }
  m2:    { block: model }",
        "
  - { from: a,   to: gw1, carries: model_call }
  - { from: a,   to: gw2, carries: model_call }
  - { from: gw1, to: m1,  carries: model_call }
  - { from: gw1, to: m2,  carries: model_call }
  - { from: gw2, to: m1,  carries: model_call }",
        "",
    );
    assert_eq!(kpi(&a, "model_call_paths"), 3.0);
}

#[test]
fn a_direct_model_call_is_ungated_until_a_gateway_is_on_its_path() {
    let direct = doc(
        "
  a:     { block: agent }
  model: { block: model }",
        "  - { from: a, to: model, carries: model_call }",
        "",
    );
    assert_eq!(kpi(&direct, "ungated_model_calls"), 1.0);
    let gated = doc(
        "
  a:     { block: agent }
  gw:    { block: gateway }
  model: { block: model }",
        "
  - { from: a,  to: gw,    carries: model_call }
  - { from: gw, to: model, carries: model_call }",
        "",
    );
    assert_eq!(kpi(&gated, "ungated_model_calls"), 0.0);
}

#[test]
fn a_lone_gateway_of_two_agents_is_a_single_point_and_a_parallel_one_lifts_it() {
    let lone = doc(GATED_PARTS, GATED_LINKS, "");
    assert_eq!(single_point_parts(&lone), vec!["gw", "model"]);
    assert_eq!(kpi(&lone, "single_points"), 2.0);
    let parallel = doc(
        &format!("{GATED_PARTS}\n  gw2:   {{ block: gateway }}"),
        &format!(
            "{GATED_LINKS}
  - {{ from: a,   to: gw2,   carries: model_call }}
  - {{ from: b,   to: gw2,   carries: model_call }}
  - {{ from: gw2, to: model, carries: model_call }}"
        ),
        "",
    );
    assert_eq!(single_point_parts(&parallel), vec!["model"]);
    assert_eq!(kpi(&parallel, "single_points"), 1.0);
}

#[test]
fn an_agent_without_a_model_path_makes_no_single_point() {
    let a = doc(
        "
  a:     { block: agent }
  b:     { block: agent }
  model: { block: model }",
        "  - { from: a, to: model, carries: model_call }",
        "",
    );
    assert_eq!(single_point_parts(&a), vec!["model"]);
}

#[test]
fn border_parts_count_the_border_names_that_are_parts() {
    let a = doc(
        "
  a: { block: agent }
  b: { block: agent }",
        "  []",
        "border: [a, b, a, ghost]",
    );
    assert_eq!(kpi(&a, "border_parts"), 2.0);
}

#[test]
fn egress_targets_count_each_target_of_the_sandboxes_once() {
    let a = doc(
        "
  gw:   { block: gateway }
  repo: { block: tool }
  one:  { block: sandbox, egress: [gw, repo] }
  two:  { block: sandbox, egress: gw }
  mdl:  { block: model }
  agt:  { block: agent, egress: [mdl] }",
        "  []",
        "",
    );
    assert_eq!(kpi(&a, "egress_targets"), 2.0);
}

#[test]
fn shared_workspaces_count_every_sharing_but_exclusive() {
    let a = doc(
        "
  mine:  { block: workspace, sharing: exclusive }
  ours:  { block: workspace, sharing: shared_read_only }
  board: { block: workspace, sharing: { value: shared, vary: [exclusive, shared] } }
  bare:  { block: workspace }",
        "  []",
        "",
    );
    assert_eq!(kpi(&a, "shared_workspaces"), 3.0);
}

#[test]
fn capacity_ratio_is_the_largest_claim_on_a_pool() {
    let a = doc(
        "
  w:     { block: agent, count: { value: 12, vary: [6, 12] } }
  v:     { block: agent, count: 4 }
  slots: { block: pool, capacity: { value: 8, vary: [8, 16] } }
  gpus:  { block: pool, capacity: 2 }
  open:  { block: pool }",
        "
  - { from: w, to: slots, carries: claim }
  - { from: v, to: slots, carries: claim }
  - { from: v, to: gpus,  carries: claim }
  - { from: v, to: open,  carries: claim }",
        "",
    );
    assert_eq!(kpi(&a, "capacity_ratio"), 2.0);
    let none = doc("  a: { block: agent }", "  []", "");
    assert_eq!(kpi(&none, "capacity_ratio"), 0.0);
}

/// The KPIs of each architecture under `architectures/`, in the order of the list.
const ARCHITECTURES: &[(&str, [f64; 11])] = &[
    (
        "author-reviewer",
        [10.0, 1.0, 1.0, 1.0, 3.0, 0.0, 2.0, 1.0, 1.0, 0.0, 0.5],
    ),
    (
        "handoff",
        [2.0, 1.0, 2.0, 0.0, 2.0, 2.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    ),
    (
        "orchestrator-workers",
        [9.0, 1.0, 1.0, 1.0, 2.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.5],
    ),
    (
        "parallel-unaware",
        [2.0, 1.0, 2.0, 0.0, 2.0, 2.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    ),
    (
        "planner-executor",
        [9.0, 1.0, 1.0, 0.0, 2.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.5],
    ),
    (
        "scheduled-loops",
        [3.0, 1.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.375],
    ),
    (
        "scope-partition",
        [4.0, 1.0, 1.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.5],
    ),
    (
        "shared-board",
        [8.0, 1.0, 1.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.5],
    ),
    (
        "solo",
        [1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    ),
    (
        "workstation",
        [13.0, 1.0, 1.0, 1.0, 2.0, 0.0, 2.0, 1.0, 1.0, 0.0, 0.75],
    ),
];

#[test]
fn every_architecture_has_its_committed_numbers() {
    let dir = repo_root().join("architectures");
    let mut files: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter_map(|f| f.strip_suffix(".yaml").map(str::to_string))
        .collect();
    files.sort();
    let named: Vec<&str> = ARCHITECTURES.iter().map(|(n, _)| *n).collect();
    assert_eq!(files, named, "one row per architecture");
    for (name, want) in ARCHITECTURES {
        let text = std::fs::read_to_string(dir.join(format!("{name}.yaml"))).unwrap();
        let got: Vec<f64> = measure(&yaml(&text)).into_iter().map(|(_, v)| v).collect();
        assert_eq!(got, want.to_vec(), "{name}");
    }
}

#[test]
fn check_reports_the_kpis_of_the_workstation() {
    let r = cli::check(&repo_root().join("architectures/workstation.yaml")).unwrap();
    let k = &r["kpis"];
    assert_eq!(k["agents_total"], 13, "{r}");
    assert_eq!(k["principals_total"], 1, "{r}");
    assert_eq!(k["border_parts"], 1, "{r}");
    assert_eq!(k["ungated_model_calls"], 0, "{r}");
    assert_eq!(k["capacity_ratio"], 0.75, "{r}");
    assert_eq!(r["single_points"], serde_json::json!(["gateway", "model"]));
    assert_eq!(k.as_object().unwrap().len(), ARCHITECTURE_KPIS.len(), "{r}");
}

#[test]
fn the_catalogue_lists_every_kpi_with_block_architecture_and_its_summary() {
    let text = std::fs::read_to_string(repo_root().join("kpis/catalogue.yaml")).unwrap();
    let cat = yaml(&text);
    for m in ARCHITECTURE_KPIS {
        let entry = cat["kpis"]
            .as_array()
            .unwrap()
            .iter()
            .find(|k| k["id"] == m.id)
            .unwrap_or_else(|| panic!("{} is in the catalogue", m.id));
        assert_eq!(entry["block"], "architecture", "{}", m.id);
        assert_eq!(entry["source"], "design", "{}", m.id);
        assert_eq!(entry["kind"], m.kind, "{}", m.id);
        assert_eq!(entry["summary"], m.what, "{}", m.id);
        assert_eq!(entry["gate"], false, "{}", m.id);
    }
}
