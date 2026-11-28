# KPI catalogue

`catalogue.yaml` (format `agenticspec-kpis/1`) lists every KPI of every building block: what it
measures, how it is computed, its direction under each work mode, whether it is a gate, and
where it is computed today. The docs site renders it on each block's page.

It holds the KPIs of the core only. A pack keeps its KPIs in `packs/<pack>/kpis.yaml` and a field
in `fields/<field>/kpis.yaml`, in the same format: a field's KPI names the core or pack block whose
events it reads (`block: workspace`), and the core catalogue never names a field. No KPI here may
use a word a field lists in `fields/<field>/words.txt` (`task check:neutral`).

```yaml
format: agenticspec-kpis/1
kpis:
  - id: owner_wait_share            # snake_case, unique
    block: principal                # a block of blocks/system.yaml, without the `agentic.` prefix
    kind: quality                   # correctness | completeness | quality
    unit: share                     # share | count | seconds | usd | tokens | ratio | points
    summary: Share of agent time spent waiting on the human owner.
    how: Seconds agents wait on an owner answer, divided by agent wall seconds.
    direction: lower                # higher | lower
    modes: { autonomous: lower, pairing: ignore, review: ignore }   # optional overrides
    gate: false                     # true: must be 0 (or 1 for a share that must be complete) for a variant to count
    reads: [agentic.question.QuestionAsked, agentic.question.QuestionAnswered]
    status: computed                # computed (agenticspec kpi today) | simulated (agenticspec sim) | both | planned
    source: research/2611.07166     # a paper id, docs/kpis.md, or design
```
