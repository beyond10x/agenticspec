---
format: aep.planning-md/3
id: story:architecture-catalog
kind: story
status: draft
title: A catalog of pre-built optimized architectures on the docs site, to download and run
relations:
- decomposes: epic:docs-site
- depends_on: story:optimization-view
- depends_on: task:site-s01-change-set
revision: 1
---
## Outcome

The docs site has a catalog of pre-built, optimized architectures: one entry per architecture with
its full experiment result page (graph, objective, landscape, trade-offs, sensitivity, variants,
timeline) and downloads to run it, later to deploy it.

## Source

The human, 2026-11-27, about the experiment result pages: "put this onto the docs as our catalog
of pre-built optimized architectures. The idea is to download it later and run or deploy it."

## Scope

- `agenticspec-sim` writes the best variant of an experiment as an architecture document
  (`best.yaml`, every marked place set to the best value, markers kept as comments-free `vary`
  ranges so it can be searched again).
- The docs generator (`crates/agenticspec-docs`) writes `docs/catalog/` (an index with one card
  per entry: what it is for, the headline gain of the search, the trade-off, the stage of the
  agentic ladder it belongs to) and one page per entry embedding the full result page from
  `website/static/catalog/<name>/`.
- Downloads per entry: the architecture as written, the optimized architecture, the experiment,
  the behaviour profile, `result.json`, and a one-line command to run it
  (`agenticspec-sim run experiments/<name>.yaml --out out/`).
- Deploy: a stated placeholder (the agent runtime and compute implement the blocks later); no
  deploy button that does nothing.
- `task check:docs` holds the generated catalog byte for byte.

## Acceptance

- Every experiment under `experiments/` has a catalog entry; every download link resolves on the
  built site.
- A downloaded optimized architecture passes `agenticspec-sim check` and reproduces the best
  fitness when its experiment is run with the same seeds.
- The catalog pages are in the site sweep of `task check:site`.
