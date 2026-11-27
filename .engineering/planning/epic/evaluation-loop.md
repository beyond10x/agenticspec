---
format: aep.planning-md/3
id: epic:evaluation-loop
kind: epic
status: draft
title: From observed sessions to tasks, protocols, scores and KPIs
summary: Extract tasks from sessions, draft protocols, score agents and architectures against them, and grow the KPI catalogue.
relations:
- depends_on: epic:building-blocks
revision: 1
---
## Outcome

The loop from observed work to KPIs: lift a source into sessions, extract every concrete task,
draft a protocol per task or family, score an agent or an architecture against it, explore the
protocol's outcomes, and add KPIs that separate good runs from bad ones.

## Source

`design/building-blocks.md` § "Evaluation as a process".

## Acceptance

- A block `evaluation` (layer 10) declares `Source`, `ExtractedTask`, `DraftProtocol`, `Case`,
  `Score`, `KpiCandidate`, with the refusals the design names; `task check:blocks` passes.
- Tasks are extracted from the session store with their evidence turns.
- A protocol drafted for a task is scored against at least one recorded run.
- The formats map to the existing `eval-case/1` and `trace-spec/1` of the aep repository rather
  than adding a third.

## Before decomposition

The block's nouns are modelled in ESS first (guardrail 7).
