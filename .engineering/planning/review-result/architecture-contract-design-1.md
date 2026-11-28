---
format: aep.planning-md/3
id: review-result:architecture-contract-design-1
kind: review-result
status: active
title: 'architecture-contract plan, design critic, round 1: needs-revision'
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
needs-revision

Round 1, `plan-critic-design`. The set has no cycle. Every `depends_on` edge a reader needs is declared. The one finding is a split abstraction (the predicate language).

1. `story:contract-derivation` — its body defines the routing-invariant predicate language and writes predicates in it, but nothing in the story parses them: the contract check at line 65 lists ports, parts, links and invariant ports only, and no acceptance line parses a predicate. The only evaluator is in `story:routing-invariants`, so the grammar is first exercised in that later story. The fix is to have `story:contract-derivation` own a parser and add "every invariant's predicate parses" to its contract check and acceptance. `story:routing-invariants` then evaluates a parsed form. — `.engineering/planning/story/contract-derivation.md:62`, `.engineering/planning/story/routing-invariants.md:53`

**What you read**
- 7 artifacts: the epic and 6 stories, shown whole with `aep plan artifact show`.
- Commands: `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate`. Validate reported `valid`.
- I also read `crates/agenticspec-docs/src/results.rs` for `SIM_EXCLUDED`, and checked the file paths and line numbers of the finding.

**What you could not establish**
- **Edges and cycles:** I walked 28 `depends_on` and `decomposes` edges, including those out to `story:architecture-kpis`, `epic:building-blocks` and `epic:simulator-and-search`. No cycle.
- **Acceptance reads:** I traced 11 acceptance reads of state another item in the set produces. All 11 have a `depends_on` edge.
  - `block-bindings` bindings, read by `port-derivation`, `contract-derivation` and `contract-conformance`.
  - `blocks/ports.json` and the ESS-gap exemption in the epic body, read by `contract-derivation`.
  - The contract output, read by `contract-docs`, `contract-conformance` and `routing-invariants`.
  - The contract downloads in `website/static/catalog`, read by `routing-invariants`.
- **The chain:** the six stories form a full linear chain. Each edge has its shared surface written beside it (`docs/contract.md`, `CHANGELOG.md`, `crates/agenticspec-sim/src/contract/`, `main.rs`, `lib.rs`, `tests/`). By the rubric that is the trade-off, not a finding.
  - The edges `contract-conformance` to `contract-docs`, and `routing-invariants` to `contract-docs`, are only `docs/contract.md` and `CHANGELOG.md`.
  - Keeping them is a sound ordering choice. The alternative is to give `docs/contract.md` one section per story and drop those edges.
- **Out of my lane:**
  - The four shared surfaces with `story:architecture-patches` and `story:guards-and-limits` (the block files, "whichever lands second rebases") have no edge. That is for `plan-critic-parallel-safety`.
  - `story:architecture-kpis` edits `crates/agenticspec-sim/src/check.rs`. `results.rs` does not exclude that file, so the catalog `sources.sha256` digests change when it lands. That is not named in this set and belongs to scope or parallel-safety.

```findings
- file: .engineering/planning/story/contract-derivation.md
  line: 62
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:contract-derivation defines the routing-invariant predicate language and writes predicates in it, but its contract check and acceptance never parse a predicate; the only evaluator is in story:routing-invariants (routing-invariants.md:53), so the language is first exercised in that later story; the contract-derivation body should own a parser and check that every invariant's predicate parses, leaving routing-invariants to evaluate the parsed form"
```
