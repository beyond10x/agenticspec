---
format: aep.planning-md/3
id: story:governance-types
kind: story
status: active
title: 'Governance types: approver identity, grant issuers, decisions bound to a subject digest'
summary: Seven governance gaps a consumer's adversarial reviews found; agenticspec holds the types and invariants, the decision stays a port
relations:
- decomposes: epic:building-blocks
scope:
- confidence: cited
  path: blocks/0-foundation/shared.yaml
- confidence: cited
  path: blocks/1-principals/delegation.yaml
- confidence: cited
  path: blocks/6-governance/authority.yaml
- confidence: cited
  path: blocks/7-collaboration/channel.yaml
- confidence: cited
  path: blocks/7-collaboration/question.yaml
- confidence: cited
  path: blocks/8-place/workspace.yaml
- confidence: cited
  path: kpis/catalogue.yaml
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-27T23:44:08Z", actor: "human:operator", revision: 4}
---
## Outcome

The governance blocks can state who may decide what, on which subject revision, and refuse a
trace that breaks it. A consumer's two read-only adversarial reviews of the blocks at `25fbfcb`
against a running setup rated governance the weakest area (24 parts: 1 covered, 12 partial, 11
missing). This story adds the types the findings show missing. The decision itself stays a port
(`principal.decide`, design/architecture-runtime.md § "The boundary: ports"): agenticspec holds
the types and the invariants, an evaluator such as Canon decides, a runtime such as Runbook runs
person actions and protocol cases. Not taken: importing Canon's `protocol/1` into the blocks.

## The gaps, each with the type that closes it

1. `approver` is a role (Human or Delegate), not a person (`blocks/6-governance/authority.yaml`
   approver field): an approver becomes a principal reference or a set, with exclusions (not the
   author, not the requesting agent, not the agent whose write is decided).
2. A grant can be recorded by anyone (`blocks/1-principals/delegation.yaml` GrantDelegation): a
   grant names its issuer, and recording it is refused unless the issuer holds the right to grant
   it (the delegation chain).
3. Nothing compares a request with its grant (`authority.yaml` DecideRequest and its views): a
   decision names the grant it relied on, and is refused when that grant does not cover the
   request.
4. A prohibition applies only when the caller names it (`authority.yaml` the prohibition input of
   a request): prohibitions attach to a scope and apply to every request in it.
5. Decisions do not bind to a subject revision: `MergeProposal` needs no approval and does not bind
   to the approved head (`blocks/8-place/workspace.yaml` MergeProposal); a post never checks the
   digest its pick bound (`blocks/7-collaboration/channel.yaml` PostMessage, TakePost). Both
   require a decision or pick of the same digest.
6. Every question requires a default and a deadline (`blocks/7-collaboration/question.yaml` Ask):
   a question kind that never defaults and blocks until answered.
7. A message's delegation and proof are fields the sender fills in (`channel.yaml` SendMessage):
   provenance becomes an attestation a port verifies, recorded with its verifier.

## Scope

- `blocks/1-principals/delegation.yaml`, `blocks/6-governance/authority.yaml`,
  `blocks/7-collaboration/channel.yaml`, `blocks/7-collaboration/question.yaml`,
  `blocks/8-place/workspace.yaml`, `blocks/0-foundation/shared.yaml`: the types and refusals above,
  with authored scenarios under `blocks/scenarios/` for every new refusal that synthesis cannot
  reach.
- `kpis/catalogue.yaml`: at least one KPI per new refusal kind where a block's KPI count requires
  it (`scripts/check-kpis.sh`), appended as text.
- Generated pages through `task docs:generate`.

## Shared surfaces

- `blocks/` files: `story:block-bindings` adds bindings across the same blocks and
  `story:guards-and-limits` adds a guard domain; whichever lands second rebases. Ordering edges to
  be added when this story leaves draft.

## Acceptance

- Given an approval rule "not the author", a decision by the author of the subject is refused with
  a named error, where before no rule could say it.
- Given a grant recorded by a principal that holds no right to grant it, recording is refused.
- Given a decision whose relied-on grant does not cover the request, the decision is refused.
- Given a prohibition on a scope, a request in that scope that does not name it is still refused.
- Given an approval of head digest D, `MergeProposal` of a proposal at head D' ≠ D is refused, and
  with no approval at all it is refused; a post whose pick bound digest D is refused for D'.
- Given a question of the never-default kind, it has no default and no deadline, `TakeDefault` is
  refused on it, and it stays open until `Answer` or `Withdraw`.
- Given a message whose provenance no verifier attested, a command that requires attested
  provenance refuses it.
- `task check:blocks` passes with every new refusal witnessed by synthesis or an authored scenario.

## Not in this story

The evaluator (Canon) and the runtime of person actions (Runbook); the `principal.decide` and
`process.run` ports themselves (`epic:architecture-contract`); importing `protocol/1`.
