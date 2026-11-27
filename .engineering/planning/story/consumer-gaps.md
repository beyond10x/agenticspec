---
format: aep.planning-md/3
id: story:consumer-gaps
kind: story
status: implemented
title: Give the 15 concepts of a consumer's multi-session setup a home in the blocks
relations:
- decomposes: epic:building-blocks
scope:
- confidence: cited
  path: blocks
- confidence: cited
  path: design/consumer-gaps.md
- confidence: cited
  path: kpis/catalogue.yaml
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T18:10:25Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T18:10:25Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-27T20:12:30Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The 15 concepts a consumer's multi-session agent setup uses, and that agenticspec 0.3.0
(`spec/domains/session.yaml`) could not express, each have a typed home in `blocks/`, at the layer
the rule allows.

## Source

A gap report from a consumer session (2026-11-27), read against 0.3.0: G1 to G15, 14 verified and
1 inferred, each with the lines tried and the smallest addition. The consumer offered its
validated extension domain as a patch.

## Mapping onto the blocks

- G1 delegation of authority, a proof of a human order: `principal`, `delegation` (scope,
  exclusions, proof kind); `on_behalf_of` on what a delegate submits and decides.
- G2 standing rules as policy: `authority` Grant (grantor, granted_at, effect, applies_to,
  condition, excludes, supersedes); a decision names the grant it rests on.
- G3 spend caps and the start cost of a model: `authority` Budget (scope, window, cap, booked);
  the wake of a self-hosted route is a booking against a budget, outcome booked, joined,
  cap_reached or ceiling_refused (`runtime` or `model`).
- G4 a sub-agent's hand-back: `workspace` Proposal or `work` Deliverable with worktree, branch,
  base, head, merge request, merged under a grant.
- G5 protocol runs: `process` revision; `work` Run with intake digest, parent run, route, budget,
  outcome, and its steps (kind program, agent or person action).
- G6 model routes as records: `model` Route (gateway, model, kind, phase, prices, start wait).
- G7 a decision with options: `question` with lettered options, a recommended option, a holder,
  a default on silence, who answered for whom, the digest of the text a pick binds.
- G8 a message between sessions names its sending session and tool call: `channel` Message.
- G9 relayed chat: `channel` Subscription with relays_for and a heartbeat.
- G10 a work queue in click order with cancel: `work` or `resource` queue.
- G11 the requested model of a sub-agent and the rule that chose it: `harness` sub-agent spawn.
- G12 a session's name and role: `harness` HarnessSession.
- G13 the audience of an outbound post: `channel` Post (audience, post_as, picked in a question).
- G14 a human act outside a session (a button on a page): `principal` or `channel`.
- G15 trigger rules: the `trigger` block (`story:triggers-schedules-swarms`).

## Acceptance

- Each gap names its block, entity and field in `design/consumer-gaps.md`, or says why it is left
  out.
- `task check:blocks` passes with the additions: validate, the layer check, the KPI check and the
  scenarios.
- One scenario per new entity lifecycle.
