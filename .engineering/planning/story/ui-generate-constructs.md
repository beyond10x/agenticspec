---
format: aep.planning-md/3
id: story:ui-generate-constructs
kind: story
status: implemented
title: ui generate draws row filters, sort, groups and more ess-ui/1 constructs (R108)
relations:
- decomposes: epic:control-plane
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

reads.filter row predicates, sort, group_by, style cards, refresh, actor, header help and metrics, icon actions, confirm and choice fields are drawn; the docs list every released construct not drawn yet.

## Delivered

aff9f03, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of aff9f03 pass in `slot task check`.
