# Deploying an architecture

An architecture document says what a system is, in block terms, and names no host, product or
target. To run it somewhere, three more things are needed: a **binding** says which concrete thing
plays each part on one target and how the parts are **packed** into units; **admission** checks the
architecture, its contract, the binding and the packing together; and the **deployment bundle** is
the one file a translator turns into its target's own architecture. agenticspec writes the bundle;
translating it and running it are the target's work.

```sh
agenticspec packing default architectures/workstation.yaml                  # the default packing
agenticspec deploy bundle architectures/workstation.yaml \
  --binding bundles/bindings/split.yaml --out workstation.bundle.yaml       # admit, then write
```

## The binding

A binding (`agenticspec-binding/1`) is a YAML file for one architecture and one target. Every key
but `format` and `binding` is optional.

| key | holds |
|---|---|
| `format` | `agenticspec-binding/1` |
| `binding` | the binding's name |
| `architecture` | the name of the architecture it is for; any architecture when absent |
| `refines` | a base binding file, relative to this one. The base's keys are read first and this file's keys go over them, mapping by mapping |
| `target` | what the target gives each unit: `isolation`, one of `none`, `process`, `container`, `user_kernel`, `micro_vm` (`container` when absent) |
| `parts` | per part: `agent` (the agent document that plays it, a path relative to the binding file), `digest` (that document's `sha256:`, checked), `state` (`memory`, the default, or `service`: where a part with shared state keeps it), `inside` (a sandbox that runs inside the unit), `secrets` (names, never values), `size` (of a workspace's volume) |
| `transports` | per carried value (`model_call`, `order`, `question`, `claim`, ...), the transport a link that crosses units uses |
| `runtimes` | runtime images pinned by digest: `core` for the core and service units, `agent` for agent units. Each has `digest` (`sha256:` and 64 hex digits) and `image` |
| `claims` | per unit name: `cpu_millis`, `memory_mib`, `accelerators` (500, 512 and 0 when absent; a model unit takes one accelerator) |
| `packing` | the packing section below |

A binding never changes the architecture: promoting a revision to another target is a new binding
and a new bundle of the same architecture.

## Packing

A **unit** is one thing a target starts and watches. Each part of the architecture is in exactly
one unit, or is `external`: not started by the target at all.

| unit kind | lives | holds |
|---|---|---|
| `core` | long, exactly one | the event log, the inner behaviour of the blocks, the control plane and the parts no other unit takes |
| `agent` | long, replicas | agent parts, with the parts that are `per` one of them (workspace, sandbox) |
| `service` | long, addressable | a gateway, a pool, a tool, a runtime |
| `job` | until done, started on demand | an agent part marked `schedule: job` that no clock wakes |
| `scheduled` | until done, started by a clock | an agent part marked `schedule: job` that a schedule or an `on_schedule` trigger wakes |
| `store` | long, keeps state | a shared workspace |
| `model` | long | a model part served by the target itself (marked `external: false`) |

Principals and model parts are `external` unless marked otherwise.

The `packing:` section names a **strategy**, or lists the units:

| strategy | units |
|---|---|
| `single` (the default, also when the section is absent) | every part in `core`, external parts outside |
| `by_kind` | one unit per block that is not held by the core: all agents in one unit, each kind of service in one |
| `by_part` | one unit per part; a part's `count` becomes the unit's replicas |
| `groups` | the named lists of `groups:` are units; the rest is in `core` |

Per part, `parts: { <part>: { schedule: job } }` puts an agent into a `job` or `scheduled` unit of
its own, and `external: true` or `external: false` overrides where a part goes. A part that is
`per` another goes with it. A `count` marked for optimization is packed at its built value.

```yaml
packing:
  strategy: groups
  groups:
    edge: [gateway, builds]
  parts:
    sweeper: {schedule: job}
```

`agenticspec packing default <architecture> [--strategy <strategy>]` prints the section in its
**resolved** form, every unit listed, to paste into a binding and change:

```yaml
packing:
  units:
    - name: core
      kind: core
      parts: [box, builds, chat, gateway, operator, tree, worker]
      replicas: 1
  external: [model, owner]
```

In the resolved form, `units:` lists every unit (`name`, `kind`, `parts`, `replicas` for a unit
that runs as copies) and `external:` the parts outside; `strategy`, `groups` and `parts` do not go
with it. A section of the wrong form (an unknown key, strategy, kind or part, two units of one
name) is refused with every problem as `binding_invalid`.

## Admission

`agenticspec deploy bundle` writes a bundle only for an architecture it admits: `agenticspec sim
check` accepts it, its contract derives and is valid, the binding is of the right form, and the
packing passes the rules below. A refusal exits 1 with `not_admitted`, names every finding in the
order of the table, and writes nothing.

