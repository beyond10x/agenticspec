# Deployment: from an agentic architecture to a running target

Status: design for approval, nothing built. Decisions K1-K12, open points Q1-Q9 with the option
recommended. Builds on `design/architecture-runtime.md`, `design/ingress.md` (I1-I12),
`design/control-plane.md` (C1-C8), `design/view-layer.md` (D1-D11) and
`design/events-and-storage.md` (E1-E10) without repeating them; cloudspec is cited as there.

## The question, and the verdict

An agentic architecture is more than its agents: it has a control plane (a web service), stores,
a gateway, lease authorities, scheduled runs and endpoints. Should agenticspec write deployment
files for each target itself, or **translate** the architecture into a cloudspec architecture
that cloudspec then realizes on a target?

**Verdict: translation into cloudspec works, with eight conditions (V1-V8, § Conditions).**
agenticspec keeps what only it knows: packing, admission of the contract, translation; it writes
no file of any target format. Until the conditions hold, only the in-process runtime
(`design/architecture-runtime.md` § "The smallest first runtime") runs an architecture: the local test.

## K1 Words, defined once

| word | means here | why this word |
|---|---|---|
| **pack**, packing | decide which parts share a unit | new; "placement" is taken twice: the entity-to-part `Placement` (`design/view-layer.md` M45) and cloudspec's target kind and scheduling decision (cloudspec `docs/boundary.md` rows 2, 12) |
| **unit** | one thing a target starts and watches: an agent host, a service, a job, a scheduled job, a store | neutral; becomes one `agentic.runtime.Workload` (`blocks/8-place/runtime.yaml`) |
| **translate**, translation | architecture, contract and binding with its packing go to a cloudspec architecture and a coverage report, with a digest | the word cloudspec takes over for a spec source turned into plain components (`deployments`: `Translator`, `Translation`, `CoverageEntry`, cloudspec `docs/boundary.md` row 19) |
| **realize**, realization | a cloudspec architecture goes onto one target | cloudspec `epic:realization`; ESS `ess specify realization` |
| **project**, projection | typed input to files of one format, never applied | ESS `ess generate project`: "never apply them"; inside cloudspec's realization only |
| **deploy** | translate, realize and start, ending in a running deployment | the user's word for the whole step |

