---
format: aep.planning-md/3
id: story:routing-invariants
kind: story
status: draft
title: Check a contract's routing invariants against a simulated trace
summary: agenticspec sim trace records a port-level trace of one run and agenticspec sim contract --trace checks every routing invariant on it, naming the link and event of a violation.
relations:
- decomposes: epic:architecture-contract
- depends_on: story:contract-conformance
- depends_on: story:contract-docs
- depends_on: story:contract-derivation
- depends_on: story:port-derivation
- depends_on: story:block-bindings
- depends_on: story:architecture-kpis
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-sim/src/cli.rs
- confidence: cited
  path: crates/agenticspec-sim/src/contract
- confidence: cited
  path: crates/agenticspec-sim/src/lib.rs
- confidence: cited
  path: crates/agenticspec-sim/src/sim
- confidence: cited
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/tests
- confidence: cited
  path: crates/agenticspec/src/main.rs
- confidence: cited
  path: docs/contract.md
- confidence: cited
  path: spec/cli.yaml
- confidence: cited
  path: spec/domains/cli.yaml
- confidence: cited
  path: website/static/catalog
revision: 8
---
## Outcome

The routing invariants of a contract are checked against a trace. The simulator records, in one
simulated run, a trace in port terms (each use of a port by a part instance, with its start, its
end and the use it nests in): `agenticspec sim trace` writes it. `agenticspec sim contract
<architecture.yaml> --trace <trace.json>` says whether every invariant of the contract holds on
it, naming the link and the event of the first violation of each. The same check later reads a
runtime's trace (next epic).

## Scope

- `crates/agenticspec-sim/src/sim.rs`, `src/sim/swarm.rs`: with `record`, keep the port uses the
  simulator already models (model calls through a gateway, builds in a pool, questions to a
  principal or a delegate, channel delivery, board posts, trigger firings) as trace events. A run
  without `record`, every search run, gives the same result as before.
- `crates/agenticspec-sim/src/contract/trace.rs`: the trace format `agenticspec-trace/1` (events:
  index, time, part, instance, port, begin or end, the index of the use it nests in) and the
  evaluator. The evaluator takes the typed `Predicate` that `story:contract-derivation`'s parser
  (`src/contract/predicate.rs`) returns for each invariant and evaluates that form on the trace;
  it does not parse predicate text. A predicate form the evaluator cannot evaluate is reported as
  not checked, not as held.
- `agenticspec sim trace <architecture.yaml> --profile <profile.yaml> --seed <n> --out <trace.json>`
  (one run at the architecture's built values; under 5 s for every architecture under
  `architectures/`) and `--trace` on `agenticspec sim contract`. Both are declared in
  `spec/cli.yaml`, their input and result types in `spec/domains/cli.yaml`; `task generate`
  regenerates `crates/agenticspec-cli-contract/` (held by `task check:generated`);
  `crates/agenticspec/src/main.rs` dispatches them to `crates/agenticspec-sim/src/cli.rs`;
  `src/lib.rs` lists the module.
- `crates/agenticspec-sim/tests/main.rs`, `tests/cases/`: a hand-written trace that holds and one
  that breaks each kind of invariant; each test under 5 s.
- Changing `sim.rs` changes the digest of every committed catalog result (`src/sim.rs` and
  `src/sim/` count; `src/contract/` is excluded since `story:port-derivation`). `task
  catalog:generate` runs in this story (`slot --timeout 30m`, as its experiments' budgets need),
  each `website/static/catalog/<entry>/result.json` stays byte for byte the same, and only
  `sources.sha256` changes; a changed `result.json` is a defect of this story. Code the recorder
  in `sim.rs` needs lives under `src/sim/`, not `src/contract/`, so the exclusion stays true.
- `docs/contract.md` (the trace format and the check), `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed unless it names its time.

## Shared surfaces

- `crates/agenticspec-sim/src/contract/`, `src/cli.rs`, `src/lib.rs`, `tests/`:
  `story:port-derivation`, `story:contract-derivation`, `story:contract-conformance` (all come
  first). Outside this epic, `story:architecture-kpis` changes `crates/agenticspec-sim/src/check.rs`
  and adds tests.
- `spec/cli.yaml`, `spec/domains/cli.yaml`, `crates/agenticspec-cli-contract/`,
  `crates/agenticspec/src/main.rs`: `story:port-derivation`, `story:contract-derivation`,
  `story:contract-conformance` (all come first). Outside this epic,
  `story:architecture-patches` records all of `crates/agenticspec-sim` and
  `crates/agenticspec/src` as its scope; `aep plan artifact waves` keeps them in different waves,
  and whichever lands second rebases on the other.
- `crates/agenticspec-sim/src/sim.rs`, `crates/agenticspec-docs/src/results.rs` (the digest of a
  result): `story:port-derivation` (it excluded `contract/` from the digest by prefix).
- `website/static/catalog/`: `story:contract-docs` writes the contract downloads there first; this
  story regenerates `sources.sha256` beside them.
- `docs/contract.md`: `story:port-derivation`, `story:contract-derivation`,
  `story:contract-docs`, `story:contract-conformance`.
- `CHANGELOG.md`: every story of this epic.

## Acceptance

- Given a contract whose invariant uses a predicate form the evaluator does not evaluate (a fixture
  over tool calls, which the simulator does not record), when `agenticspec sim contract --trace
  <trace>` runs, that invariant is reported as `not_checked`, never as `held`, and the run exits 0.
- Given `architectures/workstation.yaml` and `profiles/coding.yaml`, when `agenticspec sim trace
  architectures/workstation.yaml --profile profiles/coding.yaml --seed 1 --out <trace>` writes the
  trace and `agenticspec sim contract architectures/workstation.yaml --trace <trace>` reads it, it
  exits 0 and reports each invariant of the contract held, with the number of port uses it checked
  (more than 0 for the link `worker -> gateway carries model_call`).
- Given that trace with one inference use of a `worker` instance moved outside its gateway use,
  when the check reads it, it exits 1 naming the link `worker -> gateway carries model_call` and
  the index of the moved event.
- Given `spec/cli.yaml` with `sim trace` and `--trace`, when `task check:generated` runs, it
  exits 0.
- Given `task catalog:generate` after the change of `sim.rs`, when `git diff --stat
  website/static/catalog` runs, it lists only `sources.sha256` files, and `task check:docs` exits 0.

## Not in this story

A runtime's trace; scenario documents and their expectations (next epic); invariants the
simulator does not model (it records no tool calls), which the check reports as not checked
rather than held.
