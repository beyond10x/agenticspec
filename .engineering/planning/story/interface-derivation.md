---
format: aep.planning-md/3
id: story:interface-derivation
kind: story
status: draft
title: Resolve each ingress's interface into operations in the contract
summary: 'agenticspec sim contract resolves scope, commands, views, feeds and metrics into {kind, name} operations from the contract, ports.json, actors'' may: and the KPI catalogue; agenticspec-contract/1 gets its ingress section.'
relations:
- decomposes: epic:ingress
- depends_on: story:ingress-format
- depends_on: story:ingress-entities
- depends_on: story:contract-derivation
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-sim/src/contract
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: docs/contract.md
revision: 2
---
## Outcome

`agenticspec sim contract` resolves each ingress's interface into operations `{kind, name}` with
kind `command`, `view`, `feed` or `metric`, from the contract's block-to-domain table,
`blocks/ports.json` (principal ports, the configuration and administration split, actors that
issue reports), the actors' `may:` and `kpis/catalogue.yaml`. `agenticspec-contract/1` gets its
`ingress` section (ingress, audience, operations, export protocols), covered by the contract's
digest. The interface stores names only; fields come from the compiled model at projection.
Design: `design/ingress.md` I2, I4, I5 (upper bound), I8. Epic: `epic:ingress`.

## Scope

- `crates/agenticspec-sim/src/contract/`: interface resolution and the `ingress` section.
- `crates/agenticspec-sim/tests/`: the workstation example and one case per key, each under 5 s.
- `docs/contract.md`: the `ingress` section.
- `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `architectures/workstation.yaml` with the design's example, when
  `agenticspec sim contract` runs, ingress `user` resolves to exactly the commands
  `agentic.channel.SendMessage` and `agentic.question.Answer`, and its views include
  `agentic.question.OpenQuestions` and `agentic.harness.Turns`.
- Given the same document, ingress `operator` resolves to exactly the commands `DrainGateway`,
  `StopGateway` and `DetachUpstream`, and neither `OpenGateway` nor `AttachUpstream`.
- Given the same document, ingress `metrics` resolves to the 5 KPI ids and no command.
- Given an interface with `views: { except: [agentic.harness.Turns] }`, when the contract is
  derived, `Turns` is not among its operations.
- Given each document under `architectures/` without `ingress:`, when the contract is derived
  before and after this story, it is the same bytes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Projections per export (`story:ingress-projections`); the default rule values
(`story:control-plane-default`).
