---
format: aep.planning-md/3
id: story:store-and-bus-ports
kind: story
status: draft
title: Store and bus ports in the port and contract derivations
summary: store.put, store.get, store.query, store.expire, bus.publish, bus.deliver join the expected port table; the contract copies kind, scope, retention, distribution, may_lose and an agent's store.put lies inside tool.invoke.
relations:
- decomposes: epic:events-and-storage
- depends_on: story:architecture-store-and-bus
- depends_on: story:port-derivation
- depends_on: story:contract-derivation
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: blocks/3-capability/store.yaml
- confidence: inferred
  path: blocks/7-collaboration/channel.yaml
- confidence: inferred
  path: blocks/ports.json
- confidence: inferred
  path: crates/agenticspec-sim/src/contract
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: docs/contract.md
revision: 2
---
## Outcome

Storing and publishing are ports. The port derivation finds `store.put` (`PutRecord (ConfirmPut |
FailPut)`), `store.get` (`GetRecord (ReturnRecord | FailGet)`), `store.query` (`QueryStore
(AnswerQuery | FailQuery)`), the clock port `store.expire` (`ExpireRecord`), `bus.publish`
(`PublishEvent (AckPublish | FailPublish)`, only for an explicit bus bound outside the runtime) and
`bus.deliver` (`DeliverEvent (AckDelivery | FailDelivery)`, per subscription); the six rows join
the expected table of `story:port-derivation`, and the contract derivation copies the narrowing
settings `kind`, `scope`, `retention`, `distribution`, `may_lose`. An agent's `store.put` lies
inside one of its `tool.invoke` as a routing invariant. Design: `design/events-and-storage.md` E6,
E8, P7 A. Epic: `epic:events-and-storage`.

## Scope

- `blocks/3-capability/store.yaml`, `blocks/7-collaboration/channel.yaml`: the intent and report
  commands the derivation reads, where `story:store-block` and `story:event-channel` left them
  short.
- `blocks/ports.json` (regenerated), the expected table in the port derivation.
- `crates/agenticspec-sim/src/contract/`: the settings table and the store invariant.
- `crates/agenticspec-sim/tests/`, `docs/contract.md`, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `blocks/` at this story's commit, when `agenticspec sim ports` runs, `blocks/ports.json`
  holds the six ports with the protocols above and the derivation reports no difference from the
  expected table for them.
- Given a document with an agent part linked to a `record` store over `carries: record` and a
  `retention: run` setting, when `agenticspec sim contract` runs, the contract holds `store.put`
  used by the store part with settings `kind: record` and `retention: run`.
- Given the same document, when the contract is derived, it holds an invariant that every
  `store.put` of the agent part lies inside a `tool.invoke`.
- Given a document with no store or event channel, when `agenticspec sim contract` runs before and
  after this story, the contract is the same bytes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Bindings and fixtures for the ports (`story:binding-store-and-bus`); delivery conformance
(`story:bus-delivery-conformance`).
