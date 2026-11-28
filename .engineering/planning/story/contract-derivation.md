---
format: aep.planning-md/3
id: story:contract-derivation
kind: story
status: implemented
title: 'Derive and check the contract of an architecture: agenticspec sim contract'
summary: 'agenticspec sim contract writes agenticspec-contract/1 for every accepted architecture document: ports per part with protocol, narrowing settings and instances, routing invariants whose predicates parse, border.'
relations:
- decomposes: epic:architecture-contract
- depends_on: story:port-derivation
- depends_on: story:block-bindings
- depends_on: story:architecture-kpis
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: cited
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-sim/src/cli.rs
- confidence: cited
  path: crates/agenticspec-sim/src/contract
- confidence: cited
  path: crates/agenticspec-sim/src/lib.rs
- confidence: cited
  path: crates/agenticspec-sim/tests
- confidence: cited
  path: crates/agenticspec/src/main.rs
- confidence: inferred
  path: docs/blocks/9-composition/architecture.md
- confidence: cited
  path: docs/contract.md
- confidence: cited
  path: spec/cli.yaml
- confidence: cited
  path: spec/domains/cli.yaml
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:33:37Z", actor: "human:operator", revision: 10, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-11-27T23:33:37Z", actor: "human:operator", revision: 11, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:11Z", actor: "human:operator", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Outcome

`agenticspec sim contract <architecture.yaml>` derives the contract of an architecture document
from the document and `blocks/ports.json`, checks it, and writes it as `agenticspec-contract/1`:
the ports each part uses with each port's protocol, the settings that narrow them and their
number of instances, the routing invariant of each link, and the border. It derives a contract for
every document `agenticspec sim check` accepts (the 10 under `architectures/` and the 9 ladder
stages) and refuses one the check refuses. The contract is the content of
`agentic.architecture.Contract` (`blocks/9-composition/architecture.yaml`); the protocol is copied
from the port (`epic:architecture-contract` § "Nouns (ESS)"). This story owns the predicate
language of routing invariants: its grammar, its parser and the check that every predicate a
contract writes parses. Design: `design/architecture-runtime.md` § "The contract of one
architecture".

## Scope

- `crates/agenticspec-sim/src/contract/derive.rs`: from the document as `check.rs` and `sim.rs`
  read it (`parts`, `links`, `border`, `per`, `count` and its marked `value`):
  - which part uses which port. Settles the first `UNMAPPED:` of `ContractPort` in
    `blocks/9-composition/architecture.yaml`: a fixed table from a document block (`agent`,
    `pool`, `schedule`, `principal`, `authority`, `work`, `tool`, `knowledge`, `experiment`,
    `kpi`, ...) to the domain whose ports it uses, and the link rules (`principal.answer` for a
    principal part a link carrying `question` reaches; whether a question to an agent acting for
    a principal uses it). Blocks of an inner domain (work, knowledge, experiment, kpi) use no port.
    The block file's comment is changed to what is taken, and the marker removed;
  - which settings narrow which port. Settles the second `UNMAPPED:`: per port, the setting names
    of the using part copied into the contract (`harness`, `route`, `isolation`, `egress`,
    `capacity`, `sharing`, ...); a setting not in the list is not copied;
  - each port's protocol, copied from the port's `protocol` in `blocks/ports.json`;
  - instances: the part's `count` times the count of the part it is `per`, along the chain;
  - border: the parts of `border`;
  - routing invariants. Settles the `UNMAPPED:` of `RoutingInvariant`: for a link carrying
    `model_call` to a gateway, every inference port use of `from` lies inside a gateway port use of
    `to`; for every other carried value the invariant the story writes down (counterparty
    restricted to `to`, delivered over `over` when given), or none, said per value.
- `crates/agenticspec-sim/src/contract/predicate.rs`: the predicate language. The story defines
  its grammar on `docs/contract.md` and implements its parser here: text in, a typed form
  (`Predicate`) out, or a refusal naming the offset and what was expected. The derivation writes
  each predicate by printing a `Predicate`, so printing then parsing gives the same form.
  `story:routing-invariants` evaluates the parsed form and does not parse text itself.
- The check of a contract: every port it names is in `blocks/ports.json`, every part and link it
  names is in the document, every invariant names a port used by its link's `from` part, and
  every invariant's predicate parses.
- `agenticspec-contract/1` as YAML: `architecture`, `ports_digest` (the `sources_sha256` of
  `blocks/ports.json`), `ports` (port, protocol, part, settings, instances), `invariants`,
  `border`. The same input gives the same bytes.
