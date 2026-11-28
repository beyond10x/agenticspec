---
format: aep.planning-md/3
id: story:control-plane-units
kind: story
status: draft
title: Units and requirement status on the control plane
summary: The Operations page gains the Units view and the Overview a tile, rows reach their unit through placements (M45), the Metrics page shows each requirement met or missed, predicted beside observed.
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:control-plane-pages
- depends_on: story:deployment-domain
- depends_on: story:requirements-kpis
- depends_on: story:placements
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-ui
- confidence: inferred
  path: ui/views/examples
- confidence: inferred
  path: ui/views/fixtures
revision: 2
---
## Outcome

The control plane shows a deployment's units and its requirements. The Operations page gains the
`Units` view of `agentic.deployment` (desired and ready replicas, last failure, parts) and the
Overview a tile; a row of any view reaches its unit through its part (`Placement`, M45). The Metrics
page shows each requirement with its target, met or missed, predicted beside observed. Extends
`story:control-plane-pages`. Design: `design/deployment.md` K10, K12; `design/control-plane.md` C3.
Epic: `epic:architecture-deployment`.

## Scope

- `crates/agenticspec-ui`: the `Units` panel on Operations, the Overview tile, requirement status
  on Metrics, the unit link through placements.
- `ui/views/examples/`: the golden examples regenerated.
- `ui/views/fixtures/`: unit and requirement fixtures.
- Tests (vitest, Playwright on fixtures, one server per run), `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given the workstation's default control plane on a fixture with two units, one with a failure,
  when the Operations page opens, the `Units` panel shows both with desired and ready replicas and
  the failed unit's last failure.
- Given a harness session placed to a worker part packed in unit `workers`, when its row is opened,
  it links to unit `workers`.
- Given a document with one requirement met and one missed in the fixture, when the Metrics page
  opens, it shows both with their targets and marks one met and one missed.
- Given `task check:generated`, when it runs on the regenerated golden examples, it passes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Recording the target's reports (`story:agent-run-lift`, cloudspec C5).
