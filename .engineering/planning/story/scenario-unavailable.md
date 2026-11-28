---
format: aep.planning-md/3
id: story:scenario-unavailable
kind: story
status: implemented
title: Scenarios with an unavailable part or dependency
relations:
- decomposes: epic:architecture-scenarios
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:27Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A scenario can make a part or one dependency unavailable and state what each remaining part shows.

## Delivered

feb3ab3, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of feb3ab3 pass in `slot task check`.
