---
format: aep.planning-md/3
id: story:architecture-references
kind: story
status: implemented
title: An architecture as a part of another, referenced by path and digest
summary: 'A part { block: architecture, ref, digest } whose border is its interface; links address <part>.<border part>; check resolves it, render draws one box (consumer gaps M1, M2)'
relations:
- decomposes: epic:simulator-and-search
- depends_on: story:part-kinds-and-settings
scope:
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: cited
  path: blocks/scenarios/architecture
- confidence: cited
  path: crates/agenticspec-docs/src/arch.rs
- confidence: cited
  path: crates/agenticspec-render/src/graph.rs
- confidence: cited
  path: crates/agenticspec-render/src/lib.rs
- confidence: cited
  path: crates/agenticspec-sim/src/check.rs
- confidence: cited
  path: crates/agenticspec-sim/src/doc.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/tests
- confidence: cited
  path: design/building-blocks.md
- confidence: cited
  path: docs/architectures/index.md
- confidence: cited
  path: docs/blocks/9-composition/architecture.md
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: inferred
  path: website/src/generated
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:07Z", actor: "human:operator", revision: 16}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:07Z", actor: "human:operator", revision: 17}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:12Z", actor: "human:operator", revision: 18, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An architecture document can hold another architecture as one part, pinned by digest. A
consumer's gap report (M1, M2) asked for a group or sub-architecture and for a reference from one
architecture document to another; at `25fbfcb` neither exists: `agentic.architecture.PartKind`
has no variant for an architecture (`blocks/9-composition/architecture.yaml:34-36`), and a link
that names `team.lead` is refused by `agenticspec-sim check` as `unknown_link_parts`
(`crates/agenticspec-sim/src/check.rs:260-272`).

The decided shape:

- a part `{ block: architecture, ref: architectures/<x>.yaml, digest: <sha256> }`;
- the referenced document's `border` is the part's interface;
- links address `<part>.<border part>` (`operator -> team.lead carries order`);
- the checker resolves the reference, compares the digest and refuses a link to an inner part
  that is not on the inner border;
- the renderer draws the part as one box that carries the reference, so it opens as its own
  document (in the studio, as its own draft);
- each document keeps its own contract.

Taken: the digest is the SHA-256 of the referenced file's bytes, as `sources.sha256` digests
sources (`crates/agenticspec-docs/src/results.rs:2-18`). Open point, settled here:
`agentic.architecture.Digest` is declared as the digest of the canonical form of parts, links and
border (`blocks/9-composition/architecture.yaml:20-24`), so a reformatted file keeps its
canonical digest but not its byte digest. The story either keeps the byte digest and says why in
the block's comment, or computes the canonical one.

Taken: `agenticspec-sim run` on a document with an architecture part exits 1 naming the part
(not simulated); simulating the inner document is a later story.

## Scope

- `blocks/9-composition/architecture.yaml`: the `Architecture` part kind in the one source of
  kinds `story:part-kinds-and-settings` makes; `ref` and `digest` on `agentic.architecture.Part`
  and `AddPart` (Part at line 118, AddPart at line 411 at `25fbfcb`); an authored scenario under
  `blocks/scenarios/architecture/` for each new refusal synthesis cannot reach.
- `crates/agenticspec-sim/src/doc.rs`, `src/check.rs`: load the referenced file relative to the
  referring one, compare its digest, read its `border`, resolve `<part>.<border part>` in `from`,
  `to` and `over`. New refusals, each a gate KPI in `GATE_KPIS` (`check.rs:53-69`):
  `unresolved_reference`, `reference_digest_mismatch`, `link_to_inner_non_border`,
  `reference_cycle`.
- `crates/agenticspec-render/src/graph.rs`, `src/lib.rs`: one box per architecture part with its
  `ref`, edges to it labelled with the inner border part.
- `crates/agenticspec-docs/src/arch.rs:571`: the format paragraph of the architectures overview
  names the architecture part, `ref`, `digest` and `<part>.<border part>`; regenerated
  `docs/architectures/index.md` (line 15) through `task docs:generate`.
- `design/building-blocks.md` § "An architecture document" (line 206): the format.
- `kpis/catalogue.yaml`: one entry per new refusal, `block: architecture`, appended as text.
- `crates/agenticspec-sim/tests/cases/check.rs` and fixtures under `crates/agenticspec-sim/tests/`;
  no file under `architectures/` is added, so the 10 docs examples stay 10.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s; a pool job over
  10 minutes is killed.

## Shared surfaces

- `blocks/9-composition/architecture.yaml`: `story:part-kinds-and-settings` (comes first, this
  story reads its kind source), `story:contract-derivation` (active), `story:architecture-patches`,
  `story:guards-and-limits`, `story:block-bindings`.
- `crates/agenticspec-sim/src/check.rs`: `story:part-kinds-and-settings`,
  `story:simulator-fidelity`, `story:architecture-kpis` (adds a module beside it and the `check`
  report).
- `crates/agenticspec-render/src/graph.rs`, `src/lib.rs`: `story:large-graph-view`,
  `story:site-ui-loop` (active).
- Contract: `story:contract-derivation` reads no reference at this commit (its body names no
  `ref` and its table from document blocks to domains has no `architecture` entry), so there is no
  edge. Whichever of the two lands second adds the `architecture` entry to that table: under "each
  document keeps its own contract", the outer contract names the part and the inner document's
  digest and lists none of the inner ports.

## Acceptance

- Given a fixture `outer.yaml` with a part `team: { block: architecture, ref: inner.yaml, digest:
  <sha256 of inner.yaml> }`, where `inner.yaml` has `border: [lead]`, and a link `operator ->
  team.lead carries order`, `agenticspec-sim check outer.yaml` exits 0, where before it refused
  the link as `unknown_link_parts`.
- Given `inner.yaml` changed by one byte after the digest was written, `check` exits 1 with
  `reference_digest_mismatch` naming `team` and both digests.
- Given a link `operator -> team.worker` where `worker` is a part of `inner.yaml` but not on its
  border, `check` exits 1 with `link_to_inner_non_border` naming the link.
- Given `ref: missing.yaml`, `check` exits 1 with `unresolved_reference`; given `a.yaml`
  referencing `b.yaml` that references `a.yaml`, it exits 1 with `reference_cycle`.
- Given `agenticspec-sim render outer.yaml`, the graph has one node for `team` carrying `ref:
  inner.yaml`, no node for a part of `inner.yaml`, and the edge from `operator` ends at `team`
  labelled `lead`.
- Given `agenticspec-sim run` on `outer.yaml`, it exits 1 naming `team` as an architecture part
  that is not simulated.
- Given the generated docs, `docs/architectures/index.md` names the architecture part, `ref`,
  `digest` and `<part>.<border part>`, and `task check:docs` passes.
- `task check:kpis` and `task check:blocks` pass with the four new refusals listed and witnessed.

## Not in this story

Simulating an inner architecture (flattening it into the outer run); the contract of a document
with an architecture part (see § "Shared surfaces"); opening a referenced document as a draft in
the studio (`story:studio-drafts`); references to documents outside the repository.
