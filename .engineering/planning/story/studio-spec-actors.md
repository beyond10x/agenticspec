---
format: aep.planning-md/3
id: story:studio-spec-actors
kind: story
status: implemented
title: Server-issued studio commands have an actor; exposure aliases are shown
relations:
- decomposes: epic:agenticstudio
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:36Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:37Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:37Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

studio.session.Server may issue ReviseDraft, AnswerToolCall and FailToolCall; studio.evaluation leaves studio-server's owns; the Exposures view carries alias.

## Delivered

fb8d925, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of fb8d925 pass in `slot task check`.
