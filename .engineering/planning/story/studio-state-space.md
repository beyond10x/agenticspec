---
format: aep.planning-md/3
id: story:studio-state-space
kind: story
status: draft
title: 'State-space views of an evaluation: trace, variants, landscape, reached specification states'
summary: The studio shows a search's trace, variants and landscape, and a state graph of the block lifecycle states the simulation reached.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-evaluation
- depends_on: story:studio-editor
- depends_on: story:studio-drafts
scope:
- confidence: cited
  path: crates/agenticspec-render
- confidence: cited
  path: crates/agenticspec-sim
- confidence: cited
  path: ui/studio
revision: 5
---
## Outcome

The studio shows the state space of an evaluation: the hill-climbing trace of a search, its
variants, the landscape over the marked places, and the states of the specification the
simulation reached (the lifecycle states of the blocks' entities, with how often each was
entered and left).

## Scope

- `crates/agenticspec-sim`: the result records the lifecycle states each block entity of the
  simulation reached and the transitions taken, with counts (`reached_states` in `result.json`,
  a new key of `agenticspec-sim-result/1`). Open point: which block entities the simulator
  models states of; taken: those whose lifecycle the run moves.
- `crates/agenticspec-render`: a state graph view beside the views of `story:optimization-view`
  (trace, landscape, trade-offs, sensitivity in `crates/agenticspec-render/src/optim.rs`), usable
  as a fragment, as those views are made usable as fragments here.
- `ui/studio`: a state-space panel of an evaluation with four views: the trace (`trace` of
  `result.json`), the variants (`variants`), the landscape over the marked places (`marked`), the
  state graph (`reached_states`).
- Tests: a test ends within 60 s, the test step within 120 s; views tested on a recorded
  `result.json`, no simulation per test.

## Shared surfaces

- `crates/agenticspec-sim`: after `story:guards-and-limits`, `story:architecture-patches` and
  `story:studio-evaluation` (depends on evaluation, which depends on patches, which depends on
  guards).
- `crates/agenticspec-render`: after `story:studio-editor` (depends on it).
- `ui/studio`: after `story:studio-editor`, `story:studio-evaluation` and `story:studio-drafts`
  (depends on drafts); before `story:studio-chat`, which depends on this story.

## Acceptance

- Given a `Done` search on a draft of `architectures/workstation.yaml`, the panel's trace shows
  one point per entry of `trace` in its `result.json`, and its best-so-far line ends at
  `best.fitness`.
- Given that search, the panel's variants view lists every entry of `variants` with its fitness
  and whether it is feasible, and no other.
- Given that search, the panel's landscape offers as its places exactly the `path`s of `marked`
  in its `result.json`.
- Given that search, the panel's state graph shows counts equal to the `reached_states` of its
  `result.json`, where before this story `result.json` had no `reached_states`.
