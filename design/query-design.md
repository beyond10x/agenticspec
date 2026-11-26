# sq: a query language over the session IR — design (draft, 2026-11-26)

Status: proposed. Design only: no code, no stories. Inputs: the session IR
(`spec/domains/session.yaml`), `design/design.md`, one session lifted with `sessionspec lift
--output json` (shapes only), shellspec's design (`../shellspec/design/design.md`), cq of codespec
and codesec, and the formats Runbook and Canon execute. Every claim about a sibling repository
cites `file:line` relative to the directory that holds the organization's repositories side by
side; inference is labelled.

## 1. Why

A query language over the session IR finds sessions by their patterns: a refusal followed by a
retry, a sub-agent result nobody read, a cache bust after an idle gap. Patterns that recur can be
mined automatically and lifted into procedures, which export as runbooks Canon can execute.

```text
lifted sessions ──replay──▶ events ──facts──▶ fact base ──sq──▶ rows, verdicts
                                                   │
                                                   └──mine──▶ patterns ──lift──▶ procedure
                                                                                    │
                                     protocol/1 + loom.bindings/1 + canon-evidence/1 ◀──export
```

codespec holds `cir/1`, the readers and the cq engine for code. Here the IR, the engine and the
authored checks all stay in sessionspec.

## 2. What the IR gives today

One lifted session (`sessionspec lift <main transcript> --output json`) has this shape:

```text
{ ok, result: { session_id, agents: <count>, refused: <count>,
  commands: [ { seq, at, actor, command, outcome, input: { … } } ] } }
```

`input` holds the fields of the command as `spec/domains/session.yaml:352-777` declares them:
`Content` is `{digest, bytes}` (session.yaml:93-103), `Usage` the five token counts
(session.yaml:105-119), a decision has `decision`, `decider` and a free-text `reason`
(session.yaml:636-649). The lifted list carries the command and its outcome; the events come from
replaying it through the generated component (`crates/sessionspec/src/replay.rs:100`). An event
carries no `actor` and no `seq` (session.yaml:779-865), so the fact base joins each published event
with the command that emitted it.

Note on the sample: of the 17 main transcripts under 300 KB on this host, none held a tool call; the
shapes above come from one of 828 KB with sub-agents, tool calls, hook refusals and a compaction.

What the queries of § 4 need and the IR does not hold yet (requests to the session domain, not
edits; another session owns `spec/`):

| gap | needed by | proposal |
|---|---|---|
| no sub-commands of an `Execute` call | `cd-build-tail`, `red-green-loop`, `build-outside-slot` | `sub_commands: [Invocation]` from shellspec (shellspec/design/design.md:344-358); sessionspec/AGENTS.md already says it will |
| no `cwd` on `RequestToolCall` | shellspec's `cwd` per invocation (shellspec/design/design.md:354) | additive `cwd`, path relative to the repository root as redaction writes it (sessionspec/design/design.md:101) |
| no link from a `Spawn` call to the agent it started; `StartAgent` names the parent only (session.yaml:405-438, claude.rs:581-587) | `fanout-unread` | additive `spawned_by: Optional<ToolCallId>` on `StartAgent` |
| no background flag on a tool call | `fanout-unread`, blocking waits | additive `background: Boolean` |
| `Content` has no class, though the redaction design names one (sessionspec/design/design.md:97) | `human-correction` | `PromptClass` on `SubmitPrompt` (§ 9, Q4) |
| `reason` is free text | grouping refusals by rule | a reason class next to the text |

Until a field exists, the fact that rests on it is `unk` (§ 5.3), never guessed.

## 3. Decision

**sq is a dialect of cq: the same typed Datalog with named-field record patterns, `?-` queries,
names lists, aggregates, path patterns and properties (codespec/docs/query-language.md:47-56,
158-239), over a fact base derived from the session IR, plus one sugar of its own, `seq`, for
ordered steps within a time bound.**

Why cq and not a new language:

