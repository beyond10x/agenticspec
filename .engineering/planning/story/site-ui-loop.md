---
format: aep.planning-md/3
id: story:site-ui-loop
kind: story
status: active
title: Run the docs site and result pages under the ui-iteration-loop
relations:
- decomposes: epic:docs-site
scope:
- confidence: cited
  path: crates/agenticspec-docs
- confidence: cited
  path: crates/agenticspec-render
- confidence: cited
  path: ux/site
- confidence: cited
  path: website/e2e
- confidence: cited
  path: website/src
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T16:40:04Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T16:40:04Z", actor: "human:operator", revision: 3}
---
## Outcome

The docs site (website/, the generated pages of `crates/agenticspec-docs`) and the simulation
result pages (`crates/agenticspec-render`) run under the ui-iteration-loop: a second adapter beside
the dashboard's, gates on every merge, rounds `S-nn` with findings turned into tests and a ratchet
that never gets worse.

## Scope

- `ux/site/`: adapter, scenarios (docs tasks), baseline, rounds, findings.
- `website/e2e/`: Playwright specs on the built site: layout contract, axe-core, keyboard, the
  screen-only user, the ratchet.
- Fixes to `website/src/`, `crates/agenticspec-docs`, `crates/agenticspec-render`.

## Acceptance

- `task check:site` runs the gates on the built site within the 120 s step budget, no test over
  60 s, and is part of `task check`.
- Round S-01 measured with its report, two review files and a triage; its change set of at most 5
  built failing-first; round S-02 shows each target metric moved.
- Every closed S1/S2 site finding has a regression asset.
