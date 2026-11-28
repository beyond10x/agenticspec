---
format: aep.planning-md/3
id: story:administration-commands
kind: story
status: implemented
title: 'Configuration and administration: split entry commands by their outcomes'
summary: The port derivation classes an entry command creating its entity as configuration and one moving an existing entity as administration; blocks/ports.json marks both and every actor that issues a port report.
relations:
- decomposes: epic:ingress
- depends_on: story:port-derivation
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/ports.json
- confidence: inferred
  path: crates/agenticspec-sim/src/contract
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: docs/contract.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The port derivation splits the entry commands it calls configuration in two, by the command's
outcomes in the compiled model: **configuration**, an outcome `creates:` the entity (`OpenGateway`,
`AttachUpstream`, `RegisterPrincipal`, `DefineKpi`), never exposed in an interface; and
**administration**, every accepting outcome moves or updates an existing entity and the command is
no report of a port (`DrainGateway`, `StopGateway`, `DetachUpstream`, `RevokeGrant`), exposed
through an ingress's `commands.plays`. `blocks/ports.json` marks both classes and each ESS actor
whose `may:` holds the report of a port, so the ingress check needs no command name. Design:
`design/ingress.md` I4. Epic: `epic:ingress`.

## Scope

- `crates/agenticspec-sim/src/contract/`: the classification of `story:port-derivation`, split by
  outcome.
- `blocks/ports.json` (regenerated): a class per entry command, and the actors that issue reports.
- `crates/agenticspec-sim/tests/`, `docs/contract.md`, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `blocks/` at this story's commit, when `agenticspec sim ports` runs, `blocks/ports.json`
  classes `agentic.gateway.OpenGateway` and `AttachUpstream` as configuration and `DrainGateway`,
  `StopGateway`, `DetachUpstream` and `agentic.authority.RevokeGrant` as administration.
- Given the same file, it marks `GatewayProcess`, `Transport` and `resource.Scheduler` as actors
  that issue reports, and does not mark `GatewayOperator`.
- Given `blocks/ports.json` before and after this story, every port row other than the new class
  fields is the same.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The ingress check that reads the marks (`story:ingress-format`); interface resolution
(`story:interface-derivation`).
