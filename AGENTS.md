# AGENTS.md — sessionspec

sessionspec lifts agent sessions of any harness into one IR (actors, commands, decisions, events,
state over time), redacts it into a persistable form, and evaluates KPIs on it. Design:
`docs/design.md`. Baseline numbers: `research/`.

Organization-wide rules live in the workspace `AGENTS.md` (in the directory that holds the
organization's repositories side by side); this file adds what is specific to sessionspec.

This repository is public. No person's name, e-mail address, host path, private repository,
session name, employer or customer goes into a tracked file, a commit message or a fixture. Use
"the owner" for the person who decides.

## Serves

- **O3 — any harness, observed and compared.** A session of any harness is lifted into one IR
  (actors, commands, decisions, events), so runs are recorded in one vocabulary and compared on the
  same KPIs.
- **O6 — self-improvement, built into all of it.** KPIs, sq checks and mined patterns over the
  store say where agent work loses time, tokens or the owner's attention, and re-measure after a
  change.
- **O2 — decisions as data, with evidence.** Every permission decision and every prompt carries
  who made it and how sure the record is (`Known`, `Inferred`, `NotRecorded`).

## Phase: implementation

Opened 2026-11-26 by the owner ("I think you can now already start that").

## Specification first (ESS)

- Every noun is declared in `spec/` before code uses it; `task check:spec` passes first.
- `spec/domains/session.yaml` is the session IR. `spec/domains/cli.yaml` and the `ess-cli/2`
  binding `spec/cli.yaml` are the command line, including the typed KPI report.
- Generated, never edited by hand, checked byte for byte by `task check:generated`:
  - `generated/`: `ess generate synthesize --target rust` (IR types, command behaviours, system,
    wire codecs). Its own workspace, excluded from the root one.
  - `crates/sessionspec-cli-contract/`: `ess generate cli`.
- After a change of `spec/`: `task generate`, then fix what the compiler names.
- `ui/sessionspec.ui.yaml` is the UI as an `ess-ui/1` document over the views of
  `sessionspec.view`, which the `session-view` component serves; `task check:ui` checks it against
  `spec/` and runs `ui/tests/` on the fixtures in `ui/fixtures/` (format: `ui/README.md`).

## Layout of the hand-written crates

`crates/sessionspec-records` is the library the gateway links (`docs/interaction-records.md`);
`crates/sessionspec` is the command line over it and re-exports its modules.

| crate | module | what |
|---|---|---|
| `sessionspec-records` | `claude.rs` | Claude Code adapter: transcript files to IR commands, with actor and attribution |
| `sessionspec-records` | `redact.rs` | secret and personal-data detection; aliases, never values |
| `sessionspec-records` | `replay.rs` | storage ports and the replay loop through the generated component |
| `sessionspec-records` | `kpi.rs` | KPIs over published events only |
| `sessionspec-records` | `store.rs` | the local store `<state-dir>/sessions.db` (SQLite): `sync`, stored commands, view rows |
| `sessionspec-records` | `wire.rs` | the wire source: one finished gateway call to IR commands (`lift_call`), the session hash, `join` on the message id |
| `sessionspec-records` | `append.rs` | the non-blocking writer a gateway appends through (`Appender::spawn`, a bounded queue, drops counted) |
| `sessionspec-records` | `view.rs`, `sq/`, `pricing.rs`, `time.rs` | view projection, the sq engine, API list prices, time helpers |
| `sessionspec` | `mine.rs`, `export.rs` | pattern mining, procedure export to canon |
| `sessionspec` | `serve.rs` | `sessionspec serve`: the `session-view` views from the store, the built UI, SSE channels |
| `sessionspec` | `html.rs`, `assets/dashboard.html` | the dashboard file `sessionspec analyze` writes |
| `sessionspec` | `main.rs` | handler behind the generated CLI |

Beside the crate: `ui/` holds the UI (`sessionspec.ui.yaml`), its synthetic fixtures, its
`ess-ui-test/1` tests and the host components (`ui/host/`); renderers are generated from it
(`ess generate ui`), never written here. `task ui:build` builds the React app into `target/ui/www`.

## Repository files

| path | what |
|---|---|
| `README.md`, `LICENSE`, `CHANGELOG.md` | for humans; Apache-2.0; the changelog in the Keep a Changelog 1.1.0 form, `## [Unreleased]` first. A change a user of the repository notices writes its line there |
| `scripts/check-generated.sh` | one line per generated directory, the drift check and the regeneration |
| `scripts/ui-build.sh` | `task ui:build` |
| `scripts/change-class.sh`, `change-class.test.sh` | the change classifier of `pages.yml` (planning, docs-site, code) and its cases, run by `task check:scripts`. A new site source goes into its path list with a case |
| `website/` | the public documentation site at `https://beyond10x.github.io/sessionspec/` (Docusaurus on `@beyond10x/docs-system`, pinned by commit in `package.json`). Its pages are `docs/*.md`, `research/*.md` and `ui/README.md`, read in place; `docusaurus.config.ts` gives each its route, title and header (`pages`). `product.json` is the landing page. A new page needs an entry in `pages` and in `sidebars.ts`. A Markdown link to a file the site does not publish fails the build, so link such files by GitHub URL. `task site` builds it |
| `.github/workflows/pages.yml` | `Documentation validation`: classifies the changed files with `scripts/change-class.sh`, builds the site, refuses a machine path in it, and on `main` uploads the artifact `b10x-project-site`. Its job `gate` runs `task check` for the class `code`, with `task`, `cargo-nextest` and `ess` downloaded at a pinned version and refused unless their SHA-256 is the one in the step |
| `.github/workflows/b10x-docs-site.yml` | publishes that artifact to GitHub Pages through `beyond10x/website`'s `project-site.yml`, pinned by commit, for bot-authored pushes to `main` |
| `.github/workflows/shared-gates.yml` | the organization's shared source gates (`beyond10x/gates`, pinned by commit) |

## Local store

- `sessionspec sync` lifts each main transcript of the window whose files changed (size and
  modification time of the transcript and its `subagents/` files) and writes, in one
  transaction per session: the redacted commands (zstd), the `sessionspec.view` rows (events
  zstd), the session's KPIs and its check verdicts (`queries/session.sq`).
- `query`, `mine`, `procedure`, `kpi` and `analyze` sync first and replay the stored commands;
  `query --transcript` reads the file.
- The store is `$XDG_DATA_HOME/sessionspec/sessions.db` (or `--state-dir`), never inside this
  repository: it holds the harness's session titles. `SCHEMA_VERSION` in `store.rs` rises when
  what it holds changes meaning; a store of another version is emptied and filled again.
- `sessionspec serve` binds 127.0.0.1 only and answers as the one local `Viewer` (`ui/README.md`).

## Rules

- Raw transcripts and unredacted IR never enter this repository. Only redacted IR and aggregate
  numbers do. Test fixtures are synthetic.
- A finding about secrets or personal data names the class and the count, never the value.
- KPIs read the events the generated component published, never a transcript.
- An author or a decision the transcript does not record is `Inferred` or `NotRecorded`, never
  guessed as `Known`.
- `GatewayFacts`, `WireCall`, `Wire`, `Answer` and the signatures of `lift_call`,
  `Appender::spawn` and `AppenderHandle::append` are a contract with llm-gateway (its
  interaction-records epic). Change a name or a signature only after telling the gateway's
  maintainers, and with a new tag.
- The sessions that act for the owner are named at run time (`SESSIONSPEC_DELEGATES`), never in
  code or fixtures; a fixture uses a neutral name such as `operator`.
- `shellspec` (the sibling repository) will lift the input of an `Execute` tool call into its
  sub-commands (the owner, 2026-11-26: "if we're getting somewhere with the shell spec thing, then
  we're using it in session spec").

## Commits

Every commit and push is `b10x-bot[bot]`'s through `b10x-gates bot --repository
beyond10x/sessionspec -- <git command>`; every other GitHub write goes through `b10x-gates api`,
with the gates policy file of the primary checkout. Every commit has a body. A tag and a release are
publications and wait for the owner's order.
