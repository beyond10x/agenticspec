---
format: aep.planning-md/3
id: story:ingress-conformance
kind: story
status: draft
title: 'Ingress conformance: the edge''s refusals and the block suite through http'
summary: NotInInterface, wrong-audience and unauthenticated calls refused, the block suite run through the http export restricted to the interface, and feed resume reusing the bus delivery suite.
relations:
- decomposes: epic:ingress
- depends_on: story:ingress-admission
- depends_on: story:ingress-projections
- depends_on: story:bus-delivery-conformance
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-ui
revision: 2
---
## Outcome

An ingress has conformance beside block and scenario conformance: the block suite runs through the
`http` export (ESS stage 3, an HTTP conformance target through the binding), restricted to the
interface's operations; an operation outside the interface is refused as `NotInInterface`, a caller
of another audience is refused, and an unauthenticated call to an authenticated export is refused;
a feed resumed from cursor c yields exactly the events after c, reusing
`story:bus-delivery-conformance`. Design: `design/ingress.md` I5, I9 (conformance). Epic:
`epic:ingress`.

## Scope

- The ingress conformance cases and their runner against a fixture target (inferred: beside the
  suites of `story:runtime-ui-interface` in `crates/agenticspec-ui`).
- Tests, `docs/` section on ingress conformance, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given the workstation example on the fixture target, when a caller of `user` issues
  `DrainGateway`, the call is refused as `NotInInterface` before any block sees it, and the case
  passes; a target that executes it makes the case fail.
- Given a caller authenticated as `admin`, when it calls ingress `user`, the call is refused.
- Given ingress `operator` and a call with no credentials, when it reaches the `http` export, it is
  refused.
- Given the block suite restricted to ingress `operator`'s operations, when it runs through the
  `http` export, every selected scenario executes and no scenario outside the interface runs.
- Given the feed of ingress `user` resumed from cursor c, when the case runs, it receives exactly
  the events after c.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Running the suite against a real runtime (`story:first-runtime`); call records
(`story:ingress-records`).
