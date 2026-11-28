---
format: aep.planning-md/3
id: story:ui-conform-server
kind: story
status: implemented
title: Check a running server against the runtime UI interface
relations:
- decomposes: epic:control-plane
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:25Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

agenticspec ui conform --architecture [--source] checks a server's reads, channel resume, commands and action sets against ui/runtime.openapi.yaml and names each failed obligation.

## Delivered

411fb2b, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of 411fb2b pass in `slot task check`.
