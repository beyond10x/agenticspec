---
format: aep.planning-md/3
id: story:durable-state
kind: story
status: implemented
title: A runtime part's durable state gets a volume (R88)
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

durable_state on a runtime part gives its unit an exclusive <part>-state claim mounted from the binding; durable_state_unplaceable for a job, scheduled or model unit.

## Delivered

b440e2b, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of b440e2b pass in `slot task check`.
