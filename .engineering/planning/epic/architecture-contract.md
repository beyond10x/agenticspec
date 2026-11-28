---
format: aep.planning-md/3
id: epic:architecture-contract
kind: epic
status: active
title: 'The contract of an architecture: derived ports, narrowing settings, routing invariants, multiplicities'
summary: For every architecture, agenticspec sim derives its contract from ports derived from the blocks, checks it, and the docs show it on every architecture page and catalog entry; runtimes and bindings build against it in a later epic.
relations:
- depends_on: epic:building-blocks
- depends_on: epic:simulator-and-search
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:33:37Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-27T23:33:40Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

For every architecture document, `agenticspec sim` derives its contract and checks it: the ports
its parts use, the settings that narrow each of them, the protocol of each port, the routing
invariants its links give and the number of instances of each port. The ports themselves are
derived from the building blocks (`blocks/`, system `agentic`), never authored. The docs show the
contract on every architecture page and every catalog entry, with the contract file to download.

A later epic builds runtimes and bindings against contracts. This epic gives them their input:
what a runtime must provide to run a given architecture, written down and checked, before any
runtime exists.

Every command of the simulator named here is a subcommand of the one binary, `agenticspec sim
<cmd>`. A story that adds a subcommand or a flag declares it in `spec/cli.yaml` (its input and
result types in `spec/domains/cli.yaml`), regenerates the CLI contract with `task generate`
(`crates/agenticspec-cli-contract/`, held by `task check:generated`), dispatches it in
`crates/agenticspec/src/main.rs` and implements it in `crates/agenticspec-sim/src/cli.rs`.

