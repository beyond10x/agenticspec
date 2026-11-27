---
format: aep.planning-md/3
id: story:architecture-patches
kind: story
status: draft
title: Change architectures by selector and transform patches
relations:
- decomposes: epic:agenticstudio
- depends_on: story:guards-and-limits
- depends_on: story:studio-first-slice
- depends_on: story:studio-editor
scope:
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: cited
  path: blocks/scenarios/architecture
- confidence: cited
  path: crates/agenticspec-patch
- confidence: cited
  path: crates/agenticspec-sim
- confidence: cited
  path: crates/agenticspec/src
- confidence: cited
  path: crates/agenticstudio/src/generated
- confidence: inferred
  path: docs/blocks
- confidence: cited
  path: ui/studio/src/generated
- confidence: inferred
  path: website/src/generated
revision: 11
---
## Outcome

Architectures are changed by patches: a selector in the query language of `queries/*.sq` over
architecture facts, and a transform from a closed set of verbs. The studio agent, the user and the
optimizer use the same operations. Design: `epic:agenticstudio` § "Operations: select, then
transform". Nouns: `agentic.architecture.Patch` and `agentic.architecture.PatchApplication` in
`blocks/9-composition/architecture.yaml`.

## Builds on

`story:studio-first-slice` brings `crates/agenticspec-patch` with the format, the facts `part`,
`setting`, `link` and `marker`, the verbs `set` and `add`, atomic application and the refusal with
the rule and its error. Its § "Facts of an architecture" fixes the fields of every relation,
`policy` and `kpi` included; this story makes those two and uses no other field. Guard parts come
from `story:guards-and-limits`, in the document form it fixes (taken there: a part `{ block:
guard, kind: timeout, after: 10m }`).

## Scope

- `crates/agenticspec-patch`: the verbs `remove`, `connect`, `replicate`, `wrap`, `prepend`,
  `append`, `mark`, `unmark`; `set` on a policy; the facts `policy{id, value}` and `kpi{part, id,
  value}` (from `per_part` of `best` of a result document, `baseline` when no variant is
  feasible); the closed list of a patch's own errors (settles the `UNMAPPED:` of
  `PatchApplication`: an unparsable rule, an unknown verb, an unbound variable, a stale base).
- `wrap` of a link `A -> B` with a guard inserts a guard part G and replaces the link by `A -> G`
  and `G -> B`, both carrying what the link carried. Open point: the ids the applier mints for
  inserted parts. `wrap` with `reviewer` or `verifier` inserts an agent part with `role:
  reviewer` or `role: verifier`; open point: the links a reviewer wrap adds, since the check
  wants a `review` link to end at an agent with `role: reviewer` (`check.rs`).
- Open point, settled here: the sq type of a value that is not a whole number, which `kpi.value`
  needs (`story:studio-first-slice` § "Facts of an architecture").
- `blocks/9-composition/architecture.yaml`: removal of a part and a link (settles the
  `UNMAPPED:` of `PatchVerb`), and authored scenarios under `blocks/scenarios/architecture` that
  commit a revision through an applied patch, so the `applied` outcomes are witnessed (synthesis
  alone reaches no committed revision). The generated pages of the block (`docs/blocks/`,
  `website/src/generated/`) follow through `task generate`.
- `crates/agenticspec`: `agenticspec query` runs a selector over an architecture document, and
  over a result document for `kpi` (open point: the flag names).
- `crates/agenticspec-sim`: the search accepts patches as structural mutations beside the marked
  numbers. Open point: the experiment key that lists them and the variant field that names the
  mutations a variant applied; taken: `mutations` for both.
- Tests: a test ends within 60 s, the test step within 120 s; the search test runs a budget
  small enough for that.

## Shared surfaces

- `crates/agenticspec-patch`: after `story:studio-first-slice` (depends on it).
- `crates/agenticspec-sim`: after `story:guards-and-limits` (depends on it); before
  `story:studio-evaluation` and `story:studio-state-space`, which depend on this story directly
  or through evaluation.
