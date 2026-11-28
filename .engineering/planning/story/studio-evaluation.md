---
format: aep.planning-md/3
id: story:studio-evaluation
kind: story
status: draft
title: 'Evaluate a draft from the studio: simulate, search, accept a variant'
summary: simulate(budget) or the editor's run control runs the simulator on a draft revision, records an evaluation, shows KPIs per part, turns them into kpi facts, answers a dry run's KPI change, and accepts any variant as a patch.
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-live
- depends_on: story:studio-editor
- depends_on: story:architecture-patches
- depends_on: story:studio-mcp
scope:
- confidence: cited
  path: crates/agenticspec-sim
- confidence: cited
  path: crates/agenticstudio
- confidence: cited
  path: crates/agenticstudio-mcp
- confidence: cited
  path: docs/studio.md
- confidence: cited
  path: studio
- confidence: cited
  path: ui/studio
revision: 10
---
## Outcome

A draft is evaluated from the studio: `simulate(budget)`, or the run control in the editor, runs
the simulator on the draft's current revision, as a short simulation or a search over its marked
places, and records an evaluation (`studio/domains/evaluation.yaml`). Its result shows the KPIs
per part, baseline against best, and the user accepts any variant of the search into the draft as
a patch, not only the best. A finished evaluation's per-part KPIs become the draft's `kpi` facts,
and a dry run answers the KPI change of a short simulation. This story adds the `simulate` tool
to the agent channel of `story:studio-mcp` and the KPI change to its `apply` dry run (decision A,
taken on default 2026-11-28: the channel lands right after the editor).

## Scope

- `crates/agenticspec-sim`: `Experiment::from_docs` and `run` already take the experiment,
  architecture and profile as documents in memory (`crates/agenticspec-sim/src/search.rs`).
  This story adds what the studio needs beyond them: the short simulation of a dry run (one seed
  of the experiment, no search). Open point: whether that needs a new entry or `Experiment`'s
  existing ones suffice. The profile, load and seeds come from the experiment under
  `experiments/` that names the draft's source document.
- `studio`: settles the `UNMAPPED:` of `studio.evaluation.Evaluation`
  (`studio/domains/evaluation.yaml`): the studio takes profile, load and seeds from that
  experiment.
- `crates/agenticstudio`: `StartEvaluation`, `CompleteEvaluation`, `FailEvaluation`; the run on a
  worker thread, its progress on the draft's event channel; the `simulate` tool call answers with
  the evaluation, and `crates/agenticstudio-mcp` gains `simulate(budget)` as its fourth tool; the result page at `/evaluations/<id>` from `crates/agenticspec-render`.
- `kpi` facts: when an evaluation is `Done`, the draft's facts gain `kpi{part, id, value}` from
  its `result.json` as `story:studio-first-slice` § "Facts of an architecture" fixes them, built
  by the relation of `story:architecture-patches`; they replace those of the draft's previous
  finished evaluation. Open point: whether `kpi` facts of an evaluation stay after the draft moves
  to a later revision; taken: they stay until the next evaluation is `Done`.
- Accepting a variant: any `id` of the evaluation's `variants` in `result.json`, `best` or not.
  Its patch is one `set` rule per path of the variant's `params` whose value differs from the
  evaluated revision (a part's setting, or a policy through `set` on a `policy` of
  `story:architecture-patches`), applied as one revision. A draft that moved past the evaluated
  revision is refused as a stale base (`agentic.architecture.PatchApplication`).
- Dry run: an `apply` tool call with `dry_run: true` answers, beside the diff and the check
  result, the run KPIs (`kpis` of the run) of a short simulation of the current revision and of
  the patched document. Open point: the editor sends a dry run on every edit, and the cost of a
  simulation per edit is not measured; taken: one seed and the experiment's horizon.
- Explaining a KPI: each KPI a `simulate` or dry-run answer reports carries the `summary`, `unit`
  and `direction` of its entry in `kpis/catalogue.yaml`; a KPI with no entry is answered with its
  id and says it has none. The catalogue is read, not changed.
- `ui/studio`: the run control in the editor (simulate or search, with a budget), the result
  panel of an evaluation (KPIs per part, baseline against best; the variants), an accept button
  per variant, the KPI change in the dry run view.
- `docs/studio.md`: evaluating a draft, accepting a variant.
- Tests: a test ends within 60 s, the test step within 120 s. Open point: the search budget of
  the acceptance test, chosen so that both runs below fit in 60 s.

## Shared surfaces

- `crates/agenticstudio`, `crates/agenticstudio-mcp`: after `story:studio-first-slice`,
  `story:studio-live`, `story:studio-editor` and `story:studio-mcp` (depends on live, editor and
  mcp); before `story:studio-drafts` and `story:studio-chat`.
- `crates/agenticspec-sim`: after `story:guards-and-limits` and `story:architecture-patches`
  (depends on patches, which depends on guards); before `story:studio-state-space`, which
  depends on this story.
- `ui/studio`: after `story:studio-editor`; before `story:studio-drafts`,
  `story:studio-state-space` and `story:studio-chat`.
- `docs/studio.md`: after `story:studio-first-slice`, `story:studio-live` and `story:studio-mcp`;
  before `story:studio-drafts`.
- `studio`: after `story:studio-first-slice`; before `story:studio-drafts`, which settles the
  `UNMAPPED:` of `studio.draft.Draft`.

## Acceptance

- Given a draft of `architectures/workstation.yaml` at revision 1, when `simulate` runs a search
  with budget B and the seeds of `experiments/workstation-tuning.yaml`, the evaluation moves from
  `Running` to `Done` with a best fitness equal to the `best.fitness` of the `result.json` that
  `agenticspec-sim run` writes for that experiment with `search.budget` set to B.
- Given that `Done` evaluation, the query `kpi{part: "worker", id: "tasks_correct", value: V}`
  on the draft answers V equal to `best.per_part.worker.tasks_correct` of its `result.json`,
  where before the evaluation it answered no row.
- Given that `Done` evaluation on revision 1, accepting a variant that is not `best` and whose
  `params` differ from the baseline's moves the draft to revision 2, whose marked values equal
  that variant's `params`.
- Given a draft at revision 1, an `apply` tool call with `dry_run: true` and `set(P, count, 4)
  :- part{id: P, block: "agent", count: 12}.` answers each run KPI of the short simulation
  before and after the patch, and the draft stays at revision 1.
- Given the editor on a draft with no evaluation, when the user starts a search from the run
  control, an evaluation of the draft's current revision appears as `Running`.
- Given a `Done` evaluation, its result panel shows for each part the KPIs of `baseline.per_part`
  and `best.per_part` of its `result.json`.
- Given a draft at revision 1, when an MCP client calls `apply` with `dry_run: true`, the answer
  carries the diff, the check result and the KPI change, and the draft stays at revision 1.
- Given a draft of `architectures/workstation.yaml`, when an MCP client calls `simulate`, the
  answer carries for the KPI `tasks_correct` the `summary`, `unit` and `direction` written for
  it in `kpis/catalogue.yaml`.
