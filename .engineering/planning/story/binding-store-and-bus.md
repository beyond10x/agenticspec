---
format: aep.planning-md/3
id: story:binding-store-and-bus
kind: story
status: draft
title: Binding store and bus parts, with in-memory and replay fixtures
summary: agenticspec-binding/1 binds a store part to a service kind, place, secret grant, schema mapping and vector route, an explicit bus to a transport; in-memory and replay store fixtures; unbound external store refused.
relations:
- decomposes: epic:events-and-storage
- depends_on: story:store-and-bus-ports
- depends_on: story:first-runtime
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-runtime
- confidence: inferred
  path: docs/binding.md
revision: 2
---
## Outcome

`agenticspec-binding/1` says how a store part and an explicit bus are played. Per store part: the
kind of service that plays it (in memory, a directory of files, a database service, an object
store, a vector index, a graph service: kinds, never products), where it lives, its credentials
through a sandbox secret grant, the mapping of record types to its schema, and for a semantic
store the route that makes vectors. Per explicit bus: the transport, retention and where cursors
are kept. An in-memory store and a store replayed from a recording are fixtures, admitted like any
implementation, with no test-only code path; admission refuses an `external` store the binding
does not bind. Design: `design/events-and-storage.md` E5, E8, P8 A. Epic:
`epic:events-and-storage`.

## Scope

- The binding format's store and bus sections and their check (inferred: `docs/binding.md` and the
  binding module of `story:first-runtime`).
- The in-memory and replay fixtures for `store.put`, `store.get`, `store.query`, `store.expire`
  and `bus.deliver` (inferred: the runtime crate of `story:first-runtime`).
- Admission: the refusal of an unbound `external` store.
- Tests and `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given a binding whose store section names a product instead of a service kind, when the binding
  check runs, it is refused naming the field.
- Given a document with an `external` store and a binding that does not bind it, when admission
  runs, it refuses naming the store, beside every other missing port.
- Given a document with a `record` store bound to the in-memory fixture, when a scenario puts and
  gets a record in the first runtime, the get returns the put's digest.
- Given a recording of three puts, when the replay fixture serves `store.get` for the second key,
  it returns the recorded digest.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The binding format itself and admission (`story:first-runtime`); a real storage service.
