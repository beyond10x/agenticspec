# KPIs

```console
agentspec kpi --since-hours 24
```

`kpi` prints one JSON report over every session with activity in the window. The dashboard shows
the same figures. Most figures come twice: once for main agents and once for sub-agents
(`scope: main` or `sub`).

Whether a figure is good depends on how you work. Full autonomy is the goal of an unattended run.
In pair work, frequent prompts are normal.

## Volume

| field | what it counts |
|---|---|
| `sessions` | sessions with activity in the window |
| `transcripts` | transcript files read |
| `sub_agents` | sub-agents started in those sessions |
| `refused_commands` | records agentspec could not replay. Figures over those agents may be incomplete |

## Cost and models

| field | what it counts |
|---|---|
| `cost_microdollars` | the tokens of every model message at the API list price of its model, in millionths of a dollar |
| `unpriced_messages` | messages of a model with no known price, left out of the cost |
| `models[]` | per model and scope: messages, output tokens, context size (median, 90th percentile, largest), cost |

The cost is what the same tokens would cost on the public API. A subscription bills differently.
Use it to compare sessions with each other.

## Cache

| field | what it counts |
|---|---|
| `cache[].read_tokens` | input tokens served from the prompt cache |
| `cache[].write_tokens` | input tokens written to the cache |
| `cache[].uncached_tokens` | input tokens neither read nor written |
| `cache[].hit_permille` | reads as a share of all input, in thousandths |
| `busts[]` | turns that wrote most of the context again, by cause, with the tokens written again |

A cache bust is a model turn that read less than half of the previous turn's context from the cache
and wrote more than half of it again. Contexts of 20,000 tokens or fewer are not counted.
agentspec gives each bust one cause:

| cause | meaning |
|---|---|
| `compacted` | the context was compacted just before |
| `model_changed` | the turn used another model than the turn before |
| `ttl_expired` | the gap since the turn before was longer than the cache lifetime: 5 minutes, or 1 hour for the long cache |
| `unexplained` | none of the above |

Many `ttl_expired` busts mean the agent often resumed after a pause. Each one pays for the whole
context again.

## Concurrency

`concurrency[]` gives how many agents were active in the same minute: the median, the 90th
percentile and the maximum. An agent counts as active in a minute when two of its records lie at
most 5 minutes apart around it.

## Autonomy

| field | what it counts |
|---|---|
| `autonomy.working_seconds` | time in which at least one agent of the session was working. Gaps up to 10 minutes count as working |
| `autonomy.waiting_on_human_seconds` | time in which no agent worked, from a main agent's turn end until your next prompt |
| `autonomy.away_seconds` | waits longer than one hour, counted apart so a lunch break does not count as waiting |
| `autonomy.prompts[]` | prompts to main agents by author and by how sure the author is |
| `autonomy.decisions[]` | permission decisions on tool calls of main agents, by who decided and the outcome |

Prompt authors:

| author | who wrote the prompt |
|---|---|
| `Human` | you |
| `Delegate` | a session acting for you, see `AGENTSPEC_DELEGATES` in [Getting started](getting-started.md) |
| `PeerSession` | another agent session |
| `ParentAgent` | the agent that started this sub-agent |
| `Harness` | the tool itself: task notifications, compaction summaries, auto-continue |
| `Hook` | a hook |

Each author comes with an attribution. `Known` means a field of the transcript says so. `Inferred`
means it was read from the opening words of the prompt. `Unknown` means neither.

Who decided a tool call: `PermissionMode`, `Classifier`, `Hook`, `Human`, `SafetyCheck`, `Rule`, or
`NotRecorded` when the transcript carries no decision.

## Secrets and personal data

`exposures[]` counts the secrets and personal data found in sessions, by kind, by where they
appeared and by verdict. See [Privacy](privacy.md) for the kinds.

| channel | where the value appeared |
|---|---|
| `Prompt` | in a prompt to the agent |
| `ToolOutput` | in the result of a tool call |
| `ToolInput` | in the arguments of a tool call. For a shell, network or message tool, the value left the session |
| `AgentText` | in the agent's own text |

| verdict | meaning |
|---|---|
| `Ok` | a known placeholder such as `EXAMPLE` or `example.com` |
| `Unk` | a match nobody has confirmed. Look at these |
| `Gap` | a confirmed exposure |
