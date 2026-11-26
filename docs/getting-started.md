# Getting started

## Install

You need a Rust toolchain. Install the binary from the repository:

```console
cargo install --locked --git https://github.com/beyond10x/sessionspec sessionspec
```

## Fill the store

```console
sessionspec sync
```

This reads every Claude Code session with a file changed in the last 72 hours and writes it to the
store. It prints how many sessions it read, lifted and skipped. The first run reads everything in
the window. Later runs only read what changed.

Choose another window or another transcript directory:

```console
sessionspec sync --since-hours 168
sessionspec sync --root /path/to/projects
```

You rarely need to run `sync` yourself. `analyze`, `kpi`, `query`, `mine`, `procedure` and `serve`
sync first.

## Open the dashboard

```console
sessionspec serve
```

Open http://127.0.0.1:7420/. The server listens on 127.0.0.1 only and syncs every 5 seconds, so
running sessions show up while they work. Use `--port` to pick another port.

The live UI is a web app that is built from a checkout of the repository with `task ui:build`.
`serve` finds it in `target/ui/www` beside the binary, in the directory named by
`SESSIONSPEC_UI_DIR`, or in `ui/` inside the state directory. Without it, `serve` still answers the
data endpoints and says the UI is not built.

## Write a dashboard file

```console
sessionspec analyze --out sessions.html
```

This writes one self-contained HTML file: the KPIs of the window and each session. Without `--out`
it writes `sessionspec-dashboard.html` beside the store, so a run inside a checkout leaves nothing
there. The file contains your session titles, so keep it out of repositories you publish.

## Where your data lives

| what | where |
|---|---|
| transcripts read | `~/.claude/projects`, or `--root` |
| the store | `$XDG_DATA_HOME/sessionspec/sessions.db`, else `~/.local/share/sessionspec/sessions.db` |
| another store | `--state-dir <dir>` on any command keeps the store in `<dir>/sessions.db` |
| the dashboard file | `sessionspec-dashboard.html` beside the store, unless `analyze --out` names another |

A new version of sessionspec may change what the store holds. It then empties the store and fills
it again on the next sync.

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
export SESSIONSPEC_DELEGATES=operator,night-shift
```

A message from a named session counts as a delegate's prompt, not a peer session's. A message that
says it was sent on someone's order also counts as a delegate's.
