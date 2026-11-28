---
format: aep.planning-md/3
id: review-result:architecture-contract-parallel-safety-2
kind: review-result
status: active
title: 'architecture-contract plan, parallel-safety critic, round 2: approve'
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
approve

Round-1 finding, resolved. The one finding was `story:port-derivation` omitting the sim crate's manifest and lockfile from its scope.
- Its scope now lists `crates/agenticspec-sim/Cargo.toml` and `Cargo.lock` (cited, `port-derivation` body, Scope and Shared surfaces).
- The claim that `regex` and `sha2` are already workspace dependencies holds: `Cargo.toml:23-24` has `sha2 = "0.10"` and `regex = "1"`.
- The sim crate's `[dependencies]` has neither today, so the edit it names is real.
- `Shared surfaces` says no other story of the epic changes a dependency, and that a later story needing one adds both files and comes after `port-derivation` by `depends_on`.
- `SIM_DEPS` at `crates/agenticspec-docs/src/results.rs:28` is `rayon`, `serde_json`, `serde_yaml_ng`, and the digest reads only those locked versions. The lockfile change therefore moves no catalog digest.

Round-1 out-of-lane note on `SIM_EXCLUDED`: also addressed. The match at `results.rs:177` is still exact-path (`rel == format!(...)`). `port-derivation` now plans a prefix match, a docs test and an empty `git diff` of `website/static/catalog`.

What I read: 7 artifacts, plus the round-1 record. I ran `aep plan artifact show` on each, `aep plan artifact graph` and `aep plan artifact waves`. I read the root `Cargo.toml`, `crates/agenticspec-sim/Cargo.toml` and `results.rs`. I checked that `architectures/workstation.yaml`, `profiles/coding.yaml` and `scripts/check-blocks-conform.sh` exist. The `UNMAPPED` count the `contract-derivation` acceptance names is 3 in the tree now, so its "3 before" holds.

Surfaces: 6 stories placed by cited surfaces, 0 inferred, 0 unplaceable. Some individual entries are marked inferred by the stories themselves: `design/ess-repro`, `docs/blocks/9-composition/architecture.md`, `ux/site/baseline.json`.

Collisions inside the set: the six stories form a full `depends_on` chain. Every shared file is ordered by an edge and named under `Shared surfaces`: `CHANGELOG.md`, `docs/contract.md`, `Taskfile.yml`, the sim `lib.rs` and `cli.rs`, `main.rs`, `tests/`, `src/contract/`, `spec/cli.yaml`, `website/static/catalog` and `website/src/generated`.

Readers and producers: 10 acceptance reads traced to a producer inside the set, and all 10 are recorded by a `depends_on` edge. Examples are `blocks/ports.json`, the predicate parser, the contract downloads, the bindings, and the cached conformance suite.

What I could not establish:
- Collisions with `story:studio-evaluation` and `story:studio-state-space` (epic:agenticstudio) are outside the set, so I did not assess them and they set no verdict.
  - Both claim all of `crates/agenticspec-sim`.
  - `aep plan artifact waves` reports collision lines against all six stories on `cli.rs`, `lib.rs`, `tests/` and `src/contract/`, plus `Cargo.toml` against `port-derivation`, and `sim.rs` and `src/sim` against `routing-invariants`.
  - Neither body is linked to the set by an edge, and no body in the set names either. The body that would have to name it is theirs.
- Collisions with `story:architecture-catalog`, `story:ui-generator`, `story:site-ui-loop`, `story:guards-and-limits` and others the tool lists are likewise outside the set.

```findings
[]
```
