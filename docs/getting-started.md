# Getting started

## Install

Each [release](https://github.com/beyond10x/agenticspec/releases) carries the binary for Linux
(x86_64 and aarch64, static or for glibc), macOS (Apple silicon and Intel) and Windows (x86_64),
one archive per platform, with `SHA256SUMS` to check it against.

With a Rust toolchain, install the binary from the repository instead:

```console
cargo install --locked --git https://github.com/beyond10x/agenticspec agenticspec
```

## Fill the store

```console
agenticspec sync
```

This reads every Claude Code session with a file changed in the last 72 hours and writes it to the
store. It prints how many sessions it read, lifted and skipped. The first run reads everything in
the window. Later runs only read what changed.

Choose another window or another transcript directory:

```console
agenticspec sync --since-hours 168
agenticspec sync --root /path/to/projects
```

You rarely need to run `sync` yourself. `analyze`, `kpi`, `query`, `mine`, `procedure` and `serve`
sync first.

## Open the dashboard

```console
agenticspec serve
```

Open http://127.0.0.1:7420/. The server listens on 127.0.0.1 only and syncs every 5 seconds, so
running sessions show up while they work. Use `--port` to pick another port.

The live UI is a web app that is built from a checkout of the repository with `task ui:build`,
which needs Node.js 22 and npm.
`serve` finds it in `target/ui/www` beside the binary, in the directory named by
`AGENTICSPEC_UI_DIR`, or in `ui/` inside the state directory. Without it, `serve` still answers the
data endpoints and says the UI is not built.

## Write a dashboard file

```console
agenticspec analyze --out sessions.html
```

This writes one self-contained HTML file: the KPIs of the window and each session. Without `--out`
it writes `agenticspec-dashboard.html` beside the store, so a run inside a checkout leaves nothing
there. The file contains your session titles, so keep it out of repositories you publish.

## Where your data lives

| what | where |
|---|---|
| transcripts read | `~/.claude/projects`, or `--root` |
| the store | `$XDG_DATA_HOME/agenticspec/sessions.db`, else `~/.local/share/agenticspec/sessions.db` |
| another store | `--state-dir <dir>` on any command keeps the store in `<dir>/sessions.db` |
| the dashboard file | `agenticspec-dashboard.html` beside the store, unless `analyze --out` names another |

A new version of agenticspec may change what the store holds. It then empties the store and fills
it again on the next sync.

A store from before the name agenticspec, in `~/.local/share/agentspec/` or
`~/.local/share/sessionspec/`, is moved to `~/.local/share/agenticspec/` the first time
agenticspec opens it.

## Common options

`sync`, `serve`, `analyze`, `kpi`, `query`, `mine` and `procedure` take these:

| option | effect |
|---|---|
| `--since-hours <n>` | the window: from `n` hours ago until now, 72 by default |
| `--root <dir>` | the Claude Code projects directory |
| `--state-dir <dir>` | the directory of the store |
| `--output json` | print the result as JSON |

## Sessions that act for you

Some setups let one agent session type into another on your behalf. Name those sessions so their
messages count as yours:

```console
export AGENTICSPEC_DELEGATES=operator,night-shift
```

A message from a named session counts as a delegate's prompt, not a peer session's. A message that
says it was sent on someone's order also counts as a delegate's.

The older names `AGENTSPEC_DELEGATES` and `SESSIONSPEC_DELEGATES` are still read when
`AGENTICSPEC_DELEGATES` is not set.
