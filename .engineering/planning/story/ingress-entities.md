---
format: aep.planning-md/3
id: story:ingress-entities
kind: story
status: draft
title: Ingress and Export in the composition block, and Contract.ingress
summary: agentic.architecture gets Ingress, Export, AddIngress, AddExport and the view Ingresses; agentic.shared IngressId and ExportId; Contract gets ingress inside its digest.
relations:
- decomposes: epic:ingress
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: inferred
  path: blocks/scenarios/architecture
- confidence: inferred
  path: docs/blocks/9-composition/architecture.md
- confidence: inferred
  path: website/src/generated
revision: 3
---
## Outcome

The composition block records ingresses. `agentic.architecture` gets `Ingress` (its rules as
written, as `PatchRule` keeps its text) and `Export`, the commands `AddIngress` and `AddExport`
with their events and refusals, and the view `Ingresses`; `agentic.shared` gets `IngressId` and
`ExportId`; `Contract` gets `ingress: List<ContractIngress>` (the ingress, its resolved operations,
its export protocols), inside `Contract.digest`. Design: `design/ingress.md` I8. Epic:
`epic:ingress`.

## Scope

- `blocks/9-composition/architecture.yaml`: `Ingress`, `Export`, `ContractIngress`,
  `Contract.ingress`, `AddIngress`, `AddExport`, `Ingresses`.
- `blocks/0-foundation/shared.yaml`: `IngressId`, `ExportId`.
- `blocks/scenarios/architecture/`: authored scenarios for both commands and the view.
- Generated: `docs/blocks/9-composition/architecture.md`, `website/src/generated/`.
- `CHANGELOG.md`.
- Time synthesis before and after; builds and tests through `slot`; no test over 60 s, the test
  step within 120 s, a pool job over 10 minutes is killed.

## Acceptance

- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, it lists `Ingress`, `Export`, `AddIngress`, `AddExport`, the view `Ingresses` and the field
  `Contract.ingress`; `task check:layers` passes.
- Given an authored scenario that adds an export to an ingress that does not exist, when
  `task check:blocks` runs, the command is refused by name.
- Given an authored scenario that adds an ingress with two exports, when `task check:blocks` runs,
  `Ingresses` returns one row with both protocols.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Filling `Contract.ingress` from a document (`story:interface-derivation`); records of calls
(`story:ingress-records`).
