# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- A new live UI, written by hand in Vue over types generated from the specification, with a light and a dark theme: an overview with figures per time slice and a searchable session table, a session page with a timeline per agent, agents, checks, cost and prompt breakdowns and a paged event log, and embed pages; built by `task ui:build` (Node.js 22)
- Views `figures`, `session-timeline` and `session-event-counts`; `session-list` pages by `offset`, `session-events` pages newest first by `before` and filters by `agent`, `kind` and `notable`
- `task check:app`: the app's unit tests and browser tests of layout, accessibility, keyboard use and a screen-only user on the fixtures, with a ratchet in `ux/baseline.json`; `task ux:round` measures a review round into `ux/rounds/`
- A static Linux binary in each release, `sessionspec-<version>-x86_64-unknown-linux-musl.tar.gz`, which needs no glibc of the host; the release workflow checks it is static and runs its `--version` in a plain Alpine container

### Changed

- The repository is `https://github.com/beyond10x/agentspec` and the documentation `https://beyond10x.github.io/agentspec/`; the old repository URL redirects there. The command and the crates keep the name `sessionspec` for now
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
