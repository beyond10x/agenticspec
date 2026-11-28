---
format: aep.planning-md/3
id: story:bundle-fixtures
kind: story
status: active
title: Fixture bundles, their digests and the copy of cloudspec's accepted list
summary: bundles/<architecture>.<binding>.yaml per catalog architecture with bundles/sources.sha256, a fast digest check and a slow byte comparison, and bundles/cloudspec-accepts.yaml enforced by bundle and admission.
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:deployment-bundle
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Taskfile.yml
- confidence: cited
  path: bundles
- confidence: inferred
  path: crates/agenticspec-runtime
- confidence: inferred
  path: scripts/check-generated.sh
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:21Z", actor: "human:operator", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:21Z", actor: "human:operator", revision: 5}
---
## Outcome

agenticspec commits fixture bundles, `bundles/<architecture>.<binding>.yaml`, one per catalog
architecture with a test binding, and `bundles/sources.sha256` (architecture, nested documents,
binding, agent documents, generator version). `task check` compares the digests; the `slow` lane
writes the bundles afresh and compares bytes. A copy of cloudspec's accepted unit kinds and roles,
`bundles/cloudspec-accepts.yaml`, with the cloudspec commit it came from, makes `deploy bundle`,
admission (`unit_not_hosted`) and a test fail when a bundle would carry a kind or role not in it.
cloudspec vendors the fixtures with a lock and a coverage test (request C7). Design:
`design/deployment.md` K16. Epic: `epic:architecture-deployment`.

## Scope

- `bundles/` (new): the fixture bundles, test bindings, `sources.sha256`, `cloudspec-accepts.yaml`.
- `scripts/check-generated.sh`, `Taskfile.yml`: the digest check and the `slow` lane.
- The accepted-list check in the bundle and admission modules.
- Tests, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `bundles/` at this story's commit, it holds one bundle per catalog architecture and
  `task check` passes, comparing digests without writing a bundle.
- Given a changed architecture under `architectures/`, when `task check` runs without
  regeneration, it fails naming the stale bundle.
- Given the `slow` lane, when it runs, it writes every bundle afresh and the bytes equal the
  committed ones.
- Given a copy of `cloudspec-accepts.yaml` with the `scheduled` kind removed, when
  `agenticspec deploy bundle` runs on `architectures/scheduled-loops.yaml` packed with a scheduled
  unit, it exits 1 as `unit_not_hosted` naming the kind.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The vendoring and coverage test in cloudspec (request C7); cloudspec's published list (request C4).

## Remaining at 9bb5998

Delivered in 43047c2. Missing: cloudspec_commit is null in bundles/cloudspec-accepts.yaml:2; waits on cloudspec C4.