Not used: *synthesize* (ESS suites, code), *materialize* (control-plane C1), *compile*, *distribute*,
*lower* (weft's word for deploying itself onto a host or cluster: realization here).

## The chain

```
architecture (portable)          architectures/<name>.yaml           revision digest
  + contract (derived)           agenticspec-contract/1              contract digest
  + binding with packing         agenticspec-binding/1, packing:     binding digest
  -> admit                       plan with units, or every refusal
  -> translate                   cloudspec architecture + coverage   translation digest
  -> realize (cloudspec)         target files or calls, per target
  -> running: the agentic adapter reports units and block events to the control plane
```

What exists: binding and admission as concepts with an open format
(`design/architecture-runtime.md` § "Three documents", § "Admission"); the runtime block
(`blocks/8-place/runtime.yaml`: `WorkloadKind` `Agent, Service, Job, Model, Snippet`, `Shape`,
`StartInstance` to `ReportFailed`) with nothing that defines workloads from an architecture; a
simulator that runs all parts in one world (`crates/agenticspec-sim/src/sim.rs:1-6`); cloud
blocks to be (cloudspec `docs/boundary.md` rows 9-22) and no cloudspec format yet (scope of
cloudspec `epic:simulation-and-search`).

## K2 Packing lives in the binding, never in the architecture

A `packing:` section of `agenticspec-binding/1`: the architecture stays portable
(`design/architecture-runtime.md:8-10`), the packing differs per target as the binding does.
Absent, it is the default (K4). `agenticspec packing default <architecture>` prints the resolved
packing to paste and change, as `agenticspec ingress default` does (`design/control-plane.md` C1).

## K3 Unit kinds

| unit kind | lives | workload kind | holds |
|---|---|---|---|
| `core` | long, exactly one | `Service` | the event log, the blocks' inner behaviour, the adapter serving every ingress (D7, I10) and with it the control plane |
| `agent` | long, replicas | `Agent` | agent parts, their `per` parts (workspace, sandbox), channels inside |
| `service` | long, addressable | `Service` | gateway, pool lease authority, tool server, a bus |
| `job` | until done, started on demand | `Job` | an agent part started by a spawn or an `on_event` reaction |
| `scheduled` | until done, started by a clock | `Job` | an agent part whose only input is an `on_schedule` trigger |
| `store` | long, keeps state | none | a store part (E4) bound to a storage service |
| `model` | long | `Model` | a self-hosted endpoint (`model.serve`) |
| `external` | not started | none | principals, hosted model routes, external stores and tools |

A unit names its least isolation (the highest of its sandboxes); its form is the target's (K8).

## K4 Strategies, and the default

| strategy | units | for |
|---|---|---|
| `single` (default) | everything in `core`, `external` parts outside | the local test, day one, small setups |
| `by_kind` | one unit per part kind | fewer units with separate scaling of agents |
| `by_part` | one unit per part; `count: n` becomes n replicas | independent scaling and failure |
| `groups` | named lists of parts, the rest in `core` | anything in between |

Per part, `schedule: job` (an agent woken only by `on_schedule` becomes `scheduled`; the trigger's
`policy` and `max_parallel` become the overlap rule) and `external` override the strategy. `per`
parts follow the part they belong to. A `count` marked `vary` is packed at its built value (`sim.rs`).

## K5 Admission of a packing

`admit` (`design/architecture-runtime.md` § "Admission") also admits the packing: the plan lists
the units, their part instances and the links that cross units; a refusal lists every finding:

| refusal | when |
|---|---|
| `part_unpacked`, `part_packed_twice` | a part instance is in no unit, or in two; `external` counts as packed |
| `per_split` | a `per: X` part is not in the unit of its X instance |
| `shared_state_split` | a pool, a gateway's rate limit, an architecture-scope store or an exclusive workspace is reached from two units and bound in memory |
| `link_without_transport` | a link crosses units and the binding names no transport for its carry |
| `isolation_unmet` | a sandbox's isolation exceeds what the target gives a unit and the binding names no sandbox inside it |
| `scheduled_with_inputs` | a `scheduled` unit holds a part with an input other than its trigger |
| `core_count` | not exactly one `core` |
| `unit_not_hosted` | the target's hosting table (K8), when given, hosts no unit of that kind |

## K6 Translation: what agenticspec emits

One cloudspec architecture in cloudspec's own format, with `sources.sha256` (architecture,
contract, binding, packing, translator version), and a coverage report: per part and link mapped
(to which object), mapped with loss (what is dropped), served by `core` (behaviour, not
infrastructure) or refused by name. Same inputs, same bytes. Every object carries its source
(architecture digest, unit, part paths). A refused part stops the deployment.

## K7 Mapping to cloudspec

| agentic | cloudspec construct (cloudspec `docs/boundary.md` row) |
|---|---|
| `core` unit | component, shape service, package the blocks' ESS system and the adapter (row 9, 10); its log in storage (row 6) |
| `agent` unit, replicas n | component, shape service, n replicas, package an agent document by digest or an image; stateful with a volume per replica when it holds workspaces |
| `scheduled` unit | schedule of a job, overlap from the trigger's `policy` (row 10) |
| `job` unit | job component started at run time; needs a grant to create jobs in the scope |
| `service` unit | component, shape service; capacity and rate stay the part's settings |
| `store` unit | volume, bucket or managed service by kind: record, key-value, semantic, graph (E4; rows 6, 21) |
| `model` unit | package model with an accelerator claim (rows 10, 11) |
| link across units | link with protocol; sandbox `egress` becomes its allow-list |
| export `http`, `event_stream`, `ui`, `tool`, `agent` (I7) | endpoint and route on `core`; certificate when public (rows 9, 18) |
| export `metrics_scrape` | observability: series named by KPI id (row 16) |
| sandbox secret names | secret grants by name (row 15); values never |
| `external` model route or tool | an external dependency with an egress route (cloudspec E4: not modelled yet) |

