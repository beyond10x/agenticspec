# Bringing an architecture into existence: the runtime interface

Status: concept, not yet planned. Written before an example architecture is chosen for the first
runtime.

## What exists

- An architecture document (`agenticspec-architecture/1`, `architectures/*.yaml`) names parts by
  **block** and settings: `{ block: agent, harness: coding }`, `{ block: model, route: frontier }`.
  It names no product, no model id, no prompt, no host. It is portable on purpose.
- `agenticspec sim check` refuses an architecture that is structurally wrong; `agenticspec sim run`
  simulates it with a behaviour profile.
- `blocks/` (system `agentic`) declares what each block means: its entities, commands, refusals,
  events and views. `blocks/8-place/runtime.yaml` declares a runtime **as a noun**: which workload
  kinds it runs, a workload, an instance, start and stop.
- Nothing declares **the interface between an architecture and what runs it**: which document a
  runtime accepts, what it must provide for a given architecture, how it refuses one it cannot run,
  and how anybody proves that a runtime runs an architecture as the blocks say.

## The pattern to copy

Spindle runs ESS specifications this way: the specification implies **capabilities**, an
**environment** says which implementation provides each capability, and **admission** binds every
required capability or refuses with the full list of what is missing. The same specification runs
in process or in a microVM; only admission differs.

An architecture is the same kind of input. Giving an architecture to a runtime is no different from
giving it a container or an ESS system: the runtime admits it or refuses it, then runs it and
reports what it observed.

## Three documents, kept apart

1. **Architecture** (exists): what the system is, in block terms. Portable.
2. **Binding** (new, `agenticspec-binding/1`): for one architecture and one runtime, which concrete
   thing plays each part. `worker: { block: agent, harness: coding }` becomes a harness product, its
   configuration, its instructions and its model id; `box: { block: sandbox }` becomes a substrate
   workspace with its egress list; `model: { route: frontier }` becomes an endpoint. A binding may
   leave a part to the runtime's default. The architecture never changes when the binding does.
3. **Scenario** (new, `agenticspec-scenario/1`): the stimuli of one run (a prompt on a channel, a
   trigger firing, a task arriving, the clock advancing), the **fixtures** that stand in for
   external parts (recorded or scripted model answers, tool results, a human's reply after a
   delay), and the **expectations** (events of the blocks, in order or as sets, and KPI bounds).

## The boundary: ports

The blocks are state machines that record what happened. Most of their commands are inner: a
registry entry, a decision on recorded facts. A few are the edge to the outside world: one command
states an intent the world must carry out, and later commands report what the world did. Today
that edge is visible only in the naming of the commands; no block marks it.

A **port** makes it explicit: a name, the intent command, the report commands, and the protocol
between them as a regular expression over the commands.

Ports are derived, not authored. A leaf with effects needs an implementation; everything else is
behaviour the specification already carries:

- **internal**: a command an ESS binding causes (an event-to-command reaction), or a decision over
  recorded facts;
- **entry**: a command no binding causes; **exit**: an event no binding consumes;
- **outbound port**: an exit event that leaves its entity in a lifecycle state whose only way out
  is an entry command, so the world must act and report (`StartInference` leaves the inference
  started; only `FinishInference` ends it);
- **clock port**: ESS periodic host causes and clock-reading attachments;
- **principal port**: entry commands a principal issues (`Answer`, `ApproveDecision`);
- **configuration**: entry commands whose entity has no such waiting state (`RegisterPrincipal`,
  `DefineAgent`, `DefineKpi`); the deployer issues them from the architecture document, and they
  need no implementation.

`blocks/` declares no binding at this commit, so the derivation would read every command as an
entry. The blocks first declare their cross-block causation as bindings (`FireTrigger` starts a
run, an admitted gateway call starts an inference); then the table below is the expected output of
the derivation, checked against it, and not a source. Read by hand from `blocks/` at this commit:

