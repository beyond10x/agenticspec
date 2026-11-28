---
format: aep.planning-md/3
id: story:bus-delivery-conformance
kind: story
status: draft
title: 'Bus delivery as conformance: at least once, order per source, resume, expired cursor'
summary: 'One suite holds the delivery obligations shared by the R2 feed, triggers and bus.deliver, run against the fixture feed first: exact resume after cursor c, dedup, order per source, CursorExpired.'
relations:
- decomposes: epic:events-and-storage
- depends_on: story:event-channel
- depends_on: story:runtime-ui-interface
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-ui
- confidence: inferred
  path: ui/README.md
- confidence: inferred
  path: ui/views/fixtures
revision: 2
---
## Outcome

The delivery semantics of the bus are conformance obligations, and they are the same obligations
as the R2 feed of the view layer: each subscription receives every matching event at least once; a
reaction is deduplicated by (subscription, sequence); events of one source and subject arrive in
emission order; resuming from cursor c yields exactly the events after c; a cursor older than the
retention is refused as `CursorExpired`. One suite holds them, run against the fixture feed of
`story:runtime-ui-interface` first and against a runtime's `bus.deliver` after it. Design:
`design/events-and-storage.md` E3; `design/view-layer.md` S6. Epic: `epic:events-and-storage`.

## Scope

- The conformance suite of the feed obligations beside the R2 suite of
  `story:runtime-ui-interface` (inferred: `crates/agenticspec-ui`, `ui/views/fixtures/`).
- A recorded event log fixture with two sources, a gap and an expired cursor.
- `docs/` page of the runtime interface: the obligations (inferred: `ui/README.md`).
- `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given the fixture feed and a cursor c, when the suite resumes from c, it receives exactly the
  events after c and the case passes; a fixture that drops one event after c makes the case fail.
- Given a fixture that delivers one event twice to one subscription, when the suite runs, the
  reaction counted for it is one.
- Given a fixture that reorders two events of one source and subject, when the suite runs, the
  order case fails naming both sequence numbers.
- Given a cursor older than the fixture's retention, when the suite resumes from it, the feed
  answers `CursorExpired` and the case passes; a feed that skips silently makes it fail.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

A real runtime (`story:first-runtime`); the ingress feed's own refusals
(`story:ingress-conformance`), which reuse this suite.
