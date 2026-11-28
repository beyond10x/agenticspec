# Events and storage: an event bus and stores as building blocks

Status: concept, not planned, nothing built. Each decision is numbered (E1-E10), each open point
(P1-P9) names the option recommended. Line numbers refer to this branch unless a branch is named.

Two ideas are missing from the blocks. Parts emit events and other parts subscribe to them, and a
subscription with a condition starts an action, such as storing a record. And data has a place
to live: records, artifacts, key-value pairs, semantic items and graph facts, in a store whose
scope may be one part, the whole architecture, or something outside it.

## What exists, and where the gap is

| concept | where | what it covers | what it lacks |
|---|---|---|---|
| reaction to an event | `trigger` `TriggerKind` `OnEvent` with `event_name` (`blocks/7-collaboration/trigger.yaml:21-27`, `:101-103`) | a named block event starts something | the only action is waking an agent: `agent_revision_id` is required (`:104-105`). `condition` and `intake` are text with no grammar (`:116-129`, gap G15). A duplicate firing per `subject_key` is not refused (`:179-184`) |
| firing on events in the simulator | branch `consumer/w-sim-fidelity`, `crates/agenticspec-sim/src/sim/swarm.rs:73-81` (8 events), `:109-112` (`fires_on`, optional `source: <part>`), `src/sim.rs:973-984` | `on_event` triggers fire on 8 events at the instant they happen. On this branch they are still "checked, never fired" (`design/building-blocks.md:630`) | no condition, no delay, no loss, no action other than a run |
| publish and subscribe | `channel` `Distribution` `Topic`, `Queue`, `Board` (`blocks/7-collaboration/channel.yaml:52-59`), `Subscription` with filters (`:250-300`), `ordered` and `may_lose` (`:136-142`), actor `Transport` (`:390-396`) | fan-out, competing consumers, filters, declared order and loss | parties are a principal or a harness session only (`PartyKind`, `:32-34`). A post is a message with content, not a typed block event. Per-subscriber delivery is not modelled (`:302-309`) |
| the event log | `design/architecture-runtime.md:125` (a conformant runtime emits the events of the blocks it runs); `design/view-layer.md:167` (R2: resuming from cursor c yields exactly the events after c) | every event of every block, resumable | it is stated for a UI consumer only; nothing else subscribes to it |
| in-spec causation | `design/architecture-runtime.md:57` (an ESS binding is an event-to-command reaction) | causation that always holds, for every architecture | it cannot be configured per architecture; `blocks/` declares no binding yet (`:69`) |
| curated knowledge | `knowledge` (`blocks/3-capability/knowledge.yaml:3-9`, kinds `:15-22`) | what an agent is given before it acts, as digests; `Memory` is one kind | no general data, no query, no store it lives in |
| deliverables and evidence | `work` `DeliverableKind` `Proposal`, `Artifact` (`blocks/5-work/work.yaml:49-53`); `Deliverable.evidence`, a list of digests (`:265-267`) | an artifact is named and accepted only with evidence | the digests resolve to nothing; an artifact has no location, provenance or reader |
| handback | `workspace` `Proposal` with `branch` and `review_ref` (`blocks/8-place/workspace.yaml:128-136`) | a change leaves a workspace | field vocabulary; `design/fields.md:72-73` already plans one neutral `handback` reference |
| evidence as a part | `architectures/ladder/08-live-loop.yaml:40`, `09-factory.yaml:40`: `evidence: { block: work, kind: evidence }`, written to over `carries: record` (`09-factory.yaml:95`, `:103`, `:106`) | the architectures already need a store | `work` is borrowed for it; `Carries` has no `Record` on this branch (`blocks/9-composition/architecture.yaml:41-43`, 9 variants); PR #21 adds `Record` and `Read` (branch `consumer/w-arch-refs`, `architecture.yaml:55-60`) |
| measurements | `kpi` `Measurement` (`blocks/10-evaluation/kpi.yaml:132-150`) | a typed, append-only record | it is the block's own state, not a store a part writes |
| a cache | `resource` `PoolKind` `Cache` (`blocks/8-place/resource.yaml:18-20`) | units held by claims | capacity only, no content |

The gap, in short: the event log exists but only a UI reads it, a trigger can only wake an agent,
and nothing holds data that a part writes and another part reads. Two existing blocks are extended
(`channel`, `trigger`) and one is added (`store`).

