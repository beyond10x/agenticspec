---
format: aep.planning-md/3
id: story:architecture-store-and-bus
kind: story
status: draft
title: Store parts and event links in the architecture format, with their check rules
summary: PartKind Store, Carries Event, settings kind, scope, retention; sim check refuses unlinked reach to a store or bus, a semantic store without a model path, an external store outside egress.
relations:
- decomposes: epic:events-and-storage
- depends_on: story:store-block
- depends_on: story:event-channel
- depends_on: story:architecture-references
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: inferred
  path: crates/agenticspec-sim/src/check.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/doc.rs
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: docs/architectures/index.md
- confidence: inferred
  path: docs/blocks/9-composition/architecture.md
- confidence: inferred
  path: website/src/generated
revision: 3
---
## Outcome

An architecture document can declare stores and buses. `PartKind` gets `Store`; `Carries` gets
`Event` beside `Record` and `Read` (from `story:architecture-references`); a store part takes the
settings `kind`, `scope` (`per: <part>`, architecture by default, or `scope: external`) and
`retention` (duration, `run`, `forever`); a `channel` part of kind `event` is a named bus.
`agenticspec sim check` gets rules: a part reaches a store only over a `record` or `read` link, a
bus only over an `event` link; a semantic store that makes its own vectors needs a model path (like
`agent_without_model_path`); a part reaching an external store names it in its sandbox's `egress`.
Design: `design/events-and-storage.md` E1, E5, E6, E10, P8 A, P9 A. Epic:
`epic:events-and-storage`.

## Scope

- `blocks/9-composition/architecture.yaml`: `PartKind` `Store`, `Carries` `Event`.
- `crates/agenticspec-sim/src/doc.rs`, `src/check.rs`: the part kind, settings and the four check
  rules.
- `crates/agenticspec-sim/tests/`: one accepted and one refused case per rule, each under 5 s.
- `docs/architectures/index.md`: the store part, its settings and the rules.
- Generated: `docs/blocks/9-composition/architecture.md`, `website/src/generated/`.
- `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given a document with `evidence: { block: store, kind: record }` and a `worker -> evidence
  carries record` link, when `agenticspec sim check` runs, it exits 0.
- Given a document where a part reads a store with no `read` link to it, when
  `agenticspec sim check` runs, it exits 1 naming the part and the store.
- Given a document with a `semantic` store that makes vectors and no link to a gateway or model,
  when `agenticspec sim check` runs, it exits 1 naming the store.
- Given a document with an `external` store reached by an agent whose sandbox `egress` does not
  name it, when `agenticspec sim check` runs, it exits 1 naming the sandbox and the store.
- Given each of the documents under `architectures/`, when `agenticspec sim check` runs before and
  after this story, its output is the same bytes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Simulating store parts (`story:sim-store-and-bus`); contract ports (`story:store-and-bus-ports`);
the ladder documents (`story:evidence-stores`).
