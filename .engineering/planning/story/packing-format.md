---
format: aep.planning-md/3
id: story:packing-format
kind: story
status: draft
title: 'Packing: the binding''s packing section, unit kinds, strategies, packing default'
summary: 'agenticspec-binding/1 packing: single (default), by_kind, by_part, groups; per-part schedule: job and external; unit kinds core to external; agenticspec packing default prints the resolved packing.'
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:first-runtime
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-cli-contract
- confidence: inferred
  path: crates/agenticspec-runtime
- confidence: inferred
  path: crates/agenticspec/src/main.rs
- confidence: inferred
  path: docs/binding.md
- confidence: inferred
  path: spec/cli.yaml
- confidence: inferred
  path: spec/domains/cli.yaml
revision: 2
---
## Outcome

A binding says which parts share a unit. `agenticspec-binding/1` gets a `packing:` section, never
the architecture: a strategy `single` (default: everything in `core`, `external` parts outside),
`by_kind`, `by_part` (`count: n` becomes n replicas) or `groups` (named part lists, the rest in
`core`); per part `schedule: job` (an agent woken only by `on_schedule` becomes a `scheduled` unit,
its trigger's `policy` and `max_parallel` the overlap rule) and `external` override it; `per` parts
follow their part; a `count` marked `vary` is packed at its built value. Unit kinds: `core`,
`agent`, `service`, `job`, `scheduled`, `store`, `model`, `external`, each naming its least
isolation. `agenticspec packing default <architecture>` prints the resolved packing to paste and
change. Design: `design/deployment.md` K1-K4, Q9 A, Q10 A. Epic: `epic:architecture-deployment`.

## Scope

- The binding module of `story:first-runtime`: the `packing:` section and its resolution (inferred:
  the runtime crate, `docs/binding.md`).
- `agenticspec packing default`: declared in `spec/cli.yaml`, `spec/domains/cli.yaml`;
  `crates/agenticspec-cli-contract/` regenerated; `crates/agenticspec/src/main.rs` dispatches it.
- Tests over `architectures/workstation.yaml` and `architectures/scheduled-loops.yaml`, each under
  5 s; `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `architectures/workstation.yaml` and no `packing:`, when `agenticspec packing default` runs,
  it prints one `core` unit holding every part except the `external` ones.
- Given the workstation with `strategy: by_part`, when the packing resolves, the worker unit has
  12 replicas and each worker's tree and sandbox are in the worker's unit.
- Given `architectures/scheduled-loops.yaml` with `schedule: job` on its scheduled agent, when the
  packing resolves, that agent is a `scheduled` unit whose overlap rule is its trigger's `policy`.
- Given the printed default pasted into a binding, when the packing resolves from it, the result
  is the same bytes as the default.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Refusals of a packing (`story:packing-admission`); the bundle (`story:deployment-bundle`).
