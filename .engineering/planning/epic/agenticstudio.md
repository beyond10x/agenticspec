---
format: aep.planning-md/3
id: epic:agenticstudio
kind: epic
status: draft
title: 'agenticstudio: an architecture builder with an embedded agent channel'
summary: A multi-crate app in this repository where a user and an agent design an architecture of building blocks, check it, simulate and hill-climb it, visualize the state space, and keep drafts.
tags:
- idea
relations:
- depends_on: epic:simulator-and-search
- depends_on: epic:docs-site
revision: 2
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
  interactive), with the layer check and the structural check live.
- Evaluation: run an experiment from the editor; show the KPIs per part, baseline against best.
- Hill climbing and state space visualizations: the search trace, the variants, the space of the
  marked places, and the states of the specification an experiment reached.
- Optimizations: accept a variant into the draft.
- Persistence of drafts: named drafts with revisions, compared side by side.

## Operations: select, then transform

One operation set for the user, the studio agent and the optimizer.

- An architecture is facts for the query language of `queries/*.sq` (`docs/queries.md`):
  `part{id, kind, role, count}`, `link{id, from, to, kind}`, `policy{...}`, `marker{...}`, and the
  KPIs of the last run per part, `kpi{part, id, value}`. The same language then reads transcripts,
  observed runs and architectures.
- A selector is a rule body: `part{id: P, role: "worker"}, kpi{part: P, id:
  "false_completion_claim_share", value: V}, V > 0.5`.
- A transform is a rule head from a small closed set of verbs:
  - `set` a field;
  - `add` / `remove` a part or link;
  - `connect` two selections;
  - `replicate` (count);
  - `wrap` a link or part with a guard part (timeout, retry, budget cap, approval gate,
    reviewer, verifier), and `prepend` / `append` on a link;
  - `mark` / `unmark` a place for optimization with its `vary`.
- Example: `wrap(L, timeout{after: 10m}) :- link{id: L, kind: "model_call", from: A},
  part{id: A, role: "worker"}.`
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
- The `serve` command's server and the UI in `ui/app` (Vue), or a new app beside it.

## Before decomposition

The epic introduces nouns no ESS document declares yet: a studio session, a draft with revisions,
a tool call of the studio agent on a draft. Model them first (an ESS domain of the studio, or the
`architecture` block's revisions extended with drafts), then decompose.

## Status

An idea recorded for later. Not scheduled; the building blocks, the simulator and the docs site
come first.

