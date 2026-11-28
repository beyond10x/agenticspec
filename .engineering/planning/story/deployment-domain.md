---
format: aep.planning-md/3
id: story:deployment-domain
kind: story
status: draft
title: 'The agentic.deployment domain: deployments, units, states, refusals'
summary: Deployment with its digests, target and scope by value and nine states; Unit; RequestDeployment refused as NotReleased, NotAdmitted, CoverageRefused, TargetRefused, ScopeTaken; the Units view.
relations:
- decomposes: epic:architecture-deployment
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: blocks/9-composition/deployment.yaml
- confidence: inferred
  path: blocks/ess-inputs.yaml
- confidence: inferred
  path: blocks/scenarios/deployment
- confidence: inferred
  path: blocks/system.yaml
- confidence: inferred
  path: docs/blocks
- confidence: inferred
  path: website/src/generated
revision: 2
---
## Outcome

A new domain `agentic.deployment` (layer 9) records deployments. `Deployment`: revision; contract,
binding, packing and bundle digests; the translation digest once translated; target and scope by
value; the deployment it was promoted from; states `Requested`, `Refused`, `Bundled`, `Translated`,
`Realizing`, `Running`, `Degraded`, `Stopped`, `Failed`. `Unit`: name, kind, part instances,
replicas, its workload, its target object by value. `RequestDeployment` is refused as `NotReleased`
(except to the in-process runtime), `NotAdmitted` (with the list), `CoverageRefused`,
`TargetRefused` and `ScopeTaken`. A unit's health comes from `ReportRunning`, `ReportExited` and
`ReportFailed` of `workload.run`. A view `Units` gives desired and ready replicas, last failure and
parts. Design: `design/deployment.md` K10, K11. Epic: `epic:architecture-deployment`.

## Scope

- `blocks/9-composition/deployment.yaml` (new): entities, commands, refusals, events, views;
  `blocks/system.yaml`, `blocks/ess-inputs.yaml`.
- `blocks/scenarios/deployment/` (new): authored scenarios for each refusal and a promotion.
- Generated: `docs/blocks/`, `website/src/generated/`.
- `CHANGELOG.md`.
- Time synthesis before and after; builds and tests through `slot`; no test over 60 s, the test
  step within 120 s, a pool job over 10 minutes is killed.

## Acceptance

- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, it lists `agentic.deployment` with `Deployment`, `Unit`, the nine states and the view
  `Units`; `task check:layers` passes.
- Given an authored scenario that requests a deployment of an unreleased revision to a remote
  target, when `task check:blocks` runs, it is refused as `NotReleased`; to the in-process runtime
  it is accepted.
- Given an authored scenario that requests a second deployment into a taken scope, when
  `task check:blocks` runs, it is refused as `ScopeTaken`.
- Given an authored scenario where a unit reports running and then failed, when `Units` is read,
  the row shows the failure as its last failure.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The Operations page (`story:control-plane-units`); writing the bundle (`story:deployment-bundle`).
