---
format: aep.planning-md/3
id: story:architecture-ui
kind: story
status: draft
title: 'Derive the operator UI of an architecture: overview, nested pages, part pages, channels, actions'
summary: 'agenticspec ui derive turns an architecture, its contract and its view: key into one ess-ui/1 composition; the 37-part fixture opens every part in two steps at 1366x768 (M43).'
relations:
- decomposes: epic:view-layer
- depends_on: story:block-panels
- depends_on: story:contract-derivation
- depends_on: story:architecture-references
- informed_by: story:large-graph-view
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Taskfile.yml
- confidence: cited
  path: architectures/workstation.yaml
- confidence: inferred
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-docs/src/results.rs
- confidence: cited
  path: crates/agenticspec-sim/src/best.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/doc.rs
- confidence: cited
  path: crates/agenticspec-ui
- confidence: inferred
  path: docs/architectures/index.md
- confidence: cited
  path: scripts/check-generated.sh
- confidence: inferred
  path: spec/cli.yaml
- confidence: cited
  path: ui/views/examples
revision: 4
---
## Outcome

An architecture document derives its operator UI. `agenticspec ui derive` reads the document, the
documents it references, its contract and its optional `view:` key, and writes one `ess-ui/1`
composition: the overview, the navigation, the pages, the order of the panels and the actions.
Nothing at this commit goes from an architecture to `ess-ui/1`; `story:ui-generator` starts from an
`ess-ui/1` document. Consumer gap M43, second half. Design: `design/view-layer.md` D1, D2, D3, D4,
D5, D6, D9, D11, O1, O6, O7 and O9. Epic: `epic:view-layer`.

## Scope

- `crates/agenticspec-ui`: the derivation and `agenticspec ui derive --path <architecture>
  --contract <contract> --out <file>`, declared in `spec/cli.yaml` (the CLI is generated).
  - The rule of D3: the top document is the app and its overview is the navigation home; a nested
    architecture part `{ block: architecture, ref, digest }` is one overview page and one
    navigation section, recursively, and one row and one box in its parent, with an `instance`
    parameter when `count` > 1; a part is one part page; a link is a row of the reach table;
    policies and marked places are read-only on the overview.
  - Overview of one level: the parts table (nested architectures first, then by block kind, then
    by name), the reach table on `agentic.architecture.Reach` (by `architecture_id`, filterable by
    part), and the one-level graph through the studio's `DraftGraph` host component, read-only, on
    `KitZoomCanvas`, at natural size with pan, zoom and fit.
  - Part page: the panels of the views of every domain the part's ports use, read from its
    contract (the table from document block to domain belongs to `story:contract-derivation`; this
    story holds no copy), plus its own block's domain, all scoped to the part. The instance list
    comes first; the other panels follow in collapsed frames, one per domain (`frame: shared,
    collapsed: true`). A part with `count` or `per` lists its instances first, and parts that are
    `per` it are shown inside each instance's row.
  - Channels (D5, O7): one per architecture level, `carries` the events of the domains on that
    page, `direction: server_to_client`, `delivery: every_event`, `resume: from_last_seen`,
    `scope: param`, `session: {per: architecture_id}`.
  - Actions (D6, O6): principal ports of the contract become row actions (`principal.answer` is
    `Answer` on `OpenQuestions` rows; `principal.decide` is `ApproveDecision`/`RejectDecision` on
    `OpenApprovals` rows); links into the border from a principal part or from outside become
    header actions on the border part's page, each taking the intent command of the port its
    carried value uses. `model_call`, `tool_call` and `claim` are never actions. Every action is
    `visible` by `actor.may(<command>)`. The derivation holds no command name: it reads commands
    from `blocks/ports.json` and the contract (consumer gap M49: a carry maps to a command only
    through the port the contract names for the link).
  - Panels are copied from `ui/views/blocks.ui.yaml` byte for byte (D2).
  - The `view:` key (D4, O1): `order`, `labels` (by part path, panel, column), `hide` (panels,
    columns, actions), `groups` (labels only) and `home`. An entry that names a part, panel or
    column that does not exist, or that holds a read, a filter, a field list or a command, is
    refused with the entry's node path. A nested document's key applies on that document's page.
  - `sources.sha256` beside each derived document: the digest of the architecture document, every
    referenced document, `blocks/ports.json` and the generator's version (D1).
- `crates/agenticspec-docs/src/results.rs`: the catalog's `sources.sha256` digests the document
  without `view:` (for example through its canonical form), so a label edit forces no
  `task catalog:generate`.
- `crates/agenticspec-sim/src/best.rs`: `view` in the `TOP` key order, so the key survives the
  rewrite of a best variant.