## E1 The bus: the runtime's event log, and a channel of kind `Event` when it needs a name

- **Implicit, always there.** Every admitted architecture has one event log: the runtime's. It
  already holds every block event (`design/architecture-runtime.md:125`). A subscription names an
  event (and optionally its source part) and reads this log. The common case needs no part.
- **Explicit, when it matters.** A `channel` part with the new kind `Event` and `distribution:
  topic` (every subscriber gets each event) or `queue` (one subscriber takes it) is a named bus. It
  is used when events cross an architecture boundary (a nested architecture, or a bus outside the
  architecture), when the bus has settings worth simulating or varying (delay, loss, retention),
  or when the reach from emitter to subscriber is to be checked by links. A link `carries: event`
  from a part to the bus is an emit; a link from the bus to a subscriber is a delivery.
- Why extend `channel` and not add a bus block: fan-out (`Topic`), competing consumers (`Queue`),
  filters on a `Subscription`, declared order (`ordered`) and loss (`may_lose`), and a `Transport`
  that reports delivery or loss all exist (`channel.yaml:52-59`, `:136-142`, `:250-300`,
  `:390-396`). What changes:
  - `ChannelKind` gets `Event`;
  - a post on an event channel carries the event's identity (E3) instead of a message's act and
    proof;
  - an actor `Publisher` (the runtime) posts on it, so no party sends;
  - the party of a subscription on it is the principal that owns the reaction. That principal
    may be of kind `Service` (`blocks/0-foundation/shared.yaml:96-98`), so `PartyKind` needs no new
    variant.

## E2 Subscription with a condition, and its action: a trigger with a reaction

- A subscription with an action is a **trigger**. `OnEvent` exists; what changes is what it
  does. `Trigger` gets a reaction kind:
  - `Wake`: today's behaviour, `agent_revision_id` then becomes optional;
  - `Command`: a named command of the blocks (text, as `event_name` is), with `intake` as the
    mapping from the event's fields to the command's input. Example: on
    `agentic.work.DeliverableAccepted` where the deliverable's kind is `Artifact`, issue
    `agentic.store.PutRecord` into the architecture's evidence store.
- `ConcurrencyPolicy` (`Skip`, `Queue`, `Parallel`, `trigger.yaml:32-34`) stays the back-pressure
  of a slow reaction. `Firing` stays the record of each reaction.
- Relation to ports: a `Command` reaction issues an intent command. If that command opens a port
  (`PutRecord` opens `store.put`, E8), the trigger causes the port's intent and the world reports
  as for any other caller. The trigger itself is internal behaviour.
- Relation to ESS bindings: a binding is causation that holds in every architecture and is part
  of the specification; a trigger is causation one architecture configures, and a datum. A
  reaction that every architecture needs belongs in a binding, not in a trigger.
- The condition is a predicate over the event's fields, in the predicate form the blocks already
  use in guards (`all`, `any`, comparisons, `defined`; e.g. `trigger.yaml:551-557`). It reads no
  other state (P3).

## E3 Delivery semantics

- **Identity.** An event in the log has a sequence number (the cursor), its qualified name, its
  source (architecture, part path, instance, as `Placement` in `design/view-layer.md:283-292`
  gives it), its subject key (149 of 152 events carry their view's key,
  `design/view-layer.md:24-26`), its time and the digest of its payload.
- **At least once.** Each subscription receives every matching event at least once. A reaction
  is deduplicated by (subscription, sequence), and a `Command` reaction by a key the command takes
  (`subject_key` exists on `Firing`; its refusal of duplicates is the unwritten guard of G15).
- **Order per source.** Events of one source and one subject arrive in the order they were
  emitted. No order is promised across sources.
- **Resume.** A subscription keeps a cursor. Resuming from cursor c yields exactly the events after
  c: the obligation R2 already states (`design/view-layer.md:167`). A UI feed and a trigger are two
  consumers of one obligation.
- **Retention.** The log keeps events for a declared time. A cursor older than that is refused by
  name (`CursorExpired`), never skipped silently; the consumer then rereads state through views.
- **Loss.** The implicit log loses nothing within retention. An explicit event channel may declare
  `may_lose`, for faults in simulation or for a bus that loses events in fact.

