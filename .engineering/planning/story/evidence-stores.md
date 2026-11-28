---
format: aep.planning-md/3
id: story:evidence-stores
kind: story
status: draft
title: The ladder's evidence parts become record stores
summary: architectures/ladder/08-live-loop.yaml and 09-factory.yaml move evidence from block work to block store, kind record; ladder and catalog regenerated.
relations:
- decomposes: epic:events-and-storage
- depends_on: story:architecture-store-and-bus
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: architectures/ladder/08-live-loop.yaml
- confidence: cited
  path: architectures/ladder/09-factory.yaml
- confidence: inferred
  path: docs/catalog
- confidence: inferred
  path: docs/ladder.mdx
- confidence: inferred
  path: ladder
- confidence: inferred
  path: website/static/catalog
revision: 3
---
## Outcome

The ladder architectures stop borrowing `work` for evidence. In `architectures/ladder/08-live-loop.yaml`
and `09-factory.yaml`, `evidence: { block: work, kind: evidence }` becomes
`{ block: store, kind: record }`, written to over `carries: record`; the ladder page, catalog entries
and their digests are regenerated. Design: `design/events-and-storage.md` E7. Epic:
`epic:events-and-storage`.

## Scope

- `architectures/ladder/08-live-loop.yaml`, `architectures/ladder/09-factory.yaml`.
- Generated: `ladder/`, `docs/ladder.mdx`, `docs/catalog/`, `website/static/catalog/`
  (`task ladder:generate`, `task catalog:generate`).
- `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given the two ladder documents at this story's commit, when
  `grep -c 'block: work, kind: evidence' architectures/ladder/*.yaml` runs, it prints 0 for every
  file, and both name an `evidence` part of block `store`, kind `record`.
- Given both documents, when `agenticspec sim check` runs, each exits 0.
- Given `task check:ladder` and `task check:generated`, when they run, both pass with the
  regenerated files committed.

## Not in this story

Simulating store costs (`story:sim-store-and-bus`); other architectures, none of which names
evidence.
