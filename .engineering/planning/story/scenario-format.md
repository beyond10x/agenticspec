---
format: aep.planning-md/3
id: story:scenario-format
kind: story
status: active
title: 'agenticspec-scenario/1 and its check: given rows, border inputs, fixtures, expected block events'
summary: The scenario format with its ESS nouns and agenticspec-sim scenario check; the then is the oracle of a run (consumer gaps M5, M10)
relations:
- decomposes: epic:architecture-scenarios
scope:
- confidence: cited
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/10-evaluation
- confidence: cited
  path: blocks/ess-inputs.yaml
- confidence: cited
  path: crates/agenticspec-sim/src/main.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/scenario
- confidence: cited
  path: crates/agenticspec-sim/tests/cases
- confidence: inferred
  path: docs/blocks
- confidence: inferred
  path: docs/scenarios.md
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: inferred
  path: scenarios
- confidence: inferred
  path: website/src/generated
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 13}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 14}
---
## Outcome

`agenticspec-scenario/1` exists and is checked. A consumer's gap report (M5, M10) asked for a
scenario format and for the scenario's expected events to be the oracle of a run; at `25fbfcb`
the format is a paragraph of `design/architecture-runtime.md` (§ "Three documents, kept apart",
lines 39-42) and nothing reads it.

A scenario has four sections:

- `given`: the architecture as `{ ref: architectures/<x>.yaml, digest: <sha256> }` (the SHA-256
  of the file's bytes, the same rule as `story:architecture-references`), and the initial state
  as block rows: `{ entity, identity, fields }`, each entity and field one the blocks declare.
- `when`: inputs, each at a time offset, at a border part of the architecture, with its author (a
  principal part, or an outside party) and its proof (`agentic.channel.Proof`,
  `blocks/7-collaboration/channel.yaml:61-63`).
- `fixtures`: stand-ins for external parts: model answers, tool results, a person's reply after a
  delay, keyed by part.
- `then`: expected block events, in order or as a set, each by its full name
  (`agentic.resource.ClaimGranted`, `blocks/8-place/resource.yaml:485`), and expected refusals,
  each as a command and its error (`agentic.resource.RequestClaim` refused with
  `agentic.resource.NoTimeLimit`, `resource.yaml:193-204`).

The `then` is the oracle (M10): a run of the scenario passes when its events meet the `then`;
the scenario states nothing else a run is judged by.

Open point, settled here: `epic:evaluation-loop` asks that its formats map to the existing
`eval-case/1` and `trace-spec/1` of the aep repository rather than adding a third. This story
says whether `agenticspec-scenario/1` maps to `eval-case/1` or why it cannot.

## Scope

- The ESS model of the scenario's nouns (epic § "Nouns (ESS)"): a domain file under
  `blocks/10-evaluation/`, listed in `blocks/ess-inputs.yaml`, identifiers in
  `blocks/0-foundation/shared.yaml`; at least two KPIs in `kpis/catalogue.yaml` if it is a new
  block; generated pages through `task docs:generate`.
- `crates/agenticspec-sim/src/scenario/`: parse and check. The names of entities, fields, events,
  commands and errors come from the compiled blocks (`ess specify compile --path blocks`), made
  once per test run (a committed generated file with a digest, or one fixture cached under
  `target/` by the digest of its sources), never per test.
- `agenticspec-sim scenario check <file>` in `crates/agenticspec-sim/src/main.rs` (at `25fbfcb`
  the simulator dispatches `check`, `run`, `render`, `ladder`, `main.rs:231-235`); if the
  `agenticspec sim <cmd>` dispatch of `epic:architecture-contract` has landed first, the
  subcommand is declared there under that epic's rule (`spec/cli.yaml`, `task generate`).
- `scenarios/workstation/owner-task.yaml`: the first scenario, of `architectures/workstation.yaml`.
- `docs/scenarios.md`: the format.
- `crates/agenticspec-sim/tests/cases/` and fixtures; builds and tests through `slot`, no test
  over 60 s, the test step within 120 s, a pool job over 10 minutes is killed.

## Shared surfaces

- `blocks/0-foundation/shared.yaml`, `blocks/ess-inputs.yaml`, `kpis/catalogue.yaml`:
  `story:governance-types`, `story:block-remainders`, `story:block-bindings`,
  `story:guards-and-limits` and the uncommitted contract nouns of `epic:architecture-contract`.
- `crates/agenticspec-sim/src/main.rs`: `story:contract-derivation` and the stories after it move
  the simulator's subcommands under `agenticspec sim`.
- The digest rule: `story:architecture-references` uses the same one; neither reads the other's
  code, so there is no edge.

## Acceptance

- Given the scenario nouns modelled, `ess specify validate --path blocks --strict-requires`,
  `task check:layers` and `task check:blocks` pass, where before no ESS document declared them.
- Given `scenarios/workstation/owner-task.yaml` (a prompt from `owner` at border part
  `operator`, a `then` with `agentic.work.RunStarted` and `agentic.resource.ClaimGranted`),
  `agenticspec-sim scenario check` exits 0.
- Given the same scenario with a digest that does not match `architectures/workstation.yaml`, it
  exits 1 with `architecture_digest_mismatch`.
- Given an input at part `worker`, which is not on the border `[operator]`, it exits 1 with
  `input_not_at_border`.
- Given a `then` naming the event `agentic.resource.ClaimGrantd`, it exits 1 with
  `unknown_event`; given a refusal of `agentic.resource.RequestClaim` with an error that command
  does not declare, it exits 1 with `unknown_refusal`; with `agentic.resource.NoTimeLimit` it
  passes.
- Given a `given` row of `agentic.resource.Claim` without `pool_id`, it exits 1 with
  `invalid_row` naming the entity and the field.
- Given a fixture for a part the architecture does not have, it exits 1 with
  `fixture_without_part`.
- The check run twice on one file prints the same bytes.

## Not in this story

Running a scenario (`story:scenario-simulator-run`); corpus tasks as inputs (`story:task-corpus`);
scenarios extracted from sessions (`story:scenario-input-extraction`); a runtime running a
scenario (the epic after `epic:architecture-contract`).

## Remaining at 9bb5998

Missing: then matches simulator event names instead of qualified block events; unknown_event, input_not_at_border and architecture_digest_mismatch do not exist (crates/agenticspec-sim/src/scenario.rs).
