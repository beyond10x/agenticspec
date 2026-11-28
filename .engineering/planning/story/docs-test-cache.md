---
format: aep.planning-md/3
id: story:docs-test-cache
kind: story
status: implemented
title: Build the docs once per test run
relations:
- decomposes: epic:docs-site
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The docs tests read pages generated once per run from a cache under target/; the slowest docs test went from 45.8 s to about 5 s.

## Delivered

0d0d92e, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of 0d0d92e pass in `slot task check`.
