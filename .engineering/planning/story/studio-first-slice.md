---
format: aep.planning-md/3
id: story:studio-first-slice
kind: story
status: implemented
title: 'agenticstudio first slice: open a draft in the browser, apply a patch, see the check'
summary: agenticstudio serve opens a draft of an architecture document, applies a set/add patch atomically and shows the graph and the check result.
relations:
- decomposes: epic:agenticstudio
scope:
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/agenticspec-patch
- confidence: cited
  path: crates/agenticspec-records/src/sq
- confidence: cited
  path: crates/agenticstudio
- confidence: cited
  path: crates/agenticstudio/src/generated
- confidence: cited
  path: docs/studio.md
- confidence: cited
  path: scripts/check-generated.sh
- confidence: cited
  path: studio
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T22:24:14Z", actor: "human:operator", revision: 12, decided_on: {"recorded":{"review":1,"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-27T22:24:17Z", actor: "human:operator", revision: 13, decided_on: {"recorded":{"review":1,"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:12Z", actor: "human:operator", revision: 14, decided_on: {"recorded":{"test_result":1,"review":1,"review_outcome":3}}}
---
## Outcome

The first slice of agenticstudio: a user runs `agenticstudio serve`, opens a draft of an
architecture under `architectures/` in the browser, applies a patch, and sees the re-rendered
graph and the check result. A refused patch shows the refusing rule and its error and changes
nothing. Design: `epic:agenticstudio` § "Operations: select, then transform".

## Facts of an architecture

This story introduces the facts an architecture gives the query language of `queries/*.sq`
(`docs/queries.md`). Their fields are fixed here, once, and every selector in the stories of
`epic:agenticstudio` uses these fields and no others. They are read from the format
`agenticspec-architecture/1` (`docs/architectures/index.md`, `architectures/workstation.yaml`) as
`crates/agenticspec-sim` reads it: an untyped document whose `parts` map an id to a `block` and
the settings of that block, and whose `links` are a list of `from`, `to`, `carries` and an
optional `over` (`crates/agenticspec-sim/src/check.rs`, `crates/agenticspec-sim/src/sim.rs`).

