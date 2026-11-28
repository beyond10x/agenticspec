---
format: aep.planning-md/3
id: story:unit-runtime-check
kind: story
status: implemented
title: deploy check compares a unit's runtime with the image its binding resolves
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:09Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:09Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:09Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

deploy check and deploy bundle refuse unit_runtime_mismatch when a core or service unit's code.runtime is not the image the carried binding resolves or is not among the bundle's runtimes.

## Delivered

0d73c86, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of 0d73c86 pass in `slot task check`.
