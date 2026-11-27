---
format: aep.planning-md/3
id: epic:simulator-and-search
kind: epic
status: active
title: Check, simulate and optimize an architecture
summary: A behaviour profile drives a discrete-event simulation of an architecture; a search over marked places finds the variant with the best KPIs.
relations:
- depends_on: epic:building-blocks
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T15:13:46Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T15:13:46Z", actor: "human:operator", revision: 3}
---
## Outcome

An architecture document is checked, simulated with a behaviour profile and optimized over its
marked places; the result is one `result.json` and one page.

## Source

`design/building-blocks.md` §§ "Simulation", "Optimization markers", "Rendering", "Build order"
steps 3 and 4. Code: `crates/agenticspec-sim`, `crates/agenticspec-render`.

## Acceptance

- `agenticspec-sim check|run|render` on every file under `architectures/` and `experiments/`.
- Same experiment and seeds give the same `result.json` byte for byte at any thread count.
- The workstation search weighs trade-offs (it does not only raise the worker count to its
  maximum).
- The behaviour profile is fitted from the session store (not yet: values come from
  `design/research-findings.md`).
- The architecture check runs as commands of `agentic.architecture` on the ESS interpreter (not
  yet: the simulator applies the same rules directly).
