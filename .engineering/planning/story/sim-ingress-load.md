---
format: aep.planning-md/3
id: story:sim-ingress-load
kind: story
status: draft
title: 'Load per ingress in the simulator: arrivals, operations, rate limits, scrapes'
summary: load.ingress.<name> arrivals at border parts, administration operations injected by experiments, an ingress rate limit that may be marked, sampled scrapes; results byte for byte at the defaults.
relations:
- decomposes: epic:ingress
- depends_on: story:ingress-format
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-sim/src/marked.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/profile.rs
- confidence: cited
  path: crates/agenticspec-sim/src/sim.rs
- confidence: inferred
  path: crates/agenticspec-sim/tests
revision: 3
---
## Outcome

The simulator models load per ingress. A profile may give `load.ingress.<name>:
{ commands_per_hour, reads_per_hour }`; commands on work inputs are task arrivals at the border
part they land on. An experiment's `operations: [{ at, ingress, command }]` injects an
administration command like a burst, so a drain under load can be simulated. Reads cost 0 by
default. An ingress's `rate_limit_per_minute` (a place that may be marked with `vary`) refuses
arrivals over it and counts them. A scrape of a `metrics_scrape` export samples each KPI at each
interval, giving the predicted series. Without the new keys every result stays byte for byte.
Design: `design/ingress.md` I11. Epic: `epic:ingress`.

## Scope

- `crates/agenticspec-sim/src/sim.rs`, `src/profile.rs`, `src/marked.rs`: arrivals per ingress,
  operations, the rate limit as a marked place, sampled scrapes.
- `crates/agenticspec-sim/tests/`, each under 5 s.
- `docs/` simulator reference, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given every committed experiment result, when the simulator runs at this story's commit, each
  `result.json` is the same bytes as before.
- Given the workstation example with `load.ingress.user.commands_per_hour: 60` and
  `rate_limit_per_minute: 0.5` on `user`, when the simulator runs one hour, the result counts at
  most 30 admitted arrivals, at least one refused, and admitted plus refused equals the arrivals.
- Given an experiment with `operations: [{ at: 2h, ingress: operator, command:
  agentic.gateway.DrainGateway }]`, when it runs, the gateway is draining from 2 h in the timeline.
- Given a `metrics_scrape` export with a 60 s interval, when the simulator runs one hour, the
  result holds 60 samples per KPI.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Cloud load from clients (cloudspec, Q15 of `design/deployment.md`).
