---
format: aep.planning-md/3
id: story:studio-deploy
kind: story
status: draft
title: 'Studio deploy tool: dry run, confirmation, promotion'
summary: A deploy tool issues RequestDeployment over packing default and deploy bundle; its dry run answers admission, a real deploy asks for confirmation; the studio holds the deployment by id only.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-release
- depends_on: story:deployment-bundle
- depends_on: story:deployment-domain
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticstudio
- confidence: inferred
  path: docs/studio.md
- confidence: inferred
  path: studio/domains/session.yaml
- confidence: inferred
  path: studio/domains/shared.yaml
revision: 2
---
## Outcome

The studio deploys a released revision. A new `deploy` tool issues
`agentic.deployment.RequestDeployment` over `agenticspec packing default` and
`agenticspec deploy bundle`; its dry run answers admission (and coverage once cloudspec's
translator is reachable); a real deploy asks for its own confirmation, since deploying to a target
is not reversible by the studio. Promotion is a `deploy` naming the deployment promoted from. The
studio holds the deployment by identifier only (`DeploymentRef`); the records stay in
`agentic.deployment` (LQ6, recommended option A). `studio.session.Tool` gains `Deploy`. Design: `design/lifecycle.md`
L1, L4, L6; over `design/deployment.md` stories 1-4. Epic: `epic:agenticstudio`.

## Scope

- `studio/domains/session.yaml` (`Tool` `Deploy`), `studio/domains/shared.yaml`
  (`DeploymentRef`).
- `crates/agenticstudio`: the `deploy` tool, dry run and confirmation.
- `docs/studio.md`, tests, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given an unreleased revision, when `deploy` is called for a remote target, it is refused as
  `NotReleased`.
- Given a released revision and a binding that fails admission, when `deploy` runs as a dry run, it
  returns every finding and records no deployment.
- Given a released revision and an admitted binding, when `deploy` runs without confirmation, it
  writes nothing and asks for confirmation; with it, a deployment to the in-process runtime is
  requested and its id is held by the release.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Translation and realization on a target (cloudspec); the control plane of a running deployment
inside the studio.
