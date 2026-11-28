# Ingress: how an architecture exposes interfaces to the outside

Status: design for approval, nothing built. Each decision is numbered (I1-I12), each open point
(Q1-Q10) names the option recommended. Line numbers refer to this branch unless a branch or
repository is named. ESS facts are from `ess` 0.99.2.

An architecture says what runs and who may reach whom inside it. Nothing says what a client
outside it may call, read or subscribe to, nor over which protocol. An **ingress** is that outermost
layer. It keeps two questions apart:

- **what** is exposed: an interface, a named set of operations (commands, views, event feeds,
  metrics), each typed by the ESS model of the blocks;
- **how** it is exported: request and response over HTTP, an event stream, a command line, a
  model-context tool protocol, an agent-to-agent protocol, a metrics scrape endpoint, a UI.

One architecture has several ingresses, one per audience (a user, an operator, a monitoring
system), and each ingress is exported over one or more protocols. Ports are the outbound edge, where
the blocks ask the world to act (`design/architecture-runtime.md:44-67`). The ingress is the inbound
edge, where the world asks the architecture to act or to answer.

## What exists

| concept | where | what it covers | what it lacks |
|---|---|---|---|
| border | `Part.on_border`, "a human principal reaches this part from outside the architecture" (`blocks/9-composition/architecture.yaml:223-224`); `SetBorder` refuses a part no principal part links to (`:853-896`); the contract lists it as "the only parts that take input from outside" (`:384-406`) | which parts take traffic from outside, by links | no operation, no protocol, no audience beyond "a principal part links here". Only 2 of the 10 documents in `architectures/` declare one (`workstation.yaml:26`, `author-reviewer.yaml:35`) |
| border of a nested document | branch `consumer/w-arch-refs`, `crates/agenticspec-sim/src/nest.rs:4-6`: "The nested document's `border` is its interface"; `docs/architectures/index.md:55` there | which inner parts an enclosing document may link to | it is flattened into one document (`nest.rs:10-13`), so it has no wire at all |
| principal ports | `PortKind` `Principal`, "entry commands a principal issues (Answer, ApproveDecision)" (`architecture.yaml:127-138`); `principal.answer`, `principal.decide` (`design/architecture-runtime.md:84-85`) | the commands a human issues | nothing says how they reach the runtime |
| actions in the UI | `design/view-layer.md:137-156` (D6): principal ports and inputs into the border become actions; hiding is not authorization | the commands an operator UI offers | the UI is the only export; other protocols and other audiences are not covered |
| the runtime interface | `design/view-layer.md:158-182` (D7): R1 read, R2 feed with a cursor, R3 placements, R4 command, R5 action set, over one adapter | one HTTP and event-stream wire per admitted architecture | one surface for every caller; no audience, no subset |
| event log and delivery | `design/events-and-storage.md:75-92` (E3: identity, at least once, order per source, resume, retention); `bus.deliver` (`:190`) | the obligations of a feed | stated for the UI and triggers; no outside subscriber |
| the gateway | "The one place model calls pass ... Every call by a caller principal on a route is decided once and recorded" (`blocks/2-intelligence/gateway.yaml:3-8`); `Admission` `Admitted, OverBudget, RateLimited, NoUpstream` (`:15-17`); a `Call` record per call (`:90-140`); drain (`:44-54`) | an inbound decision point with a verdict per call, for model calls between parts | its callers are parts inside the architecture, its verdicts are budget, rate and upstream; it decides nothing for a client outside |
| actors | 30 ESS actors in `blocks/`, each with `may:`; `may:` can name views since `ess/22` (`ess specify formats`), as `agentic.authority.Authority` does (`blocks/6-governance/authority.yaml:210-224`) | which role may issue which command and read which view | mixes human roles (`Steward`, `Answerer`) and machine roles (`GatewayProcess`, `Transport`) with no mark |
| grants | `Grant` per action class on a scope, default deny (`authority.yaml:3-10`, `:54-84`) | who may do what, per scope | `action_class` is a `ToolCategory` (`:69-70`); no block command maps to one yet (`design/view-layer.md:297-301`) |
| KPIs | `kpis/catalogue.yaml`: 125 KPIs, each with `id`, `block`, `unit`, `reads`, `status`; status `computed` 12, `both` 2, `simulated` 43, `planned` 68 (`kpis/README.md:20` defines the values) | metric names, units and the events they read | no export |
| agenticspec's own exports | `agenticspec serve`: views over HTTP, two SSE channels resumed with `Last-Event-ID`, `/openapi.json`, the built UI, 127.0.0.1 only, every request answered as the one `Viewer` (`crates/agenticspec/src/serve.rs:1-11`, `:672-676`, `:690`; `ui/README.md:46-49`). The CLI is an `ess-cli/2` binding (`spec/cli.yaml:1`) | one audience, four protocols over one surface, already | these are the `agenticspec` system's, not an architecture's |

