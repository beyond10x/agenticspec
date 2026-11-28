---
format: aep.planning-md/3
id: story:artifact-references
kind: story
status: draft
title: Deliverables, evidence and handback name artifacts and records
summary: Deliverable.artifact_id, evidence as record ids checked at acceptance, workspace handback as an artifact id, knowledge naming its store; branch, review_ref and KnowledgeScope Repository leave the core.
relations:
- decomposes: epic:events-and-storage
- depends_on: story:store-block
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/3-capability/knowledge.yaml
- confidence: cited
  path: blocks/5-work/work.yaml
- confidence: cited
  path: blocks/8-place/workspace.yaml
- confidence: inferred
  path: blocks/scenarios/work
- confidence: inferred
  path: blocks/scenarios/workspace
- confidence: inferred
  path: docs/blocks
- confidence: inferred
  path: website/src/generated
revision: 3
---
## Outcome

Deliverables, evidence and handback name records and artifacts instead of bare digests or source-
control fields. `agentic.work.Deliverable` of kind `Artifact` names its artifact id, and
`Deliverable.evidence` names record or artifact ids, so acceptance can check that the evidence
exists and that its `recorded_by` is not the submitter alone. The workspace's neutral `handback`
reference is an artifact id; `Proposal.branch` and `review_ref` leave the core (rows of the
source-control pack). A knowledge item may name the store its content is drawn from, one direction
only. `KnowledgeScope` `Repository` gets its neutral form, a scope that names a store or a
workspace. This takes step 3 of `design/fields.md` for the handback. Design:
`design/events-and-storage.md` E7, E9, E10, P6 A. Epic: `epic:events-and-storage`.

## Scope

- `blocks/5-work/work.yaml`: `Deliverable.artifact_id`, evidence as record ids, the acceptance
  guard on evidence.
- `blocks/8-place/workspace.yaml`: handback as an artifact id; `branch`, `review_ref` removed from
  the core.
- `blocks/3-capability/knowledge.yaml`: a knowledge item's optional store; the neutral scope.
- `blocks/scenarios/work/`, `blocks/scenarios/workspace/`: scenarios updated and added.
- Generated: `docs/blocks/`, `website/src/generated/`.
- `CHANGELOG.md`.
- Time synthesis before and after; builds and tests through `slot`; no test over 60 s, the test
  step within 120 s, a pool job over 10 minutes is killed.

## Acceptance

- Given an authored scenario that accepts an `Artifact` deliverable whose evidence names a record
  id that no store holds, when `task check:blocks` runs, the acceptance is refused by name.
- Given an authored scenario whose only evidence was recorded by the submitter, when
  `task check:blocks` runs, the acceptance is refused; with one record by another principal it is
  accepted.
- Given `blocks/8-place/workspace.yaml` at this story's commit, when
  `grep -cE 'branch|review_ref' blocks/8-place/workspace.yaml` runs, it prints 0.
- Given `blocks/` at this story's commit, when `task check:layers` runs, it passes and `knowledge`
  depends on `store` while `store` does not depend on `knowledge`.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The source-control pack rows that take `branch` and `review_ref` (`epic:fields`); evidence stores
in the ladder (`story:evidence-stores`).
