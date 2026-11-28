---
format: aep.planning-md/3
id: story:port-derivation
kind: story
status: active
title: Derive the ports of the blocks from bindings and lifecycles, and compare them with the design's table
summary: agenticspec sim ports classifies every command and event of blocks/, derives Outbound, Clock and Principal ports into the committed blocks/ports.json, and lists every difference from the design's 15 ports.
relations:
- decomposes: epic:architecture-contract
- depends_on: story:block-bindings
- depends_on: story:architecture-kpis
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: blocks/ports.json
- confidence: cited
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-docs/src/results.rs
- confidence: cited
  path: crates/agenticspec-docs/tests
- confidence: cited
  path: crates/agenticspec-sim/Cargo.toml
- confidence: cited
  path: crates/agenticspec-sim/src/cli.rs
- confidence: cited
  path: crates/agenticspec-sim/src/contract
- confidence: cited
  path: crates/agenticspec-sim/src/lib.rs
- confidence: cited
  path: crates/agenticspec-sim/tests
- confidence: cited
  path: crates/agenticspec/src/main.rs
- confidence: cited
  path: docs/contract.md
- confidence: cited
  path: spec/cli.yaml
- confidence: cited
  path: spec/domains/cli.yaml
- confidence: cited
  path: website/docusaurus.config.ts
- confidence: cited
  path: website/sidebars.ts
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:33:36Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-27T23:33:36Z", actor: "human:operator", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`agenticspec sim ports` derives the ports of the building blocks from the compiled system of
`blocks/` (its bindings and lifecycles), by the rule of `epic:architecture-contract` § "Ports are
derived, not authored", and writes them as one committed file. A check compares the derived list
with the table of `design/architecture-runtime.md` § "The boundary: ports" and lists every
difference. Each derived port has the fields of `agentic.architecture.Port`
(`blocks/9-composition/architecture.yaml`): name, block, kind, intent event, waiting state, report
commands, protocol.

## Scope

- `crates/agenticspec-sim/src/contract/` (new module directory, `ports.rs`): read the output of
  `ess specify compile --path blocks --format json`; classify every command as internal (a
  binding causes it, or it decides on recorded facts), entry or configuration, and every event as
  exit or consumed; derive Outbound, Clock and Principal ports; build each port's protocol as a
  regular expression over short command names (the `regex` crate) and refuse one that does not
  parse or names a command outside the port.
- `crates/agenticspec-sim/Cargo.toml` and `Cargo.lock`: the sim crate gains `regex` and `sha2`
  (both already workspace dependencies in the root `Cargo.toml`, as `regex.workspace = true` and
  `sha2.workspace = true`). `SIM_DEPS` in `crates/agenticspec-docs/src/results.rs` is a fixed
  list (`rayon`, `serde_json`, `serde_yaml_ng`) and is not extended, so the lockfile change does
  not change any catalog digest: neither crate is on a simulation's path.
- A port's name: the block and the entity of its waiting state, in the form the story fixes and
  writes on the docs page (the design's names such as `model.infer` are matched by commands, not
  by name).
- `blocks/ports.json` (new, committed, format `agenticspec-ports/1`): the derived ports and, in
  `sources_sha256`, the SHA-256 digest of `blocks/ess-inputs.yaml`, every file it lists and the
  `ess --version` line. Generated, never edited by hand.
- `agenticspec sim ports --blocks blocks --out blocks/ports.json` writes it;
  `agenticspec sim ports --check blocks/ports.json` compares the digest only (no `ess` call, under
  a second) and fails naming the command that regenerates it; `task ports:generate`,
  `task check:ports` (in `task check`), `task slow:ports` (regenerates and compares bytes).
- `agenticspec sim ports --expect <file>`: one line per difference between the derived ports and
  an expected list (a port missing, a port extra, a different intent event, waiting state, report
  set or protocol), exit 0 when there is none and 1 otherwise. The expected list is the design's
  15 rows, copied into `crates/agenticspec-sim/tests/cases/ports-expected.yaml`.
- The subcommand `sim ports` (flags `--blocks`, `--out`, `--check`, `--expect`) is declared in
  `spec/cli.yaml`, its input and result types in `spec/domains/cli.yaml`; `task generate`
  regenerates `crates/agenticspec-cli-contract/` (held by `task check:generated`);
  `crates/agenticspec/src/main.rs` dispatches it to `crates/agenticspec-sim/src/cli.rs`;
  `crates/agenticspec-sim/src/lib.rs` lists the module.