- `agenticspec sim contract <architecture.yaml> [--ports blocks/ports.json] [--out <file>]`:
  prints the contract or writes it; exit 1 with the check's refusals, writing nothing, when the
  document is refused. The subcommand is declared in `spec/cli.yaml`, its input and result types
  in `spec/domains/cli.yaml`; `task generate` regenerates `crates/agenticspec-cli-contract/`
  (held by `task check:generated`); `crates/agenticspec/src/main.rs` dispatches it to
  `crates/agenticspec-sim/src/cli.rs`; `src/lib.rs` lists the modules.
- `crates/agenticspec-sim/tests/main.rs`, `tests/cases/`: the 19 accepted documents and one refused
  one; the parser on every predicate form the grammar names and on malformed text (refused with
  its offset); a contract whose invariant carries a malformed predicate refused by the check;
  each test under 5 s.
- `blocks/9-composition/architecture.yaml` (comments and markers only, unless a field must change;
  then `task docs:generate` regenerates `docs/blocks/9-composition/architecture.md`).
- `docs/contract.md` (the contract format, the command, the predicate language), `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Shared surfaces

- `blocks/ports.json`, `crates/agenticspec-sim/src/contract/`, `src/cli.rs`, `src/lib.rs`,
  `tests/`, `docs/contract.md`: `story:port-derivation` (comes first),
  `story:contract-conformance`, `story:routing-invariants`.
- `spec/cli.yaml`, `spec/domains/cli.yaml`, `crates/agenticspec-cli-contract/`,
  `crates/agenticspec/src/main.rs`: `story:port-derivation` (comes first),
  `story:contract-conformance`, `story:routing-invariants`. Outside this epic,
  `story:architecture-patches` records all of `crates/agenticspec-sim` and
  `crates/agenticspec/src` as its scope; `aep plan artifact waves` keeps them in different waves,
  and whichever lands second rebases on the other.
- `crates/agenticspec-sim/src/contract/predicate.rs`: `story:routing-invariants` reads its typed
  form and adds the evaluator beside it, not in it.
- `blocks/9-composition/architecture.yaml`, `docs/blocks/`: `story:block-bindings`; outside this
  epic `story:architecture-patches` and `story:guards-and-limits` change the same block file.
- The contract format and its derivation: `story:contract-docs` renders it.
- `crates/agenticspec-sim/tests`: outside this epic, `story:architecture-kpis`.
- `CHANGELOG.md`: every story of this epic.

## Acceptance

Each line below runs `agenticspec sim contract <document>` and reads the contract it prints.

- Given `architectures/workstation.yaml`, the command exits 0 and the contract holds exactly the
  ports whose commands are those of the design's `agent.session`, `model.infer`, `gateway.call`,
  `channel.deliver`, `principal.answer`, `pool.lease`, `workspace.provide` and `sandbox.enforce`.
- Given `architectures/workstation.yaml`, the contract holds no port whose commands are
  `StartInstance`, `FireTrigger` or `PostMessage` (no `workload.run`, `clock.fire`, `board.post`).
- Given `architectures/workstation.yaml`, the workspace port is used by part `tree` with 12
  instances.
- Given `architectures/workstation.yaml`, the sandbox port is used by part `box` with settings
  `isolation: process` and `egress: [gateway]`.
- Given `architectures/workstation.yaml`, the border is `[operator]`.
- Given `architectures/workstation.yaml`, the contract has an invariant for the link
  `worker -> gateway carries model_call`.
- Given `architectures/workstation.yaml`, every port of the contract has a `protocol` equal to
  that port's `protocol` in `blocks/ports.json`.
- Given `architectures/scheduled-loops.yaml`, the contract holds the clock port; given
  `architectures/shared-board.yaml`, the contract holds the board port.
- Given each of the 19 documents `agenticspec sim check` accepts, when the contract is derived
  twice, both runs exit 0 with the same bytes, and every invariant's predicate in it parses
  (the check passed).
- Given a document the check refuses (an unknown link part), the command exits 1 with the
  check's refusal and writes no file.
- Given a contract fixture whose invariant carries a malformed predicate, when the contract check
  runs on it, it exits 1 naming the link and the offset where the predicate stops parsing.
- Given `blocks/9-composition/architecture.yaml`, when
  `sed -n '/name: agentic.architecture.PortKind$/,/name: agentic.architecture.RoutingInvariant$/p' blocks/9-composition/architecture.yaml | grep -c UNMAPPED`
  runs, it prints 3 before this story and 0 after it.
- A port `story:port-derivation` recorded as left to an ESS gap (`epic:architecture-contract`
  § "Port derivation against the design's table") is exempt from the port lines above, and the
  exemption is named in this story's evidence.

## Not in this story

The docs sections (`story:contract-docs`); evaluating an invariant on a trace
(`story:routing-invariants`); filtering conformance (`story:contract-conformance`); recording a
contract through `RecordContract` in a running system; admission and bindings (next epic).
