---
format: aep.planning-md/3
id: story:unit-config
kind: story
status: implemented
title: Environment values and arguments of a unit from the binding (R87)
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:09Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A binding gives parts and units env and args, carried per unit as config; a {secret} value stays a secret; unit_config_conflict and unit_config_invalid.

## Delivered

75d24f3, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of 75d24f3 pass in `slot task check`.
