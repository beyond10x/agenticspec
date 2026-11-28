---
format: aep.planning-md/3
id: story:scenario-input-extraction
kind: story
status: draft
title: Extract scenario inputs from a lifted session, joined to what they caused
summary: Outside inputs of a lifted session become when entries of a draft scenario, joined through the receiving turn to the rows they caused; a message names the turn that received it (consumer gap M8)
relations:
- decomposes: epic:evaluation-loop
- depends_on: story:session-lift
- depends_on: story:scenario-format
scope:
- confidence: inferred
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/7-collaboration/channel.yaml
- confidence: cited
  path: crates/agenticspec-cli-contract
- confidence: cited
  path: crates/agenticspec-records/src
- confidence: cited
  path: crates/agenticspec-records/tests
- confidence: cited
  path: docs/blocks/7-collaboration/channel.md
- confidence: inferred
  path: queries/blocks.sq
- confidence: cited
  path: spec/cli.yaml
- confidence: cited
  path: spec/domains/cli.yaml
- confidence: inferred
  path: website/src/generated
revision: 11
---
## Outcome

From a lifted session, the inputs that came from outside are extracted as scenario inputs and
joined to what they caused, so an observed session becomes a draft scenario. A consumer's gap
report (M8) asked for this. At `25fbfcb` the blocks cannot hold the join: a harness turn names
its inference, usage, context size and time, and nothing it received
(`agentic.harness.Turn`, `blocks/4-agent/harness.yaml:163-182`).

The layer rule forbids the turn naming a message: `harness` is layer 4, `channel` layer 7, and a
block names only its own layer or a lower one (`design/consumer-gaps.md`, the layer rule). Taken:
it is turned around: `agentic.channel.Message` names the turn that received it, as a reference to
`agentic.harness.Turn`. A prompt that is not a message (a human typing in a session) is joined the
same way by the turn it started (`turn_of`, `docs/queries.md:141`).

The extraction:

- an input is a prompt whose author is `Human`, `Delegate` or `PeerSession`, or a received
  message (`session.yaml:63-65`); a prompt by `Harness` or `Hook` (task notifications,
  compaction summaries) is internal and not an input;
- each input is written as a `when` entry at the border part the session's main agent plays, with
  its author and its proof (`agentic.channel.Proof`; `None` where the record names none) and its
  time offset;
- what it caused: the lifted rows (`story:session-lift`) of the turn that received it and of every
  step after it up to the next input, written as the scenario's `then` in block events;
- the architecture and the mapping of session agents to parts are given by the caller (the
  architecture file and `main=operator`), so the draft scenario passes `scenario check`.

## Scope

- `blocks/7-collaboration/channel.yaml`: the reference from `Message` to the receiving turn, with
  its command input; `blocks/0-foundation/shared.yaml` if an identifier is needed; generated pages
  through `task docs:generate`.
- `crates/agenticspec-records/src/`: the extraction, writing `agenticspec-scenario/1`; the command
  declared in `spec/cli.yaml` and `spec/domains/cli.yaml`, regenerated with `task generate`.
- `queries/blocks.sq`: the join rules beside the lift's rules.
- `crates/agenticspec-records/tests/` with the transcript fixture of `story:session-lift`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s; a pool job over
  10 minutes is killed. No new Python.

## Shared surfaces

- `crates/agenticspec-records/`, `queries/blocks.sq`, `spec/cli.yaml`: `story:session-lift` (comes
  first, its rows are read here).
- `blocks/7-collaboration/channel.yaml`: `story:governance-types` (message provenance as an
  attestation), `story:block-bindings`; this story reads neither, and the ordering is set when
  they leave draft.
- The scenario format: `story:scenario-format` (comes first).

## Acceptance

- Given `blocks/7-collaboration/channel.yaml` after this story, a `Message` names the turn that
  received it, and `ess specify validate --path blocks --strict-requires`, `task check:layers`
  and `task check:blocks` pass, where before no field joined a received message to a turn.
- Given the transcript fixture of `story:session-lift`, `--architecture
  architectures/workstation.yaml` and the mapping `main=operator`, the extraction writes a
  scenario that `agenticspec-sim scenario check` accepts, with one input at `operator` authored by
  `owner` and a `then` that holds `agentic.work.TaskCreated` and `agentic.work.RunStarted` for
  the task that input started.
- Given a prompt by the `Harness` author in the same fixture, the scenario holds no input for it.
- Given an input whose caused rows cannot be joined (no turn after it), the extraction's report
  lists it as `unjoined`, and the scenario holds the input with no expectation attributed to it.
- Given the same transcript and arguments, two extractions write the same bytes.

## Not in this story

Running the extracted scenario (`story:scenario-simulator-run`); a corpus written from extracted
inputs (`story:task-corpus` reads a corpus, filling it is later); naming the architecture of a
session without the caller's mapping.