The gap, in short: the border names parts, the view layer names one UI, and nothing names an
audience, the operations it may use, or the protocols it uses them over.

## What ESS already offers, and what it does not

ESS separates the same two axes for one system, so the ingress reuses its split instead of a
parallel one:

- **What:** a component's surface: `owns` domains, `accepts` commands, `publishes` events, and
  `reached_by` (`network` in `spec/components.yaml:35`, `:53`; ESS also admits `in_process` and
  `command_line`). The ESS model holds no wire, port, path or verb (ESS repository,
  `docs/design/http-presentation-binding.md:33-35`).
- **How:** sibling documents, each pinned to the specification: `ess-http/1` binds commands and
  views to methods and paths (`ess generate --kind openapi --http <binding>` projects OpenAPI);
  `ess-transport/1` and `/2` bind events to brokers, subjects and streams (`--kind asyncapi
  --transport`); `ess-cli/2` is a command-line presentation (`spec/cli.yaml`); `ess-ui/1` is a UI
  with channels (`carries`, `resume: from_last_seen`, `scope: param`,
  `design/view-layer.md:269-270`).
- **Two levels:** an `ess-http/1` *protocol binding* is published with the specification and may
  leave paths open; a *server binding* `refines:` it and fills them in (ESS repository,
  `docs/design/http-presentation-binding.md:98-110`).
- **Who:** actors and `may:`; the scheme a caller authenticates with belongs on the binding (ESS
  story `an-http-binding-declares-how-a-caller-authenticates`, active).
- **Conformance:** a target answers `ExecuteCommand` and `QueryView` (`ess-conformance-target/1`);
  an HTTP conformance target through the binding is stage 3 of `ess-http/1`
  (`http-presentation-binding.md:57`). Streaming a route as server-sent events is the active ESS
  story `an-event-channel-may-be-bound-to-server-sent-events`.
- **Schemas:** `--kind schema` gives the JSON Schema of every input and row.

Not offered: `ess generate --kind` has 6 values (`docs`, `site`, `docs-ir`, `schema`, `openapi`,
`asyncapi`; `ess generate --help`). There is no projection to a tool list, an agent card or a
metric exposition, and nothing selects an audience's subset of a component's surface. `blocks/`
declares 0 components (`design/view-layer.md:264-267`); `story:block-bindings` adds the
`reached_by: network` component that serves the 22 domains.

## I1 Three entities: Ingress, Interface, Export

| entity | answers | holds |
|---|---|---|
| **Ingress** | for whom | a name, an audience (I3), whether callers authenticate (I6), one interface, one or more exports |
| **Interface** | what | selection rules over the compiled blocks and the KPI catalogue (I2). Resolved by the contract derivation into a list of operations, each `{kind, name}` with kind `command`, `view`, `feed` or `metric` |
| **Export** | how | one protocol (I7) and its protocol settings. Its projection (OpenAPI, tool list, agent card, metric list) is generated, never authored |

Rules that keep the axes apart:

- An interface names no protocol. An export names no operation.
- An export never narrows the interface. Every operation of the interface must be carried by at
  least one export of the ingress, by the operation kinds its protocol carries (I7). A different
  subset for another protocol is another ingress, because it is another "what".
- The interface stores operation names only. Fields, types, wire names and summaries come from the
  compiled model when a projection is generated, so the interface is never a copy of fields, and
  a change of a block reaches every projection through regeneration.

Why "export" and not "binding": `agenticspec-binding/1` already means which concrete thing plays
each part (`design/architecture-runtime.md:34-38`), and ESS `bindings:` mean an event causing a
command (`design/architecture-runtime.md:57`). An export is what ESS calls a protocol binding; the
runtime binding supplies the server binding (I8).

## I2 How an interface is derived

An interface is derived from the contract, never written as a list of fields. Its keys:

