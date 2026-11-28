---
format: aep.planning-md/3
id: story:conform-writes-and-scope
kind: story
status: implemented
title: ui conform proves same-origin writes, skips views not served, and honours a named scope (R90-R92)
relations:
- decomposes: epic:control-plane
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

ui conform sends Origin and given headers on writes and lists every check; a view answering 404 view_not_served is skipped by name; a top-level scope on a row or signal decides its scope on every plane.

## Delivered

ae5dc96, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of ae5dc96 pass in `slot task check`.
