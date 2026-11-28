---
format: aep.planning-md/3
id: story:control-plane-default
kind: story
status: draft
title: 'The default control plane: operator and metrics ingresses by name'
summary: 'Default ingresses operator and metrics unless declared or none, implied audiences under the reserved path control, rule values principal: all, plays: administration, metrics: computed, and agenticspec ingress default.'
relations:
- decomposes: epic:control-plane
- depends_on: story:ingress-format
- depends_on: story:interface-derivation
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-cli-contract
- confidence: inferred
  path: crates/agenticspec-sim/src/check.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/contract
- confidence: inferred
  path: crates/agenticspec-sim/src/doc.rs
- confidence: inferred
  path: crates/agenticspec/src/main.rs
- confidence: inferred
  path: docs/ingress.md
- confidence: inferred
  path: spec/cli.yaml
- confidence: inferred
  path: spec/domains/cli.yaml
revision: 2
---
## Outcome

Every architecture has a default control plane with nothing authored. The ingresses `operator` and
`metrics` exist unless the document declares an ingress of that name (which replaces it) or sets it
to `none`; `ingress: none` removes both, replacing "nothing is implicit" of `design/ingress.md`
I8. Their audiences `control.operator` (human) and `control.monitor` (service) are principal parts
of the contract only, under the reserved path `control`, so no document digest, catalog digest or
simulator result changes; the check refuses a part named `control`. Three rule values join I2:
`principal: all`, `plays: administration`, `metrics: computed`. `agenticspec ingress default
<architecture>` prints the default as `ingress:` rules; pasted into the document they give the same
contract byte for byte. Extends `story:ingress-format` and `story:interface-derivation`. Design:
`design/control-plane.md` C1, C2, C7, P1 A, P2 A, P3 A, P5 A, P6 A. Epic: `epic:control-plane`.

## Scope

- `crates/agenticspec-sim/src/doc.rs`, `src/check.rs`: defaults by name, `none`, the reserved path.
- `crates/agenticspec-sim/src/contract/`: the implied audiences and the three rule values.
- `agenticspec ingress default`: declared in `spec/cli.yaml` and `spec/domains/cli.yaml`;
  `crates/agenticspec-cli-contract/` regenerated; `crates/agenticspec/src/main.rs` dispatches it.
- Tests, `docs/` reference of the default (inferred: `docs/ingress.md`), `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `architectures/workstation.yaml` without `ingress:`, when `agenticspec sim contract` runs,
  the contract holds ingresses `operator` and `metrics` with audiences `control.operator` and
  `control.monitor`.
- Given the output of `agenticspec ingress default architectures/workstation.yaml` pasted into a
  copy of the document, when the contract is derived from both, the two contracts are the same
  bytes.
- Given a document with `ingress: none`, when the contract is derived, it holds no ingress; with
  `operator: none` only `metrics` remains.
- Given a document with a part named `control`, when `agenticspec sim check` runs, it exits 1
  naming the reserved path.
- Given the default `metrics` ingress of the workstation, its metrics are exactly the KPIs of
  blocks in scope whose status is `computed` or `both`.
- Given every committed experiment result and catalog digest, when regenerated at this story's
  commit, each is the same bytes.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Pages (`story:control-plane-pages`); admission of the default (`story:ingress-admission`).
