---
format: aep.planning-md/3
id: story:ingress-format
kind: story
status: implemented
title: 'The ingress: key of the architecture format and its check rules'
summary: 'A top-level ingress: key (audience, authenticated, interface, exports), the eleven I9 check refusals, the audience counted as a name, best.rs TOP, a reference page and the workstation example.'
relations:
- decomposes: epic:ingress
- depends_on: story:administration-commands
- depends_on: story:contract-derivation
- depends_on: story:architecture-references
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: architectures/workstation.yaml
- confidence: cited
  path: crates/agenticspec-sim/src/best.rs
- confidence: cited
  path: crates/agenticspec-sim/src/check.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/doc.rs
- confidence: inferred
  path: crates/agenticspec-sim/tests
- confidence: inferred
  path: docs/architectures
- confidence: inferred
  path: docs/catalog
- confidence: inferred
  path: docs/ingress.md
- confidence: inferred
  path: website/static/catalog
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:18Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`agenticspec-architecture/1` gets a top-level `ingress:` key: per ingress an audience (a principal
part), `authenticated` (default true), an interface (`scope`, `commands`, `views`, `feeds`,
`metrics`) and a list of exports by protocol (`http`, `event_stream`, `cli`, `tool`, `agent`,
`metrics_scrape`, `ui`). `agenticspec sim check` holds the eleven refusals of I9, counts an
ingress's audience as a name (so a principal part reached only by an ingress is not unreachable),
and refuses a protocol outside the table. `best.rs` `TOP` keeps `ingress` after `border`. The key
stays outside the nesting digest. `architectures/workstation.yaml` gets the worked example (`user`,
`operator`, `metrics`) and a reference page documents the key. Design: `design/ingress.md` I1-I3,
I6, I8, I9 (check), Q1 A, Q2 A, Q3 A, Q4 A, Q9 A, Q10 A. Epic: `epic:ingress`.

## Scope

- `crates/agenticspec-sim/src/doc.rs`, `src/check.rs`, `src/best.rs`: the key, the check rules,
  `TOP`.
- `crates/agenticspec-sim/tests/`: one refused case per rule and the accepted example, each under
  5 s.
- `architectures/workstation.yaml`: parts `admin`, `monitor` and the three ingresses.
- `docs/architectures/index.md` and a reference page for `ingress:` (inferred:
  `docs/ingress.md`), with the boundary to cloudspec.
- Generated: `docs/architectures/`, `docs/catalog/`, `website/static/catalog/`
  (`task catalog:generate`, `task docs:generate`).
- `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given `architectures/workstation.yaml` with the three ingresses of the design's example, when
  `agenticspec sim check` runs, it exits 0 and does not refuse `admin` or `monitor` as unreachable.
- Given one fixture per I9 check rule (`ingress_audience_not_principal` through
  `principal_port_without_ingress`), when `agenticspec sim check` runs on each, it exits 1 naming
  that rule and the ingress.
- Given an ingress with an export `protocol: grpc`, when the check runs, it exits 1 naming the
  value and the allowed list.
- Given the workstation document with and without `ingress:`, when `agenticspec sim run` runs the
  committed experiment with no `load.ingress`, both `result.json` files are the same bytes; if they
  differ, the example moves to a copy of the document and this line names the copy.
- Given a nested document whose `ingress:` changes, when the enclosing document is checked, its
  pinned digest still matches.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The entities and `Contract.ingress` (`story:ingress-entities`); resolving operations
(`story:interface-derivation`); the default ingresses (`story:control-plane-default`).
