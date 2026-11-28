---
format: aep.planning-md/3
id: story:block-panels
kind: story
status: active
title: 'Block panels: one generated ess-ui/1 widget per block view, with fixtures and a gallery'
summary: agenticspec ui panels writes ui/views/blocks.ui.yaml from the compiled blocks, one widget per view and a gallery page per domain, held by ess ui check --model blocks at 0 errors (M43).
relations:
- decomposes: epic:view-layer
- depends_on: story:block-bindings
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Taskfile.yml
- confidence: cited
  path: blocks/1-principals/principal.yaml
- confidence: cited
  path: blocks/10-evaluation/experiment.yaml
- confidence: cited
  path: blocks/10-evaluation/kpi.yaml
- confidence: cited
  path: blocks/4-agent/agent.yaml
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: inferred
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-ui
- confidence: inferred
  path: docs/blocks
- confidence: cited
  path: scripts/check-generated.sh
- confidence: inferred
  path: spec/cli.yaml
- confidence: cited
  path: ui/views/blocks.ui.yaml
- confidence: inferred
  path: ui/views/fixtures
- confidence: inferred
  path: ui/views/gallery
- confidence: inferred
  path: website/src/generated
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T00:20:31Z", actor: "human:operator", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-28T00:20:31Z", actor: "human:operator", revision: 5}
---
## Outcome

Every view of the building blocks has a generated panel, and the panels are the reusable
components of the view layer. `agenticspec ui panels` reads the compiled `blocks/` alone and writes
`ui/views/blocks.ui.yaml`: one `ess-ui/1` widget per view plus a gallery page per domain, held
whole by `ess ui check --model blocks`. No panel exists at this commit: the one hand-written
`ess-ui/1` document is `ui/agenticspec.ui.yaml`, and a probe document over `blocks` reported
`coverage: 2 covered, 0 host-covered, 220 uncovered of 222 views and commands`. Consumer gap M43,
first half. Design: `design/view-layer.md` D2, D5, D10 and "What ess-ui/1 cannot express". Epic:
`epic:view-layer`.

## Scope

- `crates/agenticspec-ui`: the panel generator and `agenticspec ui panels --model <path> --out
  <file>`, declared in `spec/cli.yaml` (the CLI is generated).
  - One widget per view. Columns come from the view's fields, labels from ESS `naming.display`,
    the row key from the view's first field (`<entity>_id` in all 67 views at commit `310373a`),
    and a `state` field is drawn as a badge.
  - A panel takes a scope (`source`, `architecture`, `part`, `instance`) and nothing else; scoping
    is the runtime's job (D7).
  - `live` from the compiled model (D5): the events whose outcome subject is the view's source
    entity drive `effect: refetch, coalesce: 1s`; only the events that carry every field of the
    view use `insert_or_patch`, matched on the key. No panel polls.
  - Field neutrality (D10): the generator reads only the compiled model and names no block, view
    or command.
- An actor in each of the five domains that declare none (consumer gap M48: `agent`,
  `architecture`, `experiment`, `kpi`, `principal`), in `blocks/4-agent/agent.yaml`,
  `blocks/9-composition/architecture.yaml`, `blocks/10-evaluation/experiment.yaml`,
  `blocks/10-evaluation/kpi.yaml` and `blocks/1-principals/principal.yaml`. Without them
  `ess ui check --model blocks` refuses every section over their views (`section_readable`, 97
  errors over two documents in a consumer's probe); the network component of
  `story:block-bindings` removes only the `view_in_model` errors. Generated: `docs/blocks/` and
  `website/src/generated/` (`task docs:generate`).
- `ui/views/blocks.ui.yaml` (new, generated) and its `sources.sha256` (the digest of `blocks/` and
  the generator's version), following `website/static/catalog/*/sources.sha256`.
- `ui/views/fixtures/` (new): one fixture file per view, answering R1 in tests and the gallery.
- `ui/views/gallery/` (new): the generated gallery app on the fixtures, one page per domain.
- `scripts/check-generated.sh`: the digest check of `blocks.ui.yaml` and its generated SFCs.
- `Taskfile.yml`: `ui:panels`, and the slow lane that regenerates and compares bytes.
- `CHANGELOG.md`.
- Time `agenticspec ui panels` and `ess ui check` once each; the generator runs once per test
  run, not per test. Each test ends within 60 s and the test step within 120 s; a pool job over
  10 minutes is killed.

## Shared surfaces

- `crates/agenticspec-ui`: `story:ui-generator-panels` before this story, `story:architecture-ui`
  after it.
- `blocks/components.yaml`: written by `story:block-bindings` (S1 there adds the
  `reached_by: network` component this story's check needs); this story only reads it.
- The five block files that get an actor: `story:block-bindings`, `story:governance-types`,
  `story:architecture-references`, `story:contract-derivation` and `story:placements` change some
  of the same files; whichever lands second rebases on the other. `docs/blocks/` and
  `website/src/generated/` are regenerated by every story that changes a block.
- `ui/views/`: `story:architecture-ui` adds `ui/views/examples/`, `story:views-package` the package
  files.
- `scripts/check-generated.sh`, `Taskfile.yml`, `CHANGELOG.md`: one more line each, beside other
  stories.

## Acceptance

- Given the compiled `blocks/`, when `agenticspec ui panels` runs, `ui/views/blocks.ui.yaml` has
  exactly one widget per view in `ess specify compile --path blocks --format json` (67 at commit
  `310373a`) and one gallery page per domain that declares views (21).
- Given the widget of `agentic.harness.HarnessSessions`, when it is read, its columns are the
  view's fields in order with the labels of `naming.display`, its row key is
  `harness_session_id`, its `state` column is a badge, and its only inputs are `source`,
  `architecture`, `part` and `instance`.
- Given every widget, when its `live` is read, it lists exactly the events whose outcome subject
  is the view's source entity, with `effect: refetch, coalesce: 1s`, uses `insert_or_patch` only
  for an event that carries every field of the view, and no widget declares polling.
- Given a test system with one domain that is not in `blocks/`, when `agenticspec ui panels` runs
  on it, it writes a widget for that domain's view by the same rule.
- Given the generated document, when `agenticspec ui panels` runs again, no byte changes
  (`task check:generated` passes); given a changed block file and no regeneration, the digest
  check fails and names `ui/views/blocks.ui.yaml`.
- Given the gallery on the fixtures at 1366x768, when each domain page is opened, every panel
  shows its fixture rows and axe reports 0 serious or critical violations.
- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, each of `agent`, `architecture`, `experiment`, `kpi` and `principal` declares an actor,
  where before none did; `task check:blocks` and `task check:layers` pass.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.
- Given `ui/views/blocks.ui.yaml`, when `ess ui check --path ui/views/blocks.ui.yaml --model blocks`
  runs, it exits 0 with 0 errors (no `view_in_model`, `command_in_model` or `section_readable`
  error) and no read or command uncovered, where a probe document over `blocks` had 220 of 222
  uncovered.

## Not in this story

The derivation from an architecture, part pages and actions (`story:architecture-ui`); scoping of
reads (`story:placements`, `story:runtime-ui-interface`); the Vue package (`story:views-package`).
