# Dashboard

agenticspec shows your sessions in two ways: a live web UI and a single HTML file.

## Live UI

```console
agenticspec serve
```

Open http://127.0.0.1:7420/. The button at the top right switches between the light and the dark
theme; the choice is kept in the browser. The dot beside it says whether updates arrive.

### Overview

- **Time slice.** 24 h, 72 h, 7 d or 30 d. Every figure and the table cover the sessions active in
  that slice.
- **Figures.** The number of sessions, cost per session, agent working time per session, the share
  of wall time spent waiting on you, the cache hit share and tool calls per session. Each shows the
  change against the slice before when that slice holds enough sessions, and a small trend line.
- **Search.** Finds sessions by project, title, model names and tool names. Case does not matter.
- **Most recent.** The sessions of the slice, the last active first, 100 at a time: title, project,
  when it was last active, wall time, agents, tool calls and refused calls, cache hits, waiting
  share and cost. Click a row, or press Enter on it, to open the session.

### Session

- **Header.** Title, project, start and end, wall time, the number of agents and the cost at API
  list price.
- **Figures.** Agent working time, waiting on a human, peak context, cache hit share and busts,
  tool calls and refused calls, human prompts and secret exposures.
- **Timeline.** One lane per agent on one time axis, with the running cost above. Each tick is the
  tool calls of a stretch of time, coloured by the kind of call that stretch made most; the legend
  above the lanes names the colours and the marks for prompts, refusals, cache busts and
  compactions. Hover over a lane to read a stretch's counts. Click an agent, or press Enter on it,
  to show only that agent here and in the event log; click it again to show all. Idle stretches
  are folded and labelled with their length. A long session shows its busiest agents first, with a
  button to show the rest.
- **Agents.** Turns, tool calls, peak context, active time and cost per agent.
- **Checks.** The result of each check from [queries](queries.md), the ones that found something
  first.
- **Cost by model** and **prompts by author.**
- **Events.** The session's events, newest first. Key events (prompts, agent starts, refusals,
  failed tool results, cache busts and compactions) are shown first; the chips switch to all events
  or one kind, each with its count. The button under the list shows more rows.

The UI updates while sessions run: new sessions, figures and events arrive over server-sent events.

### Embedding a session

A host program, such as an LLM gateway dashboard, can show one session without the overview. It
opens one of these pages with the session id in the path; they have no top bar and no navigation.

| page | shows |
|---|---|
| `/embed/session/<id>` | the whole session page |
| `/embed/timeline/<id>` | the header and the timeline |
| `/embed/events/<id>` | the header, the figures and the event log |

Add `?theme=light` or `?theme=dark` to fix the theme in the frame.

### Data endpoints

The UI reads JSON views that you can also query yourself:

```console
curl 'http://127.0.0.1:7420/view/views/session-list?search=refactor&limit=20'
```

| path | returns |
|---|---|
| `/view/views/session-list?since&search&offset&limit` | the sessions, the last active first |
| `/view/views/figures?since&bucket_seconds` | the overview's sums per time bucket |
| `/view/views/session?session_id=<id>` | one session |
| `/view/views/session-agents?session_id=<id>` | its agents |
| `/view/views/session-events?session_id=<id>&agent&kind&notable&before` | its events, newest first, 500 at a time |
| `/view/views/session-event-counts?session_id=<id>&agent` | its events counted by kind |
| `/view/views/session-timeline?session_id=<id>&slot_seconds` | its events counted per agent and time slot |
| `/view/views/session-turns?session_id=<id>` | its model turns |
| `/view/views/agent-turns?session_id=<id>&agent=<agent>` | one agent's model turns |
| `/view/views/session-checks?session_id=<id>` | the check results |
| `/view/channels/sessions` | a stream of session updates |
| `/view/channels/session_events?session_id=<id>` | a stream of one session's new events |

`serve` answers each request with one query on the store and keeps no rows in memory.

## Dashboard file

```console
agenticspec analyze --out sessions.html
```

Without `--out`, the file is `agenticspec-dashboard.html` beside the store, in the same directory
as `sessions.db`. The file opens offline in any browser. It holds the KPIs of the window and, per
session:

- models with messages, output tokens, context size and cost
- cache busts by cause
- tool calls by category
- prompts to the main agent by author
- permission decisions of the main agent
- secrets and personal data found, by kind and count

The file contains session titles, which come from your prompts. Treat it like the transcripts.
