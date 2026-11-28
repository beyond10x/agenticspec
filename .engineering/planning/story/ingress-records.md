---
format: aep.planning-md/3
id: story:ingress-records
kind: story
status: draft
title: 'Records of ingress calls: the agentic.ingress domain and its KPIs'
summary: agentic.ingress in layer 9 with one Call per inbound call, decided once (Admitted, Unauthenticated, NotInInterface, Forbidden, RateLimited, Draining); three KPIs in the catalogue.
relations:
- decomposes: epic:ingress
- depends_on: story:ingress-entities
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: blocks/9-composition/ingress.yaml
- confidence: inferred
  path: blocks/ess-inputs.yaml
- confidence: inferred
  path: blocks/scenarios/ingress
- confidence: inferred
  path: blocks/system.yaml
- confidence: inferred
  path: docs/blocks
- confidence: inferred
  path: docs/kpis-catalogue.md
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: inferred
  path: website/src/generated
revision: 3
---
## Outcome

An ingress records its decisions as the gateway does. A domain `agentic.ingress` (layer 9, beside
`architecture`) holds one `Call` per inbound call, decided once (`Admitted`, `Unauthenticated`,
`NotInInterface`, `Forbidden`, `RateLimited`, `Draining`) and completed with its outcome. It copies
the gateway's shape, not its record. Its events give three KPIs in `kpis/catalogue.yaml`:
`ingress_refused_share`, `ingress_latency_p90_seconds`, and the gate `feed_resume_gap_count`
(events missing after a resume, must be 0). Design: `design/ingress.md` I12, Q5 A. Epic:
`epic:ingress`.

## Scope

- `blocks/9-composition/ingress.yaml` (new): `Call`, its decision and completion commands, events,
  views; `blocks/system.yaml`, `blocks/ess-inputs.yaml`.
- `blocks/scenarios/ingress/` (new): one scenario per decision.
- `kpis/catalogue.yaml`: the three KPIs; `task check:kpis`.
- Generated: `docs/blocks/`, `docs/kpis-catalogue.md`, `website/src/generated/`.
- `CHANGELOG.md`.
- Time synthesis before and after; builds and tests through `slot`; no test over 60 s, the test
  step within 120 s, a pool job over 10 minutes is killed.

## Acceptance

- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, it lists the domain `agentic.ingress` with `Call` and the six decisions; `task check:layers`
  passes.
- Given an authored scenario that decides a call twice, when `task check:blocks` runs, the second
  decision is refused.
- Given `kpis/catalogue.yaml` at this story's commit, when `task check:kpis` runs, it passes and
  lists the three KPIs with block `ingress`.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Recording calls in a running runtime (`story:first-runtime` and later); the edge's refusals as
conformance (`story:ingress-conformance`).
