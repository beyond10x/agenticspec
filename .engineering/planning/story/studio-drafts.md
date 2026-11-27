---
format: aep.planning-md/3
id: story:studio-drafts
kind: story
status: draft
title: Drafts persist, archive, undo and compare side by side
summary: Drafts with their revisions, sessions, tool calls and evaluations survive a restart; undo; two revisions compared, structure and KPIs.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-editor
- depends_on: story:studio-evaluation
- depends_on: story:studio-mcp
scope:
- confidence: cited
  path: crates/agenticstudio
- confidence: cited
  path: docs/studio.md
- confidence: cited
  path: studio
- confidence: cited
  path: ui/studio
revision: 7
---
## Outcome

Drafts persist and compare: named drafts with their revisions, sessions, tool calls and
evaluations survive a restart of the server; a draft is archived when done; undo returns to an
earlier revision; two revisions are compared side by side, structure and KPIs. Two drafts are
compared through their current revisions.

## Scope

- `crates/agenticstudio`: a store under the state directory (SQLite, as `agenticspec`'s
  `sessions.db`) for drafts, sessions and tool calls (those of the editor and of the MCP clients
  of `story:studio-mcp`), evaluations and the architecture block's patches, applications and
  revisions; the revision routes of `story:studio-live` read from it; `ArchiveDraft`; undo; a
  comparison of two revisions (parts, links, settings and policies changed, and the per-part KPIs
  of the last finished evaluation of each).
- `studio`: settles the `UNMAPPED:` of `studio.draft.Draft` (`studio/domains/draft.yaml`):
  undo as the inverse patch applied as a further revision, or a head that points back.
- `ui/studio`: the drafts list, archive, undo, the comparison view.
- `docs/studio.md`: drafts, undo, comparison.
- Tests: a test ends within 60 s, the test step within 120 s; the store of a test lives in its
  own temporary directory.

## Shared surfaces

- `crates/agenticstudio`: after `story:studio-first-slice`, `story:studio-live`,
  `story:studio-editor`, `story:studio-evaluation` and `story:studio-mcp` (depends on editor,
  evaluation and mcp); before `story:studio-chat`.
- `docs/studio.md`: after `story:studio-first-slice`, `story:studio-live`,
  `story:studio-evaluation` and `story:studio-mcp`.
- `ui/studio`: after `story:studio-editor` and `story:studio-evaluation`; before
  `story:studio-state-space` and `story:studio-chat`, which depend on this story directly or
  through state-space.
- `studio`: after `story:studio-first-slice` and `story:studio-evaluation`.

## Acceptance

- Given a draft with three applied patches, after a restart of `agenticstudio serve` its revision
  list answers the same revisions with the same digests as before the restart.
- Given an open draft, when it is archived, a later `apply` on it is refused with
  `studio.draft.ArchivedDraft` and its revision list is unchanged.
- Given a draft at revision 3, when the user undoes once, the draft's current document has the
  digest of revision 2.
- Given revisions 1 and 2 of a draft of `architectures/workstation.yaml` where revision 2 set
  `worker`'s count from 12 to 4, the comparison of the two lists one change, the field `count` of
  `part{id: "worker"}` from 12 to 4, and no other change.
- Given a finished evaluation on each of two revisions, their comparison shows for each part the
  per-part KPIs of both evaluations side by side.
