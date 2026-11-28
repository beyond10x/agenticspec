---
format: aep.planning-md/3
id: story:simulator-fidelity
kind: story
status: active
title: 'Simulator fidelity: process parts, on_event triggers, roles not part names, routes and prices'
summary: Non-agent process parts run, on_event triggers fire, the profile role comes from role and the answerer from its role, model calls split over routes and an unpriced route is refused (consumer gaps M11-M14)
relations:
- decomposes: epic:simulator-and-search
- depends_on: story:part-kinds-and-settings
scope:
- confidence: cited
  path: architectures/orchestrator-workers.yaml
- confidence: cited
  path: architectures/planner-executor.yaml
- confidence: cited
  path: crates/agenticspec-sim/src/check.rs
- confidence: cited
  path: crates/agenticspec-sim/src/profile.rs
- confidence: cited
  path: crates/agenticspec-sim/src/sim
- confidence: cited
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/tests/cases
- confidence: inferred
  path: docs/architectures
- confidence: inferred
  path: docs/catalog
- confidence: cited
  path: profiles/coding.yaml
- confidence: cited
  path: website/static/catalog
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 13}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 14}
---
## Outcome

The simulator runs what an architecture document says rather than what its part names suggest. A
consumer's gap report (M11 to M14) found four places at `25fbfcb` where a run diverges from the
document:

- M11: only agent parts run. Both loops that build the running kinds skip every part whose block
  is not `agent` (`crates/agenticspec-sim/src/sim.rs:401-404`, `sim.rs:419-422`).
- M12: an `on_event` trigger is checked and never fires. The check accepts it when it names an
  `event_name` (`crates/agenticspec-sim/src/check.rs:240-242`); the simulator maps it to
  `Source::Event`, which "never fires here" (`crates/agenticspec-sim/src/sim/swarm.rs:62-67`,
  `swarm.rs:204`). A trigger link must end at an agent (`check.rs:343-351`), so a trigger cannot
  start a process.
- M13, a defect: the part's `harness` setting is read as the profile role
  (`sim.rs:377-382`), so documents write `harness: planner` and `harness: orchestrator` for a role
  (`architectures/planner-executor.yaml:17,20`, `architectures/orchestrator-workers.yaml:18`). An
  agent that answers questions is found in the profile by its part name
  (`sim.rs:445`, `sim.rs:484-488`; `crates/agenticspec-sim/src/profile.rs:177-178`), and the
  profile names the part `operator` (`profiles/coding.yaml:53-55`): renaming the part breaks the
  run.
- M14: an agent uses the first model it reaches (`sim.rs:383-384`), so its calls cannot split
  over routes; a route with no price in the profile costs 0 without a word (`sim.rs:391-394`),
  and the profile prices only `frontier` (`profiles/coding.yaml:73-75`).

Changing `sim.rs` changes the committed catalog results: `task catalog:generate` rewrites
`website/static/catalog/<entry>/result.json` and its `sources.sha256`, then `task docs:generate`.

## Scope

- `crates/agenticspec-sim/src/sim.rs`, `src/sim/swarm.rs`: process parts (kind `Process` from
  `story:part-kinds-and-settings`) run their program steps, with step durations from the profile;
  `on_event` triggers fire on the events the simulator produces (a run of a part ended, a claim
  granted, a firing ended), and a trigger whose event the simulator does not produce is reported
  `not_simulated` in the result; the profile role comes from the part's `role`, the harness from
  `harness`; the answer delay of an answering agent is read by its role, not its part name; an
  agent's model calls split over the routes it names, with a share per route (taken: an ordered
  list on the part, as `AgentRevision.models` orders its bindings, `blocks/4-agent/agent.yaml:111`;
  one route by default, as today); a route with no price refuses the run.
- `crates/agenticspec-sim/src/check.rs`: a trigger link may end at a process part.
- `crates/agenticspec-sim/src/profile.rs`, `profiles/coding.yaml`: roles keyed by role, answer
  delays keyed by role, step durations of process parts; a route without a price named in the
  refusal.
- `architectures/planner-executor.yaml`, `architectures/orchestrator-workers.yaml`: `harness:
  coding` with `role: planner`, `executor`, `orchestrator`; generated pages and catalog results.
- `crates/agenticspec-sim/tests/cases/sim.rs`, `swarm.rs`, `check.rs`, `determinism.rs`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s; a pool job over
  10 minutes is killed, and `task catalog:generate` names its time with `slot --timeout` (at most
  30 minutes) if it needs more.

## Shared surfaces

- `crates/agenticspec-sim/src/sim.rs`, `src/sim/`: `story:guards-and-limits`,
  `story:routing-invariants` (records a port-level trace in the same loop),
  `story:scenario-simulator-run`, `story:task-corpus`; whichever lands second rebases.
- `crates/agenticspec-sim/src/check.rs`: `story:part-kinds-and-settings` (comes first; this story
  reads its `Process` kind), `story:architecture-references`, `story:architecture-kpis`.
- `crates/agenticspec-sim/src/profile.rs`, `profiles/coding.yaml`: `story:task-corpus`.
- `website/static/catalog/`: every story that changes a simulated KPI regenerates it.

## Acceptance

- Given a document where a trigger wakes a process part of two program steps, `agenticspec-sim
  run` reports that part's runs in its per-part KPIs, where before the part was skipped.
- Given a trigger part `kind: on_event` whose `event_name` is the end of a run of another part,
  the run reports firings of that trigger above 0, where before it reported none; given an
  `event_name` the simulator does not produce, the result names the trigger `not_simulated`.
- Given a trigger link that ends at a process part, `check` raises no `trigger_target_not_agent`.
- Given `architectures/workstation.yaml` with the part `operator` renamed `assistant` (its
  links and `border` renamed with it), `run` exits 0 with the same whole-run KPIs as before,
  where before it exited 1 with `profile has no agents.assistant.answer_seconds`.
- Given `architectures/planner-executor.yaml` after this story (`harness: coding`, `role:
  planner`), the planner's model calls are those of the profile role `planner`.
- Given an agent part that names routes `frontier` and `local` with shares 0.75 and 0.25, the
  run's model calls of that part split in that ratio within the tolerance the test names; given a
  model part with `route: local` and no `prices.local` in the profile, `run` exits 1 naming the
  route, where before its calls cost 0.
- Given the same experiment and seeds, `result.json` is the same bytes at any thread count, and
  `task check:docs` passes on the regenerated catalog.

## Not in this story

Part kinds other than process running in the simulator; person steps of a process
(`story:block-remainders` settles whether they are required); fitting role or route parameters
from sessions (`epic:evaluation-loop`); a scenario's inputs (`story:scenario-simulator-run`).
