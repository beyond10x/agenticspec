---
format: aep.planning-md/3
id: story:operator-ui-build
kind: story
status: implemented
title: The operator UI as static files from the binary alone
relations:
- decomposes: epic:control-plane
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:25Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:25Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:25Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

agenticspec ui build writes the operator UI of an architecture as static files with no node and no app shell, and a reproducible archive with its sha256; ui serve answers the runtime UI interface on built-in fixtures from the binary alone.

## Delivered

eeee723, c4a0427, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of eeee723, c4a0427 pass in `slot task check`.
