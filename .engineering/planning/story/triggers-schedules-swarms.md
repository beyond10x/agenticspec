---
format: aep.planning-md/3
id: story:triggers-schedules-swarms
kind: story
status: active
title: Triggers, schedules, multi-party channels and swarms
summary: A trigger block wakes agents on messages, time or events; channels get topics, queues and boards; architectures get swarms.
relations:
- decomposes: epic:building-blocks
scope:
- confidence: cited
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/7-collaboration/channel.yaml
- confidence: cited
  path: blocks/7-collaboration/trigger.yaml
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: cited
  path: blocks/ess-inputs.yaml
- confidence: cited
  path: blocks/scenarios/architecture
- confidence: cited
  path: blocks/system.yaml
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T15:14:28Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T15:14:28Z", actor: "human:operator", revision: 3}
---
## Outcome

Agents are woken by what they read or by time, and many agents work as one swarm.

## Scope

- `channel`: many parties; kinds Topic, Queue and Board (typed entries); subscriptions; a message is a Request or an Inform.
- New block `trigger` (layer 7): OnMessage, OnSchedule and OnEvent triggers with a concurrency policy, schedules, firings.
- `architecture`: a part's coordination (Orchestrator, Board, Mesh, None) and start stagger; Trigger and Schedule parts; a Trigger link.

## Source

`design/research-findings.md`: wake rules 21 of 50 to 4 of 32 total coordination failures (2611.00689); scheduled agent loops 2.2 to about 35 merged pull requests per day (2610.36254); a team without an orchestrator on a typed board, staggered start (2609.26781).

## Acceptance

- `task check:blocks` passes with the new block; `trigger` depends on `channel`, never the reverse.
- A scenario commits a schedule that triggers a worker swarm coordinated by a board; a trigger link to a part that is not an agent is refused with a named error.
- The simulator runs a scheduled or triggered architecture (a later story).
