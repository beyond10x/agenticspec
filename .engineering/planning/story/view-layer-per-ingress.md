---
format: aep.planning-md/3
id: story:view-layer-per-ingress
kind: story
status: draft
title: One operator UI per ingress with a ui export
summary: agenticspec ui derive writes one composition per ingress with a ui export, actions from its interface; R1-R5 under {root}/ingress/{ingress}/scopes/..., R5 answered per ingress.
relations:
- decomposes: epic:ingress
- depends_on: story:interface-derivation
- depends_on: story:architecture-ui
- depends_on: story:runtime-ui-interface
- depends_on: story:action-sets
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticspec-ui
- confidence: inferred
  path: crates/agenticspec/src/serve.rs
- confidence: inferred
  path: scripts/check-generated.sh
- confidence: inferred
  path: ui/README.md
- confidence: inferred
  path: ui/views/examples
revision: 2
---
## Outcome

The operator UI of the view layer is the `ui` export of an ingress. `agenticspec ui derive`
derives one composition per ingress with a `ui` export: its pages show the panels of the
interface's views, its actions are the interface's commands (D6 becomes the `principal` rule of
I2), its channels are the interface's feeds. R1-R5 are the `http` and `event_stream` exports of the
same ingress, with the ingress name in the base path
`{root}/ingress/{ingress}/scopes/{architecture_id}[/{part path}]`; R5 answers per ingress (the
interface's commands, each with the evaluator's verdict). Two audiences of one runtime get two
surfaces from one adapter. Design: `design/ingress.md` I10, I5; `design/view-layer.md` S4, S6.
Epic: `epic:ingress`.

## Scope

- `crates/agenticspec-ui`: derivation per ingress, the base path with the ingress name, R5 per
  ingress.
- `crates/agenticspec/src/serve.rs`, `ui/README.md`: the path.
- `ui/views/examples/`: the workstation compositions for `user` and `operator`.
- `scripts/check-generated.sh`, tests, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given the workstation example, when `agenticspec ui derive` runs, it writes two compositions,
  `user` and `operator`, and `ess ui check` exits 0 on each.
- Given the `user` composition, its actions are exactly `SendMessage` on the border part and
  `Answer` on open questions; the `operator` composition offers `DrainGateway`, `StopGateway` and
  `DetachUpstream` and no `OpenGateway`.
- Given a fixture server, when the `operator` UI reads a view, the request path starts with
  `/ingress/operator/scopes/`.
- Given R5 for a caller of `user`, when it is answered, it lists only the commands of `user`'s
  interface.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

The default control plane's pages (`story:control-plane-pages`); per-scope grants
(`story:action-sets`).
