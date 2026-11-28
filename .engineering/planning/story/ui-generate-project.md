---
format: aep.planning-md/3
id: story:ui-generate-project
kind: story
status: implemented
title: ui generate --project writes a buildable project (R101)
relations:
- decomposes: epic:control-plane
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:34Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A project with the app, the kit sources, pinned vue and vite, a lock and a packer builds with npm ci && npm run build into the archive form of ui build --archive.

## Delivered

f05856d, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of f05856d pass in `slot task check`.
