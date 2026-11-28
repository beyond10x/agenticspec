---
format: aep.planning-md/3
id: epic:agenticstudio
kind: epic
status: active
title: 'agenticstudio: an architecture builder with an embedded agent channel'
summary: A multi-crate app in this repository where a user and an agent design an architecture of building blocks, check it, simulate and hill-climb it, visualize the state space, and keep drafts.
tags:
- idea
relations:
- depends_on: epic:simulator-and-search
- depends_on: epic:docs-site
- informed_by: epic:architecture-deployment
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T21:51:12Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-27T21:51:12Z", actor: "human:operator", revision: 4}
---
## Outcome

agenticstudio: an architecture builder for the agenticspec building blocks, a dedicated app inside
this repository built as several crates. A user and an agent design an architecture together in
one place, check it, simulate and optimize it, and keep drafts.

## What it holds

- An embedded agent channel: an agent works with the user on the architecture through tools that
  add and remove parts and links, set marked places, run the check, run an experiment, and explain
  a refusal or a KPI.
- The architecture graph as an editor (the renderer of `crates/agenticspec-render`, made
  interactive), with the structural check of `agenticspec-sim check` live. The layer check
  (`task check:layers`) holds the sources under `blocks/`; a draft changes an architecture and no
  block, so the layer check is not run on a draft (decided in round 2 of the plan review).
- Evaluation: run an experiment from the editor; show the KPIs per part, baseline against best.
- Hill climbing and state space visualizations: the search trace, the variants, the space of the
  marked places, and the states of the specification an experiment reached.
- Optimizations: accept a variant into the draft.
- Persistence of drafts: named drafts with revisions, compared side by side.

## Operations: select, then transform

One operation set for the user, the studio agent and the optimizer.

- An architecture is facts for the query language of `queries/*.sq` (`docs/queries.md`):
  `part{id, block, count}`, `setting{part, name, value}`, `link{index, from, to, carries, over}`,
  `policy{id, value}`, `marker{path, value}`, and the KPIs of the last run per part, `kpi{part,
  id, value}`. The fields are read from the format `agenticspec-architecture/1` and fixed once in
  `story:studio-first-slice` § "Facts of an architecture"; every selector of the stories uses
  those fields only. The same language then reads transcripts, observed runs and architectures.
- A selector is a rule body: `part{id: P, block: "agent"}, kpi{part: P, id:
  "false_completions_shipped", value: V}, V > 0`.
- A transform is a rule head from a small closed set of verbs:
  - `set` a field;
  - `add` / `remove` a part or link;
  - `connect` two selections;
  - `replicate` (count);
  - `wrap` a link or part with a guard part (timeout, retry, budget cap, approval gate,
    reviewer, verifier), and `prepend` / `append` on a link;
  - `mark` / `unmark` a place for optimization with its `vary`.
- Example: `wrap(L, timeout{after: 10m}) :- link{index: L, carries: "model_call", from: A},
  part{id: A, block: "agent"}.`
- A patch (`agenticspec-patch/1`) is an ordered list of such rules. It compiles to the commands of
  the `architecture` block (AddPart, AddLink, SetPer, SetSwarm, ...), applies atomically as one
  architecture revision, and is refused as a whole with the rule's named error.
- The agent's tools are four: `query(selector)`, `apply(patch, dry_run)`, `check`,
  `simulate(budget)`. A dry run returns the diff, the check result and the KPI change of a short
  simulation. Selectors keep ids out of the conversation, so a change is one call with a small
  answer.
- The optimizer's mutations are patches too. The search then explores structure (wrap the workers
  with a reviewer, add a verifier, swap a board for an orchestrator), not only the marked numbers.
- Undo, drafts and comparisons are revisions of patches.

Needed before it works: guards as concepts in the blocks (timeout, retry and limits are planned
in `epic:research-blocks` as enforced harness limits), and the architecture facts emitted by
`agenticspec-sim`.

## Builds on

- `blocks/` (the ESS system `agentic`), `architectures/`, `profiles/`, `experiments/`.
- `crates/agenticspec-sim` (check, simulate, search) and `crates/agenticspec-render` (the page).
- `crates/agenticspec-docs` (block pages, KPI catalogue).
- The pattern of the `serve` command (std HTTP, SSE channels, the generated route module).

## Shape

