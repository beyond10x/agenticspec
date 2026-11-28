# Scenarios and task corpora

A scenario tests one architecture with given inputs. It puts the inputs at the architecture's
border, scripts the parts outside it (the model, the build tool, a person's reply), and lists the
events it expects, named as the [building blocks](/docs/blocks) name them. `agenticspec sim
scenario run` drives the simulator with it on a virtual clock and checks each expected event.

A task corpus lists inputs taken from real sessions, each with the outcome it had. `agenticspec
sim corpus run` runs every entry and prints the outcome the simulator predicts beside the one the
entry expects. A behaviour profile can also draw its tasks from a corpus.

The examples for the workstation architecture are in
[`scenarios/workstation/`](https://github.com/beyond10x/agenticspec/tree/main/scenarios/workstation).

## Commands

```sh
agenticspec sim scenario check scenarios/workstation/question-answered.scenario.yaml
agenticspec sim scenario run scenarios/workstation/question-answered.scenario.yaml --out target/scenarios/question
agenticspec sim corpus run scenarios/workstation/workstation.corpus.yaml
```

| command | what it does | prints |
|---|---|---|
| `sim scenario check <scenario>` | reads the scenario and checks it against its architecture and profile | the scenario name, the architecture, the number of inputs and of expectations |
| `sim scenario run <scenario> --out <dir>` | checks the scenario, runs it and writes `<dir>/scenario-result.json` | the same, with the verdict, the predicted class and the counts of expectations held, failed and not checked |
| `sim corpus run <corpus>` | runs every entry of the corpus | the rows as JSON; the table goes to standard error |
| `blocks scenario <session> ...` | writes a draft scenario of a session (below) | the file, the counts, the inputs left unjoined or unmapped |

`task check:scenarios` runs every `scenarios/*/*.scenario.yaml` and every
`scenarios/*/*.corpus.yaml`.

A refused command exits 1 and prints `<code>: <detail>` to standard error:

| code | when |
|---|---|
| `document_unreadable` | a file cannot be read or is not YAML |
| `scenario_invalid` | the scenario is not a valid `agenticspec-scenario/1` document; every problem is named by its path, such as `when.0.atx: unknown key` |
| `architecture_digest_mismatch` | `architecture.digest` is not the sha256 of the architecture file; `scenario run` refuses before the simulation starts |
| `profile_digest_mismatch` | `profile.digest` is not the sha256 of the profile file |
| `architecture_refused` | the architecture itself is refused, with the rules `sim check` names |
| `input_not_at_border` | an input is at a part the architecture's `border` does not list |
| `invalid_row` | a `given` row does not fit its entity ([below](#given-rows)) |
| `fixture_without_part` | a fixture names a part the architecture does not have |
| `unknown_event` | an expected `event` is no event a block declares, such as `agentic.resource.ClaimGrantd` |
| `unknown_refusal` | an expected `refusal` is no command a block declares, or its `error` is no error that command is refused with |
| `scenario_refused` | the scenario does not fit its architecture in another way: an author names no part, a fixture names a part of the wrong block, an `unavailable` entry or a part of `then.parts` names no part or no dependency, a view, command or error is not declared by a block; or it has problems of more than one code |

A check names every problem, each after its code. When all of them have one code, the command
exits with that code; when their codes differ, with `scenario_refused`.
| `out_missing` | `scenario run` without `--out` |
| `out_unwritable` | `scenario-result.json` cannot be written |
| `expectation_failed` | an expected event or a KPI bound failed; `scenario-result.json` is written first |
| `corpus_invalid` | the corpus is not a valid `agenticspec-corpus/1` document, or an entry's digest, kind or input digest does not match its scenario |
| `corpus_failed` | an entry's expectations failed, or it predicts another class than it expects |

## A scenario

```yaml
format: agenticspec-scenario/1
scenario: question-answered
summary: The worker asks a question only the owner can answer; the owner picks B.
architecture: { ref: ../../architectures/workstation.yaml, digest: 9f48c790a19597559396e9cb96d13e753d6d7b427c48eaf6bd9a91cd49fdfe99 }
profile: { ref: ../../fields/coding/profiles/coding.yaml, digest: f8f5d0007605f7d4b4da01c9996165d3a93e7e3e81ca23168412fc508bf83335 }
seed: 1
horizon: 2h

when:
  - { at: operator, kind: prompt, author: owner, text: "Rename the config key the release script reads." }

fixtures:
  model:
    - { for: worker, outcome: question, turns: 3, options: [A, B], default: A, needs_owner: true }
  principals:
    - { part: owner, option: B, after: 10m }

then:
  order: sequence
  events:
    - { event: agentic.work.RunStarted, part: worker, task: when.0 }
    - { event: agentic.question.QuestionAsked, part: worker, task: when.0, options: [A, B] }
    - { event: agentic.question.QuestionAsked, part: operator, task: when.0 }
    - { event: agentic.channel.MessageSent, part: operator, channel: chat }
    - { event: agentic.question.QuestionAnswered, part: owner, task: when.0, option: B }
    - { event: agentic.work.TaskCompleted, part: worker, task: when.0 }
  kpis:
    questions_forwarded: { min: 1, max: 1 }
    questions_defaulted: { max: 0 }
```

| key | required | what |
|---|---|---|
| `format` | yes | `agenticspec-scenario/1` |
| `scenario` | yes | the name |
| `summary` | no | one paragraph |
| `architecture` | yes | `{ ref, digest }`: the architecture file, relative to the scenario, and the sha256 of its bytes (`sha256sum <file>`) |
| `profile` | yes | `{ ref, digest }`: the behaviour profile the simulator draws everything else from |
| `seed` | no | the seed of the run; 0 by default. The same scenario and seed give the same `scenario-result.json`, byte for byte |
| `horizon` | no | how long the run lasts; `4h` by default |
| `given` | no | rows that exist when the run starts |
| `when` | yes | the inputs, in order |
| `fixtures` | no | stand-ins for the parts outside the architecture |
| `unavailable` | no | the parts and dependencies that are unavailable, and from when ([below](#unavailable-parts-and-dependencies)) |
| `then` | no | the expected events, KPI bounds and what each part shows |

Every key that is not in these tables is refused with its path. Durations are written `90s`,
`15m`, `2h` or `1d`.

### Inputs

Each entry of `when` is one input at a border part:

| key | required | what |
|---|---|---|
| `at` | yes | the part that receives it; it must be listed in the architecture's `border` |
| `kind` | yes | `prompt`, `order`, `message` or `control` |
| `author` | yes | the part that sends it: a principal or another part of the architecture |
| `act` | for `control` | `send`, `pick`, `run`, `retry`, `dispatch`, `approve` or `reject`. A prompt and a message `send` by default, an order `dispatch`s |
| `proof` | no | the proof that the input comes from its author, such as `client_record`, `process_identity` or `passkey`; `none` counts as no proof |
| `text` | no | the input as written; the simulator does not read it, a corpus entry holds its digest |
| `after` | no | the time since the input before it (the first: since the start); `0s` by default |

The run admits an input or refuses it by a rule. Every input is a message to its border part, so
a refused input is a refusal of `agentic.channel.SendMessage`:

| rule | refused when | command | block error |
|---|---|---|---|
| `input_without_link` | no link from the author to the part carries the input: a prompt needs a `prompt` link, an order an `order` link, a message or a control input any link | `agentic.channel.SendMessage` | `agentic.channel.NotAParty` |
| `control_without_authority` | a control input comes from an author that is not the principal the part acts for (`acts_for`) | `agentic.channel.SendMessage` | `agentic.delegation.NotTheDelegator` |
| `control_without_proof` | a control input carries no proof | `agentic.channel.SendMessage` | none: no block refuses a message without proof |
| `claim_conflict` | a copy claims a board entry another copy claimed first | `agentic.channel.TakePost` | `agentic.channel.PostAlreadyTaken` |
| `corpus_refused` | a corpus entry a profile drew was refused, so it makes no task | `agentic.channel.SendMessage` | none |
| `part_unavailable` | the part, the author or the channel of the input is unavailable at its time, or the part is cut from the author or the channel ([below](#unavailable-parts-and-dependencies)) | `agentic.channel.SendMessage` | none: no block declares one |

An admitted prompt (`send`), order (`send`, `dispatch`) or control input (`run`, `retry`,
`dispatch`) makes a task, named after the input: `when.0` for the first. Other admitted inputs are
delivered and change nothing else in the simulation.

### Given rows

```yaml
given:
  tasks:
    - { id: backlog-1 }
  questions:
    - { id: q-1, asked_by: worker, addressee: owner, options: [A, B, C], default: C }
```

A given task waits in the backlog at the start. A given question is open at the start: a copy of
`asked_by` holds it and waits for the answer of `addressee`. Neither records a creation event; the
events that follow carry the row's `id` as their task.

`rows` holds rows of any block entity, such as the rows `agenticspec blocks lift` writes for a
session ([Queries](queries.md)):

```yaml
given:
  rows:
    - entity: agentic.question.Question
      id: e77e1af7-84f9-f4c4-12b9-2c0d99596d5c
      state: Answered
      fields: { asked_by: 19faf7cc-c2aa-c558-1b93-cdd35cd4c467, addressee: c1582bfa-36f6-f8ad-4777-2e7feb7f5a80, text: { digest: cc780a105da069ae2ec1b36669e62f7891da7c136719992ea3752c419d572b56, length: 100, class: tool_input } }
      not_recorded: [kind, options, default_option, deadline]
      evidence: "lift-demo#16"
```

| key | required | what |
|---|---|---|
| `entity` | yes | a block entity, by its full name |
| `id` | yes | its identifier |
| `state` | no | a state of the entity's lifecycle |
| `fields` | no | field values, each a field the entity declares, of its type |
| `not_recorded` | no | required fields the record the row comes from does not hold |
| `evidence` | no | where the row comes from, such as a node of a session |

`scenario check` refuses a row of an entity no block declares, a field the entity does not have,
a value of the wrong type, a state outside its lifecycle, and a required field that is neither in
`fields` nor in `not_recorded`, each by its path (`given.rows.0.fields.pool_id`). The simulator
does not read `rows`.

### Fixtures

```yaml
fixtures:
  model:
    - { for: worker, outcome: correct, turns: 4, builds: 1 }
    - { outcome: question, options: [A, B], default: A, needs_owner: true }
  tools:
    - { tool: build, seconds: 2m }
  principals:
    - { part: owner, option: B, after: 10m }
```

| fixture | key | what |
|---|---|---|
| `model` | `outcome` | how a run ends: `correct`, `wrong` (the run ends failed and the task is worked on again), `false_completion` (reported done, not correct) or `question` (one question, then correct) |
| | `for` | only a run of this agent part takes it; any run by default |
| | `turns` | the model calls of the run; drawn from the profile when not given |
| | `builds` | the builds of the run; 0 by default |
| | `options`, `default`, `needs_owner` | the question's options, its default, and whether it needs the principal behind the agent it asks |
| `tools` | `tool`, `seconds` | `build`: how long the next build takes. The pool's job limit decides whether it ends or is killed |
| `principals` | `part`, `after`, `option` | the next question that reaches this principal is answered `after` it arrives, with `option` (the question's default when not given) |

Runs take the model fixtures in the order they start, builds take the tool fixtures in the order
they start, and questions take the replies in the order they reach the principal. A run with no
model fixture left, and a build with no tool fixture left, draws from the profile. A principal with
no reply left does not answer, and the question takes its default after the architecture's
`question_default_after`. The simulator scripts a run as a whole: `outcome` decides how the run
ends, not each model call.

### Expected events

```yaml
then:
  order: sequence
  events:
    - { refusal: agentic.channel.SendMessage, part: operator, author: owner, rule: control_without_proof }
    - { event: agentic.work.TaskCreated, absent: true }
  kpis:
    tasks_arrived: { max: 0 }
```

An expectation names one block event by its full name (`event`), or one refusal (`refusal`,
`error`, `rule`, or any of them). `order: sequence` (the default) expects each after the one before
it; `order: set` expects each on an event of its own, in any order. An expectation with
`absent: true` holds when no event matches it.

| key | matches |
|---|---|
| `event` | the block event, as a block declares it: `agentic.resource.ClaimGranted` |
| `refusal` | the block command refused: `agentic.resource.RequestClaim` |
| `error` | the block error of a refusal; it must be one the command is refused with: `agentic.resource.NoTimeLimit` |
| `rule` | the rule of the run that refused it (the table above) |
| `part` | the part the event is about |
| `task` | the task: `when.<n>` for an input, the `id` of a given row |
| `options`, `option` | the options of a question, the option of its answer or default |
| `channel`, `author` | the channel and the sender of a message |
| `process` | the process the task follows; the simulator follows none, so this is never checked |

`scenario check` refuses an event no block declares (`unknown_event`, with the nearest declared
name when there is one) and a command or error no block declares for it (`unknown_refusal`). An
expectation that names an `event` and a key of a refusal, or neither, is not a valid scenario.

The simulator records these block events:

| block event | recorded by the simulator |
|---|---|
| `agentic.work.TaskCreated` | yes |
| `agentic.work.RunStarted` | yes |
| `agentic.work.RunEnded` | yes, when a run ends failed |
| `agentic.work.DeliverableSubmitted` | yes, when a reviewer receives it |
| `agentic.work.AssignmentOffered` | yes |
| `agentic.work.TaskCompleted` | yes |
| `agentic.work.DeliverableRejected` | yes |
| `agentic.work.TaskCancelled` | yes |
| `agentic.gateway.CallDecided` | yes, when the rate limit holds a call |
| `agentic.resource.ClaimQueued` | yes |
| `agentic.resource.ClaimGranted` | yes |
| `agentic.resource.ClaimReleased` | yes |
| `agentic.question.QuestionAsked` | yes |
| `agentic.channel.MessageSent` | yes |
| `agentic.question.QuestionAnswered` | yes |
| `agentic.question.DefaultTaken` | yes |
| `agentic.resource.ClaimKilled` | yes |
| `agentic.channel.Posted` | yes |
| `agentic.trigger.ScheduleAdvanced` | yes |
| `agentic.trigger.TriggerFired` | yes |
| `agentic.trigger.FiringQueued` | yes |
| `agentic.trigger.FiringSkipped` | yes |
| `agentic.trigger.FiringStarted` | yes |
| `agentic.trigger.FiringFailed` | yes |

It refuses only `agentic.channel.SendMessage` (an input) and `agentic.channel.TakePost` (a board
claim). An expectation of any other block event, such as `agentic.workspace.ChangeProposed`, or of
a refusal of any other command, is `not_checked`, not failed: a runtime that records it can check
the same scenario. `kpis` bounds any KPI of a simulator run with `min`, `max` or both, inclusive.

## Unavailable parts and dependencies

A scenario can make a part unavailable, or only one dependency of a part: a store, a model route,
an upstream it reaches over a link. `then.parts` then says what each remaining part shows. The
same scenario runs on the simulator now and on a runtime later.

```yaml
unavailable:
  - { part: chat, from: 30m }
  - { part: worker, dependency: builds, from: 0s }

then:
  parts:
    operator:
      views:
        - { view: agentic.channel.ChannelMessages, shows: error, error: agentic.channel.ChannelIsClosed }
      rest: answers
    worker:
      commands:
        - { command: agentic.resource.RequestClaim, shows: refused }
```

| key | required | what |
|---|---|---|
| `part` | yes | the part that is unavailable; with `dependency`, the part that loses it |
| `dependency` | no | a part that `part` depends on: one its links go to or over, one in its `egress`, one that exists once `per` copy of it, or one any of those reach over their links. Only `part` loses it; the other parts still reach it |
| `from` | yes | when it becomes unavailable; `0s` is at the start, before the part listens |
| `for` | no | how long it stays unavailable; to the end of the run by default |

`then.parts` maps a part to what it shows. A part that is unavailable itself shows nothing and is
refused there.

| key | what |
|---|---|
| `views` | a list of `{ view, shows, error }`. `view` is the block name of a view; `shows` is `answer` (no error, rows or none), `empty` (no rows, no error) or `error`; `error`, only with `shows: error`, is the block name of the error the view answers with, any error when left out |
| `commands` | a list of `{ command, shows, error }`. `command` is the block name of a command; `shows` is `accepted` or `refused`; `error`, only with `shows: refused`, is the block name of the error it is refused with, any error when left out |
| `exits` | the status, 1 to 255, the part's process exits with before it listens. Only for a part with a dependency unavailable from `0s`, and with no views, commands or `rest` |
| `rest` | `answers`: every view and command of the part that is not named answers as it does with nothing unavailable |

`check` refuses, each by its path and name: a part or a dependency that is not in the
architecture, a dependency the part does not reach, a part named as its own dependency, a `from`
at or after the horizon, a view or a command that no block declares, an error that no block
declares, and a view or command of a block that is neither the part's own nor one of a part it
depends on. An agent part also shows the views and takes the commands of the blocks it works in:
`agentic.harness`, `agentic.work` and `agentic.question`. A pack block covers every domain under
it (`scm` covers `agentic.scm.change` and the others).

The four kinds of outcome, each with an example in
[`scenarios/workstation/`](https://github.com/beyond10x/agenticspec/tree/main/scenarios/workstation):

| outcome | how it is written | example |
|---|---|---|
| an error inside one view while the rest answers | `views: [{ view, shows: error, error }]` and `rest: answers` | `chat-closed-view-error` |
| an empty view with no error | `views: [{ view, shows: empty }]` | `model-route-down-empty-view` |
| the process exits with a status before it listens | `unavailable` with `from: 0s` and `exits: <status>` | `operator-exits-without-gateway` |
| a dependency only one command reaches: that command is refused, nothing else is missed | `commands: [{ command, shows: refused }]` and `rest: answers` | `build-pool-cut-claim-refused` |

### On the simulator

The simulator makes an entry unavailable where it can model it, and `scenario-result.json` says
how in `unavailable[].simulated`:

| `simulated` | when | what the run does |
|---|---|---|
| `fault` | a pool or a gateway, or a principal; a pool, gateway or principal only `part` reaches; a gateway's only model | the pool or gateway falls to no capacity, the principal is away, for the window |
| `inputs_refused` | a channel; a border part, also one that exits at the start, when the scenario has no given rows; a border part cut from a channel | the inputs that reach the part, come from it or come over the channel in the window are refused by the rule `part_unavailable` |
| `not_modeled` | anything else, or an `inputs_refused` entry while the simulation still records an event of the part or over the channel in the window | none of the expected events and bounds is checked: each is `not_checked` with the reason |

The simulator serves no views, decides no commands of a part and starts no processes, so each
observable of `then.parts` is `not_checked` on it, with the reason. The expected events and KPI
bounds say what the simulator can see of the same outage: an input refused, a claim never
granted, a task that does not complete while another one does.

### On a runtime

A conformance run of a runtime reads the same scenario and writes the same result document, with
the observables checked:

1. It starts the system with every entry that has `from: 0s` in force: the part is stopped, or
   the link from the part to its dependency is cut (the store, the model route, the upstream does
   not answer the part). The other entries come into force at `from` and end after `for`.
2. A part with `exits` must exit with that status and must not listen: no port of it accepts a
   connection. Its status is `held` or `failed`.
3. It sends the inputs of `when` at their times. An input the runtime refuses because a part or
   a link is unavailable counts as a refusal by `part_unavailable`.
4. While the entries are in force it queries each named view of each part of `then.parts` and
   compares: `answer` holds when the view answers with no error; `empty` when it answers with no
   rows and no error; `error` when it answers with an error, the named one when `error` is given.
5. It sends each named command with an input the scenario would accept with nothing
   unavailable: `refused` holds when the command is refused, with the named error when given;
   `accepted` when it is not refused.
6. For `rest: answers` it queries every other view of the part's blocks: each must answer with no
   error. A command of the part that is not named must not be refused because of the entry.
7. It checks the expected events and KPI bounds on the events it records, as the simulator does.

## The result

`scenario-result.json` (format `agenticspec-scenario-result/1`):

```json
{
  "format": "agenticspec-scenario-result/1",
  "scenario": "question-answered",
  "verdict": "held",
  "predicted": "question",
  "counts": { "held": 8, "failed": 0, "not_checked": 0 },
  "expectations": [
    { "index": 4, "expect": { "event": "agentic.question.QuestionAnswered", "part": "owner", "task": "when.0", "option": "B" }, "status": "held", "trace": 6 }
  ],
  "kpi_bounds": [ { "kpi": "questions_forwarded", "min": 1, "max": 1, "value": 1, "status": "held" } ],
  "inputs": [ { "index": 0, "t_seconds": 0, "at": "operator", "kind": "prompt", "act": "send", "author": "owner", "status": "accepted", "rule": null } ],
  "trace": [
    { "t_seconds": 654.63, "event": "agentic.question.QuestionAnswered", "part": "owner", "sim": "question_answered", "task": "when.0", "option": "B" }
  ],
  "kpis": { "tasks_done": { "value": 1, "kind": "Quality" } }
}
```

| field | what |
|---|---|
| `verdict` | `held` when no expectation, no bound and no observable failed |
| `predicted` | the outcome class of the run (below) |
| `expectations` | each expected event with `status` `held`, `failed` or `not_checked`, the index of the trace event it matched, and the reason when it did not hold |
| `kpi_bounds` | each bound with the measured value and its status |
| `inputs` | each input, when it arrived, and whether it was accepted or refused by which rule |
| `trace` | every event of the run in block terms, in time order: the block event (`refusal` for a refusal, with its `command`, `rule` and `error`), the part, the task, and `sim`, the simulator's own event it comes from |
| `kpis` | every KPI of the run |
| `unavailable` | each `unavailable` entry, its window (`from_seconds`, `until_seconds`) and how the simulator made it unavailable: `simulated` is `fault`, `inputs_refused` or `not_modeled` with the reason |
| `observables` | each observable of `then.parts`: the part, `observable` (`view`, `command`, `exits` or `rest`), the expectation as written, its status and the reason when it was not checked |

`counts` adds up the expected events, the bounds and the observables.

## Outcome classes

The class a run predicts is the first that applies:

| class | when |
|---|---|
| `refused` | an input was refused and no task was made |
| `question` | a question was asked |
| `false_completion` | a task was reported done and is not correct |
| `correct` | a task was completed and is correct |
| `wrong` | a run ended failed |
| `open` | none of these within the horizon |

## A task corpus

```yaml
format: agenticspec-corpus/1
corpus: workstation-examples
architecture: { ref: ../../architectures/workstation.yaml, digest: 9f48c790a19597559396e9cb96d13e753d6d7b427c48eaf6bd9a91cd49fdfe99 }
profile: { ref: ../../fields/coding/profiles/coding.yaml, digest: f8f5d0007605f7d4b4da01c9996165d3a93e7e3e81ca23168412fc508bf83335 }
seed: 1
horizon: 4h

entries:
  - scenario: { ref: question-answered.scenario.yaml, digest: 6c675930da48357e350cc404d4a48704d3474261efbe9785eaa73d103524dd03 }
    kind: prompt
    input_digest: 674bead15a2e6fe07d6bffb0d9e6c35e414ddeb8701cd9b3521840636b2d3d62
    expected: question
  - { id: s-0042, kind: control, input_digest: 30c100fc28b40e6128c414cf0d212ffe1de6554a456e952020749b0eb0fc3071, expected: refused }
```

| key | what |
|---|---|
| `format`, `corpus`, `summary` | `agenticspec-corpus/1`, the name, one paragraph |
| `architecture`, `profile`, `seed`, `horizon` | what inline entries run on; needed only when the corpus has inline entries |
| `entries` | one or more entries |

An entry is a scenario reference or an inline entry:

| key | what |
|---|---|
| `scenario` | `{ ref, digest }` of a scenario file, relative to the corpus |
| `id` | the entry's name; a reference is named after its file by default |
| `kind` | the kind of the input; for a reference it must be the kind of the scenario's first input |
| `input_digest` | the sha256 of the input's text; for a reference it must match the text of the scenario's first input |
| `expected` | the outcome class: `correct`, `wrong`, `false_completion`, `question`, `refused` or `open` |
| `weight` | how often a profile draws the entry, against the others; 1 by default |
| `at`, `author`, `act`, `proof` | an inline entry's input: at the architecture's border part (when it has one), from the principal that part acts for, unless given |

`sim corpus run` runs a reference with its fixtures and expectations, and an inline entry as its
one input with no fixtures, so the profile and the architecture decide what happens to it. An
entry holds when none of its expectations failed and its predicted class is its expected class.

## Drawing tasks from a corpus

A behaviour profile draws the outcome of its tasks from a corpus:

```yaml
load:
  tasks_per_hour: 8
  corpus: ../scenarios/workstation/workstation.corpus.yaml
```

Tasks still arrive at `tasks_per_hour`. Each arrival draws one entry by its weight, and the entry's
class decides how the task's first run ends: `correct`, `wrong`, `false_completion` and `question`
as the model fixture of that name, `refused` makes no task, and `open` leaves the run to the
profile. The path is relative to the profile file.

## A draft scenario from a session

```sh
agenticspec blocks scenario <session>.jsonl --architecture architectures/workstation.yaml \
  --profile fields/coding/profiles/coding.yaml --map main=operator,sub=worker --out drafts/session.scenario.yaml
```

`blocks scenario` lifts the session, writes each of its outside inputs as an entry of `when`, and
the block events of the rows each input caused as `then`. An outside input is a prompt of the
main agent by a human (kind `prompt`), a delegate or another session (kind `message`), with proof
`none` and the time since the input before it. A prompt of the harness or a hook, such as a task
notification, is no input. What an input caused is every row lifted from the turn that received it
and from every step after it, up to the next input: a task makes `agentic.work.TaskCreated`, a run
`agentic.work.RunStarted`, a question `agentic.question.QuestionAsked` (and
`agentic.question.QuestionAnswered`), a sent message `agentic.channel.MessageSent`,
each with the input's task, `when.<n>`. A comment beside each entry names the node it comes from.

`--map` names the part each side of the session plays, as `key=part` pairs:

| key | the part of |
|---|---|
| `main` | the main agent; required, and a border part of the architecture, where every input arrives |
| `sub` | every sub-agent |
| `human` | the human who prompts; the part `main` acts for when not given |
| `delegate` | a delegate's prompts; the human's part when not given |
| `peer` | another session's messages |

The draft is checked like any scenario before it is written; a draft the check refuses exits 1
with `scenario_refused` and writes nothing. It prints:

| field | what |
|---|---|
| `out`, `scenario` | the file written and the scenario's name (`--name`, or the file name) |
| `inputs`, `expectations` | the entries of `when` and of `then.events` |
| `unjoined` | inputs no turn received, as `when.<n> (<node>)`: in the scenario, with no expectation |
| `unmapped` | inputs whose author plays no part, as `<node> (<author>)`: left out of the scenario |

Other refusals: `transcript_unreadable`, `document_unreadable` (the architecture or profile),
`map_invalid` (a pair that is not `key=part`, an unknown key, no `main`) and `out_unwritable`.
