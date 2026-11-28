---
format: aep.planning-md/3
id: story:render-page-model
kind: story
status: implemented
title: Render any graph from a neutral page model
relations:
- decomposes: epic:docs-site
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:26Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

agenticspec_render::page::render draws a page from agenticspec-render-model/1, which names no agenticspec term, with unresolved ports, via and region frames; the architecture pages use the same function.

## Delivered

3e0572b, 9623bbf, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of 3e0572b, 9623bbf pass in `slot task check`.
