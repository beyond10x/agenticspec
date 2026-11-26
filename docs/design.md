# sessionspec design (draft, 2026-11-26)

Status, 2026-11-26: first slice built. `spec/` declares the session IR (4 entities, 12 commands,
12 events, 4 views) and the CLI (`lift`, `kpi`); synthesis generates 82 of 82 capabilities and a
55-scenario conformance suite with 0 refusals. `crates/sessionspec` holds the Claude Code adapter,
the redaction pass, the replay through the generated component, and the KPIs. `sessionspec kpi`
over this host's 72 hours (3,309 transcripts) runs in 12 s and replays every lifted command with
0 refusals; the aggregate report is `research/2026-11-26-kpi-72h.json`. Field names stay proposals
until a release.

Adapter facts the first run found:

- Records are stamped out of order inside one transcript; replay keeps each agent's record order
  and never lets an agent's step sort before its `StartAgent`.
- A sub-agent transcript can open with records copied from another agent of the session (the
  same record `uuid` in up to 3 files). The agent read first keeps them; the others skip them.
  Without this, 4,665 commands were refused and copied model turns were counted twice.
- One tool result can be recorded twice; the call is decided once.

## Why

The owner, 2026-11-26: "we're basically building now KPIs describing the quality of our Claude sessions,
or in general agentic work." A session of any harness (Claude Code, Codex, a metaharness run, a
chat) is lifted into one formal representation, and the KPIs are evaluated on that representation,
never on a vendor transcript.

```text
vendor transcript ──adapter──▶ session IR ──redact──▶ redacted IR ──kpi──▶ report
 (claude, codex, …)            (local only)           (persistable)        (numbers + verdicts)
```

The name follows the workspace's `*spec` family: `codespec` holds an IR of code (`cir/1`) and
`codesec` holds rules over it; here the IR of a session and the KPIs over it live in one
repository (the owner: "It should all be in this one repo").

## Model: actors issue commands, commands cause events, events change state

The owner, 2026-11-26: "is it more about actors taking actions causing events, which would be more the
ESS approach? … it would also track the command side of things … And maybe also state changes over
time."

An event stream alone loses two things the KPIs need:

- **The decision on a command.** A tool call is requested, then accepted or refused, and by whom:
  the permission mode, a classifier, a hook, the human once. The baseline counts 352,166 accepts by
  mode, 119,899 by classifier, 2,588 refusals by hook and 345 accepts by the owner in 72 hours.
- **State that no record states.** No record says "the cache went cold" or "this session now waits
  on the owner". Both are derived from timestamps and usage. KPIs are mostly durations spent in a state.

### Actors

| actor | example |
|---|---|
| human | the owner typing in a terminal or in Slack |
| agent | the main loop of a session |
| sub-agent | spawned by an agent; forms a tree (parent, depth) |
| peer session | another session sending a message |
| delegate | a session acting for the human (the operator session); carries the human it acts for |
| hook | a pre/post tool hook, a stop hook |
| harness | compaction, notifications, auto-continuation, rate limits |
| tool provider | a built-in tool, an MCP server, a shell |

Attribution is tri-valued: known, inferred (from a text prefix), unknown. The baseline shows why:
Claude Code transcripts do not tell the owner's prompt from a prompt the operator session typed.

### Commands

`Prompt`, `ToolCall`, `Spawn`, `SendMessage`, `PermissionDecision`, `Interrupt`, `Compact`,
`SwitchModel`, `ClearContext`, `EndTurn`.

`ToolCall` carries the tool name and a category from a closed list (read, search, edit, execute,
network, spawn, message, plan, other). `Spawn` creates an actor. A command has an outcome:
accepted, refused (by whom, with what code), or no decision recorded.

### Events

`ModelTurn` (model, usage: uncached input, cache read, cache write per TTL, output, thinking),
`ToolResult` (ok or error, size), `Notified`, `TurnEnded`, `RateLimited`, `Compacted`,
`SessionStarted`, `SessionEnded`. Derived events are marked derived: `CacheBusted` with a cause
(TTL expired, model changed, compaction, unexplained).

### State over time

Projections, each a step function over wall time:

- **waiting on**: nothing (working), human, tool, sub-agent, peer, rate limit, build queue.
- context size, model, permission mode, cache warmth per actor.
- open sub-agents and their depth.

The autonomy KPI is a share of wall time in "waiting on: human", not a count of prompts.

## Redaction

A second transform, session IR → redacted IR. The redacted IR is the form that is persisted and
shared; the raw transcript and the unredacted IR stay on the host.

- Content (prompt text, tool input, tool output, model text) becomes digest, length and class.
- Detected secrets become typed placeholders with a per-session stable alias
  (`<secret:gitlab_token#1>`), so "the same token appeared again" stays visible.
- Personal data (email, names, phone numbers, customer identifiers) becomes per-session pseudonyms.
- Paths are rewritten relative to `$HOME` and to the repository root.
- Kept: structure, timing, usage, tool names and categories, decisions, actors.

Detection is pattern plus allow list of known fixtures (`AKIAIOSFODNN7EXAMPLE` and the like; the
baseline's AWS matches were mostly fixtures). A match that is neither a known fixture nor confirmed
is `unk`, never counted as a leak.

## Exposure as a quality criterion

The owner, 2026-11-26: "whether personal identifier information or secrets would actually be used that
could also be a actual quality criteria". The redaction pass produces these facts for free:

- entered context: a secret or personal datum appeared in a tool result or prompt;
- echoed: the agent wrote it in its own text;
- sent outward: it appeared in the argument of a command of category network, message, or a
  commit or push (the severe one);
- each by channel, actor and count, with verdict ok / gap / unk.

## KPI catalogue (first list)

- **cost**: output tokens, cache write tokens, priced per model.
- **cache**: hit ratio; busts by cause and tokens written again.
- **context**: largest context, time above 200k, compactions.
- **autonomy**: share of wall time waiting on the human; human prompts per 100 tool calls;
  permission dialogs shown to the human; longest run without human input.
- **concurrency**: actors active per minute (p50, p90, max).
- **delegation**: sub-agents per session, tree depth, share of tokens spent in sub-agents.
- **guardrails**: refusals per 1,000 tool calls by hook, classifier, safety check.
- **exposure**: the section above.

A KPI has a direction only relative to a declared profile. the owner: "full autonomy isn't necessarily
a quality thing … It only is if you're looking for autonomous execution". A session declares or is
assigned a profile (autonomous, pairing, review); the profile says which KPIs are ratcheted and in
which direction.

## Relations

- `ratchet`: a KPI threshold can later be expressed as a `ratchet/1` requirement. Not a dependency.
- `aep` `trace-ir/1` (`aep/crates/observe/trace-domain`): a flat, harness-neutral event IR used
  for transcript conformance of governed runs. An adapter from it is a later input format.
- `metaharness` `metaharness.event/1`: the event stream of runs metaharness starts; its `audit`
  refuses foreign transcripts. Not used (the owner, 2026-11-26: "Maybe let's not use the meta harness
  thing").

## Build order (proposal)

1. ESS domain for actor, command, decision, event, state; `ess specify validate` first.
2. Claude Code adapter, then the redaction pass, then KPIs reproducing
   `research/2026-11-26-baseline.md`.
3. Codex adapter.

## Open, with the default taken

- Q1. Attribution of prompts typed by another session. Default A: text-prefix inference, marked
  inferred; otherwise unknown.
- Q2. Where redacted IR is persisted. Default A: files in the repository that owns the work, one
  per session, content-addressed.
