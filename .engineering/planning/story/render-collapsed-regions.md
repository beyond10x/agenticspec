---
format: aep.planning-md/3
id: story:render-collapsed-regions
kind: story
status: implemented
title: A region drawn collapsed opens in place
relations:
- decomposes: epic:docs-site
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

collapsed: true draws a region as one card with its node count, merges edges with a count, opens and closes in place by click or keyboard; region_collapsed_empty and region_collapsed_invalid.

## Delivered

5832701, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of 5832701 pass in `slot task check`.
