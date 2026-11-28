---
format: aep.planning-md/3
id: epic:view-layer
kind: epic
status: active
title: 'The view layer: a default operator UI derived from an architecture tree'
summary: 'An architecture tree provides its own operator UI: generated block panels, part pages and overviews, live over the runtime''s adapter at a scope, with actions from the viewer''s action set (consumer gaps M43-M47).'
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T00:20:31Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-28T00:20:31Z", actor: "human:operator", revision: 3}
---
## Outcome

An architecture tree provides its own default operator UI. The tree is the architecture document
together with the documents it references by path and digest. From the components inside it, their
grouping, and the data, views and events those components expose, this repository generates an
operator UI: an overview per architecture level, a page per part, live panels of the block views
in scope, and the actions a principal may take. This repository builds the reusable view
components (one panel per block view, part pages, architecture overviews) and the generators.
agenticstudio renders the same components. A consumer builds its own app, places the components in
it, and adjusts them a little through a presentation-only `view:` key and the `--kit-*` tokens.

Design: `design/view-layer.md` (decisions D1-D11, open points O1-O9, runtime requests R1-R5,
Appendix A). The owner approved it as written on 2026-11-28, taking the recommended option of every
open point.

## Decisions taken

- D1: the authored half is an optional top-level `view:` key in the architecture document,
  outside the digest pin (`parts`, `links`, `border`); the derived half is a generated `ess-ui/1`
  document with a `sources.sha256` of its inputs, held by a fast digest check and a slow lane that
  compares bytes.
- D2: two generated products: block panels (`ui/views/blocks.ui.yaml`, one widget per block view,
  no dependency on any architecture) and one composition per architecture that copies the widgets
  it uses byte for byte.
- D3: the derivation rule from architecture element to UI element; a part's domains come from its
  contract, not from its block's name.
- D4: everything is generated except the `view:` key, which holds presentation only (`order`,
  `labels`, `hide`, `groups`, `home`) and is refused, with the entry's node path, when it names
  something that does not exist.
- D5: one channel per architecture level; panels refetch on the events of their view's source
  entity and never poll.
- D6: actions come from principal ports and from inputs into the border; machine traffic is never
  an action; visibility follows the viewer's action set and the runtime still refuses.
- D7: the UI talks to the adapter of the runtime that admitted the architecture, over R1-R5, with
  the scope in the base path `{root}/scopes/{architecture_id}[/{part path}]`.
- D8: `agenticspec ui panels` and `agenticspec ui derive` feed `agenticspec ui generate`, which
  writes Vue SFCs over `ui/kit/`.
- D9: large architectures lead with the parts table; the graph draws one level; links are read in
  a reach table.
- D10: the panel generator reads only the compiled model and names no block, view or command.
- D11: the only structural group is the nested architecture; the format gets no new group
  construct.
- O1 A (`view:` key inline), O2 A (Vue package and custom elements from one source), O3 A
  (placements of the entities a part's ports create, plus one hop over the view's id fields), O4 A
  (scope in the base path), O5 A (R5 answered by the authority evaluator), O6 A (principal ports
  plus inputs into the border), O7 A (one channel per architecture level), O8 A (field and pack
  domains compile into the system `agentic` under their own domains), O9 A (the per-architecture
  UI is generated in the consumer's repository by this CLI; this repository keeps `workstation`
  and the 37-part fixture as golden examples under `ui/views/examples/`).

## Consumer gaps

- M43, no generator from an architecture to `ess-ui/1`: `story:block-panels` and
  `story:architecture-ui`, on `story:ui-generator-panels`. This is the consumer's first gap, so the
  UI works on fixtures before any runtime exists.
- M44, no binding of a part to a live source: the UI side (one adapter address per admitted
  architecture, the scoped base path) in `story:runtime-ui-interface`. The address in the
  admission plan belongs to the runtime epic that `epic:architecture-contract` defers to, which
  has no story yet.
- M45, no common key to filter a view to one part: `story:placements`.
- M46, no declared event feed: `story:runtime-ui-interface` (R2 with a cursor and its
  conformance), with no ESS change.
- M47, no action set derived from grants: `story:action-sets`.
- M48, five domains declare no actor (`agent`, `architecture`, `experiment`, `kpi`,
  `principal`), so `ess ui check --model blocks` refuses every section over their views:
  `story:block-panels`, whose 0 errors include declaring those actors.
- M49, no block says which command a link's `carries` stands for: covered by D6 and
  `story:port-derivation`; the contract maps a link to a port and `blocks/ports.json` names the
  port's intent command (`story:architecture-ui` reads both). A group other than the machine
  carries with no action is a gap of `story:contract-derivation`.
- Grouping beyond the nested architecture: no new construct (D11).

## Stories, in order

1. S1: a `reached_by: network` component serving the 22 domains, added to `story:block-bindings`
   (its `blocks/components.yaml`), not a new story.
2. S2 `story:ui-generator-panels`: the generator additions of D8.
3. S3 `story:block-panels`: block panels, fixtures and gallery (M43).
4. S4 `story:architecture-ui`: the derivation from an architecture, the `view:` key and the
   37-part acceptance (M43).
5. S8 `story:views-package`: the Vue package and custom elements; agenticstudio shows the same
   panels.
6. S5 `story:placements`: placements and the scope rule (M45).
7. S6 `story:runtime-ui-interface`: R1-R4 with scope and the R2 cursor, and their conformance
   (M44, M46).
8. S7 `story:action-sets`: R5 and per-scope visibility (M47).

Every story that runs tests keeps each test within 60 s and its test step within 120 s. A job in
the build pool ends within 10 minutes.

## Not in this epic

The runtime itself: binding, admission and the adapter (`design/architecture-runtime.md`); the
core `ActionClass` catalog (`design/fields.md`, no story yet), which `story:action-sets` needs; a
React emitter; a new grouping construct in the architecture format.