| refusal | when |
|---|---|
| `part_unpacked` | a part is in no unit and not external |
| `part_packed_twice` | a part is in two units, or in a unit and external |
| `per_split` | a part that is `per` another is not in that part's unit |
| `shared_state_split` | a pool, a gateway with a rate limit or an exclusive workspace is reached from two units and its binding keeps its state in memory |
| `link_without_transport` | a link crosses two units and the binding names no transport for what it carries. A clock starting a `scheduled` unit needs none: the target's schedule starts it |
| `isolation_unmet` | a sandbox asks for more isolation than the target gives a unit, and the binding names no sandbox `inside:` it |
| `scheduled_with_inputs` | a `scheduled` unit takes an input other than its clock, or no clock wakes it |
| `core_count` | the packing has not exactly one `core` unit |
| `unit_not_hosted` | a unit's kind or one of its roles is not in the accepted list |
| `agent_document_missing` | an agent, job or scheduled unit holds an agent part the binding names no readable agent document for |
| `agent_document_digest` | an agent document is not the digest the binding pins |

The **accepted list** (`bundles/cloudspec-accepts.yaml`, `agenticspec-cloudspec-accepts/1`) is
the copy of the unit kinds and roles the translator accepts, with the commit it was copied from
(`cloudspec_commit`, empty until the translator publishes its list). `--accepts <file>` reads
another.

## The bundle

`agenticspec deploy bundle <architecture> --binding <file> [--out <file>] [--ports <file>]
[--accepts <file>] [--kpis <file>]` writes one YAML file, `agenticspec-deployment/1`, with every
member inline, or prints it (`jq -r .bundle`) without `--out`. Its contract has the ingresses
resolved, the default `metrics` ingress against the KPI catalogue (`kpis/catalogue.yaml` unless
`--kpis` names another). The same inputs give the same bytes. A digest is `sha256:` and the
SHA-256 of the canonical JSON of what it covers (keys sorted, no space).

| member | holds |
|---|---|
| `format`, `generator` | `agenticspec-deployment/1`; the agenticspec version that wrote it |
| `architecture` | the document, its name, its revision digest (of its `parts`, `links` and `border`) and the digest of its bytes (`sha256`) |
| `nested` | the documents the architecture names, by digest; empty while architectures do not nest |
| `contract` | the contract (`agenticspec-contract/1`, see [the contract](contract.md)) and its digest |
| `binding` | the binding with its `refines:` chain resolved, its name and digest |
| `packing` | the resolved packing, its strategy and digest |
| `units` | one entry per unit, below |
| `ingress` | empty for now: who reaches the system from outside, and over what, is not yet part of the contract |
| `requirements` | the architecture's `requirements:`, empty when it has none |
| `agents` | each agent document the binding names, by the digest of its bytes, with the parts it plays |
| `runtimes` | each runtime image the binding pins, by digest |
| `digest` | the bundle digest, of every member but this one |

Each unit:

| field | holds |
|---|---|
| `name`, `kind`, `parts` | the unit, its kind and its parts |
| `replicas` | the copies that run; absent for a `job` or `scheduled` unit |
| `code` | what runs. An agent, job or scheduled unit stays an agent: the role `agent-runtime` and its agent documents by digest, never a process command. The core and service units run the pinned core runtime, or name the role `core-runtime` |
| `claims` | `cpu_millis`, `memory_mib`, `accelerators`, and `volumes`: one per workspace, with its access (`read_write_once` when exclusive, `read_only_many` when shared read-only, `read_write_many` otherwise), whether there is one per copy of its part, and its size when the binding gives one |
| `isolation` | the highest isolation its sandboxes ask for, in agenticspec's own levels |
| `checks` | how the target sees it start and stay up: an agent unit's agent runtime checks its agent document before it runs; the core and service units answer `/ready` and `/alive` |
| `schedule` | a scheduled unit's clock: the trigger and schedule that wake it, `every`, `next_fire_at`, and the trigger's `policy` and `max_parallel` as the rule for overlapping runs |
| `secrets` | the secret names its parts need |
| `egress` | every link from a part of the unit to a part outside it: to another unit with its transport, or to an external part |
| `roles` | what the unit needs from its target, never a provider: `agent-runtime`, `core-runtime`, `start-jobs` (the core, when a job unit exists), `model-server`, `store`, `model-route:<route>` for each hosted model route it calls |
| `ports` | the contract ports its parts use, by their derived names, with their settings and instances |
| `source` | the unit and its parts, carried into what the translator makes of it |

The bundle carries no secret value, host, provider or target name.

## Fixture bundles

`bundles/` holds a bundle of each catalog architecture under each test binding of
`bundles/bindings/` (`bundles/<architecture>.<binding>.yaml`), with the agent documents they name
and `bundles/sources.sha256`, one digest of the sources of each bundle. `task bundles:generate`
writes them, `task check:bundles` (in the gate) fails when their sources changed, and `task
slow:bundles` builds them afresh and compares bytes.
