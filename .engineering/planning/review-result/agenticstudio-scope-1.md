---
format: aep.planning-md/3
id: review-result:agenticstudio-scope-1
kind: review-result
status: active
title: 'agenticstudio plan, scope critic, round 1: needs-revision'
owner: aep:plan-critic-scope
relations:
- reviews: epic:agenticstudio
- reviews: story:architecture-patches
- reviews: story:guards-and-limits
- reviews: story:optimization-view
- reviews: story:studio-evaluation
- reviews: story:studio-first-slice
- reviews: story:studio-live
revision: 1
---
needs-revision

- epic:agenticstudio — "A dry run returns the diff, the check result and the KPI change of a short simulation" is promised, and no story claims the KPI change. `studio-first-slice` (line 40) returns diff and check only, and `studio-mcp` (line 35) answers a dry run with the diff, the check result and "nothing committed". `studio-evaluation` would most naturally take it. — .engineering/planning/epic/agenticstudio.md:61
- epic:agenticstudio — "Evaluation: run an experiment from the editor" is promised, and no story gives `ui/studio` a control that starts a simulation. `studio-editor` has a selector box, a patch editor and a revision list, and `studio-evaluation` only adds a route and a result page. The set claims the run, not the start from the editor. `studio-evaluation` or `studio-editor` would take it. — .engineering/planning/epic/agenticstudio.md:31
- epic:agenticstudio — "tools that … explain a refusal or a KPI" is promised. A refusal is covered by the named rule and error. Nothing claims a KPI explanation: `studio-mcp` (lines 33-35) says its tool descriptions carry only "the patch verbs and the fact relations", and `query` returns kpi values with no meaning. `studio-mcp` would take it. — .engineering/planning/epic/agenticstudio.md:28
- story:studio-evaluation — the parent's "Optimizations: accept a variant into the draft" is narrowed to "accepts the best variant", and nothing says accepting another variant of the search was dropped. — .engineering/planning/story/studio-evaluation.md:35 (parent .engineering/planning/epic/agenticstudio.md:34)

**What I read:** 11 artifacts (`epic:agenticstudio`, the 9 stories, `story:guards-and-limits`) with `aep plan artifact show`, plus `story:optimization-view`. I ran `aep plan artifact graph` and `grep` for `UNMAPPED` in `studio/` and `blocks/9-composition/architecture.yaml`. I extracted 24 promises from the parent and traced 20 to an item. The 4 untraced are the findings above, one of them narrowed rather than missing.

**What I could not establish:**
- Reach beyond the parent: none found. `studio-chat` is traceable only to the title phrase "embedded agent channel" (line 6). The parent names no gateway or API key, and the story is tagged `later`. I did not count it as reach.
- Out of my lane, not counted: how `kpi` facts reach the draft's `query` is a gap in the set. `story:architecture-patches` (line 41) defines the fact. `story:studio-live` queries "the draft's facts". Nothing in `story:studio-evaluation` says a finished evaluation's per-part KPIs become those facts. The parent's example selector (line 45-46) depends on that.
- The parent's "architecture facts emitted by `agenticspec-sim`" (line 68-70) is placed in `crates/agenticspec-patch` by `story:studio-first-slice`. I read that as a division of the work, not a gap.

```findings
- file: .engineering/planning/epic/agenticstudio.md
  line: 61
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "epic:agenticstudio promises that a dry run returns \"the KPI change of a short simulation\" and no story claims it; studio-evaluation would most naturally take it"
- file: .engineering/planning/epic/agenticstudio.md
  line: 31
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "epic:agenticstudio promises \"run an experiment from the editor\" and no story gives ui/studio a control that starts a simulation; studio-evaluation or studio-editor would take it"
- file: .engineering/planning/epic/agenticstudio.md
  line: 28
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "epic:agenticstudio promises tools that \"explain a refusal or a KPI\" and no story claims the KPI explanation; studio-mcp would most naturally take it"
- file: .engineering/planning/story/studio-evaluation.md
  line: 35
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:studio-evaluation narrows the parent's \"accept a variant into the draft\" to the best variant only and records no decision about accepting other variants"
```