| port | block | protocol (intent, then reports) |
|---|---|---|
| `model.infer` | model | `StartInference FinishInference` |
| `model.serve` | model | `OpenEndpoint … CloseEndpoint` (only a self-hosted endpoint) |
| `gateway.call` | gateway | `AdmitCall CompleteCall` |
| `agent.session` | harness | `OpenSession (RecordTurn \| CompactContext)* EndSession` |
| `agent.spawn` | harness | `SpawnSubAgent` (then a session of the child) |
| `tool.invoke` | harness | `RequestToolCall DecideToolCall CompleteToolCall?` |
| `channel.deliver` | channel | `SendMessage (MarkDelivered MarkRead? \| MarkLost)` |
| `board.post` | channel | `PostMessage TakePost?` |
| `principal.answer` | question | `Ask (Answer \| TakeDefault \| Withdraw)` |
| `principal.decide` | authority | `DecideRequest (ApproveDecision \| RejectDecision \| DenyRequest)` |
| `clock.fire` | trigger | `FireTrigger (RecordStart (RecordEnd \| RecordFailure) \| SkipFiring)` |
| `workspace.provide` | workspace | `CreateWorkspace`, then `RecordWrite*`, `MergeProposal` |
| `sandbox.enforce` | sandbox | `DefineSandbox`, then `CheckEgress \| CheckSecretRead \| CheckWrite` |
| `workload.run` | runtime | `StartInstance (ReportRunning (ReportExited \| ReportFailed) \| ReportFailed)` |
| `pool.lease` | resource | `RequestClaim (GrantClaim (ReleaseClaim \| ExpireClaim \| KillClaim) \| WithdrawClaim)` |

Inner, no port: principal, delegation, agent definitions, tool and knowledge catalogues, work and
process bookkeeping, KPIs, experiments, architecture. These a runtime gets for free from the
specification (an ESS interpreter or Spindle runs them); only the ports need an implementation.

## The contract of one architecture

The contract is the subset of ports one architecture uses, derived without any runtime:

1. each part's block gives its ports; its settings narrow them (`harness: coding`, `route:
   frontier`, `isolation: process`, `egress: [gateway]`, `capacity: 16`);
2. each link gives a routing invariant over the trace: `worker -> gateway carries model_call` means
   every `model.infer` of `worker` happens inside a `gateway.call`, never around it;
3. `per`, `count` and `border` give multiplicity and exposure: `tree: { per: worker }` with
   `worker.count: 12` means 12 `workspace.provide`; only `border` parts take input from outside.

Example, `architectures/workstation.yaml`: `agent.session`, `model.infer`, `gateway.call`,
`channel.deliver`, `principal.answer`, `pool.lease`, `workspace.provide`, `sandbox.enforce`. It has
no runtime part, no trigger, no board, so `workload.run`, `clock.fire` and `board.post` are not in
its contract, and a runtime that cannot start workloads still runs it.

The contract is a document (`agenticspec-contract/1`): the ports with their narrowing settings and
protocols, the routing invariants, the multiplicities. Each catalog entry and architecture page of
the docs shows it.

## Admission

`admit(contract, binding?, environment) -> Admitted(plan) | Refused(missing[])`. The plan names,
per port, the implementation and how many instances. A refusal lists every port the
environment cannot serve, never only the first. Several architectures, or several instances of one, are
several admissions against one environment; the environment's capacity is a capability too.

## What the runtime emits, and what conformance means

A conformant runtime emits the events of the blocks it runs (`agentic.agent`, `agentic.work`,
`agentic.channel`, ...), answers their views, and honours their refusals. Two layers of proof:

1. **Block conformance**: the synthesized suite of `blocks/` (`task check:blocks` already runs it on
   the ESS interpreter) runs against the runtime's adapter instead, for every block the runtime
   claims. A runtime may claim a subset; admission then refuses architectures that need more.
2. **Scenario conformance**: given an architecture, a binding and a scenario, the runtime's
   observed event log meets the scenario's expectations, with the fixtures in place of the real
   model, tools and humans. The same scenario run in `agenticspec-sim` gives the expected KPIs, so
   the simulator and a real runtime can be compared on the same input.

## Fixtures are capabilities, not a test mode

A fixture is one more implementation of a capability: `model.infer` by replay of recorded answers,
`principal.answer` by a script with delays, `clock.fire` by a virtual clock. Admission binds them
like any other implementation. A test run and a production run differ only in the environment, so
the runtime has no test-only code path.

## The smallest first runtime

One process, every capability in memory:

- `agent.run`: a loop that sends the agent's turn to `model.infer` and acts on the answer;
- `model.infer`: fixture replay only, in the first version;
- `channel.deliver`, `pool.lease`, `clock.fire`: in-process queue, semaphore, virtual clock;
- `workspace.provide`: a temporary directory; `sandbox.confine`: refused unless the binding names
  substrate, so the runtime never claims confinement it does not have;
- output: the blocks' events as an event log, checked against the scenario.

The example architecture decides which of these the first runtime needs; the rest are refused at
admission.

## Open points

- Where the interface lives: a new block in `blocks/` (layer 9, beside `architecture`), or its own
  ESS system as `studio/` is.
- The binding format: one document per runtime, or one per architecture with a section per runtime.
- Whether `blocks/8-place/runtime.yaml` (a runtime as a noun) and this interface (a runtime as a
  role) merge.
- How substrate, Loom and Spindle each appear: as implementations of capabilities in an
  environment, which this document assumes.
