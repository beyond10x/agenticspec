---
format: aep.planning-md/3
id: story:block-bindings
kind: story
status: active
title: Declare the cross-block causation of the blocks as ESS bindings
summary: blocks/ gets ESS bindings for the commands one block causes in another; every binding ESS cannot express is a named gap with a reproduction.
relations:
- decomposes: epic:architecture-contract
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: blocks
- confidence: inferred
  path: design/ess-repro
- confidence: cited
  path: docs/blocks
- confidence: cited
  path: scripts/check-layers.sh
- confidence: cited
  path: scripts/fixtures/layers
- confidence: cited
  path: website/src/generated
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:33:36Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-27T23:33:36Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

The cross-block causation of the building blocks is declared as ESS bindings in `blocks/`, so a
command one block causes in another is internal behaviour and no longer looks like an entry from
the outside world. `blocks/` declares no binding at this commit (no `bindings:` section in any
file). Where ESS 0.99.2 cannot express a binding the blocks need, the exact gap is written into
`epic:architecture-contract` § "What ESS lacks", with a reproduction, instead of a workaround.
Every candidate below ends in exactly one of three states, written as one line in
`epic:architecture-contract` § "Cross-block causation": a binding, a gap, or dropped with a reason.
Design: `epic:architecture-contract` § "Ports are derived, not authored".

## Scope

- `blocks/components.yaml` (new): the components the bindings need (one owning all 22 domains,
  unless a binding needs two) and the `bindings:`, each with `when.event`, `invoke.command`,
  `mapping`, `delivery` and `on_failure` (ESS guide "Components and bindings").
- `blocks/ess-inputs.yaml`, `blocks/system.yaml`: list the new file as ESS requires.
- Block files whose events must carry a field a binding maps (inferred: `blocks/2-intelligence/`,
  `blocks/4-agent/harness.yaml`, `blocks/5-work/work.yaml`, `blocks/6-governance/authority.yaml`,
  `blocks/7-collaboration/`), each change the smallest that lets the mapping type-check.
- Candidates, read from the protocols of the design's port table, each with an id the epic's
  § "Cross-block causation" uses:
  - `C1` gateway `CallDecided` (an admitted call) causes model `StartInference`;
  - `C2` model `InferenceFinished` causes gateway `CompleteCall`;
  - `C3` harness `ToolCallRequested` causes authority `DecideRequest`;
  - `C4` the authority's decision causes harness `DecideToolCall`;
  - `C5` trigger `TriggerFired` causes the trigger's `RecordStart`;
  - `C6` a started firing (`FiringStarted`) causes work `StartRun` or harness `OpenSession`;
  - `C7` work `RunStarted` and harness `SessionOpened` (which causes which);
  - `C8` question `QuestionAsked` over a channel causes channel `SendMessage`;
  - `C9` resource `ClaimGranted` and runtime `StartInstance` where a pool runs workloads;
  - `C10` Clock: a schedule of the trigger block as an ESS periodic host cause, or the gap (a
    periodic cause has a fixed `every:`, a schedule's period is data).
  The list is a starting point. The story adds a causation it finds in the commands' comments as
  `A1`, `A2`, ..., and drops a candidate the blocks do not imply. Each candidate and each added
  causation ends as exactly one of:
  - `binding <name>`: declared in `blocks/components.yaml`;
  - `gap`: an entry in § "What ESS lacks" naming the binding and the missing construct, with
    `design/ess-repro/<gap>/`, a minimal system that `ess specify validate` refuses or that
    synthesis handles wrongly, as the entry says;
  - `dropped`: the reason the blocks do not imply it (the event's comment, or the command
    already caused another way), in one sentence.
- `epic:architecture-contract` § "Cross-block causation" (through `aep plan artifact body
  --section`): one line per `C<n>` and `A<n>` with its state.
- `scripts/check-layers.sh` and a negative fixture under `scripts/fixtures/layers/`: a binding
  counts as a dependency of the emitting event's domain on the invoked command's domain (the
  cause holds the effect, as the higher block holds the identifier of the lower one), so a
  binding from a lower layer's event to a higher layer's command is refused.
- `blocks/scenarios/`: an authored scenario for a binding whose scenario synthesis refuses.
- Generated: `docs/blocks/` and `website/src/generated/blocks.json` (`task docs:generate`).
- `epic:architecture-contract` § "What ESS lacks" (through `aep plan artifact body --section`)
  and `design/ess-repro/<gap>/` per gap.
- `CHANGELOG.md`.
- Time synthesis before and after (`scripts/check-blocks-conform.sh --slow`, 4 jobs; 29 s for
  692 scenarios at the epic's commit). A jump over 60 s is fixed in this story. No test runs over
  60 s; the test step stays within 120 s; a pool job over 10 minutes is killed.

## Shared surfaces

- `blocks/`: `story:port-derivation` reads the compiled system and digests `blocks/`;
  `story:contract-derivation` and `story:contract-conformance` read `blocks/` through it.
  Outside this epic, `story:architecture-patches` and `story:guards-and-limits` change block
  files; whichever lands second rebases on the other.
- `docs/blocks/`, `website/src/generated/`: `story:contract-docs` (its page generation).
- `epic:architecture-contract` § "What ESS lacks": `story:contract-conformance` may add an entry.
- `CHANGELOG.md`: every story of this epic.

## Acceptance

- Given this story done, when `epic:architecture-contract` § "Cross-block causation" is read, it
  has exactly one line for each of `C1` to `C10` and for each added causation, and each line is
  `binding <name>`, `gap` or `dropped` with a reason.
- Given `blocks/` at this story's commit, when `ess specify compile --path blocks --format json`
  runs, every line marked `binding <name>` has that binding in its output, and the binding's
  invoked command is no longer an entry of the compiled system (a binding causes it).
- Given every line marked `gap`, when § "What ESS lacks" is read, it has one entry naming the
  binding and the missing construct, and `ess specify validate` (or synthesis) on that entry's
  `design/ess-repro/<gap>/` fails as the entry describes.
- Given the bindings, when `task check:blocks` runs, it exits 0 with every scenario passing, the
  binding scenarios (`<binding>/binding/...`) among them, and the step ends within 120 s on the
  cached suite.
- Given a fixture under `scripts/fixtures/layers/` with a binding from an event of layer 2 to a
  command of layer 7, when `task check:layers` runs, the fixture is refused (exit 1) and the real
  blocks pass.

## Not in this story

Deriving ports from the bindings (`story:port-derivation`); any runtime; changing a block's
behaviour beyond what a binding's mapping needs.
