---
format: aep.planning-md/3
id: story:render-row-wrap
kind: story
status: implemented
title: A band's or region's wide row wraps into rows
relations:
- decomposes: epic:docs-site
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A row wider than page::WRAP_WIDTH wraps in order; edges between bands share lanes; the 109-node banded fixture opens at fit 0.268 (was 0.097).

## Delivered

dd1a86e, eeb418b, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of dd1a86e, eeb418b pass in `slot task check`.
