---
format: aep.planning-md/3
id: task:site-s01-change-set
kind: task
status: implemented
title: Build the change set of docs-site round S-01
relations:
- decomposes: story:site-ui-loop
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T18:31:46Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T18:31:46Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-27T20:12:25Z", actor: "human:operator", revision: 4}
---
## Outcome

The change set of round S-01 (`ux/site/rounds/S-01/triage.md`): SUF-001, SUF-002, SUF-008,
SUF-005, SUF-013, plus tool fixes T2, T3, T4, T5, T6, T9.

## Acceptance

- Each finding's regression asset fails before its change and passes after; its expected-fail mark
  is removed.
- `task check:site` green within the 120 s step budget.
- Round S-02 reports each target metric named in the triage.
