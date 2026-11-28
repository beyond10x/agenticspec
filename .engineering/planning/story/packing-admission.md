---
format: aep.planning-md/3
id: story:packing-admission
kind: story
status: draft
title: 'Admission of a packing: the plan''s units and every refusal'
summary: 'admit() lists units, part instances and links crossing units, or refuses with every finding: part_unpacked, part_packed_twice, per_split, shared_state_split, link_without_transport, isolation_unmet, scheduled_with_inputs, core_count, unit_not_hosted.'
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:packing-format
- depends_on: story:ingress-admission
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

Admission also admits the packing. The plan lists the units, their part instances and the links
that cross units; a refusal lists every finding: `part_unpacked`, `part_packed_twice`, `per_split`,
`shared_state_split` (a pool, a gateway's rate limit, an architecture-scope store or an exclusive
workspace reached from two units and bound in memory), `link_without_transport`, `isolation_unmet`,
`scheduled_with_inputs`, `core_count`, `unit_not_hosted` (a unit kind or role the copy of
cloudspec's accepted list does not hold). Extends `story:ingress-admission`. Design:
`design/deployment.md` K5. Epic: `epic:architecture-deployment`.

## Scope

- The admission module of `story:first-runtime` (inferred: the runtime crate): the packing
  refusals and the plan's units.
- Tests: one refused fixture per refusal and one accepted packing, each under 5 s.
- `docs/binding.md` (inferred): the refusals; `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given one fixture binding per refusal, when admission runs on each, it refuses naming that
  refusal and the parts or link involved.
- Given a binding with three packing faults at once, when admission runs, the refusal lists all
  three together with any missing ports and exports.
- Given the workstation with `by_part` and an in-memory gateway reached from two units, when
  admission runs, it refuses as `shared_state_split` naming the gateway.
- Given the workstation with the default packing, when admission runs, the plan lists one `core`
  unit and no link crossing units.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The accepted list's copy itself (`story:bundle-fixtures`); writing the bundle
(`story:deployment-bundle`).
