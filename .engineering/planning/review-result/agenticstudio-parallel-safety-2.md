---
format: aep.planning-md/3
id: review-result:agenticstudio-parallel-safety-2
kind: review-result
status: active
title: 'agenticstudio plan, parallel-safety critic, round 2: needs-revision'
owner: aep:plan-critic-parallel-safety
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

All seven round-1 findings are resolved. I found one new collision, on generated type output. It rests on an inferred surface, so I rate it a warning.

- story:studio-editor — its `ui/studio` types come from `ess generate types` over the `architecture` block ("read, not changed"). `story:architecture-patches` and `story:guards-and-limits` both change `blocks/9-composition/architecture.yaml`, and the editor has no edge to either. The repo commits and byte-checks generated type output (`scripts/check-generated.sh` lists `ui/app/src/generated`, held by `task check:generated`), so whichever lands second turns the check red. This is **inferred**: the editor's body does not name where the types land or whether they are committed. The check script's header says one more line is all a new generated directory needs. Remedy, no choice made: an ordering edge from editor to patches or guards that records the architecture block as its reason, or generating the types at build time under `target/` so nothing committed depends on the block — .engineering/planning/story/studio-editor.md:31, .engineering/planning/story/architecture-patches.md:47
- story:studio-first-slice — it types drafts and revisions from `studio/` and the `architecture` block with `ess generate types`. `story:guards-and-limits` adds `Guard` to `agentic.architecture.PartKind` in that block, and the two have no edge. A committed generated crate would drift the same way. This is **inferred**, for the same reason: the body does not say where the types land. Remedy, no choice made: say so in the body and add an ordering edge, or keep the generated types out of the tree — .engineering/planning/story/studio-first-slice.md:31, .engineering/planning/story/guards-and-limits.md:41

**Round-1 findings, one by one:**
1. Editor and evaluation on `crates/agenticstudio`: **resolved**. Evaluation depends_on editor (studio-evaluation.md relations). Its "Shared surfaces" section names the crate.
2. Patches, evaluation and state-space on `crates/agenticspec-sim`: **resolved**. Evaluation depends_on patches, and state-space follows through evaluation. All three bodies name the crate.
3. Drafts against mcp, state-space and chat: **resolved**. Drafts depends_on mcp, state-space depends_on drafts, and chat depends_on state-space. Each body has a "Shared surfaces" section.
4. Mcp and editor on `crates/agenticstudio`: **resolved**. Mcp depends_on editor.
5. Chat and state-space on `ui/studio`: **resolved**. Chat depends_on state-space.
6. Guards had no `scope`: **resolved**. A scope is recorded, and it is ordered before patches, evaluation and state-space through patches. Two of its surfaces were established only from the body, and they sit in the same chain: `docs/blocks` and `website/src/generated` are inferred.
7. Evaluation's scope omitted `studio`: **resolved**. `studio` and `ui/studio` are now in its scope, and `studio` is in drafts' scope. Evaluation and drafts are ordered through editor, mcp and evaluation.

**What I read:** 10 stories and epic:agenticstudio, plus review-result:agenticstudio-parallel-safety-1. Commands: `aep plan artifact show` on all ten and on the review result, `aep plan artifact graph`, and `aep plan artifact waves`. I also read `scripts/check-generated.sh`, `studio/ess-inputs.yaml` and `studio/system.yaml`, and ran `git grep` for "generate types" and for PartKind/PatchVerb.

**Surfaces:** I placed 10 of 10 items from their recorded scope, which covers files and directories. Of those, 10 are cited, because every item has at least one cited surface. Within them, 3 sit on inferred scope entries (`docs/blocks`, `docs/kpis-catalogue.md` and `website/src/generated` on guards, and `docs/blocks` and `website/src/generated` on patches). None are unplaceable.

I traced 17 acceptance reads to a producer in the set. A `depends_on` chain records all 17: 16 as a direct edge, and mcp's read of patch verbs (the `story:architecture-patches` verbs its tool descriptions carry) through the evaluation edge.

After the revision the set runs as a near-linear chain: first-slice, live, editor, guards, patches, evaluation, mcp, drafts, state-space, chat. The only unordered pairs are guards against first-slice, live and editor, and patches against live and editor. Their cited scopes are disjoint, apart from the two inferred findings above.

**What I could not establish:**
- Where the studio's generated types land and whether they are committed. No story body says. That is why both findings are inferred.
- The `waves` output lists collisions with items outside this set: consumer-gaps and triggers-schedules-swarms on `blocks/9-composition/architecture.yaml`, and site-ui-loop on `website/src/generated`. They are outside my set and I did not judge them.
- Whether `crates/agenticstudio` and `ui/studio` collisions touch the same files or only the same directory. All the ordered pairs on them are safe at either granularity.
- Out of my lane: `crates/agenticstudio` is listed in 8 stories, which is wide. Whether to split it is the design critic's call.

```findings
- file: .engineering/planning/story/studio-editor.md
  line: 31
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "inferred: the editor generates ui/studio types with ess generate types over the architecture block (read, not changed), while story:architecture-patches and story:guards-and-limits both change blocks/9-composition/architecture.yaml and no depends_on edge orders them; the repo commits generated types and byte-checks them (scripts/check-generated.sh), so the second to land turns check:generated red, and the body does not say where the types land or admit the dependency"
- file: .engineering/planning/story/studio-first-slice.md
  line: 31
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "inferred: the first slice types drafts and revisions from studio/ and the architecture block with ess generate types, while story:guards-and-limits adds Guard to agentic.architecture.PartKind in that block and no depends_on edge orders them; if the generated output is committed it drifts, and the body does not say where it lands or admit the dependency"
```
