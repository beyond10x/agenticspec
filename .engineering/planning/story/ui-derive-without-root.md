---
format: aep.planning-md/3
id: story:ui-derive-without-root
kind: story
status: implemented
title: ui derive runs outside the repository
relations:
- decomposes: epic:control-plane
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:35Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Without --root, ui derive reads the compiled blocks, panels, scripts and port catalogue built into the binary and gives the committed examples byte for byte.

## Delivered

93879b5, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of 93879b5 pass in `slot task check`.
