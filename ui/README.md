# agenticspec UI

`agenticspec.ui.yaml` is the UI as an `ess-ui/1` document. It reads the views of
`agenticspec.view` (`spec/domains/view.yaml`), which the `session-view` component serves, and takes
live events on two channels. It is checked against the specification and tested headless; the live
UI is not generated from it but written by hand in `app/` (see below):

```console
ess ui check --path ui/agenticspec.ui.yaml --model spec
ess ui test --path ui/agenticspec.ui.yaml ui/tests/session.yaml
ess ui run --tui --path ui/agenticspec.ui.yaml
```

## Pages

| page | shell | what |
|---|---|---|
| `overview` | `app` | KPI metrics over all sessions, and the sessions table |
| `session` | `app` | one session: page kind `session_view` |
| `embed.session` | `embed` | the same view in a frame with nothing but the page, plus the host slot `SessionLanes` |
| `embed.timeline` | `embed` | metrics, the session's cost over time, agents with a context and a cost chart each |
| `embed.events` | `embed` | metrics, lanes by event kind, event log |

Every session page takes the path parameter `session_id`; each of its reads and its channel are
narrowed by it. A host that shows one session (the LLM gateway UI) opens an `embed.*` page.

## The app

`app/` is the live UI: Vue 3, Vite and TypeScript, written by hand over the types of
`app/src/generated/` (`ess generate types`). `app/src/testing/fixtures.ts` answers its views from
`fixtures/` for the unit tests and the browser tests of `app/e2e/` (`task check:app`).

## The design system and the studio app

`kit/` is the design system every generated page draws with: tokens taken from the docs site, a
light and a dark theme, and the `Kit*` components; its gallery shows them all (`task check:ui-kit`).
`studio/` is the agenticstudio app, generated from `../studio/studio.ui.yaml` by `agenticspec ui
generate` into `studio/src/generated/` (`task ui:generate`), with its host slots in
`studio/src/slots/`. Both share `package.json` here; `docs/studio.md` describes the app.

## Served from the local store

```console
task ui:build          # build the app of ui/app into target/ui/www
agenticspec serve      # http://127.0.0.1:7420/, the store synced every 5 seconds
```

`agenticspec serve` answers the views of `session-view` (`GET /view/views/<view>`) from
`<state-dir>/sessions.db`, serves `target/ui/www` (or `$AGENTICSPEC_UI_DIR`) for every other path, and streams the two channels as server-sent events:
`GET /view/channels/sessions` and `GET /view/channels/session_events?session_id=<id>`, one message
per event (`event:` its qualified name, `data:` its wire JSON, `id:` the `seq` of an
`EventAppended`; a reconnect with `Last-Event-ID` gets what followed).

Sign-in: the pages say `actor: from_session`, and the views grant `agenticspec.view.Viewer`. The
local server listens on 127.0.0.1 only and answers every request as that one `Viewer`; it reads no
`Authorization` header. A host that serves the views to more than its own user authenticates the
caller before `dispatch` instead.

## Fixtures

`fixtures/index.yaml` names one file per view and one script per channel. The files are JSON (YAML
reads them). Synthetic data only: no session of `~/.claude`, no content.

| file | view | rows |
|---|---|---|
| `sessions.json` | `agenticspec.view.SessionList` | one per session; `SessionById` is answered from it |
| `agents.json` | `agenticspec.view.SessionAgents` | one per agent of every session |
| `events.json` | `agenticspec.view.SessionEvents` | one per view event of every session |
| `turns.json` | `agenticspec.view.SessionTurns` | the rows of `events.json` with `kind` `model_turn`; `AgentTurns` is answered from it |
| `checks.json` | `agenticspec.view.SessionChecks` | one per check of every session |

A view file is

```json
{"view": "agenticspec.view.SessionList", "rows": [{"session_id": "...", "project": "...", "...": 0}]}
```

and a row holds the fields of the view as `spec/domains/view.yaml` declares them, by their names:
a `Timestamp` is RFC 3339 text in UTC, a `Binary64` a JSON number, an enum its variant
(`ClaudeCode`, `Main`, `model_turn`), an absent `Optional` field is left out. `event_id` is
`<session_id>.<seq>`, with a dot: a row key is one segment of a node path, which `/` separates.
A renderer narrows rows to a read's parameters by the field of the same name (`session_id`,
`agent`), so a file holds the rows of every session.

| script | channel | entries |
|---|---|---|
| `scripts/sessions.json` | `sessions` | `SessionUpdated` |
| `scripts/session_events.json` | `session_events` | `EventAppended`, `AgentUpdated`, `SessionUpdated` of one session |

A script is

```json
{"channel": "session_events", "session": {"session_id": "..."}, "loop": true,
 "events": [{"at": "2s", "event": "agenticspec.view.EventAppended", "payload": {"event_id": "...", "...": 0}},
            {"at": "10s", "lifecycle": "reconnecting"}]}
```

`session` is required on `session_events` (the channel has one instance per `session_id`); `at`
is the offset from the start of the script; a payload is the event's fields, which are the fields
of the row it adds or replaces. A live source (the gateway) sends the same payloads on the
channel.

## Block panels

`views/` holds the panels of the building blocks, generated from `blocks/` and never edited by
hand:

```console
agenticspec ui panels           # task ui:panels: write views/
agenticspec ui panels --check   # held by task check:ui
ess ui check --path ui/views/blocks.ui.yaml --model blocks
ess ui run --tui --path ui/views/blocks.ui.yaml
```

| file | what |
|---|---|
| `views/blocks.ui.yaml` | an `ess-ui/1` document over the system `agentic`: one widget per view, named as the view (`agentic.agent.Agents`), and a gallery page per block that places each of its widgets |
| `views/fixtures/` | three synthetic rows per view, in the format above, and one script per block channel that plays each event once |
| `views/blocks.sources.sha256` | the SHA-256 of every input: `ess --version`, `blocks/ess-inputs.yaml` and each file it lists, and the generator's sources |

A panel is a `collection` of its view's rows: one column per field, the first field as row key,
an enum (the `state` among them) as a badge, a time as a timestamp. It takes a scope, `source`,
`architecture`, `part` and `instance`, which no read binds: the runtime that answers the read
narrows the rows. A view parameter is a required parameter of the panel. When events change the
view's rows, the panel takes them on the channel its page names (`channel`): it inserts or patches
a row when each event carries every field of the view, and re-reads the view otherwise.

`views/gallery/` draws the panels: `agenticspec ui generate` writes one Vue component per widget
(`src/generated/widgets/`, the same file for the same widget in any document), a page per block and
the navigation of the layers into `src/generated/` (`task ui:generate`, held by
`task check:generated`), and `src/fixtures.ts` answers every read from `views/fixtures/` and plays
its channel scripts in the browser, so the gallery needs no server (`?still` keeps it from playing).
A widget reads through the client of its scope: the adapter at `source`, else the app's own under
`/scopes/<architecture>[/<part>[/<instance>]]`. `task views:build` builds it into
`target/views/gallery`; `task check:views` runs its unit tests and its browser test.
