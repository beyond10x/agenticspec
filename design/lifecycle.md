# Lifecycle: the operations on an architecture, from design to operation

Status: design for approval, nothing built. Decisions L1-L6, open points LQ1-LQ6 with the option
recommended. The lifecycle is not part of any architecture: it is what a designer, an agent and
an operator do to a specification and its architecture: design, check, test, evaluate, optimize,
release, deploy, operate. Deploying is the last of them (`design/deployment.md`).

## L1 Where it lives: the `studio` system, extended

| option | for | against |
|---|---|---|
| **A: extend `studio/`** (taken) | it already holds the operation set ("query, apply a patch, check, simulate", `studio/system.yaml`), the caller of each call (`studio.session.Caller` `User, Agent`) and the evaluations; one tool protocol serves the UI, an external agent and the optimizer (epic `agenticstudio` § "Operations") | the name says app, the scope is the designer's workbench |
| B: a new ESS system | neutral name | a third system naming `agentic` by value, with the same drafts |
| C: in `blocks/` | one system | the blocks say what an architecture is made of; a designer's verdicts on it are not building blocks |

Deploy and operate records stay in `agentic` (`agentic.deployment`, `design/deployment.md` K10):
a running runtime and its control plane read them without any studio. The studio holds a
deployment by identifier only, as it holds revisions (`studio/domains/shared.yaml`). The
command line stays stateless: `agenticspec sim …` computes, the studio server records.

## L2 The stage of a revision is derived, not a state machine

Evidence binds to a revision's digest, and a new revision carries none of it. Optimizing does not
move a revision forward: accepting a variant applies a patch and commits a new revision
(`agentic.architecture.PatchApplication`), which starts again at `Drafted`. So the stage is a
view (`studio.release.RevisionStages`) computed from evidence, never a lifecycle field:

| stage | reached when, at the revision's digest |
|---|---|
| `Drafted` | the revision exists (`agentic.architecture.ArchitectureRevision`) |
| `Checked` | a `Verification` of kind `Check` passed |
| `Tested` | a `Verification` of kind `Scenario` or `Corpus` passed for every scenario the draft names, at least one |
| `Evaluated` | an `Evaluation` of kind `Simulate` is `Done`, and every requirement decided at evaluation is met (`design/deployment.md` K12) |
| `Released` | a `Release` is `Committed` |
| `Deployed` | a `Deployment` of the revision is `Bundled`, `Translated` or `Realizing` |
| `Operating` | a `Deployment` of the revision is `Running` or `Degraded` |

## L3 New nouns in `studio/`

- `studio.verification.Verification`: draft, revision, digest, kind (`Check`, `Scenario`,
  `Corpus`), subject (the scenario or corpus path), result digest, the tool call; `Running`, then
  `Passed` or `Failed`.
- `studio.release.Release`: draft, revision, digest, contract digest, path, the verifications and
  evaluation it rests on, commit (optional); `Written`, `Committed`, then `Superseded` or
  `Withdrawn`.
- `studio.session.Tool` gains `Test`, `Release`, `Deploy`; `Caller` gains `Optimizer` and `File`
  (L5). `studio.evaluation.Evaluation` gains `requirements_met`. `studio.shared` gains
  `VerificationId`, `ReleaseId` and `DeploymentRef` (an `agentic` deployment id by value).

## L4 Operations, refusals, who, and the commands that exist

