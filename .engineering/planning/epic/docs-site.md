---
format: aep.planning-md/3
id: epic:docs-site
kind: epic
status: active
title: 'Public docs: every block by layer with its KPIs, and architecture examples'
summary: Generated block pages, a KPI catalogue and up to 10 rendered architecture examples on the docs site.
relations:
- depends_on: epic:building-blocks
- depends_on: epic:simulator-and-search
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T15:13:46Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T15:13:46Z", actor: "human:operator", revision: 3}
---
## Outcome

The public docs site shows every building block by layer with its entities, commands, events,
views, dependencies and KPIs; a KPI catalogue; and up to 10 architecture examples with their
rendered graph and simulation result.

## Source

`kpis/README.md`, `kpis/catalogue.yaml`, `crates/agenticspec-docs`.

## Acceptance

- `agenticspec-docs --check` passes in the gate: the generated pages match the specification,
  the catalogue and the architectures byte for byte.
- The site builds (`task site`) and shows the layer map, one page per block, one per architecture
  and the KPI catalogue, readable in light and dark at 1280 px.
- 10 architecture examples under `architectures/`, each accepted by `agenticspec-sim check`.
