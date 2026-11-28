---
format: aep.planning-md/3
id: story:check-json
kind: story
status: implemented
title: Print the architecture check as JSON
relations:
- decomposes: epic:simulator-and-search
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:28Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

agenticspec sim check --json prints agenticspec-check/1.

## Delivered

425fc50, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of 425fc50 pass in `slot task check`.
