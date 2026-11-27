---
format: aep.planning-md/3
id: story:studio-mcp
kind: story
status: draft
title: 'The agent channel: query, apply, check and simulate as MCP tools on a draft'
summary: crates/agenticstudio-mcp lets any agent harness drive a draft through four tools whose answers explain refusals and KPIs, while every client on the draft sees each change live.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-live
- depends_on: story:studio-evaluation
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
revision: 8
---
## Outcome

The agent channel: any agent harness drives a draft through four MCP tools, `query(selector)`,
`apply(patch, dry_run)`, `check` and `simulate(budget)`, and every client open on the draft sees
each change live. Selectors keep part and link ids out of the conversation, so a change is one
call with a small answer. The answers explain a refusal (the rule and its error) and a KPI (its
catalogue entry). Each call is a tool call with caller `Agent` in a studio session.

## Scope

- `crates/agenticstudio-mcp` (new): an MCP server (stdio, and streamable HTTP under
  `agenticstudio serve`) whose four tools call the tool-call route of `crates/agenticstudio`
  (`story:studio-live`).
- Tool descriptions carry the patch verbs (`story:studio-first-slice` and
  `story:architecture-patches`) and the fact relations with their fields as
  `story:studio-first-slice` § "Facts of an architecture" fixes them.
- An `apply` answers with the diff and the check result, on a dry run also the KPI change of a
  short simulation (`story:studio-evaluation`) and nothing committed.
- Explaining a refusal: a refused `apply` answers the 1-based position of the refusing rule and
  the error's name, or, when the check of the whole result refused, the check rule and the part
  it names (`Refusal` in `crates/agenticspec-sim/src/check.rs`).
- Explaining a KPI: each KPI a `simulate` or dry-run answer reports carries the `summary`, `unit`
  and `direction` of its entry in `kpis/catalogue.yaml`; a KPI with no entry is answered with its
  id and says it has none. The catalogue is read, not changed.
- `crates/agenticstudio`: the session an MCP client opens on a draft.
- `Cargo.toml`: the workspace member.
- `docs/studio.md`: connecting a harness to a draft.
- Tests replay a recorded MCP exchange; no model call and no network. A test ends within 60 s,
  the test step within 120 s.

## Shared surfaces

- `crates/agenticstudio`: after `story:studio-first-slice`, `story:studio-live`,
  `story:studio-editor` and `story:studio-evaluation` (depends on live, editor and evaluation);
  before `story:studio-drafts` and `story:studio-chat`, which depend on this story.
- `docs/studio.md`: after `story:studio-first-slice`, `story:studio-live` and
  `story:studio-evaluation`; before `story:studio-drafts`.
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
  carries the diff, the check result and the KPI change, and the draft stays at revision 1.
- Given a draft at revision 1, when an MCP client calls `apply` with a patch whose second rule has
  an unknown verb, the answer is `Refused` with rule position 2 and the error's name, and the
  draft stays at revision 1.
- Given a draft of `architectures/workstation.yaml`, when an MCP client calls `simulate`, the
  answer carries for the KPI `tasks_correct` the `summary`, `unit` and `direction` written for
  it in `kpis/catalogue.yaml`.
