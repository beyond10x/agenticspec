---
format: aep.planning-md/3
id: story:placements
kind: story
status: implemented
title: 'Placements: record which part an entity sits on, and the scope rule over views'
summary: agentic.architecture.Placement, PlaceEntity and Placements; a row is in scope of a part when its id or an id field of its view is placed to the part (M45, O3).
relations:
- decomposes: epic:view-layer
- depends_on: story:views-package
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: inferred
  path: blocks/scenarios/architecture
- confidence: inferred
  path: crates/agenticspec-ui
- confidence: inferred
  path: docs/blocks/9-composition/architecture.md
- confidence: inferred
  path: ui/views/fixtures
- confidence: inferred
  path: website/src/generated
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:14Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:14Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:14Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A view's rows can be filtered to one part of an architecture. Measured at commit `310373a`
(`design/view-layer.md` Appendix A): 3 of 67 views carry a `PartId`, 5 carry only an architecture
or revision id, and 59 carry neither. Adding `part_id` to block entities is refused: the layer rule
lets a block name only its own layer or a lower one, and parts are layer 9. Instead the
composition block records where an entity sits: `agentic.architecture.Placement` (architecture,
revision, part path, instance, entity name, entity id), recorded by the runtime through
`PlaceEntity` and read through the view `Placements`. A row is in scope of a part when its id, or
an id field of its view, is placed to the part (O3 A: placements of the entities a part's ports
create, plus one hop over the view's id fields). Consumer gap M45. Design: `design/view-layer.md`
"Verdicts on M43-M47" (M45), O3 and Appendix A. Epic: `epic:view-layer`.

## Scope

- `blocks/9-composition/architecture.yaml`: the entity `Placement`, the command `PlaceEntity`
  with its event and refusals, and the view `Placements`. The part path follows
  `story:architecture-references` (`team.lead`), with an instance index for a part with `count`.
- `blocks/scenarios/architecture/`: authored scenarios for `PlaceEntity` and `Placements`.
- The scope rule as one function over a view's rows, its id fields and the placements, used by
  the fixture source of the view layer (inferred: `crates/agenticspec-ui`, `ui/views/fixtures/`),
  with the classes of Appendix A as test cases.
- Generated: `docs/blocks/9-composition/architecture.md` and `website/src/generated/`
  (`task docs:generate`).
- `CHANGELOG.md`.
- Time synthesis before and after (`scripts/check-blocks-conform.sh --slow`); a jump is fixed in
  this story. Each test ends within 60 s and the test step within 120 s; a pool job over
  10 minutes is killed.

## Shared surfaces

- `blocks/9-composition/architecture.yaml`: `story:architecture-references`,
  `story:contract-derivation` and `story:architecture-patches` change the same file; whichever
  lands second rebases on the other.
- `docs/blocks/`, `website/src/generated/`: every story that changes a block regenerates them.
- `crates/agenticspec-ui`, `ui/views/fixtures/`: `story:block-panels` and `story:architecture-ui`
  before this story; `story:runtime-ui-interface` reads the scope rule after it.
- `CHANGELOG.md`: every story.

## Acceptance

- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, it lists the entity `Placement` with architecture, revision, part path, instance, entity
  name and entity id, the command `PlaceEntity`, and the view `Placements`, where before none
  existed; `task check:layers` passes.
- Given an authored scenario that places a harness session to part `team.lead` of a revision, when
  `task check:blocks` runs, `Placements` returns that placement and the step ends within 120 s on
  the cached suite.
- Given a harness session placed to `team.lead` and a turn whose `harness_session_id` names it,
  when the scope rule filters `agentic.harness.Turns` to `team.lead`, the turn is in scope (one
  hop), and a turn of a session placed to another part is not.
- Given a row whose own id is placed to a part, when the scope rule filters its view to that part,
  the row is in scope; to any other part, it is not.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Recording placements in a running system (the runtime's job, `design/architecture-runtime.md`);
the scoped wire and its conformance (`story:runtime-ui-interface`); a `part_id` in any block
entity, which the layer rule refuses.