## E4 One `store` block with five kinds

A store holds data a part writes and another part reads. The blocks record that a write happened,
its key, its digest and who wrote it, never the bytes (as `knowledge` holds content as a digest,
`knowledge.yaml:3-9`, and `agentic.shared.Content` is digest, length and class,
`shared.yaml:121-128`). The bytes live behind the port.

| kind | holds | written by | queried by |
|---|---|---|---|
| `record` (the general database) | typed records of a declared record type, each under a key, revised | put (a new revision), delete (a tombstone) | get by key; query by a predicate over fields, with order and limit |
| `artifact` | immutable content with a class, a digest and provenance | put once, addressed by digest; a new version is a new artifact that names the one it derives from | get by id or digest; list by class or provenance |
| `key_value` | an opaque value per key, with an optional expiry | put, delete, compare-and-set | get by key; scan by key prefix |
| `semantic` | items with a vector and a payload | put; the vector is supplied, or made by a model call over the store's own link to a gateway or model | nearest k to a vector or a text, with a filter over the payload |
| `graph` | nodes and typed edges with properties | assert and retract a fact | match a pattern; traverse a path |

- An **artifact is a record**: a record in a store of kind `artifact`, with three invariants. Its
  content is required and never revised. It is addressed by its digest. Its provenance is
  required: `recorded_by` (a principal; agents are principals of kind `Agent`, `shared.yaml:98`)
  and `derived_from` (artifact ids). Creating an artifact is putting a file into an artifact store;
  handing it across a workspace boundary is granting another part read on it (E6).
- Entities: `Store` (kind, scope, retention, record type) and `Record` (store, key, revision,
  content, `recorded_by`, `recorded_at`, `derived_from`, `expires_at`; lifecycle `Present`, then
  `Superseded`, `Expired` or `Deleted`). Events: `RecordPut`, `RecordSuperseded`, `RecordExpired`,
  `RecordDeleted`, which the bus carries like any other event, so a store write can trigger the
  next reaction.
- A semantic store that makes its own vectors needs a model path. The check gets a rule like
  `agent_without_model_path` (`design/building-blocks.md:289`). Its embedding calls pass the
  gateway like every model call.
- The blocks' own state (tasks, firings, measurements) is the runtime's persistence. It is not a
  store part. A store part holds the data of the architecture's work.

## E5 Scope and retention

