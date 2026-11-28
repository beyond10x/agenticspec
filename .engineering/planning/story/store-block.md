---
format: aep.planning-md/3
id: story:store-block
kind: story
status: draft
title: 'The store block: stores and records of five kinds'
summary: 'agentic.store in layer 3: Store and Record, kinds record, artifact, key_value, semantic, graph, put/get/query/expire/delete, record events, artifact invariants, delete needing approval.'
relations:
- decomposes: epic:events-and-storage
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/3-capability/store.yaml
- confidence: inferred
  path: blocks/ess-inputs.yaml
- confidence: inferred
  path: blocks/scenarios/store
- confidence: inferred
  path: blocks/system.yaml
- confidence: inferred
  path: docs/blocks/3-capability/store.md
- confidence: inferred
  path: website/src/generated
revision: 3
---
## Outcome

A new domain `agentic.store` (layer 3, beside `tool` and `knowledge`) declares a store and its
records: `Store` (kind `record`, `artifact`, `key_value`, `semantic`, `graph`; scope; retention;
record type) and `Record` (store, key, revision, content as `agentic.shared.Content`,
`recorded_by`, `recorded_at`, `derived_from`, `expires_at`; lifecycle `Present`, then `Superseded`,
`Expired` or `Deleted`). The blocks record that a write happened, its key, digest and writer, never
the bytes. An artifact is a record of an artifact store with three invariants: content required and
never revised, addressed by digest, provenance required. Deleting an unexpired record defaults to
`ApprovalRequired`. Design: `design/events-and-storage.md` E4, E5, E10, P4 A, P5 A. Epic:
`epic:events-and-storage`.

## Scope

- `blocks/3-capability/store.yaml` (new): entities, the commands put, get, query, expire, delete
  and their refusals, the events `RecordPut`, `RecordSuperseded`, `RecordExpired`, `RecordDeleted`,
  views over stores and records, an actor per role.
- `blocks/0-foundation/shared.yaml`: `StoreId`, `RecordId`.
- `blocks/system.yaml`, `blocks/ess-inputs.yaml`: the new domain.
- `blocks/scenarios/store/` (new): authored scenarios for each kind's write and read, the artifact
  invariants and the delete approval.
- Generated: `docs/blocks/3-capability/store.md`, `website/src/generated/` (`task docs:generate`).
- `CHANGELOG.md`.
- Time synthesis before and after (`scripts/check-blocks-conform.sh --slow`); a jump is fixed in
  this story. Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool
  job over 10 minutes is killed.

## Acceptance

- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, it lists the domain `agentic.store` with `Store`, `Record`, the five kinds and the four
  record events, where before none existed; `task check:layers` passes with `store` depending only
  on layers 0 and 1.
- Given an authored scenario that puts a second artifact under an existing artifact's digest with
  other content, when `task check:blocks` runs, the put is refused by name and the scenario passes.
- Given an authored scenario that puts a record twice under one key of a `record` store, when
  `task check:blocks` runs, the first record is `Superseded` and the second `Present` with revision
  2.
- Given an authored scenario that deletes an unexpired record without an approval, when
  `task check:blocks` runs, the delete is refused and `RecordDeleted` is not emitted.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Names of stores from other blocks (`story:artifact-references`); the part kind and links
(`story:architecture-store-and-bus`); ports (`story:store-and-bus-ports`).
