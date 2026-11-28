---
format: aep.planning-md/3
id: review-result:architecture-contract-scope-2
kind: review-result
status: active
title: 'architecture-contract plan, scope critic, round 2: approve'
owner: aep:plan-critic-scope
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
approve

Round-1 finding, resolved: the protocol promise ("the settings that narrow each of them, the protocol of each port", epic line 16) is now claimed by two stories.
- `story:contract-derivation` writes each port's `protocol` into `agenticspec-contract/1`. It copies the value from `blocks/ports.json`. Its acceptance has a line that compares it with the port's protocol there.
- `story:contract-docs` shows a protocol column in the Contract table, and its docs test checks the protocol per row.
- The epic's Nouns and Stories table state the same, and the fix is recorded as `review_outcome` evidence on both artifacts.

I found no new gap, reach beyond the parent, double claim or narrowing, so there are no findings.

**What you read:** 7 artifacts (the epic and the 6 stories), plus `review-result:architecture-contract-scope-1` and the design-1 result. I ran `aep plan artifact show` on the epic and each story, `aep plan artifact graph`, and `grep` on `scripts/check-layers.sh` and the epic.

I extracted 16 promises from the epic's Outcome, Nouns, Stories table and Acceptance, and traced all 16 to a story:
- ports derived from blocks and compared with the design's 15-row table
- contract per document, with ports, settings, protocol, invariants, instances and border
- predicate language parsed
- docs sections on 10 pages and 6 catalog entries, with the download
- conformance cut of the suite
- invariants checked against a trace
- bindings and `C1` to `C10`
- the "What ESS lacks" section
- the UNMAPPED count going to 0
- `check:ports`, `check:docs` and `check:generated` in `task check`

The ladder page and the runtime are excluded by the epic's own "Not in this epic".

**What you could not establish:**
- Unease, not a finding: `story:block-bindings` extends `scripts/check-layers.sh` with a layering rule for bindings and a negative fixture, and no parent sentence asks for it. `grep -i binding scripts/check-layers.sh` shows the script does not read bindings today. The rule keeps the epic's own precondition (`task check:layers` passes) true once bindings exist, so I left it out of the findings. The same holds for the new `docs/contract.md` page and the `results.rs` prefix exclusion in `story:port-derivation`.
- Out of my lane (acceptance): the `story:contract-derivation` line that compares each port's protocol with `blocks/ports.json` is stated for `workstation.yaml` only. The epic's acceptance asks that every port of all 19 documents has a protocol.

```findings
[]
```
