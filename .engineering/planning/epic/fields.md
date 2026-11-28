---
format: aep.planning-md/3
id: epic:fields
kind: epic
status: active
title: 'Fields of work: a neutral core and field packs'
summary: Keep the blocks field-neutral; coding and later fields supply rows, profiles, KPIs, corpora and protocol references
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:40:30Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T23:40:30Z", actor: "human:operator", revision: 3}
---
## Outcome

The core blocks stay neutral across fields of work; field vocabulary, profiles, KPIs, corpora and
protocol references live in field packs under `fields/<field>/`. Plan and measured leaks:
`design/fields.md`.

## Stories

- `story:field-packs`: the rule in AGENTS.md, the `agenticspec-field/1` format, `fields/coding/`
  extracted from todays profile, KPIs and catalog rows, the blocks leaks made neutral.

## Acceptance

- `fields/coding/` holds the coding profile, its KPIs and catalog rows, and `task check` passes.
- No core block file contains an enum variant or field that `design/fields.md` lists as a leak.
- Every experiment and catalog entry names its field.

## Not in this epic

A second field without a real corpus; a protocol catalog (Runbook, Canon).
