---
format: aep.planning-md/3
id: story:control-plane-pages
kind: story
status: draft
title: 'Control plane pages across parts: Inbox, Activity, Operations, Metrics, Architecture, Events'
summary: Pages selected by port class and the configuration and administration split, never by view name, overview tiles, and the default control planes of workstation and team-with-console as golden examples.
relations:
- decomposes: epic:control-plane
- depends_on: story:control-plane-default
- depends_on: story:view-layer-per-ingress
- depends_on: story:architecture-ui
- depends_on: story:architecture-references
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-ui
- confidence: inferred
  path: scripts/check-generated.sh
- confidence: cited
  path: ui/views/examples
revision: 3
---
## Outcome

The default control plane's UI has pages across parts, selected by port class and the
configuration and administration split, never by view name: Inbox (views over the entities of the
contract's principal ports, with `principal: all` as row actions), Activity (views over the
entities outbound and clock ports open, newest first), Operations (views over the entities
configuration creates, `Placements`, with `plays: administration` as row actions), Metrics (the
`metrics: computed` KPIs, a simulator prediction of the same revision beside each), Architecture
(`agentic.architecture` views, read-only), Events (the level's channel as a live log), and
overview tiles. Configuration commands, commands of an actor that issues a port's report and
machine traffic are never exposed. The default control planes of `workstation` and
`team-with-console` are golden examples. Extends `story:architecture-ui` and
`story:view-layer-per-ingress`. Design: `design/control-plane.md` C3, C7, C8. Epic:
`epic:control-plane`.

## Scope

- `crates/agenticspec-ui`: page selection by port class, the tiles, the Metrics page's prediction.
- `ui/views/examples/`: the default control planes of `workstation` and `team-with-console`, with
  `sources.sha256`.
- `scripts/check-generated.sh`, tests (vitest, Playwright on fixtures, one server per run),
  `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `team-with-console.yaml` with nothing added, when the default composition is derived, the
  Overview lists `console` (nested, first), `owner`, `worker` (3), `gateway`, `model`, and
  `ess ui check` exits 0.
- Given the same document, the Inbox answers the steward's questions to `owner` and the Operations
  page shows `gateway` and `console.gateway` with exactly `DrainGateway`, `StopGateway` and
  `DetachUpstream` as row actions.
- Given the same document, the Metrics page shows 9 KPIs.
- Given the derivation's source, when it is searched for a block, view, command or KPI name, none
  is found (C7).
- Given `task check:generated`, when it runs on the committed golden examples, it passes; a changed
  block makes it fail naming the stale example.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The deployment's units on the Operations page (`story:control-plane-units`); serving the pages
(`story:ui-serve-fixtures`).