| key | values | resolves to |
|---|---|---|
| `scope` | `linked` (default): the parts the audience links to; `all`; or a list of part paths | the parts whose domains the rules below read. The domains of a part come from its contract, by the table `story:contract-derivation` owns (`design/view-layer.md:105-109`) |
| `commands` | `principal`: the principal ports the audience answers and the inputs it sends into the border, the two sources of D6 (`design/view-layer.md:144-151`); `plays: [<ESS actor>]`: the administration commands (I4) in those actors' `may:`; `except: [<command>]` | command names |
| `views` | `true` (all views of the scope's domains), `false`, or `except: [<view>]` | view names, the panel set of the view layer (`design/view-layer.md:99`) |
| `feeds` | `true`, `false`, or `except: [<event>]` | the events of the scope's domains, delivered under E3 (`design/events-and-storage.md:75-92`) |
| `metrics` | a list of KPI ids of `kpis/catalogue.yaml` | metric names; each KPI's `block` must be a block of a part in scope |

An interface only selects. It cannot rename an operation, add a field, filter rows or define a
type; row scoping is the runtime's job under the placement rule (`design/view-layer.md:283-292`).
The interface is an upper bound, not a grant (I5).

Commands and reads differ on purpose. A command has an effect, so it enters only through the border
(a work input), a principal port (an answer) or an administration command (I4). A view, a feed or
a metric has none, so it may read any part in scope, as the operator UI reads every part
(`design/view-layer.md:99`). In a nested document this matches the two rules already there: links
reach only border parts (`nest.rs:4-6`), while reads follow the scope of the part path
(`design/view-layer.md:172-177`).

## I3 The audience is a principal part

The audience of an ingress names a principal part of the document: `owner: { block: principal,
kind: human }`, or a part of kind `service` for a monitoring system (`PrincipalKind` `Human,
Organization, Service, Agent`, `blocks/0-foundation/shared.yaml:96-98`). So:

- one mechanism maps an audience to concrete callers: the runtime binding binds a principal part to
  identities, as it binds every part (`design/architecture-runtime.md:34-38`);
- `count` gives several callers of one audience (`admin: { block: principal, kind: human, count: 3
  }`);
- "no link, no reach" holds: work inputs require the audience's link to a border part, as
  `SetBorder` already does (`architecture.yaml:879-887`);
- the check counts an ingress's audience as a name, so a principal part reached only by an
  ingress is not refused as unreachable (`crates/agenticspec-sim/src/check.rs:41`).

## I4 A fourth class of entry command: administration

The port derivation classifies an entry command whose entity has no waiting state as
configuration, issued "by the deployer from the architecture document"
(`design/architecture-runtime.md:65-67`, `architecture.yaml:132-135`). By that rule
`agentic.gateway.DrainGateway`, `StopGateway` and `DetachUpstream` are configuration, but an
operator issues them at run time on an existing gateway (`gateway.yaml:44-54`, `:86-88`,
`:146-154`). An interface must never expose configuration (a client would change the setup
without a revision), and must expose these.

Taken: the derivation splits configuration in two, by the command's outcomes in the compiled
model:

- **configuration**: an outcome `creates:` the entity (`OpenGateway`, `AttachUpstream`,
  `RegisterPrincipal`, `DefineKpi`). Never in an interface; the architecture document, through a
  revision, is its only source.
- **administration**: every accepting outcome moves or updates an existing entity, and the command
  is no report of a port (`DrainGateway`, `StopGateway`, `DetachUpstream`, `RevokeGrant`). Exposed
  through `commands.plays`.

An audience may play an ESS actor only when none of the actor's commands is the report of a port:
those are the implementation's to issue (`GatewayProcess` may `AdmitCall`, `CompleteCall`,
`gateway.yaml:156-164`; `Transport` may `MarkDelivered`, `MarkLost`,
`blocks/7-collaboration/channel.yaml:390-396`; `resource.Scheduler` may `GrantClaim`,
`ExpireClaim`, `KillClaim`, the reports of `pool.lease`, `blocks/8-place/resource.yaml:115-123`).
`blocks/ports.json` gives both lists, so the check needs no command name.

## I5 Who may call what

Three layers, each from something that exists; the ingress adds no authorization of its own:

1. **Interface** (static, the document): an operation outside the ingress's interface is refused
   at the edge as `NotInInterface`, before any block sees it.
