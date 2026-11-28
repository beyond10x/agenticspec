# agenticspec

agenticspec lifts the transcripts of an agent harness into one specified IR of actors, commands,
decisions and events, redacts it, stores it in a local SQLite file, and evaluates KPIs, queries and
mined patterns over that store. It reads Claude Code transcripts today. An LLM gateway can append
the same records from the wire through the `agenticspec-records` library.

Documentation: <https://beyond10x.github.io/agenticspec/>, from the pages in `docs/`:
[getting started](docs/getting-started.md), [dashboard](docs/dashboard.md), [KPIs](docs/kpis.md),
[queries](docs/queries.md), [patterns and procedures](docs/patterns.md), [privacy](docs/privacy.md),
[records library](docs/library.md). Design notes for contributors are in `design/`.

Field names and formats may still change between 0.x releases.

## Install

Each [release](https://github.com/beyond10x/agenticspec/releases) carries one archive per
platform with the `agenticspec` binary, `LICENSE` and this file:

| platform | archive |
|---|---|
| Linux x86_64, static (any distribution) | `agenticspec-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64, static (any distribution) | `agenticspec-<version>-aarch64-unknown-linux-musl.tar.gz` |
| Linux x86_64, glibc | `agenticspec-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux aarch64, glibc | `agenticspec-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS, Apple silicon | `agenticspec-<version>-aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `agenticspec-<version>-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `agenticspec-<version>-x86_64-pc-windows-msvc.zip` (`agenticspec.exe`) |

Download an archive and `SHA256SUMS` from the same release into one directory and check the archive
before you unpack it (on macOS, `shasum -a 256 -c --ignore-missing SHA256SUMS`; on Windows, compare
`Get-FileHash <archive>` with its line in `SHA256SUMS`):

```console
sha256sum --check --ignore-missing SHA256SUMS
tar -xzf agenticspec-<version>-<target>.tar.gz
```

On Windows, set `HOME` (for example to `%USERPROFILE%`) or pass `--root` and `--state-dir`: the
default paths below start at `HOME`.

Or build from source with the Rust toolchain pinned in `rust-toolchain.toml`:

```console
cargo install --locked --path crates/agenticspec
```

## Use

```console
agenticspec sync                   # lift new and changed sessions of the last 72 hours into the store
agenticspec serve                  # the dashboard on http://127.0.0.1:7420/, synced every 5 seconds
agenticspec analyze                # write a self-contained dashboard file beside the store
agenticspec kpi --output json      # the KPIs over the window
agenticspec query --text '<sq>'    # an sq module over the sessions of the window
agenticspec mine                   # recurring runs of tool calls across sessions
```

The commands above read `~/.claude/projects` unless `--root` names another directory, and covers the
last 72 hours unless `--since-hours` says otherwise. The store is
`$XDG_DATA_HOME/agenticspec/sessions.db` (`~/.local/share/agenticspec/sessions.db`), or the
`--state-dir` you give. `agenticspec serve` needs the UI built once with `task ui:build` (Node.js 22).
The UI has a light and a dark theme and pages for the overview, a session and embedding a session
in another page: see the dashboard page of the documentation.

`AGENTICSPEC_DELEGATES` names, comma-separated, the sessions that act for you; a message from one
of them counts as a delegate's prompt, not a peer's.

`agenticspec sim` is the simulator. It reads no session and no store: it checks an architecture
document, runs an experiment (each entry of the
[catalog](https://beyond10x.github.io/agenticspec/docs/catalog) has the files to download), and
writes a page of an architecture:

```console
agenticspec sim check <architecture.yaml>
agenticspec sim check --json <architecture.yaml>   # agenticspec-check/1 on stdout, also when refused
agenticspec sim run <experiment.yaml> --out out/   # result.json, index.html, best.yaml
agenticspec sim render <architecture.yaml> --out page.html
```

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
