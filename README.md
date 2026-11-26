# sessionspec

sessionspec lifts the transcripts of an agent harness into one specified IR of actors, commands,
decisions and events, redacts it, stores it in a local SQLite file, and evaluates KPIs, queries and
mined patterns over that store. It reads Claude Code transcripts today. An LLM gateway can append
the same records from the wire through the `sessionspec-records` library.

Documentation: <https://beyond10x.github.io/sessionspec/>

There is no release yet. Field names and formats may change until the first one.

## Install

Build from source with the Rust toolchain pinned in `rust-toolchain.toml`:

```console
cargo install --locked --path crates/sessionspec
```

## Use

```console
sessionspec sync                   # lift new and changed sessions of the last 72 hours into the store
sessionspec serve                  # the dashboard on http://127.0.0.1:7420/, synced every 5 seconds
sessionspec analyze                # write a self-contained dashboard file
sessionspec kpi --output json      # the KPIs over the window
sessionspec query --text '<sq>'    # an sq module over the sessions of the window
sessionspec mine                   # recurring runs of tool calls across sessions
```

The commands above read `~/.claude/projects` unless `--root` names another directory, and covers the
last 72 hours unless `--since-hours` says otherwise. The store is
`$XDG_DATA_HOME/sessionspec/sessions.db` (`~/.local/share/sessionspec/sessions.db`), or the
`--state-dir` you give. `sessionspec serve` needs the UI built once with `task ui:build`.

`SESSIONSPEC_DELEGATES` names, comma-separated, the sessions that act for you; a message from one
of them counts as a delegate's prompt, not a peer's.

## What stays on your machine

The store holds redacted records: secrets and personal data are replaced by aliases, and a
finding names its class and count, never the value. It also holds the harness's session titles,
which come from prompt text, so it stays local. Nothing is sent anywhere.

## Develop

`task check` is the gate: the specification, the UI document and its tests, the generated crates,
the script tests, format, lints and tests. `task site` builds the documentation site. `AGENTS.md`
holds the rules for changes.

## License

Apache-2.0. See `LICENSE`.
