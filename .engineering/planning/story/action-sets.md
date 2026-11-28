---
format: aep.planning-md/3
id: story:action-sets
kind: story
status: draft
title: 'Action sets: actions visible by the viewer''s grants at a scope'
summary: R5 answered by the authority evaluator returns the commands the signed-in principal may issue at a scope; actor.may reads it per scope; R4 still refuses (M47).
relations:
- decomposes: epic:view-layer
- depends_on: story:runtime-ui-interface
- depends_on: story:governance-types
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: blocks/6-governance/authority.yaml
- confidence: inferred
  path: crates/agenticspec-ui
- confidence: inferred
  path: ui/views/fixtures
revision: 2
---
## Outcome

A viewer sees only the actions their grants allow at the scope they look at. R5,
`GET {source}/api/actions`, returns `[{command, allowed, grant_id?}]` for the signed-in principal at
that scope, answered by the authority evaluator (O5 A), not an ESS view. The generated `visible:
actor.may(<command>)` reads it per scope. Hiding is a convenience, not authorization: the runtime
still refuses the call through R4. Today `Grant.action_class` is a `ToolCategory` (`Read` …
`Other`), the scope is text that the evaluator matches, and nothing maps a block command to an
action class. Consumer gap M47. Design: `design/view-layer.md` D6, D7 (R5), O5 and the verdict on
M47. Epic: `epic:view-layer`.

Precondition: the core `ActionClass` catalog (`design/fields.md`), which has no story yet; this
story starts once it exists, beside `story:governance-types` (a decision names the grant it relied
on).

## Scope

- R5 on the wire: `GET {source}/api/actions` at a scope, `[{command, allowed, grant_id?}]`; the
  mapping from block command to action class through the `ActionClass` catalog.
- `blocks/6-governance/authority.yaml`: `Grant.action_class` typed by the `ActionClass` catalog,
  if `story:governance-types` and the catalog leave it a `ToolCategory`.
- The fixture source (inferred: `ui/views/fixtures/`): R5 answers per scope from fixture grants.
- The conformance suite of `story:runtime-ui-interface` (inferred: `crates/agenticspec-ui`): an R5
  check per scope, and a check that a hidden action is still refused by R4.
- `crates/agenticspec-ui` (inferred): `actor.may` per scope in the derived compositions.
- `CHANGELOG.md`.
- Tests: each ends within 60 s and the test step within 120 s; a pool job over 10 minutes is
  killed.

## Shared surfaces

- `blocks/6-governance/authority.yaml`: `story:governance-types` changes it first;
  `story:block-bindings` may change it for a binding's mapping.
- `ui/views/fixtures/`, `crates/agenticspec-ui`: after `story:runtime-ui-interface`.
- `CHANGELOG.md`: every story.

## Acceptance

- Given a principal with a grant for the action class of `Answer` at part `team.lead` and none at
  part `team.worker`, when R5 is read at each scope, `Answer` is `allowed: true` with the grant's
  id at `team.lead` and `allowed: false` at `team.worker`.
- Given those two scopes in the derived UI on fixtures, when each part page is opened, the
  `Answer` row action is shown on `team.lead` and hidden on `team.worker`.
- Given a hidden action posted directly through R4, when the source answers, it refuses with 422
  and the refusal's row.
- Given a source with no R5 (404), when a part page is opened, every action is shown and refusals
  come from R4.
- Given a source whose R5 allows a command the principal's grants do not cover at that scope, when
  the conformance suite runs, the R5 check fails and names the command and the scope.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The `ActionClass` catalog itself (`design/fields.md`); the authority evaluator inside a runtime;
an ESS view over `Grants` with client-side matching (O5 B, not taken).
