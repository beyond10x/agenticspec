---
format: aep.planning-md/3
id: review-result:agenticstudio-acceptance-1
kind: review-result
status: active
title: 'agenticstudio plan, acceptance critic, round 1: needs-revision'
owner: aep:plan-critic-acceptance
relations:
- reviews: epic:agenticstudio
- reviews: story:studio-live
revision: 1
---
needs-revision

architecture-patches — the acceptance holds three independent outcomes (a wrap diff, a refusal, a search result), so one can pass while the others fail and the story is neither done nor not done — .engineering/planning/story/architecture-patches.md:52

architecture-patches — the selector `part{id: A, role: "worker"}` matches nothing in `architectures/workstation.yaml`, because no part there carries a `role` (only `worker: { block: agent, harness: coding, count: … }`), and "gives the expected diff" names no diff, so the check cannot be run or compared — .engineering/planning/story/architecture-patches.md:52

architecture-patches — "A refused patch changes nothing and names its rule and error" names no patch, no rule position and no error, so any refusal satisfies it — .engineering/planning/story/architecture-patches.md:55

architecture-patches — "An experiment that lists patches as mutations reports the best structure" names no experiment, no expected structure and no comparison to the numbers-only search, so it reads the same before and after the work — .engineering/planning/story/architecture-patches.md:56

studio-first-slice — the acceptance selector `part{id: P, name: "worker"}` uses a field `name` that is neither in the epic's `part{id, kind, role, count}` nor on any part of `workstation.yaml`, so the browser test cannot match `worker` as written — .engineering/planning/story/studio-first-slice.md:58

studio-mcp — "receives the diff and the check result in one answer, and a browser open on that draft shows the new revision without a reload" joins two independent outcomes (the MCP answer, the live browser update), and the second is already the acceptance of `story:studio-live` — .engineering/planning/story/studio-mcp.md:42

studio-drafts — the outcome promises archive, undo and side-by-side comparison, but the one-sentence acceptance checks only persistence across a restart, so the story can close with those three unbuilt — .engineering/planning/story/studio-drafts.md:40

studio-state-space — the outcome promises trace, variants and landscape as well as the state graph, but the acceptance checks only that the state graph's counts equal `reached_states`, so three of the four views are never checked — .engineering/planning/story/studio-state-space.md:38

studio-editor — the outcome promises a patch editor with live dry run, apply and stepping through revisions, but the acceptance checks only selector highlighting, so those three are never checked — .engineering/planning/story/studio-editor.md:43

guards-and-limits — the acceptance holds three independent outcomes (declared guards, KPIs per guard, simulator run) — .engineering/planning/story/guards-and-limits.md:24

guards-and-limits — "Guards declared in `blocks/` … (`task check:blocks` passes)" names none of the six guards and reads the same before the work as after it, since `task check:blocks` already passes and no timeout, approval-gate or budget-cap block exists under `blocks/` or `kpis/` — .engineering/planning/story/guards-and-limits.md:24

Nothing found in studio-live, studio-evaluation and studio-chat. Each has one observable statement and states a before and after.

**What I read:** all 10 ids were read in full with `aep plan artifact show` (studio-first-slice, studio-live, studio-editor, studio-evaluation, studio-mcp, studio-drafts, studio-state-space, studio-chat, architecture-patches, guards-and-limits). I also ran `aep plan artifact kinds` and `lifecycle story`, read `epic:agenticstudio`'s "Operations" section, and checked `architectures/workstation.yaml`, `experiments/workstation-tuning.yaml`, `Taskfile.yml` and the `.engineering/planning/story/*.md` line numbers.

**What I could not establish:**
- Whether the not-yet-built `part` fact maps `block` or `kind` to a `role`, which would make the `role: "worker"` selector valid. No story defines the fact's fields. `studio-editor`'s `block: "agent"` selector has the same open question, but it is not a finding because it does match `operator` and `worker` in the document.
- Whether the `studio-evaluation` equality of best fitness holds, since it needs the search to be deterministic for a given seed. I did not run it.
- Out of my lane: the fact schema is defined nowhere across `studio-first-slice` and `architecture-patches`. That is a design and scope matter. `guards-and-limits` decomposes `epic:research-blocks`, not `epic:agenticstudio`. `studio-evaluation` and `studio-mcp` call for budget-bound runs, so their test timing is a parallel-safety and budget question, not an acceptance one. `studio-evaluation` has a smaller version of the unchecked-outcomes gap: its acceptance does not check the result page or accepting the best variant.

```findings
- file: .engineering/planning/story/architecture-patches.md
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance holds three independent outcomes (a wrap diff, a refusal, a search result), so one can pass while the others fail"
- file: .engineering/planning/story/architecture-patches.md
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the selector part{id: A, role: \"worker\"} matches nothing in architectures/workstation.yaml (no part has a role) and 'gives the expected diff' names no diff, so the check cannot be run or compared"
- file: .engineering/planning/story/architecture-patches.md
  line: 55
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "'A refused patch changes nothing and names its rule and error' names no patch, rule position or error, so any refusal satisfies it"
- file: .engineering/planning/story/architecture-patches.md
  line: 56
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "'An experiment that lists patches as mutations reports the best structure' names no experiment, no expected structure and no comparison to the numbers-only search, so it reads the same before and after"
- file: .engineering/planning/story/studio-first-slice.md
  line: 58
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance selector part{id: P, name: \"worker\"} uses a field `name` that is neither in the epic's part{id, kind, role, count} nor on any part of workstation.yaml, so the browser test cannot match worker as written"
- file: .engineering/planning/story/studio-mcp.md
  line: 42
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins two independent outcomes (the MCP answer carrying diff and check result, and the browser showing the new revision without a reload), the second already being the acceptance of story:studio-live"
- file: .engineering/planning/story/studio-drafts.md
  line: 40
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the outcome promises archive, undo and side-by-side comparison but the acceptance checks only persistence across a restart, so the story can close with those three unbuilt"
- file: .engineering/planning/story/studio-state-space.md
  line: 38
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the outcome promises trace, variants and landscape as well as the state graph but the acceptance checks only that the state graph's counts equal reached_states"
- file: .engineering/planning/story/studio-editor.md
  line: 43
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the outcome promises a patch editor with live dry run, apply and stepping through revisions but the acceptance checks only selector highlighting, so those three are never checked"
- file: .engineering/planning/story/guards-and-limits.md
  line: 24
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: pre-existing
  message: "the acceptance holds three independent outcomes (declared guards, KPIs per guard, simulator run)"
- file: .engineering/planning/story/guards-and-limits.md
  line: 24
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: pre-existing
  message: "'Guards declared in blocks/ (task check:blocks passes)' names none of the six guards and reads the same before and after, since task check:blocks already passes and no timeout, approval-gate or budget-cap block exists"
```
