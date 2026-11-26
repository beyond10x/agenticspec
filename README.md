# sessionspec

sessionspec lifts the transcripts of an agent harness into one specified IR of actors, commands,
decisions and events, redacts it, stores it in a local SQLite file, and evaluates KPIs, queries and
mined patterns over that store. It reads Claude Code transcripts today. An LLM gateway can append
the same records from the wire through the `sessionspec-records` library.

Documentation: <https://beyond10x.github.io/agentspec/>, from the pages in `docs/`:
[getting started](docs/getting-started.md), [dashboard](docs/dashboard.md), [KPIs](docs/kpis.md),
[queries](docs/queries.md), [patterns and procedures](docs/patterns.md), [privacy](docs/privacy.md),
[records library](docs/library.md). Design notes for contributors are in `design/`.

Field names and formats may still change between 0.x releases.

## Install

Each release on GitHub carries a static Linux binary that runs on any x86_64 Linux,
`sessionspec-<version>-x86_64-unknown-linux-musl.tar.gz`, beside builds for glibc on x86_64 and
aarch64. Or build from source with the Rust toolchain pinned in `rust-toolchain.toml`:

```console
cargo install --locked --path crates/sessionspec
```

## Use

```console
sessionspec sync                   # lift new and changed sessions of the last 72 hours into the store
sessionspec serve                  # the dashboard on http://127.0.0.1:7420/, synced every 5 seconds
sessionspec analyze                # write a self-contained dashboard file beside the store
sessionspec kpi --output json      # the KPIs over the window
sessionspec query --text '<sq>'    # an sq module over the sessions of the window
sessionspec mine                   # recurring runs of tool calls across sessions
```

The commands above read `~/.claude/projects` unless `--root` names another directory, and covers the
last 72 hours unless `--since-hours` says otherwise. The store is
`$XDG_DATA_HOME/sessionspec/sessions.db` (`~/.local/share/sessionspec/sessions.db`), or the
`--state-dir` you give. `sessionspec serve` needs the UI built once with `task ui:build` (Node.js 22).
The UI has a light and a dark theme and pages for the overview, a session and embedding a session
in another page: see the dashboard page of the documentation.

`SESSIONSPEC_DELEGATES` names, comma-separated, the sessions that act for you; a message from one
of them counts as a delegate's prompt, not a peer's.

## What stays on your machine

The store holds redacted records: secrets and personal data are replaced by aliases, and a
finding names its class and count, never the value. It also holds the harness's session titles,
which come from prompt text, so it stays local. Nothing is sent anywhere.

## Develop

`task check` is the gate: the specification, the UI document and its tests, the app in `ui/app`
(types, unit tests, and layout, accessibility, keyboard and screen-only-user tests in a browser,
with a ratchet on their metrics), the generated crates, the script tests, format, lints and tests.
`task ux:round -- R-nn` measures a round of UI review into `ux/rounds/`. `task site` builds the
documentation site. `AGENTS.md` holds the rules for changes.

## License

Apache-2.0. See `LICENSE`.
