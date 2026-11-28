# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- The building blocks declare their cross-block causation as ESS bindings (`blocks/components.yaml`): an admitted gateway call starts its inference, and the authority's decision, approval or rejection decides a harness tool call. `task check:layers` counts bindings among the dependencies, and `task check:blocks` requires a scenario per binding. Causations ESS cannot express yet have reproductions under `design/ess-repro/`
- `agenticspec sim ports` derives the ports of the building blocks, where the specification waits on the world, into `blocks/ports.json`: each port's kind (outbound, clock, principal), intent, reports and protocol. `--check` compares the file's digest with the blocks without calling `ess` (`task check:ports`), `--expect` lists every difference from an expected list
- `agenticspec sim contract <architecture>` derives and checks the contract of an architecture (`agenticspec-contract/1`): the ports its parts use with their protocols, narrowing settings (an agent's `role`, an `on_event` trigger's `event_name` and `source`) and instances, a routing invariant per link (a runtime's `run` and `model_call` links among them) in a small predicate language, and the border; `--check` checks a written contract. `docs/contract.md` describes ports, the contract and the predicate language
- Every architecture page and catalog entry of the docs site has a "Contract" section and its contract file to download (`<name>.contract.yaml`)

## [0.4.1] - 2026-11-28

### Fixed

- Release binaries name no path of the machine that built them: `--remap-path-prefix` maps the runner's home, cargo home and checkout to neutral prefixes on every target, and `scripts/release-paths.sh` fails the packaging job when such a path is left. 0.4.0 was tagged but not published, because its binaries carried the build runner's cargo registry path

### Added

- Packs and fields above the core blocks: four packs named without any product (`packs/source-control`: forge, repository, change request with reviews and head-bound approvals, check runs; `packs/tracking`: issue, board, work queue; `packs/files`: the conceptual file tools `read_file`, `write_file`, `edit_file`, `list_dir`, `search`, `move`, `copy`, `delete`; `packs/shell`: `run_command`), each with its KPIs and authored scenarios, and the field `fields/coding` with the example architecture `auto-merger`. The root `ess-inputs.yaml` validates the whole system; `task check:fields` checks it, its manifests and its conformance suite, and `task check:layers` refuses a core block that depends on a pack or a field, a pack on a field or an undeclared pack, and a field on another field. The docs site has pages for every pack block and field
- `agenticspec sim` simulates more of an architecture: an agent that reaches several model parts splits its calls by its `model_share` and each call is priced by its model's route; a runtime part given runs (a `run` or `order` link, or a trigger) executes their steps, model calls and person steps; an `on_event` trigger fires when the simulation emits its event (`agentic.work.TaskCompleted` and seven more), optionally only for the part its `source` names, and a trigger may wake a runtime
- Scenarios (`agenticspec-scenario/1`) and task corpora (`agenticspec-corpus/1`): `agenticspec sim scenario check|run` checks a scenario against its architecture (border parts, authors, digests) and runs it on the simulator with scripted model outcomes, build times and principal replies, then writes `scenario-result.json` with every expected block event held, failed or not checked and exits 1 on a failure; `agenticspec sim corpus run` runs every entry of a corpus and prints the predicted outcome class beside the expected one; a behaviour profile draws its tasks from a corpus with `load.corpus`. Examples for the workstation in `scenarios/workstation/`, run by `task check:scenarios` in the gate; format on the new docs page `docs/scenarios.md`

### Changed

- `agenticspec sim` reads a part's profile role from its `role` (`coding` when an agent names none); `harness` names the kind of harness only. An agent that answers questions takes its delay from `roles.<role>.answer_seconds`, so renaming it changes no result; a profile with the old `agents` section, a model whose route the profile does not price and a runtime that executes runs without a `role` are refused with the name of what is missing

## [0.4.0] - 2026-11-28

### Added

- The documentation site shows the building blocks of agentic development: a layer map of every block with its dependencies, a page per block with its entities, lifecycles, commands and their refusals, events, views and KPIs, a page per architecture with its check result and rendered graph, and the KPI catalogue as one table to filter and sort. The pages are generated by `task docs:generate` and held by `task check:docs`
- The building blocks of agentic development as a second ESS system, `blocks/` (system `agentic`): 22 blocks in 11 layers, from principals, models and gateways through agents, harnesses, work, authority, channels, triggers and schedules to architectures, KPIs and experiments. `task check:blocks` validates them, refuses a dependency on a higher layer or a cycle inside a layer (`scripts/check-layers.sh`), checks the KPI catalogue and runs 671 conformance scenarios on the ESS interpreter
- `kpis/catalogue.yaml`: 125 KPIs, each with its block, kind (correctness, completeness, quality), direction per work mode, gate, the events it reads and its source (`scripts/check-kpis.sh`)
- `agenticspec sim`, the simulator as a subcommand: checks an architecture document, simulates it with a behaviour profile and searches the places marked for optimization, exhaustively or by hill climbing with restarts; 10 example architectures (`architectures/`) and 6 experiments (`experiments/`)
- The documentation site has a catalog of pre-built, optimized architectures (`/docs/catalog`): one entry per experiment with its full result page and the files to run it again (the architecture as written, the optimized architecture, the experiment, the behaviour profile and `result.json`), and an index of cards with the gain of the search and its trade-off. `agenticspec sim run` writes the best variant as an architecture document, `best.yaml`, with every marked place at its best value and its `vary` kept
- Release archives for macOS (`aarch64-apple-darwin`, `x86_64-apple-darwin`), Windows (`x86_64-pc-windows-msvc`, a `.zip` with `agenticspec.exe`) and static Linux on aarch64 (`aarch64-unknown-linux-musl`), beside the three Linux archives. Every binary's `--version` is checked on its own runner before it is packed, and a pull request from `release/<version>` builds every archive before the tag
- `task check:site`: layout, accessibility, keyboard and screen-only-user gates on the built docs site, with a ratchet in `ux/site/baseline.json`; `task ux:site-round` measures a review round into `ux/site/rounds/`

## [0.3.0] - 2026-11-26

### Changed

- The tool is named agenticspec: the repository `https://github.com/beyond10x/agenticspec` (the old URLs redirect there), the documentation `https://beyond10x.github.io/agenticspec/`, the command `agenticspec`, the crates `agenticspec-records` and `agenticspec-types` (Rust paths `agenticspec_records`, `agenticspec_types`), the specification system `agenticspec`, the release archives `agenticspec-<version>-<target>.tar.gz`. Types, functions and their signatures are unchanged; the store stays version 5
- The store is `~/.local/share/agenticspec/sessions.db`; a store in `~/.local/share/agentspec/` or else `~/.local/share/sessionspec/` is moved there the first time it is opened. The environment variables are `AGENTICSPEC_DELEGATES`, `AGENTICSPEC_UI_DIR` and `AGENTICSPEC_SYNC_MS`; `AGENTSPEC_DELEGATES` and `SESSIONSPEC_DELEGATES` are still read when `AGENTICSPEC_DELEGATES` is not set

## [0.2.0] - 2026-11-26

### Added

- A new live UI, written by hand in Vue over types generated from the specification, with a light and a dark theme: an overview with figures per time slice and a searchable session table, a session page with a timeline per agent, agents, checks, cost and prompt breakdowns and a paged event log, and embed pages; built by `task ui:build` (Node.js 22)
- Views `figures`, `session-timeline` and `session-event-counts`; `session-list` pages by `offset`, `session-events` pages newest first by `before` and filters by `agent`, `kind` and `notable`
- `task check:app`: the app's unit tests and browser tests of layout, accessibility, keyboard use and a screen-only user on the fixtures, with a ratchet in `ux/baseline.json`; `task ux:round` measures a review round into `ux/rounds/`
- A static Linux binary in each release, `sessionspec-<version>-x86_64-unknown-linux-musl.tar.gz`, which needs no glibc of the host; the release workflow checks it is static and runs its `--version` in a plain Alpine container

### Changed

- The repository is `https://github.com/beyond10x/agentspec` and the documentation `https://beyond10x.github.io/agentspec/`; the old repository URL redirects there. The command and the crates keep the name `sessionspec` for now
- The tool is named agentspec: the command `agentspec`, the crates `agentspec-records` and `agentspec-types` (Rust paths `agentspec_records`, `agentspec_types`), the specification system `agentspec`, the release archives `agentspec-<version>-<target>.tar.gz`. Types, functions and their signatures are unchanged
- The store is `~/.local/share/agentspec/sessions.db`; a store in `~/.local/share/sessionspec/` is moved there the first time it is opened. The environment variables are `AGENTSPEC_DELEGATES`, `AGENTSPEC_UI_DIR` and `AGENTSPEC_SYNC_MS`; `SESSIONSPEC_DELEGATES` is still read when `AGENTSPEC_DELEGATES` is not set
- `sessionspec serve` reads every view from tables of the store with one query per request and keeps no rows in memory; on a store of 116 sessions its resident memory measured 84 to 164 MB, where it grew past 1.9 GB before
- `sessionspec serve` runs each sync, the first included, as a `sessionspec sync` child process, one at a time, and never lifts a transcript itself, so the memory of a re-lift goes back to the system when the child ends
- `sync` lifts transcripts in chunks bounded by size and returns freed memory after a sync
- The store is version 5; opening a store of an older version fills its derived tables again and keeps its wire rows
- `sessionspec analyze` without `--out` writes `sessionspec-dashboard.html` beside the store instead of the working directory

## [0.1.1] - 2026-11-26

### Changed

- The documentation site holds user documentation: getting started, dashboard, KPIs, queries, patterns and procedures, privacy and the records library; the design notes moved to `design/` and are no longer published

### Fixed

- Opening a store of another schema version no longer drops its wire rows: only the derived tables are rebuilt, every wire session is projected again from its `wire_steps`, and a store whose wire rows are in a format this build cannot read is refused with an error and left unchanged
- A long wire session no longer grows without bound: a `WireSession` keeps the 4,096 most recent response ids, conversation ids and spawn inputs (`LINKS_HELD`); a call whose link was evicted starts an agent without a parent
- `AppenderHandle::append` documents that an empty batch returns true and queues nothing

## [0.1.0] - 2026-11-26

### Added

- An ESS specification of the session IR (`sessionspec.session`), the command line (`sessionspec.cli`) and the read models of the UI (`sessionspec.view`), with generated types, components and command-line parser
- A Claude Code adapter that lifts a session and its sub-agents into IR commands, each prompt with an author and an attribution (`Known`, `Inferred`, `NotRecorded`)
- Redaction of secrets and personal data into aliases; a finding names its class and count, never the value
- KPIs over the published events: models, context length, cache hits and busts, concurrency, autonomy, permission decisions and API list-price cost
- `sessionspec sync`: an incremental SQLite store of the redacted records, KPIs and view rows; `query`, `mine`, `procedure`, `kpi` and `analyze` read the store
- `sessionspec serve`: the views of the UI from the store on 127.0.0.1, the built UI and server-sent events
- The sq query language (typed Datalog with sequence, aggregates and certainty) and the checks of `queries/session.sq`
- Pattern mining of recurring tool-call runs, and the export of a mined procedure as a canon `protocol/1` file with loom bindings
- The UI as an `ess-ui/1` document with synthetic fixtures and headless tests
- `sessionspec-records`, the library an LLM gateway links: the wire lift of one finished call, the session hash, the join with a transcript by message id, and a non-blocking appender
- `SESSIONSPEC_DELEGATES` names the sessions whose messages count as a delegate's prompt
- Releases: the `sessionspec` binary for x86_64 and aarch64 Linux as archives with `SHA256SUMS`, built by the tag workflow
