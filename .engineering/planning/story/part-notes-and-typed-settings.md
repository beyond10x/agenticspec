---
format: aep.planning-md/3
id: story:part-notes-and-typed-settings
kind: story
status: implemented
title: Part notes and typed settings for spend, hosting and defaults (R97)
relations:
- decomposes: epic:simulator-and-search
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T15:11:34Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T15:11:34Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T15:11:34Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Every part takes an untyped notes map; channel kind screen, pool kind queue and order, model hosting and billing, spend caps per_day_usd, per_month_usd and route on authority and runtime, question default and default_after are declared and checked.

## Delivered

2037eb3, in batch 6 (integration/2026-11-28-d).

## Acceptance

- The tests named in the commit messages of 2037eb3 pass in `slot task check`.