2. **Actor** (static, the blocks): the caller authenticates as a principal of the audience, and the
   runtime holds it to the `may:` of the ESS actors it plays (`actor.may`, as the UI uses it,
   `design/view-layer.md:192`).
3. **Grant** (per scope, at run time): the authority evaluator decides each command on its grants,
   default deny (`authority.yaml:3-10`). The action set R5 (`design/view-layer.md:170`, O5 there) is
   answered per ingress: the interface's commands, each with the evaluator's verdict.

The interface is the upper bound, the grant decides per instance. Hiding an action is still a
convenience, not authorization (`design/view-layer.md:152-153`).

## I6 Authentication is the binding's, its absence is the document's

The scheme (a bearer token, a key, a client certificate) and the issuer are concrete, so they are in
the runtime binding, as ESS puts the scheme on the HTTP binding. The document says only whether an
ingress authenticates: by default it does. `authenticated: false` is admitted only for an interface
without commands (views, feeds, metrics); the check refuses it beside any command. Admission
refuses an authenticated export for which the binding names no authenticator.

## I7 Protocols, what each carries and what each needs

| protocol | carries | projection, generated | needs | reuses |
|---|---|---|---|---|
| `http`, request and response | `command` (POST, the outcome or the refusal's row, R4), `view` (GET rows, R1) | an `ess-http/1` protocol binding over exactly the interface's operations, paths left open; OpenAPI from `ess generate --kind openapi --http` | the server binding fills the paths (I8) | `ess-http/1`, OpenAPI, R1, R4 |
| `event_stream` | `feed` | the stream list: per scope, the events it carries, resumable from a cursor (the `ess-ui/1` channel shape, `design/view-layer.md:127-131`); AsyncAPI when the binding names a broker through `ess-transport/1` or `/2` | cursor, retention, `CursorExpired` (E3) | R2, `bus.deliver`, `ess-transport` |
| `cli` | `command`, `view` | an `ess-cli/2` binding and its generated parser | the binding names the program | `ess-cli/2`, `reached_by: command_line` |
| `tool`, a model-context tool protocol | `command`, `view` | a tool list: one tool per command (input schema from `--kind schema`, description from `naming.summary`, a refusal as a tool error naming the ESS error) and one read tool per view (its parameters as input). Tool names are the ESS wire names (`admit-call`, `gateway.yaml:369`) | none beyond the server | `--kind schema`, `naming`; the studio's planned agent channel drives one operation set as tools beside its UI (`studio/system.yaml:5-10`, `story:studio-mcp`) |
| `agent`, an agent-to-agent protocol | `command` as skills, `feed` as the updates of a conversation | an agent card: the architecture's name, one skill per command with its input schema, how to reach it | the work inputs must travel over a channel part (`over:`), so `channel.deliver` serves the conversation and the channel's message protocols apply (`design/building-blocks.md:110-112`, which names the a2a specification as the network form of agent message; `ChannelKind` `AgentToAgent`, `channel.yaml:36-45`) | `ess-protospec/1`, the channel block |
| `metrics_scrape`, a metrics scrape endpoint | `metric` | the metric list: per KPI its catalogue `id` (under a prefix the binding may set), `summary` as help, `unit`, labels architecture, part path and instance (the placement, `design/view-layer.md:283-292`); every KPI a gauge, since a KPI is a value over a window, not a running total | each KPI computable from the events the runtime emits (I9) | the KPI block (`blocks/10-evaluation/kpi.yaml:73-94`), the catalogue |
| `ui` | `command`, `view`, `feed`, `metric` | the view layer's `ess-ui/1` composition for this ingress (I10) | `http` and `event_stream` exports on the same ingress, since the UI's client uses R1, R2, R4, R5 (`design/view-layer.md:158-177`) | `ess-ui/1`, the view layer |

Not an operation: machine traffic (`model_call`, `tool_call`, `claim`) is never an interface
operation, as it is never a UI action (`design/view-layer.md:150-151`). Q8 covers a gateway offered
to outside callers.

A field adds no protocol. An inbound webhook is an `http` export of commands for a `service`
audience; the mapping from a third party's payload to a command is the runtime binding's adapter,
or a pack's (`design/fields.md` § "Three tiers"). The protocol names stay generic: the check
refuses a protocol value outside the table.

## I8 Where each piece lives

| piece | lives in | why |
|---|---|---|
| ingresses, audiences, interfaces, exports | a top-level `ingress:` key of the architecture document (Q1) | portable: names no host, product or address, as the document promises (`design/architecture-runtime.md:8-10`) |
| entities | `agentic.architecture` (layer 9): `Ingress` (the rules as written, as `PatchRule` keeps its text, `architecture.yaml:76-83`) and `Export`; commands `AddIngress`, `AddExport`; view `Ingresses`; `IngressId`, `ExportId` in `agentic.shared` | an ingress names parts, so it cannot sit below layer 9 |
| resolved interface | `Contract` gets `ingress: List<ContractIngress>`: the ingress, its resolved operations, its export protocols (`architecture.yaml:384-406`) | derived like the ports, never authored |
| address, server, authenticator, path per export | `agenticspec-binding/1`; for `http`, the ESS server binding that `refines:` the generated protocol binding | concrete, per runtime |
| network entry point, certificate, scaling, scrape registration | cloudspec (§ "Boundary with cloudspec") | infrastructure |

Digests: the ingress changes what a runtime must serve, so it is in the contract and its digest
(`Contract.digest`). It stays outside the nesting digest (`nest.rs:31`, `DIGESTED = ["border",
"links", "parts"]`), so a change of a nested document's ingress breaks no pin of the documents
that nest it (Q10). `crates/agenticspec-sim/src/best.rs:72-80` (`TOP`) gets `ingress` after
`border`, so a rewritten best variant keeps the key.

An architecture without `ingress:` declares none; nothing is implicit. Admission still needs each
principal port served (I9), by an ingress export or by a fixture.

## I9 Contract and admission

- **Check of the document** (`agenticspec sim check`), new refusals:

  | rule | refused |
  |---|---|
  | `ingress_audience_not_principal` | an audience that names no principal part |
  | `ingress_without_export` | an ingress with no export |
  | `ingress_unknown_operation` | a name in `plays`, `except` or `metrics` that the compiled blocks or the catalogue lack |
  | `ingress_actor_is_implementation` | `plays` names an actor whose `may:` holds a report of a port (I4) |
  | `ingress_outside_scope` | an actor or a KPI whose domain or block is not in the scope |
  | `ingress_command_off_border` | a work command whose target part is not on the border |
  | `ingress_operation_not_carried` | an operation kind no export of the ingress carries |
  | `ingress_ui_without_wire` | a `ui` export without `http` and `event_stream` beside it |
  | `ingress_unauthenticated_command` | `authenticated: false` beside a command |
  | `ingress_agent_without_channel` | an `agent` export whose work inputs travel over no channel part |
  | `principal_port_without_ingress` | in a document that declares `ingress:`, a principal port of the contract that no ingress of the answering principal carries |

- **Admission** extends `admit(contract, binding?, environment)`
  (`design/architecture-runtime.md:116-121`): each export is a capability, named
  `export.<protocol>`, beside the ports. The plan names, per export, the server and its address. A
  refusal lists every export the environment cannot serve, never only the first, together with
  the missing ports. It also refuses:
  - an authenticated export whose audience has no authenticator in the binding (I6);
  - a `metrics_scrape` export of a KPI whose status is neither `computed` nor `both`: a real
    runtime cannot compute it from observed events (`kpis/README.md:20`). The simulator serves
    every KPI whose status is `simulated` or `both`;
  - a principal port served by neither an ingress export nor a fixture. A scripted answer is a
    fixture like any other (`design/architecture-runtime.md:136-141`), so a test run needs no
    ingress.
- **Conformance** of an ingress, beside block and scenario conformance
  (`design/architecture-runtime.md:123-134`):
  - the block suite runs through the `http` export (ESS stage 3, an HTTP conformance target through
    the binding), restricted to the interface's operations;
  - an operation outside the interface is refused as `NotInInterface`, a caller of another
    audience is refused, and an unauthenticated call to an authenticated export is refused;
  - a feed resumed from cursor c yields exactly the events after c, the one obligation that R2,
    triggers and `bus.deliver` share (`design/events-and-storage.md:86-88`).

## I10 The view layer's UI is one export

The operator UI of `design/view-layer.md` is the `ui` export of an ingress, not a surface of its
own:

- the composition (D2, D3) is derived per ingress with a `ui` export: its pages show the panels of
  the interface's views, its actions are the interface's commands (D6 becomes the `principal` rule
  of I2), its channels are the interface's feeds;
- R1-R5 (D7) are the `http` and `event_stream` exports of the same ingress; `{source}` gains the
  ingress name, `{root}/ingress/{ingress}/scopes/{architecture_id}[/{part path}]`, so two
  audiences of one runtime get two surfaces;
- one runtime serving three ingresses runs one adapter that refuses per ingress (I5); there is
  still no view that exists only for the UI (`design/view-layer.md:160-162`).

A UI for an owner and a UI for an operator are two ingresses with a `ui` export each, not two
hand-made apps.

## I11 The simulator

Today tasks arrive for the whole architecture and wait in one backlog
(`crates/agenticspec-sim/src/sim.rs:3-4`), at `load.tasks_per_hour` (`profiles/coding.yaml:69-70`),
plus an experiment's `bursts` (`sim.rs:71`).

- **Arrivals per ingress.** A profile may give `load.ingress.<name>: { commands_per_hour,
  reads_per_hour }`. Commands on work inputs are task arrivals at the border part they land on;
  without the key, arrivals stay as today, so every existing result stays byte for byte.
- **Administration** is an injected event, like a burst: an experiment's
  `operations: [{ at: 2h, ingress: operator, command: agentic.gateway.DrainGateway }]`, so a drain
  under load can be simulated and its KPIs read.
- **Reads** cost 0 by default; a profile may give a read a delay and a cost.
- **A rate limit** of an ingress (`rate_limit_per_minute` on the ingress, a place that may be
  marked with `vary`, as the gateway's is in `architectures/workstation.yaml:9`) refuses arrivals
  over it; refused arrivals are counted.
- **A scrape** of a `metrics_scrape` export is a sample of each KPI at each scrape interval, so the
  simulator predicts the series a monitoring system would read, comparable with the observed one.

## I12 Records of ingress calls

Like the gateway, an ingress is a decision point, and its decisions are worth a record: a domain
`agentic.ingress` (layer 9, beside `architecture`) with one `Call` per inbound call, decided once
(`Admitted`, `Unauthenticated`, `NotInInterface`, `Forbidden`, `RateLimited`, `Draining`) and
completed with its outcome. It copies the gateway's shape (`gateway.yaml:15-17`, `:90-140`) and
not its record: the gateway decides budget, rate and upstream for model calls between parts, an
ingress decides authentication, interface, authority and rate for calls from outside. Its events
give KPIs: `ingress_refused_share`, `ingress_latency_p90_seconds`, and the gate
`feed_resume_gap_count` (events missing after a resume, must be 0). Later than I1-I11 (Q5).

## Boundary with cloudspec

cloudspec, a sibling repository at the planning stage (no specification and no code yet,
cloudspec `AGENTS.md:13-15`), is to cloud architectures what agenticspec is to agent sessions. A
cluster's entry points (ingresses, gateways, load balancers) are in its scope (cloudspec
`docs/vision.md:73-77`). The word "ingress" means a network object there and an interface here, so
the cut is drawn by what each side knows:

| concern | agenticspec | cloudspec |
|---|---|---|
| what is exposed, to whom, under which authority | the interface, the audience, I5 | nothing |
| protocol semantics | the export and its projection (OpenAPI, stream list, tool list, agent card, metric list) | nothing |
| a rate limit per audience, as policy of the architecture | `rate_limit_per_minute` on the ingress (I11) | nothing |
| address, server, authenticator | the runtime binding | nothing |
| the network entry point and its route | nothing | its architecture block's `Endpoint` and `Route` (cloudspec `docs/boundary.md:52`, layer 4 at `:172`) |
| the certificate of a public entry point | nothing | `certificates` (`EdgeCertificate`, `docs/boundary.md:61`, layer 5 at `:173`) |
| network limits a workload or endpoint consumes | nothing | `limits`: `Rate`, `Concurrency`, `Deadline` (`docs/boundary.md:65`, `:173`) |
| who scrapes metrics, where series are kept | nothing | `observability`: `MetricSeries`, `TelemetrySink` (`docs/boundary.md:59`) |
| isolation of callers by ownership | nothing | `scope` (proposed cut D1, `docs/boundary.md:146-152`) |
| load from clients | `load.ingress.<name>` per audience (I11) | "clients that generate load" as actors a profile drives (`docs/vision.md:41-46`) |
| deployment to a target | admission of the contract | the ports a target must serve (cloud-layer ports, cloudspec `.engineering/planning/epic/realization.md:12-24`) |

Where they meet: when an admitted architecture runs on a cloud, each export is realized by a
cloudspec `Endpoint` (with its `Route` and certificate). The runtime binding names that endpoint by
identifier, by value, since ESS declares no relation across two systems (cloudspec
`docs/boundary.md:218-223`; agenticspec does the same in `studio/domains/shared.yaml`). A
`metrics_scrape` export is a target of cloudspec's observability: the KPI ids become the names of
its `MetricSeries`. Ingestion runs the other way only: a cluster ingested into a cloudspec
architecture shows the runtime's entry point as an endpoint (cloudspec `docs/vision.md:83-86`);
nothing flows back into an agenticspec document by itself.

## Worked example: `architectures/workstation.yaml`

Today: one owner, an operator agent acting for it, 12 workers, a gateway, a model, a build pool,
the `chat` channel, per-worker trees and sandboxes; `border: [operator]`
(`architectures/workstation.yaml:5-26`). Its contract: `agent.session`, `model.infer`,
`gateway.call`, `channel.deliver`, `principal.answer`, `pool.lease`, `workspace.provide`,
`sandbox.enforce` (`design/architecture-runtime.md:107-110`). Added:

```yaml
parts:
  # ... the nine parts as they are ...
  admin:    { block: principal, kind: human }
  monitor:  { block: principal, kind: service }

ingress:
  user:
    audience: owner
    interface:
      scope: linked
      commands: [principal]
      views: true
      feeds: true
    exports:
      - { protocol: http }
      - { protocol: event_stream }
      - { protocol: ui }
  operator:
    audience: admin
    interface:
      scope: all
      commands: { plays: [agentic.gateway.GatewayOperator] }
      views: true
      feeds: true
    exports:
      - { protocol: http }
      - { protocol: event_stream }
      - { protocol: ui }
      - { protocol: tool }
  metrics:
    audience: monitor
    authenticated: false
    interface:
      scope: all
      metrics:
        - owner_wait_share
        - cost_usd
        - rate_limited_seconds
        - queue_wait_p90_seconds
        - pool_utilization
    exports:
      - { protocol: metrics_scrape }
```

Resolved:

| ingress | commands | views and feeds | metrics | exports and projections |
|---|---|---|---|---|
| `user` | `agentic.channel.SendMessage` (the prompt `owner -> operator over chat` into the border part, `workstation.yaml:17`, the D6 example); `agentic.question.Answer` (`principal.answer` for `operator -> owner carries question`, `:18`; `Answerer` may `Answer`, `blocks/7-collaboration/question.yaml:122-129`) | the domains of `operator` and `chat` and the question block: among them `agentic.question.OpenQuestions`, `agentic.channel.ChannelMessages`, `agentic.harness.HarnessSessions`, `agentic.harness.Turns`; their events as the feed | none | OpenAPI over 2 commands and those views; one stream; the owner's UI |
| `operator` | `DrainGateway`, `StopGateway`, `DetachUpstream`: the administration commands of `GatewayOperator`; its `OpenGateway` and `AttachUpstream` are configuration and drop out (I4) | every view of the contract's domains, one feed per level (the view layer's operator UI) | none | OpenAPI, a stream, the operator UI, and a tool list of 3 command tools plus one read tool per view, so an agent can drain the gateway through a model-context tool protocol under the same grants |
| `metrics` | none (so `authenticated: false` passes) | none | the 5 KPIs; their blocks principal, model, gateway, resource are blocks of `owner`, `model`, `gateway`, `builds` | a metric list of 5 gauges with help text and units from the catalogue |

