---
format: aep.planning-md/3
id: story:studio-mcp
kind: story
status: implemented
title: 'The agent channel: query, apply, check and simulate as MCP tools on a draft'
summary: crates/agenticstudio-mcp lets any agent harness drive a draft through four tools whose answers explain refusals and KPIs, while every client on the draft sees each change live.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-live
- depends_on: story:studio-editor
scope:
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/agenticstudio
- confidence: cited
  path: crates/agenticstudio-mcp
- confidence: cited
  path: docs/studio.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 10, decided_on: {"recorded":{"test_result":1,"review":1,"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 11, decided_on: {"recorded":{"test_result":1,"review":1,"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 12, decided_on: {"recorded":{"test_result":1,"review":1,"review_outcome":3}}}
---
## Outcome

The agent channel: any agent harness drives a draft through three MCP tools, `query(selector)`,
`apply(patch, dry_run)` and `check`, and every client open on the draft sees each change live.
Selectors keep part and link ids out of the conversation, so a change is one call with a small
answer. A refused answer explains the rule and its error. Each call is a tool call with caller
`Agent` in a studio session. The channel comes right after the editor (decision A, taken on
default 2026-11-28): the `simulate` tool and the KPI change of a dry run are added by
`story:studio-evaluation`, which depends on this story.

## Scope

- `crates/agenticstudio-mcp` (new): an MCP server (stdio, and streamable HTTP under
  `agenticstudio serve`) whose three tools call the tool-call route of `crates/agenticstudio`
  (`story:studio-live`).
- Tool descriptions carry the patch verbs (`story:studio-first-slice` and
  `story:architecture-patches`) and the fact relations with their fields as
  `story:studio-first-slice` § "Facts of an architecture" fixes them.
- An `apply` answers with the diff and the check result; a dry run commits nothing.
- Explaining a refusal: a refused `apply` answers the 1-based position of the refusing rule and
  the error's name, or, when the check of the whole result refused, the check rule and the part
  it names (`Refusal` in `crates/agenticspec-sim/src/check.rs`).
- `crates/agenticstudio`: the session an MCP client opens on a draft.
- `Cargo.toml`: the workspace member.
- `docs/studio.md`: connecting a harness to a draft.
- Tests replay a recorded MCP exchange; no model call and no network. A test ends within 60 s,
  the test step within 120 s.

## Shared surfaces

- `crates/agenticstudio`: after `story:studio-first-slice`, `story:studio-live` and
  `story:studio-editor` (depends on live and editor); before `story:studio-evaluation`,
  `story:studio-drafts` and `story:studio-chat`, which depend on this story.
- `docs/studio.md`: after `story:studio-first-slice` and `story:studio-live`; before
  `story:studio-evaluation` and `story:studio-drafts`.
- `Cargo.toml`: after `story:studio-first-slice`.

## Acceptance

- Given a draft of `architectures/workstation.yaml`, when an MCP client calls `query` with the
  selector `part{id: P, block: "agent"}`, the answer lists the rows `operator` and `worker` and no
  other, and the session's `ToolCalls` view gains one `query` call.
- Given a draft of `architectures/workstation.yaml` at revision 1, when an MCP client calls
  `check`, the answer is the result of `agenticspec-sim check` on revision 1, passing, and the
  draft stays at revision 1.
- Given a draft at revision 1, when an MCP client calls `apply` with a patch and `dry_run:
  false`, one answer carries the diff and the check result, and the draft moves to revision 2.
- Given that call, the session's `ToolCalls` view lists it with caller `Agent` and status
  `Answered`, where before it listed no call.
- Given that call, a second client subscribed to `/drafts/<id>/events` receives revision 2
  without a reload.
- Given a draft at revision 1, when an MCP client calls `apply` with `dry_run: true`, the answer
  carries the diff and the check result, and the draft stays at revision 1.
- Given a draft at revision 1, when an MCP client calls `apply` with a patch whose second rule has
  an unknown verb, the answer is `Refused` with rule position 2 and the error's name, and the
  draft stays at revision 1.
