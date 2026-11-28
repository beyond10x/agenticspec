---
format: aep.planning-md/3
id: story:event-channel
kind: story
status: draft
title: 'Event channels: channel kind Event, Publisher, event identity, CursorExpired'
summary: 'A channel of kind Event is a named bus: the runtime posts events with their identity as actor Publisher, subscriptions belong to the reacting principal, an expired cursor is refused by name.'
relations:
- decomposes: epic:events-and-storage
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/7-collaboration/channel.yaml
- confidence: inferred
  path: blocks/scenarios/channel
- confidence: inferred
  path: docs/blocks/7-collaboration/channel.md
- confidence: inferred
  path: website/src/generated
revision: 3
---
## Outcome

The `channel` block can name a bus. `ChannelKind` gets `Event`; a post on an event channel carries
an event's identity (sequence number as the cursor, qualified name, source as architecture, part
path and instance, subject key, time, payload digest) instead of a message's act and proof; an
actor `Publisher` (the runtime) posts on it, so no party sends; the party of a subscription on it is
the principal that owns the reaction (a `Service` principal where needed, no new `PartyKind`). A
subscription resuming from a cursor older than the channel's retention is refused as
`CursorExpired`, never skipped. Design: `design/events-and-storage.md` E1, E3, P1 A. Epic:
`epic:events-and-storage`.

## Scope

- `blocks/7-collaboration/channel.yaml`: `ChannelKind` `Event`, the event identity on a post, the
  actor `Publisher`, retention, the refusal `CursorExpired`.
- `blocks/scenarios/channel/`: scenarios for an event post, a topic fan-out, a queue taken once and
  an expired cursor.
- Generated: `docs/blocks/7-collaboration/channel.md`, `website/src/generated/`.
- `CHANGELOG.md`.
- Time synthesis before and after; builds and tests through `slot`; no test over 60 s, the test
  step within 120 s, a pool job over 10 minutes is killed.

## Acceptance

- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, `agentic.channel.ChannelKind` lists `Event` and an actor `Publisher` may post on a channel.
- Given an authored scenario where a principal party sends on an event channel, when
  `task check:blocks` runs, the send is refused; a `Publisher` post is accepted.
- Given an authored scenario with two subscriptions on a `topic` event channel and one post, when
  `task check:blocks` runs, both subscriptions receive it; on a `queue` channel exactly one does.
- Given an authored scenario that resumes from a cursor older than the retention, when
  `task check:blocks` runs, the resume is refused as `CursorExpired`.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Reactions (`story:trigger-reactions`); the `Carries` value and part settings
(`story:architecture-store-and-bus`); the delivery obligations as conformance
(`story:bus-delivery-conformance`).
