---
format: aep.planning-md/3
id: story:part-kinds-and-settings
kind: story
status: active
title: Part kinds from one source, typed settings per block
summary: The checker refuses part and link kinds the architecture domain does not declare; PartKind gains authority, process, work, question, delegation; settings are typed per block (consumer gaps M3, M4)
relations:
- decomposes: epic:simulator-and-search
- depends_on: story:contract-derivation
scope:
- confidence: inferred
  path: architectures
- confidence: cited
  path: blocks/9-composition/architecture.yaml
- confidence: cited
  path: blocks/scenarios/architecture
- confidence: cited
  path: crates/agenticspec-sim/src/check.rs
- confidence: cited
  path: crates/agenticspec-sim/tests/cases/check.rs
- confidence: cited
  path: docs/blocks/9-composition/architecture.md
- confidence: cited
  path: kpis/catalogue.yaml
- confidence: inferred
  path: website/src/generated
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 11}
---
## Outcome

The kinds of parts and links an architecture document may use come from one source, and a part's
settings are typed per block. A consumer's gap report (M3, M4) found two gaps at `25fbfcb`:

- M3: `agentic.architecture.PartKind` has no variant for authority, process, work, question or
  delegation (`blocks/9-composition/architecture.yaml:26-36`; its `UNMAPPED:` comment at lines
  30-33 leaves them out). `agenticspec-sim check` compares neither a part's `block` with
  `PartKind` nor a link's `carries` with `Carries` (`architecture.yaml:41-43`): `Refusal` has no
  variant for an unknown kind (`crates/agenticspec-sim/src/check.rs:12-43`), and `check`
  (`check.rs:249-409`) reads `block` and `carries` only to match the values it knows. So the
  accepted documents use kinds the domain does not declare: `block: authority`, `work` and
  `experiment` (`architectures/ladder/06-experiments.yaml:24-28`, `08-live-loop.yaml:29-40`),
  `carries: review` (`architectures/author-reviewer.yaml:28`, checked at `check.rs:318`) and
  `carries: handoff` (`architectures/handoff.yaml:17`).
- M4: a part's settings are a name and a text value (`agentic.architecture.Setting`,
  `architecture.yaml:53-62`; its `UNMAPPED:` at lines 56-57 leaves the typed settings to the
  block domains).

## Scope

- `blocks/9-composition/architecture.yaml`: `PartKind` gains `Authority`, `Process`, `Work`,
  `Question`, `Delegation`, and every other block value the 19 accepted documents use
  (`experiment`, `kpi`); `Carries` gains every carried value they use, or the document is changed
  to an existing variant where the value is a synonym, decided per value here. The typed settings
  of each kind: per kind, the setting names and their types, each type a block's own where the
  block declares one (a pool's `capacity` an Integer and its `kind` `agentic.resource.PoolKind`,
  `blocks/8-place/resource.yaml:18-35`; a sandbox's `isolation` `agentic.shared.IsolationLevel`,
  `blocks/8-place/sandbox.yaml:54`). Open point, settled here: a struct per kind in this domain, or `Setting` with a typed
  value; the comments at lines 26-33 and 53-57 change to what is taken and their markers go.
- One source: `check.rs` reads the kinds from the block file, or from a file generated from it and
  held by `task check:generated`; a test fails when the two differ. New refusals, gate KPIs in
  `GATE_KPIS` (`check.rs:53-69`): `unknown_part_kind`, `unknown_link_kind`, `unknown_setting`,
  `setting_type_mismatch`.
- `kpis/catalogue.yaml`: one entry per new refusal, `block: architecture`, appended as text.
- The authored scenarios under `blocks/scenarios/architecture/` whose `settings:` lists change.
- Documents under `architectures/` and `architectures/ladder/` changed only where a value is
  renamed; generated pages through `task docs:generate`.
- `crates/agenticspec-sim/tests/cases/check.rs`; builds and tests through `slot`, no test over
  60 s, the test step within 120 s, a pool job over 10 minutes is killed.

## Shared surfaces

- `blocks/9-composition/architecture.yaml`: `story:contract-derivation` (active) holds a fixed
  table from document blocks (`agent`, `pool`, `authority`, `work`, `experiment`, `kpi`, ...) to
  the domains whose ports they use, and the list of settings that narrow each port. This story
  makes the kind list and the typed settings that table and list read, so it depends on that
  story and lands after it. Also `story:architecture-patches`, `story:guards-and-limits`,
  `story:block-bindings`.
- `crates/agenticspec-sim/src/check.rs`: `story:architecture-references` (reads this story's kind
  source, comes after), `story:simulator-fidelity` (comes after, reads the `Process` kind),
  `story:architecture-kpis`.

## Acceptance

- Given a document with a part `{ block: frobnicator }`, `agenticspec-sim check` exits 1 with
  `unknown_part_kind` naming the part, where before it accepted it.
- Given a document with a link `carries: telepathy`, `check` exits 1 with `unknown_link_kind`
  naming the link, where before it accepted it.
- Given a document with parts of block `authority`, `process`, `work`, `question` and
  `delegation`, `check` raises no kind refusal for them.
- Given a pool part with `capacity: many`, `check` exits 1 with `setting_type_mismatch` naming
  the part and `capacity`; given a sandbox part with a setting name its kind does not declare, it
  exits 1 with `unknown_setting`.
- Given each of the 19 documents `agenticspec-sim check` accepts at `25fbfcb` (the 10 under
  `architectures/` and the 9 under `architectures/ladder/`), `check` still exits 0.
- Given a variant added to `PartKind` in the block file and not to the checker's source, the test
  of the one source fails.
- `ess specify validate --path blocks --strict-requires`, `task check:blocks` and
  `task check:kpis` pass.

## Not in this story

Simulating the new part kinds (`story:simulator-fidelity` runs process parts); which port a kind
uses (`story:contract-derivation`); the architecture part kind (`story:architecture-references`).
