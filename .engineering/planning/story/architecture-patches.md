---
format: aep.planning-md/3
id: story:architecture-patches
kind: story
status: draft
title: Change architectures by selector and transform patches
relations:
- decomposes: epic:agenticstudio
- depends_on: story:guards-and-limits
revision: 1
---
## Outcome

Architectures are changed by patches: a selector in the query language of `queries/*.sq` over
architecture facts, and a transform from a closed set of verbs. The studio agent, the user and the
optimizer use the same operations. Design: `epic:agenticstudio` § "Operations: select, then
transform".

## Scope

- `agenticspec-sim` emits an architecture (and the KPIs of its last run per part) as facts:
  `part`, `link`, `policy`, `marker`, `kpi`.
- The query engine (`crates/agenticspec-records/src/sq/`) reads those facts; `agenticspec query`
  runs a selector over an architecture.
- `agenticspec-patch/1`: an ordered list of rules whose heads are `set`, `add`, `remove`,
  `connect`, `replicate`, `wrap`, `prepend`, `append`, `mark`, `unmark`. A patch applies
  atomically as one revision and is refused whole with a named error.
- A patch compiles to the commands of the `architecture` block.
- The search accepts structural mutations as patches beside the marked numbers.

## Acceptance

- `wrap(L, timeout{after: 10m}) :- link{id: L, kind: "model_call", from: A}, part{id: A, role:
  "worker"}.` applied to `architectures/workstation.yaml` gives the expected diff and passes
  `agenticspec-sim check`.
- A refused patch changes nothing and names its rule and error.
- An experiment that lists patches as mutations reports the best structure.

## Needs

Guard parts (timeout, retry, budget cap, approval gate) as block concepts:
`story:guards-and-limits`.
