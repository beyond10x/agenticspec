---
format: aep.planning-md/3
id: story:runtime-ui-interface
kind: story
status: draft
title: 'The runtime interface of the UI: scoped reads, a resumable feed, commands, and their conformance'
summary: R1-R4 at {root}/scopes/{architecture_id}[/{part path}], the R2 cursor, and a conformance suite run against fixtures first and then the first runtime (M44, M46).
relations:
- decomposes: epic:view-layer
- depends_on: story:placements
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-ui
- confidence: inferred
  path: crates/agenticspec/src/serve.rs
- confidence: cited
  path: ui/README.md
- confidence: inferred
  path: ui/views/fixtures
revision: 3
---
## Outcome

The UI is one more consumer of the runtime that admitted the architecture: it talks to that
runtime's adapter, the same adapter block conformance drives, with no second server and no view
that exists only for the UI. This story fixes the wire of R1-R4 with the scope in the base path,
and the conformance that holds an adapter to it: scoped reads and feed resume from a cursor. It is
proved against the fixture source first and then against the first runtime that exists. Consumer
gaps M44 (the UI side: one adapter address per admitted architecture) and M46 (a declared event
feed, with no ESS change). Design: `design/view-layer.md` D7, O4, and the verdicts on M44 and M46.
Epic: `epic:view-layer`.

## Scope

- The wire (D7, O4), with `{source}` = `{root}/scopes/{architecture_id}[/{part path}]` and an
  instance index for a part with `count`:
  - R1 read: `GET {source}/api/views/{view}?{params}` returns `{"rows": [...]}`, only the rows in
    scope under the rule of `story:placements`;
  - R2 feed: `GET {source}/api/channels/{channel}?{params}`, SSE with `event:` the qualified name,
    `data:` the event JSON and `id:` the cursor; a reconnect with `Last-Event-ID` gets exactly what
    followed (the wire of `agenticspec serve` today);
  - R3 placements: R1 on `agentic.architecture.Placements`, `Parts`, `Reach`, `Revisions`,
    `Contracts`;
  - R4 command: `POST {source}/api/commands/{command}` returns `{"row"}`, or 422 with the
    refusal's row.
  - The adapter authenticates the caller before it dispatches.
- The fixture source (inferred: `ui/views/fixtures/` and its server for tests): answers R1-R4 at a
  scope from the fixture files and channel scripts.
- A conformance suite of R1-R4 (inferred: `crates/agenticspec-ui`), run against a source URL:
  scoped reads, feed resume from cursor c, R3 views, R4 refusal shape, an unauthenticated request.
- `crates/agenticspec/src/serve.rs` (inferred): the R2 cursor obligation, checked against the wire
  `agenticspec serve` already uses.
- `ui/README.md`: the wire, for a host that serves the panels.
- `CHANGELOG.md`.
- Tests: each ends within 60 s and the test step within 120 s; one server per run; no network
  request outside the local test server; a pool job over 10 minutes is killed.

## Shared surfaces

- `ui/views/fixtures/`, `crates/agenticspec-ui`: `story:block-panels`, `story:architecture-ui`
  and `story:placements` before this story; `story:action-sets` adds R5 after it.
- `crates/agenticspec/src/serve.rs`: the sessions UI's server; a change here keeps its wire.
- The admission plan's adapter address per architecture: the runtime epic that
  `epic:architecture-contract` defers to, which has no story yet; this story reads it as the
  `{root}` of a source and does not define it.

## Acceptance

- Given the fixture source and `{source}` = `{root}/scopes/{architecture_id}/team.lead`, when R1
  reads `agentic.harness.Turns`, it returns `{"rows": [...]}` with exactly the rows in scope of
  `team.lead` under the rule of `story:placements`, and no generated client or view parameter
  changed.
- Given a source that returns one out-of-scope row, when the conformance suite runs against it,
  the scoped-read check fails and names the view and the row's key.
- Given a channel at a scope that has sent events 1 to 5, when a client reconnects with
  `Last-Event-ID` set to the cursor of event 3, it receives exactly events 4 and 5, in order, each
  with `event:` its qualified name; this holds against the fixture source and against `agenticspec
  serve`.
- Given a source that resends event 3 after that reconnect, when the conformance suite runs, the
  resume check fails.
- Given R3 at a scope, when `Placements`, `Parts`, `Reach`, `Revisions` and `Contracts` are read,
  each answers `{"rows": [...]}`.
- Given R4 with a command the model refuses, when it is posted, the answer is 422 with the
  refusal's row; with an accepted command, the answer is `{"row"}`.
- Given a request with no credentials, when it reaches a conforming source, it is refused before
  any dispatch and returns no row.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

R5 and action sets (`story:action-sets`); the runtime, binding and admission themselves
(`design/architecture-runtime.md`); an ESS change: channels already have `carries`,
`resume: from_last_seen`, `scope: param` and `session.per`.
