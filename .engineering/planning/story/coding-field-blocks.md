---
format: aep.planning-md/3
id: story:coding-field-blocks
kind: story
status: implemented
title: 'Coding field blocks: repository, forge, change request, issue, board, work queue, file tools'
summary: Vendor-neutral coding concepts as field blocks that depend on the core only
relations:
- decomposes: epic:fields
scope:
- confidence: cited
  path: docs/fields
- confidence: cited
  path: fields/coding/architectures
- confidence: cited
  path: fields/coding/blocks
- confidence: cited
  path: scripts/check-layers.sh
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:40:30Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-27T23:40:30Z", actor: "human:operator", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:12Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Coding agents can be modelled with agenticspec without naming a third party: the coding field has
its own blocks (`design/fields.md` § "Two layers", its own blocks): repository, forge, change
request, issue, board and work queue, and a toolset of conceptual file and command tools typed by
inputs and effects. They depend on core blocks only; a concrete forge or harness is a binding.

## Scope

- `fields/coding/blocks/`: the ESS domains, validated and with a conformance run like `blocks/`.
- `scripts/check-layers.sh`: field blocks may depend on core, core never on a field.
- `fields/coding/architectures/`: one example (an auto-merger: watches change requests, merges
  on green checks under an approval) that `agenticspec sim check` accepts.
- Docs pages for the field blocks through `task docs:generate`.

## Acceptance

- `ess specify validate` over the coding field blocks exits 0 and their conformance run passes.
- A core block that references a coding block makes `task check:layers` fail; the reverse passes.
- `grep -riE "github|gitlab|bitbucket|gitea|claude" fields/coding/blocks` prints nothing.
- The auto-merger example passes `agenticspec sim check` and its docs page shows the field blocks.

## Not in this story

Moving the profile and KPIs (`story:field-packs`); bindings to a concrete forge.
