---
format: aep.planning-md/3
id: epic:events-and-storage
kind: epic
status: draft
title: 'Events and storage: an event bus and stores as building blocks'
summary: Parts subscribe to block events and react with a command; data lives in a store block of five kinds, scoped to a part, the architecture or outside it (E1-E10).
relations:
- informed_by: epic:building-blocks
revision: 1
---
## Outcome

Parts of an architecture emit events and other parts subscribe to them; a subscription with a
condition starts an action, such as storing a record. Data a part writes and another part reads has
a place to live: a store of kind `record`, `artifact`, `key_value`, `semantic` or `graph`, scoped to
one part, the whole architecture, or outside it. Two existing blocks are extended (`channel`,
`trigger`) and one is added (`store`, layer 3). The runtime's event log is the bus; a `channel` of
kind `Event` names it when it must cross a boundary or carry settings. Storing and publishing are
IO, so they are ports (`store.put`, `store.get`, `store.query`, `store.expire`, `bus.publish`,
`bus.deliver`), narrowed in the contract and bound by the runtime binding to a kind of service,
never a product.

Design: `design/events-and-storage.md` (decisions E1-E10, open points P1-P9). Approved as written,
every open point at its recommended option.

## Decisions taken

- E1: the bus is the runtime's implicit event log, plus a `channel` of kind `Event` with
  `distribution: topic | queue` when it needs a name; a link `carries: event` to the bus is an emit,
  from it a delivery.
- E2: a subscription with an action is a trigger with a reaction kind `Wake` (today's) or `Command`
  (a named command, `intake` as the field mapping); `ConcurrencyPolicy` stays the back-pressure.
- E3: delivery is at least once, ordered per source and subject, resumable from a cursor, retained
  for a declared time with `CursorExpired` refused by name; the obligation R2 of the view layer.
- E4: one `store` block with five kinds; the blocks record the write, key, digest and writer, never
  the bytes. An artifact is a record of a store of kind `artifact` (immutable, digest-addressed,
  provenance required).
- E5: scope `part` (`per:`), `architecture` (default) or `external`; retention as a duration,
  `run` or `forever`; deleting an unexpired record defaults to `ApprovalRequired`.
- E6: reach by link, grant by authority, egress by sandbox; an agent reaches a store through a tool.
- E7: knowledge names the store it draws from; handback, deliverables and evidence name artifact or
  record ids.
- E8: the six ports above; fixtures (in memory, replay) are capabilities; the simulator gives
  delays, costs and loss with defaults that keep every result byte for byte.
- E9: neutral names only; a field brings record types and artifact classes as data.
- E10: layer placement as in the design's table.
- P1 A, P2 A, P3 A (a condition reads the event's fields only, in the guard predicate form), P4 A,
  P5 A, P6 A, P7 A (reads are recorded ports), P8 A (`scope: external`, bound at admission), P9 A
  (one `Carries` value `Event`).

## Stories, in order

1. `story:store-block`: the `store` domain, its entities, events, views and scenarios.
2. `story:artifact-references`: deliverables, evidence and handback name artifacts and records.
3. `story:event-channel`: `channel` kind `Event`, `Publisher`, event identity, `CursorExpired`.
4. `story:trigger-reactions`: `Wake` or `Command`, the condition as a guard predicate, the
   duplicate refusal per subject key.
5. `story:architecture-store-and-bus`: part kind `Store`, `Carries` `Event`, settings and check
   rules.
6. `story:store-and-bus-ports`: the six ports in the port and contract derivations.
7. `story:bus-delivery-conformance`: the delivery obligations, shared with the R2 feed.
8. `story:sim-store-and-bus`: store and delivery costs, store events and `Command` reactions in the
   simulator.
9. `story:evidence-stores`: the ladder's evidence parts become store parts.
10. `story:binding-store-and-bus`: the binding of store and bus parts and the in-memory and replay
    fixtures.

Every story that runs tests keeps each test within 60 s and its test step within 120 s. A job in
the build pool ends within 10 minutes.

## Not in this epic

A concrete storage product or service (a binding's concern); a query language beyond the guard
predicate form (P3); a new bus block (P1).
