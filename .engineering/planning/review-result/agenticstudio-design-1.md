---
format: aep.planning-md/3
id: review-result:agenticstudio-design-1
kind: review-result
status: active
title: 'agenticstudio plan, design critic, round 1: needs-revision'
owner: aep:plan-critic-design
relations:
- reviews: epic:agenticstudio
- reviews: story:architecture-patches
- reviews: story:studio-editor
- reviews: story:studio-first-slice
- reviews: story:studio-live
- reviews: story:studio-mcp
revision: 1
---
needs-revision

- story:studio-mcp — its acceptance reads a browser that "shows the new revision without a reload", and that live view is built by story:studio-editor ("live updates from the draft's event channel"). The first-slice page has no SSE, and no `depends_on` edge runs from mcp to editor. Either add `depends_on story:studio-editor`, or reword the line to a second client subscribed to `/drafts/<id>/events`, as story:studio-live's acceptance does. — .engineering/planning/story/studio-mcp.md:43, .engineering/planning/story/studio-editor.md:34
- story:studio-first-slice — it produces the `part`, `link` and `marker` facts but does not fix their fields, and three acceptance lines in the set read them under different keys. Its own acceptance uses `part{name: "worker"}`, the editor uses `part{block: "agent"}`, and patches uses `part{role: "worker"}` and `link{id, kind: "model_call"}`. `architectures/workstation.yaml` has `block`, not `name` or `role`, and its links carry `carries`, with no `id` or `kind`. The body should state each relation's fields once, derived from the document, so the editor and patches read the same schema. — .engineering/planning/story/studio-first-slice.md:37, .engineering/planning/story/studio-first-slice.md:58, .engineering/planning/story/studio-editor.md:44, .engineering/planning/story/architecture-patches.md:52, architectures/workstation.yaml:17

**What I read**
- 11 artifacts: the ten you named plus `epic:agenticstudio`. I ran `aep plan artifact show` on each.
- I ran `relations`, `graph` and `validate`; validate reported the store as valid.
- I also read `architectures/workstation.yaml` and the `UNMAPPED:` markers in `studio/domains/*.yaml`.

**Edges and reads traced**
- I walked all 38 declared edges in the whole-store graph, including edges to artifacts outside the set.
- I found no cycle, and the set is not a chain: `studio-editor` and `studio-evaluation` both hang off `studio-live`.
- I traced 14 acceptance reads to a producer in the set. 12 have an edge, counting transitive edges. The 2 without one are the first finding above (the live browser view) and the second (the fact fields).

**What I could not establish**
- Outside my lane (acceptance or scope): no story creates a route that lists or reads a revision.
  - `studio-editor` says the user steps through the draft's revisions.
  - `studio-drafts` acceptance says a draft "lists the same three revisions with the same digests".
  - Both consume this, and `studio-first-slice` and `studio-live` publish only a "revision committed" event.
- Unease, not a finding:
  - `studio-mcp` tool descriptions carry "the patch verbs", but `story:architecture-patches` completes that list and mcp does not depend on it.
  - `studio-chat` depends on `studio-mcp` although it uses the tool-call route that `studio-live` adds.
- I did not check `studio-sim` library entry points against the code.

```findings
- file: .engineering/planning/story/studio-mcp.md
  line: 43
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads a browser view that shows a new revision live without a reload, which story:studio-editor builds (live updates from the draft's event channel), and no depends_on edge runs from story:studio-mcp to story:studio-editor; add the edge or reword the line to a second SSE subscriber as story:studio-live does"
- file: .engineering/planning/story/studio-first-slice.md
  line: 37
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the body does not state the fields of the part, link and marker facts it produces, and the set's acceptances read them under different keys (name in first-slice, block in studio-editor, role and link id/kind in architecture-patches) that do not match architectures/workstation.yaml, so the body must fix one schema from the document"
```
