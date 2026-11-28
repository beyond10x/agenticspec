---
format: aep.planning-md/3
id: story:forge-thread-parity
kind: story
status: implemented
title: The local test step runs on the forge's thread count
relations:
- decomposes: epic:building-blocks
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

task test passes --test-threads 4 everywhere; the default profile ends the run at 75 s, the forge profile at 120 s; the PR #44 defect put back is stopped locally.

## Delivered

de82503, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of de82503 pass in `slot task check`.
