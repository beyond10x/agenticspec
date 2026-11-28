---
format: aep.planning-md/3
id: story:ingress-admission
kind: story
status: draft
title: Exports as capabilities in admission, and the control plane's floor
summary: admit() binds each export as export.<protocol>, lists every unserved export with the missing ports, refuses a missing authenticator, a non-computed scraped KPI and an unserved principal port; the default operator floor.
relations:
- decomposes: epic:ingress
- depends_on: story:interface-derivation
- depends_on: story:first-runtime
- depends_on: story:control-plane-default
- informed_by: epic:control-plane
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

Admission serves ingresses. In `admit(contract, binding?, environment)` each export is a
capability named `export.<protocol>`, beside the ports; the plan names per export its server and
address; a refusal lists every export the environment cannot serve together with the missing ports,
never only the first. Admission also refuses an authenticated export whose audience has no
authenticator in the binding, a `metrics_scrape` export of a KPI whose status is neither `computed`
nor `both`, and a principal port served by neither an export nor a fixture (a scripted answer is a
fixture). `agenticspec-binding/1` says per export the address, server, authenticator and, for
`http`, the ESS server binding that `refines:` the generated protocol binding. Design:
`design/ingress.md` I6, I8, I9 (admission). Epic: `epic:ingress`.

## Scope

- The admission and binding modules of `story:first-runtime` (inferred: the runtime crate,
  `docs/binding.md`): export capabilities, the three refusals, the binding's export section.
- Tests on the workstation example, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given the workstation example and an environment with an HTTP server, an event-stream server,
  the UI and authenticators for `owner` and `admin` only, when admission runs, it refuses with
  exactly: `operator: export.tool has no server`; `metrics: export.metrics_scrape has no server`;
  `metrics: rate_limited_seconds, queue_wait_p90_seconds, pool_utilization are not computed from
  observed events`.
- Given the simulator's environment, when admission runs on the same document, every export is
  admitted.
- Given a binding that names no authenticator for an authenticated export's audience, when
  admission runs, it refuses naming the ingress and the audience.
- Given a test binding with a scripted answer for `principal.answer` and no ingress, when
  admission runs, the principal port is admitted on the fixture.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Control plane additions

From `design/control-plane.md` C5 and P4 A (`epic:control-plane`), on
`story:control-plane-default`:

- The default `operator` ingress over `http`, `event_stream` and `ui`, with an authenticator for
  `control.operator`, is the admission floor; a local runtime may bind a loopback authenticator.
- The default's `tool` and `metrics_scrape` exports are served when the environment has a server
  and listed as `unserved` in the plan otherwise, not refused.
- A headless test binding says `control_plane: unserved`, so the absence is explicit; fixtures
  answer principal ports there. Authored ingresses keep every refusal above.

Acceptance:

- Given a document with no `ingress:` key and an environment without a tool server, when admission
  runs, the plan lists `operator: export.tool unserved` and admits.
- Given a binding with no authenticator for `control.operator` and no `control_plane: unserved`,
  when admission runs, it refuses naming `control.operator`.
- Given a test binding with `control_plane: unserved`, when admission runs, it admits with no
  default export served.

## Not in this story

Packing admission (`story:packing-admission`); conformance through an export
(`story:ingress-conformance`).
