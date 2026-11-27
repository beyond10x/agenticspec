---
format: aep.planning-md/3
id: story:guards-and-limits
kind: story
status: draft
title: 'Guards as building blocks: timeout, retry, budget cap, approval gate'
summary: Four guard kinds declared in blocks/6-governance/guard.yaml with two KPIs each, a timeout and a retry simulated on a link; reviewer and verifier stay agent parts.
relations:
- decomposes: epic:research-blocks
- depends_on: story:studio-first-slice
scope:
- confidence: cited
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/6-governance/guard.yaml
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: cited
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/tests/cases
- confidence: cited
  path: crates/agenticstudio/src/generated
- confidence: inferred
  path: docs/blocks
- confidence: inferred
  path: docs/kpis-catalogue.md
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: inferred
  path: website/src/generated
revision: 7
---
## Outcome

Guards are building blocks: a timeout, a retry, a budget cap and an approval gate can be put on a
link or around a part, with their refusals and KPIs, so that a patch can `wrap` with them and the
simulator can run them.

The six guards of `epic:agenticstudio` § "Operations: select, then transform", and where each
lives:

| guard | where | this story |
|---|---|---|
| timeout | new guard kind `Timeout` | declares it, two KPIs, simulated on a link |
| retry | new guard kind `Retry` | declares it, two KPIs, simulated on a link |
| budget cap | new guard kind `BudgetCap` (the budgets themselves stay in `agentic.authority`) | declares it, two KPIs |
| approval gate | new guard kind `ApprovalGate` (the verdict `ApprovalRequired` is in `agentic.shared.Verdict`) | declares it, two KPIs |
| reviewer | an agent part with `role: reviewer`, already checked (`review_without_reviewer` in `crates/agenticspec-sim/src/check.rs`) and simulated (`architectures/author-reviewer.yaml`) | nothing |
| verifier | an agent part with `role: verifier` | nothing; open point: what the simulator models of a verifier |

`story:architecture-patches` wraps with all six; reviewer and verifier as agent parts.

## Source

`design/research-findings.md` § "Changes to existing blocks" (harness: enforced limits, a revision
budget); `epic:agenticstudio` § "Operations: select, then transform".

## Scope

- `blocks/6-governance/guard.yaml` (new, domain `agentic.guard`): the guard kinds `Timeout`,
  `Retry`, `BudgetCap` and `ApprovalGate`, their settings and refusals. Open point: the layer;
  taken: 6, beside `agentic.authority`, which holds budgets and approval requirements.
- `blocks/0-foundation/shared.yaml`: the guard's identifier.
- `blocks/9-composition/architecture.yaml`: `Guard` in `agentic.architecture.PartKind`.
- The document form of a guard part in `agenticspec-architecture/1`: a part with `block: guard`,
  `kind` naming the guard kind and its settings (taken: `{ block: guard, kind: timeout, after:
  10m }`, `{ block: guard, kind: retry, attempts: 2 }`), placed between the two ends of the link
  it guards.
- `kpis/catalogue.yaml`: at least two KPIs per guard kind, with `block: guard` and an id starting
  with the kind (`timeout_`, `retry_`, `budget_cap_`, `approval_gate_`).
- `crates/agenticspec-sim/src/sim.rs`: a timeout and a retry guard on a link.
- `crates/agenticspec-sim/tests/cases/guards.rs` (new, a module of the crate's one test binary
  `tests/main.rs`): the simulator test below. A test ends within 60 s, the test step within 120 s.
- Generated pages that `task generate` rewrites from the blocks and the catalogue: `docs/blocks/`,
  `docs/kpis-catalogue.md`, `website/src/generated/`.

## Shared surfaces

- `crates/agenticspec-sim` (`src/sim.rs`, `tests/cases`): before `story:architecture-patches`,
  `story:studio-evaluation` and `story:studio-state-space`, which depend on this story through
  patches.
- `blocks/9-composition/architecture.yaml`, `docs/blocks/`, `website/src/generated/`: before
  `story:architecture-patches`, which depends on this story.
- `crates/agenticstudio/src/generated/` (types generated from the `architecture` block): after
  `story:studio-first-slice`, which made them and which this story depends on; this story
  regenerates them when it adds `Guard` to `agentic.architecture.PartKind`. Before
  `story:studio-editor`, which depends on this story and generates `ui/studio/src/generated/`
  from the block as this story leaves it.

## Acceptance

- Given `blocks/` with no guard domain, after this story `ess specify validate --path blocks
  --strict-requires` and `task check:layers` pass with `blocks/6-governance/guard.yaml`
  declaring the guard kinds `Timeout`, `Retry`, `BudgetCap` and `ApprovalGate`.
- Given `kpis/catalogue.yaml` with no entry of `block: guard`, after this story `task
  check:kpis` passes with at least two such entries for each of the four kinds.
- Given an architecture whose `worker -> gateway` link is wrapped by a timeout guard whose
  `after` is shorter than every model call of the profile, the `agenticspec-sim` test reads a
  `timeout_` KPI of the guard part above 0, where the same run without the guard has no such part.
- Given an architecture whose `worker -> gateway` link is wrapped by a retry guard with
  `attempts: 2` and whose gateway's `rate_limit_per_minute` limits calls, the `agenticspec-sim`
  test reads a `retry_` KPI of the guard part above 0.