- `blocks/9-composition/architecture.yaml`, `docs/blocks/`, `website/src/generated/`: after
  `story:guards-and-limits`, which adds the guard part kind to the same block.
- `crates/agenticstudio/src/generated/` and `ui/studio/src/generated/` (types generated from the
  `architecture` block): after `story:studio-first-slice` and `story:studio-editor`, which made
  them and which this story depends on; this story regenerates both in the change that alters
  the block.

## Acceptance

- Given `architectures/workstation.yaml`, applying `wrap(L, timeout{after: 10m}) :- link{index:
  L, carries: "model_call", from: A}, part{id: A, block: "agent"}.` adds exactly two parts `{
  block: guard, kind: timeout, after: 10m }`, replaces link 4 (`operator -> gateway`) and link 5
  (`worker -> gateway`) each by a link to its guard and a link from the guard to `gateway`, all
  carrying `model_call`, and changes nothing else.
- Given the document that wrap produced, `agenticspec-sim check` accepts it.
- Given a draft at revision n, a patch whose rule 1 is that wrap and whose rule 2 is
  `frobnicate(P) :- part{id: P}.` is refused with rule position 2 and this story's error for an
  unknown verb, and the draft stays at revision n with the same digest.
- Given a draft at revision n, the patch `remove(L) :- link{index: L, carries: "model_call",
  from: "worker"}.` is refused with no rule position and the check rule
  `agent_without_model_path` naming the part `worker`, and the draft stays at revision n.
- Given `architectures/workstation.yaml`, the patch `set(Q, value, 5m) :- policy{id: Q, value:
  15m}.` changes `question_default_after` from 15m to 5m and keeps its `vary`.
- Given `architectures/workstation.yaml` with 8 links, `remove(L) :- link{index: L, from:
  "worker", to: "builds"}.` is applied, leaves 7 links with no link from `worker` to `builds`, and
  changes nothing else.
- Given `architectures/workstation.yaml`, `connect(A, B, "question") :- part{id: A, block:
  "agent", count: 12}, part{id: B, block: "principal"}.` adds exactly one link `worker -> owner`
  carrying `question`, where before no link ran from `worker` to `owner`.
- Given `architectures/workstation.yaml`, `replicate(P, 2) :- part{id: P, block: "agent", count:
  12}.` changes `worker`'s count from 12 to 24, keeps its `vary`, and leaves `operator` at 1.
- Given the document where link 6 (`gateway -> model`) is wrapped with `timeout{after: 10m}`,
  `prepend` of `retry{attempts: 3}` on the link from `gateway` gives the path `gateway -> retry ->
  timeout -> model`, and `append` of the same guard on that path's last link gives `gateway ->
  timeout -> retry -> model`: prepend inserts before the guards already on a link, append after.
- Given `architectures/workstation.yaml`, `unmark(M) :- marker{path:
  "parts.gateway.rate_limit_per_minute", value: V}.` leaves `rate_limit_per_minute: 120` with no
  `vary`, and `marker` no longer lists that path; `mark` of `parts.builds.job_limit` with `vary:
  [5m, 10m, 20m]` makes `marker` list it with value 10m.
- Given `architectures/workstation.yaml`, `agenticspec query` with the selector `part{id: P,
  block: "agent"}` prints the rows `operator` and `worker` and no other.
- Given a `result.json` of `experiments/workstation-tuning.yaml`, `agenticspec query` of
  `kpi{part: "worker", id: "tasks_correct", value: V}` prints V equal to
  `best.per_part.worker.tasks_correct`.
- Given an experiment over `architectures/workstation.yaml` that lists the wrap patch above as a
  mutation, `agenticspec-sim run` writes a `result.json` whose `variants` hold variants with and
  without that mutation, each naming the mutations it applied, where before the search varied
  only the marked numbers.
- Given the authored scenarios under `blocks/scenarios/architecture`, the conformance run of
  `task check:blocks` witnesses the `applied` outcome of `agentic.architecture.RecordPatchApplied`,
  where before no scenario reached it.