- Every example of § 4 is a join plus an order plus, often, a negation ("a result the parent never
  read"). cq is built for "a closure plus a negation" (codespec/docs/query-language.md:71-73).
- cq already has the two answers this domain needs: rows with `definite` / `possible` certainty
  and the unknown sites they rest on (codespec/crates/query/src/lib.rs:508-579), and properties as
  small automata with at most 3 registers (codespec/docs/query-language.md:202-223).
- One language for code and sessions: a session that edits code can later be joined with the
  `cir/1` of that code (inference: same engine, two fact bases; not designed here).

What changes against cq:

- The fact base (§ 4.1). The edge relations are time and agent order, not control flow and calls.
- `path(A -> B, over: next)` walks one agent's command order; `over: spawn` walks the agent tree.
- `seq` (§ 4.3), because "A, then B within 10 min, at most 3 steps apart" is the common question.
- Properties take `scope: agent` or `scope: session`, and a terminal state `match` beside cq's
  `error`, so a property can find a wanted shape as well as a violation.
- Durations are literals: `30s`, `5m`, `1h`. Instants are milliseconds; no clock is read.

## 4. Queries

### 4.1 The fact base

One fact base per lifted session; a corpus query runs over many and unions the rows. Every command
is a node `<session>#<seq>`. A relation is keyed by the node of the command that produced it.

From the lifted commands and their published events:

| relation | fields | from |
|---|---|---|
| `session` | `id, harness, opened_at, ended_at, profile` | `SessionOpened`, `SessionEnded`; `profile` from the corpus selection (sessionspec/design/design.md:131-134) |
| `agent` | `id, session, parent, role, depth, started_at, ended_at` | `AgentStarted`, `AgentEnded`; `depth` derived |
| `cmd` | `node, session, agent, seq, at, actor, kind, outcome` | every command: `kind` is the command name, `outcome` its outcome |
| `prompt` | `node, agent, author, attribution, class, bytes, digest, mid_turn` | `PromptSubmitted`; `mid_turn` true when the agent was `Working` (session.yaml:469-471) |
| `call` | `node, id, agent, tool, category, bytes, digest, cwd, background` | `ToolCallRequested` |
| `decided` | `node, call, decision, decider, reason` | `ToolCallDecided` |
| `completed` | `node, call, ok, bytes, digest, dur` | `ToolCallCompleted`; `dur` from the request's `at` |
| `turn` | `node, agent, model, ctx, uncached, cache_read, write_5m, write_1h, output` | `ModelTurnRecorded` |
| `turn_end` | `node, agent` | `TurnEnded` |
| `compacted` | `node, agent, before` | `ContextCompacted` |
| `exposure` | `node, id, agent, channel, class, kind, alias, verdict, call` | `ExposureDetected` |
| `refused_cmd` | `node, error` | a command replay refused: every row over that agent rests on it |

Derived by the engine:

| relation | meaning |
|---|---|
| `at(N, T)` | the instant of node `N` |
| `next(A, B)` | `B` is the next command of `A`'s agent, in that agent's record order (sessionspec/design/design.md:13-14) |
| `before(A, B)` | `A` precedes `B`: by `seq` within one agent, by `at` across agents; equal instants across agents are possible both ways |
| `turn_of(N, P)` | node `N` belongs to the turn the prompt `P` started |
| `interval` | `agent, state, on, from, to, dur, open`: the Agent lifecycle (`Idle`, `Working`) as a step function, with `on` the waiting-on value of sessionspec/design/design.md:86 (`human`, `tool`, `sub_agent`, `peer`, `rate_limit`, `build_queue`); `open` when the transcript ends inside it |
| `bust` | `node, agent, cause, written`: the cache-bust rule of `kpi.rs:460-485` as a relation, causes `TtlExpired`, `ModelChanged`, `Compacted`, `Unexplained` |
| `spawned(C, Child)` | the `Spawn` call `C` started agent `Child`: from `spawned_by` when the IR has it, else the nearest `StartAgent` of a child of the caller after `C` (possible, § 5.3) |

From shellspec, one set per `Execute` call (shellspec/design/design.md:146-167, 263-333):

| relation | fields |
|---|---|
| `inv` | `id, call, index, program, sub, category, operation, role, pipeline, cwd, remote, context` |
| `inv_wrapper(I, W)` | `slot`, `timeout`, `env`, … |
| `inv_effect(I, E)` | `fs.write`, `net.out`, `remote.write`, … |
| `inv_flag(I, F)` | flag names of static words |
| `inv_guard(I, Op, J)` | `I` runs only if `J` succeeded (`and`) or failed (`or`) |
| `inv_pipe(I, J)` | `J` is the next stage after `I` in one pipeline |
| `shape(C, S)` | `shellspec shape` of the call: program, subcommand path, flag names, word kinds, no values |

`shellspec shape` is the redacted form (shellspec/design/design.md:363-367), so every relation above
exists on redacted IR. A dynamic program or a parse failure is `program: unk` or category `unk`
(shellspec/design/design.md:129-144) and makes the rows over it possible.

### 4.2 Syntax

cq's rule syntax unchanged (codespec/crates/query/src/lib.rs:53-79): `head :- body.`, record
patterns `rel{field: V}`, `!p(X)`, `~"re"`, `X in @names.list`, `N = count{…}`, `?- body.`.

```
// a rule
refused_by_hook(C, R) :- call{node: C, id: I}, decided{call: I, decision: "Refuse", decider: "Hook", reason: R}.

// a query: prints its bindings, sorted
?- refused_by_hook(C, R).

// an ordered sequence: each element the nearest later node of the scope that satisfies its atoms
?- seq(A, B, C, scope: agent, within: 10m, gap: 3), call{node: A, category: "Execute"}, ….

// a property: an automaton over the commands of each agent, ending in match or error
property name(scope: agent) { start -> s1: <record pattern> => r := X  …  s2 -> match: <pattern> }

// a check: a named query with an expectation, giving ok / gap / unk per session
check name { about "…" severity medium expect none profiles [autonomous] find rule_name }
```

### 4.3 `seq`

`seq(N1, …, Nk, scope: agent | session, within: D, gap: G, pick: nearest | all)` holds when the
nodes are strictly ordered by `before`, all in one agent (`scope: agent`, the default) or one
session, the span from `N1` to `Nk` is at most `D`, and at most `G` other commands of the scope
lie between two neighbours. With `pick: nearest`, the default, `Ni+1` is the nearest later node
that the rest of the rule allows; `all` gives every ordered tuple. It compiles to rules over
`next` and `at` with a `min` aggregate per element, one stratum above the element's atoms, as
cq's path pattern sits above its body (codespec/crates/query/src/lib.rs:92-95).

### 4.4 Eight example queries

**1. `cache-bust-after-idle`.** The first model turn after an idle gap longer than the cache TTL
writes the whole context again.

```
idle_bust(A, Gap, Written) :-
    interval{agent: A, state: "Idle", to: T, dur: Gap, open: false}, Gap > 5m,
    prompt{node: P, agent: A}, at(P, T),
    seq(P, B, scope: agent), turn{node: B, agent: A},
    bust{node: B, cause: "TtlExpired", written: Written}.
?- idle_bust(A, Gap, W).
```

**2. `refused-then-retried`.** A hook refused a tool call and the agent asked for the same call
again: the refusal taught nothing.

```
retried(C1, C2, Reason) :-
    call{node: C1, id: I, digest: D},
    decided{call: I, decision: "Refuse", decider: "Hook", reason: Reason},
    seq(C1, C2, scope: agent, within: 10m), call{node: C2, digest: D}.
// same shape, other arguments: replace the digest join with shape(C1, S), shape(C2, S)
?- retried(C1, C2, R).
```

**3. `fanout-unread`.** Three or more sub-agents spawned in one turn, and the parent never took a
model turn after a child ended: its result was never read.

```
spawns(P, C) :- call{node: C, category: "Spawn"}, turn_of(C, P).
fanout(P, N) :- prompt{node: P}, N = count{ spawns(P, _) }, N >= 3.
read_after(A, E) :- turn{node: M, agent: A}, at(M, T), T > E.
fanout_unread(P, C, Child) :-
    fanout(P, _), spawns(P, C), call{node: C, agent: A},
    spawned(C, Child), agent{id: Child, ended_at: E}, !read_after(A, E).
?- fanout_unread(P, C, Child).
```

**4. `cd-build-tail`.** `cd X && cargo test 2>&1 | tail -n N`: a navigate, a build or test
guarded by it, piped into `tail`. Output cut before the agent reads it; the count per session is
the KPI.

```
cd_build_tail(C, Prog) :-
    inv{id: I1, call: C, category: "navigate"},
    inv{id: I2, call: C, category: ~"^(build|test)$", program: Prog},
    inv_guard(I2, "and", I1),
    inv_pipe(I2, I3), inv{id: I3, program: "tail"}.
?- session{id: S}, N = count{ cmd{node: C, session: S}, cd_build_tail(C, _) }, N > 0.
```

**5. `human-correction`.** The human corrects the agent: a prompt classed as a correction, or a
human prompt typed into a running turn shortly after an edit or a command.

```
correction(P) :- prompt{node: P, author: "Human", class: "Correction"}.
correction(P) :- prompt{node: P, author: "Human", mid_turn: true}.
?- correction(P), seq(X, P, scope: agent, within: 2m), call{node: X, category: ~"^(Edit|Execute)$"}.
```

Until the IR has `PromptClass`, the first rule rests on the missing field: its rows are possible
(§ 5.3); the second is definite.

**6. `waiting-on-human`.** The main agent of an autonomous session sat idle on the human for more
than 30 minutes.

```
?- session{id: S, profile: "autonomous"}, agent{id: A, session: S, role: "Main"},
   interval{agent: A, state: "Idle", on: "human", dur: D}, D > 30m.
```

An open interval (the transcript ends while idle) is definite once its observed part exceeds 30
minutes, else possible.

**7. `red-green-loop`.** A test fails, the agent edits, the same test shape runs again until it
passes: the shape a bug-fix runbook formalises (runbook/protocol/sdc-bugfix.protocol.yaml:6-16).

```
test_run(C, Ok, S) :- inv{call: C, category: "test"}, shape(C, S),
                      call{node: C, id: I}, completed{call: I, ok: Ok}.
property red_green(scope: agent) {
  start  -> red:    test_run{ok: false, shape: S} => s := S
  red    -> fixing: call{category: "Edit"}
  fixing -> red:    test_run{ok: false, shape: s}
  fixing -> match:  test_run{ok: true, shape: s}
  message "red at {start}, green at {match}, test {s}"
}
?- red_green(Start, At, Msg).
```

**8. `build-outside-slot`.** A build, test or lint that does not go through the build pool, against
the host's rule that every build uses the pool. `cargo` and `go` go through a shim, so their absence of the
wrapper is not a finding.

```
names shim = ["cargo", "go"].
outside_slot(C, P) :-
    inv{id: I, call: C, category: ~"^(build|test|lint)$", program: P},
    !inv_wrapper(I, "slot"), !(P in @names.shim).
check build-outside-slot {
  about "a build, test or lint started without the slot wrapper"
  severity low
  expect none
  find outside_slot
}
```

## 5. Evaluation

### 5.1 Input and order

- Input is the redacted IR of one or more sessions, replayed through the generated component; the
  fact base reads the published events and the command list, never a transcript
  (sessionspec/AGENTS.md, rule "KPIs read the events the generated component published").
- Within one agent, order is the agent's record order (`seq`), which replay keeps even where
  timestamps are out of order (sessionspec/design/design.md:13-14). Across agents, order is by `at`.
- Durations come from `at` only. No clock, network or model call: same input and query, same bytes.

### 5.2 Rows

A query gives rows, sorted by their values (codespec/docs/query-language.md:271-273). Each row
carries:

| field | what |
|---|---|
| `values` | the query's variables in order of first occurrence |
| `session`, `agent` | where the row is |
| `span` | first and last node of the match and their instants |
| `certainty` | `definite`, or `possible` with the unknown sites it rests on |
| `unknown` | `[{node, reason}]`: why it is possible |

Report format `sessionspec-query-report/1`, the shape of `codespec-query-report/1`
(codespec/crates/query/src/lib.rs:202, 546-579) plus `session`, `agent` and `span`. Text output:
one line per row, values tab-separated, then `possible: <node> (<reason>)`.

### 5.3 Unknown

A fact is possible, never absent and never false, when it rests on:

- a field the IR does not hold yet (§ 2): `prompt.class`, `call.background`, `call.cwd`;
- an inferred link: `spawned` without `spawned_by`; an author with attribution `Inferred`
  (session.yaml:53-59);
- a decision `NotRecorded` (session.yaml:69-73);
- a shellspec `unk`: dynamic program, parse failure, depth (shellspec/design/design.md:129-141);
- two nodes of different agents at the same instant, for `before`;
- a command replay refused (`refused_cmd`).

Negation follows cq: `!p(X)` is definite when no fact satisfies it and none of the possible facts
would; otherwise possible, naming the sites (codespec/crates/query/src/lib.rs:31-51). This is the
workspace's rule that unknown differs from false (aep/AGENTS.md, invariant "Unknown differs from
false"; canon/AGENTS.md, "`UNKNOWN` is not `FALSE`").

### 5.4 Verdicts

A `check` turns rows into one verdict per session, the `ok` / `gap` / `unk` of the workspace's
checkers (aep/docs/design/transcript-conformance-design-v0.1.md:82-95) and of the session IR's own
`Verdict` (session.yaml:87-91):

| `expect` | `ok` | `gap` | `unk` |
|---|---|---|---|
| `none` (an anti-pattern) | no row | a definite row | only possible rows |
| `some` (a required shape) | a definite row | no row, and no possible one | only possible rows |
| `count <op> N` | the count of definite rows holds and so does every count the possible ones allow | it fails on definite rows alone | it depends on possible rows |

A `gap` cites its rows' spans; an `unk` names the unknown sites. A check names the profiles it
applies to; in a session of another profile it is not evaluated and the report lists it as not
applicable, which is not `ok`. Exit codes follow the transcript checker
(aep/docs/design/transcript-conformance-design-v0.1.md:707-715): 0 every verdict `ok`, 1 a `gap`, 3
no gap and an `unk`, 2 an error of the query or the input. A plain `?-` query has no verdict: exit 0
with rows or without.

### 5.5 Cost

- The fact base is built once per redacted session and cached on disk keyed by the digest of the
  redacted IR, the sessionspec version and the shellspec ruleset digest, as cq caches
  (codespec/docs/query-language.md:80-85): `$XDG_CACHE_HOME/sessionspec/facts/<digest>.facts`.
- Tests build their fixture fact bases once per run into `target/` and read them; fixtures are
  synthetic (sessionspec/AGENTS.md). Each test stays under 60 s and the test step under 120 s
  (workspace `AGENTS.md` § Tests stay fast).
- A corpus query is a loop over per-session fact bases; only aggregates and mining join across
  sessions. Not measured; `sessionspec kpi` over 3,309 transcripts runs in 12 s
  (sessionspec/design/design.md:6-8), the scale the corpus query aims at (inference).

## 6. Pattern mining

Goal: find command sequences that recur across sessions without anyone writing the query, and lift
one into a parameterised procedure.

### 6.1 Alphabet

Each node becomes a token at a chosen level of abstraction. The token never holds content.

| level | an `Execute` call becomes | other nodes |
|---|---|---|
| `coarse` | the shellspec category of `dominant(invocations)` (shellspec/design/design.md:355-358): `exec:test` | `edit`, `read`, `spawn`, `prompt:human`, `refuse:hook`, `bust`, `idle>5m` |
| `shape` | the invocation chain: `exec:navigate&&test|filter` | tool name and category |
| `fine` | program, subcommand and flags: `exec:cd&&cargo-nextest-run|tail` | plus the result: `ok` / `fail` |

### 6.2 Mining

1. **Sequences.** One sequence per agent turn (`turn_of`), tokens in agent order, each token with
   its node.
2. **Frequent n-grams.** Contiguous runs of 2 to 8 tokens, support counted in distinct sessions,
   not in occurrences. Cheap and deterministic.
3. **Gapped sequences.** A PrefixSpan-style search over the same sequences with `max_gap` tokens
   between neighbours and `within` a time bound, the semantics of `seq` (§ 4.3), so every mined
   pattern is also an sq query.
4. **Closed only.** A pattern is kept when no longer pattern has the same support.
5. **Score.** Support (sessions, repositories), tokens and wall time spent inside matches, share of
   matches that end with `ok` results, refusals inside. Ties sort by token string.

Output: `sessionspec-patterns/1`, one entry per pattern with its tokens, gaps, support, score, the
sq query it compiles to and the digest of that query. Matches are listed by node, never by text.

### 6.3 Lifting a pattern into a procedure

Over the matches of one pattern, field by field (anti-unification):

- **Constant** across matches (the program, the subcommand, a flag): a literal of the step.
- **Varying** (the repository, a package, `tail -n N`): a parameter, typed by the shellspec word
  kind (`path_like`, `flag_value`, `positional`).
- **Shared** between steps of one match (the same `cwd` in the `cd` and the test, the same shape in
  the red and the green run of § 4.4 example 7): one parameter used by both steps.
- **Loops** (a red-green property that matched with several reds): a step with a retry, its bound
  the largest number of rounds observed.
- **Results**: each step's `completed.ok` becomes the result table of the step (`ok` / `failed`).
- **Durations**: the largest observed `dur` of a step, as its timeout (inference: a bound from the
  data, to be reviewed).

The result is a procedure, `sessionspec-procedure/1`: parameters, steps (kind `program`,
`judgement`, `person` or `child`; argv template from the shape; category; effect; result table;
retry and bound; timeout; the steps it waits on), outcomes, and provenance (pattern id, query
digest, the session ids it came from). A mined procedure is a draft; a person or the operator
session promotes it. Promotion is the human step of runbook's `record` skill, which today has "no
recorder, no generator and no importer" (runbook/plugin/skills/record/SKILL.md:8): mining fills its
step 2, the table of steps and facts (record/SKILL.md:35-49).

## 7. Export

### 7.1 Targets

| target | format | evidence |
|---|---|---|
| the protocol | Canon `protocol/1` | `canon/crates/canon/src/model/parse.rs:8`; the document `canon/crates/canon/src/model/mod.rs:71-107` |
| the bindings Runbook runs it with | Loom `loom.bindings/1` | `loom/crates/loom-bindings/src/lib.rs:26`; members `loom/crates/loom-bindings/src/decode.rs:14-47`; an instance `runbook/bindings/sdc-bugfix.yaml:1-144` |
| a session judged against the protocol | Canon `canon-evidence/1` records, one case `canon-case/1` | `canon/crates/canon/src/model/evidence.rs:12-60`; `canon/crates/canon/src/model/case.rs:16-49` |

Runbook runs a case tree on Canon protocols through Loom: "reads `canon-decision/1`, runs the tool
bound to one admissible action, records `canon-evidence/1`, evaluates again"
(runbook/AGENTS.md, "What it is"; runbook/README.md:45-46). So "a runbook" is a `protocol/1` file
plus a `loom.bindings/1` file. "Execute in Canon" means evaluate: Canon decides what is known,
admissible and earned for one case (canon/README.md:5-8); it runs no tool. The third target lets
Canon evaluate a recorded session as a case of an exported protocol: did the session do what the
runbook asks.

### 7.2 Field mapping

| IR or mined fact | procedure | `protocol/1` | `loom.bindings/1` |
|---|---|---|---|
| pattern id, description, provenance | `id`, `description` | `protocol.id`, `revision: 1`, `description` (mod.rs:149-155); provenance in the description, since the model refuses unknown fields (mod.rs:73) | `protocol: <id>@1` (decode.rs:14) |
| shared `cwd`, the files edited | artifact | `artifacts` (mod.rs:178-188) | `artifacts: {<id>: <path>}` (decode.rs:14; sdc-bugfix.yaml:4-10) |
| varying parameter with a closed set of observed values | parameter | `case.inputs` with `values` (mod.rs:125-144) | none |
| varying free parameter (a package, a path) | parameter | action `arguments`, a JSON Schema (mod.rs:282-287) | `arguments` / `arguments_from` (decode.rs:18-19) |
| one step: an `Execute` call's invocations | step, kind `program` | `actions.<id>` (mod.rs:252-306), `author: program` (mod.rs:295-300) | `executor: {kind: program, argv}` (decode.rs:27-40; sdc-bugfix.yaml:28-30) |
| `shellspec` category | effect | `effect`: `read` for read, list, search, build, test, lint; `write` for edit; `external` for `remote.write` effects (EffectClass is an open name, ids.rs:178-181; values as runbook/plugin/skills/record/SKILL.md:64) | none |
| a model's `Edit` or `Write` calls between two program steps | step, kind `judgement` | action with `author: judgement`, `revises: [<artifact>]` (mod.rs:295-305) | `executor: {kind: program, argv: [sdc-node, …]}` as Runbook's model step (sdc-bugfix.yaml:13-15; inference: `model` from `RecordModelTurn.model`) |
| `completed.ok` per step | result table | `evidence_kinds.<id>` (mod.rs:193-223); a claim `true_when: evidence {kind, result}` (mod.rs:228-232) | `evidence: [{kind, subject, producer, from: exit, results: {"0": ok, "*": failed}}]` (decode.rs:41; sdc-bugfix.yaml:89-95) |
| order of steps in the matches | `after` | `precondition` over claims (mod.rs:256); an order Canon must prove: `runs_only_when` (mod.rs:257-268) | none: Canon decides the order |
| a loop (red, edit, red, …, green) | retry, bound | the writer's `precondition` admits it again while the claim is false or unknown (sdc-bugfix.protocol.yaml:173-184) | `retry_when: {<kind>: [failed]}`, `budget: {max_runs: N}` (decode.rs:24, 42-47; sdc-bugfix.yaml:71-75) |
| largest observed `dur` | timeout | none | `timeout_seconds` (decode.rs:23) |
| `DecideToolCall` by `Human`, `Accept` | step, kind `person` | `requires: [{capability}]` (mod.rs:270, 350-361) | `executor: {kind: person, principal}` (decode.rs:34; runbook/docs/loom-bindings.md:49) |
| `Spawn` call and its child's matched pattern | step, kind `child` | `case_open: {protocol, subject}` (mod.rs:288-294, 339-348) | `executor: {kind: open, name \| each, bindings}` (decode.rs:35-37; runbook/docs/loom-bindings.md:61) |
| last step `ok`, every step `ok` | outcome `done` | `outcomes.<id>.requires: all: [claims]` (mod.rs:387-395) | none |
| a step that ended `failed` after its bound | outcome `not_done` | an outcome over the claim `is: "false"` (sdc-bugfix.protocol.yaml:318-340) | none |
| a later edit of an artifact a check read | invalidation | `invalidation: {upstream, invalidates}` (mod.rs:397-411) | `upstream: [<artifact>]` on the entry (decode.rs:41) |
| wrappers `slot`, `timeout`, `env` | dropped, recorded in the step | none | none: the runner owns pool and timeout (inference) |
| a hook refusal inside the matches | none | none: reported beside the export | none |
| `Exposure` | never exported | none | none |
| one matched session (third target) | none | a case `canon-case/1` with each artifact's `revision` (case.rs:21-49) | none |
| each matched step's result | none | one `canon-evidence/1` record: `kind`, `result`, `subject`, `subject_revision`, `observed_at` from `at`, `producer` from the actor (evidence.rs:19-49) | none |

The IR holds no artifact revisions; for the third target a revision is `<session>#<seq>` of the
last edit before the record (inference, Q8).

### 7.3 Path of an export

`sessionspec export <procedure> --out <dir>` writes `<id>.protocol.yaml` and `<id>.yaml` into a
directory given by the caller, never into a sibling repository. They become a runbook by
runbook's own steps: `runbook canon validate` and `runbook canon check` on the draft
(runbook/plugin/skills/record/SKILL.md:87-99), then the move into `protocol/` and `bindings/` and
the `harden` skill (record/SKILL.md:101-107). Export runs `canon validate` on what it wrote and
fails on a refusal.

## 8. ESS first: the nouns

Declared in a new domain `sessionspec.query` before code uses them (sessionspec/AGENTS.md,
"Specification first"). Listed, not written:

| noun | kind | holds |
|---|---|---|
| `Module` | entity | one `.sq` source: digest, rules, properties, checks, names lists |
| `Check` | entity | name, about, severity, `expect`, profiles, the rule it finds |
| `NamesList` | value | name and entries; replaceable by a file |
| `Expectation` | enum | `None`, `Some`, `Count` |
| `Corpus` | entity | a selection of sessions: window, projects, profiles, the digest of the redacted IR set |
| `FactBase` | entity | digest, session, sessionspec version, shellspec ruleset digest; lifecycle Built, Stale |
| `Relation` | value | name and field schema, printed by `--schema` |
| `Row` | value | values, session, agent, span, certainty, unknown sites |
| `Certainty` | enum | `Definite`, `Possible` |
| `UnknownSite` | value | node and reason (closed list of § 5.3) |
| `CheckResult` | value | check, session, `Verdict` (reuse `sessionspec.session.Verdict`), cited rows |
| `QueryReport` | value | `sessionspec-query-report/1` |
| `Token`, `AlphabetLevel` | value, enum | § 6.1 |
| `Pattern` | entity | tokens, gaps, within, support, score, compiled query digest; lifecycle Mined, Promoted, Rejected |
| `Match` | entity | pattern or check, session, span, bindings by node |
| `Procedure` | entity | parameters, steps, outcomes, provenance; lifecycle Draft, Validated, Exported, Retired |
| `Parameter` | value | name, word kind, constant / varying / shared, observed value set when closed |
| `Step` | value | kind (`Program`, `Judgement`, `Person`, `Child`), argv template, category, effect, result table, retry bound, timeout, after |
| `Export` | entity | procedure, target, path, digest, validation result |
| `ExportTarget` | enum | `CanonProtocol`, `LoomBindings`, `CanonEvidence` |
| `PromptClass` | enum | for the session domain: `Task`, `Correction`, `Approval`, `Question`, `Continue`, `Unknown` |

Commands: `CompileModule`, `BuildFactBase`, `RunQuery`, `RunCheck`, `MinePatterns`,
`PromotePattern`, `RejectPattern`, `LiftProcedure`, `ExportProcedure`. CLI (an `ess-cli/2` binding,
as `spec/cli.yaml` is): `sessionspec query`, `sessionspec check`, `sessionspec mine`,
`sessionspec export`.

## 9. Open, with the default taken

- Q1. Language. A: a cq dialect (§ 3). B: a sequence-regex language (`MATCH_RECOGNIZE` style).
  Default A: one language across the workspace, and `seq` covers the regex case.
- Q2. Engine. A: depend on codespec's `crates/query` by git revision once it takes a fact base
  from outside, a story filed in codespec. B: a copy in sessionspec. Default A; no copy. Until the
  seam exists, sq is a design.
- Q3. Raw or redacted. Default: redacted IR only. No sq predicate reads text; shapes, digests,
  lengths and classes are all a query sees. Raw transcripts stay on the host (sessionspec/AGENTS.md).
- Q4. Prompt class. Default A: a rule-based classifier run by the redaction pass on the host, no
  model call, tri-valued; `Unknown` when no rule matches. B: a model call, refused by the test rule
  (no model call in the test step).
- Q5. Spawn link. Default A: ask for `spawned_by` on `StartAgent`; until then `spawned` is inferred
  from order and every row over it is possible.
- Q6. Mining thresholds. Default: support at least 5 sessions in at least 2 repositories, tokens at
  `shape` level, `within: 15m`, `max_gap: 3`. Changed only with a corpus measurement.
- Q7. Where patterns live. Default: authored modules in `queries/*.sq` of this repository; mined
  patterns and procedures as redacted data under `research/patterns/`, never transcript text.
- Q8. Revisions for the third target. Default: `<session>#<seq>` of the last edit before the
  record, labelled inferred in the export; a real tree digest when the IR gets one.
- Q9. Export destination. Default: a directory the caller names; nothing written into `runbook/` or
  `canon/` by sessionspec. Landing there is runbook's `record` steps 5 and 6.
- Q10. Cross-agent ties. Default: equal instants across agents order both ways, possible; no
  tie-break by agent id.