Design: `design/architecture-runtime.md` (§ "The boundary: ports", § "The contract of one
architecture", § "What the runtime emits, and what conformance means").

## Ports are derived, not authored

A port is a leaf of the blocks with effects outside them: the part a runtime must implement.
Everything else is behaviour or computation that the specification runs (an ESS interpreter or
Spindle). The derivation reads the compiled system (`ess specify compile --path blocks`):

- **Internal** (behaviour): a command an ESS binding causes (an event-to-command reaction), or a
  decision over recorded facts.
- **Entry**: a command no binding causes. **Exit**: an event no binding consumes.
- **Outbound port**: an exit event that leaves its entity in a lifecycle state whose only way out
  is an entry command. The world must act and report: `StartInference` emits `InferenceStarted`
  and leaves the inference `Started`, which only `FinishInference` leaves.
- **Clock port**: an ESS periodic host cause or a clock-reading attachment of a binding.
- **Principal port**: entry commands a principal issues (`Answer`, `ApproveDecision`).
- **Configuration**, no implementation: entry commands whose entity has no such waiting state
  (`RegisterPrincipal`, `DefineAgent`, `DefineKpi`). The deployer issues them from the
  architecture document.

`blocks/` declares no binding at this commit (no `bindings:` section in any file), so the
derivation would call every command an entry. The first story declares the cross-block causation
as ESS bindings. Where ESS cannot express a binding the blocks need, the story writes the exact gap
into § "What ESS lacks" below instead of working around it.

The table in `design/architecture-runtime.md` § "The boundary: ports" (15 ports, from
`model.infer` to `pool.lease`) is the output the derivation is expected to give, not a source.
The derivation story compares its output with that table and lists every difference.

## Nouns (ESS)

Modelled before decomposition, in `blocks/9-composition/architecture.yaml` (domain
`agentic.architecture`, layer 9). Identifiers `PortId`, `ContractId` in
`blocks/0-foundation/shared.yaml`. `ess specify validate --path blocks --strict-requires`,
`task check:layers` and `task check:blocks` pass.

- `agentic.architecture.Port`: one derived port. `name`, `block` (the domain of its commands),
  `kind` (`PortKind`: `Outbound`, `Clock`, `Principal`), `intent_event` and `waiting_state`
  (absent for a clock port), `reports` (the entry commands that leave the waiting state) and
  `protocol` (a regular expression over the short names of its commands, intent first). Recorded
  by `DeclarePort` (refused as `PortNameTaken` for a second port of one name), read through the
  view `Ports`. Immutable.
- `agentic.architecture.Contract`: the contract of one `ArchitectureRevision`
  (`agenticspec-contract/1`). `ports` (`ContractPort`: port, part, the part's narrowing settings,
  instances), `invariants` (`RoutingInvariant`: link, port, the port it nests in, the predicate as
  written), `border` (the parts that take input from outside), `digest`. Relations: `revision`
  (references, one) and `border_parts` (references `Part`, many). Recorded by `RecordContract`
  (refused as `EmptyContract` with no port, `NoSuchRevision`, `ContractExists` for a second
  contract of one revision), read through the view `Contracts`. Immutable.
- The protocol of each port is in the contract file: `agenticspec-contract/1` copies it from
  `blocks/ports.json` beside each port, so a runtime reads one file. In ESS it is read through
  `ContractPort.port_id` on `Port.protocol`, so `ContractPort` needs no field of its own.
- Names of commands, events and lifecycle states are text (`CommandName`, `EventName`,
  `StateName`): ESS 0.99.2 has no type that references a command, an event, a state or a domain.
  The derivation writes only names `ess specify compile` lists.
- The authored scenario `blocks/scenarios/architecture/solo-contract-recorded.yaml` commits a
  revision and records its contract: synthesis cannot arrange a committed revision (the witness
  of `CommitRevision/committed` is a row set), so the three outcomes of `RecordContract` that need
  one are covered there.

Why there, and not elsewhere:

- In ESS, not in an authored marking file: the language can say what a port and a contract hold
  and how a contract relates to a revision and its parts; only the references to commands,
  events and states are text, and the derivation holds those.
- In the `architecture` domain, not a new domain of layer 9: a contract is derived from an
  architecture revision and has no behaviour of its own yet. A new domain is a new block, with its
  own docs page and at least two KPIs in `kpis/catalogue.yaml` (`scripts/check-kpis.sh`), which a
  derived document does not earn. Admission and bindings, the next epic, have behaviour (an
  admission is refused or admitted) and may move the interface into a domain of its own then.
- Not in `studio/`: a contract is a building block of agentic development, not a noun only the
  studio has.

Open, as `UNMAPPED:` markers in the file, each settled by the story named there:

- which part of an architecture uses which port: documents name a block per part (`agent`,
  `pool`, `schedule`), not a domain (`harness`, `resource`, `trigger`); some ports are reached
  through a link (`principal.answer`); documents name parts and carried values that `PartKind`
  and `Carries` have no variant for (`authority`, `work`, `call`, `context`, `grant`, ...);
- which settings narrow which port (the design names examples only);
- the routing invariant of every link that does not carry model calls, and the predicate
  language;
- whether an architecture owns its revisions' contracts or a contract only references its
  revision (taken: references).

The first three are the three markers between `agentic.architecture.PortKind` and
`agentic.architecture.RoutingInvariant`; `story:contract-derivation` removes them.

## Stories

| promise | story |
|---|---|
| cross-block causation declared as ESS bindings; the gaps of ESS named | `story:block-bindings` |
| ports derived from the blocks, compared with the design's table, every difference listed | `story:port-derivation` |
| the contract of an architecture derived and checked, with each port's protocol, written as `agenticspec-contract/1`; the predicate language parsed | `story:contract-derivation` |
| the contract on every architecture page and catalog entry, with the file to download | `story:contract-docs` |
| the blocks' conformance suite filtered to a contract's ports | `story:contract-conformance` |
| the routing invariants checked against a simulated trace | `story:routing-invariants` |

Order, by shared surfaces: `block-bindings`, `port-derivation`, `contract-derivation`,
`contract-docs`, `contract-conformance`, `routing-invariants`. Each story's § "Shared surfaces"
names the others; every pair that shares a surface has a `depends_on` edge, so the six run one
after another. Every story but `block-bindings` also depends on `story:architecture-kpis` (epic
`simulator-and-search`), which lands on `crates/agenticspec-sim/tests`, `crates/agenticspec-docs`,
`docs/architectures`, `docs/catalog` and `website/static/catalog` too.

## Cross-block causation

Filled by `story:block-bindings`: one line per candidate of that story's list (`C1` to `C10`) and
per causation it added (`A1`, ...), each ending as `binding <name>` (in `blocks/components.yaml`),
`gap` (its entry in § "What ESS lacks") or `dropped` with the reason the blocks do not imply it.

## Port derivation against the design's table

Filled by `story:port-derivation`: the output of `agenticspec sim ports --expect`, verbatim, one
line per difference from the 15 rows of `design/architecture-runtime.md` § "The boundary: ports",
each marked as fixed, left to an ESS gap (§ "What ESS lacks"), or a correction of the design's
table.

## What ESS lacks

Filled by `story:block-bindings` (and by `story:contract-conformance` if `ess` refuses a cut
suite): one entry per construct the blocks need and ESS 0.99.2 cannot express, with the binding or
use, the construct missing and a reproduction under `design/ess-repro/`. Empty until those stories
run; nothing is worked around.

## Acceptance

- Given the six stories merged, when `task check` runs, it exits 0 with `task check:ports`,
  `task check:docs` and `task check:generated` among its steps, and the test step ends within
  120 s.
- Given `blocks/ports.json` committed, when `agenticspec sim ports --check blocks/ports.json` runs,
  it exits 0, and § "Port derivation against the design's table" holds the verbatim output of
  `agenticspec sim ports --expect`, each line marked.
- Given each of the 19 documents `agenticspec sim check` accepts (the 10 under `architectures/`
  and the 9 ladder stages), when `agenticspec sim contract <document>` runs, it exits 0 and every
  port of the contract has a `protocol`.
- Given the generated docs, when the docs test of `story:contract-docs` runs, each of the 10
  architecture pages and 6 catalog entries has a "Contract" section and a contract file to
  download.
- Given `architectures/workstation.yaml`, when `task contract:conform --
  architectures/workstation.yaml` runs, it exits 0 with every kept scenario passed.
- Given the trace of seed 1 of `architectures/workstation.yaml` from `agenticspec sim trace`, when
  `agenticspec sim contract architectures/workstation.yaml --trace <trace>` reads it, it exits 0.
- Given `blocks/9-composition/architecture.yaml`, when the markers between
  `agentic.architecture.PortKind` and `agentic.architecture.RoutingInvariant` are counted
  (`grep -c UNMAPPED` on that range), the count is 0; it is 3 at this epic's commit.
- § "Cross-block causation" has a line for each of `C1` to `C10`, and every line marked `gap` has
  an entry in § "What ESS lacks" with a reproduction under `design/ess-repro/`.

## Not in this epic

- Building a runtime, the smallest first one included (`design/architecture-runtime.md` § "The
  smallest first runtime").
- Bindings of parts to real implementations (`agenticspec-binding/1`): Docker, substrate, a
  Claude service, an LLM gateway; admission (`admit(contract, binding, environment)`);
  environments; fixtures as capabilities; scenario documents (`agenticspec-scenario/1`) and
  scenario conformance.
- These are the next epic, `runtimes and bindings against contracts`, which starts from the
  contract file this epic writes.
- A contract on the ladder page (`docs/ladder.mdx`): the ladder stages under
  `architectures/ladder/` get a derived contract (the derivation runs on every document
  `agenticspec sim check` accepts) but no docs section.
- Merging `blocks/8-place/runtime.yaml` (a runtime as a noun) with the interface (a runtime as a
  role).
