---
format: aep.planning-md/3
id: story:sim-store-and-bus
kind: story
status: draft
title: Stores, delivery and Command reactions in the simulator
summary: Store port delays and costs from the profile, delivery delay and loss, store events for on_event, Command reactions counted per store; every result byte for byte at the defaults.
relations:
- decomposes: epic:events-and-storage
- depends_on: story:architecture-store-and-bus
- depends_on: story:trigger-reactions
- depends_on: story:simulator-fidelity
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-sim/src/profile.rs
- confidence: cited
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/src/sim/swarm.rs
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: profiles/coding.yaml
revision: 3
---
## Outcome

The simulator runs stores, buses and `Command` reactions. A put, get or query is a delay and a
cost drawn from the profile; a delivery is a delay (default 0) and a loss rate under a fault; the
store events join the simulator's list of events, so `on_event` can fire on them; a `Command`
reaction is a delay and a store write counted per store. With every default at 0, every existing
`result.json` stays byte for byte. Design: `design/events-and-storage.md` E8. Epic:
`epic:events-and-storage`.

## Scope

- `crates/agenticspec-sim/src/sim.rs`, `src/sim/swarm.rs`: store parts, delivery, store events,
  `Command` reactions.
- `crates/agenticspec-sim/src/profile.rs`, `profiles/coding.yaml`: store and delivery delays and
  costs, defaults 0.
- `crates/agenticspec-sim/tests/`: a store write, a reaction chain, a lossy channel, each under
  5 s.
- `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given every committed experiment result, when the simulator runs at this story's commit with the
  profile's new keys absent, each `result.json` is the same bytes as before.
- Given a document where a `Command` trigger on `DeliverableAccepted` puts a record into a store,
  when `agenticspec sim run` runs it, the result counts one store write per accepted deliverable.
- Given an `on_event` trigger on `RecordPut`, when the simulator runs a put, the trigger fires at
  the put's instant plus the delivery delay.
- Given an event channel with `may_lose` and a loss fault at rate 1, when the simulator runs, no
  subscription receives an event and the result counts the losses.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The ladder's evidence parts (`story:evidence-stores`); a runtime's real stores
(`story:binding-store-and-bus`).
