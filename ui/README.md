# sessionspec UI

`sessionspec.ui.yaml` is the UI as an `ess-ui/1` document. It reads the views of
`sessionspec.view` (`spec/domains/view.yaml`), which the `session-view` component serves, and takes
live events on two channels. Renderers are generated from it, never written here:

```console
ess ui check --path ui/sessionspec.ui.yaml --model spec
ess ui test --path ui/sessionspec.ui.yaml ui/tests/session.yaml
ess ui run --tui --path ui/sessionspec.ui.yaml
ess generate ui --target react --path ui/sessionspec.ui.yaml --out <dir>
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

## Served from the local store

```console
task ui:build          # generate the React app with --model spec, build it into target/ui/www
sessionspec serve      # http://127.0.0.1:7420/, the store synced every 5 seconds
```

`sessionspec serve` answers the views of `session-view` (`GET /view/views/<view>`, the paths
`ess generate ui --model` binds to) from `<state-dir>/sessions.db`, serves `target/ui/www` (or
`$SESSIONSPEC_UI_DIR`) for every other path, and streams the two channels as server-sent events:
`GET /view/channels/sessions` and `GET /view/channels/session_events?session_id=<id>`, one message
per event (`event:` its qualified name, `data:` its wire JSON, `id:` the `seq` of an
`EventAppended`; a reconnect with `Last-Event-ID` gets what followed). The React runtime does not
open a channel against a served surface yet: its live sections read again every 5 seconds.

Sign-in: the pages say `actor: from_session`, and the views grant `sessionspec.view.Viewer`. The
local server listens on 127.0.0.1 only and answers every request as that one `Viewer`; it reads no
`Authorization` header. A host that serves the views to more than its own user authenticates the
caller before `dispatch` instead.

## Fixtures

`fixtures/index.yaml` names one file per view and one script per channel. The files are JSON (YAML
reads them). Synthetic data only: no session of `~/.claude`, no content.

| file | view | rows |
|---|---|---|
| `sessions.json` | `sessionspec.view.SessionList` | one per session; `SessionById` is answered from it |
| `agents.json` | `sessionspec.view.SessionAgents` | one per agent of every session |
| `events.json` | `sessionspec.view.SessionEvents` | one per view event of every session |
| `turns.json` | `sessionspec.view.SessionTurns` | the rows of `events.json` with `kind` `model_turn`; `AgentTurns` is answered from it |
| `checks.json` | `sessionspec.view.SessionChecks` | one per check of every session |

A view file is

```json
{"view": "sessionspec.view.SessionList", "rows": [{"session_id": "...", "project": "...", "...": 0}]}
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
 "events": [{"at": "2s", "event": "sessionspec.view.EventAppended", "payload": {"event_id": "...", "...": 0}},
            {"at": "10s", "lifecycle": "reconnecting"}]}
```

`session` is required on `session_events` (the channel has one instance per `session_id`); `at`
is the offset from the start of the script; a payload is the event's fields, which are the fields
of the row it adds or replaces. A live source (the gateway) sends the same payloads on the
channel.
