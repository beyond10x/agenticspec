---
format: aep.planning-md/3
id: story:durable-state-writers
kind: story
status: implemented
title: A runtime's durable state names its outside writers (R109)
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

state_writers on a part with durable_state: true is carried into the volume claim and the architecture page; one volume, no workspace ports.

## Delivered

9ce1bf1, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of 9ce1bf1 pass in `slot task check`.
