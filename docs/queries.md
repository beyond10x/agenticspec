# Queries

sq is a small query language over your sessions. A query finds the places in sessions where
something happened: a call a hook refused, an edit followed by a test run, a long wait on you.

```console
agenticspec query --text '?- exposure{kind: K, channel: "ToolInput", verdict: "Unk"}.'
```

This lists the kinds of unconfirmed secrets and personal data that appeared in tool arguments in
the last 72 hours. `query` prints a JSON report with one row per answer and session.

## Run a query

| form | what runs |
|---|---|
| `agenticspec query file.sq` | the rules, queries and checks in the file |
| `agenticspec query --text '...'` | sq given inline |
| `agenticspec query file.sq --text '...'` | both, the text appended to the file |
| `agenticspec query file.sq --transcript <session>.jsonl` | one transcript file, read directly, without the store |

Without `--transcript`, a query runs over every session of the window. `--since-hours` and `--root`
choose the window.

## Rules and queries

A rule gives a name to a combination of facts. A query, starting with `?-`, prints every set of
values that makes it true.

```text
refused(C, T, R) :- call{node: C, id: I, tool: T},
                    decided{call: I, decision: "Refuse", reason: R}.
?- refused(C, T, R).
```

This prints each refused tool call, its tool and the reason.

- Names starting with an upper-case letter are variables. `_` matches anything.
- `rel{field: Value}` matches facts of a relation by field name. Fields you leave out match anything.
- `"text"` is a literal. `~"^(Edit|Execute)$"` matches a regular expression.
- `!p(X)` holds when `p(X)` has no answer.
- Comparisons: `=`, `!=`, `<`, `<=`, `>`, `>=`.
- Durations are written `250ms`, `30s`, `5m` or `1h` and compare as milliseconds.
- `//`, `#` and `%` start a comment.

## Counting

`count`, `sum`, `min` and `max` aggregate over the answers inside the braces:

```text
calls(A, N) :- agent{id: A, role: "Main"}, N = count{ call{agent: A, node: C} }, N >= 50.
?- calls(A, N).
```

`count` counts distinct answers. Bind a field that differs per fact, here `node: C`, or every call
counts once.

## Order in time: `seq`

`seq(A, B, ...)` holds when the steps happen in this order. Each one is the nearest later step that
matches.

```text
edit_then_run(E, X) :-
    seq(E, X, scope: agent, within: 5m),
    call{node: E, category: "Edit"}, call{node: X, category: "Execute"}.
?- edit_then_run(E, X).
```

| option | effect |
|---|---|
| `scope: agent` | the steps belong to the same agent |
| `scope: session` | the steps may belong to different agents of the session |
| `within: 10m` | the whole sequence lies within this time |
| `gap: 3` | at most this many other steps between two matched steps |
| `pick: all` | every later match, not only the nearest |

## Checks

A check turns a rule into a pass or fail result per session:

```text
check refused-then-retried {
  about "a call a hook refused was asked for again unchanged"
  severity medium
  expect none
  find retried
}
```

`expect none` means the rule should have no answers. `expect some` means it should have at least
one. Each session gets one verdict per check:

| verdict | meaning |
|---|---|
| `Ok` | the expectation holds |
| `Gap` | it does not |
| `Unk` | it depends on facts the transcript does not record for certain |

The report counts the verdicts of each check under `ok`, `gap` and `unk`.

The checks that ship with agenticspec are in
[`queries/session.sq`](https://github.com/beyond10x/agenticspec/blob/main/queries/session.sq).
`sync` evaluates them for every session, and the dashboard shows the results.

| check | finds |
|---|---|
| `cache-bust-after-idle` | a turn after an idle gap over 5 minutes that wrote the whole context again |
| `refused-then-retried` | a call a hook refused that the agent asked for again unchanged |
| `fanout-unread` | a sub-agent of a fan-out of three or more that ended after the parent's last turn, so its result was not read |
| `human-correction` | a correction from you within 2 minutes of an edit or a command |
| `waiting-on-human` | the main agent waiting more than 30 minutes for you |

## Definite and possible answers

Some facts are not certain: an author read from the opening words of a prompt, a decision the
transcript does not record, a sub-agent link that had to be inferred. An answer that rests on such a
fact is marked `possible`, and `unknown` lists the facts it rests on. Every other answer is
`definite`.

## Relations

Every step of a session is a node, written `<session>#<number>`.

| relation | fields |
|---|---|
| `session` | `id, project, harness, opened_at, ended_at` |
| `agent` | `id, session, parent, role, depth, spawned_by, started_at, ended_at` |
| `cmd` | `node, session, agent, seq, at, actor, kind, outcome`: every step |
| `prompt` | `node, agent, author, attribution, class, bytes, digest, mid_turn` |
| `call` | `node, id, agent, tool, category, bytes, digest, cwd, background` |
| `decided` | `node, call, decision, decider, reason` |
| `completed` | `node, call, ok, bytes, digest, dur` |
| `turn` | `node, agent, model, ctx, uncached, cache_read, write_5m, write_1h, output` |
| `turn_end` | `node, agent` |
| `compacted` | `node, agent, before` |
| `exposure` | `node, id, agent, channel, class, kind, alias, verdict, call` |
| `refused_cmd` | `node, error`: a step agenticspec could not replay |
| `at` | `node, t`: the time of a node |
| `next` | `a, b`: `b` is the next step of the same agent |
| `turn_of` | `node, prompt`: the step belongs to the turn this prompt started |
| `interval` | `agent, state, on, from, to, dur, open`: when an agent was `Idle` or `Working`, and what it waited on |
| `bust` | `node, agent, cause, written`: a cache bust, see [KPIs](kpis.md) |
| `spawned` | `call, child`: the call started this sub-agent |

Values you will match on:

| field | values |
|---|---|
| `call.category` | `Read`, `Search`, `Edit`, `Execute`, `Network`, `Spawn`, `Message`, `Plan`, `Other` |
| `prompt.class` | `Task`, `Correction`, `Approval`, `Question`, `Continue`, `Unknown` |
| `prompt.author` | `Human`, `Delegate`, `PeerSession`, `ParentAgent`, `Harness`, `Hook` |
| `decided.decision` | `Accept`, `Refuse` |
| `agent.role` | `Main`, `SubAgent`, `SideCall` (a model call of the tool outside every conversation, such as a title) |

`bytes` and `digest` stand for content: its length and its SHA-256. The text itself is not stored,
so a query never sees it.

## Relations of an architecture

The same language selects in an architecture document, over the relations `part`, `setting`,
`link` and `marker`. agenticstudio uses them in the selectors of its patches; their fields are on
the [agenticstudio](studio.md) page.
