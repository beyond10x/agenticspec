---
format: aep.planning-md/3
id: story:render-bands-and-panels
kind: story
status: implemented
title: Pages in bands, panels below the graph, refusal of unknown edge ends, fitted first view
relations:
- decomposes: epic:docs-site
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

layout: bands stacks one band per top-level region in list order; panels[].place: below draws a panel full width; edge_end_unknown refuses an edge whose end is no node or declared port; default output is byte-identical.

## Delivered

389b0eb, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of 389b0eb pass in `slot task check`.
