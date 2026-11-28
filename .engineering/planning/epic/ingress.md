---
format: aep.planning-md/3
id: epic:ingress
kind: epic
status: draft
title: 'Ingress: the interfaces an architecture exposes to the outside'
summary: 'Per audience an ingress: an interface resolved from the contract (commands, views, feeds, metrics) and exports over http, event_stream, cli, tool, agent, metrics_scrape and ui (I1-I12).'
relations:
- depends_on: epic:architecture-contract
- informed_by: epic:view-layer
revision: 1
---
## Outcome

An architecture says what a client outside it may call, read or subscribe to, and over which
protocol. An ingress is that outermost layer, one per audience: an interface (what: commands,
views, feeds and metrics selected by rules and resolved from the contract) and one or more exports
(how: `http`, `event_stream`, `cli`, `tool`, `agent`, `metrics_scrape`, `ui`), each with a generated
projection. Ports are the outbound edge of the blocks; the ingress is the inbound edge. The operator
UI of the view layer becomes the `ui` export of an ingress.

Design: `design/ingress.md` (decisions I1-I12, open points Q1-Q10). Approved as written, every open
point at its recommended option.

## Decisions taken

- I1: three entities, `Ingress` (for whom), `Interface` (what), `Export` (how); an interface names
  no protocol, an export never narrows the interface.
- I2: an interface is derived from the contract by the keys `scope`, `commands` (`principal`,
  `plays`, `except`), `views`, `feeds`, `metrics`; it only selects and is an upper bound, not a
  grant.
- I3: the audience is a principal part of the document.
- I4: the port derivation splits configuration (creates its entity, never exposed) from
  administration (moves an existing entity, exposed through `plays`); an audience may not play an
  actor that issues a port's report.
- I5: three layers decide a call: interface (`NotInInterface`), actor `may:`, grant.
- I6: the scheme is the binding's; `authenticated: false` only for an interface without commands.
- I7: the protocol table, what each carries, its generated projection and what it needs.
- I8: a top-level `ingress:` key; entities in `agentic.architecture`; the resolved interface in
  `Contract.ingress` and its digest, outside the nesting digest.
- I9: eleven check refusals, exports as capabilities in admission, ingress conformance.
- I10: the view layer's UI is the `ui` export; `{root}/ingress/{ingress}/scopes/...`.
- I11: arrivals per ingress, administration operations and scrapes in the simulator, byte for byte
  at the defaults.
- I12: a domain `agentic.ingress` with one `Call` per inbound call, after I1-I11.
- Q1 A (top-level `ingress:` key), Q2 A (a principal part), Q3 A (inline per ingress), Q4 A, Q5 A
  (`agentic.ingress` after I1-I11), Q6 A (not served when nested), Q7 A (projections ESS lacks are
  generated here, committed with `sources.sha256`), Q8 A (no export kind for machine traffic), Q9 A
  (the name `ingress`), Q10 A (the contract's digest, outside the nesting digest).

## Stories, in order

1. `story:administration-commands`: the configuration and administration split in the port
   derivation.
2. `story:ingress-format`: the `ingress:` key, the check rules and the workstation example.
3. `story:ingress-entities`: `Ingress`, `Export`, their commands and view, `Contract.ingress`.
4. `story:interface-derivation`: interfaces resolved into operations in the contract.
5. `story:ingress-projections`: the generated projection per export.
6. `story:ingress-admission`: exports as capabilities in admission (with the control plane's floor).
7. `story:ingress-conformance`: the ingress refusals and the block suite through the `http` export.
8. `story:view-layer-per-ingress`: one UI per ingress with a `ui` export.
9. `story:sim-ingress-load`: arrivals, operations, rate limits and scrapes in the simulator.
10. `story:ingress-records`: `agentic.ingress` and its KPIs.

Every story that runs tests keeps each test within 60 s and its test step within 120 s. A job in
the build pool ends within 10 minutes.

## Boundary

The network entry point, its route, certificates, network limits and scrape registration are
cloudspec's (`design/ingress.md` § "Boundary with cloudspec"); this epic writes nothing of them.
Asked of ESS only when a second consumer needs one: a tool list, agent card or metric list
projection (Q7); a subset projection of `ess-http/1` is filed in ESS with a reproduction if
`story:ingress-projections` finds it missing.

## Not in this epic

A shared `interfaces:` key (Q3 B); an export for model traffic (Q8 B); re-exporting a nested
document's ingress (Q6 B); the default ingresses (`epic:control-plane`).
