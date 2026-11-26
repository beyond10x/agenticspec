# Privacy

agentspec runs on your machine. It reads transcripts, writes one SQLite file and makes no network
request. `serve` listens on 127.0.0.1 only.

## What the store holds

| stored | not stored |
|---|---|
| which tools ran, in which order, how long they took and whether they succeeded | the text of your prompts |
| token counts, models and cache figures of each model turn | the arguments of tool calls |
| permission decisions and the reason a hook gave | the output of tool calls |
| the working directory, relative to your home directory | the agent's replies |
| the length and SHA-256 of every prompt, tool input and tool output | secrets and personal data, which become aliases |
| the session title Claude Code derived from your prompts, and the transcript's path | |

Keep the store, the dashboard file and the UI to yourself. The session titles and paths come from
your work.

## Secrets and personal data

Before a step is stored, agentspec scans prompts, tool inputs, tool outputs and agent text for
these kinds:

| kind | class |
|---|---|
| `aws_access_key` | secret |
| `private_key` (a PEM private key block) | secret |
| `slack_token` | secret |
| `github_token` | secret |
| `gitlab_token` | secret |
| `anthropic_key` | secret |
| `jwt` | secret |
| `bearer` (an `Authorization: Bearer` value) | secret |
| `email` | personal data |

Each value found becomes an alias that stays the same within a session, such as `gitlab_token#1`.
The second appearance of the same token is still visible as `gitlab_token#1`, without the token.
A finding reports the kind and the count, never the value.

A value that is a known placeholder counts as `Ok`: it contains `example`, `dummy`, `fake`,
`placeholder`, `redacted`, `xxxx`, `abcdef` or `1234567`, or it is an e-mail address under
`example.com`, `example.org`, `example.net`, `.test`, `.invalid` or `localhost`, or a no-reply
address. Every other match counts as `Unk` until
someone confirms it. See [KPIs](kpis.md) for where these figures show up.

## Removing data

Delete the store to forget everything: `~/.local/share/agentspec/sessions.db`, or the
`sessions.db` in the directory you passed as `--state-dir`. The next `sync` fills it again from the
transcripts in the window.
