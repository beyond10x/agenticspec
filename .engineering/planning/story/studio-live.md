---
format: aep.planning-md/3
id: story:studio-live
kind: story
status: draft
title: Studio sessions and tool calls, published live on a draft's event channel
summary: Every query, apply, check and simulate on a draft is a tool call of a session, each answer and revision reaches every client over SSE, and a draft's revisions can be listed and read.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-first-slice
scope:
- confidence: cited
  path: crates/agenticstudio
- confidence: cited
  path: docs/studio.md
revision: 5
---
## Outcome

Every operation on a draft is a tool call of a studio session, and every client open on the
draft sees it at once: the server records sessions and tool calls (`studio/domains/session.yaml`)
and publishes each answered call and each new revision on a server-sent-events channel of the
draft. The draft's revisions can be listed and read. This is the surface the editor, the agent
channel, the drafts and the chat all drive.

## Scope

- `crates/agenticstudio`: `StartSession`, `CloseSession`, `CallTool`, `AnswerToolCall`,
  `FailToolCall` per `studio/domains/session.yaml`; one route that takes a tool call (query,
  apply with `dry_run`, check; simulate answers `Failed` until `story:studio-evaluation`) with
  its caller; a `query` runs a selector over the draft's facts (the relations fixed in
  `story:studio-first-slice` § "Facts of an architecture") and answers the rows; the SSE channel
  `/drafts/<id>/events` (call answered, revision committed); the `ToolCalls` view of a session.
- `crates/agenticstudio`, revisions of a draft: `GET /drafts/<id>/revisions` lists them (the
  `number` and `digest` of each `agentic.architecture.ArchitectureRevision`, and the patch
  application that committed it), and `GET /drafts/<id>/revisions/<number>` reads one (its
  document, its graph, its check result). `story:studio-editor` steps through revisions with
  them and `story:studio-drafts` lists them after a restart.
- `docs/studio.md`: the tool calls, the channel and the revision routes.
- Tests in process, no network; a test ends within 60 s and the test step within 120 s.

## Shared surfaces

- `crates/agenticstudio`: after `story:studio-first-slice` (depends on it); before
  `story:studio-editor`, `story:studio-evaluation`, `story:studio-mcp`, `story:studio-drafts`
  and `story:studio-chat`, which depend on this story directly or through another of them.
- `docs/studio.md`: after `story:studio-first-slice`; before `story:studio-evaluation`,
  `story:studio-mcp` and `story:studio-drafts`.

## Acceptance

- Given two clients subscribed to the event channel of one draft at revision 1, when the first
  applies a patch through a tool call, the second receives revision 2 on the channel within one
  second, without a reload.
- Given a draft of `architectures/workstation.yaml` opened at revision 1 and then changed by two
  applied patches, the revision list route answers revisions 1, 2 and 3 with three different
  digests, and reading revision 1 answers the document with part `worker` at count 12.