Admission:

- In the simulator every export is served. `owner_wait_share` and `cost_usd` are `both`;
  `rate_limited_seconds`, `queue_wait_p90_seconds` and `pool_utilization` are `simulated`
  (`kpis/catalogue.yaml:163-173`, `:912-935`), and the simulator computes all five.
- Against a runtime whose environment has an HTTP server, an event-stream server and the UI, an
  authenticator for `owner` and `admin`, and nothing else, the refusal names everything at once:
  `operator: export.tool has no server`; `metrics: export.metrics_scrape has no server`; `metrics:
  rate_limited_seconds, queue_wait_p90_seconds, pool_utilization are not computed from observed
  events`.

The simulator's result for this document must stay byte for byte while the profile names no
`load.ingress`. `admin` and `monitor` link to nothing; `ingress-format` proves that they change no
result, and moves the example to a copy of the document if they do.

## Open points

| # | question | options | recommended |
|---|---|---|---|
| Q1 | where an ingress is declared | A: a top-level `ingress:` key of the architecture document. B: a part kind `ingress` with links (its rules would be `Setting` text, `architecture.yaml:53-62`, and a protocol would become a part). C: in the runtime binding (the interface would stop being portable) | A |
| Q2 | the audience | A: a principal part of the document (I3). B: a principal kind written inline in the ingress. C: an ESS actor name only (no identities to bind) | A |
| Q3 | reuse of one interface by several ingresses | A: inline per ingress; two audiences with one interface write the rules twice. B: a named `interfaces:` key that ingresses reference | A |
| Q4 | callers without authentication | A: only for an interface without commands (I6). B: never. C: any interface, the binding decides | A |
| Q5 | records of ingress calls | A: the domain `agentic.ingress` of I12, in a story after I1-I11. B: no record; the blocks' events suffice (refusals at the edge would leave no trace). C: reuse the gateway's `Call` (its verdicts are budget and upstream) | A |
| Q6 | the ingresses of a nested document | A: not served when nested; the enclosing document declares its own over the flattened contract. B: the enclosing document re-exports one by name, pinned by a digest of its own. C: inherited | A |
| Q7 | the projections ESS lacks (tool list, agent card, metric list) | A: generated in agenticspec from the compiled model, `--kind schema` and `naming`, committed with `sources.sha256`; asked of ESS when a second consumer needs one. B: asked of ESS first, nothing until then | A |
| Q8 | a gateway or other machine traffic offered to outside callers | A: a border part linked from a `service` principal; its wire is the gateway implementation's, named in the binding; no export kind. B: an export `model_api` | A |
| Q9 | the name | A: `ingress`, with § "Boundary with cloudspec" saying what it is not. B: `surface`, to avoid the network meaning | A |
| Q10 | which digest covers an ingress | A: the contract's, outside the nesting digest (I8). B: the revision's too, so a nested document's ingress change breaks every pin of it | A |