| operation | commands | refused as | who | command line | studio tool |
|---|---|---|---|---|---|
| design | `OpenDraft`; `CallTool(Apply)` to `ApplyPatch` | `ArchivedDraft`; the patch's rule and error | designer, agent | the files | `query`, `apply` (`story:studio-mcp`) |
| check | `StartVerification(Check)`, `CompleteVerification` | `NoSuchRevision` | designer, agent | `agenticspec sim check` (exists) | `check` |
| test | `StartVerification(Scenario \| Corpus)` | `NotChecked`, `NoScenario` | designer, agent | `agenticspec-sim scenario run` (`story:scenario-simulator-run`), a corpus load (`story:task-corpus`) | `test` (new) |
| evaluate | `StartEvaluation(Simulate)` | `NotChecked` (new), `InvalidBudget`, `ArchivedDraft` | designer, agent | `agenticspec sim run` (exists) | `simulate` (`story:studio-evaluation`) |
| optimize | `StartEvaluation(Search)`; accepting a variant is an `apply` | `NotChecked`, `NoMarkedPlaces` (new) | optimizer, designer, agent | `agenticspec sim run`, `optimize: marked` | `simulate(budget)`, accept |
| release | `ReleaseRevision(path)`, then `RecordCommit(commit)` | `NotChecked`, `NotTested`, `RequirementsMissed`, `StaleEvidence`, `FileChangedOnDisk` | designer, agent | `agenticspec sim contract` (`story:contract-derivation`) for the contract digest; the commit and push are the user's version control | `release` (new) |
| deploy | `agentic.deployment.RequestDeployment` | `NotReleased`, `NotAdmitted`, `CoverageRefused`, `TargetRefused`, `ScopeTaken` | deployer | `agenticspec packing default`, `agenticspec deploy bundle` (new), then cloudspec's translator and the target (`design/deployment.md` K13, K14) | `deploy` (new); its dry run answers admission, and coverage once cloudspec's translator is reachable |
| operate | the blocks' principal and administration commands | the ingress refusals (`design/ingress.md` I5, I9) | operator (`control.operator`) | none | none: the control plane (`design/control-plane.md` C3) |
| promote | `RequestDeployment` naming the deployment promoted from | as deploy | deployer | as deploy | `deploy` |

Who: the **designer** works in the studio's editor; an **agent** calls the same tools through the
tool protocol (`story:studio-mcp`) or the in-studio chat (`story:studio-chat`); the
**optimizer** is the search; the **deployer** is the designer for a local deployment and a
pipeline identity for a remote one; the **operator** is the `control.operator` audience; the
target's host reports units (`agentic.runtime.Host`).

## L5 Files on disk

A draft opened from a file follows it. A change on disk becomes a revision: the difference as a
patch, applied by a tool call with caller `File`, published on the draft's channel
(`story:studio-live`), so every client sees it. A release writes the revision back to its path,
refused as `FileChangedOnDisk` when the file changed since the studio last read it; then the
user's version control commits and pushes, and `RecordCommit` binds the commit to the release.

## L6 Every operation is reversible except two

Design, check, test, evaluate and optimize write evidence or a new revision; nothing is lost. A
release can be withdrawn. Deploying to a target and deleting a scope's state are not reversible
by the studio: the deploy tool asks for its own confirmation, and deleting stored data keeps the
approval rule of `design/events-and-storage.md` E5.

## Open points

| # | question | options | recommended |
|---|---|---|---|
| LQ1 | the system | A: extend `studio/`. B: a new system. C: `blocks/` | A |
| LQ2 | the stage | A: derived from evidence at a digest. B: a lifecycle on the revision | A |
| LQ3 | the test gate of a release | A: at least one scenario passed. B: waivable with a reason | A |
| LQ4 | who commits | A: the user's version control; the studio records the commit. B: the studio commits | A |
| LQ5 | a change on disk | A: a new revision with caller `File`. B: a prompt to reload | A |
| LQ6 | records of deploy and operate | A: `agentic.deployment`, the studio holds ids. B: in `studio/` | A |

## Proposed stories

1. **studio-verification**: `Verification`, the `test` tool, `NotChecked` on evaluations;
   extends the tool set of `story:studio-mcp` and `story:studio-evaluation`.
2. **studio-release**: `Release`, `RevisionStages`, the `release` tool, write-back; new.
3. **studio-file-follow**: L5; extends `story:studio-live`.
4. **studio-deploy**: the `deploy` tool over `design/deployment.md` stories 1-4; new.

Every story that runs tests keeps each test within 60 s and its step within 120 s. A job in the
build pool ends within 10 minutes.
