---
format: aep.planning-md/3
id: story:first-runtime
kind: story
status: draft
title: 'The first runtime: in process, with the first binding format and admission'
summary: One process, every capability in memory, model answers replayed; agenticspec-binding/1 and admit(contract, binding?, environment) refusing with every missing port; its event log checked against a scenario.
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:contract-derivation
- depends_on: story:block-bindings
- depends_on: story:scenario-format
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Cargo.toml
- confidence: inferred
  path: crates/agenticspec-cli-contract
- confidence: inferred
  path: crates/agenticspec-runtime
- confidence: inferred
  path: crates/agenticspec/src/main.rs
- confidence: inferred
  path: docs/binding.md
- confidence: inferred
  path: spec/cli.yaml
- confidence: inferred
  path: spec/domains/cli.yaml
revision: 3
---
## Outcome

The smallest first runtime of `design/architecture-runtime.md` runs an architecture in one process,
every capability in memory: an agent loop that sends each turn to `model.infer` (fixture replay
only), `channel.deliver`, `pool.lease` and `clock.fire` as an in-process queue, semaphore and
virtual clock, `workspace.provide` as a temporary directory, and sandbox confinement refused unless
the binding names a real one. Its output is the blocks' events as an event log, checked against a
scenario. This story also gives the first runtime its two inputs, which no other story owns: the
first version of `agenticspec-binding/1` (which concrete thing plays each part, a fixture being one
such thing) and `admit(contract, binding?, environment)`, which returns a plan or refuses with
every port the environment cannot serve. It is the local test of `design/deployment.md` K11.
Design: `design/architecture-runtime.md` § "Three documents", § "Admission", § "Fixtures are
capabilities", § "The smallest first runtime"; `design/deployment.md` K11, K14. Epic:
`epic:architecture-deployment`.

## Scope

- A runtime crate (inferred: `crates/agenticspec-runtime/`): admission, the in-memory capabilities,
  fixture replay of `model.infer`, the event log; `Cargo.toml`.
- The binding format and its check (inferred: `docs/binding.md`, a schema beside
  `docs/contract.md`).
- A command that admits and runs an architecture with a binding and a scenario, declared in
  `spec/cli.yaml` and `spec/domains/cli.yaml`; `crates/agenticspec-cli-contract/` regenerated;
  `crates/agenticspec/src/main.rs` dispatches it.
- Tests: the example architecture chosen for the first runtime with its scenario, each under 5 s.
- `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given the chosen example architecture, a test binding and a scenario with recorded model answers,
  when the runtime runs it, its event log meets every expectation of the scenario and the command
  exits 0.
- Given the same architecture and scenario, when `agenticspec-sim scenario run` predicts it, the
  runtime's event log and the prediction name the same block events per expectation.
- Given an environment that cannot serve two of the contract's ports, when admission runs, it
  refuses listing both ports, never only the first, and nothing runs.
- Given a binding that asks for sandbox confinement the runtime does not have, when admission runs,
  it refuses naming the sandbox part.
- Given the same inputs run twice, both event logs are the same bytes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Packing and units (`story:packing-format`); exports (`story:ingress-admission`); store and bus
fixtures (`story:binding-store-and-bus`); any remote target.

## Decision

Owner decision, 2026-11-28: not now. Consumers run their own servers as service units behind an adapter that `agenticspec ui conform` checks against the runtime UI interface; per-unit images (R83) and consumer UIs (R84) carry them in a bundle. This story stays draft until the owner reopens it.
