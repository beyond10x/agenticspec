---
format: aep.planning-md/3
id: story:hardening-findings
kind: story
status: draft
title: Close the open findings of the batch 5 hardening pass
relations:
- decomposes: epic:building-blocks
revision: 1
---
Open findings of the hardening pass of batch 5 (263cf32), each confirmed in source unless marked:

- 59 stored identifiers in blocks are never checked to exist (example: OfferAssignment, blocks/5-work/work.yaml:568).
- No conformance target for agenticstudio or the generated Rust of spec/: their suites run on the interpreter only.
- No mutation audit of blocks: one mutation class exceeds the 30-minute pool limit; split by domain or wait for a faster ess.
- No actor may issue ReviseDraft, AnswerToolCall or FailToolCall (studio/domains/session.yaml:297); studio.evaluation is served by no component.
- No view shows what RecordExposure stores (spec/domains/session.yaml:842).
- Not verified: a rollback that can leave no active installation (blocks/9-composition/deployment.yaml:580), two confirmed puts of one artifact digest (blocks/3-capability/store.yaml:539), deletion with no approval, a message over a missing link, a gateway with no model, Store and Event missing from the part kinds.
- 24 breaking enum additions since 0.4.1 with no CHANGELOG entry (scripts/spec-diff.sh 0.4.1).
