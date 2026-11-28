---
format: aep.planning-md/3
id: story:ui-serve-fixtures
kind: story
status: implemented
title: 'agenticspec ui serve: the default control plane on block fixtures'
summary: agenticspec ui serve <architecture> derives the default and answers R1 from block fixtures, R2 from channel scripts, R4 by a recording fixture and R5 allowing every action, on 127.0.0.1.
relations:
- decomposes: epic:control-plane
- depends_on: story:control-plane-default
- depends_on: story:block-panels
- depends_on: story:runtime-ui-interface
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-cli-contract
- confidence: inferred
  path: crates/agenticspec/src/serve.rs
- confidence: inferred
  path: spec/cli.yaml
- confidence: inferred
  path: spec/domains/cli.yaml
- confidence: inferred
  path: ui/README.md
- confidence: inferred
  path: ui/fixtures/scripts
- confidence: inferred
  path: ui/views/fixtures
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:19Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:20Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:20Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

On day one, with no runtime, `agenticspec ui serve <architecture>` derives the default control
plane and answers it on 127.0.0.1: R1 from the block fixtures of `story:block-panels`, R2 from
channel scripts, R4 by a fixture that records the command and returns a scripted outcome, R5 with
every action allowed. Fixtures are capabilities, not a test mode. Extends
`story:runtime-ui-interface` ("against fixtures first"). Design: `design/control-plane.md` C5, P7
A. Epic: `epic:control-plane`.

## Scope

- `crates/agenticspec/src/serve.rs` or a module beside it: the `ui serve` subcommand on R1, R2, R4,
  R5 fixtures; declared in `spec/cli.yaml`, `spec/domains/cli.yaml`;
  `crates/agenticspec-cli-contract/` regenerated.
- `ui/views/fixtures/`, `ui/fixtures/scripts/`: the channel scripts and the recorded-command
  fixture.
- `ui/README.md`, tests (one server per run), `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `agenticspec ui serve architectures/workstation.yaml`, when a client reads a view under
  `/ingress/operator/scopes/`, it receives the block fixture's rows, and the server listens on
  127.0.0.1 only.
- Given the same server, when a client resumes the channel from a cursor, it receives the scripted
  events after the cursor.
- Given the same server, when a client issues `DrainGateway`, the response is the scripted outcome
  and the fixture's record lists the command once.
- Given the same server, when R5 is read, every interface command is allowed.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Replaying a simulator run (`story:sim-event-log`); a real runtime (`story:first-runtime`).
