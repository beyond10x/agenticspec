# Interaction records: one session model for transcripts and gateway traffic

Status: proposal, 2026-11-26. Written by sessionspec with the gateway's dashboard maintainers and
the gateway's maintainers. No code is written for it until the owner accepts it.

## Problem

Two programs describe the same agent sessions and today share nothing:

- sessionspec reads harness transcripts on disk (Claude Code JSONL; Codex next) and lifts them
  into a session IR: sessions, agents, prompts, model turns, tool calls, decisions, exposures.
- llm-gateway sees every model call on the wire. It knows what a transcript cannot (route,
  upstream, attempts, declared-price cost, latency) and shows recent calls on its dashboard.

The owner asked for one concept: the gateway turns requests and responses into the same records
sessionspec lifts from transcripts, stores them the same way, and shows them in the same view.

## 1. One domain, defined once

The records are the domain `sessionspec.session` in sessionspec's ESS specification
(`spec/domains/session.yaml`). It is the agent-interaction model: it already holds the nouns the
gateway needs (session, agent, model turn with usage, tool call, decision, compaction) and every
actor and attribution rule.

ESS has no import across systems: a domain is named inside its system, and llm-gateway already
names llm's entities by `BOUNDARY:` comments for that reason. So "defined once" works this way:

- sessionspec owns the domain and publishes it as three artifacts, each generated from the one
  specification:
  1. the JSON Schema of every command and event (`ess generate --kind schema`), the wire contract;
  2. a Rust crate of the types, codecs and the generated component that checks every record
     (`sessionspec-types`, `session-ir`, today generated in `generated/`);
  3. a Rust library crate `sessionspec-records`: the append API, the store, the lifts from each
     source and the view projection. The gateway links it by git tag, the way it links llm.
- llm-gateway's specification names the records by a `BOUNDARY:` comment beside the field that
  carries them (one new field on its call record, section 6), and does not copy the domain.
- When ESS gains a cross-system import, the comment becomes the import. That is an ESS story, not a
  precondition.

## 2. Sources are adapters that append the same records

Every source turns what it sees into the IR's commands (`OpenSession`, `StartAgent`,
`SubmitPrompt`, `RecordModelTurn`, `RequestToolCall`, `DecideToolCall`, `CompleteToolCall`,
`CompactContext`, `EndTurn`, `RecordExposure`). The generated component accepts or refuses each
one and publishes the events; nothing downstream reads a source format.

### Disk: harness transcripts (exists)

One file per main agent plus `subagents/agent-<id>.jsonl` with a meta file naming the parent's
spawning tool call. Every record carries the session id, the response message id (`msg_…`) and the
request id (`req_…`).

### Wire: the gateway (new)

Per finished model call (not `POST /v1/messages/count_tokens`, which is no model turn). Content
comes from the inbound wire (`messages`, `chat`, `responses`), facts from the upstream; on a
translated call the two wires differ.

| record | from the request | from the answer |
|---|---|---|
| session | `x-claude-code-session-id`, else `session-id`, `session_id`, `x-opencode-session`; else the user agent (marked Inferred) | |
| agent | the message chain: a request whose history extends an earlier request's history plus its answer continues that agent; a history no chain extends is a new agent. On `responses`, `previous_response_id` (or `conversation`) keeps the history on the server, so that id is the chain | |
| side call | Claude Code's title, summary and quota calls share the session header and break the chain rule: a call with a small model, no tools and no history is a side call of the session, not a new agent | |
| parent agent, spawning call | the new agent's first user message equals, by digest, the input of a spawn tool call (`Task`, `Agent`) of another agent of the session | |
| prompt | the last user message when it holds text and no tool result | |
| tool call completed | each `tool_result` in the last user message: id, error flag, size | |
| model turn | | model, usage (input, output, cache read, cache write 5 m / 1 h), stop reason, message id |
| tool call requested | | each `tool_use` block: id, name, input digest and size |
| gateway facts | | call id (`x-llm-gateway-call`), route, upstream, attempts, queue wait, time to first token, declared-price cost, outcome, refusal |
| incomplete turn | | a client disconnect, an answer cut short by its upstream, a wire's terminal error or a gateway refusal marks the turn incomplete. A shed request or a failed authentication has no call id and no record |

Cost on the wire is a declared-price cost: token counts times a price the owner declares per
route. It is absent on routes without a price (a subscription, a local model). It is not a
provider's bill.

