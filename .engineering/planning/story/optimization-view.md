---
format: aep.planning-md/3
id: story:optimization-view
kind: story
status: implemented
title: Explain the optimization on the experiment result page
relations:
- decomposes: epic:simulator-and-search
scope:
- confidence: cited
  path: crates/agenticspec-render
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T18:53:20Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T18:53:20Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-27T20:12:30Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The result page of an experiment (`crates/agenticspec-render`) explains the optimization, not only
its outcome: what the search optimized, where the fitness came from, what the space looks like and
which trade-offs it made.

## Observed on the shared-board result (2026-11-27)

- The KPI table is 70 rows in name order; gate KPIs at 0 and the weighted KPIs look alike; the
  delta column is cut off at 1440 px.
- The variants table shows the first three KPIs by name (`acts_for_without_part`, ...) instead of
  the weighted ones.
- Token counts print as `1,716,652,500`.
- The search trace is the only view of the search.

## Scope

- Objective first: the fitness formula, each weighted KPI with baseline, best and its contribution
  to the fitness change; gates folded into one line (`18 of 18 gate KPIs at 0`) that opens.
- The landscape: a heatmap of fitness over two marked places (the rest held at best), a line per
  place for one; infeasible cells marked.
- Trade-offs: a scatter of every variant on two chosen KPIs (cost against correct tasks by
  default) with the Pareto front.
- Sensitivity: per marked place, fitness across its values with the others at best.
- The variants table shows the weighted KPIs; large numbers in compact form.
- Under the ui-iteration-loop of the site (`story:site-ui-loop`): the pages are in the sweep.

## Acceptance

- The six experiments' pages show the five views; `cargo test -p agenticspec-render` covers each.
- No column cut off at 1280 and 1440 px; axe 0 serious in both themes.
