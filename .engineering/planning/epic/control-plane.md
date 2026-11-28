---
format: aep.planning-md/3
id: epic:control-plane
kind: epic
status: active
title: 'The default control plane: what every architecture brings for its operator'
summary: 'Default operator and metrics ingresses derived from every architecture, printable as ingress: rules, with Inbox, Activity, Operations, Metrics, Architecture and Events pages and a day-one fixture server (C1-C8).'
relations:
- depends_on: epic:ingress
- informed_by: epic:view-layer
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:23Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:23Z", actor: "human:operator", revision: 3}
---
## Outcome

Every admitted architecture brings a default control plane for its operator, with nothing
authored: two ingresses, `operator` and `metrics`, derived from the architecture, printable as
`ingress:` rules a consumer can paste, copy and change (`agenticspec ingress default`), and
replaceable piece by piece. Its pages (Overview, Parts, Inbox, Activity, Operations, Metrics,
Architecture, Events) are selected by port class and the configuration and administration split,
never by view name. On day one, before any runtime, `agenticspec ui serve` answers the default from
block fixtures or a simulator run.

Design: `design/control-plane.md` (decisions C1-C8, open points P1-P9), on `design/ingress.md` and
`design/view-layer.md`. Approved as written, every open point at its recommended option.

## Decisions taken

- C1: a derived default, materialized on request as `ingress:` rules; pasting it gives the same
  contract byte for byte; the pin is the generated `ess-ui/1` composition.
- C2: defaults by name (`operator`, `metrics` unless declared or `none`); implied audiences
  `control.operator` (human) and `control.monitor` (service) under the reserved path `control`, in
  the contract only; rule values `principal: all`, `plays: administration`, `metrics: computed`.
- C3: the page table; never exposed: configuration commands, commands of an actor that issues a
  port's report, machine traffic.
- C4: the steps from the default to a consumer's own.
- C5: one runtime and adapter; admission floor `http`, `event_stream`, `ui` with an authenticator
  for `control.operator`; `tool` and `metrics_scrape` listed `unserved` when absent;
  `control_plane: unserved` in a headless test binding; `agenticspec ui serve` on day one.
- C6: the studio and the control plane share one shell component in the views package, two apps.
- C7: the derivation names no block, view, command or KPI.
- C8: simulator results stay byte for byte; architecture pages list the default's resolved
  operations; `workstation` and `team-with-console` are golden examples under
  `ui/views/examples/`.
- P1 A, P2 A, P3 A, P4 A, P5 A (`principal: all`, the grant decides), P6 A (`computed` and `both`
  only, predictions beside them), P7 A (block fixtures or a simulator run), P8 A (one shell, two
  apps), P9 A (a nested console's surface is not served when nested).

## Stories, in order

1. `story:control-plane-default`: defaults by name, `none`, the reserved path and implied
   audiences, the three rule values, `agenticspec ingress default`.
2. `story:control-plane-pages`: the pages across parts, overview tiles and golden examples.
3. `story:ui-serve-fixtures`: `agenticspec ui serve <architecture>` on block fixtures.
4. `story:sim-event-log`: the simulator's run as block events, replayed by `ui serve --sim`.

Extensions of other stories, no new story:

- `story:ingress-admission` (`epic:ingress`): the floor, `unserved` exports,
  `control_plane: unserved`, a loopback authenticator for a local runtime.
- `story:views-package` (`epic:view-layer`): the shared shell component; the studio's draft view
  previews the default control plane.
- `story:contract-docs` (`epic:architecture-contract`): each architecture page lists the default's
  resolved operations.

Every story that runs tests keeps each test within 60 s and its test step within 120 s. A job in
the build pool ends within 10 minutes.

## Not in this epic

A control-plane UI built per catalog entry (C8); one app for the studio and the control plane (P8
B); re-exporting a nested console's surface (P9 B); the units of a deployment on the Operations
page (`story:control-plane-units`, `epic:architecture-deployment`).
