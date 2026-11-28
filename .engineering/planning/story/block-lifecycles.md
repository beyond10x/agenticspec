---
format: aep.planning-md/3
id: story:block-lifecycles
kind: story
status: implemented
title: Run, install, drain, rollback and restart lifecycles in the blocks
relations:
- decomposes: epic:building-blocks
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The blocks model the run, install, drain, rollback and restart lifecycles and the trigger subject policy.

## Delivered

9788c3e, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of 9788c3e pass in `slot task check`.
