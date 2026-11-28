# The view layer: an operator UI derived from an architecture

Status: design for approval, nothing built. Each decision is numbered (D1-D11) and each open point
(O1-O9) names the option recommended, so the owner can approve or change them one at a time.
Counts were measured at commit `310373a` with `ess` 0.99.2 (Appendix B shows how).

The owner, 2026-11-28: one tree-like artifact that may also provide a view; a default UI for
operators, generated from the components inside the architecture and their grouping, provided
those components expose data, views or events. agenticspec builds the reusable view components
and the generator. agenticstudio renders the same components. A consumer builds its own app, places
the components in it and adjusts them a little.

## What exists

- An architecture document (`agenticspec-architecture/1`, `architectures/*.yaml`): parts by block,
  links, `border`, `policies`. The 10 documents have at most 10 parts and 10 links
  (`author-reviewer.yaml`, `scheduled-loops.yaml`). Only 2 declare a `border` (`workstation.yaml`,
  `author-reviewer.yaml`). A consumer reports architectures of 37-42 parts and 63-95 links.
- `blocks/`, the ESS system `agentic`: 22 domains, 21 of which declare views. 67 views, 152
  events, 155 commands, 60 entities, 30 actors. 108 commands appear in some actor's `may`.
  0 components and 0 bindings. A consumer counted 65 views, 150 events and 153 commands. The
  difference is commit `310373a`, which added `Ports`, `Contracts`, `DeclarePort`,
  `RecordContract`, `PortDeclared` and `ContractRecorded`.
- Every one of the 152 events is emitted by an outcome that names a subject entity. For 149 of
  them that entity is the source of a view, and each of those 149 events carries the view's key
  (its first field, `<entity>_id` in all 67 views). Only 3 carry every field of the view. The 3
  events whose entity has no view are `ContextCompacted`, `WorkModeCreated` and
  `ProviderRegistered`.
- Only 13 of the 67 views declare a parameter. The other 54 take none.
- `ui/agenticspec.ui.yaml` is the one hand-written `ess-ui/1` document. It has two SSE channels,
  resumed with `Last-Event-ID`, which `agenticspec serve` serves under `/view/views/...` and
  `/view/channels/...`. It also has `embed.*` pages in a shell with nothing but the page, for a
  host that shows one session.
- The work in progress on `story:ui-generator` (`crates/agenticspec-ui`, `ui/kit/`):
  - `agenticspec ui generate` writes Vue 3 SFCs from `ess-ui/1`. It draws 5 components
    (`collection`, `record`, `form`, `filter_bar`, `choice`) and host slots.
  - Its typed client calls `GET /api/views/<view>`, `POST /api/commands/<command>` (a refusal is a
    422 with its row) and `GET /api/channels/<channel>` (SSE, events named by the event's
    qualified name). `createClient(base)` takes a base URL.
  - `ui/kit/` has 30 components. Its tokens are CSS custom properties named `--kit-*`, and
    `KitZoomCanvas` provides pan and zoom.
- `studio/studio.ui.yaml` draws its graph in the host slot `DraftGraph`.
- Planned, not built: ports, contract and border (`epic:architecture-contract`,
  `story:contract-derivation`), the architecture as a part `{ block: architecture, ref, digest }`
  (`story:architecture-references`), binding, admission and the runtime
  (`design/architecture-runtime.md`, no story yet), governance types (`story:governance-types`).

## D1 Where it lives in the one artifact

The tree is the architecture document together with the documents it references by path and
digest. The view layer has two halves:

- **Authored:** an optional top-level `view:` key in the architecture document itself (D4, O1),
  holding presentation only. This makes the tree the one artifact that also provides its view.
- **Derived:** the generated `ess-ui/1` document, never edited by hand. Beside it,
  `sources.sha256` holds the digest of its inputs: the architecture document, every referenced
  document, `blocks/ports.json` and the generator's version. This is the pattern of
  `website/static/catalog/*/sources.sha256`. A fast check compares the digests. A slow lane
  regenerates and compares bytes.

The `view:` key breaks no pin. The nested-architecture work (`crates/agenticspec-sim/src/nest.rs`,
`DIGESTED = ["border", "links", "parts"]`) pins a referenced document by the SHA-256 of the
canonical JSON of its `parts`, `links` and `border` only, as `agentic.architecture.Digest` says.
`view:` therefore sits outside the digest, as `summary` and `policies` already do. It stays
portable: it names parts by path and never names a host, a product or a renderer.

