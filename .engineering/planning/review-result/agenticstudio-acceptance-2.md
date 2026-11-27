---
format: aep.planning-md/3
id: review-result:agenticstudio-acceptance-2
kind: review-result
status: active
title: 'agenticstudio plan, acceptance critic, round 2: needs-revision'
owner: aep:plan-critic-acceptance
relations:
- reviews: epic:agenticstudio
- reviews: story:studio-first-slice
- reviews: story:studio-live
- reviews: story:studio-editor
- reviews: story:studio-evaluation
- reviews: story:studio-mcp
- reviews: story:studio-drafts
- reviews: story:studio-state-space
- reviews: story:studio-chat
- reviews: story:architecture-patches
- reviews: story:guards-and-limits
revision: 1
---
needs-revision

All 11 round-1 findings are resolved. The revision adds four new findings (origin introduced), each the same kind as round 1's "outcome promises more than the acceptance checks".

architecture-patches — the Outcome and Scope promise the verbs `remove`, `connect`, `replicate`, `wrap`, `prepend`, `append`, `mark` and `unmark`, but the acceptance exercises only `wrap`, a refused `remove`, and `set` on a policy, so the story can close with six verbs unbuilt and `remove` never shown to succeed — .engineering/planning/story/architecture-patches.md:47 (Scope) against :81 (Acceptance)

studio-first-slice — the Outcome says "A refused patch shows the refusing rule and its error and changes nothing", but the only acceptance bullet is a successful `set`, so the refusal path of the page can close unbuilt — .engineering/planning/story/studio-first-slice.md:31 (Outcome) against :108 (Acceptance)

studio-mcp — the Outcome promises four tools, `query(selector)`, `apply`, `check` and `simulate(budget)`, but the acceptance calls only `apply` and `simulate`, so `query` and `check` can close unchecked — .engineering/planning/story/studio-mcp.md:26 (Outcome) against :63 (Acceptance)

studio-drafts — the comparison bullet says it lists "that one changed setting", but the story's fact table makes `count` a field of `part` and not a `setting`, so the check can pass or fail depending on how "setting" is read — .engineering/planning/story/studio-drafts.md:66

**Round-1 findings, one by one:**
- **architecture-patches, three outcomes in one sentence (resolved):** the acceptance is now eight bullets, one statement each.
- **architecture-patches, selector matched nothing (resolved):** the selector is now `link{index: L, carries: "model_call", from: A}, part{id: A, block: "agent"}`. In `architectures/workstation.yaml` the model_call links with an agent source are 4 (`operator -> gateway`) and 5 (`worker -> gateway`), as the bullet says.
- **architecture-patches, refusal names nothing (resolved):** the refusal bullets now name their patches. One is a wrap plus `frobnicate` refused at rule position 2. The other is `remove(L) ... from: "worker"` refused with `agent_without_model_path` naming `worker`. That refusal holds, because `check.rs:297-304` follows only `model_call` links and `worker -> operator` carries `question`.
- **architecture-patches, experiment names nothing (resolved):** the bullet now names the wrap patch as a mutation, requires `variants` with and without it, each naming its mutations, and states the before.
- **studio-first-slice, field `name` (resolved):** the selector is now `part{id: P, block: "agent", count: 12}`. Only `worker` matches, since `operator` has count 1. The fact table fixes the fields.
- **studio-mcp, two outcomes joined (resolved):** the live update, the `ToolCalls` view, the dry run, the refusal and the KPI explanation are separate bullets.
- **studio-drafts, archive, undo and compare unchecked (resolved):** each has a bullet. `studio.draft.ArchivedDraft` exists in `studio/domains/draft.yaml:51`.
- **studio-state-space, trace, variants and landscape unchecked (resolved):** one bullet per view, and the state graph bullet states its before ("`result.json` had no `reached_states`").
- **studio-editor, only highlighting checked (resolved):** the bullets now cover highlight, live dry run with the draft staying at revision 1, apply to revision 2, and viewing revision 1 while the current revision stays 2.
- **guards-and-limits, three outcomes (resolved):** the acceptance is split into four bullets.
- **guards-and-limits, names no guard and reads the same before and after (resolved):** the bullets name the four kinds, state "no guard domain" as the before, and check at least two KPIs per kind. `blocks/6-governance/` holds only `authority.yaml` today, so the before is true.

**What I read:** all 10 ids in full with `aep plan artifact show`, plus `review-result:agenticstudio-acceptance-1`. I checked `architectures/workstation.yaml`, `experiments/workstation-tuning.yaml`, `crates/agenticspec-sim/src/check.rs`, `kpis/catalogue.yaml`, `blocks/6-governance`, `studio/domains/draft.yaml` and `Taskfile.yml`. I took the line numbers for the findings from the story files under `.engineering/planning/story/`.

**What I could not establish:**
- Whether `studio-evaluation`'s equality of best fitness holds. It needs the search to be deterministic for a given seed, and its budget B is a named open point in the story. I did not run it.
- Whether `studio-live`'s Outcome, "every operation is a tool call of a session", is checked. Its bullets cover only the event channel and the revision routes. The session and `ToolCalls` record is checked only in `studio-mcp`, with caller `Agent`. I did not raise it as a finding because round 1 passed this story.
- Out of my lane: `guards-and-limits` decomposes `epic:research-blocks`, not `epic:agenticstudio`. Whether wrapping with `BudgetCap` or `ApprovalGate` works is promised in `guards-and-limits.md:36` ("wraps with all six") and is a scope matter.

```findings
- file: .engineering/planning/story/architecture-patches.md
  line: 47
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Outcome and Scope promise the verbs remove, connect, replicate, wrap, prepend, append, mark and unmark, but the acceptance exercises only wrap, a refused remove and set on a policy, so the story can close with six verbs unbuilt and remove never shown to succeed"
- file: .engineering/planning/story/studio-first-slice.md
  line: 31
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Outcome says 'A refused patch shows the refusing rule and its error and changes nothing', but the only acceptance bullet is a successful set, so the refusal path of the page can close unbuilt"
- file: .engineering/planning/story/studio-mcp.md
  line: 26
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Outcome promises four tools, query(selector), apply, check and simulate(budget), but the acceptance calls only apply and simulate, so query and check can close unchecked"
- file: .engineering/planning/story/studio-drafts.md
  line: 66
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the comparison bullet says it lists 'that one changed setting', but the story's fact table makes count a field of part and not a setting, so the check can pass or fail depending on how 'setting' is read"
```
