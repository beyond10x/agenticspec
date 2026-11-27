---
format: aep.planning-md/3
id: story:agentic-ladder
kind: story
status: implemented
title: 'The agentic ladder: a defined model and an animated architecture evolution on the site'
relations:
- decomposes: epic:docs-site
scope:
- confidence: cited
  path: architectures/ladder
- confidence: cited
  path: docs/ladder.mdx
- confidence: cited
  path: ladder
- confidence: cited
  path: website/src/components/ladder
- confidence: cited
  path: website/static/ladder
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T18:57:01Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T18:57:02Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-27T21:31:22Z", actor: "human:operator", revision: 9, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The agentic ladder on the docs site: a defined model (stages, barriers, the trust each stage
needs) and an animated page where one operator's architecture grows rung by rung, from one
sandboxed agent session at a workstation to a factory of teams. Each rung is a real architecture
document that `agenticspec-sim check` accepts, so the illustration is drawn from the blocks, not
painted. Usable as marketing material.

## Source

The ladder's owner's own draft of an agentic mastery ladder (seven stages measured by throughput,
three barriers, each stage with a trust need), restated without any company, person or assessment
data. The operator's journey asked for on 2026-11-27: a sandboxed agent session by default, then
the sandbox switched off, then sub-agents, then several background sessions, then scheduled
agents, then a swarm, a team, several departments, a full flow.

## Scope

- `ladder/ladder.yaml` (`agenticspec-ladder/1`): stages with marker, trust need, unit of work and
  the architecture of the rung; barriers between rungs.
- `architectures/ladder/NN-*.yaml`: one architecture per rung.
- A generated data file for the page and its check in the gate.
- `docs/ladder.mdx` and `website/src/components/ladder/`: the animated page.

## Acceptance

- Every rung's architecture passes `agenticspec-sim check`; the ladder file is checked (every
  stage has an architecture, every barrier sits between two existing stages).
- The page plays the evolution step by step (play, pause, previous, next, keyboard), shows what
  each rung adds and which trust part arrives with it, and honours reduced motion.
- No company, person or customer names; readable in light and dark at 1280 and 390 px; axe 0
  serious.
