---
format: aep.planning-md/3
id: review-result:agenticstudio-scope-2
kind: review-result
status: active
title: 'agenticstudio plan, scope critic, round 2: needs-revision'
owner: aep:plan-critic-scope
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

- epic:agenticstudio — "with the layer check and the structural check live" promises two checks, and the set claims one. `studio-first-slice` and `studio-editor` show only "the check result" of `agenticspec-sim`'s check. That check has no layer rule: `check.rs` has only structural refusals, and `grep -i layer crates/agenticspec-sim/src/*.rs` finds nothing. The block-layer check (`task check:layers`) runs on `blocks/`, not on a draft. No body says what a layer check means for a draft, and none records that it was dropped. `story:studio-editor` would most naturally take it, or say it is not claimed. — .engineering/planning/epic/agenticstudio.md:30

**Round-1 findings**
1. Resolved. The KPI change of a dry run is now claimed by `story:studio-evaluation`: its scope bullet "Dry run" and an acceptance line. `story:studio-mcp` passes it through (line 35 of the scope and an acceptance line).
2. Resolved. "Run an experiment from the editor" is now claimed by `story:studio-evaluation`: scope bullet `ui/studio` "the run control in the editor" and the acceptance line on `Running`.
3. Resolved. "Explain a KPI" is now claimed by `story:studio-mcp`: scope "Explaining a KPI" (`summary`, `unit` and `direction` from `kpis/catalogue.yaml`) and the last acceptance line.
4. Resolved. Accepting a variant is no longer narrowed to the best one: `studio-evaluation` scope "Accepting a variant: any `id` ... `best` or not" and an acceptance line that uses a variant that is not best.
5. My round-1 out-of-lane note on how `kpi` facts reach `query` is also resolved. `story:studio-evaluation` scope "`kpi` facts" and an acceptance line now cover it, and the first-slice table names `story:studio-evaluation` as a maker of `kpi`.

**What I read:** I read 12 artifacts: `epic:agenticstudio`, the 9 stories, `story:guards-and-limits`, and `review-result:agenticstudio-scope-1`. I also read `epic:research-blocks`. I used `aep plan artifact show` on each and `aep plan artifact graph`. I checked `check.rs`, `Taskfile.yml` and `studio/domains/draft.yaml`. I extracted 26 promises from the parent and traced 25 to an item. The one untraced is the layer check above.

**What I could not establish:**
- Reach beyond the parent: none found.
  - `studio-chat` is traceable to the parent's status table ("an agent channel embedded in the browser (later)") and is tagged `later`.
  - `architecture-patches` authors scenarios under `blocks/scenarios/architecture`. The parent does not name them, but `task check:blocks` runs authored scenarios and the scenarios witness the patch nouns the parent declares. I did not count it as reach.
  - `agenticspec query` traces to the parent's "the studio, the CLI and the optimizer" and "the same language then reads ... architectures".
- Out of my lane, not counted: whether "named drafts" has an acceptance line for naming. `studio-drafts` has none, and the name field and `DraftNameTaken` exist in `studio/domains/draft.yaml`. That belongs to the acceptance critic.

```findings
- file: .engineering/planning/epic/agenticstudio.md
  line: 30
  category: scope
  severity: warning
  verdict: needs-revision
  origin: undecided
  message: "epic:agenticstudio promises \"the layer check and the structural check live\" and the set claims only the structural check of agenticspec-sim (which has no layer rule), with nothing saying what a layer check is for a draft or that it was dropped; story:studio-editor would most naturally take it"
```
