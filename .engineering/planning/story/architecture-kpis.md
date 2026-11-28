---
format: aep.planning-md/3
id: story:architecture-kpis
kind: story
status: implemented
title: KPIs of an architecture as a whole, read from its document
summary: Size, autonomy, exposure, weak points and fixed limits of an architecture, computed by agenticspec sim check and shown on every architecture page and catalog entry
relations:
- decomposes: epic:simulator-and-search
scope:
- confidence: cited
  path: crates/agenticspec-docs
- confidence: cited
  path: crates/agenticspec-sim/src/check.rs
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: docs/architectures
- confidence: inferred
  path: docs/catalog
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: inferred
  path: website/src/generated
- confidence: inferred
  path: website/static/catalog
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:12Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:12Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:12Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An architecture is measured as a whole, from its document alone, before any simulation. Today the
catalogue's 12 KPIs of the `architecture` block (`kpis/catalogue.yaml`, `block: architecture`) are
structural refusals (`unknown_link_parts`, `agent_without_model_path`, ...) and two coordination
counts; none says what kind of system the architecture is. This story adds KPIs of the whole
specification: its size, its autonomy, its exposure, its weak points and its fixed limits. They are
computed by `agenticspec sim check`, written to its JSON output, listed in the catalogue, and shown
on every architecture page and catalog entry of the docs, so two architectures can be compared on
them without running a search.

## The KPIs

Each is a pure function of the architecture document (`agenticspec-architecture/1`), with
`source: design` in the catalogue like the existing architecture KPIs:

| id | kind | what it counts |
|---|---|---|
| `agents_total` | completeness | agent copies: the sum of `count` over parts with `block: agent` |
| `principals_total` | completeness | parts with `block: principal` |
| `human_touchpoints` | quality | links that end at a principal and carry `question` or a decision |
| `autonomy_depth` | quality | the longest chain of agent-to-agent links (`order`, `question`) with no principal on it |
| `model_call_paths` | completeness | distinct agent-to-model paths over `model_call` links |
| `ungated_model_calls` | quality | agent-to-model paths over `model_call` that pass no gateway part |
| `single_points` | quality | parts whose removal leaves some agent with no path to a model (cut vertices of the `model_call` graph) |
| `border_parts` | quality | parts named in `border`, the parts that take input from outside |
| `egress_targets` | quality | distinct targets named in `egress` over all sandbox parts |
| `shared_workspaces` | quality | workspace parts with `sharing` other than `exclusive` |
| `capacity_ratio` | quality | agent copies that claim a pool divided by the pool's `capacity`, the largest over pools |
| `contract_ports` | completeness | ports in the architecture's contract; added when `epic:architecture-contract` derives contracts, absent before |

Open point, settled here: the exact link kinds that count as a decision for
`human_touchpoints`, read from the `carries` values the 10 architectures under `architectures/`
use. Open point: whether `capacity_ratio` reads marked values at their built value (taken: yes,
as `sim.rs` does).

## Scope

- `crates/agenticspec-sim`: the measures in a new module beside `check.rs`, reported by `check`;
  one test per KPI over a small document written for it, and one over each of the 10
  architectures that compares with numbers committed in the test.
- `kpis/catalogue.yaml`: one entry per KPI with `block: architecture`, `source: design`, appended
  as text (a round trip through `yq` reformats the whole file); `scripts/check-kpis.sh` holds
  them.
- `crates/agenticspec-docs`: a section "The architecture as a whole" on each architecture page and
  each catalog entry, and the KPIs as sortable columns of the catalog index; generated pages under
  `docs/` and `website/` through `task docs:generate` and `task catalog:generate`.
- Tests: each ends within 60 s, the test step within 120 s; no simulation runs in these tests.

## Shared surfaces

- `crates/agenticspec-sim`: `story:guards-and-limits`, `story:architecture-patches`,
  `story:studio-evaluation` and `story:studio-state-space` also change it; this story adds a
  module of its own and the `check` report and touches neither `sim.rs` nor `search.rs`.
- `kpis/catalogue.yaml`: `story:guards-and-limits` also appends to it.
- `crates/agenticspec-docs` and the generated pages: the docs story of
  `epic:architecture-contract` adds a Contract section to the same pages.

## Acceptance

- Given `architectures/workstation.yaml`, `agenticspec sim check` reports `agents_total` 13
  (operator 1, worker 12), `principals_total` 1, `border_parts` 1 and `ungated_model_calls` 0,
  where before its report held none of these KPIs.
- Given a document where an agent links to a model over `model_call` with no gateway on the path,
  `ungated_model_calls` is 1; given the same document with a gateway inserted on that path, it is
  0.
- Given a document where one gateway part is the only path from two agents to a model, that
  gateway is counted in `single_points`; given a second gateway on a parallel path, it is not.
- Given the catalogue after this story, `task check:kpis` passes with every KPI of the table above
  except `contract_ports` listed under `block: architecture`.
- Given the generated docs, the page of `architectures/workstation.yaml` and its catalog entry show
  the section "The architecture as a whole" with the numbers `agenticspec sim check` reports, and
  `task check:docs` passes.

## Not in this story

KPIs that need a simulation (they exist per block); guard coverage, which waits for guard parts
(`story:guards-and-limits`); `contract_ports` before contracts exist; weights of these KPIs in an
experiment's fitness.
