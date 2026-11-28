---
format: aep.planning-md/3
id: story:session-lift
kind: story
status: implemented
title: Lift an observed session into block rows
summary: Rules over the session IR write rows of agentic.work Task, Run, Step, agentic.question Question, agentic.channel Message and, where recorded, agentic.resource Claim, each with its evidence node (consumer gap M7)
relations:
- decomposes: epic:evaluation-loop
- depends_on: story:scenario-format
scope:
- confidence: cited
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-records/src
- confidence: cited
  path: crates/agenticspec-records/tests
- confidence: cited
  path: docs/queries.md
- confidence: inferred
  path: queries/blocks.sq
- confidence: cited
  path: spec/cli.yaml
- confidence: cited
  path: spec/domains/cli.yaml
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-28T08:40:17Z", actor: "human:operator", revision: 9, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-28T08:40:17Z", actor: "human:operator", revision: 10, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-28T08:40:17Z", actor: "human:operator", revision: 11, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An observed session is lifted into rows of the building blocks. A consumer's gap report (M7)
asked for the lift of a session into block rows of `Task`, `Run`, `Step`, `Question`, `Message`
and `Claim`. At `25fbfcb` the session store holds the session IR (`spec/domains/session.yaml`):
its entities are `Session`, `Agent`, `Exposure` and `ToolCall` (`session.yaml:147-290`), and the
query relations over it are `session`, `agent`, `cmd`, `prompt`, `call`, `decided`, `completed`,
`turn`, ... (`docs/queries.md:121-154`). No rule turns them into rows of `agentic.work`,
`agentic.question`, `agentic.channel` or `agentic.resource`.

The lift reads the IR and its relations:

- `agentic.work.Task`: a prompt classed `Task` (`prompt.class`, `session.yaml:79-81`), by its
  author (`session.yaml:63-65`);
- `agentic.work.Run`: an agent of the session working that task (main or sub-agent, `agent`
  relation, `spawned`);
- `agentic.work.Step`: the turns and tool calls of that run, in order;
- `agentic.question.Question`: a call of category `Message` that asks the user
  (`AskUserQuestion`, `crates/agenticspec-records/src/claude.rs:467`) or a prompt classed
  `Question`;
- `agentic.channel.Message`: a call that sends a message (`SendMessage`, same line) or a prompt
  whose author is a `Delegate` or a `PeerSession`;
- `agentic.resource.Claim`: only where the transcript records a claim; otherwise the lift reports
  `not_observable` for it and writes no row. Nothing is invented.

Every row carries the IR node it was lifted from. The rows are written in the row form of the
scenario's `given` (`story:scenario-format`), so a lifted session can be the starting state of a
scenario; this story depends on that story for the form.

## Scope

- `queries/blocks.sq` (new): the lift as rules over the relations of `docs/queries.md`, so
  `agenticspec query queries/blocks.sq --transcript <file>` (`docs/queries.md:20`) shows the rows.
- `crates/agenticspec-records/src/`: write the rows of one session in the row form, checked
  against the compiled blocks (made once per test run, never per test); the command that writes
  them declared in `spec/cli.yaml` with its types in `spec/domains/cli.yaml`, regenerated with
  `task generate` (held by `task check:generated`).
- `crates/agenticspec-records/tests/` with a transcript fixture written for the test;
  `docs/queries.md` (the new relations).
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s; a pool job over
  10 minutes is killed. No new Python.

## Shared surfaces

- `spec/cli.yaml`, `spec/domains/cli.yaml`, `crates/agenticspec-cli-contract/`:
  `story:contract-derivation` and the stories of `epic:architecture-contract` that add simulator
  subcommands, and `story:ui-generator`.
- `crates/agenticspec-records/`: `story:scenario-input-extraction` (comes after, reads these rows).

## Acceptance

- Given a transcript fixture with one human prompt classed `Task`, one sub-agent spawned, one
  `AskUserQuestion` call that was answered and one `SendMessage` call, the lift writes one
  `agentic.work.Task`, two `agentic.work.Run` (the main agent and the sub-agent), at least one
  `agentic.work.Step` per run, one `agentic.question.Question` and one `agentic.channel.Message`,
  each naming the IR node it came from, where before no rule wrote a block row.
- Given the same fixture, every row's entity and fields are declared by the compiled blocks, and
  `agenticspec-sim scenario check` accepts a scenario whose `given` is these rows.
- Given a transcript with no record of a claim, the lift writes no `agentic.resource.Claim` row
  and its report names `agentic.resource.Claim` as `not_observable`.
- Given the same transcript, two lifts write the same bytes.

## Not in this story

Scenario inputs and what they caused (`story:scenario-input-extraction`); lifting sources other
than the session store; fitting a behaviour profile from lifted rows.