- **Scope** follows what the architecture format already has:
  - `part`: `per: <part>`, one store in each instance of that part (an agent's memory), as
    `tree: { per: worker }`;
  - `architecture` (the default): one store shared by every part that links to it; a nested
    architecture reaches its parent's store only over a link;
  - `external`: `scope: external`, a store that outlives the architecture and is shared beyond it,
    for a cluster or globally. The architecture declares it as a part. Which store and where is
    the binding's concern. Admission refuses an external store the binding does not bind
    (`design/architecture-runtime.md:118`).
- **Retention**: `retention: <duration> | run | forever`, with the duration format the
  architecture format already uses (`design/building-blocks.md:238`). `run` ends with the
  instance of the architecture. Expiry is a clock port (E8).
- **Deletion** of a record that is not expired is a decision that needs approval. Deleting data is
  irreversible, so its default grant effect is `ApprovalRequired`
  (`blocks/6-governance/authority.yaml:79-84`).

## E6 Who may read and write

Three checks, each from an existing block:

1. **Reach** (architecture): no link, no reach. `carries: record` from a part to a store allows
   writes; `carries: read` allows reads (both added by PR #21). Any other access is refused by
   the check.
2. **Grant** (authority): a `Grant` per action class on a scope (`authority.yaml:54-84`). The
   scope is a store, a key prefix or a record type. `action_class` is a `ToolCategory` today
   (`:69-70`); store read, write and delete become rows of the planned `ActionClass` catalog
   (`design/view-layer.md:297-301`).
3. **Egress** (sandbox): an `external` store lies outside the architecture, so the part's sandbox
   must name it in `egress` (`blocks/8-place/sandbox.yaml:57`), as `box: { egress: [gateway] }`
   names the gateway.

An agent reaches a store through a tool whose provider is the store. The routing invariant has
the same form as `worker -> gateway` (`design/architecture-runtime.md:102-103`): every `store.put`
of an agent part happens inside one of its `tool.invoke`.

## E7 Relation to knowledge, workspace, deliverables and evidence

- **knowledge** stays what an agent is given before it acts, curated and published. A knowledge
  item may name the store its content is drawn from (a `Memory` item drawn from a part-scoped
  semantic store). The edge runs from knowledge to store, never back, so the two layer-3 domains
  form no cycle.
- **workspace**: the neutral `handback` reference of `design/fields.md:72-73` becomes an artifact
  id. A change handed back across a workspace boundary is an artifact; `branch` and `review_ref`
  become rows of the source-control pack.
- **work**: `Deliverable` of kind `Artifact` names its artifact id, and `evidence` (`work.yaml:265-267`)
  names records or artifacts rather than bare digests. Acceptance can then check that the evidence
  exists and that its `recorded_by` is not the submitter alone.
- **evidence and KPIs**: the ladder's `evidence: { block: work, kind: evidence }` becomes
  `{ block: store, kind: record }`. A KPI may read a store's events like any block's events;
  `Measurement` stays in `kpi`.

## E8 Ports

Storing and publishing are IO, so by the derivation rule (`design/architecture-runtime.md:54-67`)
they are ports: the intent leaves its entity waiting, and only the world's report moves it on.

| port | block | protocol (intent, then reports) |
|---|---|---|
| `store.put` | store | `PutRecord (ConfirmPut \| FailPut)` |
| `store.get` | store | `GetRecord (ReturnRecord \| FailGet)` |
| `store.query` | store | `QueryStore (AnswerQuery \| FailQuery)` (query and answer as digests, with a count) |
| `store.expire` | store | clock port: `ExpireRecord` at `expires_at` |
| `bus.publish` | channel (kind `Event`) | `PublishEvent (AckPublish \| FailPublish)`, only for an explicit bus bound outside the runtime |
| `bus.deliver` | channel (kind `Event`) | `DeliverEvent (AckDelivery \| FailDelivery)`, per subscription, at least once |

- The implicit log has no `bus.publish`: emitting is the runtime's own act
  (`design/architecture-runtime.md:125`). `bus.deliver` is the same shape as `channel.deliver`
  (`:82`).
- Settings narrow them in the contract: `kind`, `scope`, `retention`, `distribution`, `may_lose`.
- **What a binding supplies**, per store part: the kind of service that plays it (in memory, a
  directory of files, a database service, an object store, a vector index, a graph service: kinds,
  never products), where it lives, its credentials through a sandbox secret grant, the mapping of
  record types to its schema, and for a semantic store the route that makes vectors. Per explicit
  bus: the transport, retention and where cursors are kept.
- **Fixtures** are capabilities (`design/architecture-runtime.md:136-141`): an in-memory store and a
  store replayed from a recording serve tests and the first runtime, with no test-only code path.
- **Simulator**: a put, get or query is a delay and a cost drawn from the profile. A delivery is a
  delay, by default 0, and a loss rate under a fault (a channel that drops a message is already a
  fault, `design/building-blocks.md:615`). The store events join the simulator's list of events, so
  `on_event` can fire on them. A `Command` reaction is a delay and a store write counted per
  store. A default of 0 keeps today's results byte for byte.

## E9 Field neutrality

- The names are neutral: event, bus, store, record, artifact, key-value, semantic, graph. No
  product, service or vendor appears in a block, a part kind or a setting. A concrete service is a
  binding.
- A field brings data, not semantics (`design/fields.md:42-45`): its record types, its artifact
  classes (values of `Content.class`) and its conditions and reactions, as rows and documents.
  A coding field registers a record type for a test report and an artifact class for a patch; no
  core block names either.
- Two leaks this concept touches and removes: `Proposal.branch` and `review_ref` (E7), and
  `KnowledgeScope` `Repository` (`knowledge.yaml:24-30`), which is a source-control term. Its
  neutral form is a scope that names a store or a workspace.

## E10 Layers

| change | layer | depends on |
|---|---|---|
| new domain `store` (`blocks/3-capability/store.yaml`); `StoreId`, `RecordId` in `agentic.shared` | 3 capability, beside `tool` and `knowledge` | `shared` (0), `principal` (1, `recorded_by`) |
| `knowledge` item names a store | 3 | `store` (3), one direction |
| `work` `Deliverable.artifact_id`, evidence as record ids | 5 | `store` (3) |
| `channel` kind `Event`, `Publisher`, event identity on a post | 7 | unchanged |
| `trigger` reaction kind `Wake` or `Command` | 7 | `channel` (7, already), command names as text |
| `workspace` handback as an artifact id | 8 | `store` (3) |
| `architecture` `PartKind` `Store`, `Carries` `Event` (with PR #21's `Record`, `Read`) | 9 | text settings, as today |

Layer 3 is the lowest layer from which every user of a store can name it: deliverables (5),
handback (8) and triggers (7). In layer 8, beside `workspace`, a deliverable could not name its
artifact. Provenance points down, to a principal and to other artifacts. The run that produced
an artifact names it from above (the turn-around rule of `AGENTS.md`).

## Open points

| # | question | options | recommended |
|---|---|---|---|
| P1 | what the bus is | A: the runtime's implicit log, plus a `channel` of kind `Event` when it needs a name (E1). B: a new `bus` block and part kind. C: the runtime's log only, no part | A |
| P2 | where the action of a subscription lives | A: `trigger` gets a reaction kind `Wake` or `Command` (E2). B: a new `reaction` domain beside `trigger`. C: ESS bindings only, the same in every architecture | A |
| P3 | what a condition reads | A: the event's fields only, in the guard predicate form; an event that lacks a field gets it. B: the event plus its subject's view row at delivery. C: the query language of `queries/*.sq` | A |
| P4 | where storage sits | A: one `store` domain in layer 3 with five kinds. B: in layer 8 beside `workspace` (a deliverable could not name its artifact). C: a record noun in layer 3 and a store place in layer 8 | A |
| P5 | the artifact | A: a record of a store of kind `artifact`, with immutability, digest address and provenance as invariants. B: its own entity or domain | A |
| P6 | knowledge and stores | A: separate; a knowledge item may name the store it draws from. B: knowledge becomes a store kind. C: merge both | A |
| P7 | are reads recorded | A: `store.get` and `store.query` are ports recorded with digests and counts (cost, latency and audit). B: only writes are recorded | A |
| P8 | stores beyond the architecture | A: `scope: external`, a part whose binding is required at admission. B: outside the architecture altogether, reached as a tool | A |
| P9 | the link value for events | A: one `Carries` `Event`; direction tells emit from delivery. B: two values, `Emit` and `Subscribe` | A |

## Proposed stories

1. **store-block**: `blocks/3-capability/store.yaml` with `Store`, `Record`, the five kinds,
   put, get, query, expire and delete, events, views, ids in `agentic.shared`, and authored
   scenarios. `task check:layers` and `task check:blocks` pass.
2. **artifact-references**: `Deliverable.artifact_id`, evidence as record ids, and the workspace
   handback as an artifact id. This takes `design/fields.md` step 3 for the handback.
3. **event-channel**: `channel` kind `Event`, actor `Publisher`, event identity on a post, and
   `CursorExpired`.
4. **trigger-reactions**: reaction kind `Wake` or `Command`, the condition as a guard predicate,
   `intake` as a field mapping, and the duplicate refusal per subject key (closes G15's unmapped
   rest).
5. **architecture-store-and-bus**: `PartKind` `Store`, `Carries` `Event`, settings `kind`,
   `scope`, `retention`, the check rules (reach for store and bus links, a semantic store's model
   path, an external store's egress), and docs.
6. **store-and-bus-ports**: the six port rows join the expected table of
   `story:port-derivation`, and their narrowing settings join `story:contract-derivation`.
7. **bus-delivery-conformance**: at-least-once, order per source, resume from a cursor and an
   expired cursor as conformance obligations, shared with the R2 feed of
   `design/view-layer.md` S6.
8. **sim-store-and-bus**: delays and costs for store ports, delivery delay and loss, store events
   in the simulator's event list, and `Command` reactions. Results stay byte for byte at the
   defaults.
9. **evidence-stores**: the ladder architectures move `evidence` from `block: work` to
   `block: store, kind: record`.
10. **binding-store-and-bus**: what `agenticspec-binding/1` says for store and bus parts, and the
    in-memory and replay fixtures.

Every story that runs tests keeps each test within 60 s and its step within 120 s. A job in the
build pool ends within 10 minutes.
