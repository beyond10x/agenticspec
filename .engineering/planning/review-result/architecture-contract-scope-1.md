---
format: aep.planning-md/3
id: review-result:architecture-contract-scope-1
kind: review-result
status: active
title: 'architecture-contract plan, scope critic, round 1: needs-revision'
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
needs-revision

```
epic:architecture-contract — "the settings that narrow each of them, the protocol of each port" is promised, and no item claims the protocol in the contract: story:contract-derivation's agenticspec-contract/1 holds `ports` as (port, part, settings, instances) with no protocol, and story:contract-docs' section lists "port, kind, block, part, narrowing settings, instances" with no protocol. story:contract-derivation would most naturally take it; either it writes each port's protocol into the contract and the docs, or the epic records that the protocol stays in blocks/ports.json only — .engineering/planning/epic/architecture-contract.md:16
```

**What you read:** 7 artifacts (the parent and 6 stories). I ran `aep plan artifact show epic:architecture-contract` and `aep plan artifact graph`. I read the six story files and the epic's UNMAPPED markers in `blocks/9-composition/architecture.yaml`. I read `design/architecture-runtime.md` § "The contract of one architecture".

**Promises extracted and traced:** I extracted 14 promises from the parent and traced 13 to a story. The untraced one is the protocol promise above. Two other things are named as not covered, so they are not gaps. Recording a contract through `RecordContract` is left out by story:contract-derivation. Contracts for the ladder page are excluded by the epic's own "Not in this epic".

**What I could not establish:**
- The unease about `scripts/check-layers.sh`: story:block-bindings adds a new layering rule for bindings that no sentence in the parent mentions. It looks like a guard the bindings need, so I did not make it a finding.
- Out of my lane, for the acceptance critic: story:contract-derivation's last acceptance bullet exempts ports "left to an ESS gap", and that exemption has no checkable form.

```findings
- file: .engineering/planning/epic/architecture-contract.md
  line: 16
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "\"the settings that narrow each of them, the protocol of each port\" is promised, and no item claims the protocol in the contract: story:contract-derivation's agenticspec-contract/1 holds `ports` as (port, part, settings, instances) with no protocol, and story:contract-docs' section lists \"port, kind, block, part, narrowing settings, instances\" with no protocol. story:contract-derivation would most naturally take it; either it writes each port's protocol into the contract and the docs, or the epic records that the protocol stays in blocks/ports.json only"
```
