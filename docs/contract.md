# The contract of an architecture

A runtime that brings an architecture into existence implements a few commands of the building
blocks and gets the rest from the specification. The commands it implements are grouped into
**ports**. The **contract** of an architecture document lists the ports its parts use, with the
settings that narrow each port, how many instances run, and what each link promises about a run.
Both are derived from files in this repository; nothing here is written by hand. A contract cuts
the conformance suite of the blocks to what a runtime of the architecture must pass, and its
routing invariants are checked on a trace of a run.

## Ports

The building blocks (`blocks/`) are one ESS system. Most of their commands record what happened
inside the specification. A port is where the specification waits on the world: one command states
an intent, and later commands report what the world did. A port has a name (`<block>.<entity>`),
a kind, the event of its intent, the state the entity waits in, the commands that report, and a
protocol: the order those commands may come in.

`agenticspec sim ports` derives the ports from the compiled system
(`ess specify compile --path blocks --format json`):

- A command is **internal** when a binding in `blocks/components.yaml` invokes it, an **entry**
  otherwise. An entry the actor `agentic.architecture.Deployer` may issue is **configuration**:
  the deployer issues it from the architecture document, and no runtime implements it.
- An entity **waits** in a state that commands enter, that no internal command leaves, and that
  some command other than configuration leaves or reports into. A terminal state that still has
  a way out (a message delivered and maybe read) waits too, and the protocol may end in it.
- A **port** is an entity with a waiting state. Its protocol is the language of its lifecycle from
  the commands that enter a waiting state to the end, over the commands' short names.
- **Kinds.** A port is a **principal** port when a principal reports into it (a command the actor
  `agentic.architecture.Principal` may issue: `Answer`, `ApproveDecision`); otherwise
  **outbound**. A periodic binding gives a **clock** port. A command that creates a row of
  another entity in a final state and guards on the waiting entity's state (a turn of a session,
  a step of a run) reports into that entity's waiting states.

ESS has no kind of actor, so the derivation reads the two actors by name; the blocks declare them
in `blocks/9-composition/architecture.yaml`.

### The ports file

`blocks/ports.json` (`agenticspec-ports/1`) holds the derived ports:

| key | meaning |
|---|---|
| `format` | `agenticspec-ports/1` |
| `ess` | the `ess --version` line the ports were derived with |
| `sources_sha256` | SHA-256 over the `ess` line, then `<sha256> <path>` for `blocks/ess-inputs.yaml` and every file it lists |
| `bindings` | each binding of the blocks and the command it invokes: the commands no runtime is asked for |
| `ports` | each port: `name`, `block`, `kind` (`Outbound`, `Clock`, `Principal`), `intent_event`, `waiting_state`, `reports`, `protocol` |
| `actors` | each actor of the blocks and the commands its `may:` lists |
| `domains` | each domain of the blocks: its `commands`, each with its role (below), its `views` and its `events` |

The role of a command is the first of these that applies:

| role | the command |
|---|---|
| `internal` | a binding invokes it |
| `report` | reports into a waiting state of a port (`CompleteCall`, `Answer`) |
| `intent` | states the intent of a port: a first command of its protocol (`SendMessage`, `AdmitCall`) |
| `configuration` | an accepting outcome creates its entity (`OpenGateway`, `RegisterPrincipal`): set up from the architecture document only |
| `administration` | every accepting outcome moves or updates an existing entity (`DrainGateway`, `StopGateway`, `RevokeGrant`): an operator issues it at run time |
| `entry` | any other command |

An ingress reads the roles, the views, the events and the actors from this file, so its check
calls no `ess`.

```sh
task ports:generate                                     # derive blocks/ports.json (calls ess)
agenticspec sim ports --check blocks/ports.json         # digest only: exits 1 when the blocks changed
agenticspec sim ports --expect <expected.yaml>          # one line per difference from an expected list
```

`task check:ports` runs `--check` in the gate; `task slow:ports` derives afresh and compares bytes.

### Protocols

