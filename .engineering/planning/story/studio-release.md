---
format: aep.planning-md/3
id: story:studio-release
kind: story
status: draft
title: 'Studio releases: RevisionStages, the release tool and write-back'
summary: studio.release.Release and the derived RevisionStages view; ReleaseRevision writes the revision back, refused as NotChecked, NotTested, RequirementsMissed, StaleEvidence or FileChangedOnDisk; RecordCommit binds the commit.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-verification
- depends_on: story:requirements-kpis
- depends_on: story:contract-derivation
- depends_on: story:studio-drafts
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticstudio
- confidence: inferred
  path: docs/studio.md
- confidence: cited
  path: studio/domains/evaluation.yaml
- confidence: inferred
  path: studio/domains/release.yaml
- confidence: cited
  path: studio/domains/session.yaml
- confidence: cited
  path: studio/domains/shared.yaml
- confidence: inferred
  path: studio/ess-inputs.yaml
- confidence: inferred
  path: studio/system.yaml
revision: 3
---
## Outcome

The studio releases a revision and writes it back to its file. `studio.release.Release`: draft,
revision, digest, contract digest, path, the verifications and evaluation it rests on, commit
(optional); `Written`, `Committed`, then `Superseded` or `Withdrawn`. `ReleaseRevision(path)` writes
the revision to its path and is refused as `NotChecked`, `NotTested` (at least one scenario passed,
LQ3, recommended option A), `RequirementsMissed`, `StaleEvidence` or `FileChangedOnDisk`; `RecordCommit(commit)` binds
the commit the user's version control made (LQ4, recommended option A). The stage of a revision is a derived view,
`studio.release.RevisionStages` (`Drafted`, `Checked`, `Tested`, `Evaluated`, `Released`,
`Deployed`, `Operating`), computed from evidence at the digest, never a lifecycle field (LQ2, recommended option A). A
new `release` tool. `Evaluation` gains `requirements_met`. Design: `design/lifecycle.md` L2-L6.
Epic: `epic:agenticstudio`.

## Scope

- `studio/domains/release.yaml` (new), `studio/domains/session.yaml` (`Tool` `Release`),
  `studio/domains/evaluation.yaml` (`requirements_met`), `studio/domains/shared.yaml`
  (`ReleaseId`, `DeploymentRef`), `studio/system.yaml`, `studio/ess-inputs.yaml`.
- `crates/agenticstudio`: the `release` tool, the write-back, `RevisionStages`.
- `docs/studio.md`, tests, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given a checked revision with no passed scenario, when `release` is called, it is refused as
  `NotTested`.
- Given a checked and tested revision whose evaluation missed a requirement decided at evaluation,
  when `release` is called, it is refused as `RequirementsMissed`.
- Given a released revision whose file changed on disk since the studio read it, when `release`
  writes back, it is refused as `FileChangedOnDisk` and the file is unchanged.
- Given a release written and a commit recorded, when `RevisionStages` is read, the revision is
  `Released`; a new revision from an accepted variant is `Drafted`.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Committing or pushing (the user's version control); deploying (`story:studio-deploy`).
