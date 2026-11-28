---
format: aep.planning-md/3
id: decision-blocker:shared-search
kind: decision-blocker
status: open
title: Search work waits for the shared search and its batch scoring mode
relations:
- blocks: story:studio-evaluation
- blocks: story:studio-state-space
- blocks: story:architecture-patches
- blocks: story:requirements-kpis
revision: 1
---
# Search moves to a shared search

Owner decision, 2026-11-28 (relayed): one shared search runs hill climbing, beam and bounded
enumeration of variants for every pack; a pack only scores a candidate through `check --json` and
`simulate`. This repository keeps `check` and `sim run` answering in JSON on one document, writes
variable values as `{value, vary}` and gives each KPI a `direction` (kpis/catalogue.yaml).

The search of this repository (crates/agenticspec-sim/src/search.rs, best.rs, ladder.rs) stays as
it is and gets no new work. Stories that extend it or build on it wait for the shared search and
its batch scoring mode: this blocker clears when that mode is specified and a story for this
repository is filed.
