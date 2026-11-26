# What sessionspec is

sessionspec reads the transcripts of your coding-agent sessions and turns them into numbers you
can compare: cost, cache use, time spent waiting on you, refused tool calls, secrets that reached
the agent. It runs on your machine and sends nothing anywhere.

It reads Claude Code transcripts from `~/.claude/projects`.

## What you get

| you want to know | run |
|---|---|
| how your sessions went over the last days, at a glance | `sessionspec serve`, then open http://127.0.0.1:7420/ |
| the same as one file you can keep or share | `sessionspec analyze` |
| the raw figures as JSON | `sessionspec kpi` |
| which sessions show a pattern, such as a refused call asked for again | `sessionspec query` |
| which runs of tool calls repeat across your sessions | `sessionspec mine` |
| a repeated run as a draft procedure an executor can run | `sessionspec procedure`, then `sessionspec export` |

## How it works

1. **Lift.** Each transcript record becomes a typed step: a prompt, a tool call, a permission
   decision, a model turn. Every step records who acted. Where the transcript does not say who,
   the step says so instead of guessing.
2. **Redact.** Secrets and personal data are replaced by aliases such as `gitlab_token#1`. Only
   the kind and the count are kept, never the value.
3. **Store.** The redacted steps go into one SQLite file. `sessionspec sync` adds new and changed
   sessions and skips the rest, so later runs take seconds.
4. **Evaluate.** KPIs, queries and pattern mining read the store, not the transcript files.

## Next

- [Getting started](getting-started.md): install, first sync, first dashboard.
- [Dashboard](dashboard.md): what the pages show.
- [KPIs](kpis.md): each figure and how to read it.
- [Queries](queries.md): find sessions by what happened in them.
- [Patterns and procedures](patterns.md): mine repeated work and export it.
- [Privacy](privacy.md): what is stored and what is not.
- [Records library](library.md): append records from your own program, such as an LLM gateway.
