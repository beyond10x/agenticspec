---
format: aep.planning-md/3
id: story:settings-the-simulator-reads
kind: story
status: implemented
title: Every setting the simulator reads is declared (R96)
relations:
- decomposes: epic:simulator-and-search
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

model_share is declared on runtime and process parts; a test fails when the simulator reads a part setting the table does not declare.

## Delivered

b97b620, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of b97b620 pass in `slot task check`.
