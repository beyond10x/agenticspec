---
format: aep.planning-md/3
id: story:guards-and-limits
kind: story
status: draft
title: 'Guards as building blocks: timeout, retry, budget cap, approval gate, reviewer, verifier'
relations:
- decomposes: epic:research-blocks
revision: 1
---
## Outcome

Guards are building blocks: a timeout, a retry policy, a budget cap, an approval gate, a reviewer
and a verifier can be put on a link or around a part, with their refusals and KPIs, so that a
patch can `wrap` with them and the simulator can run them.

## Source

`design/research-findings.md` § "Changes to existing blocks" (harness: enforced limits, a revision
budget); `epic:agenticstudio` § "Operations: select, then transform".

## Acceptance

- Guards declared in `blocks/` at the layer the rule allows (`task check:blocks` passes).
- Each guard has at least two KPIs in `kpis/catalogue.yaml`.
- The simulator runs a timeout and a retry on a link (`agenticspec-sim` test).
