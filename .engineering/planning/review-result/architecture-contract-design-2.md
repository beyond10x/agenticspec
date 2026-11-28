---
format: aep.planning-md/3
id: review-result:architecture-contract-design-2
kind: review-result
status: active
title: 'architecture-contract plan, design critic, round 2: approve'
owner: aep:plan-critic-design
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

Round-1 finding, `story:contract-derivation` (the predicate language had no parser in the set): resolved. `story:contract-derivation` now has:
- Its own `src/contract/predicate.rs` with the grammar and a parser that returns a typed `Predicate` or a refusal naming the offset (`contract-derivation.md:75`).
- A contract check that every invariant's predicate parses (`:82`).
- Parser tests on every predicate form and on malformed text (`:94`).
- An acceptance line for a malformed fixture, which exits 1 naming the link and the offset (`:146`).
- A shared-surface note that `story:routing-invariants` adds its evaluator beside the parser, not in it (`:113`).

`story:routing-invariants` evaluates the typed `Predicate` and parses no text (`routing-invariants.md:63-64`). The grammar is first exercised in the story that defines it.

A fresh read found no new design defect.

**What you read.** 7 artifacts shown whole: the epic and 6 stories, plus `review-result:architecture-contract-design-1`. I ran `aep plan artifact relations`, `graph` and `validate` (valid, 45 artifacts). I also read `crates/agenticspec-docs/src/results.rs` and `git ls-files` for the simulator crate.

**What you could not establish**
- **Edges and cycles:** I walked 39 `depends_on` and `decomposes` edges, including those out to `story:architecture-kpis`, `epic:building-blocks` and `epic:simulator-and-search`. I found no cycle.
- **Acceptance reads:** I traced 14 reads of state a sibling produces, and all 14 have a `depends_on` edge. Examples are `blocks/ports.json`, the bindings, the contract output, the contract downloads, `docs/contract.md`, the "What ESS lacks" section and the cached suite.
- **The chain:** the six stories form a full linear chain. Each edge has its shared surface written beside it (`docs/contract.md`, `CHANGELOG.md`, `spec/cli.yaml`, `src/contract/`, `src/cli.rs`, `main.rs`, `tests/`). This is the trade-off the rubric names, not a finding. The alternative is to split `docs/contract.md` into one section per story and drop the `contract-docs` edges to `contract-conformance` and `routing-invariants`.
- **Outside the set, not a defect:** at this commit the simulator is the separate `agenticspec-sim` binary.
  - There is no `agenticspec sim` subcommand, no `crates/agenticspec-sim/src/cli.rs` and no `sim` entry in `spec/cli.yaml`, yet the epic body and four stories name them.
  - They exist only on unmerged branch `build/release-platforms` (commit 33c0392), which no set item depends on.
  - `cli.rs` is not in `SIM_EXCLUDED` today (`results.rs:25`), so the digest effect of that move is not settled here.
- **Out of my lane:**
  - `story:architecture-patches` and `story:guards-and-limits` share block files and `crates/agenticspec-sim` with these stories, and no edge records it (parallel-safety).
  - The unmerged-branch dependency above is for scope or parallel-safety to take up.

```findings
[]
```
