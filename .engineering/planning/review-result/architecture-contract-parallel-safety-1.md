---
format: aep.planning-md/3
id: review-result:architecture-contract-parallel-safety-1
kind: review-result
status: active
title: 'architecture-contract plan, parallel-safety critic, round 1: needs-revision'
owner: aep:plan-critic-parallel-safety
relations:
- reviews: epic:architecture-contract
- reviews: story:block-bindings
- reviews: story:port-derivation
- reviews: story:contract-derivation
- reviews: story:contract-docs
- reviews: story:contract-conformance
- reviews: story:routing-invariants
revision: 1
---
needs-revision

- story:port-derivation — the scope omits `crates/agenticspec-sim/Cargo.toml` and `Cargo.lock`, which this story must edit. It uses "the `regex` crate of the workspace" and a SHA-256 `sources_sha256` digest, but the sim crate depends on neither `regex` nor `sha2`. The file is cited, not inferred. No story in the set lists a manifest, so the plan hides the one file that every later `contract/` story's build also reads. State the manifest and lockfile as a surface of this story, and add them to `Shared surfaces` if a later story also changes dependencies. — .engineering/planning/story/port-derivation.md:55, crates/agenticspec-sim/Cargo.toml (`[dependencies]` has no `regex` or `sha2`)

**What I read.** 7 artifacts, all by `aep plan artifact show` (the epic and the six stories). I also ran `aep plan artifact graph`, `aep plan artifact waves`, and read `Cargo.toml`, `crates/agenticspec-sim/Cargo.toml`, `crates/agenticspec-docs/Cargo.toml` and `crates/agenticspec-docs/src/results.rs`.

**Parts 3 and 4.**
- **Surfaces.** I placed all 6 stories by cited surfaces, with 0 inferred and 0 unplaced. A few individual entries are marked inferred in the stories themselves: `design/ess-repro`, `docs/blocks/9-composition/architecture.md` and `ux/site/baseline.json`.
- **Collisions.** The six stories form a complete chain of `depends_on` edges, so no pair can run at once. Every shared file (`CHANGELOG.md`, `docs/contract.md`, `Taskfile.yml`, `crates/agenticspec-sim/src/{lib,main}.rs`, `tests/`, `src/contract/`, `website/static/catalog`, `docs/blocks`, `website/src/generated`) is ordered by an edge.
- **Waves.** `aep plan artifact waves` puts the six in separate waves (5, 6, 8, 9, 10, plus `block-bindings` in wave 2), with no collision line for any of them.
- **Readers and producers.** I traced 10 acceptance reads to a producer inside the set, and a `depends_on` edge records all 10. Examples are `ports.json`, the bindings, the contract format and predicate language, the contract downloads under `website/static/catalog`, and the epic sections the stories write.
- **Not assessed.** Collisions with `story:architecture-kpis`, `story:architecture-patches` and `story:guards-and-limits` are outside the set. The stories name them and `architecture-kpis` has an edge from each story.

**Out of your lane, not a finding.** The `SIM_EXCLUDED` check in `crates/agenticspec-docs/src/results.rs:177` matches exact paths. Adding `contract/` as a directory entry, as `port-derivation` plans, would not exclude the files under it. That is an acceptance or design matter for the other critics.

```findings
- file: .engineering/planning/story/port-derivation.md
  line: 55
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the scope omits crates/agenticspec-sim/Cargo.toml and Cargo.lock, which this story edits to add regex and sha2 (the sim crate depends on neither), so a manifest surface that later stories and outside items also read is not recorded (cited)"
```
