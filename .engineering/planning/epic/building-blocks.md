---
format: aep.planning-md/3
id: epic:building-blocks
kind: epic
status: active
title: The building blocks of agentic development as an ESS system
summary: One domain per block, one directory per layer, dependencies pointing down only, checked in the gate.
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T15:13:46Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T15:13:46Z", actor: "human:operator", revision: 3}
---
## Outcome

The building blocks of agentic development as an ESS system, `blocks/` (system `agentic`): one
domain per block, one directory per layer, dependencies pointing down only.

## Source

`design/building-blocks.md` §§ "The blocks", "A block is a contract", "Build order" steps 1 and 5.

## Acceptance

- `task check:blocks` passes: `ess specify validate --path blocks --strict-requires`, the layer
  check (`scripts/check-layers.sh`) and its negative fixtures, and the scenarios on the
  interpreter.
- Every block has a page on the docs site with its KPIs.

## State at creation

21 domains written and validated; 335 dependencies, none upward; 6 architecture scenarios pass.
Open: hand-written scenarios for `workspace` and `resource` (synthesis refuses most of their lease
and claim routes), the `UNMAPPED:` markers in the domain files.
