# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
