---
format: aep.planning-md/3
id: review-result:architecture-contract-acceptance-1
kind: review-result
status: active
title: 'architecture-contract plan, acceptance critic, round 1: needs-revision'
owner: aep:plan-critic-acceptance
relations:
- reviews: epic:architecture-contract
- reviews: story:block-bindings
- reviews: story:port-derivation
- reviews: story:contract-derivation
- reviews: story:contract-docs
- reviews: story:contract-conformance
- reviews: story:routing-invariants
revision: 1
---
needs-revision

Seven findings, listed here and repeated in the findings block below:

- `epic:architecture-contract` — the epic has no `## Acceptance` section, so only an assertion could close it. Its headings are Outcome, Ports are derived, Nouns, Stories, Port derivation, What ESS lacks and Not in this epic. Every other epic in the store has one (`building-blocks`, `docs-site`, `simulator-and-search`, `evaluation-loop`, `research-blocks`). — .engineering/planning/epic/architecture-contract.md:13
- `story:routing-invariants` — the acceptance runs `agenticspec-sim trace`, a binary that is going away, so the check names a command that will not exist. The same stale command is in the scope bullet, where the acceptance and the scope both need to say `agenticspec sim trace`. — .engineering/planning/story/routing-invariants.md:83
- `story:block-bindings` — the acceptance requires every candidate without a binding to have a "What ESS lacks" entry. The scope lets the story drop a candidate the blocks do not imply, and adds a Clock item and any causation it finds. A dropped candidate has neither a binding nor an honest gap entry, and the Clock item and added causations are not "candidate above". — .engineering/planning/story/block-bindings.md:85
- `story:contract-derivation` — the first acceptance bullet joins about eight independent outcomes: the eight ports, the three absent ports, 12 `tree` instances, the `box` settings, the border and the invariant. One can pass while another fails. — .engineering/planning/story/contract-derivation.md:94
- `story:contract-derivation` — the scope says the story settles three `UNMAPPED:` markers in `architecture.yaml` and removes them, but no acceptance line can show this. Nothing checks that the markers are gone, so the epic's "settled by the story named there" can be asserted and never checked. — .engineering/planning/story/contract-derivation.md:53
- `story:contract-conformance` — the acceptance requires `ess verify conform run --target interpreted` to run the cut suite. The scope (line 48) says `ess` may refuse a suite with scenarios removed, and then the story selects by scenario id instead. The acceptance names no observable for that branch, so on it the story can never meet it as written. — .engineering/planning/story/contract-conformance.md:74
- `story:contract-docs` — the outcome promises a "Contract" section on every one of the 10 architecture pages and 6 catalog entries. The acceptance checks `workstation` only, so nothing observable shows the other nine pages and five entries changed. — .engineering/planning/story/contract-docs.md:84

What I read: 7 of 7 ids (the epic and the six stories), in full with `aep plan artifact show`. I also ran `aep plan artifact kinds`, `aep plan artifact lifecycle story` and `aep plan artifact lifecycle epic`, `aep plan artifact validate` (valid), and `git grep`/`ls` on the tree for the named architectures, profile, scripts, `design/ess-repro` and the port-table names (`InferenceStarted`, `FinishInference`).

What I could not establish:
- Whether `agenticspec sim trace` exists. `spec/cli.yaml` is not on this branch, so I took your word that `sim` is a command group on the branch about to merge.
- Whether the derivation will give the design's 15 ports. That is the work of `story:port-derivation`, not something I could run.
- Out of my lane: `contract-derivation` and `contract-conformance` list `story:contract-docs` and `story:contract-conformance` as dependencies that look chosen for shared-surface ordering. That is a question for the parallel-safety and design critics.

```findings
- file: .engineering/planning/epic/architecture-contract.md
  line: 13
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the epic has no Acceptance section (headings: Outcome, Ports are derived, Nouns, Stories, Port derivation, What ESS lacks, Not in this epic), so nothing observable says when the epic is done, unlike every other epic in the store"
- file: .engineering/planning/story/routing-invariants.md
  line: 83
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance runs `agenticspec-sim trace`, a binary that is going away, so it names a command that will not exist; the scope bullet at line 54 repeats it and both need `agenticspec sim trace`"
- file: .engineering/planning/story/block-bindings.md
  line: 85
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance requires every candidate without a binding to have a 'What ESS lacks' entry, but the scope lets the story drop a candidate the blocks do not imply and adds a Clock item and any causation it finds, so a dropped or added item has no checkable end state"
- file: .engineering/planning/story/contract-derivation.md
  line: 94
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the first acceptance bullet joins about eight independent outcomes (eight ports, three absent ports, 12 tree instances, box settings, border, invariant), so one can pass while another fails"
- file: .engineering/planning/story/contract-derivation.md
  line: 53
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the scope says the story settles and removes the UNMAPPED markers in architecture.yaml, but no acceptance line shows they are gone, so the transition from marked to settled is unobservable"
- file: .engineering/planning/story/contract-conformance.md
  line: 74
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance requires `ess verify conform run --target interpreted` to run the cut suite, but the scope (line 48) allows that ess refuses it and the story selects by scenario id; the acceptance names no observable for that branch"
- file: .engineering/planning/story/contract-docs.md
  line: 84
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the outcome promises a Contract section on every one of the 10 architecture pages and 6 catalog entries, but the acceptance checks only workstation, so the other pages and entries have no observable check"
```
