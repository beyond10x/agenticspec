---
format: aep.planning-md/3
id: review-result:architecture-contract-acceptance-2
kind: review-result
status: active
title: 'architecture-contract plan, acceptance critic, round 2: needs-revision'
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

Six of the seven round-1 findings are resolved. The seventh is only partly fixed, and I found one new gap. Each round-1 finding, in order:

- **`epic:architecture-contract`, no Acceptance section: resolved.** It now has eight lines, each a command with an exit status or count. I checked the UNMAPPED count by running the command. It prints 3 now, as the line says.
- **`story:routing-invariants`, stale `agenticspec-sim trace`: resolved.** Scope, outcome and acceptance now say `agenticspec sim trace`, and a line runs `task check:generated`. `profiles/coding.yaml` exists. No `agenticspec-sim <cmd>` text is left in the bodies.
- **`story:block-bindings`, dropped and added candidates: resolved.** Candidates are numbered C1 to C10, added ones A1 onward, and each ends as `binding`, `gap` or `dropped`. The acceptance has one line per end state.
- **`story:contract-derivation`, joined first bullet: resolved.** It is now separate lines for the port set, absent ports, `tree`, `box`, border, invariant and protocol. I checked these against `architectures/workstation.yaml`. The ports and counts agree with the design table and the 12 `tree` instances.
- **`story:contract-derivation`, unobservable UNMAPPED removal: partly resolved.** The `sed … | grep -c UNMAPPED` line (3 before, 0 after) is observable. The same sentence adds "and the comments there state what was taken", which nothing can check. It is the first finding below.
- **`story:contract-conformance`, fallback branch: resolved.** Path A and path B each have an acceptance line, and path B names the refusal quoted in "What ESS lacks". A further line requires `docs/contract.md` to name the path taken.
- **`story:contract-docs`, only workstation checked: resolved.** The docs test covers all 10 pages and 6 entries and asserts the counts. A separate line counts 16 contract files.

- `story:contract-derivation` — the UNMAPPED acceptance line ends "and the comments there state what was taken", a second outcome no command or reader can check twice, so drop it or name what must appear in the comment — .engineering/planning/story/contract-derivation.md:150
- `story:routing-invariants` — the scope says "A predicate form the evaluator cannot evaluate is reported as not checked, not as held", but no acceptance line covers it, so all four lines pass even if an unevaluable invariant is reported as held — .engineering/planning/story/routing-invariants.md:105

What I read: 8 of 8 ids. These were the epic, the six stories and `review-result:architecture-contract-acceptance-1`, each read in full with `aep plan artifact show`. I also ran `aep plan artifact validate` (valid), the UNMAPPED `sed | grep -c` command (prints 3), and `ls`/`grep` on `architectures/`, `profiles/`, `docs/`, `Taskfile.yml`, `design/architecture-runtime.md` and `blocks/`.

What I could not establish:
- Whether `agenticspec sim trace` and `sim ports` exist. I found no `sim` entry in `spec/cli.yaml` on this branch. Commit 33c0392 is not in this tree, so I took your word that it is merging.
- Whether the derivation gives the design's 15 ports. That is the work of `story:port-derivation`.
- Whether `ess` admits a cut suite. The path A or path B choice in `story:contract-conformance` settles it.
- Out of my lane: several acceptance lines join two checks, for example `story:port-derivation` line 1, which checks the model port and also that no configuration port appears. I did not count these because each is one observable run.

```findings
- file: .engineering/planning/story/contract-derivation.md
  line: 150
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the UNMAPPED acceptance line ends 'and the comments there state what was taken', a second outcome that no command or reader can check twice; drop it or name what must appear in the comment"
- file: .engineering/planning/story/routing-invariants.md
  line: 105
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the scope says 'A predicate form the evaluator cannot evaluate is reported as not checked, not as held' (line 65) but no acceptance line covers it, so all four lines pass even if an unevaluable invariant is reported as held"
```