- Crates: `crates/agenticspec-patch` (the patch language and its applier, a library the
  studio, the CLI and the optimizer link), `crates/agenticstudio` (the `agenticstudio` binary:
  `serve`, the drafts, sessions, tool calls and evaluations), `crates/agenticstudio-mcp` (the
  agent channel as MCP tools).
- UI: `ui/studio`, a Vue 3 app beside `ui/app` (same toolchain), served by `agenticstudio serve`.

## Nouns (ESS)

Modelled before decomposition; each validates with `ess specify validate --strict-requires`.

- Building blocks, `blocks/9-composition/architecture.yaml` (system `agentic`, layer 9): a patch
  is `agentic.architecture.Patch` (ordered `PatchRule`s, verb from `PatchVerb`), its application
  `agentic.architecture.PatchApplication` (Requested, then Applied with the committed revision or
  Refused with the rule's position and error; a dry run commits none), with the commands
  `RecordPatch`, `ApplyPatch`, `RecordPatchApplied`, `RecordDryRunApplied`, `RecordPatchRefused`
  and the views `Patches`, `PatchApplications`. Identifiers `PatchId`, `PatchApplicationId` in
  `blocks/0-foundation/shared.yaml`. A draft's revisions are the architecture's
  `ArchitectureRevision`s.
- App-only, the ESS system `studio` in `studio/` (`studio/system.yaml`, `studio/ess-inputs.yaml`):
  - `studio.draft.Draft` (`studio/domains/draft.yaml`): a named handle on an architecture, opened
    from a document, Open or Archived.
  - `studio.session.Session` and `studio.session.ToolCall` (`studio/domains/session.yaml`): a
    user and an agent on one draft; each query, apply (dry run or not), check or simulate is a
    tool call by the user or the agent, Answered or Failed.
  - `studio.evaluation.Evaluation` (`studio/domains/evaluation.yaml`): one simulate or search run
    on a revision of a draft, with its budget, the experiment it made, its best fitness and the
    digest of its result.
- The studio names the `agentic` system's architecture, revision, patch application and
  experiment by identifier only (`studio/domains/shared.yaml`): ESS declares no relation across
  two systems. Open questions are the `UNMAPPED:` markers in those files.

## Status

Decomposed into draft stories (the stories that decompose this epic). The smallest slice comes
first: `story:studio-first-slice`.

Which story claims which promise:

| promise | story |
|---|---|
| the facts and their fields; open a draft, apply `set` and `add`, see the check | `story:studio-first-slice` |
| the other verbs, `policy` and `kpi` facts, structural mutations in the search | `story:architecture-patches` |
| guard parts | `story:guards-and-limits` (of `epic:research-blocks`) |
| sessions, tool calls, live channel, listing and reading revisions | `story:studio-live` |
| the graph as an editor: select, dry run, apply, step through revisions | `story:studio-editor` |
| run an experiment from the editor; KPIs per part, baseline against best; accept a variant; per-part KPIs as `kpi` facts; the KPI change of a dry run | `story:studio-evaluation` |
| the agent channel; tools that explain a refusal or a KPI | `story:studio-mcp` |
| named drafts with revisions, archive, undo, side-by-side comparison | `story:studio-drafts` |
| search trace, variants, landscape, reached states | `story:studio-state-space` |
| an agent channel embedded in the browser (later) | `story:studio-chat` |

The stories that share `crates/agenticstudio`, `ui/studio`, `docs/studio.md`, `studio` or
`crates/agenticspec-sim` are ordered by `depends_on` edges; each story's § "Shared surfaces"
names the others.

## Lifecycle stories

The operations on an architecture from design to operation (`design/lifecycle.md`, decisions
L1-L6, open points LQ1-LQ6, each with its recommended option): the studio records verifications
and releases, derives each revision's stage from evidence, follows its files and deploys through
`epic:architecture-deployment`. The stories below are drafts on the design's recommended options;
the open points are not recorded here as decided.

- `story:studio-verification`: `Verification` and the `test` tool; `NotChecked` on evaluations.
- `story:studio-release`: `Release`, `RevisionStages`, the `release` tool, write-back.
- `story:studio-file-follow`: a change on disk becomes a revision with caller `File`.
- `story:studio-deploy`: the `deploy` tool over the deployment epic's packing, admission, domain
  and bundle.
