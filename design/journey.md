# The user's flow, mapped onto the plan

Status: design note. Each step of the end-to-end flow names the design section or story that
covers it, or "gap" with the story proposed. Lifecycle operations: `design/lifecycle.md` L4.
Deployment: `design/deployment.md`. Story ids are those of the planning store.

## 1. Design the architecture in the studio

| step | covered by | status |
|---|---|---|
| start the studio, open a draft | `story:studio-first-slice` | covered |
| state an intent (an agent for some task, running on another host); an interview turns it into a first draft and its requirements | none: the studio opens a document or an empty draft; nothing asks questions, picks a catalog architecture or writes `requirements:` | **gap**: `studio-intent` |
| the interview as a chat inside the studio | `story:studio-chat` (tagged later, depends on `story:studio-mcp`, `story:studio-editor`, `story:studio-state-space`) | planned, ordered last; the flow prefers it, so it moves up after `story:studio-mcp` |
| an external agent drives the studio through its tool protocol | `story:studio-mcp` (query, apply, check), `story:studio-evaluation` (simulate) | covered |
| every change shown live | `story:studio-live` (the draft's channel, revisions) | covered |
| the studio reloads when spec files change on disk | none | **gap**: `studio-file-follow` (`design/lifecycle.md` L5) |
| a remote, multi-user studio | none | later, out of scope |

## 2. The control plane, inside the studio, with KPIs for the user's requirements

| step | covered by | status |
|---|---|---|
| the control plane of the draft, inside the studio | `design/control-plane.md` C6, P8; proposed `control-plane-pages`, `ui-serve-fixtures`, `sim-event-log`; `story:views-package` | planned (preview on fixtures or a simulator run) |
| the control plane of a running local deployment, inside the studio | `design/control-plane.md` C4 ("place panels in its own app") allows it; nothing plans it | **gap**: `studio-control-plane`: the studio mounts the shell with the local deployment's `operator` ingress as source |
| a requirement becomes a KPI with a target | `story:architecture-kpis` (design KPIs: `agents_total`, `capacity_ratio`); the catalogue (`cache_hit_share`, `cost_usd`) | **gap**: `requirements-kpis` (`design/deployment.md` K12) |
| KPIs shown met or missed | Metrics page, `design/control-plane.md` C3 | partly; status per requirement in `control-plane-units` |
| a KPI of the infrastructure (build or worker node count) | cloudspec's catalogue | **gap** outside this repository: request V7 (`design/deployment.md` § Conditions) |

## 3. Refine by hill climbing until the gaps are closed; commit and push

| step | covered by | status |
|---|---|---|
| search the marked places, accept a variant | `story:studio-evaluation`, `story:studio-state-space`; command line `agenticspec sim run` | planned |
| structural changes in the search | `story:architecture-patches` | planned |
| stop when every requirement is met | none: the search maximizes weights under gates (`experiments/*.yaml` `search`) | **gap**: `requirements-kpis` (requirements as constraints) |
| commit and push | none: drafts persist (`story:studio-drafts`) but are never written back | **gap**: `studio-release` (`design/lifecycle.md` L4, L5) |

## 4. Test once locally, then deploy to a target

| step | covered by | status |
|---|---|---|
| test in the simulator | `story:scenario-format`, `story:scenario-simulator-run`, `story:task-corpus` | planned |
| test on a real local runtime | `design/architecture-runtime.md` § "The smallest first runtime"; no story | **gap**: `first-runtime` |
| a test record before a release | none | **gap**: `studio-verification` (`design/lifecycle.md` L3) |
| slice parts into processes, containers, services, jobs | none | **gap**: `packing-format`, `packing-admission` (`design/deployment.md` K2-K5) |
| recurring schedules as jobs | `architectures/scheduled-loops.yaml` simulated; no unit | **gap**: the `scheduled` unit (K3, K4), in `packing-format` |
| is a target always a namespace | `design/deployment.md` K8: yes, one scope per deployment | decided here |
| deploy to this host or a remote cluster namespace | none here; cloudspec `epic:realization` (draft) | **gap**: `deployment-domain`, `cloud-translation`; requests V1-V8 to cloudspec |
| agents run as Loom agents on weft | `design/deployment.md` K8 | **gap**: `agent-run-lift`; weft and cloudspec realizations |
| deploy from the studio | none | **gap**: `studio-deploy` |
| the running units on the control plane | `design/control-plane.md` C3 Operations | **gap**: `control-plane-units` |

## Proposed stories, by where they are defined

- `design/deployment.md`: `packing-format`, `packing-admission`, `deployment-domain`,
  `cloud-translation`, `requirements-kpis` (extends `story:architecture-kpis`),
  `control-plane-units` (extends `control-plane-pages`), `first-runtime`, `agent-run-lift`.
- `design/lifecycle.md`: `studio-verification`, `studio-release`, `studio-file-follow`
  (extends `story:studio-live`), `studio-deploy`.
- Here: **studio-intent**: an interview from a stated intent to a first draft and its
  `requirements:`, as tool calls of one session, starting from a catalog architecture or an empty
  draft; new, after `story:studio-mcp`. **studio-control-plane**: the studio mounts the control
  plane shell of a local deployment; extends `story:views-package`.
- Order change: `story:studio-chat` loses its "later" tag and follows `story:studio-mcp`.
