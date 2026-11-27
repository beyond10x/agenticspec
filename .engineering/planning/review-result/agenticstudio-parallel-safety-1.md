---
format: aep.planning-md/3
id: review-result:agenticstudio-parallel-safety-1
kind: review-result
status: active
title: 'agenticstudio plan, parallel-safety critic, round 1: needs-revision'
owner: aep:plan-critic-parallel-safety
relations:
- reviews: epic:agenticstudio
- reviews: story:architecture-patches
- reviews: story:consumer-gaps
- reviews: story:guards-and-limits
- reviews: story:optimization-view
- reviews: story:site-ui-loop
- reviews: story:studio-chat
- reviews: story:studio-drafts
- reviews: story:studio-editor
- reviews: story:studio-evaluation
- reviews: story:studio-mcp
- reviews: story:studio-state-space
- reviews: story:triggers-schedules-swarms
revision: 1
---
needs-revision

- story:studio-evaluation — it and story:studio-editor both land on `crates/agenticstudio` (cited, both bodies: the editor serves the built app, evaluation adds the run, routes and result page). No `depends_on` edge orders them and neither body says so. `aep plan artifact waves` separates them only because the scope overlap pushes evaluation to wave 4, and that separation goes away if either scope is narrowed. Remedy: an ordering edge that records the shared crate as its reason, or splitting the crate surface — .engineering/planning/story/studio-evaluation.md:32, .engineering/planning/story/studio-editor.md:38
- story:architecture-patches — it, story:studio-evaluation and story:studio-state-space all land on `crates/agenticspec-sim` (cited, all three bodies). Patches adds structural mutations to the search, evaluation adds the in-memory experiment entry, and state-space adds `reached_states` to the result. No edge runs between patches and either of the other two, and none of the bodies says so. Remedy: an ordering edge naming the shared crate as its reason, or splitting the surface — .engineering/planning/story/architecture-patches.md:47, .engineering/planning/story/studio-evaluation.md:29, .engineering/planning/story/studio-state-space.md:30
- story:studio-drafts — it shares `crates/agenticstudio` and `docs/studio.md` with story:studio-mcp, and `ui/studio` with story:studio-state-space and story:studio-chat (all cited). Drafts has no edge to mcp, state-space or chat, and its body admits none of the overlaps. Drafts also persists the sessions and tool calls mcp opens. Remedy: an ordering edge per pair that records the shared file, or splitting the surface — .engineering/planning/story/studio-drafts.md:29, .engineering/planning/story/studio-mcp.md:36
- story:studio-mcp — it and story:studio-editor both land on `crates/agenticstudio` (cited: "the session an MCP client opens on a draft" against "serves the built app"). Nothing orders them and neither body mentions the other. Remedy: an ordering edge that records the shared crate, or splitting the surface — .engineering/planning/story/studio-mcp.md:36, .engineering/planning/story/studio-editor.md:38
- story:studio-chat — it and story:studio-state-space both add a panel under `ui/studio` (cited, both bodies). The two are unordered, and chat's body does not say it shares the app directory with state-space. Remedy: an ordering edge that records `ui/studio`, or splitting the app into per-panel directories that the scopes name — .engineering/planning/story/studio-chat.md:33, .engineering/planning/story/studio-state-space.md:34
- story:guards-and-limits — the item has no `scope` key, so the CLI lists it as unassessed. Its body names `blocks/` (too wide to be a surface), `kpis/catalogue.yaml` and a `agenticspec-sim` test. That sim surface collides with story:studio-evaluation and story:studio-state-space, which sit in the same set with no edge to guards. Remedy: record a narrowed scope (the guard block files, the catalogue, the sim test), then order guards against those two or split the sim surface — .engineering/planning/story/guards-and-limits.md:26
- story:studio-evaluation — its body says it "settles the `UNMAPPED:` of `studio.evaluation.Evaluation`", which sits in `studio/domains/evaluation.yaml:27` (cited), but its scope omits `studio`. story:studio-editor generates types "over `studio/`" and has no edge to evaluation. The collision on that file is inferred, but the omission is cited. Remedy: add `studio` to the scope and order the two, or split the surface — .engineering/planning/story/studio-evaluation.md:31, .engineering/planning/story/studio-editor.md:31

**What you read:** 10 artifacts (the 9 stories plus story:guards-and-limits) and epic:agenticstudio, through `aep plan artifact list`, `waves`, `show` on each, `relations` and `graph`. I also checked `studio/domains/*.yaml`, `architectures/workstation.yaml`, `experiments/` and the `UNMAPPED:` markers by `ls` and `grep`.

**Surfaces:** 9 items cited through their scope, 0 inferred, 1 not placeable by scope (story:guards-and-limits, which has no scope key; I took its paths from the body only). I traced 13 acceptance reads to a producer in the set, and a `depends_on` edge records all 13. The reader and producer pairs I traced:
- live to first-slice
- editor to live
- evaluation to live
- mcp to live
- mcp to evaluation
- drafts to editor
- drafts to evaluation
- state-space to evaluation
- state-space to editor
- chat to mcp
- chat to editor
- patches to guards
- patches to first-slice

**What I could not establish:**
- The studio-mcp tool descriptions "carry the patch verbs". Those verbs come from story:architecture-patches, which has no edge from or to mcp. It is not an acceptance read, so it is not a finding.
- story:studio-drafts also settles an `UNMAPPED:` in `studio/domains/draft.yaml:20` without listing `studio` in its scope. It is ordered after the editor, so I did not report it.
- Whether the `crates/agenticstudio` and `ui/studio` collisions touch the same files or only the same directory. The scopes name directories, so those findings are at crate or app granularity. A single crate listed in 8 stories is also a wide surface, but whether to split it is the design critic's call and out of my lane.
- The collisions with story:consumer-gaps, story:triggers-schedules-swarms and story:site-ui-loop, and the `agenticspec-render` collisions with story:optimization-view and story:site-ui-loop. Those items are outside the set.

```findings
- file: .engineering/planning/story/studio-evaluation.md
  line: 32
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "this and story:studio-editor both land on crates/agenticstudio (cited, both bodies), no depends_on edge orders them and neither body says so; waves separates them only through the scope overlap"
- file: .engineering/planning/story/architecture-patches.md
  line: 47
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "this, story:studio-evaluation and story:studio-state-space all land on crates/agenticspec-sim (cited, all three bodies) with no edge between patches and the other two and no body admitting it"
- file: .engineering/planning/story/studio-drafts.md
  line: 29
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "this shares crates/agenticstudio and docs/studio.md with story:studio-mcp and ui/studio with story:studio-state-space and story:studio-chat (cited), with no edge to any of the three and no body admitting it"
- file: .engineering/planning/story/studio-mcp.md
  line: 36
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "this and story:studio-editor both land on crates/agenticstudio (cited, both bodies) with no edge between them and neither body mentioning the other"
- file: .engineering/planning/story/studio-chat.md
  line: 33
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "this and story:studio-state-space both add a panel under ui/studio (cited, both bodies), are unordered, and chat's body does not admit the shared app directory"
- file: .engineering/planning/story/guards-and-limits.md
  line: 26
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "no scope key, so the item is unassessed; its body names blocks/ (too wide), kpis/catalogue.yaml and an agenticspec-sim test, and the sim surface collides with story:studio-evaluation and story:studio-state-space, which have no edge to it"
- file: .engineering/planning/story/studio-evaluation.md
  line: 31
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the body settles an UNMAPPED in studio/domains/evaluation.yaml but the scope omits studio, which story:studio-editor generates types over with no edge between them; the collision on that file is inferred, the omission is cited"
```
