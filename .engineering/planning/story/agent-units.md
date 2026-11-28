---
format: aep.planning-md/3
id: story:agent-units
kind: story
status: implemented
title: 'Agent units: runtime pin, exact document bytes, shape and cron'
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:25Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:25Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:25Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

runtimes.agent-runtime pins the agent runtime; agent documents travel as their exact bytes; an agent unit carries shape service, job or scheduled, with a cron derived from every in exact forms only (schedule_not_cron).

## Delivered

9eac003, 393b004, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of 9eac003, 393b004 pass in `slot task check`.
