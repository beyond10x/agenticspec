---
format: aep.planning-md/3
id: story:deployment-bundle
kind: story
status: active
title: 'agenticspec deploy bundle: the agenticspec-deployment/1 format'
summary: Admit, then write one YAML bundle with every member inline keyed by digest, per-unit fields and abstract roles, no secret, host, provider or target; a refusal writes nothing and lists every finding.
relations:
- decomposes: epic:architecture-deployment
- depends_on: story:packing-admission
- depends_on: story:requirements-kpis
- depends_on: story:interface-derivation
- depends_on: story:architecture-references
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
  path: docs/deployment.md
- confidence: inferred
  path: spec/cli.yaml
- confidence: inferred
  path: spec/domains/cli.yaml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:20Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:20Z", actor: "human:operator", revision: 4}
---
## Outcome

`agenticspec deploy bundle <architecture> --binding <file> --out <file>` admits and, only when
admitted, writes `agenticspec-deployment/1`: one YAML file with every member inline, each document
keyed by its digest (`format`, `generator`, `architecture`, `nested`, `contract`, `binding` with its
`refines:` chain resolved, `packing`, `units`, `ingress`, `requirements`, `agents`, `runtimes`,
`digest`). Each unit carries name, kind, parts, replicas, code by digest (an agent unit stays one
agent: its agent document by digest and the role `agent-runtime`), claims (with `volumes` from the
`store` role, Q16 A), isolation in agenticspec's own levels (Q17 A), checks, schedule, secrets by
name, egress, abstract roles, ports and source. The bundle carries no secret value, host, provider
or target name. A refusal writes nothing and lists every finding of K14 as `NotAdmitted`. Same
inputs, same bytes. Design: `design/deployment.md` K6, K7, K14, Q1 A-Q4 A, Q8 A, Q12 A, Q16 A,
Q17 A. Epic: `epic:architecture-deployment`.

## Scope

- A bundle module (inferred: in the runtime crate of `story:first-runtime`, or
  `crates/agenticspec-deploy/`): members, digests, per-unit fields, roles, the K14 checks.
- `agenticspec deploy bundle`: declared in `spec/cli.yaml`, `spec/domains/cli.yaml`;
  `crates/agenticspec-cli-contract/` regenerated; `crates/agenticspec/src/main.rs` dispatches it.
- `docs/` reference of the format (inferred: `docs/deployment.md`); `CHANGELOG.md`.
- Tests, each under 5 s.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `architectures/workstation.yaml` and a test binding, when `agenticspec deploy bundle` runs
  twice, both runs exit 0 with the same bytes, and `digest` equals the SHA-256 of the canonical
  JSON of the document without `digest`.
- Given the same bundle, its `contract` member's digest equals `Contract.digest` from
  `agenticspec sim contract`, and its `architecture` member's revision digest equals the digest of
  `parts`, `links` and `border`.
- Given the same bundle, when it is searched for a secret value, a host name or a provider name
  from the binding, none is found; secrets appear by name only.
- Given a binding whose agent document's digest does not match the file, when the command runs, it
  exits 1 listing that finding with every other finding and writes no file.
- Given an agent unit in the bundle, its `code` names its agent document by digest and the role
  `agent-runtime`, and no process command.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The fixture bundles (`story:bundle-fixtures`); the translator and targets (cloudspec C1-C7).

## Remaining at 9bb5998

Delivered in 43047c2, 30d708d, fa4c997, 9eac003. Missing: nested documents are not carried (crates/agenticspec-sim/src/deploy/bundle.rs:280 writes an empty nested map) and requirements are copied without a check (bundle.rs:294).
