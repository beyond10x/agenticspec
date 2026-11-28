---
format: aep.planning-md/3
id: story:field-packs
kind: story
status: implemented
title: 'Field packs: the rule, the format and fields/coding'
summary: Move coding vocabulary, profile and KPIs out of the core into fields/coding
relations:
- decomposes: epic:fields
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: blocks/4-agent/harness.yaml
- confidence: cited
  path: blocks/8-place/workspace.yaml
- confidence: cited
  path: crates/agenticspec-docs
- confidence: cited
  path: crates/agenticspec-sim
- confidence: cited
  path: experiments
- confidence: cited
  path: fields
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: cited
  path: profiles
- confidence: cited
  path: scripts/check-kpis.sh
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:16Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:16Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:16Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Steps 1-3 of `design/fields.md`: the rule, the field pack format and `fields/coding/`, and the
leaks of the blocks made neutral.

## Scope

- `AGENTS.md` (the rule), `scripts/check-kpis.sh` (core KPIs name a core block).
- `fields/coding/field.yaml` (`agenticspec-field/1`), `fields/coding/profiles/coding.yaml` (moved
  from `profiles/`), `fields/coding/kpis.yaml` (the 8 KPIs), catalog rows (action classes, permission
  modes, resource kinds).
- `crates/agenticspec-sim`: reads profiles and KPIs from a field; experiments name `field: coding`.
- `blocks/8-place/workspace.yaml`, `blocks/4-agent/harness.yaml`: the leaks listed in the design.
- `crates/agenticspec-docs`: the catalog groups by field; regenerated results.

## Acceptance

- Given `fields/coding/`, `task check` passes and `profiles/coding.yaml` no longer exists.
- Given `blocks/`, `grep -n "AcceptEdits\|name: branch" blocks/4-agent/harness.yaml blocks/8-place/workspace.yaml` prints nothing.
- Given `kpis/catalogue.yaml`, none of the 8 coding KPIs is in it and each is in `fields/coding/kpis.yaml`.
- Given `experiments/`, every file names `field: coding`, and the catalog page groups entries under "Coding".

## Not in this story

A second field; protocol references (they come with a corpus).
