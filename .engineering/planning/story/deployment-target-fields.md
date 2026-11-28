---
format: aep.planning-md/3
id: story:deployment-target-fields
kind: story
status: implemented
title: 'Bundle fields a target''s translator needs: secrets, egress, ports, mounts, images, reach, network policy'
relations:
- decomposes: epic:architecture-deployment
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:24Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:24Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:24Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A bundle carries declared secrets and egress, served ports and bind per ingress export, check ports, volume mounts, local or registry images, external parts with reach, egress names and addresses, and network isolation, each with its named refusal; deploy check reads a bundle back.

## Delivered

2311851, 30d708d, in PR #42 or #43 (main 9bb5998).

## Acceptance

- The tests named in the commit messages of 2311851, 30d708d pass in `slot task check`.
