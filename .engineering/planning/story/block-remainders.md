---
format: aep.planning-md/3
id: story:block-remainders
kind: story
status: active
title: 'Block remainders from a consumer review: budgets, work, routes, claims, harness, triggers'
summary: Budget windows and model-call budgets, joined starts, person steps, step claims, process outcomes, route auth and billing, multi-unit claims, claim runs, queue order, model policy, unique session names, classifier mode, trigger grammar and duplicate firings (consumer gaps M20, M22, M23, M26, M27, M28)
relations:
- decomposes: epic:building-blocks
scope:
- confidence: cited
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/2-intelligence/model.yaml
- confidence: cited
  path: blocks/4-agent/harness.yaml
- confidence: cited
  path: blocks/5-work/process.yaml
- confidence: cited
  path: blocks/5-work/work.yaml
- confidence: cited
  path: blocks/6-governance/authority.yaml
- confidence: cited
  path: blocks/7-collaboration/trigger.yaml
- confidence: cited
  path: blocks/8-place/resource.yaml
- confidence: cited
  path: blocks/8-place/runtime.yaml
- confidence: cited
  path: blocks/scenarios
- confidence: cited
  path: docs/blocks
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: inferred
  path: website/src/generated
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 15}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 16}
---
## Outcome

The blocks hold the remaining concepts a consumer's review of the blocks at `25fbfcb` found
missing or only written as `UNMAPPED:` prose, outside governance. A consumer's gap report lists
them as M20, M22, M23, M26, M27 and M28; `story:governance-types` covers M18, M19, M21, M24 and
M25 and is not repeated here. Each item names the place at `25fbfcb`:

- M20, budgets and starts:
  - a windowed budget has no window start and no reset; the block says so in an `UNMAPPED:` note
    (`blocks/6-governance/authority.yaml:34-42`);
  - no budget resource counts model calls (`BudgetResource` is `Tokens`, `Money`, `WallTime`,
    `ToolCalls`, `authority.yaml:44-46`);
  - a start that joins a running instance is described in a field summary
    (`blocks/8-place/runtime.yaml:114-119`), and `StartInstance` has no outcome for it, so the
    join leaves no record (`runtime.yaml:328-354`).
- M22, work:
  - whether a person step is required is left to the evaluator (`blocks/5-work/work.yaml:209-213`);
  - a step names no claims (`agentic.work.Step`, `work.yaml:214-246`);
  - a process cannot name outcomes of its own: `RunOutcome` is the fixed list `Done`, `Failed`,
    `Stopped`, `BudgetExhausted` (`work.yaml:18-25`).
- M23, model routes: a route is a name, an endpoint and a model; it names neither how it
  authenticates nor how it is billed (`agentic.model.Route`, `blocks/2-intelligence/model.yaml:150-162`).
- M26, resource claims:
  - a claim is for one unit (`agentic.resource.Claim`, `blocks/8-place/resource.yaml:48-50`;
    `RequestClaim`, `resource.yaml:193-200`);
  - a claim names no run (`resource.yaml:54-68`);
  - a grant out of queue order is not refused (`resource.yaml:10-11`).
- M27, harness sessions:
  - a session can name a requested route (`blocks/4-agent/harness.yaml:123-127`), but not a named
    model policy with a ceiling;
  - a session's name is optional and not unique (`harness.yaml:117-122`); `OpenSession` has no
    outcome for a name in use (`harness.yaml:558-621`);
  - no permission mode lets a classifier decide (`PermissionMode` is `Ask`, `AcceptEdits`,
    `PlanOnly`, `Unrestricted`, `harness.yaml:39-46`), though a classifier is named as a decider
    of a call (`harness.yaml:294-297`).
- M28, triggers:
  - `condition` and `intake` are free text with no grammar (`blocks/7-collaboration/trigger.yaml:116-129`);
  - a second firing for the same subject key is not refused (`trigger.yaml:178-184`).

## Scope

- `blocks/6-governance/authority.yaml`, `blocks/8-place/runtime.yaml`, `blocks/5-work/work.yaml`,
  `blocks/5-work/process.yaml`, `blocks/2-intelligence/model.yaml`, `blocks/8-place/resource.yaml`,
  `blocks/4-agent/harness.yaml`, `blocks/7-collaboration/trigger.yaml`,
  `blocks/0-foundation/shared.yaml`: the fields, states, commands and refusals below. Each
  `UNMAPPED:` note named above is removed or rewritten to what is taken. The layer rule holds: a
  block names only its own layer or a lower one; a concept that would point upward is turned
  around, as in `design/consumer-gaps.md`.
- Authored scenarios under `blocks/scenarios/` for every new refusal synthesis cannot reach (the
  row-set guards of M26 and M28 are the expected cases, see the synthesis cost noted at
  `trigger.yaml:182-184`).
- `kpis/catalogue.yaml`: a KPI per new refusal kind where `scripts/check-kpis.sh` requires it,
  appended as text; generated pages through `task docs:generate`.
- The grammar of M28 is written on the trigger block's docs page and in the field summaries; its
  parser is not in this story.

## Shared surfaces

- `blocks/6-governance/authority.yaml`, `blocks/0-foundation/shared.yaml`: `story:governance-types`
  (approvers, grants, prohibitions). `blocks/` as a whole: `story:block-bindings` (bindings across
  the same files) and `story:guards-and-limits` (a guard domain beside `authority`). This story
  reads no output of those stories, so there is no edge; whichever lands second rebases, and the
  ordering is set when they leave draft, as `story:governance-types` says.
- `blocks/5-work/work.yaml` person steps: `story:simulator-fidelity` runs process parts and reads
  nothing of this story.

## Acceptance

- M20: given a budget with window `Day`, its record names the window's start and a reset at the
  next window opens a new spent count of 0, where before no field or command held either; given a
  budget on a new resource for model calls, a call over its limit is refused; given an instance
  `Running`, a `StartInstance` for its workload records a join that names the running instance and
  creates no second one, where before a second instance was created.
- M22: given a process whose step is marked as requiring a person, a run that ends that step
  without a person's record is refused; given a step that held a claim, the step's record names
  the claim; given a process that declares outcomes `merged` and `abandoned`, a run of it ends
  with `merged`, and a run that names an outcome its process does not declare is refused.
- M23: given a route, its record names its authentication kind and its billing kind (taken: an
  enum each, the variants settled here), and opening a route without them is refused or takes a
  stated default.
- M26: given a pool of capacity 4 and a claim for 3 units, granting it leaves 1 unit and a second
  claim for 2 waits; given a claim, it names the run it serves; given two queued claims, granting
  the later one first is refused.
- M27: given a model policy named `standard` with ceiling route R, a session under it that
  requests a route above R is refused; given an open session named `n`, opening a second session
  named `n` is refused while the first is open; a session opened in the classifier mode records
  each decision with the classifier as decider.
- M28: given a trigger whose `condition` or `intake` does not follow the grammar written in this
  story, `CreateTrigger` refuses it; given a firing for subject key K that has not ended, a second
  `FireTrigger` for K on the same trigger is refused.
- `ess specify validate --path blocks --strict-requires` and `task check:blocks` pass, with every
  new refusal witnessed by synthesis or an authored scenario, within the 120 s test step.

## Not in this story

The governance items (`story:governance-types`); a parser or evaluator of the trigger grammar; the
simulator modelling multi-unit claims or process outcomes; the evaluator that applies a model
policy or a classifier.