The inline key needs two changes, owned by S4:

- The catalog's `sources.sha256` (`crates/agenticspec-docs/src/results.rs`) digests the bytes of
  the architecture file, so a label edit in `architectures/*.yaml` would force
  `task catalog:generate`. The catalog has to digest the document without `view:`, for example
  through the canonical form.
- The `TOP` key order in `crates/agenticspec-sim/src/best.rs`, which writes the best variant as a
  document, needs `view` so the key survives a rewrite.

## D2 Two generated products

1. **Block panels**: one panel per view of a block domain, generated from the compiled
   `blocks/` alone. The output is `ui/views/blocks.ui.yaml`, one `ess-ui/1` widget per view plus a
   gallery page per domain, so `ess ui check --model blocks` holds every panel. A panel takes a
   scope (architecture, part path, instance) and nothing else. Its columns come from the view's
   fields, its labels from ESS `naming.display`, its row key from the view's first field, and
   `state` is drawn as a badge. These panels are the reusable components. They do not depend on
   any architecture.
2. **Composition**: one `ess-ui/1` document per architecture. It holds the overview, the
   navigation, the pages, the order of the panels and the actions, derived from the document, its
   contract and its `view:` key. `ess-ui/1` has no import between documents, so the derivation
   copies the widgets it uses from `blocks.ui.yaml` byte for byte. `check:generated` holds that
   the copies are equal. Because the Vue emitter writes one SFC per widget, the same panel gives
   the same file in every architecture.

## D3 The derivation rule

| architecture element | UI element |
|---|---|
| top document | the app: navigation home is its overview page |
| nested architecture part `{ block: architecture, ref, digest }` | one overview page of the referenced document and one navigation section, recursively. In the parent it is one row (and one box in the graph) that opens that page. With `count` > 1, the page takes an `instance` parameter |
| overview page of one level | the parts table (one row per part, nested architectures first, then by block kind, then by name), a reach table (`agentic.architecture.Reach`, which takes `architecture_id`), and the one-level graph (D9) |
| part | one part page. Its sections are the panels of the views of every domain the part's ports use (from its contract) plus its own block's domain, all scoped to the part |
| part with `count` or `per` | the first section of its page lists the instances (the entity of its first port: harness sessions of an agent part, workspaces of a `per: worker` tree). Parts that are `per` it are shown inside each instance's row |
| link | a row in the reach table. A link becomes an action only under D6 |
| block events | one channel per architecture level (D5) |
| policies, marked places | read-only on the overview |

The domains of a part come from its contract, not from the name of its block. For example, an
`agent` part's sessions, turns and tool calls are views of `harness`, not of `agent`. With the
block name alone, an agent part would show only `Agents` and `AgentRevisions`. The table from
document block to domain belongs to `story:contract-derivation`. The view layer reads it and owns
no copy of it.

On the part page, the instance list comes first. The other panels follow in collapsed frames, one
per domain (`frame: shared, collapsed: true` of `ess-ui/1`).

## D4 Generated and authored

Everything is generated except the architecture document's `view:` key, which is limited to
presentation. It may hold: `order` of parts and panels, `labels` (by part path, panel, column),
`hide` (panels, columns, actions), `groups` (display groups on the overview, labels only), and
`home`. It never holds a read, a filter, a field list or a command. When an entry names a part,
panel or column that does not exist, the derivation refuses it and names the entry's node path.
The derivation applies the key, so every packaging of the UI shows it (D8). The key of a nested
document applies on that document's page. The key has no format of its own: the
`agenticspec-architecture/1` reference documents it.

## D5 Live data

