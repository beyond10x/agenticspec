---
format: aep.planning-md/3
id: story:studio-verification
kind: story
status: draft
title: 'Studio verifications: the test tool and NotChecked'
summary: studio.verification.Verification at a revision's digest (Check, Scenario, Corpus), a test tool over scenario runs and corpus loads, NotChecked and NoScenario refusals, NotChecked on evaluations.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-mcp
- depends_on: story:studio-evaluation
- depends_on: story:scenario-simulator-run
- depends_on: story:task-corpus
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticstudio
- confidence: inferred
  path: docs/studio.md
- confidence: cited
  path: studio/domains/evaluation.yaml
- confidence: cited
  path: studio/domains/session.yaml
- confidence: cited
  path: studio/domains/shared.yaml
- confidence: inferred
  path: studio/domains/verification.yaml
- confidence: inferred
  path: studio/ess-inputs.yaml
- confidence: cited
  path: studio/system.yaml
revision: 3
---
## Outcome

The studio records tests as evidence at a revision's digest. `studio.verification.Verification`:
draft, revision, digest, kind (`Check`, `Scenario`, `Corpus`), subject (the scenario or corpus
path), result digest, the tool call; `Running`, then `Passed` or `Failed`. A new `test` tool runs a
scenario through `agenticspec-sim scenario run` or a corpus load and records the result;
`StartVerification(Scenario | Corpus)` is refused as `NotChecked` before a passed check and as
`NoScenario` when the draft names none; `StartEvaluation(Simulate)` is refused as `NotChecked`.
`studio.session.Tool` gains `Test`. Extends the tool set of `story:studio-mcp` and
`story:studio-evaluation`. Design: `design/lifecycle.md` L3, L4. Epic: `epic:agenticstudio`.

## Scope

- `studio/domains/verification.yaml` (new), `studio/domains/session.yaml`,
  `studio/domains/evaluation.yaml`, `studio/domains/shared.yaml` (`VerificationId`),
  `studio/system.yaml`, `studio/ess-inputs.yaml`.
- `crates/agenticstudio`: the `test` tool and the refusals.
- `docs/studio.md`, tests, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given a draft revision with no passed check, when the `test` tool is called, it is refused as
  `NotChecked` and no `Verification` is recorded.
- Given a checked revision that names a scenario, when the `test` tool runs it and it passes, a
  `Verification` of kind `Scenario` is `Passed` at that revision's digest.
- Given a checked revision that names no scenario, when the `test` tool is called, it is refused as
  `NoScenario`.
- Given an unchecked revision, when `simulate` is called, it is refused as `NotChecked`.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Releases and stages (`story:studio-release`).
