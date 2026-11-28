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
agenticspec deploy check workstation.bundle.yaml                            # admit a written bundle again
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
| `target` | what the target gives each unit: `isolation`, one of `none`, `process`, `container`, `user_kernel`, `micro_vm` (`container` when absent); `images`, where it takes runtime images from, `registry` (the default) or `local` (its own image store); `network_policy`, `enforced` or `not_enforced` (unknown when absent) |
| `parts` | per part: `agent` (the agent document that plays it, a path relative to the binding file), `digest` (that document's `sha256:`, checked), `state` (`memory`, the default, or `service`: where a part with shared state keeps it), `inside` (a sandbox that runs inside the unit), `secrets` (the secret names it needs, never values), `egress` (the destinations it reaches, below), `size` (of a workspace's volume), `volumes` (where the part mounts a workspace's volume: `[{name: <workspace>, mount: <absolute path>}]`), `network: isolated` (the part may reach only what its unit's egress lists) |
| `transports` | per carried value (`model_call`, `order`, `question`, `claim`, ...), the transport a link that crosses units uses |
| `runtimes` | runtime images: `core` for the core and service units, `agent-runtime` for the agent units (the role it plays; without it the target binds the role, and the older name `agent` is refused). Each has `image`, the image name without tag or digest (`registry.example/agenticspec-core`). From a registry it is pinned by `digest` (`sha256:` and 64 hex digits, the repository digest), and a runtime without either, or with a `tag`, is refused. From the target's local store (`target: {images: local}`) it is pinned by `tag`, by `digest` (the image's config id) or both |
| `claims` | per unit name: `cpu_millis`, `memory_mib`, `accelerators` (500, 512 and 0 when absent; a model unit takes one accelerator) |
| `checks` | per unit name, the `port` of its `startup`, `ready` or `alive` check: `{core: {ready: {port: 8080}, alive: {port: 8080}}}`. A unit has only the checks of its kind (below) |
| `ingress` | per ingress of the contract and per protocol it exports, where it listens: `{operator: {exports: {http: {port: 8080, bind: all}}}}`. `port` (1 to 65535), `bind` (`loopback`, the default, or `all`), `address` (the address it is reached at, text) and `part` (a part of a unit that serves it, when not the control plane of the core) |
| `ui` | the UIs to deploy as units of their own: `{operator: {port, bind, upstream: {unit, port}}}`, every key optional (`ui: {operator: {}}`). See [the operator UI in a bundle](#the-operator-ui-in-a-bundle) |
| `packing` | the packing section below |

A binding never changes the architecture: promoting a revision to another target is a new binding
and a new bundle of the same architecture.

## Secrets and egress

A unit is granted only what its parts declare, each by name. Under `parts:`, `secrets` lists the
secret names a part needs, and `egress` lists the destinations it reaches: a part of the
architecture (an external part, or a part in another unit), or a name the target resolves later
(a host, a registry). An egress entry is the destination's name, or
`{to: <name>, secret: <secret name>, address: <address>}`: `secret` when its link authenticates
with a secret, which a part of the same unit must declare; `address`, for a destination outside
the architecture, where the target finds it: a CIDR block (`10.0.0.0/8`) or `<host>:<port>`, or
`{namespace: <text>, labels: {<text>: <text>}}`. An external part is not given an address here:
its address is its `reach` (see [packing](#packing)).

```yaml
parts:
  gateway:
    secrets: [model-api-key]
    egress: [{to: model, secret: model-api-key}]
  operator:
    secrets: [chat-token]
    egress: [gateway, owner, worker, {to: chat-relay, secret: chat-token, address: "chat-relay.example:443"}]
```

Every link that leaves a unit needs its destination in the `egress` of the part it starts at, also
under the default packing: a link from a part of the core to an external part, such as a gateway
calling a hosted model, is egress. A binding that names no architecture declares no egress, so it
fits only an architecture whose links stay inside one unit; give each architecture a binding that
`refines:` it and declares the egress.

A secret is a name, never a value. A **secret name** is text of lower-case letters, digits and
hyphens that starts with a letter (`^[a-z][a-z0-9-]*$`), at most 63 characters, with no word
between hyphens longer than 20 characters (a key or a token is one long run, a name is words).
Admission refuses as `secret_value`:

- a secret in `secrets`, or an egress entry's `secret`, that is no secret name: a mapping, a
  number, capitals, `=`, `:`, `/`, a space, a long run of characters;
- a `secrets` that is not a list;
- a key `value`, `values`, `data`, `string_data`, `literal`, `password` or `token` at the top of
  the binding, in `target`, a part's entry, an egress entry, a runtime or a claim.

A finding says where the value is, never what it is.

## Packing

A **unit** is one thing a target starts and watches. Each part of the architecture is in exactly
one unit, or is `external`: not started by the target at all.

| unit kind | lives | holds |
|---|---|---|
| `core` | long, exactly one | the event log, the inner behaviour of the blocks, the control plane and the parts no other unit takes |
| `agent` | long, replicas | agent parts, with the parts that are `per` one of them (workspace, sandbox) |
| `service` | long, addressable | a gateway, a pool, a tool, a runtime |
| `job` | until done, started on demand | an agent part marked `schedule: job` (or `shape: job`) that no clock wakes |
| `scheduled` | until done, started by a clock | an agent part marked `schedule: job` that a schedule or an `on_schedule` trigger wakes, or given a cron rule (`schedule: <cron>`) |
| `store` | long, keeps state | a shared workspace |
| `model` | long | a model part served by the target itself (marked `external: false`) |

Principals and model parts are `external` unless marked otherwise; any part may be marked
`external`. The target reaches an external part by its `reach`: `{name: <text>}`, a name it
resolves, or `{address: <text>}`; by the part's own name when the binding gives none.

The `packing:` section names a **strategy**, or lists the units:

| strategy | units |
|---|---|
| `single` (the default, also when the section is absent) | every part in `core`, external parts outside |
| `by_kind` | one unit per block that is not held by the core: all agents in one unit, each kind of service in one |
| `by_part` | one unit per part; a part's `count` becomes the unit's replicas |
| `groups` | the named lists of `groups:` are units; the rest is in `core` |

Per part, `parts: { <part>: { schedule: job } }` (or `shape: job`) puts an agent into a `job` or
`scheduled` unit of its own; `schedule: <cron>`, a cron rule of five fields (`"0 2 * * *"`), puts it
into a `scheduled` unit that carries the rule, which counts as its clock; `external: true` or `external: false` overrides where a part goes, and `reach` says where
an external part is reached. A part that is `per` another goes with it. A `count` marked for optimization is packed at its built value.

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
that runs as copies, `schedule` for a scheduled unit with a cron rule) and `external:` the parts outside, each a part's name or `{part: <name>, reach:
{...}}`; `strategy`, `groups` and `parts` do not go with it. A section of the wrong form (an unknown key, strategy, kind or part, two units of one
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
| `network_isolation_unmet` | a part is marked `network: isolated` and the target says `network_policy: not_enforced` |
| `scheduled_with_inputs` | a `scheduled` unit takes an input other than its clock, or neither a clock nor a cron rule of the binding wakes it |
| `core_count` | the packing has not exactly one `core` unit |
| `unit_not_hosted` | a unit's kind or one of its roles is not in the accepted list |
| `agent_document_missing` | an agent, job or scheduled unit holds an agent part the binding names no readable agent document for |
| `agent_document_digest` | an agent document is not the digest the binding pins |
| `image_unpinned` | a runtime from the target's local store names neither a `tag` nor a `digest` |
| `volume_mount_missing` | a unit claims a workspace's volume and no part of the unit mounts it, or mounts it at a path that is not absolute; there is no default path |
| `secret_value` | something that looks like a secret's value stands where a secret name belongs (see [secrets and egress](#secrets-and-egress)) |
| `secret_undeclared` | an egress entry, or in a written bundle a unit, names a secret that no part of its unit declares |
| `egress_undeclared` | a link leaves its unit, for an external part or a part of another unit, and the part it starts at does not declare that destination in its `egress` |
| `address_in_architecture` | the architecture names an address: a key `address`, `host`, `hostname` or `url` anywhere in it (see [the bundle](#the-bundle)) |
| `ui_upstream_unknown` | the UI the binding asks for reads the adapter of a unit (`ui.<ui>.upstream.unit`) the packing does not have |

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
| `packing` | the resolved packing, its strategy and digest; its `external` lists each part outside as `{part, kind, reach}`, `kind` the part's block and `reach` the binding's or `{name: <part>}` |
| `units` | one entry per unit, below |
| `ingress` | one entry per export of each ingress of the contract: `ingress`, `protocol`, `audience`, `authenticated`, the `unit` and `part` that serve it (`control`, the control plane in the core unit, unless the binding names a part), `bind` (`loopback` unless the binding says `all`), and `port` and `address` when the binding gives them. An entry without `port` has no entry point on the target |
| `requirements` | the architecture's `requirements:`, empty when it has none |
| `agents` | each agent document the binding names, keyed by the digest of its file's bytes: the parts it plays, `content_base64`, those exact bytes in standard base64, and `document`, the same document parsed |
| `runtimes` | each runtime image the binding pins. From a registry: keyed by its digest, with its name and its image reference `<image>@sha256:<digest>`. From the target's local store: what the binding gave, its name, `image`, and `tag`, `digest` or both, keyed by the digest, else by `<image>:<tag>` |
| `files` | only when the binding asks for a UI: each file a unit names, by its file name, with its `digest`, its `size` in bytes and its bytes in `base64`. A UI unit's archive is carried here |
| `digest` | the bundle digest, of every member but this one |

Each unit:

| field | holds |
|---|---|
| `name`, `kind`, `parts` | the unit, its kind and its parts. The packing's `agent`, `job` and `scheduled` units are all of kind `agent` here, told apart by `shape` |
| `shape` | an agent unit only: `service` (long-running, the packing's `agent`), `job` (until done, started on demand) or `scheduled` (until done, started by a clock) |
| `replicas` | the copies that run; absent for a `job` or `scheduled` unit |
| `code` | what runs. An agent, job or scheduled unit stays an agent: the role `agent-runtime` and its agent documents by digest, never a process command, and the pinned agent runtime as `<image>@sha256:<digest>` when the binding pins one (`runtimes.agent-runtime`) (from a local store `<image>:<tag>`, else its config id). The core and service units run the pinned core runtime, by the same reference, or name the role `core-runtime`. A UI unit serves static files: `{kind: static, archive, digest}`, the file name of its archive in `files` and its `sha256:` |
| `claims` | `cpu_millis`, `memory_mib`, `accelerators`, and `volumes`: one per workspace, with its access (`read_write_once` when exclusive, `read_only_many` when shared read-only, `read_write_many` otherwise), whether there is one per copy of its part, its size when the binding gives one, and its `mount`, the absolute path a part of the unit mounts it at |
| `isolation` | the highest isolation its sandboxes ask for, in agenticspec's own levels |
| `checks` | how the target sees it start and stay up: an agent, job or scheduled unit has a `startup` check (its agent runtime checks its agent document before it runs); the core and service units answer `/ready` and `/alive` over HTTP; a model unit has a `ready` check. Each check carries the `port` the binding gives it |
| `schedule` | a scheduled unit's cron rule, when the binding gives one (`packing.parts.<part>.schedule`) |
| `clock` | a scheduled unit's clock in the architecture: the trigger and schedule that wake it, `every`, `next_fire_at`, and the trigger's `policy` and `max_parallel` as the rule for overlapping runs |
| `upstream` | a UI unit only: the unit whose adapter the page reads (`unit`), its `port` when known, and `path`, the `api_base` of the UI's `config.json` (`/api/`) |
| `secrets` | the secret names its parts declare, never a value |
| `egress` | every destination its parts declare, each with the part it starts at (`from`), the secret it authenticates with, when declared (`secret`), and `name`, what the target resolves: a link to another unit, with its transport, named by that unit; a link to an external part (`external: true`), named by its reach (its name, or its address, which is then also `address`); or a destination outside the architecture (`named: true`, no link), named by its own name, with the `address` the binding gives it, else `unresolved: true`. Nothing that is not listed is granted |
| `roles` | what the unit needs from its target, never a provider: `agent-runtime`, `core-runtime`, `start-jobs` (the core, when a job unit exists), `model-server`, `store`, `model-route:<route>` for each hosted model route it calls, `operator-ui` for the operator UI's unit |
| `ports` | the contract ports its parts use, by their derived names, with their settings and instances |
| `source` | the unit and its parts, carried into what the translator makes of it |

The bundle carries no secret value, host, provider or target name. It copies the architecture as
written, so an address in the architecture would land in the bundle: a key `address`, `host`,
`hostname` or `url` anywhere in the architecture document is refused as `address_in_architecture`,
by its path. Where a part is reached is the target's, written in the binding: a destination a part
reaches outside the architecture is a name in its `egress` that the target resolves, and the
address of what the system exports goes into the binding with the export, never into the
architecture.

## Checking a bundle

`agenticspec deploy check <bundle> [--accepts <file>]` admits a written bundle again from what it
carries, for a reader that did not write it. It refuses with `bundle_invalid` a bundle that is not
`agenticspec-deployment/1`, lacks a member, whose digests do not hold (the bundle's, the
architecture's revision, the contract's, the binding's, the packing's) or whose binding or packing
is of the wrong form (a check, a claim or an ingress export the packing or the contract does not
have among them). Otherwise it admits the architecture under the binding and packing the bundle
carries, checks that each agent unit carries the agent documents of its agent parts at the digests
the binding pins and that each agent document's `content_base64` decodes to bytes whose SHA-256 is
its key and which parse to its `document` (`agent_document_mismatch`), and checks the secrets and egress of the binding and of every unit: a unit's
secret or an egress entry's secret that no part of the unit declares, an egress entry whose part
does not declare its destination, and a secret value, each by its name. It also refuses what the
target cannot be asked to serve: a volume without an absolute mount (in the binding or in a unit),
a local runtime image with neither tag nor digest, and a part isolated on the network of a target
that enforces no network policy. Of a UI unit it hashes the archive the bundle carries and refuses
an archive that is missing or whose SHA-256 is not the unit's `digest` (`ui_archive_mismatch`),
and an `upstream` that names no other unit of the bundle (`ui_upstream_unknown`). A refusal exits
1 with `not_admitted` and names every finding.

```sh
agenticspec deploy check workstation.bundle.yaml
```

## The operator UI

`agenticspec ui build <architecture> --contract <file> --out <dir> [--api-base <path>]
[--archive]` writes the operator UI of an architecture as static files, from the binary alone: no
node and no app of your own. The directory holds:

| file | holds |
|---|---|
| `index.html`, `app.js`, `app.css` | the operator app, built into the binary, the same for every architecture |
| `views.json` | the layout of the UI, the pages and panels `agenticspec ui derive` derives from the architecture and its contract |
| `contract.json` | the contract, as JSON |
| `ports.json` | the port catalogue the binary carries |
| `config.json` | the architecture, the title, and `api_base`: where the runtime's adapter answers the runtime interface (`/api/` unless `--api-base` names another path or URL) |

The app reads these files when it starts and calls the adapter at
`<api_base>/scopes/<architecture>/<part>`. Serve the directory as it is; `config.json` may be
edited where it is served. The contract must come from the same ports as the binary
(`agenticspec sim contract` of the same version); another is refused. The directory must be new,
empty or written by an earlier `ui build`, which is replaced. The same inputs give the same bytes.

`--archive` also writes `<dir>.tar`, the same files with sorted entries, mode 0644, owner 0 and
time 0, and prints its SHA-256 as `archive_sha256`, the digest a bundle carries for the archive.

```sh
agenticspec sim contract workstation.yaml --out workstation.contract.yaml
agenticspec ui build workstation.yaml --contract workstation.contract.yaml --out ui --archive
```

### The operator UI in a bundle

A binding with `ui: {operator: {...}}` makes `deploy bundle` add the operator UI as a unit of its
own. It builds the UI in-process with the library `ui build` uses, from the architecture and its
contract, and the archive is the same bytes as `ui build --archive` writes for them (`api_base`
`/api/`). Without `ui:` the bundle is the same as before.

| key under `ui.operator` | holds |
|---|---|
| `port`, `bind` | where the UI's export listens when the binding's `ingress.operator.exports.ui` gives none. Given in both places, they must agree |
| `upstream.unit` | the unit whose adapter (the runtime interface, `ui/runtime.openapi.yaml`) the page reads; the unit that serves the operator ingress's `http` export (the core) when absent |
| `upstream.port` | its port; the `http` export's port when absent and that export is served by the same unit |

The bundle then has:

- a unit `operator-ui`, `kind: service`, `roles: [operator-ui]`, no parts, `code: {kind: static,
  archive: operator-ui.tar, digest: sha256:<hex>}`, `upstream: {unit, port, path: /api/}`, and
  `ready` and `alive` checks over HTTP at `/` on the export's port;
- the archive under `files: {operator-ui.tar: {digest, size, base64}}`, inline like every other
  member;
- the `ingress` entry of the operator ingress's `ui` export naming `unit: operator-ui` and `part:
  control`; its other exports stay where they were.

`deploy bundle` refuses, with `binding_invalid`, a `ui:` of the wrong form, a contract whose
operator ingress exports no `ui`, a packing that has a unit named `operator-ui`, and a `part` on
the `ui` export; with `not_admitted`, an upstream that names no unit of the packing
(`ui_upstream_unknown`), a role `operator-ui` the accepted list does not hold
(`unit_not_hosted`), and a UI that does not build (`ui_build_refused`, for example a contract
from other ports than the binary's). `bundles/bindings/operator-ui.yaml` is the worked example.

## Fixture bundles

`bundles/` holds a bundle of each catalog architecture under each test binding of
`bundles/bindings/` (`bundles/<architecture>.<binding>.yaml`), with the agent documents they name
and `bundles/sources.sha256`, one digest of the sources of each bundle. A binding that names no
architecture is bundled with every catalog architecture no binding of the same name names:
`local.yaml` is the base of each `local-<architecture>.yaml`, which declares the egress of that
architecture's parts. `task bundles:generate`
writes them, `task check:bundles` (in the gate) fails when their sources changed, and `task
slow:bundles` builds them afresh and compares bytes.
