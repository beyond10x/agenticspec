---
format: aep.planning-md/3
id: story:large-graph-view
kind: story
status: implemented
title: 'Large architecture graphs: natural size with pan and zoom, a header that wraps at 390 px'
summary: The rendered architecture page shows a wide graph at its natural size with keyboard pan, zoom and fit, and its header does not overflow on a phone (consumer gap M15)
relations:
- decomposes: epic:docs-site
scope:
- confidence: cited
  path: crates/agenticspec-render/assets/page.css
- confidence: cited
  path: crates/agenticspec-render/assets/page.js
- confidence: cited
  path: crates/agenticspec-render/src/lib.rs
- confidence: cited
  path: crates/agenticspec-render/tests
- confidence: inferred
  path: docs/architectures
- confidence: cited
  path: website/e2e
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:16Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:16Z", actor: "human:operator", revision: 9, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:16Z", actor: "human:operator", revision: 10, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A large architecture stays readable on the docs site. A consumer's gap report (M15) found that a
graph of 37 parts renders about 4300 px wide and is scaled down to the page, with no zoom or pan,
and that the page header breaks at 390 px. Those measurements are the consumer's; no document in
this repository has more than 10 parts. What the code shows at `25fbfcb`:

- The rendered page scales the graph to the page width: `svg.graph { ... max-width: 100%; height:
  auto }` (`crates/agenticspec-render/assets/page.css:180`). The wrapper allows horizontal scroll
  (`page.css:179`), but the SVG never overflows it, so it never scrolls.
- The page sets the graph's natural width as `--graph-w` on the wrapper
  (`crates/agenticspec-render/src/lib.rs:152-157`); no rule of `page.css` reads it.
- The header is one flex row with no wrap (`page.css:123-131`); the only width breakpoint is
  1100 px (`page.css:153-156`).
- The docs embed that page in a frame (`website/src/components/blocks/ArchitectureFrame.tsx`,
  used by `docs/architectures/workstation.md:38`), so the fix is in the renderer.

## Scope

- `crates/agenticspec-render/assets/page.css`, `assets/page.js`, `src/lib.rs`: the graph at its
  natural size when it is wider than the card, inside a viewport that pans by drag, wheel and
  arrow keys, with zoom in, zoom out and fit buttons reachable by keyboard; fit is the default
  only when the natural width is within the card. The header wraps below the breakpoint the story
  sets, so nothing overflows at 390 px.
- A 37-part fixture written for the tests (not under `architectures/`), rendered by
  `agenticspec-sim render`.
- `website/e2e/`: a layout check under the gates of `story:site-ui-loop` on the fixture's page at
  1280 px and on a docs architecture page at 390 px.
- Generated pages through `task docs:generate` where the rendered markup changes.
- Builds and tests through `slot` (`slot npx playwright test --workers=2`); no test over 60 s,
  the test step within 120 s; a pool job over 10 minutes is killed.

## Shared surfaces

- `crates/agenticspec-render`, `website/e2e/`, `website/src/`: `story:site-ui-loop` (active) owns
  the loop and its gates; this story adds its checks there. `story:architecture-references`
  changes `crates/agenticspec-render/src/graph.rs` (one box per architecture part); this story
  does not change the layout in `graph.rs`.
- The studio's graph is `story:ui-generator`'s; this story does not touch `ui/`.

## Acceptance

- Given the 37-part fixture rendered and opened at 1280 px, the part names are drawn at their
  natural font size (the `.name` rule, 13.5 px) at the default zoom, and the graph pans
  horizontally to its last part, where before the whole graph was scaled into the card.
- Given that page, the fit button scales the graph to the card's width and zoom in restores it,
  each reachable with Tab and Enter.
- Given `architectures/solo.yaml` (3 parts), the graph is drawn at its natural size with no pan
  controls shown.
- Given the docs page of `architectures/workstation.yaml` at 390 px, the rendered page's
  `scrollWidth` is at most its `clientWidth` and the header's title, summary and facts are each
  fully visible.
- `task check:site` passes with these checks among its gates, and `task check:docs` passes.

## Not in this story

The studio's editor graph (`story:studio-editor`, `story:ui-generator`); a new layout algorithm in
`graph.rs`; collapsing groups of parts (`story:architecture-references` draws one box per
referenced architecture).
