---
format: aep.planning-md/3
id: story:views-package
kind: story
status: implemented
title: 'The panel as the unit of reuse: a Vue package and custom elements from one source'
summary: The generated panels, part pages and overviews ship as a Vue package and as custom elements branded by --kit-* tokens; agenticstudio's draft view shows the same panels (O2).
relations:
- decomposes: epic:view-layer
- depends_on: story:architecture-ui
- informed_by: epic:control-plane
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: scripts/ui-build.sh
- confidence: cited
  path: studio/studio.ui.yaml
- confidence: cited
  path: ui/studio
- confidence: inferred
  path: ui/views
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:15Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:15Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:15Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The panel is the unit of reuse, packaged two ways from one SFC source (O2): a Vue package for Vue
apps and agenticstudio, and custom elements that work in any stack. A block-view panel, a part page
and an architecture overview are each parameterised by `source`, `architecture`, `part` and
`instance` only. A host brands them with the `--kit-*` custom properties and keeps its own
navigation through DOM events. agenticstudio's draft view shows the same panels, with no copy of
any panel. Nothing at this commit packages a generated component for another app. Design:
`design/view-layer.md` D8, "Who provides what" and O2. Epic: `epic:view-layer`.

## Scope

- `ui/views/` (package files, new): a Vue package (npm-style, depending on `ui/kit`) that exports
  the generated panel, part-page and overview SFCs.
- The custom-element bundle built from the same SFCs, through the build mode of
  `story:ui-generator-panels`: `<agentic-part-panel source=... architecture=... part=...>` and one
  element per panel kind. The `--kit-*` custom properties pass into the shadow DOM. The element
  emits `agentic-navigate` and `agentic-command` DOM events.
- An `ess-ui/1` example that places a custom element through a host slot whose `reads` and
  `commands` declare what the element uses, so `ess ui check` holds it.
- `ui/studio/` and `studio/studio.ui.yaml`: the draft view imports the panels from `ui/views` and
  shows them on fixtures or a simulated run.
- `Taskfile.yml`, `scripts/ui-build.sh`: build the package and the bundle.
- `CHANGELOG.md`.
- Tests: vitest and Playwright on the fixtures of `story:block-panels`, one server per run. Each
  test ends within 60 s and the test step within 120 s; a pool job over 10 minutes is killed.

## Shared surfaces

- `ui/views/`: `story:block-panels` (`blocks.ui.yaml`, fixtures, gallery) and
  `story:architecture-ui` (`examples/`) before this story; this story adds package files only.
- `ui/studio/`, `studio/studio.ui.yaml`: the studio stories extend the same document and
  regenerate; this story adds the panels to the draft view.
- `Taskfile.yml`, `scripts/ui-build.sh`, `CHANGELOG.md`: one more entry each.

## Acceptance

- Given the package, when a Vue test app imports a block panel and mounts it with a fixture
  `source`, it renders the fixture's rows.
- Given the custom-element bundle loaded in a plain HTML page and in a React page, when
  `<agentic-part-panel source=... architecture=... part=...>` points at a fixture, each page shows
  the part page's panels with the fixture's rows.
- Given the host page sets `--kit-accent` on the element's ancestor, when the element renders, its
  accent colour is the host's value, with no change inside the bundle.
- Given a click on a row link and on an action, when the element handles it, it emits
  `agentic-navigate` and `agentic-command` DOM events with the target, and the host page's URL
  does not change unless the host acts on the event.
- Given the `ess-ui/1` example that places the element through a host slot, when `ess ui check`
  runs on it, it exits 0.
- Given agenticstudio's draft view on a fixture, when it is opened, it shows the same panels, and
  no panel SFC exists under `ui/studio/` (every panel is imported from `ui/views`).
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Scoped reads and live data from a runtime (`story:runtime-ui-interface`); action sets
(`story:action-sets`); the `embed` pages, which remain for a host that cannot load scripts.

## Control plane shell

From `design/control-plane.md` C6 and P8 A (`epic:control-plane`): the views package also ships
the app frame of a control plane as one shell component (navigation, scope switcher, theme tokens,
R5 wiring, the `embed` shell). The control plane and agenticstudio both mount it, as two apps.
The studio's draft view previews the draft's default control plane through
`agenticspec ui serve --sim` (`story:ui-serve-fixtures`, `story:sim-event-log`), with no copy of
any page.

- Scope: `ui/views/` (the shell component), `ui/studio/` (the draft view mounts it).
- Acceptance: given a Vue test app that mounts the shell with a fixture `source`, it shows the
  navigation of the default control plane's pages and switches scope; given agenticstudio's draft
  view, it mounts the same shell, and no shell component exists under `ui/studio/`.
- Order: the shell lands with this story; the studio's preview of the default control plane needs
  `story:ui-serve-fixtures` and `story:sim-event-log` and follows them, so this story does not wait
  on them.