A protocol is a regular expression over command names: names separated by spaces follow each
other, `|` separates alternatives, `*` repeats zero or more times, `+` one or more, `?` makes
optional, and parentheses group. `SendMessage (MarkDelivered MarkRead? | MarkLost)` is the
protocol of a direct message.

## The contract

`agenticspec sim contract <architecture.yaml>` derives the contract of one architecture document
from the document and `blocks/ports.json`, checks it, and prints it (`jq -r .contract`) or writes
it with `--out <file>`. A document `agenticspec sim check` refuses gets no contract: the command
exits 1 with the check's refusals and writes nothing. `--check <file>` reads a written contract
and checks it against the document and the ports. `--kpis <file>` names the KPI catalogue the
metrics of its [ingresses](#ingresses) are resolved against (`kpis/catalogue.yaml` by default);
every document has the [default ingresses](#the-default-control-plane), so every derivation
reads it.

Each [architecture page](architectures/index.md) and [catalog entry](catalog/index.md) shows its
contract in a section "Contract", with the contract file to download
(`website/static/architectures/<name>.contract.yaml`, `<entry>.contract.yaml` in the entry's
downloads).

### Which part uses which port

A part uses the ports of its block:

| block | ports |
|---|---|
| `agent` | `harness.session` |
| `harness` | `harness.session`, `harness.tool-call` |
| `model` | `model.inference` |
| `gateway` | `gateway.call` |
| `pool` | `resource.claim` |
| `channel` | `channel.message`; on a topic, queue or board: `channel.post`, `channel.subscription` |
| `workspace` | `workspace.lease`, `workspace.proposal` |
| `sandbox` | `sandbox.sandbox` |
| `schedule` | `trigger.schedule`, and `trigger.firing` unless a trigger part fires on it |
| `trigger` | `trigger.firing` |
| `authority` | `authority.decision`; with `kind: budget`, `authority.budget` |
| `runtime` | `runtime.instance` |
| `principal`, `tool`, `knowledge`, `work`, `experiment`, `kpi` | none: the specification runs them |

The table holds exactly the part kinds of the architecture block (`agentic.architecture.PartKind`).
`agenticspec sim check` refuses a part of any other block (`process` or `question`, for example)
as "no part kind", so every document it accepts derives its contract.

A link adds the port its carried value travels in to its `from` part:

| carried value | `from` uses |
|---|---|
| `model_call` from an agent or a runtime | `model.inference` |
| `model_call` from a gateway | `gateway.call` (its own) |
| `question` | `question.question`; a principal it reaches uses it too and answers |
| `call` from an agent to a tool | `harness.tool-call` |
| `claim` to a pool | `resource.claim` |
| `message` | `channel.message` |
| any value sent to a channel part | the channel's sending port (`channel.post` on a topic, queue or board, else `channel.message`); `read` from a shared channel: `channel.subscription` |
| `proposal` from an agent | `workspace.proposal` |
| `order` or `run` to a runtime | `runtime.instance` |
| `trigger` from a schedule or trigger | `trigger.firing` (its own) |
| `grant` from an authority | its own decision or budget port |
| `checkout` from a workspace | `workspace.lease` (its own) |
| any value `over` a channel part | that channel's sending port, as well |

Every other value (`order` to an agent, `review`, `result`, `handoff`, `report`, `spec`,
`context`, `record`, a `prompt` not over a channel) travels in no port.

### Settings, instances, border

- **Settings.** Per block and port a fixed list of setting names is copied from the using part,
  the value of a marked place (`{value: 12, vary: …}`) taken: agent `harness`, `role`,
  `acts_for`; model `route`; gateway `rate_limit_per_minute`; pool `kind`, `capacity`,
  `job_limit`; channel `kind`, `distribution`; principal (question port) `kind`; trigger and
  schedule (firing port) `kind`, `every`, `schedule`, `channel`, `policy`, `max_parallel`,
  `event_name`, `source`; schedule (schedule port) `every`, `next_fire_at`; workspace `kind`,
  `sharing`; sandbox `isolation`, `egress`; harness, authority and runtime `kind`. A port used
  through a link takes no setting of its part (an agent's `model_share` among them), except a
  principal's `kind`.
- **Instances.** The part's `count` (1 when absent) times the count of the part it is `per`,
  along that chain: in `architectures/workstation.yaml`, `tree: { per: worker }` with 12 workers
  provides 12 workspaces.
- **Border.** The parts of the document's `border`: the only parts that take input from outside.

### Routing invariants

A link whose value travels in a port of its `from` part has one invariant on that port, written in
the predicate language below. A `model_call` from an agent or a runtime to a gateway nests the
inference in the gateway's call; every other such link restricts the counterparty, and names the
channel it is carried over:

```text
worker -> gateway carries model_call            every model.inference@worker inside gateway.call@gateway
operator -> owner carries question over chat    every question.question@operator to owner over chat
worker -> builds carries claim                  every resource.claim@worker to builds
```

A link whose value travels in no port has no invariant.

### The predicate language

```text
predicate = clause { "and" clause }
clause    = "every" use "inside" use
          | "every" use "to" part [ "over" part ]
use       = port "@" part
port      = word "." word
word      = a lowercase letter, then lowercase letters, digits and "-"
part      = a letter or "_", then letters, digits, "_", "-" and "." (`<part>.<inner part>`)
```

Words are separated by spaces. `every A@p inside B@q`: each use of port `A` by part `p` lies inside
a use of port `B` by part `q`. `every A@p to q over c`: each use of `A` by `p` has `q` as its
counterparty, carried over channel part `c`. A predicate holds when all its clauses hold. A text
that does not parse is refused naming the byte offset and what was expected there
(`every model.inference@ to x`: `expected a part at offset 22`).

### The format

```yaml
format: agenticspec-contract/1
architecture: workstation
ports_digest: <sources_sha256 of blocks/ports.json>
ports:
  - port: sandbox.sandbox
    part: box
    protocol: "DefineSandbox (CheckEgress | CheckSecretRead | CheckWrite)*"
    instances: 12
    settings: {egress: [gateway], isolation: process}
invariants:
  - link: "worker -> gateway carries model_call"
    port: model.inference
    within: gateway.call
    predicate: "every model.inference@worker inside gateway.call@gateway"
border: [operator]
ingress:
  - name: metrics
    audience: monitor
    authenticated: false
    exports: [metrics_scrape]
    operations:
      - {kind: metric, name: cost_usd}
```

Ports are ordered by part, then port; invariants follow the document's links; ingresses by name,
their operations by kind (command, view, feed, metric), then name. `ingress` lists the document's
ingresses and the default ingresses it has; it is left out only for a document that opts out of
every ingress. The same document, ports file and KPI catalogue give the same bytes.
The contract is the content of `agentic.architecture.Contract` in the building blocks.

### The check

A contract passes when its `ports_digest` is the ports file's digest, every port it names is in
the ports file with the same protocol, every part, link and border part is in the document, and
every invariant names a port its link's `from` part uses and has a predicate that parses and
starts with that use. Its ingresses are the document's and the defaults it has, with the same
audience, `authenticated` and exports. `sim contract` checks every contract it derives before
printing it.

## Ingresses

The border says which parts take input from outside. An **ingress** says who reaches the
architecture from outside, what that audience may use, and over which protocols. A document
declares its ingresses under a top-level `ingress:` key; beside them it has the two
[default ingresses](#the-default-control-plane) unless it replaces or opts out of them.

```yaml
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
    exports: [{ protocol: http }, { protocol: tool }]
  metrics:
    audience: monitor
    authenticated: false
    interface:
      scope: all
      metrics: [owner_wait_share, cost_usd]
    exports: [{ protocol: metrics_scrape }]
```

[`architectures/workstation-ingress.yaml`](architectures/workstation-ingress.md) is this example
in full.

### An ingress

| key | meaning |
|---|---|
| `audience` | the principal part the ingress serves, or one of the implied audiences `control.operator` and `control.monitor`. A part only an ingress reaches is not refused as unreachable |
| `authenticated` | whether callers authenticate; `true` by default. `false` only for an interface without commands |
| `interface` | what the audience may use (below) |
| `exports` | one or more `{protocol: …}`, each protocol at most once |
| `rate_limit_per_minute` | a limit of the ingress; it may be marked with `vary` |
| `summary` | text |

### The interface

An interface only selects. It names no field, no type and no protocol; the fields of each
operation come from the building blocks.

| key | values | selects |
|---|---|---|
| `scope` | `linked`: the parts the audience links to (`to` and `over` of its links); `all`; or a list of parts | the domains the rules below read: each part's own block's domain and the domains of the ports it uses in the contract |
| `commands` | `principal`; `{principal: all}`; `{plays: [<actor>]}`; `{plays: administration}`; `{except: [<command>]}`; or a list of these | `principal`: the answers of the principal ports the audience uses (the reports the actor `agentic.architecture.Principal` may issue: `Answer`) and the intent of each input the audience sends into the border (`owner -> operator carries prompt over chat`: `SendMessage`). `principal: all`: the answers of the principal ports of every principal part in scope; the grant decides for whom a caller answers. `plays`: the `administration` commands in the actor's `may:`. `plays: administration`: the same for every actor of a domain in scope whose `may:` holds no report of a port |
| `views` | `true`, `false` (default), or `{except: [<view>]}` | every view of the scope's domains |
| `feeds` | `true`, `false` (default), or `{except: [<event>]}` | every event of the scope's domains |
| `metrics` | KPI ids of `kpis/catalogue.yaml`, or `computed` | each KPI, whose block is a block of the scope. `computed`: every KPI of a block of the scope whose status is `computed` or `both`, which a runtime computes from the events it observes |

Configuration commands never enter an interface: the document, through a new revision, is their
only source.

### Exports

| protocol | carries |
|---|---|
| `http` | commands, views |
| `event_stream` | feeds |
| `cli` | commands, views |
| `tool` | commands, views |
| `agent` | commands, feeds |
| `metrics_scrape` | metrics |
| `ui` | commands, views, feeds, metrics |

Every operation kind of the interface is carried by at least one export. A different subset for
another protocol is another ingress.

### The check of ingresses

`agenticspec sim check` refuses a part on the reserved path `control` in every document. For a
document with `ingress:` it reads `blocks/ports.json` (`--ports`) and `kpis/catalogue.yaml`
(`--kpis`), and refuses, naming the rule:

| rule | refused |
|---|---|
| `ingress_malformed` | a key, value or shape the format does not have |
| `ingress_reserved_part` | a part named `control`, or under it: the path of the implied audiences |
| `ingress_none_not_default` | `none` for an ingress that is no default (`operator`, `metrics`) |
| `ingress_reserved_audience` | an audience under `control.` other than `control.operator` and `control.monitor` |
| `ingress_audience_not_principal` | an audience that names no principal part |
| `ingress_without_export` | an ingress with no export |
| `ingress_unknown_protocol` | a protocol outside the table above |
| `ingress_unknown_part` | a part in a `scope` list that does not exist |
| `ingress_unknown_operation` | a name in `plays`, `except` or `metrics` the blocks or the catalogue lack |
| `ingress_actor_is_implementation` | `plays` names an actor whose `may:` holds a report of a port that is not a principal port (the implementation issues those) |
| `ingress_outside_scope` | an actor or a KPI whose domain or block is not in the scope |
| `ingress_command_off_border` | an input of the audience (a `principal` command) that reaches a part not on the border |
| `ingress_operation_not_carried` | an operation kind no export of the ingress carries |
| `ingress_ui_without_wire` | a `ui` export without `http` and `event_stream` beside it |
| `ingress_unauthenticated_command` | `authenticated: false` beside a command |
| `ingress_agent_without_channel` | an `agent` export whose audience sends no input into the border, or one over no channel part |
| `principal_port_without_ingress` | a principal port a principal part uses that no ingress answers over an export that carries commands: none of that part's audience with `principal`, none with `principal: all` over a scope that holds it. The default `operator` answers every one, so this bites once it is opted out or narrowed |

A document refused here gets no contract. The contract's `ingress` lists each interface resolved
into operations, `{kind, name}`.

### The default control plane

Every architecture brings what its operator needs to watch and steer it: two ingresses, by name.

```yaml
ingress:
  operator:
    audience: control.operator
    interface:
      scope: all
      commands: { principal: all, plays: administration }
      views: true
      feeds: true
    exports: [{ protocol: http }, { protocol: event_stream }, { protocol: ui }, { protocol: tool }]
  metrics:
    audience: control.monitor
    authenticated: false
    interface: { scope: all, metrics: computed }
    exports: [{ protocol: metrics_scrape }]
```

- A document has each of them unless it declares an ingress of that name, which replaces it, or
  sets it to `none` (`operator: none`). `ingress: none` removes both.
- Their audiences, `control.operator` (a human) and `control.monitor` (a service), are implied:
  principal parts of the contract only, never of the document, so they change no digest and no
  simulation result. A runtime binds them to identities like any principal part. No part of a
  document may take the path `control`.
- The operator answers every principal port, issues every administration command of the scope
  and reads every view and event of it; configuration commands never enter. The metrics ingress
  carries the KPIs a runtime computes.

`agenticspec ingress default <architecture.yaml>` prints the `ingress:` section of the defaults a
document has (`jq -r .ingress`): paste it into the document and change it. Pasted unchanged, the
document has the same contract. A document that takes a reserved name is refused
(`architecture_refused`).

### Projections of an http export

`agenticspec ingress project <architecture.yaml> --out <dir>` writes, for each ingress with an
`http` export, two files:

| file | what |
|---|---|
| `<ingress>.http.yaml` | the `ess-http/1` protocol binding over exactly the interface's commands (`post`) and views (`get`), pinned to the compiled blocks by their source digest. Every path is open: a server binding that `refines:` it fills them in |
| `<ingress>.openapi.yaml` | the OpenAPI document of those operations: ESS's document of the component that serves the blocks, narrowed to them and the schemas they reference. Its paths are the ones ESS derives |

It calls `ess` to compile the blocks (`--blocks`, `blocks` by default), to write their OpenAPI
document and to validate each binding, and gives the same bytes on every run. `--check <dir>`
compares the files in `<dir>` with a fresh projection and writes nothing: it exits 1
(`projections_stale`) naming each changed, missing or extra file. The projections of
[`workstation-ingress`](architectures/workstation-ingress.md) are kept in the repository and
checked by the gate.

### Nested architectures

A contract is one document's. When a document names another architecture as a part, each
document gets its own contract, and the outer one composes them: the inner contract's ports are
used by the outer part, their instances multiplied by that part's instances; the inner border
parts are where the outer links into the part arrive, so an outer link's invariant names a port of
an inner border part; and the inner invariants hold unchanged, their part names prefixed with the
outer part.

## Block conformance of a contract

A runtime of an architecture is held to the behaviour of the blocks whose ports its contract
lists. The conformance suite of the building blocks (`task check:blocks` synthesizes it once per
state of `blocks/` and keeps it in `target/blocks-conform/<digest>.json`) can be cut to exactly
those scenarios:

```sh
agenticspec sim contract architectures/workstation.yaml \
  --suite target/blocks-conform/<digest>.json --out workstation.suite.json
task contract:conform -- architectures/workstation.yaml   # cut the cached suite and run it
```

`--suite` keeps a scenario when the command it is about is a command of a port of the contract
(a command of the port's protocol or one that reports into it) and drops the rest; `--out` names
the cut suite, which keeps the whole suite's provenance and events. The result counts the
scenarios kept (`kept_scenarios`) against the whole suite (`suite_scenarios`). A scenario is about
the command its id names:

| scenario id | about |
|---|---|
| `<command>/outcome/<outcome>` | the command |
| `<entity>/transition/<transition>/by/<command>/<outcome>` | the command |
| `<entity>/state/<state>/refuses/<command>` | the command |
| `<entity>/invariant/after/<command>/<outcome>` | the command |
| `<binding>/binding/<case>` | the command the binding invokes |
| any other (an authored scenario, an invariant at a view) | the command of its last `execute_command` step |

Each scenario carries its own setup steps, so a kept scenario runs alone, and `ess verify conform
run` admits the cut suite as it is: the cut suite is what runs, not the whole suite filtered by
scenario id afterwards. `task contract:conform` cuts the cached suite and runs the cut on the
interpreter (`ess verify conform run --target interpreted`), prints each kept scenario with its
result and `kept <n> of <m> scenarios`, and exits 1 when a kept scenario fails. For
`architectures/workstation.yaml` it keeps the scenarios of inference, gateway calls, harness
sessions, questions, build claims, messages, workspaces and sandboxes, and none about runtimes,
triggers or posts on a shared channel.

## Traces and the routing check

The routing invariants of a contract are checked on a trace in port terms: each use of a port by
an instance of a part, with its begin, its end, the use it lies inside and its counterparty. The
simulator writes one for a single run:

```sh
agenticspec sim trace architectures/workstation.yaml --profile fields/coding/profiles/coding.yaml \
  --seed 1 --out workstation.trace.json
agenticspec sim contract architectures/workstation.yaml --trace workstation.trace.json
```

`sim trace` simulates the architecture once at the values it is built with (each marked place at
its `value`), for one day unless `--horizon` names another duration, with seed 1 unless `--seed`
names another; the same seed writes the same bytes. Recording changes no result of the run.

### The trace format

```json
{
  "format": "agenticspec-trace/1",
  "architecture": "workstation",
  "source": {"simulator": "agenticspec-sim", "profile": "coding", "seed": 1, "horizon_seconds": 86400.0},
  "records": ["channel.message", "channel.post", "gateway.call", "model.inference", "question.question", "resource.claim", "trigger.firing"],
  "events": [
    {"index": 0, "time": 12.5, "part": "gateway", "instance": 0, "port": "gateway.call", "phase": "begin", "to": "model"},
    {"index": 1, "time": 12.5, "part": "worker", "instance": 3, "port": "model.inference", "phase": "begin", "within": 0, "to": "model"},
    {"index": 2, "time": 40.1, "part": "worker", "instance": 3, "port": "model.inference", "phase": "end", "begin": 1},
    {"index": 3, "time": 40.1, "part": "gateway", "instance": 0, "port": "gateway.call", "phase": "end", "begin": 0}
  ]
}
```

| key | meaning |
|---|---|
| `records` | the ports whose every use is in the trace |
| `index` | the event's place, from 0 |
| `time` | seconds from the start of the run |
| `part`, `instance` | the part that uses the port, and which of its copies (0 for a part of one) |
| `port`, `phase` | the port, and `begin` or `end` of the use |
| `within` | on a begin: the index of the begin of the use it lies inside |
| `begin` | on an end: the index of the begin it ends |
| `to`, `over` | on a begin: the part on the other side of the use, and the channel part it is carried over |

Events are ordered by time; an end ends an earlier begin of the same port, part and instance, at
most once, and a use without an end is still open when the trace ends. A trace that breaks one of
these is refused with the index of the event.

The simulator records model calls (the gateway's call, and the inference inside it), builds in a
pool (a claim from its request to the end of the build), questions (the asker's, and a delegate's
forward to a principal inside it, carried over the delegate's channel), the delivery of a
forwarded question and of its answer on that channel, posts to a shared channel (a post and a
delivery have no duration), and the firing of a trigger that starts a run (from the firing to the
start of the run). It records no tool calls, workspace leases, sandbox checks or channel reads.

### The check

`sim contract --trace` checks every invariant of the contract on the trace and prints one line
each: `held`, `violated` or `not_checked`, the link, and the number of uses it read.

- `every A@p inside B@q` holds when each use of `A` by `p` lies inside a use of `B` by `q`: one
  of the uses its `within` chain leads to, which begins no later and ends no earlier.
- `every A@p to q [over c]` holds when each use of `A` by `p` names `q` as its counterparty, and
  `c` as its channel when the clause names one.
- An invariant whose predicate names a port the trace does not record is `not_checked`, never
  `held`: absent uses prove nothing.

A violated invariant names its link, the use and the index of the first event that breaks it,
and the command exits 1; otherwise it exits 0. A trace of another architecture is refused. The
same check reads a runtime's trace in the same format.