| relation | fields | read from the document | made by |
|---|---|---|---|
| `part` | `id, block, count` | `id`: the part's key under `parts`; `block`: its `block`; `count`: its `count` (the `value` of a marked node), 1 when absent, as `sim.rs` reads it | this story |
| `setting` | `part, name, value` | one fact per other key of a part (`harness`, `kind`, `role`, `acts_for`, `per`, `capacity`, ...): `name` is the key, `value` the scalar or the `value` of a marked node; a list (`egress`) gives one fact per element | this story |
| `link` | `index, from, to, carries, over` | `index`: the 0-based position in `links`, as the check numbers links in its refusals (`Refusal::UnknownLinkPart` in `check.rs`); `from`, `to`, `carries` as written; `over` as written, null when absent | this story |
| `marker` | `path, value` | every marked place as `crates/agenticspec-sim/src/marked.rs` lists it: `path` such as `parts.worker.count`, `value` its value as built | this story |
| `policy` | `id, value` | the key under `policies` and its value (the `value` of a marked node) | `story:architecture-patches` |
| `kpi` | `part, id, value` | an entry of `per_part` of an `agenticspec-sim-result/1` document: of `best`, or of `baseline` when no variant is feasible (`crates/agenticspec-sim/src/search.rs`, `summary`) | `story:architecture-patches` (from a result document), `story:studio-evaluation` (from a draft's last finished evaluation) |

- No relation has a `name`, a `role` or a `kind` field, and a link has no `id`: the format has
  none of them as a field of every part or link. `role` (`architectures/author-reviewer.yaml`)
  and `kind` (`architectures/workstation.yaml`: `owner`, `builds`, `chat`) are settings of some
  parts and are read as `setting{part: P, name: "role", value: "reviewer"}`.
- `setting` is the architecture block's `agentic.architecture.Setting` (`name`, `value`) of a
  part. It is not in the list of relations in the comment of `agentic.architecture.PatchRule`
  (`blocks/9-composition/architecture.yaml`); that comment gains it with the next change of the
  block, which this story does not make.
- Values: sq holds null, booleans, integers and strings (`Val` in
  `crates/agenticspec-records/src/sq/facts.rs`). A duration is held as milliseconds, as sq
  compares durations (`docs/queries.md`).
- Open point: a value that is not a whole number has no sq type. No architecture document under
  `architectures/` has one outside its summary; a KPI value does (`per_part` holds floats), so
  `story:architecture-patches` settles it for `kpi.value`.

## Scope

- `crates/agenticspec-patch` (new library): the `agenticspec-patch/1` format (an ordered list of
  rules); the facts `part`, `setting`, `link` and `marker` of an architecture document, as the
  table above fixes them; the verbs `set` and `add`; atomic application to an architecture
  document (all rules or none), the result checked by `agenticspec-sim`'s check; a refusal names
  the 1-based rule position and the error. `set` on a marked place sets its `value` and keeps its
  `vary` (taken; the epic does not say). A dry run returns the diff and the check result and
  writes nothing; the KPI change of a short simulation is added by `story:studio-evaluation`.
- `crates/agenticspec-records/src/sq/`: a fact base built from those facts, beside the session
  facts.
- `crates/agenticstudio` (new binary): `agenticstudio serve --root <dir> --port <n>` on the
  pattern of `agenticspec serve` (std HTTP); routes to list the architectures, open a draft of
  one, read a draft (its document, its current revision's number and digest, its graph from
  `crates/agenticspec-render`, its check result) and apply a patch; one HTML page that shows the
  graph, a patch box and the check result. Drafts, their revisions, patches and patch
  applications held in memory, typed from `studio/` and the `architecture` block (`ess generate
  types`). The generated types are committed under `crates/agenticstudio/src/generated/` and
  listed in `scripts/check-generated.sh`, so `task check:generated` holds them; a later story that
  changes `studio/` or the `architecture` block regenerates them in the same change.
- `Cargo.toml` (workspace members); `Taskfile.yml`: `check:studio`, which runs the page's browser
  test against a server started once per run (open point: the runner; `ui/app` uses Playwright,
  `ui/app/playwright.config.ts`). `studio/` itself is already validated by `task check:blocks`
  (`ess specify validate --path studio --strict-requires`); this story adds no task for it.
- `docs/studio.md` (new page): run, open, apply.
- Tests within the budget of the workspace `AGENTS.md` (a test ends within 60 s, the test step
  within 120 s): the server's routes in process, no network; the page's browser test reads a
  server started once per run.

## Shared surfaces

This story comes first; every story below that shares a surface with it depends on it, directly
or through `story:studio-live`.

- `crates/agenticstudio`: `story:studio-live`, `story:studio-editor`, `story:studio-evaluation`,
  `story:studio-mcp`, `story:studio-drafts`, `story:studio-chat`.
- `docs/studio.md`: `story:studio-live`, `story:studio-evaluation`, `story:studio-mcp`,
  `story:studio-drafts`.
- `studio`: `story:studio-evaluation`, `story:studio-drafts` (each settles an `UNMAPPED:`).
- `crates/agenticspec-patch`: `story:architecture-patches`.
- `Cargo.toml`: `story:studio-mcp`. `Taskfile.yml`: `story:studio-editor`.
- `crates/agenticstudio/src/generated/` (types from the `architecture` block):
  `story:guards-and-limits` and `story:architecture-patches` change that block and depend on this
  story, so each regenerates the types.

## Acceptance

- Given a draft of `architectures/workstation.yaml` open at revision 1 with part `worker` at
  count 12, when a browser test on `agenticstudio serve` applies `set(P, count, 3) :- part{id:
  P, block: "agent", count: 12}.`, the page shows revision 2, part `worker` at count 3, part
  `operator` unchanged, and the check passing.
- Given that draft at revision 2, when the browser test applies a patch whose rule 1 is `set(P,
  count, 2) :- part{id: P, block: "agent"}.` and whose rule 2 is `frobnicate(P) :- part{id:
  P}.`, a verb this story does not know, the page shows the refusal with rule position 2 and the
  error's name, the draft stays at revision 2 with the same digest, and part
  `worker` stays at count 3.

## Not in this story

The other verbs, the `policy` and `kpi` facts (`story:architecture-patches`); sessions, tool
calls, live updates and the revision routes (`story:studio-live`); the Vue editor
(`story:studio-editor`); persistence (`story:studio-drafts`).
