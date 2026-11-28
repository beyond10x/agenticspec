---
format: aep.planning-md/3
id: story:blocks-design-review
kind: story
status: implemented
title: Guards from the design review of the blocks
relations:
- decomposes: epic:building-blocks
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:37Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:37Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:37Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A rollback keeps one installation active, one artifact is put once, a call through another gateway's upstream is NoUpstream and an architecture whose gateway has no model link does not commit.

## Delivered

cdb6965, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of cdb6965 pass in `slot task check`.
