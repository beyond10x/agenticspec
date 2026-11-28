---
format: aep.planning-md/3
id: epic:architecture-scenarios
kind: epic
status: active
title: 'Architecture scenarios: given state and inputs, expected block events, a task corpus'
summary: agenticspec-scenario/1 states the starting rows, border inputs, fixtures and expected block events of one run; the simulator runs it and reports predicted per expectation; a task corpus feeds it
relations:
- depends_on: epic:simulator-and-search
- depends_on: epic:building-blocks
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:09Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:09Z", actor: "human:operator", revision: 3}
---
## Outcome

An architecture can be given a scenario: a document that states the starting state in block
rows, the inputs that arrive at its border parts, the fixtures that stand in for external parts,
and the block events that must follow, refusals included. The simulator runs a scenario and
reports, per expectation, what it predicted. The scenario's `then` is the oracle: a run passes
when its events meet the `then`, and nothing else is judged. A task corpus gives the simulator
and the scenarios concrete tasks instead of a rate.

Source: a consumer's gap report (M5, M6, M9, M10), and `design/architecture-runtime.md`
§ "Three documents, kept apart" (the scenario, `agenticspec-scenario/1`, lines 31-42) and
§ "What the runtime emits, and what conformance means" (scenario conformance, lines 123-134). At
`25fbfcb` no scenario format exists; the behaviour profile states load only as a rate
(`profiles/coding.yaml:70-71`; `crates/agenticspec-sim/src/profile.rs:188-195`).

## Nouns (ESS)

The scenario, its inputs, fixtures and expectations, and a corpus task are new nouns. They are
modelled in ESS before a story of this epic leaves draft (planning guardrail 7);
`story:scenario-format` carries that model as its first acceptance line. Open point, settled
there: a new domain of layer 10 (a new block, with its docs page and at least two KPIs,
`scripts/check-kpis.sh`) or entities in `agentic.experiment`
(`blocks/10-evaluation/experiment.yaml`).

## Stories

| promise | story |
|---|---|
| `agenticspec-scenario/1` and its check; the `then` as the oracle (M5, M10) | `story:scenario-format` |
| the simulator runs a scenario and reports predicted per expectation (M9 simulator side, M10) | `story:scenario-simulator-run` |
| a task corpus as simulator and scenario input (M6) | `story:task-corpus` |

Order: `scenario-format` first; `scenario-simulator-run` and `task-corpus` read its format.
`scenario-simulator-run` also reads the port-level trace of `story:routing-invariants`
(`epic:architecture-contract`). From `epic:evaluation-loop`, `story:session-lift` and
`story:scenario-input-extraction` write rows and scenarios in this format.

## Acceptance

- Given `story:scenario-format` merged, `agenticspec-sim scenario check` accepts every file under
  `scenarios/` and refuses each malformed fixture of its tests with a named rule.
- Given a scenario of `architectures/workstation.yaml` and `profiles/coding.yaml`, `agenticspec-sim
  scenario run` exits 0 with every expectation `met`, and the same seed gives the same bytes at
  any thread count.
- Given a profile whose load is a corpus, the simulator takes its tasks from the corpus.
- `task check` passes with the test step within 120 s and no test over 60 s.

## Not in this epic

- The runtime side of replay (M9): running the same scenario against a runtime and filling the
  observed column. That is the epic after `epic:architecture-contract` (runtimes and bindings
  against contracts), which also owns bindings (`agenticspec-binding/1`) and admission.
- M17 of the same report: later, it needs a runtime.
- Extracting scenarios from observed sessions (`epic:evaluation-loop`).
