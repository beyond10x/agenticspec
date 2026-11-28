---
format: aep.planning-md/3
id: story:task-corpus
kind: story
status: active
title: A task corpus as simulator and scenario input
summary: agenticspec-corpus/1 lists concrete tasks; a profile's load may be a corpus, and a scenario input may name a corpus task (consumer gap M6)
relations:
- decomposes: epic:architecture-scenarios
- depends_on: story:scenario-format
scope:
- confidence: inferred
  path: corpora
- confidence: inferred
  path: crates/agenticspec-sim/src/corpus.rs
- confidence: cited
  path: crates/agenticspec-sim/src/profile.rs
- confidence: inferred
  path: crates/agenticspec-sim/src/scenario
- confidence: cited
  path: crates/agenticspec-sim/src/sim.rs
- confidence: cited
  path: crates/agenticspec-sim/tests/cases
- confidence: inferred
  path: docs/scenarios.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 10}
---
## Outcome

A task corpus is an input of the simulator and of scenarios. A consumer's gap report (M6) asked
for a corpus of concrete tasks as simulator and test input; at `25fbfcb` a behaviour profile
states load only as a rate, `tasks_per_hour` or the older `tasks_per_hour_per_worker`
(`profiles/coding.yaml:70-71`; `crates/agenticspec-sim/src/profile.rs:188-195`, read at
`profile.rs:356-362`), and every task's size is a draw from its role's distributions
(`profiles/coding.yaml:8-20`).

`agenticspec-corpus/1` lists tasks: an id, the role that takes it, its size (turns, steps,
builds, questions), an optional arrival offset, and its origin (authored, or extracted with a
reference to the evidence it came from). A profile may give `load: { corpus: corpora/<x>.yaml }`
as a third form beside the two rates; the simulator then takes tasks from the corpus in order,
each with its own size instead of a draw. A scenario's `when` may name a corpus task as an input.

## Scope

- `crates/agenticspec-sim/src/profile.rs`: the corpus form of `load`, refused beside a rate as the
  two rates are refused together today (`profile.rs:356-362`).
- `crates/agenticspec-sim/src/corpus.rs` (new): parse and check a corpus; `src/sim.rs`: arrivals
  and task sizes from the corpus.
- `crates/agenticspec-sim/src/scenario/`: a `when` input `{ task: <corpus id> }` with the
  scenario's corpus named beside its architecture.
- `corpora/example.yaml` (new); `docs/scenarios.md` and the profile's format on the docs.
- `crates/agenticspec-sim/tests/cases/`; builds and tests through `slot`, no test over 60 s, the
  test step within 120 s, a pool job over 10 minutes is killed.

## Shared surfaces

- `crates/agenticspec-sim/src/sim.rs`: `story:simulator-fidelity`, `story:routing-invariants`,
  `story:scenario-simulator-run`, `story:guards-and-limits`; whichever lands second rebases.
- `crates/agenticspec-sim/src/profile.rs`, `profiles/coding.yaml`: `story:simulator-fidelity`.
- `crates/agenticspec-sim/src/scenario/`, `docs/scenarios.md`: `story:scenario-format` (comes
  first; this story reads its format), `story:scenario-simulator-run`.

## Acceptance

- Given `corpora/example.yaml` with three tasks of 10, 40 and 160 turns and a profile with `load:
  { corpus: corpora/example.yaml }`, `agenticspec-sim run` starts exactly three tasks and their
  turns are 10, 40 and 160, where before the profile refused a load without a rate.
- Given a profile with both `corpus` and `tasks_per_hour`, it is refused with a message naming
  both.
- Given a corpus with two tasks of one id, it is refused with `duplicate_task` naming the id.
- Given a scenario whose input is `{ task: t2 }` of its corpus, `agenticspec-sim scenario check`
  exits 0; given `{ task: t9 }`, not in the corpus, it exits 1 with `unknown_corpus_task`.
- Given the same corpus, profile and seed, the result is the same bytes at any thread count.

## Not in this story

Filling a corpus from observed sessions (`epic:evaluation-loop`, `story:scenario-input-extraction`
writes inputs, a corpus may be written from them later); fitting the profile's distributions from a
corpus.
