---
format: aep.planning-md/3
id: epic:architecture-deployment
kind: epic
status: draft
title: 'Architecture deployment: packing, admission and the deployment bundle'
summary: A binding's packing groups parts into units; admission and agenticspec deploy bundle write agenticspec-deployment/1 for cloudspec to translate and a target to run; the in-process runtime is the local test (K1-K16).
relations:
- depends_on: epic:architecture-contract
- depends_on: epic:ingress
- informed_by: epic:control-plane
- informed_by: epic:events-and-storage
revision: 1
---
## Outcome

An admitted architecture reaches a running target. agenticspec writes one deployment bundle
(`agenticspec-deployment/1`) from the released architecture, its contract and a binding whose
`packing:` section groups parts into units; a cloudspec inbound translator turns the bundle into a
cloudspec architecture; weft runs that architecture, and every other target (a Helm chart, a
Compose file) is generated from it. agenticspec keeps what only it knows: packing, the contract,
admission of agentic semantics and the bundle, and writes no file of a cloudspec or target format.
Until that chain exists, the in-process runtime of `design/architecture-runtime.md` is the local
test. This is the runtime epic `epic:architecture-contract` defers to.

Design: `design/deployment.md` (decisions K1-K16, open points Q1-Q17). Approved as written, every
open point at its recommended option; weft accepts Q16 A and Q17 A.

## Decisions taken

- K1: the words pack, unit, bundle, translate, realize, project, deploy.
- K2: packing is a section of `agenticspec-binding/1`, never of the architecture;
  `agenticspec packing default` prints it.
- K3: unit kinds `core`, `agent`, `service`, `job`, `scheduled`, `store`, `model`, `external`.
- K4: strategies `single` (default), `by_kind`, `by_part`, `groups`; per part `schedule: job` and
  `external`; `per` parts follow their part.
- K5: admission of a packing and its refusals, every finding listed.
- K6: `agenticspec deploy bundle` writes one YAML file with every member inline, keyed by digest;
  no secret, host, provider or target name.
- K7: per unit name, kind, parts, replicas, code, claims, isolation, checks, schedule, secrets,
  egress, roles, ports and source; abstract roles, never a provider.
- K8: the target is cloudspec's and always one scope per deployment; an agent unit stays an agent
  workload and the target binds the role `agent-runtime`.
- K9: the boundary table with cloudspec and the targets.
- K10: the domain `agentic.deployment` (`Deployment`, `Unit`), its states and refusals; a `Units`
  view on the Operations page.
- K11: local test, then promotion as a new deployment of the same revision.
- K12: a top-level `requirements:` key, KPIs with a target, decided at check, evaluation or
  operation; the search treats them as constraints.
- K13: what the translator maps, in cloudspec.
- K14: who admits what: agenticspec (`NotAdmitted`), the translator (`CoverageRefused`), the target
  (`TargetRefused`).
- K15: what comes back: coverage, unit observations, agent runs (lifted into block events), KPI
  series; the `core` unit reads them.
- K16: fixture bundles with `bundles/sources.sha256` and a copy of cloudspec's accepted kinds and
  roles here; the equality test in cloudspec.
- Q1 A (bundle), Q2 A (one YAML file), Q3 A (`agenticspec deploy bundle`), Q4 A (a core runtime
  image by digest), Q5 A (the `core` unit reads), Q6 A (the target forwards agent run records), Q7 A
  (`core` starts jobs through the target), Q8 A (role `agent-runtime`), Q9 A (packing in the
  binding), Q10 A (`single` everywhere), Q11 A, Q12 A (the binding names the agent document by
  digest), Q13 A (one deployment per scope), Q14 A (`requirements:` key), Q15 A (one-way simulation
  coupling), Q16 A (`claims` gains `volumes`), Q17 A (agenticspec keeps its isolation levels;
  cloudspec's accepted list maps them).

## Stories, in order

1. `story:first-runtime`: the in-process runtime, with the first `agenticspec-binding/1` and
   `admit`.
2. `story:packing-format`: the `packing:` section, unit kinds, strategies, `agenticspec packing
   default`.
3. `story:packing-admission`: K5's refusals and the plan's units.
4. `story:deployment-domain`: `agentic.deployment`, states, refusals, views, scenarios.
5. `story:requirements-kpis`: the `requirements:` key and its stages.
6. `story:deployment-bundle`: `agenticspec deploy bundle`, the format, digests, per-unit fields and
   roles.
7. `story:bundle-fixtures`: fixture bundles, their digests and the accepted-list check.
8. `story:control-plane-units`: the `Units` view and requirement status.
9. `story:agent-run-lift`: agent run records lifted into block events.

Every story that runs tests keeps each test within 60 s and its test step within 120 s. A job in
the build pool ends within 10 minutes.

## Requests to cloudspec

Filed in cloudspec under its `epic:translation`; not stories here. Nothing is requested of weft
directly.

- C1 `story:architecture-format-constructs`: an architecture format with K13's constructs.
- C2 `story:translate-in-agenticspec`: the inbound translator of `agenticspec-deployment/1`, with
  K13's mapping, coverage outcomes, role binding per role and the grant rule (depends on cloudspec
  `story:agent-workload-kind`).
- C3 `story:source-field`: a source field on every object, kept into target objects and
  observations.
- C4 `story:accepted-kinds-and-roles`: a published list of accepted unit kinds and roles with
  refusal names, and `scope` as the target boundary.
- C5 `story:observation-contract`: per unit release state, instances, health, failures, drift, and
  a resumable feed per instance of a workload's own records; weft fulfils it.
- C6 `story:external-dependencies-and-kpi-ids`: external dependencies with egress routes; KPI ids
  qualified by system.
- C7 `story:agenticspec-fixture-bundles`: the vendored fixture bundles with their lock and coverage
  test.
- The load hand-off of Q15 (agent runs to cloud load, one way).

On the weft side: `story:agent-architecture-run` runs a vendored fixture bundle before the
translator exists, `story:agenticspec-render-contract` frames `agenticspec sim render` output per
part, and `story:agent-loom-runtime` binds `agent-runtime` to Loom.

## Risks

No remote target runs before cloudspec has a format, translator and realization and weft has code;
the remote path waits on cloudspec. The copy of cloudspec's accepted list can go stale; the
vendored fixtures catch it on the cloudspec side. `story:first-runtime` carries the first binding
format and admission because no other story owns them.

## Not in this epic

Any file of a cloudspec or target format; the translator; realization on a target; enforcement of
requirements by a target.