- One channel per architecture level: `carries` lists the events of the domains on that page,
  with `direction: server_to_client`, `delivery: every_event`, `resume: from_last_seen`,
  `scope: param`, and `session: {per: architecture_id}`. A probe document with this shape passed
  `ess ui check --model blocks` except for the missing component (see "What ess-ui/1 cannot
  express").
- Each panel's `live` is derived from the compiled model. The events whose outcome subject is the
  view's source entity (149 of 152) drive `effect: refetch, coalesce: 1s` on the panel. Only the
  3 events that carry every field of the view use `insert_or_patch`, matched on the key. No panel
  polls.

## D6 Actions

The consumer's rule, "each link from a border part is one command action", is not taken.
Following it, `workstation.yaml` would turn `operator -> gateway carries model_call` into an
operator button, and 8 of the 10 documents would have no actions at all, since they declare no
border. Taken instead:

1. **Principal ports** of the contract become row actions where the human acts.
   `principal.answer` becomes `Answer` on `OpenQuestions` rows. `principal.decide` becomes
   `ApproveDecision`/`RejectDecision` on `OpenApprovals` rows.
2. **Inputs into the border** become header actions on the border part's page. These are links
   whose `from` is a principal part or lies outside the architecture and whose `to` is on the
   border. Each one takes the intent command of the port its carried value uses (prompt over
   `chat` is `SendMessage`). Machine traffic (`model_call`, `tool_call`, `claim`) is never an
   action.
3. Every action is visible only when the viewer's action set (M47) allows it at that scope. The
   runtime still refuses the call. Hiding is a convenience, not authorization.

The generator holds no command name. It reads commands from `blocks/ports.json` and the
contract.

## D7 The runtime interface: one runtime, the UI is one more consumer

The UI talks to the adapter of the runtime that admitted the architecture, the same adapter block
conformance drives. There is no second server and no view that exists only for the UI. Paths are
the generated client's (`/api/...`):

| request | wire | conformance relation |
|---|---|---|
| R1 read | `GET {source}/api/views/{view}?{params}` returns `{"rows": [...]}` | the QueryView request of `ess-conformance-target/1`, over HTTP |
| R2 feed | `GET {source}/api/channels/{channel}?{params}`, SSE: `event:` qualified name, `data:` event JSON, `id:` cursor. A reconnect with `Last-Event-ID` gets what followed | new obligation: resuming from cursor c yields exactly the events after c (the wire of `agenticspec serve` today) |
| R3 placements | R1 on `agentic.architecture.Placements` (new, M45), plus `Parts`, `Reach`, `Revisions`, `Contracts` | block conformance of the new view |
| R4 command | `POST {source}/api/commands/{command}` returns `{"row"}`, or 422 with the refusal's row | the ExecuteCommand request |
| R5 action set | `GET {source}/api/actions` returns `[{command, allowed, grant_id?}]` for the signed-in principal at that scope | new, M47 |

`{source}` is `{root}/scopes/{architecture_id}[/{part path}]`. The part path follows
`story:architecture-references` (`team.lead`), with an instance index for a part with `count`.
Putting the scope in the base path changes no view parameter and no generated client, since
`createClient(base)` already exists. The runtime answers a scoped read with the in-scope rows
under the rule of M45. The adapter authenticates the caller before it dispatches, as the note on
`agenticspec serve` in `ui/README.md` asks of any host serving more than its own user.

Fixtures are one more source, as `design/architecture-runtime.md` treats fixtures as capabilities.
`ess-ui/1` fixtures (a file per view, a script per channel) answer R1 and R2 in tests and in the
gallery. A scenario's expected events (`agenticspec-scenario/1`) can later replay as a channel
script.

## D8 Pipeline and reuse of `agenticspec ui generate` and the kit

`blocks/` goes through `agenticspec ui panels` to `ui/views/blocks.ui.yaml`. The architecture,
with its `view:` key, and its contract go through `agenticspec ui derive` to `<name>.ui.yaml`. Both
documents go through `agenticspec ui generate` (D2 copies) to Vue SFCs over `ui/kit/`. The
generator needs these additions:

- widgets emitted as SFCs, one per widget;
- `visible` with `actor.may(<command>)`, answered from R5;
- `metric`, and `live` with `refetch`;
- the `embed` shell;
- a build of the panel SFCs as custom elements (O2).

The architecture graph reuses the studio's `DraftGraph` host component, read-only, on
`KitZoomCanvas`. The ESS React generator is not used.

## D9 Large architectures

- The overview leads with the parts table. A nested architecture is one row, so a 40-part
  architecture split into nested documents reads as a short table per level.
- The graph draws one level, with each nested architecture as one box (the rendering
  `story:architecture-references` decides). It is drawn at natural size with pan, zoom and fit,
  by the rule of `story:large-graph-view`. The graph is never the only way to reach a part.
- Links are read in the reach table, filterable by part, rather than drawn as 95 edges.
- Acceptance uses the 37-part fixture of `story:large-graph-view`. At 1366x768 the first screen
  shows the parts table, nothing runs past the viewport horizontally, and every part opens in two
  steps or fewer from the overview.

## D10 Field neutrality

The panel generator reads only the compiled model and names no block, view or command. The
composition names only `agentic.architecture` entities and the contract's ports. A field's own
blocks (`design/fields.md`) get panels by the same rule from their compiled domains, and a field
adds no UI construct. One limit applies: an `ess-ui/1` document names exactly one `model`, and
ESS declares no relation across two systems. Field and pack domains therefore have to compile
into the same system as the core for their panels to sit on one page (O8).

## D11 Grouping

The only structural group is the nested architecture. The overview also groups by part kind and
by `per` chains, both derived, so nothing is authored. The `view:` key may name display groups,
which are labels only. The architecture format gets no new group construct.

## Who provides what

| piece | provided by |
|---|---|
| block panels (one per view), part page kinds, architecture overview, the generators, the kit and its default tokens | agenticspec |
| the same panels inside the draft view (on fixtures or a simulated run), with no copy of any panel | agenticstudio |
| app frame, branding (its own values for the `--kit-*` tokens), navigation, where each component sits, and the `view:` key of its architecture document (order, labels, hidden, groups) | the consumer |
| the adapter that answers R1-R3 (R4 and R5 for actions) at a scope | the consumer's runtime |

**The unit of reuse is the panel**, as defined in D2: a block-view panel, a part page or an
architecture overview, each parameterised by `source`, `architecture`, `part` and `instance`
only. One SFC source is packaged two ways:

1. a Vue package (`ui/views`, npm-style, depending on `ui/kit`) for Vue apps and agenticstudio;
2. custom elements built from the same SFCs (`<agentic-part-panel source=... architecture=...
   part=...>`). These work in any stack: React, plain HTML, a server-rendered page. The `--kit-*`
   custom properties pass into the shadow DOM, so the host brands with CSS alone. The element
   emits `agentic-navigate` and `agentic-command` DOM events, so the host keeps its navigation.

An `ess-ui/1` consumer places a custom element through a host slot whose `reads` and `commands`
declare what the element uses, so `ess ui check` still holds it. The `embed` pages (one route per
panel, inside a frame) remain for a host that cannot load scripts, such as the docs site's frames.

A placed panel needs at least R1, R2 and R3 to show live data. R4 and R5 add actions. Without R5,
every action is shown and refusals come from R4.

## What ess-ui/1 cannot express that this needs

1. **Reuse across documents.** There is no import between documents, so panels are copied by
   generation (D2).
2. **A scope on reads.** A read binds only the parameters its view declares (`read_params`), and
   54 of 67 block views declare none. `reads.filter` runs on the client only (paging `client` or
   `none`), cannot join, and its roots are `row`, `params`, `state`, `shell` and `args`. Scoping
   is therefore the runtime's job (D7), and `ess ui check` does not see it.
3. **Action visibility by data.** `actor.may(<command>)` works per command, for the ESS actor.
   There is no scope argument and no use of authority `Grant` rows.
4. **One model per document** (D10).
5. **A served model.** `view_in_model` and `command_in_model` require a `reached_by: network`
   component. `blocks/` declares 0 components, so every read and command of a probe document was
   an error. The probe reported `coverage: 2 covered, 0 host-covered, 220 uncovered of 222 views
   and commands`.

Channels need nothing new: `carries`, `resume: from_last_seen`, `scope: param` and
`session.per` already exist.

## Verdicts on M43-M49 and grouping

- **M43, no generator from an architecture to `ess-ui/1`: new stories** (S3 panels, S4
  derivation). They build on `story:ui-generator`, which turns `ess-ui/1` into Vue, and planned
  nothing for the step from an architecture to `ess-ui/1`.
- **M44, no binding of a part to a live source: covered by something planned, with one
  addition.** The planned binding and admission (`design/architecture-runtime.md` § "Three
  documents", § "Admission") already say what plays each part. The UI needs one adapter address
  per admitted architecture in the admission plan, not one address per part. The per-part
  mapping is M45. There is no story yet; it belongs in the runtime epic that
  `epic:architecture-contract` defers to.
- **M45, no common key to filter a view to one part: new story (S5).** Measured (Appendix A):
  3 of 67 views carry a `PartId`, 5 carry only an architecture or revision id, and 59 carry
  neither. Of those 59, 43 carry the id of an entity a part stands for, 7 reach one through a
  single reference, 4 carry only a polymorphic party or owner id, and 5 are catalogues. Adding
  `part_id` to block entities is refused: the layer rule lets a block name only its own layer or
  a lower one, and parts are layer 9. Taken instead: `agentic.architecture.Placement`
  (architecture, revision, part path, instance, entity name, entity id), recorded by the runtime
  through `PlaceEntity` and read through `Placements`. A row is in scope when its id, or an id
  field of its view, is placed to the part (O3). The consumer's reading of `Reach` by
  `architecture_id` works for links only, since it is one of the 3 views that carry part keys.
- **M46, no declared event feed: new story (S6), with no ESS change.** The events exist, and 149
  of 152 map to a view source with their key. `ess-ui/1` channels express subscription and
  resume. What is missing is the runtime's R2 with a cursor, and its conformance. The wire is the
  one `agenticspec serve` already uses.
- **M47, no action set derived from grants: new story (S7)**, after `story:governance-types`
  (a decision names the grant it relied on) and a core `ActionClass` catalog (`design/fields.md`,
  not yet a story). Today `Grant.action_class` is a `ToolCategory` (`Read` … `Other`), and the
  scope is text that the evaluator matches. Nothing maps a block command to an action class. R5
  is a query the evaluator answers, not an ESS view.
- **M48, five domains declare no actor: covered by S3.** A consumer's probe found that
  `agent`, `architecture`, `experiment`, `kpi` and `principal` declare no actor, so
  `ess ui check --model blocks` refuses every section over their views (`section_readable`,
  97 errors over two documents). S3's goal of 0 errors includes declaring those actors in the
  blocks; S1's network component removes only the `view_in_model` errors.
- **M49, no block says which command a link's `carries` stands for: covered by D6 and
  `story:port-derivation`.** A carry is not mapped to a command directly: the contract maps a
  link to the port its parts use, and `blocks/ports.json` names each port's intent command.
  Machine carries (`model_call`, `tool_call`, `claim`) derive no action by rule, so a group of
  (carry, target block) with no action is expected for them; any other group without one is a
  gap of `story:contract-derivation`.
- **Grouping beyond the nested architecture: agree that no new construct is needed** (D11).
  Derived grouping by kind and `per` covers the overview, and a level that is still too large is
  a sign to split it into a nested document.

## Open points

| # | question | options | recommended |
|---|---|---|---|
| O1 | where the presentation adjustments live | A: a `view:` key in the architecture document, outside the pin (`nest.rs` digests only `parts`, `links` and `border`). B: a sibling `<name>.view.yaml`, only worth it for presentation over a document the consumer does not own and will not copy; a catalog document is downloaded as a copy, so this case is rare. C: in the consumer's app only | A |
| O2 | the packaging of the panel | A: Vue package plus custom elements from one source. B: `embed` pages in frames only. C: Vue package only | A |
| O3 | the scope rule | A: placements of the entities a part's ports create, plus one hop over the view's id fields. B: a placement for every entity created on a part's behalf. C: part keys in the blocks (breaks the layer rule) | A |
| O4 | the scope on the wire | A: base path `/scopes/{architecture_id}/{part path}`. B: a reserved query parameter. C: a declared `part` parameter on every block view (54 views change) | A |
| O5 | the action set | A: R5 answered by the authority evaluator. B: an ESS view over `Grants` with client-side matching | A |
| O6 | the action sources | A: principal ports plus inputs into the border (D6). B: the consumer's rule, links from a border part | A |
| O7 | channel granularity | A: one per architecture level. B: one per part | A |
| O8 | field domains in one model | A: fields and packs compile into the system `agentic` under their own domains. B: one UI document per system, with panels of different systems on separate pages | A |
| O9 | where the per-architecture UI is generated | A: in the consumer's repository by our CLI, committed with `sources.sha256`; this repository keeps `workstation` (its `view:` key inline in `architectures/workstation.yaml`) and the 37-part fixture as golden examples, with their generated UI under `ui/views/examples/`. B: this repository generates every catalog architecture | A |

## Order of stories

Goal first: the consumer's first gap is M43, so the UI works on fixtures before any runtime
exists.

1. **S1**: add a `reached_by: network` component serving the 22 domains to
   `story:block-bindings`'s `blocks/components.yaml`, which that story already plans. It is an
   addition to that story's scope, not a new story.
2. **S2 `ui-generator-panels`**: the generator additions of D8, after `story:ui-generator`.
3. **S3 `block-panels`**: `agenticspec ui panels`, `ui/views/blocks.ui.yaml`, fixtures and
   gallery, with `ess ui check --model blocks` at 0 errors (M43).
4. **S4 `architecture-ui`**: `agenticspec ui derive`, the `view:` key (with the catalog digest
   and `best.rs` changes of D1), overview, nested
   pages, part pages, channels, actions from ports, and the 37-part acceptance of D9 (M43). After
   `story:contract-derivation` and `story:architecture-references`.
5. **S8 `views-package`**: the Vue package and custom elements. agenticstudio's draft view shows
   the same panels.
6. **S5 `placements`**: `Placement`, `PlaceEntity`, `Placements`, the scope rule, and authored
   scenarios (M45).
7. **S6 `runtime-ui-interface`**: R1-R4 with scope and the R2 cursor, plus conformance of scoped
   reads and feed resume, against fixtures first and then the first runtime (M44, M46).
8. **S7 `action-sets`**: R5 and `actor.may` per scope (M47), after `story:governance-types` and
   the `ActionClass` catalog.

Every story that runs tests keeps each test within 60 s and its step within 120 s. A job in the
build pool ends within 10 minutes.

## Appendix A: view keys of the 22 block domains

The classes are exclusive and checked in order. A = carries a `PartId`. B = an architecture or
revision id. C = the id (field or parameter) of an entity a part kind stands for: principal, agent,
harness, model, route, endpoint, gateway, tool, tool provider, knowledge, channel, workspace,
sandbox, runtime, pool, trigger, schedule. D = only an id one reference away from such an entity
(session, run, inference, assignment, workload, experiment, …). E = only a polymorphic party or
owner id. F = none.

| class | count | views (domain prefix `agentic.` dropped) |
|---|---|---|
| A | 3 | architecture.Contracts, architecture.Parts, architecture.Reach |
| B | 5 | architecture.Architectures, architecture.PatchApplications, architecture.Revisions, experiment.Experiments, kpi.Measurements |
| C | 43 | agent.AgentRevisions, agent.Agents, authority.Grants, channel.ChannelMessages, channel.ChannelPosts, channel.Channels, channel.Messages, channel.Posts, channel.Subscriptions, delegation.ActiveDelegations, delegation.Delegations, gateway.Calls, gateway.Gateways, gateway.Upstreams, harness.HarnessSessions, harness.Harnesses, harness.ToolCalls, knowledge.Knowledge, knowledge.PublishedConcepts, model.Endpoints, model.Inferences, model.Models, model.Routes, principal.Principals, resource.Claims, resource.Pools, resource.Queue, runtime.Runtimes, runtime.Workloads, sandbox.Sandboxes, tool.ToolProviders, tool.Tools, trigger.Firings, trigger.RecentFirings, trigger.Schedules, trigger.Triggers, trigger.TriggersOfRevision, work.Assignments, work.Runs, work.Tasks, workspace.Leases, workspace.Proposals, workspace.Workspaces |
| D | 7 | experiment.BestVariant, experiment.Variants, harness.Turns, question.Questions, runtime.Instances, work.Deliverables, work.Steps |
| E | 4 | authority.Budgets, authority.Decisions, authority.OpenApprovals, question.OpenQuestions |
| F | 5 | architecture.Patches, architecture.Ports, kpi.Directions, kpi.Kpis, process.Processes |

Caveat on C: in 4 views the only anchor is a principal or a route, not the entity the panel is
about. `authority.Grants` has `granted_by`, `work.Assignments` has `on_behalf_of`, `work.Tasks`
has `requested_by`, and `work.Runs` has `route_id`; their subject is reached through
`agent_revision_id`, `harness_session_id` or a polymorphic `grantee_id`. No view outside layers
9 and 10 carries an architecture or part key, and no view carries an instance key of a part.

## Appendix B: how the counts were taken

- `ess specify compile --path blocks --format json`, read with `jq`: `.views`, `.events`,
  `.commands`, `.entities`, `.actors`, `.components` and `.bindings` give the totals. Each view's
  `params` and `fields[].type_ref` gives the classes above. Each command's
  `outcomes[].subject.entity` and `emits` gives the event to entity mapping, which is then
  compared with each view's `source` and fields.
- `yq '.parts|length'` and `yq '.links|length'` on `architectures/*.yaml`.
- `ess ui check --path <probe> --model blocks` on a one-page probe document (a `collection` on
  `agentic.gateway.Calls` with a client filter, a parameter-scoped channel and a row action with
  `actor.may`), for the findings on what `ess-ui/1` cannot express.
