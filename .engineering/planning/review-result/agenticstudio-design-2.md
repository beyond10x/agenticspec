---
format: aep.planning-md/3
id: review-result:agenticstudio-design-2
kind: review-result
status: active
title: 'agenticstudio plan, design critic, round 2: approve'
owner: aep:plan-critic-design
relations:
- reviews: epic:agenticstudio
- reviews: story:studio-first-slice
- reviews: story:studio-live
- reviews: story:studio-editor
- reviews: story:studio-evaluation
- reviews: story:studio-mcp
- reviews: story:studio-drafts
- reviews: story:studio-state-space
- reviews: story:studio-chat
- reviews: story:architecture-patches
- reviews: story:guards-and-limits
revision: 1
---
approve

Both round-1 findings are resolved. I found nothing in the revised set that would change what was drafted.

**Round-1 findings**
- **`story:studio-mcp`, live browser view with no edge to editor: resolved.**
  - `story:studio-mcp` now has `depends_on story:studio-editor` (graph: `story:studio-mcp -> story:studio-editor`).
  - Its acceptance at `.engineering/planning/story/studio-mcp.md` now reads "a second client subscribed to `/drafts/<id>/events` receives revision 2 without a reload". That is the channel `story:studio-live` builds, as I suggested.
- **`story:studio-first-slice`, fact fields not fixed: resolved.**
  - `story:studio-first-slice` now carries a § "Facts of an architecture". It gives one table of `part`, `setting`, `link`, `marker`, `policy` and `kpi` with their fields, and says which story makes each.
  - It also states that `name`, `role`, `kind` and a link `id` are not fields.
  - Every selector I read uses only those fields:
    - editor: `part{id: P, block: "agent"}`.
    - patches: `link{index: L, carries: "model_call", from: A}`, `policy{id: Q, value: 15m}` and `kpi{part: "worker", id: "tasks_correct", value: V}`.
    - evaluation, live and mcp: the same fields.
  - The epic points to this section.
- **Outside-lane note from round 1 (nothing creates a revision list or read route): resolved.** `story:studio-live` now owns `GET /drafts/<id>/revisions` and `/revisions/<number>`, and its acceptance checks both. The editor and drafts read them with edges to live.
- **Unease: `studio-mcp` names the patch verbs without depending on patches.** Resolved. `story:studio-mcp` reaches `story:architecture-patches` through `story:studio-evaluation`, and that edge is recorded.
- **Unease: `studio-chat` reads the tool-call route from live.** Resolved. `story:studio-chat` reaches `story:studio-live` through `story:studio-mcp` and `story:studio-editor`, and both edges are recorded.

**Fresh read of the set**
- No cycle.
- The main path is deep:
  - first-slice, live, editor, evaluation, mcp, drafts, state-space, chat.
  - guards to patches to evaluation enters from the side and runs beside live and editor.
- Every edge on that path has its reason written in the story's § "Shared surfaces", so the chain rule does not make it a finding.
- No split abstraction.
  - `story:studio-first-slice` makes `set` and `add`, and `story:architecture-patches` makes the other verbs and the `policy` and `kpi` facts. Each can be demonstrated alone.
  - The `kpi` relation is made once by `story:architecture-patches`. `story:studio-evaluation` uses it and has the edge.

**What I read**
- 11 artifacts: the epic and the ten stories, plus `review-result:agenticstudio-design-1`, all with `aep plan artifact show`.
- I also ran `aep plan artifact relations`, `graph` and `validate`; validate reports the store valid.

**Edges and reads traced**
- I walked the whole-store graph, including edges outside the set. The 19 `depends_on` edges inside the set and the 10 `decomposes` edges give no cycle.
- I traced about 24 acceptance reads to a producer in the set. All are recorded by a `depends_on` edge, directly or transitively.

**What I could not establish**
- Unease, not a finding: `story:studio-state-space` to `story:studio-drafts` and `story:studio-chat` to `story:studio-state-space` exist only because they share `ui/studio`. If panels were split into separate files, `story:studio-state-space` and `story:studio-drafts` could run at the same time. The recorded shared-file order is acceptable.
- Out of my lane (acceptance or scope):
  - `story:studio-first-slice` outcome promises that a refused patch shows its rule and error, but its one acceptance line only covers success.
  - `story:guards-and-limits` decomposes `epic:research-blocks`, not this epic.
- I did not run any command against the code, such as `workstation.yaml` link indices or `docs/queries.md`.

```findings
[]
```
