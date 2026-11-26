# Dashboard

sessionspec shows your sessions in two ways: a live web UI and a single HTML file.

## Live UI

```console
sessionspec serve
```

Open http://127.0.0.1:7420/.

**Overview.** The KPIs of the time window you choose, averaged over its sessions, and the most
recent sessions in a table. Each row shows the project, the title, cost, output tokens, cache hit
share, cache busts, prompts, tool calls, refused calls and the share of time spent waiting on you.
The search box filters by project, title, model names and tool names. Case does not matter.

**Session.** One session in detail: its agents and sub-agents, the events of each, the model turns
with their token counts, and the result of each check from [queries](queries.md).

The UI updates while sessions run. New sessions and new events arrive over server-sent events.

### Embedding a session

A host program, such as an LLM gateway dashboard, can show one session without the overview. It
opens the embed pages with the session id in the path. They show the session view, its timeline or
its event log, without navigation.

### Data endpoints

The UI reads JSON views that you can also query yourself:

```console
curl 'http://127.0.0.1:7420/view/views/session-list?search=refactor&limit=20'
```

| path | returns |
|---|---|
| `/view/views/session-list?since&search&limit` | the sessions, newest first |
| `/view/views/session?session_id=<id>` | one session |
| `/view/views/session-agents?session_id=<id>` | its agents |
| `/view/views/session-events?session_id=<id>` | its events |
| `/view/views/session-turns?session_id=<id>` | its model turns |
| `/view/views/agent-turns?session_id=<id>&agent=<agent>` | one agent's model turns |
| `/view/views/session-checks?session_id=<id>` | the check results |
| `/view/channels/sessions` | a stream of session updates |
| `/view/channels/session_events?session_id=<id>` | a stream of one session's new events |

## Dashboard file

```console
sessionspec analyze --out sessions.html
```

Without `--out`, the file is `sessionspec-dashboard.html` beside the store, in the same directory
as `sessions.db`. The file opens offline in any browser. It holds the KPIs of the window and, per
session:

- models with messages, output tokens, context size and cost
- cache busts by cause
- tool calls by category
- prompts to the main agent by author
- permission decisions of the main agent
- secrets and personal data found, by kind and count

The file contains session titles, which come from your prompts. Treat it like the transcripts.
