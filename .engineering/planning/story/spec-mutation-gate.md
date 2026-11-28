---
format: aep.planning-md/3
id: story:spec-mutation-gate
kind: story
status: implemented
title: The spec and studio suites run in the gate and their mutants are held
relations:
- decomposes: epic:building-blocks
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:11Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

check:spec runs spec's suite; check:mutation fails on a mutant neither killed nor listed; five spec defects the hardening pass found are fixed, among them a negative ready, an answer after the deadline and a write to an immutable agent revision.

## Delivered

263cf32, 5d5f838, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of 263cf32, 5d5f838 pass in `slot task check`.
