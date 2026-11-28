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

`task check:scenarios` runs every `scenarios/*/*.scenario.yaml` and every
`scenarios/*/*.corpus.yaml`.

A refused command exits 1 and prints `<code>: <detail>` to standard error:

| code | when |
|---|---|
| `document_unreadable` | a file cannot be read or is not YAML |
| `scenario_invalid` | the scenario is not a valid `agenticspec-scenario/1` document; every problem is named by its path, such as `when.0.atx: unknown key` |
| `scenario_refused` | the scenario does not fit its architecture: a digest does not match, an input is not at a border part, an author or a fixture names no part, or the architecture itself is refused |
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
architecture: { ref: ../../architectures/workstation.yaml, digest: 76b32d4bc28eac17ce6ff3d7464f69ab96218d764dc0a5ceb298ecc5029e8ad7 }
profile: { ref: ../../profiles/coding.yaml, digest: f9986e82bfc8e4b0c14f2d871e437ed19689aae2a450bae1a9f006e0589c6802 }
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
    - { event: run_started, part: worker, task: when.0 }
    - { event: question_asked, part: worker, task: when.0, options: [A, B] }
    - { event: question_asked, part: operator, task: when.0 }
    - { event: post_sent, part: operator, channel: chat }
    - { event: question_answered, part: owner, task: when.0, option: B }
    - { event: task_completed, part: worker, task: when.0 }
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
| `then` | no | the expected events and KPI bounds |

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

The run admits an input or refuses it by a rule:

| rule | refused when | block error |
|---|---|---|
| `input_without_link` | no link from the author to the part carries the input: a prompt needs a `prompt` link, an order an `order` link, a message or a control input any link | `agentic.channel.NotAParty` |
| `control_without_authority` | a control input comes from an author that is not the principal the part acts for (`acts_for`) | `agentic.delegation.NotTheDelegator` |
| `control_without_proof` | a control input carries no proof | none: no block refuses a message without proof |
| `claim_conflict` | a copy claims a board entry another copy claimed first | `agentic.channel.PostAlreadyTaken` |
| `corpus_refused` | a corpus entry a profile drew was refused, so it makes no task | none |

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
    - { event: refusal, part: operator, author: owner, rule: control_without_proof }
    - { event: task_created, absent: true }
  kpis:
    tasks_arrived: { max: 0 }
```

`order: sequence` (the default) expects each event after the one before it; `order: set` expects
each on an event of its own, in any order. An expectation with `absent: true` holds when no event
matches it.

| key | matches |
|---|---|
| `event` | the event's name, from the table below |
| `part` | the part the event is about |
| `task` | the task: `when.<n>` for an input, the `id` of a given row |
| `rule` | the rule of a refusal |
| `options`, `option` | the options of a question, the option of its answer or default |
| `channel`, `author` | the channel and the sender of a message |
| `process` | the process the run follows; the simulator models no process, so this is never checked |

| name | block event | recorded by the simulator |
|---|---|---|
| `task_created` | `agentic.work.TaskCreated` | yes |
| `run_started` | `agentic.work.RunStarted` | yes |
| `run_ended` | `agentic.work.RunEnded` | yes, when a run ends failed |
| `assignment_offered` | `agentic.work.AssignmentOffered` | yes |
| `deliverable_submitted` | `agentic.work.DeliverableSubmitted` | yes, when a reviewer receives it |
| `deliverable_rejected` | `agentic.work.DeliverableRejected` | yes |
| `task_completed` | `agentic.work.TaskCompleted` | yes |
| `task_cancelled` | `agentic.work.TaskCancelled` | yes |
| `proposal_opened` | `agentic.workspace.ChangeProposed` | no |
| `proposal_merged` | `agentic.workspace.ProposalMerged` | no |
| `question_asked` | `agentic.question.QuestionAsked` | yes |
| `question_answered` | `agentic.question.QuestionAnswered` | yes |
| `default_taken` | `agentic.question.DefaultTaken` | yes |
| `post_sent` | `agentic.channel.MessageSent` | yes |
| `board_posted` | `agentic.channel.Posted` | yes |
| `approval_granted` | `agentic.authority.ApprovalGranted` | no |
| `approval_rejected` | `agentic.authority.ApprovalRejected` | no |
| `call_decided` | `agentic.gateway.CallDecided` | yes, when the rate limit holds a call |
| `claim_queued` | `agentic.resource.ClaimQueued` | yes |
| `claim_granted` | `agentic.resource.ClaimGranted` | yes |
| `claim_released` | `agentic.resource.ClaimReleased` | yes |
| `claim_killed` | `agentic.resource.ClaimKilled` | yes |
| `schedule_advanced` | `agentic.trigger.ScheduleAdvanced` | yes |
| `trigger_fired` | `agentic.trigger.TriggerFired` | yes |
| `firing_queued` | `agentic.trigger.FiringQueued` | yes |
| `firing_skipped` | `agentic.trigger.FiringSkipped` | yes |
| `firing_started` | `agentic.trigger.FiringStarted` | yes |
| `firing_failed` | `agentic.trigger.FiringFailed` | yes |
| `refusal` | a refused input or claim, with its rule | yes |

An expectation of an event the simulator does not record is `not_checked`, not failed: a runtime
that records it can check the same scenario. `kpis` bounds any KPI of a simulator run with `min`,
`max` or both, inclusive.

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
    { "index": 4, "expect": { "event": "question_answered", "part": "owner", "task": "when.0", "option": "B" }, "status": "held", "trace": 6 }
  ],
  "kpi_bounds": [ { "kpi": "questions_forwarded", "min": 1, "max": 1, "value": 1, "status": "held" } ],
  "inputs": [ { "index": 0, "t_seconds": 0, "at": "operator", "kind": "prompt", "act": "send", "author": "owner", "status": "accepted", "rule": null } ],
  "trace": [
    { "t_seconds": 654.63, "event": "agentic.question.QuestionAnswered", "name": "question_answered", "part": "owner", "sim": "question_answered", "task": "when.0", "option": "B" }
  ],
  "kpis": { "tasks_done": { "value": 1, "kind": "Quality" } }
}
```

| field | what |
|---|---|
| `verdict` | `held` when no expectation and no bound failed |
| `predicted` | the outcome class of the run (below) |
| `expectations` | each expected event with `status` `held`, `failed` or `not_checked`, the index of the trace event it matched, and the reason when it did not hold |
| `kpi_bounds` | each bound with the measured value and its status |
| `inputs` | each input, when it arrived, and whether it was accepted or refused by which rule |
| `trace` | every event of the run in block terms, in time order: the block event, its name, the part, the task, and `sim`, the simulator's own event it comes from |
| `kpis` | every KPI of the run |

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
architecture: { ref: ../../architectures/workstation.yaml, digest: 76b32d4bc28eac17ce6ff3d7464f69ab96218d764dc0a5ceb298ecc5029e8ad7 }
profile: { ref: ../../profiles/coding.yaml, digest: f9986e82bfc8e4b0c14f2d871e437ed19689aae2a450bae1a9f006e0589c6802 }
seed: 1
horizon: 4h

entries:
  - scenario: { ref: question-answered.scenario.yaml, digest: b3308e370fbc4042ab6da7f240873b17903f5fa917ee360d0c5fc30311ddb75a }
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