- `crates/agenticspec-sim/src/doc.rs` (inferred): the reader accepts the top-level `view:` key.
- `docs/architectures/index.md`: the `agenticspec-architecture/1` reference documents `view:`.
- `architectures/workstation.yaml`: a `view:` key, as the golden example (O9).
- `ui/views/examples/` (new): the derived UI of `workstation` and of the 37-part fixture of
  `story:large-graph-view`, each with its `sources.sha256` (O9).
- `scripts/check-generated.sh`, `Taskfile.yml` (`ui:derive`, the slow lane), `CHANGELOG.md`.
- Tests: Playwright on the fixtures of `story:block-panels`, one server per run. Each test ends
  within 60 s and the test step within 120 s; a pool job over 10 minutes is killed.

## Shared surfaces

- `crates/agenticspec-ui`: after `story:block-panels`.
- `crates/agenticspec-sim/src/doc.rs` and the nested-architecture digest
  (`crates/agenticspec-sim/src/nest.rs`, `DIGESTED = ["border", "links", "parts"]`):
  `story:architecture-references`, which this story follows; `view:` stays outside that digest.
- `crates/agenticspec-docs/src/results.rs` and `website/static/catalog/`: `story:contract-docs`
  and the catalog generation.
- The 37-part fixture: written by `story:large-graph-view`; this story reads it, and writes it
  itself if that story has not landed, at the path that story names.
- `studio/studio.ui.yaml` host slot `DraftGraph`: the studio stories own it; this story uses it
  read-only.
- `scripts/check-generated.sh`, `Taskfile.yml`, `CHANGELOG.md`: one more line each.

## Acceptance

- Given `architectures/workstation.yaml` and its contract, when `agenticspec ui derive` runs, it
  writes `ui/views/examples/workstation.ui.yaml` and its `sources.sha256`; `ess ui check --path
  ui/views/examples/workstation.ui.yaml --model blocks` exits 0; running it again changes no byte.
- Given two derived documents that use the same panel, when they are generated, each copied widget
  equals its widget in `ui/views/blocks.ui.yaml` and the two SFC files are equal byte for byte
  (`task check:generated` passes).
- Given the overview of `workstation`, when it is read, the parts table orders nested
  architectures first, then block kind, then name; the reach table reads
  `agentic.architecture.Reach` with `architecture_id`; the graph shows one level; policies and
  marked places are read-only.
- Given an architecture with a nested part `{ block: architecture, ref, digest }`, when its UI is
  derived, the referenced document has its own overview page and navigation section, and the
  parent shows it as one row and one box that open that page; with `count` > 1 the page takes an
  `instance` parameter.
- Given the part page of `operator` (an `agent` part), when it is read, its panels include the
  views of `harness` (sessions, turns, tool calls) read from the contract, not only `Agents` and
  `AgentRevisions`; the instance list comes first and every other domain is a collapsed frame.
- Given the part `worker` (`count` 12) and `tree` (`per: worker`), when the page of `worker` is
  read, its first section lists the instances and `tree` is shown inside each instance's row.
- Given each architecture level, when its channel is read, it carries exactly the events of the
  domains on that page with `resume: from_last_seen`, `scope: param` and `session: {per:
  architecture_id}`.
- Given a contract with the principal ports `principal.answer` and `principal.decide`, when
  actions are derived, `Answer` is a row action on `OpenQuestions` and
  `ApproveDecision`/`RejectDecision` are row actions on `OpenApprovals`.
- Given the contract of `workstation`, when actions are derived, the link
  `owner -> operator carries prompt over chat` is a `SendMessage` header action on the page of
  `operator`, `operator -> gateway carries model_call` yields no action, and every action is
  `visible` by `actor.may(<command>)`.
- Given a `blocks/ports.json` fixture in which the intent command of `principal.answer` is
  renamed, when actions are derived, the row action calls the new name.
- Given a `view:` key with a label for part `worker`, when the UI is derived, the label is shown;
  given an entry naming a part that does not exist, or one holding a filter, the derivation exits
  non-zero naming the entry's node path.
- Given a label edit in the `view:` key of a catalog architecture, when `task check:generated`
  runs without `task catalog:generate`, it passes, and the nested-architecture digest of that
  document is unchanged.
- Given a best variant written by `crates/agenticspec-sim/src/best.rs` from a document with a
  `view:` key, when it is read, the key is present and unchanged.
- Given `docs/architectures/index.md`, when it is read, it documents the `view:` key and its
  entries.
- Given the 37-part fixture of `story:large-graph-view`, derived into `ui/views/examples/`, at
  1366x768: the first screen shows the parts table, nothing runs past the viewport horizontally,
  every part opens in two steps or fewer from the overview, links are read in the reach table
  filterable by part rather than drawn as edges, and the graph is drawn at natural size with pan,
  zoom and fit.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The Vue package and custom elements (`story:views-package`); scoped reads and the runtime
interface (`story:placements`, `story:runtime-ui-interface`); action sets from grants
(`story:action-sets`); a new grouping construct (D11).