Not mapped, served by `core` and checked by agenticspec: routing invariants beyond the network
allow-list (every model call inside a gateway call), grants, questions, decisions, lease
semantics, `on_message` and `on_event` triggers, spawned sub-agents, principal ports.

## K8 The target is cloudspec's

- **Always a namespace-like scope: yes.** Every target has one scope holding exactly one
  deployment's units, names and state, deleted with it: a state directory on this host, a project
  name on a container host, a cluster namespace, a scope of a compute cloud; cloudspec's `scope`
  (cloudspec `docs/boundary.md` D1 A). Several architectures share a scope only nested in one.
- **Target kinds** (cloudspec's realizations): this host as supervised processes; a container
  host (a composition file); a cluster namespace (a package template); a self-hosted compute
  cloud (weft); an agent runtime (Loom), which hosts agent units only.
- **Hosting table.** Each realization declares the unit kinds and isolation it hosts and refuses
  the rest by name; agenticspec reads it by value at admission (`unit_not_hosted`). Loom serves
  no web service, so it is never a whole target: `core` and the control plane need a service.
- **Composite target, default: weft alone, agent units as Loom agents on weft.** weft already
  turns an agent document of Loom into an agent runtime instance (weft `docs/domain-sketch.md`
  § "Package: spec kinds"), so one realization, one scope and one network carry everything.
  Loom and weft side by side would need two realizations and a route between them; it is taken
  only when agents already run elsewhere, by binding the agent parts `external`.

## K9 Boundary with cloudspec

| concern | agenticspec | cloudspec |
|---|---|---|
| which parts share a unit, unit kind, replicas from `count` | packing (K2-K4) | none |
| contract, ports, routing invariants, admission | yes | none |
| translation and its coverage | writes it (K6) | defines the format and accepts the translator |
| targets, hosting tables, realization, files of each format | none | yes |
| network, entry points, certificates, network policy | the reach list and exports | endpoint, route, certificates, limits |
| secrets; cluster, nodes, shapes | names; `Shape` (CPU, memory) per unit | custody and delivery; infrastructure, scheduling |
| rollout, drift | none | `deployments` (plan, release, drift) |

agenticspec keeps no target-specific code; cloudspec keeps no agentic semantics.

## K10 Records and the control plane

- New domain `agentic.deployment` (layer 9): `Deployment` (revision; contract, binding, packing
  and translation digests; target and scope by value, as ESS has no relation across systems,
  cloudspec `docs/boundary.md` E3), states `Requested`, `Refused`, `Translated`, `Realizing`,
  `Running`, `Degraded`, `Stopped`, `Failed`; `Unit` (name, kind, part instances, replicas, its
  workload, its cloudspec object by value). `RequestDeployment` is refused as `NotReleased`
  (except to the in-process runtime), `NotAdmitted` (with the list), `CoverageRefused`, `ScopeTaken`.
- Health: the target's observations become each unit's `ReportRunning`, `ReportExited`,
  `ReportFailed` (`workload.run`). Operations (C3) gains a `Units` view (desired and running
  replicas, last failure, parts), the Overview a tile; a row reaches its unit through its part
  (`Placement`, M45). Conformance: a scenario that passes in process passes on the deployment.

## K11 Local test, then promote

The local test runs the released revision in process, with the scenario's fixtures or a local
binding. Promotion is a new deployment of the same revision with another binding and packing; a
target binding `refines:` a base binding, as an ESS server binding refines a protocol binding
(`design/ingress.md` § "What ESS already offers"). `Deployment` names the one it was promoted
from, so both are compared on the same KPIs.

| changes | must not change |
|---|---|
| target and scope, packing, endpoint addresses, authenticators, secret names, transports, store kinds | revision digest, contract digest, requirements (K12), scenarios and their expectations |

## K12 Requirements as KPIs with a target

A top-level `requirements:` key of the architecture, `- { kpi: cache_hit_share, at_least: 0.7 }`,
optionally per part and window; portable, outside the nesting digest like `ingress:` (I8). The
check refuses an unknown KPI id. A requirement is decided when its KPI can be read: a design KPI
(`story:architecture-kpis`: `agents_total`, `capacity_ratio`) at check, a simulated one at
evaluation, a computed one (`cache_hit_share`, `cost_usd`) in operation. The search treats
requirements as constraints before weights and stops when all are met. The Metrics page (C3)
shows each with its target, met or missed, predicted beside observed. A cloud KPI (node count,
cost per hour) is named by its cloudspec id and evaluated by cloudspec.

## Risks

cloudspec has no format, blocks or realization yet (its epics are draft) and weft has no code
(weft `README.md` "Status"): no remote target runs before both move. Without a source field kept
through realization, an observation maps to no unit or part. Two KPI catalogues drift where a KPI
is in both. agenticspec simulates agent behaviour and cloudspec infrastructure load (Q9).

## Conditions of the verdict (requests to cloudspec)

- V1 An architecture format: component; package (an agent document, an ESS system, an image);
  shape (service, job, schedule with an overlap rule); replicas; volume per replica; link with
  protocol; endpoint and route; storage kinds; secrets by name; isolation level.
- V2 Translation open to sources registered by another repository, with K6's coverage outcomes.
- V3 A source field on every object, kept into target objects and observations.
- V4 A hosting table per realization, with refusal names. V5 `scope` (D1 A) as the target boundary.
- V6 External dependencies with egress routes (cloudspec E4). V7 KPI ids qualified by system,
  series labelled with the source. V8 A first realization without a cluster: this host.

## Open points

| # | question | options | recommended |
|---|---|---|---|
| Q1 | where the target step lives | A: translate to cloudspec, no target files here. B: projectors per format in agenticspec. C: both | A |
| Q2 | the step's word | A: translate. B: lower. C: project | A |
| Q3 | where the packing lives | A: section of the binding. B: its own document | A |
| Q4 | default strategy | A: `single` everywhere. B: `by_part` on remote targets | A |
| Q5 | a target always a scope | A: yes, one deployment per scope. B: several deployments share one | A |
| Q6 | composite target | A: weft alone, agents as Loom agents on it. B: Loom and weft side by side | A |
| Q7 | who writes the agent document of an agent unit | A: the binding names it by digest. B: agenticspec generates it | A |
| Q8 | requirements | A: `requirements:` key of the architecture. B: in the experiment | A |
| Q9 | simulation coupling | A: one way, agent run to cloud load. B: co-simulation | A |

## Proposed stories

In this repository, under a new epic `architecture-realization` (the one `epic:architecture-contract`
defers to); requests to cloudspec are V1-V8 and the load hand-off of Q9, not stories here:

1. **packing-format**: the `packing:` section, unit kinds, strategies, `agenticspec packing default`.
2. **packing-admission**: K5's refusals, the plan's units; extends `ingress-admission` (ingress 6).
3. **deployment-domain**: `agentic.deployment`, its refusals, views and scenarios.
4. **cloud-translation**: `agenticspec translate`, the document, coverage, digest; after V1.
5. **requirements-kpis**: the `requirements:` key and its stages; extends `story:architecture-kpis`.
6. **control-plane-units**: `Units` view, requirement status; extends `control-plane-pages`.
7. **first-runtime**: the in-process runtime of `design/architecture-runtime.md`: the local test.
8. **agent-run-lift**: an agent runtime's run records lifted into block events (Loom agents).

Every story that runs tests keeps each test within 60 s and its step within 120 s. A job in the
build pool ends within 10 minutes.
