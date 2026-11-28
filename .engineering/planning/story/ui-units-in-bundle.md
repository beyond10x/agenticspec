---
format: aep.planning-md/3
id: story:ui-units-in-bundle
kind: story
status: implemented
title: The operator UI as a unit of its own in a bundle
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:24Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:24Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:24Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A binding with ui.operator gets a unit operator-ui whose code is the reproducible static archive of ui build, serving the operator ingress with an upstream to the runtime UI interface; its claims and check ports come from the binding.

## Delivered

fa4c997, 393b004, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of fa4c997, 393b004 pass in `slot task check`.