What this costs the gateway (llm's reading of main at 0.25.0):

- The request body is buffered raw (at most 32 MiB). Only translated calls are decoded today; a
  same-wire relay is not. The lift decodes per inbound wire with llm's request decoders.
- The answer is not buffered: it is relayed as it streams, and the scanner reads the usage object
  only. The message id and the tool-use blocks need new work: a bounded tap of the answer bytes or
  a streaming event reader per wire, with memory per call.
- The finished call's facts sit in two records today (the usage record with the cost; the
  dashboard's call with attempts, queue wait and time to first token). The port needs one
  finished-call value carrying both.

### What each source cannot see, and how a record says so

The IR marks every value a source did not observe: `Known`, `Inferred` (derived by a stated rule)
or `NotRecorded`. A reader never treats Inferred as Known.

Only the transcript sees:

- hook and permission decisions: on the wire a refused call appears as an error `tool_result`; the
  wire records the decision `NotRecorded` and the completion with its error flag;
- the author of a prompt (human, hook, harness): the wire records `Inferred` (text after a model
  turn with no tool result: human);
- local commands that reach no model (`/compact` before it runs, shell escapes): `NotRecorded`;
- compaction as an event: the wire infers it when a chain restarts with a summary (`Inferred`);
- working directory and background flag of a tool call: `NotRecorded`.

Only the wire sees: the call id, route, upstream, attempts and retries, queue wait, time to first
token, declared-price cost, the gateway's refusals, client disconnects. These are new optional
fields on `RecordModelTurn` (section 6), absent in records from disk.

### Joining both

When both sources saw a session, records join on the response message id, which both hold. That
works for Anthropic traffic (`msg_…`; Ollama's messages wire answers `msg_…` too). The chat wire
answers `chatcmpl-…` and responses `resp_…`; a harness transcript of those wires is joined on the
same id where the transcript keeps it. The disk record keeps decisions, authors and sub-agent
links; the wire record adds route, upstream and cost. Sessions join on the session id hash
(section 5), which sessionspec computes from the transcript with the gateway's function.

## 3. Store: an append-only stream per session

The unit of storage is the session's ordered stream of IR commands, each with its instant, its
actor and the source that appended it. Events are what the generated component publishes when the
stream is replayed; views are projections of the events. A record is never updated: a late fact
(a tool result after a crash) is a new command.

Two backends behind one trait in `sessionspec-records`:

- first: SQLite, one file. sessionspec writes `~/.local/share/sessionspec/sessions.db` today
  (`sessionspec sync`, being built); a local gateway writes the same schema to a path in its TOML
  document. The append runs on a writer thread behind a bounded queue, never on the connection's
  thread.
- later: an Eventlog stream per session, and an entity-runtime projection for the views. A gateway
  in Kubernetes needs this one: a SQLite file there is per pod. Same trait, same records.

## 4. One view

The UI is an ESS UI document, `ui/sessionspec.ui.yaml` (`ess-ui/1`), over the views of the domain
`sessionspec.view`: a session list (recent and active sessions, filters by time, project, model,
tool), one session (metrics, cost over time, one card per agent, event lanes, event log) and embed
pages (`/embed/session/<id>`) that show only the session view.

- sessionspec's static build and `sessionspec serve` render it from the local store.
- The shared part is the view model, not the renderer. The gateway's dashboard writes its own Vue
  components (session list, session view) over TypeScript types generated from `sessionspec.view`,
  the split it keeps today (generated `snapshot.ts`, hand-written `.vue`). The page stays one file embedded in the gateway, with no second
  bundle.
- Session-level KPIs show in their own section beside the per-call KPIs; the page shows what the
  stream carries and computes nothing.
- Live: the channels `sessions` and `session_events` carry `SessionUpdated`, `AgentUpdated` and
  `EventAppended` as server-sent events, one per appended record. The gateway's existing snapshot
  stream stays as it is; the token-flow animation keeps reading calls, tokens and routes.

## 5. Content and identifiers

- Default: no message text. Prompts, tool inputs and outputs are a digest and a byte count, as the
  gateway's dashboard keeps them today and as sessionspec's redacted IR does. Storing text is a later
  setting per store, off by default.
- The session id leaves a source only as the gateway's hash: SHA-256 of
  `llm-gateway.session/1\0<source>\0<header value>`, its first 6 bytes as 12 hex digits. sessionspec computes
  the same hash from a transcript to join, so no raw id is needed. The dashboard shows the hash.

## 6. Joining the gateway's existing domains

- The interaction record carries the gateway's call id (`x-llm-gateway-call`; `GET /v1/calls/{id}`
  reads the call). That is its pointer to the call record, so the gateway's records gain no key
  field. The call record stays the gateway's own.
- `llm-gateway.telemetry` gains a finished-call value (the usage record's cost and the dashboard
  call's attempts, queue wait and time to first token in one value) and the port that receives it
  with the raw request bytes and the inbound wire. Specification first, in
  `spec/domains/telemetry.yaml`. An added field of a public Rust struct breaks a caller that builds
  it by literal: the gateway's version takes a minor step and its CHANGELOG says so.
- `llm-gateway.upstream` and the routes give the route and upstream names the record carries.
- `RecordModelTurn` gains optional fields for call id, route, upstream, attempts, queue wait, time
  to first token, declared-price cost and an incomplete flag (in sessionspec's specification;
  absent from disk records).
- KPIs: per call figures stay in the gateway's telemetry. Per session figures (autonomy, cache
  busts, waiting on the human, cost per session) are computed once, by `sessionspec-records`, over
  records from either source; the gateway's dashboard shows them from that library.
- The gateway crate `crates/llm-gateway` declares no dependency (its dependency-boundary test), so
  `sessionspec-records` links into the binary crate `crates/llm-gateway-cli`, behind a port the
  gateway crate declares, as usage records and call pricing do (`metrics.rs`). The port is called
  on the connection's thread and returns at once.
- The dashboard's only ratchet is its UI loop's baseline (layout, accessibility, keyboard, reading
  load). A ratchet on gateway or session numbers would be new; it is not part of this proposal.

## Order of work, after acceptance

1. sessionspec: the store and `sync` (in progress), then split `sessionspec-records` out of the
   binary crate.
2. sessionspec: the wire lift as a pure function from (request bytes, answer events, gateway facts)
   to IR commands, tested on recorded, synthetic calls.
3. llm-gateway, specification first and red tests first, in one wave: the finished-call value and
   the port in `telemetry.yaml`, the answer tap for message id and tool-use blocks, the adapter in
   `llm-gateway-cli` that calls the wire lift and appends on a writer thread, the two channels.
4. Dashboard: render the session list and the session view from the records.

## Built (steps 1 and 2)

- The crate `crates/sessionspec-records` holds the sources, the store, replay, the view
  projection, the KPIs and the sq engine; the `sessionspec` binary is the command line over it.
- `sessionspec.session` gained: `Source` (Disk, Wire) and `session_hash` on `OpenSession`; the
  agent role `SideCall`; on `RecordModelTurn` the optional `message_id`, `call_id`, `route`,
  `upstream`, `attempts`, `queue_wait_ms`, `ttft_ms`, `declared_cost_microdollars` and
  `incomplete` (`IncompleteReason`: client_disconnected, upstream_cut, wire_error, refused).
  Transcripts fill `message_id` and the session hash; the rest is absent on disk.
- The API a gateway adapter calls, per session hash and per finished call:

  ```rust
  let mut session = sessionspec_records::wire::WireSession::new();          // one per session hash
  let steps = sessionspec_records::wire::lift_call(&mut session, &call);    // call: WireCall
  let (appender, writer) = sessionspec_records::append::Appender::spawn(&db_path, capacity)?;
  appender.append(&call.facts.session_hash, call.facts.harness, steps);     // never blocks
  ```

  `WireCall { wire: Wire::{Messages, Chat, Responses}, request: raw bytes, answer:
  Answer::{Body(bytes), Events(Vec<bytes>)}, facts: GatewayFacts { call_id, session_hash, harness,
  path, route, upstream, attempts, queue_wait_ms, ttft_ms, declared_cost_microdollars, incomplete,
  started_ms, finished_ms } }`. `Events` holds the `data:` payload of each server-sent event.
  `wire::session_hash(source, value)` is the gateway's function; `wire::join(disk, wire)` copies the
  gateway's facts onto the transcript's model turns with the same message id.
- Store: a wire session is the stream `wire_steps` under the key `wire:<session hash>`. The
  writer thread keeps the projection of up to 64 open sessions and extends it by each batch's
  steps: new event rows as a new chunk, the session's and agents' rows and KPIs as they now stand.
  The check verdicts run over the whole session when the queue is empty for 500 ms, and on an
  append once they lag 30 s. A session not kept, or a batch earlier than the session's first
  instant, is projected again from every stored step. Either way the rows equal a projection of
  the whole session (tests `long::*` in `crates/sessionspec-records/tests/main.rs`).

Not built yet, from § 2: a compaction inferred from a chain that restarts with a summary (it
starts a new agent instead); redaction findings on the wire (`RecordExposure`); two agents whose
first user messages are identical share one chain.
