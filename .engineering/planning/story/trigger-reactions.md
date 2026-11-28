---
format: aep.planning-md/3
id: story:trigger-reactions
kind: story
status: draft
title: 'Trigger reactions: Wake or Command, a typed condition, one firing per subject'
summary: Trigger gets reaction kind Wake or Command with intake as a field mapping, the condition as a guard predicate over the event's fields, and a refused duplicate firing per subject_key (G15).
relations:
- decomposes: epic:events-and-storage
- depends_on: story:event-channel
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/7-collaboration/trigger.yaml
- confidence: inferred
  path: blocks/scenarios/trigger
- confidence: inferred
  path: docs/blocks/7-collaboration/trigger.md
- confidence: inferred
  path: website/src/generated
revision: 3
---
## Outcome

A trigger reacts to an event with more than waking an agent. `Trigger` gets a reaction kind:
`Wake` (today's behaviour; `agent_revision_id` becomes optional for other kinds) or `Command` (a
named command of the blocks as text, with `intake` as the mapping from the event's fields to the
command's input). The condition is a predicate over the event's fields only, in the guard predicate
form the blocks already use (`all`, `any`, comparisons, `defined`). A second firing per
`subject_key` is refused, closing the unmapped rest of gap G15. Design:
`design/events-and-storage.md` E2, P2 A, P3 A. Epic: `epic:events-and-storage`.

## Scope

- `blocks/7-collaboration/trigger.yaml`: the reaction kind, the condition and `intake` typed, the
  duplicate refusal per subject key, the `UNMAPPED:` markers of G15 removed.
- `blocks/scenarios/trigger/`: scenarios for a `Command` reaction, a condition that does not hold,
  and a duplicate firing.
- Generated: `docs/blocks/7-collaboration/trigger.md`, `website/src/generated/`.
- `CHANGELOG.md`.
- Time synthesis before and after; builds and tests through `slot`; no test over 60 s, the test
  step within 120 s, a pool job over 10 minutes is killed.

## Acceptance

- Given an authored scenario with an `OnEvent` trigger of reaction kind `Command` on
  `agentic.work.DeliverableAccepted` and no `agent_revision_id`, when `task check:blocks` runs, the
  trigger is defined and its firing names the command.
- Given an authored scenario that defines a `Wake` trigger without `agent_revision_id`, when
  `task check:blocks` runs, the definition is refused.
- Given an authored scenario that fires a trigger twice for one `subject_key`, when
  `task check:blocks` runs, the second firing is refused by name.
- Given `blocks/7-collaboration/trigger.yaml`, when
  `grep -c 'consumer gap G15' blocks/7-collaboration/trigger.yaml` runs, it prints 3 before this
  story (condition, intake, duplicate) and 0 after it.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Firing reactions in the simulator (`story:sim-store-and-bus`); a query language beyond the guard
form (P3).
