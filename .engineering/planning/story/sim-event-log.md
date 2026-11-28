---
format: aep.planning-md/3
id: story:sim-event-log
kind: story
status: draft
title: The simulator's run as block events, replayed by ui serve --sim
summary: The simulator writes its timeline as block events with subject keys and part paths; agenticspec ui serve --sim replays it; every result.json stays byte for byte.
relations:
- decomposes: epic:control-plane
- depends_on: story:ui-serve-fixtures
- depends_on: story:simulator-fidelity
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-cli-contract
- confidence: inferred
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/src/sim/swarm.rs
- confidence: inferred
  path: crates/agenticspec/src/serve.rs
- confidence: inferred
  path: spec/cli.yaml
- confidence: inferred
  path: spec/domains/cli.yaml
- confidence: inferred
  path: ui/README.md
revision: 3
---
## Outcome

The simulator writes its run as block events with subject keys and part paths, so
`agenticspec ui serve <architecture> --sim <profile>` replays a simulator run instead of fixtures
and the Inbox, Activity and Metrics pages move as the run does. The event log is an extra output;
every `result.json` stays byte for byte. Design: `design/control-plane.md` C5, P7 A. Epic:
`epic:control-plane`.

## Scope

- `crates/agenticspec-sim/src/sim.rs`, `src/sim/swarm.rs`: the timeline as block events with
  qualified names, subject keys and part paths.
- `crates/agenticspec/src/serve.rs` (the `ui serve` of `story:ui-serve-fixtures`): `--sim`.
- `spec/cli.yaml`, `spec/domains/cli.yaml`, `crates/agenticspec-cli-contract/`.
- Tests, `ui/README.md`, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given every committed experiment, when the simulator runs at this story's commit, each
  `result.json` is the same bytes as before.
- Given a simulator run of the workstation with the event log written, every event in it carries a
  qualified block event name, a subject key and a part path of the document.
- Given `agenticspec ui serve architectures/workstation.yaml --sim profiles/coding.yaml`, when a
  client follows the operator feed, it receives the run's events in order, and an Activity view
  read after them holds the sessions they opened.
- Given the same run written twice, both event logs are the same bytes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Store events and reactions in the simulator (`story:sim-store-and-bus`).
