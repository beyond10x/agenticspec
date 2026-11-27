---
format: aep.planning-md/3
id: epic:research-blocks
kind: epic
status: draft
title: Blocks and block changes the research supports
summary: verification, memory, plan and monitor blocks; the frontier and trusted time; changes to channel, harness, work and workspace.
relations:
- depends_on: epic:building-blocks
- informed_by: epic:simulator-and-search
revision: 1
---
## Outcome

The blocks and block changes the research supports are in the specification and the simulator.

## Source

`design/research-findings.md` §§ "New blocks", "Changes to existing blocks"; and the agent
runtime's host ports in `design/building-blocks.md` § "Example: the agent runtime's host ports".

## Scope

- New blocks: `verification` (layer 6), `memory` (layer 3), `plan` (layer 5), `monitor`
  (layer 6).
- `authority`: the frontier (actions admissible now, and those that need approval), the
  submitter apart from the decider, an Undecided answer.
- `shared`: a trusted evaluation time.
- `channel`: request against inform, wake rules, a typed board.
- `harness`: enforced limits, a revision budget, no approval inside a compaction summary.
- `work`: a delay state for overlapping files, a forbidden-file list.
- `workspace`: ports and databases as leased shared state.

## Acceptance

- Each change validates, passes the layer check, and has at least one scenario.
- Each new block has KPIs in `kpis/catalogue.yaml` with its paper id.