## Proposed stories

1. **ingress-format**: the `ingress:` key of `agenticspec-architecture/1`, the check rules of I9,
   the audience counted as a name, `best.rs` `TOP`, its reference page in `docs/`, and the
   workstation example with `task catalog:generate` and `task docs:generate`.
2. **ingress-entities**: `Ingress`, `Export`, `AddIngress`, `AddExport`, the `Ingresses` view,
   `IngressId` and `ExportId` in `agentic.shared`, `Contract.ingress`, with authored scenarios
   (`task check:layers`, `task check:blocks`).
3. **administration-commands**: the configuration and administration split of I4 joins
   `story:port-derivation`, and `blocks/ports.json` marks both and each actor that issues a report.
4. **interface-derivation**: resolve each interface into operations from the contract,
   `blocks/ports.json`, the actors' `may:` and the catalogue, as part of
   `story:contract-derivation`;
   `agenticspec-contract/1` gets its `ingress` section.
5. **ingress-projections**: per export the generated `ess-http/1` protocol binding and OpenAPI, the
   stream list, the `ess-cli/2` binding, the tool list, the agent card and the metric list, each
   with `sources.sha256`, a fast digest check and a slow regeneration. If `ess-http/1` cannot
   project a subset of a component's surface, the gap is filed in ESS with a reproduction.
6. **ingress-admission**: exports as capabilities in admission, the authenticator and KPI-status
   refusals, every refusal listed, and what `agenticspec-binding/1` says per export (address,
   server, authenticator, the ESS server binding).
7. **ingress-conformance**: `NotInInterface`, wrong-audience and unauthenticated refusals, the
   block suite through the `http` export, and feed resume shared with S6 of the view layer and
   `bus-delivery-conformance`.
8. **view-layer-per-ingress**: S4 and S6 of `design/view-layer.md` derive one UI per ingress with
   a `ui` export, D6 actions from the interface, and `{source}` with the ingress name; R5 answers
   per ingress.
9. **sim-ingress-load**: arrivals per ingress, administration operations in experiments, the
   ingress rate limit as a place that may be marked, and sampled scrapes; results byte for byte at
   the defaults.
10. **ingress-records**: the `agentic.ingress` domain of I12, its events, and the three KPIs in
    `kpis/catalogue.yaml`.

Every story that runs tests keeps each test within 60 s and its step within 120 s. A job in the
build pool ends within 10 minutes.
