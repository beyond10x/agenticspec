---
format: aep.planning-md/3
id: story:ui-generator-panels
kind: story
status: draft
title: 'Generator additions for the view layer: widgets as SFCs, actor.may, metric, live refetch, embed, custom elements'
summary: agenticspec ui generate emits one SFC per widget and draws visible by actor.may from R5, metric, live refetch, the embed shell and a custom-element build mode (D8).
relations:
- decomposes: epic:view-layer
- depends_on: story:ui-generator
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/agenticspec-ui
- confidence: cited
  path: ui/kit
revision: 3
---
## Outcome

`agenticspec ui generate` can draw what the view layer needs, so the block panels and the
architecture compositions generate without a hand edit. `story:ui-generator` turns an `ess-ui/1`
document into Vue 3 SFCs over `ui/kit/` with 5 components (`collection`, `record`, `form`,
`filter_bar`, `choice`) and host slots; it emits pages, not reusable widgets, and has no `metric`,
no `live` effect, no `embed` shell and no `actor.may`. This story adds those, as D8 of
`design/view-layer.md` lists them. Epic: `epic:view-layer`.

## Scope

- `crates/agenticspec-ui`: the Vue emitter and its golden tests.
  - Widgets emitted as SFCs, one per widget, named by the widget, so the same widget gives the
    same file in every document that holds it (D2).
  - `visible` with `actor.may(<command>)`, answered from R5 (`GET {source}/api/actions`, D7). With
    no R5 at the source (404), every action is shown and refusals come from R4.
  - `metric`.
  - `live` with `effect: refetch` and `coalesce`, beside the existing `insert_or_patch`.
  - The `embed` shell: a page route inside a shell that holds nothing but the page.
  - A custom-element build mode for widget SFCs (O2): one element entry per widget that takes
    `source`, `architecture`, `part` and `instance` as attributes. Packaging and the host proofs
    are `story:views-package`.
  - The typed client keeps `createClient(base)`; a widget's `source` is that base.
- `ui/kit/`: the `embed` shell component and a `metric` component, if the kit has none.
- `CHANGELOG.md`.
- Tests: golden tests on small documents and vitest on fixtures. Each test ends within 60 s and
  the test step within 120 s; a pool job over 10 minutes is killed.

## Shared surfaces

- `crates/agenticspec-ui`: created by `story:ui-generator`, which this story follows;
  `story:block-panels` and `story:architecture-ui` add the `panels` and `derive` commands to the
  same crate after this story.
- `ui/kit/`: `story:ui-generator` and the studio stories; a component added here is additive.
- `CHANGELOG.md`: every story.

## Acceptance

- Given an `ess-ui/1` document with two widgets placed on one page, when `agenticspec ui generate`
  runs, it writes one SFC per widget and the page imports both; running it again changes no byte.
- Given the same widget in two documents, when both are generated, the two SFC files are equal
  byte for byte.
- Given a row action with `visible: actor.may(<command>)` and an R5 fixture answering
  `allowed: false` for that command, when the generated widget renders its rows, the action is not
  shown; with `allowed: true` it is shown; with no R5 at the source (404) it is shown.
- Given a widget whose `live` is `effect: refetch, coalesce: 1s` and a fixture channel script that
  sends three matching events within one second, when the widget is mounted, it issues exactly one
  refetch of its read, where before `live` supported only `insert_or_patch`.
- Given a `metric` widget over a fixture read, when it renders, it shows the fixture's value.
- Given an `embed` page, when its route is opened, the shell holds the page and no navigation,
  header or other page.
- Given the custom-element build of one widget, when a plain HTML page loads it with a `source`
  pointing at a fixture, the element renders the fixture's rows.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The block panels (`story:block-panels`); the derivation from an architecture
(`story:architecture-ui`); the Vue package and custom-element bundle (`story:views-package`); the
R5 answer from grants (`story:action-sets`).
