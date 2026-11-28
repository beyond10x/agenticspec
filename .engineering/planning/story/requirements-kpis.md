---
format: aep.planning-md/3
id: story:requirements-kpis
kind: story
status: draft
title: Requirements as KPIs with a target, decided at check, evaluation or operation
summary: 'A top-level requirements: key outside the nesting digest; unknown KPI ids refused; each requirement decided at its stage; the search treats requirements as constraints and stops when all are met.'
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:architecture-kpis
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-sim/src/best.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/check.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/doc.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/search.rs
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: docs/architectures/index.md
revision: 2
---
## Outcome

An architecture states its requirements as KPIs with a target: a top-level `requirements:` key,
`- { kpi: cache_hit_share, at_least: 0.7 }`, optionally per part and window, portable and outside
the nesting digest like `ingress:`. The check refuses an unknown KPI id. Each requirement is decided
when its KPI can be read: a design KPI of `story:architecture-kpis` (`agents_total`,
`capacity_ratio`) at check, a simulated one at evaluation, a computed one in operation. The search
treats requirements as constraints before weights and stops when all are met. A cloud KPI is named
by its cloudspec id and evaluated by cloudspec. Extends `story:architecture-kpis`. Design:
`design/deployment.md` K12, Q14 A. Epic: `epic:architecture-deployment`.

## Scope

- `crates/agenticspec-sim/src/doc.rs`, `src/check.rs`: the key, the unknown-KPI refusal, the stage
  of each requirement, check-stage decisions.
- `crates/agenticspec-sim/src/search.rs`: requirements as constraints and the stop rule.
- `crates/agenticspec-sim/src/best.rs`: `TOP` keeps `requirements`.
- `crates/agenticspec-sim/tests/`, each under 5 s; `docs/architectures/index.md`; `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given a document with `requirements: [{ kpi: no_such_kpi, at_least: 1 }]`, when
  `agenticspec sim check` runs, it exits 1 naming the KPI id.
- Given a document with `{ kpi: agents_total, at_most: 4 }` and 12 agents, when
  `agenticspec sim check` runs, it reports the requirement missed at check.
- Given an experiment whose document requires a simulated KPI at a target some variant meets, when
  the search runs, it stops at the first variant that meets every requirement and the result names
  each requirement met.
- Given every committed experiment without `requirements:`, when it runs at this story's commit,
  each `result.json` is the same bytes as before.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Requirement status on the Metrics page (`story:control-plane-units`); the bundle's `requirements`
member (`story:deployment-bundle`).
