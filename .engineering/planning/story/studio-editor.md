---
format: aep.planning-md/3
id: story:studio-editor
kind: story
status: draft
title: 'The architecture graph as an editor: select, patch with live dry run, revisions'
summary: 'ui/studio, a Vue app: selectable graph, selector highlighting, a patch editor with live dry run, the revision list.'
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-live
- depends_on: story:guards-and-limits
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/agenticspec-render
- confidence: cited
  path: crates/agenticstudio
- confidence: cited
  path: scripts/check-generated.sh
- confidence: cited
  path: ui/studio
- confidence: cited
  path: ui/studio/src/generated
revision: 8
---
## Outcome

The architecture graph is an editor in the browser: the user selects parts and links in the
graph or by a selector, writes a patch and sees its dry run (diff and check) while typing, applies
it, and steps through the draft's revisions. The page of `story:studio-first-slice` becomes the
Vue app `ui/studio`.

## Scope

- `ui/studio` (new): Vue 3, Vite and TypeScript like `ui/app`; types from `ess generate types`
  over `studio/` and the `architecture` block (read, not changed), committed under
  `ui/studio/src/generated/` and listed in `scripts/check-generated.sh` as `ui/app/src/generated`
  is; a later story that changes `studio/` or that block regenerates them; the graph with selectable
  parts and links; a selector box (a `query` tool call) that highlights its selection; a patch
  editor whose every edit sends a dry run and shows its diff and check result; an apply button;
  the revision list of the draft (`GET /drafts/<id>/revisions` of `story:studio-live`) and the
  view of one earlier revision (`GET /drafts/<id>/revisions/<number>`); live updates from the
  draft's event channel. vitest and Playwright on fixtures, no server per test.
- Selectors use only the fields of `story:studio-first-slice` § "Facts of an architecture".
- `crates/agenticspec-render`: the graph carries each part's id and each link's `index`, so the
  editor can select and highlight them.
- `crates/agenticstudio`: serves the built app (`target/studio/www`).
- `Taskfile.yml`: `studio:build`, `check:studio-app`.
- Tests: a test ends within 60 s, the test step within 120 s.

## Shared surfaces

- `crates/agenticstudio`: after `story:studio-first-slice` and `story:studio-live` (depends on
  live); before `story:studio-evaluation`, `story:studio-mcp`, `story:studio-drafts` and
  `story:studio-chat`, which each depend on this story.
- `ui/studio`: this story creates it; `story:studio-evaluation`, `story:studio-drafts`,
  `story:studio-state-space` and `story:studio-chat` add to it and each depend on this story.
- `crates/agenticspec-render`: before `story:studio-state-space`, which depends on this story.
- `Taskfile.yml`: after `story:studio-first-slice`.
- `ui/studio/src/generated/` (types from the `architecture` block): before
  `story:architecture-patches`, which changes that block, depends on this story and regenerates
  the types; after `story:guards-and-limits`, which also changes that block and which this story
  depends on.

## Acceptance

- Given the editor on a draft of `architectures/workstation.yaml`, when the user types the
  selector `part{id: P, block: "agent"}`, exactly the parts `operator` and `worker` become
  highlighted.
- Given the editor on that draft at revision 1, when the user types the patch `set(P, count, 4)
  :- part{id: P, block: "agent", count: 12}.`, the dry run shows a diff that changes `worker`'s
  count from 12 to 4 and the check passing, and the draft stays at revision 1.
- Given that dry run, when the user presses apply, the draft moves to revision 2 and the graph
  shows `worker` at count 4.
- Given a draft at revision 2, when the user selects revision 1 in the revision list, the graph
  shows `worker` at count 12 and the draft's current revision stays 2.
