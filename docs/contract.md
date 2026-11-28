# The contract of an architecture

A runtime that brings an architecture into existence implements a few commands of the building
blocks and gets the rest from the specification. The commands it implements are grouped into
**ports**. The **contract** of an architecture document lists the ports its parts use, with the
settings that narrow each port, how many instances run, and what each link promises about a run.
Both are derived from files in this repository; nothing here is written by hand.

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
and checks it against the document and the ports.

Each [architecture page](architectures/index.md) and [catalog entry](catalog/index.md) shows its
contract in a section "Contract", with the contract file to download
(`website/static/architectures/<name>.contract.yaml`, `<entry>.contract.yaml` in the entry's
downloads).

### Which part uses which port

A part uses the ports of its block:

| block | ports |
|---|---|
| `agent` | `harness.session` |
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

A block not in the table is refused, so a new block gets its row before a document uses it.

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
  `sharing`; sandbox `isolation`, `egress`; authority and runtime `kind`. A port used through a
  link takes no setting of its part (an agent's `model_share` among them), except a principal's
  `kind`.
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
part      = a letter or "_", then letters, digits, "_" and "-"
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
```

Ports are ordered by part, then port; invariants follow the document's links. The same document
and ports file give the same bytes. The contract is the content of `agentic.architecture.Contract`
in the building blocks.

### The check

A contract passes when its `ports_digest` is the ports file's digest, every port it names is in
the ports file with the same protocol, every part, link and border part is in the document, and
every invariant names a port its link's `from` part uses and has a predicate that parses and
starts with that use. `sim contract` checks every contract it derives before printing it.

### Nested architectures

A contract is one document's. When a document names another architecture as a part, each
document gets its own contract, and the outer one composes them: the inner contract's ports are
used by the outer part, their instances multiplied by that part's instances; the inner border
parts are where the outer links into the part arrive, so an outer link's invariant names a port of
an inner border part; and the inner invariants hold unchanged, their part names prefixed with the
outer part.
