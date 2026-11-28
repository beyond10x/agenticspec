---
format: aep.planning-md/3
id: story:agent-run-lift
kind: story
status: draft
title: Lift agent run records into block events
summary: Per turn of a Loom agent its run record, forwarded as a resumable feed per instance, is lifted into harness session, turn, tool call and inference events labelled with unit, part path and instance.
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:deployment-domain
- informed_by: story:session-lift
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-records/src
- confidence: inferred
  path: crates/agenticspec-records/tests
revision: 2
---
## Outcome

The records of an agent's runs come back as block events. Per turn of a Loom agent its record (a
line of `record.jsonl`; the run events of Loom's generated contract `run-events-v1`) arrives as a
resumable feed per instance, which the target forwards (Q6 A, request C5); this story lifts each
record into block events (harness session, turn, tool call, inference) with the unit, part path and
instance from the source label, so the `core` unit records them and the control plane's Activity
page shows them. Loom stays unaware of the blocks. Design: `design/deployment.md` K15, Q5 A, Q6 A.
Epic: `epic:architecture-deployment`.

## Scope

- The lift from a run record to block events (inferred: `crates/agenticspec-records/src/`, beside
  the session lift of `story:session-lift`).
- A recorded fixture of run records copied from Loom's published contract (inferred:
  `crates/agenticspec-records/tests/`).
- `docs/` section on the lift; `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given a recorded feed of three turns of one agent instance, when the lift runs, it emits one
  harness session and three turns, each carrying the unit, part path and instance of the source.
- Given a record whose kind the contract does not declare, when the lift runs, it refuses that
  record by name and lifts the others.
- Given the same feed resumed from the second record, when the lift runs, it emits only the events
  of the second and third turns.
- Given the same feed lifted twice, both outputs are the same bytes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The target's feed and observation contract (cloudspec C5); the `core` unit's reader of a remote
target.