- Catalog digests stay stable by extending the exclusion to prefixes:
  `crates/agenticspec-docs/src/results.rs` matches `SIM_EXCLUDED` against exact paths today
  (`rel == crates/agenticspec-sim/src/<entry>`), so a directory entry would exclude nothing. The
  story changes the match so that an entry ending in `/` excludes every file under it, and adds
  `contract/`; no file of `contract/` changes a simulation result. A test in
  `crates/agenticspec-docs/tests/main.rs` (on a copied tree, as the existing stale-simulator test
  does) changes a file under `crates/agenticspec-sim/src/contract/` and shows the digest of every
  source unchanged, and changes `crates/agenticspec-sim/src/search.rs` and shows it refused as
  before. The committed `website/static/catalog/*/sources.sha256` stay byte for byte the same.
- `crates/agenticspec-sim/tests/main.rs` and `tests/cases/`: the classification on small compiled
  fixtures (an outbound port, a principal port, a configuration command, a command a binding
  causes); each test under 5 s, none over 60 s; the test step within 120 s.
- `docs/contract.md` (new page, with its entry in `website/docusaurus.config.ts` `pages` and in
  `website/sidebars.ts`): what a port is, the three kinds, the command. `AGENTS.md`: the new
  generated file and tasks. `CHANGELOG.md`.
- Builds and tests through `slot`; a pool job over 10 minutes is killed.

## Shared surfaces

- `blocks/`: `story:block-bindings` writes the bindings this story reads; it comes first.
- `crates/agenticspec-sim/src/contract/`, `src/lib.rs`, `src/cli.rs`, `tests/`:
  `story:contract-derivation`, `story:contract-conformance`, `story:routing-invariants`.
  Outside this epic, `story:architecture-kpis` adds tests to `crates/agenticspec-sim/tests`.
- `spec/cli.yaml`, `spec/domains/cli.yaml`, `crates/agenticspec-cli-contract/`,
  `crates/agenticspec/src/main.rs`: `story:contract-derivation`, `story:contract-conformance`,
  `story:routing-invariants` each add a subcommand or a flag after this story.
- `crates/agenticspec-sim/Cargo.toml`, `Cargo.lock`: no other story of this epic changes a
  dependency; every later story that builds the crate reads them. A later story that needs a new
  dependency adds both files to its scope and comes after this one (all do, by `depends_on`).
- Outside this epic, `story:architecture-patches` records all of `crates/agenticspec-sim` and
  `crates/agenticspec/src` as its scope (so `Cargo.toml`, `cli.rs` and `main.rs` too); no edge
  joins them, `aep plan artifact waves` keeps them in different waves, and whichever lands second
  rebases on the other.
- `crates/agenticspec-docs/src/results.rs`, `crates/agenticspec-docs/tests/`:
  `story:contract-docs` (adds docs tests), `story:routing-invariants` (it changes simulator files
  that do count).
- `docs/contract.md`, `Taskfile.yml`, `AGENTS.md`: `story:contract-derivation`,
  `story:contract-conformance`, `story:routing-invariants`.
- `CHANGELOG.md`: every story of this epic.

## Acceptance

- Given `blocks/` with the bindings of `story:block-bindings`, when `agenticspec sim ports --blocks
  blocks --out blocks/ports.json` runs, the file holds an Outbound port of block `model` whose
  intent event is `agentic.model.InferenceStarted`, waiting state `agentic.model.Inference.Started`,
  reports `[agentic.model.FinishInference]` and protocol `StartInference FinishInference`, and no
  port whose reports include a configuration command (`RegisterPrincipal`, `DefineAgent`,
  `DefineKpi`).
- Given that file, when `agenticspec sim ports --expect crates/agenticspec-sim/tests/cases/ports-expected.yaml`
  runs, it prints one line per difference from the design's 15 rows and exits 1 if there is any;
  its output, verbatim, is written into `epic:architecture-contract` § "Port derivation against
  the design's table", each line marked as fixed in this story, left to an ESS gap, or a
  correction of the design's table.
- Given `blocks/ports.json` committed and one block file changed afterwards, when
  `task check:ports` runs, it exits 1 naming `task ports:generate`; after `task ports:generate`
  it exits 0.
- Given this story's change, when `task check:docs` runs, it exits 0 and
  `git diff --stat website/static/catalog` is empty; the docs test that changes a file under
  `crates/agenticspec-sim/src/contract/` passes.
- Given `spec/cli.yaml` with `sim ports`, when `task check:generated` runs, it exits 0.

## Not in this story

Which parts of an architecture use which port, narrowing settings, contracts
(`story:contract-derivation`); docs sections on architecture pages (`story:contract-docs`).
