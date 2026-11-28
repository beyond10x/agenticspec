---
format: aep.planning-md/3
id: story:contract-conformance
kind: story
status: implemented
title: Cut the blocks' conformance suite to one contract's ports and run it
summary: agenticspec sim contract --suite keeps the scenarios of a contract's ports; task contract:conform runs them green on the ESS interpreter, as a cut suite or selected by scenario id.
relations:
- decomposes: epic:architecture-contract
- depends_on: story:contract-derivation
- depends_on: story:port-derivation
- depends_on: story:contract-docs
- depends_on: story:block-bindings
- depends_on: story:architecture-kpis
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-sim/src/cli.rs
- confidence: cited
  path: crates/agenticspec-sim/src/contract
- confidence: cited
  path: crates/agenticspec-sim/src/lib.rs
- confidence: cited
  path: crates/agenticspec-sim/tests
- confidence: cited
  path: crates/agenticspec/src/main.rs
- confidence: inferred
  path: design/ess-repro
- confidence: cited
  path: docs/contract.md
- confidence: cited
  path: spec/cli.yaml
- confidence: cited
  path: spec/domains/cli.yaml
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:13Z", actor: "human:operator", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:14Z", actor: "human:operator", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:14Z", actor: "human:operator", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

The conformance suite of `blocks/` can be cut to one contract: `agenticspec sim contract
<architecture.yaml> --suite <suite.json> --out <suite.json>` keeps the scenarios a runtime of that
architecture must pass and drops the rest, so a runtime that claims the contract's ports is held
to exactly those blocks' behaviour (`design/architecture-runtime.md` § "What the runtime emits,
and what conformance means", block conformance). The kept scenarios run green on the ESS
interpreter, by one of the two paths below, whichever `ess` admits.

## Scope

- `crates/agenticspec-sim/src/contract/suite.rs`: read an `ess-conformance/<N>` suite (the cached
  one, `target/blocks-conform/<digest>.json`, from `scripts/check-blocks-conform.sh`); keep a
  scenario when the command it is about (in its id: `<entity>/transition/<t>/by/<command>/...`,
  `<command>/outcome/<o>`, `<binding>/binding/...`) is a command of a port of the contract, or a
  binding whose invoked command is one; keep the setup steps it needs; drop the rest. Write the
  kept scenarios with the suite's provenance.
- Open point, settled in this story and written on `docs/contract.md`: whether
  `ess verify conform run` admits a suite with scenarios removed (its provenance carries
  `contract_digest` and `spec_digest`). `ess` 0.99.2 has no option that runs a subset of a named
  suite (`--known-failing` changes accounting, not the run). The story takes one of two paths and
  says which on `docs/contract.md`:
  - **Path A, `ess` admits the cut suite**: `task contract:conform` runs
    `ess verify conform run --target interpreted --path blocks --suite <cut suite>`.
  - **Path B, `ess` refuses it**: the story writes the gap into `epic:architecture-contract`
    § "What ESS lacks" with a reproduction under `design/ess-repro/<gap>/` (a cut suite and the
    refusal `ess` prints), and selects by scenario id instead: `task contract:conform` runs the
    whole cached suite with `--report-format 2 --report-out <report>`, then
    `agenticspec sim contract <architecture.yaml> --suite <suite.json> --report <report>` prints
    each kept scenario id with its result and the count kept against the count of the whole
    suite, and exits 1 when a kept scenario failed.
- `--suite` (and `--report` on path B) on `sim contract` are declared in
  `spec/cli.yaml`, their types in `spec/domains/cli.yaml`; `task generate` regenerates
  `crates/agenticspec-cli-contract/` (held by `task check:generated`);
  `crates/agenticspec/src/main.rs` passes them to `crates/agenticspec-sim/src/cli.rs`.
- `src/lib.rs`, `tests/main.rs` with a small suite fixture under `tests/cases/` (not the 10 MB
  suite), and on path B a small report fixture; each test under 5 s.
- `Taskfile.yml`: `task contract:conform -- <architecture.yaml>`, which takes the cached suite,
  cuts it and runs it on the interpreter by the path taken.
- `docs/contract.md`, `CHANGELOG.md`.
- Builds and tests through `slot`; the cut and the run end within 120 s, no test over 60 s, a pool
  job over 10 minutes is killed.

## Shared surfaces

- `crates/agenticspec-sim/src/contract/`, `src/cli.rs`, `src/lib.rs`, `tests/`: `story:port-derivation`,
  `story:contract-derivation` (both come first), `story:routing-invariants`. Outside this epic,
  `story:architecture-kpis` adds tests to `crates/agenticspec-sim/tests`.
- `spec/cli.yaml`, `spec/domains/cli.yaml`, `crates/agenticspec-cli-contract/`,
  `crates/agenticspec/src/main.rs`: `story:port-derivation`, `story:contract-derivation` (both
  come first), `story:routing-invariants`. Outside this epic, `story:architecture-patches`
  records all of `crates/agenticspec-sim` and `crates/agenticspec/src` as its scope;
  `aep plan artifact waves` keeps them in different waves, and whichever lands second rebases on
  the other.
- `Taskfile.yml`: `story:port-derivation`.
- `docs/contract.md`: `story:port-derivation`, `story:contract-derivation`,
  `story:contract-docs`, `story:routing-invariants`.
- `epic:architecture-contract` § "What ESS lacks" (path B only): `story:block-bindings`.
- `blocks/` (read only, through the cached suite): `story:block-bindings`.
- `CHANGELOG.md`: every story of this epic.

## Acceptance

- Given the cached suite of `blocks/` and `architectures/workstation.yaml`, when
  `agenticspec sim contract architectures/workstation.yaml --suite <cached suite> --out <cut>`
  runs, the cut suite holds no scenario about `agentic.runtime.StartInstance`,
  `agentic.trigger.FireTrigger` or `agentic.channel.PostMessage`, and holds at least one about
  `agentic.model.FinishInference`.
- Given `architectures/scheduled-loops.yaml`, when the suite is cut to its contract, the cut
  suite holds the scenarios about the clock port's commands that the whole suite holds.
- Given path A, when `task contract:conform -- architectures/workstation.yaml` runs, it exits 0
  and `ess verify conform run --target interpreted` on the cut suite reports every kept scenario
  passed, with the count kept printed against the count of the whole suite.
- Given path B, when `ess verify conform run --target interpreted` runs on the cut suite of
  `design/ess-repro/<gap>/`, it refuses it with the error § "What ESS lacks" quotes; and when
  `task contract:conform -- architectures/workstation.yaml` runs, it exits 0 and prints every kept
  scenario id as passed, with the count kept against the count of the whole suite.
- Given `docs/contract.md`, it names the path taken (A or B).

## Not in this story

Running a cut suite against any runtime other than the ESS interpreter (next epic); scenario
conformance (`agenticspec-scenario/1`, next epic).
