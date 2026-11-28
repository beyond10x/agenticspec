---
format: aep.planning-md/3
id: story:volume-access
kind: story
status: implemented
title: A volume claim's access follows who mounts it (R93)
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T12:11:10Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

read_write_once when every mounting part is in one unit and the claim is per instance or the unit has one replica; read_write_many only for two units or two instances; read_only_many for a shared read-only workspace.

## Delivered

78510fc, in batch 5 (integration/2026-11-28-c).

## Acceptance

- The tests named in the commit messages of 78510fc pass in `slot task check`.
