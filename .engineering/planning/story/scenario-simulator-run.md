---
format: aep.planning-md/3
id: story:scenario-simulator-run
kind: story
status: active
title: Run a scenario in the simulator, predicted per expectation
summary: agenticspec-sim scenario run feeds a scenario's inputs and fixtures to the simulator and judges each expectation met, missed or not_simulated, leaving the observed column to a runtime (consumer gaps M9 simulator side, M10)
relations:
- decomposes: epic:architecture-scenarios
- depends_on: story:scenario-format
- depends_on: story:routing-invariants
scope:
- confidence: inferred
  path: crates/agenticspec-sim/src/main.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/scenario
- confidence: cited
  path: crates/agenticspec-sim/src/sim
- confidence: cited
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/tests/cases
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 8}
---
## Outcome

`agenticspec-sim scenario run` runs one scenario in the simulator and reports, per expectation of
its `then`, what the simulator predicted. A consumer's gap report (M9, M10) asked to replay a
scenario against the simulator and a runtime with predicted against observed per scenario; this
story is the simulator side. The result has a `predicted` column per expectation and leaves the
`observed` column for a runtime (the epic after `epic:architecture-contract`).

At `25fbfcb` the simulator records a timeline of its own event kinds, not block events
(`crates/agenticspec-sim/src/sim.rs:540-563`, `RunOut.timeline` and `enum Ev`), and takes tasks
from the profile's rate (`crates/agenticspec-sim/src/profile.rs:188-195`), not from inputs.
`story:routing-invariants` adds a port-level trace of a run (`agenticspec-trace/1`, each use of a
port by a part instance); this story reads that trace and maps each port use to the block events
of its commands, so it depends on that story.

## Scope

- `crates/agenticspec-sim/src/scenario/run.rs`: load the architecture by `ref`, refusing a digest
  mismatch as `scenario check` does; apply the `given` rows the simulator models (held claims,
  queued tasks) and report every other row `not_simulated`; feed each `when` input at its time
  to its border part; use a fixture where one is given instead of a draw from the profile (a
  person's reply after its delay, a model answer); record the trace and judge each expectation
  `met`, `missed` or `not_simulated`.
- `crates/agenticspec-sim/src/sim.rs`, `src/sim/`: inputs and fixtures as sources of the run.
- `agenticspec-scenario-result/1`: per expectation the event or refusal, its `predicted` verdict
  and time, and an empty `observed`; exit 0 when nothing is `missed`.
- The subcommand where `story:scenario-format` put `scenario check`.
- `crates/agenticspec-sim/tests/cases/`; builds and tests through `slot`, no test over 60 s, the
  test step within 120 s, a pool job over 10 minutes is killed.

## Shared surfaces

- `crates/agenticspec-sim/src/sim.rs`, `src/sim/`: `story:routing-invariants` (comes first, its
  trace is read here), `story:simulator-fidelity`, `story:guards-and-limits`,
  `story:task-corpus`; whichever lands second rebases.
- `crates/agenticspec-sim/src/scenario/`: `story:scenario-format` (comes first),
  `story:task-corpus`.

## Acceptance

- Given `scenarios/workstation/owner-task.yaml` and `profiles/coding.yaml` with seed 1,
  `agenticspec-sim scenario run` exits 0, every expectation is `met`, and each has a `predicted`
  time and no `observed` value.
- Given the same scenario with an added expectation `agentic.work.TaskCancelled` that the run
  does not produce, it exits 1, that expectation is `missed` and the others stay `met`.
- Given an expectation of an event the simulator does not model (a sandbox's refusal of egress),
  it is `not_simulated`, never `met`, and the run exits 0.
- Given a fixture of the owner's reply 600 s after a question, the trace shows the answer 600 s
  after the question, where a profile draw would give another delay.
- Given a scenario whose digest does not match its architecture, `run` exits 1 before the
  simulation starts, with the rule `scenario check` names.
- Given the same scenario and seed, the result is the same bytes at any thread count.

## Not in this story

The observed column (a runtime, next epic); corpus tasks as inputs (`story:task-corpus`); KPI
bounds in a `then`; the simulator modelling more blocks than it does today.
