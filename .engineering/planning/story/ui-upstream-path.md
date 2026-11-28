---
format: aep.planning-md/3
id: story:ui-upstream-path
kind: story
status: implemented
title: A binding sets the upstream path of a UI (R100)
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:34Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:34Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:34Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

ui.<ui>.upstream.path goes into the unit's upstream.path and the built UI's config.json api_base; default /api/ unchanged; ui_upstream_path_invalid.

## Delivered

5c0d562, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of 5c0d562 pass in `slot task check`.
