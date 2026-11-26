# Patterns and procedures

`agentspec mine` finds runs of tool calls that your agents repeat across sessions and projects.
A repeated run can become a draft procedure, and the procedure can be exported for an executor.

## Mine patterns

```console
agentspec mine --since-hours 168
```

Each agent turn becomes a row of tokens, one per tool call. Several calls of the same tool in a
row become one token with a `+`, such as `Bash+`. A pattern is a run of 2 to 8 tokens.

```json
{"id": "p-424c51455f", "tokens": ["Bash+", "Write", "Bash+"],
 "sessions": 31, "projects": 12, "occurrences": 823, "ok_permille": 987,
 "query": "?- call{node: N1, tool: \"Bash\"}, call{node: N2, tool: \"Write\"}, ..."}
```

| field | meaning |
|---|---|
| `id` | a stable id for the pattern |
| `tokens` | the run of tool calls |
| `sessions`, `projects` | in how many sessions and projects it occurs |
| `occurrences` | how often it occurs in total |
| `ok_permille` | the share of its tool calls that succeeded, in thousandths |
| `query` | an sq query that finds the pattern again, see [Queries](queries.md) |

| option | default | effect |
|---|---|---|
| `--level tool` | yes | a token is the tool name |
| `--level coarse` | | a token is the tool category, such as `Edit` or `Execute` |
| `--min-sessions <n>` | 5 | keep a pattern seen in at least this many sessions |
| `--min-projects <n>` | 2 | and in at least this many projects |
| `--top <n>` | 50 | report this many patterns, most widespread first |

A pattern is left out when a pattern one token longer occurs in the same sessions.

## Turn a pattern into a procedure

```console
agentspec procedure p-424c51455f --out procedure.json
```

Pass the pattern's id or its tokens, such as `"Bash+ Write Bash+"`. Without `--out` the procedure is
printed. The procedure is a draft built from every match of the pattern:

| part | holds |
|---|---|
| `steps[]` | one step per token, with its tool, category and effect (`read`, `write`, `external` or `none`) |
| `steps[].kind` | `program` for a shell step, `child` for a sub-agent, `person` when you approved or refused most of its calls, `judgement` otherwise |
| `steps[].retry_max` | the most times the step ran again in one match after it failed |
| `steps[].timeout_seconds` | the longest observed duration, rounded up to a minute |
| `parameters[]` | values that differ between matches, such as a tool name. Paths and commands are not kept |
| `sessions[]` | the sessions it came from |

## Export a procedure

```console
agentspec export procedure.json --out exported/
```

This writes two files into `exported/`:

- `<id>.protocol.yaml`, a protocol for [Canon](https://github.com/beyond10x/canon), with one action
  and one claim per step.
- `<id>.yaml`, bindings for [Loom](https://github.com/beyond10x/loom) that say how each step runs.

When the `canon` command is installed, `export` validates the protocol with it and reports the
result. The exported procedure is still a draft. Read it and adjust it before anything runs it.
