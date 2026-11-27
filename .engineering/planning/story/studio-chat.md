---
format: aep.planning-md/3
id: story:studio-chat
kind: story
status: draft
title: 'Later: an in-browser chat through an LLM gateway driving the studio tools'
summary: A chat panel beside the editor whose model calls the four studio tools on the open draft through a configured gateway.
tags:
- later
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-mcp
- depends_on: story:studio-editor
- depends_on: story:studio-state-space
scope:
- confidence: cited
  path: crates/agenticstudio
- confidence: cited
  path: ui/studio
revision: 4
---
## Outcome

Later: an agent channel embedded in the browser. A chat panel beside the editor talks to a model
through an LLM gateway; the model's tool calls are the four studio tools on the open draft, so
the user and the agent design in one place without a separate harness.

## Scope

- `crates/agenticstudio`: a chat route that forwards to a configured gateway (an
  OpenAI-compatible or Anthropic endpoint), runs the tool loop through the same tool-call route
  as `crates/agenticstudio-mcp` (`story:studio-live`, `story:studio-mcp`), and streams the answer
  on the draft's event channel; the gateway's address and key from the environment, never
  stored.
- `ui/studio`: the chat panel.
- Tests replay recorded gateway exchanges; no model call and no network in the test step. A test
  ends within 60 s, the test step within 120 s.

## Shared surfaces

This story is last in the set; it depends on every story it shares a surface with, directly or
through another.

- `crates/agenticstudio`: after `story:studio-first-slice`, `story:studio-live`,
  `story:studio-editor`, `story:studio-evaluation`, `story:studio-mcp` and `story:studio-drafts`
  (depends on mcp and editor, and on drafts through state-space).
- `ui/studio`: after `story:studio-editor`, `story:studio-evaluation`, `story:studio-drafts` and
  `story:studio-state-space` (depends on state-space).

## Acceptance

- Given a draft of `architectures/workstation.yaml` at revision 1 and a recorded gateway
  exchange, when the user sends the chat message "run every worker as 3 copies", exactly one
  `apply` tool call is made and the editor's graph shows revision 2 with `worker` at count 3.
