# Consumer gaps: 15 concepts of a multi-session setup in the blocks (2026-11-27)

Source: a gap report from a consumer that tried to write its multi-session agent setup in
agenticspec 0.3.0 (`spec/domains/session.yaml`). The setup has one human, an operator session
acting for the human, coordinator sessions with sub-agents, a daemon running protocol runs on model
routes, and a web control page. The report lists 15 concepts the session IR could not hold (G1 to
G15). Story: `story:consumer-gaps`.

Each gap below names where the concept lives in `blocks/` now. Status:

- **covered**: the blocks already held it;
- **added**: this change added a field, a state or a command;
- **left out**: not added, with the reason.

The layer rule holds for every addition: a block names only its own layer or a lower one. A
concept that a lower block would have to name of a higher one is turned around. Every guess is an
`UNMAPPED:` comment in the block.

| gap | status | where |
|---|---|---|
| G1 delegation of authority | covered, added | `delegation` Delegation.excludes; `channel` Message and Post `delegation_id`, `proof` |
| G2 standing rules as policy | added | `authority` Grant effect, condition, excludes, supersedes; Decision.grant_id |
| G3 spend caps and model starts | added | `authority` Budget owner kinds Run and Route, window; `runtime` Workload.start_cost, Instance.budget_id |
| G4 a sub-agent's hand-back | added | `workspace` Proposal handback (an artifact id of `store`), head_revision, merged_under |
| G5 protocol runs | covered, added | `process`, `work` Task and Run; new entity `work` Step, Run.route_id |
| G6 model routes as records | covered, added | `model` Route; Endpoint.hosting, start_wait_seconds; `runtime` Workload.endpoint_id |
| G7 a decision with options | covered, added | `question` Question.topic, run_id; AnswerOption.binds |
| G8 a message between sessions | covered, added | `channel` Message.from; Message.sent_by_call |
| G9 relayed chat | added | `channel` Subscription.relays_for, last_beat_at; command Beat |
| G10 a work queue with cancel | covered, added | `resource` Claim; Claim.queued_at, state Withdrawn, command WithdrawClaim |
| G11 requested model of a sub-agent | covered, added | `agent` AgentRevision.models; `harness` HarnessSession.requested_route |
| G12 session name and role | covered, added | `harness` HarnessSession.name; `agent` AgentRevision.role; `workspace` Lease.holder |
| G13 audience of an outbound post | added | `channel` Channel.audience; Post.delegation_id; `question` AnswerOption.binds |
| G14 a human act outside a session | covered, added | `work`, `question`, `authority` commands of principals; ChannelKind Control; `proof` |
| G15 trigger rules | covered, added | `trigger` Trigger condition, process_id, intake, start_mode; Firing.subject_key |

## G1 Delegation of authority

The concept: an operator session acts for a human, with the human's permissions minus rules it
cannot lift. A human order is proven by marks of the channel it came through.

- Covered: `delegation` Delegation names the delegator, the delegate, the action and the root
  human of the chain. Acting for all of a human's decisions is one delegation per
  DelegatedAction (the block's existing rule).
- Covered: a decision taken for someone names its delegation: `authority` Decision.delegation,
  `question` Question.answered_via.
- Added: Delegation.excludes, the rules the delegate cannot lift. GrantDelegation takes them, and
  DelegateOnward copies them from the parent link.
- Added on `channel` Message and Post:
  - `delegation_id`: the message counts as the delegator's (the `on_behalf_of` of the report);
  - `proof`, type Proof: None, ClientRecord, ProcessIdentity or Passkey.
- UNMAPPED: no refusal of an act an excluded rule forbids; the evaluator checks it. The list of
  proof kinds comes from one consumer.

## G2 Standing rules as policy

The concept: written standing grants (commit, push, merge, release without asking), each with a
grantor, a date, conditions and exclusions. A guard refuses an action.

- Covered: `authority` Grant has granted_by, granted_at, action_class, scope and expires_at.
  DenyRequest records a refusal.
- Added on Grant:
  - `effect` (Verdict): Allow, ApprovalRequired, or Deny for a standing prohibition;
  - `condition`, `excludes`, `supersedes`;
  - a declared relation to the grantor.
- Added: DecideRequest refuses an Allow or ApprovalRequired under a prohibition (`grant-forbids`,
  error GrantForbids).
- Added: Decision.grant_id, the grant a decision rests on. DecideRequest sets it; DenyRequest
  takes it optionally for the prohibition a denial rests on.
- UNMAPPED: the grammar of `condition`, and who evaluates it.
- Left out: `expires_at` stays required. A grant without an end gets a far expiry.

## G3 Spend caps and model starts

The concept: caps per run, per call, per day over all runs, per route per day and per month. A
self-hosted model server costs money to start. A start is booked, joined to one already running,
or refused at a cap or a ceiling.

- Covered: `authority` Budget (limit, spent, Open to Exhausted). RecordSpend refuses a total
  over the limit (OverLimit).
- Added: Budget.owner_kind is BudgetOwnerKind: Principal, AgentRevision, Run, Route.
  Budget.window is BudgetWindow: Total, Call, Day, Month.
- Added in `runtime` (layer 8, which may name layers 6 and 2):
  - Workload.start_cost (Money);
  - Instance.budget_id, the budget the start was booked against; StartInstance takes it.
- The four answers:
  - booked: RecordSpend recorded, then StartInstance;
  - cap reached: OverLimit on the route's or run's budget;
  - ceiling refused: OverLimit on the Month budget;
  - joined: the caller finds an instance of the workload Pending or Running and starts none.
- UNMAPPED: a windowed budget is one budget per window, opened by the meter; nothing resets
  spent. The timezone of a day is not stated. A second start while one runs is not refused.

## G4 A sub-agent's hand-back

The concept: a code-changing sub-agent works in its own worktree on its own branch and hands back
when its merge request is open. The coordinator merges under a policy.

- Covered: `workspace` Workspace (kind Worktree), Lease (holder is the session as a party),
  Proposal (proposed_by, base_revision, change, Proposed to Merged).
- Added on Proposal:
  - `handback`, the artifact id (`store`) the change is handed back as, and `head_revision`, set
    by Propose; the branch and the merge or pull request are rows of a source-control pack;
  - `merged_under`, a GrantId with a relation to `authority` Grant, set by MergeProposal.
- `work` Deliverable (kind Proposal) stays the work-side record. It does not name the workspace
  proposal: `work` is layer 5, `workspace` layer 8.

## G5 Protocol runs

The concept: runs of versioned protocols, with an intake, steps (a program, an agent node or a
person action), an outcome and child runs.

- Covered:
  - `process` Process (name, version, source digest) is the protocol at a revision;
  - `work` Task is the case: goal as a digest (the intake), requested_by, process_id,
    parent_task_id (a child case);
  - `work` Run is one execution, with its outcome;
  - `harness` HarnessKind Driven is a loop driven from outside (the daemon).
- Added: entity `work` Step (StepId in `shared`): run_id, action, kind (StepKind: Program, Agent,
  Person, Child), harness_session_id, result (StepResult: Passed, Failed, Skipped).
- Added: command RecordStep, event StepRecorded, view Steps; Run owns its steps.
- Added: Run.route_id.
- A person step waits as a run suspended with AwaitingAnswer, on a question that names the run.
- A child step's task names the run's task as its parent.
- The run's budget is a Budget with owner_kind Run. A Run cannot name a budget: `authority` is
  layer 6.
- Scenario: synthesis reaches no RecordStep outcome (ESS-SYNTH-001: two rows under one ProcessId).
  `blocks/scenarios/work/run-records-steps.yaml` covers it: two steps recorded, a step after the
  end refused.
- UNMAPPED: whether a write step needs a person (the switch required, none or guarded) is a rule of
  the process, not a field.

## G6 Model routes as records

The concept: a route with a gateway, a model, a kind (hosted, managed, local), a phase, prices and
a start wait.

- Covered:
  - `model` Route (name, endpoint, model), Model.price (TokenPrice);
  - `gateway` Upstream joins a gateway to an endpoint, so a route's gateways are those with an
    upstream on its endpoint;
  - turns name their inference, and the inference names its route;
  - the route class a step picks is the agent revision's ModelUse.
- Added: Endpoint.hosting (Hosting: Vendor, Managed, Local) and Endpoint.start_wait_seconds.
- Added: `runtime` Workload.endpoint_id. The phase of a Managed or Local endpoint is the state of
  its workload's instance: Pending is starting, Running is ready, Exited is stopped.
- Left out: a stopping phase. The instance has no state between Running and Exited.

## G7 A decision with options

The concept: lettered options, a recommended option, a kind, a holder, a default on silence after
a deadline, who answered for whom. A pick can bind the digest of an exact text.

- Covered: `question` Question:
  - options with letters;
  - default_option, which is the recommended option;
  - deadline, and TakeDefault;
  - answered_by and answered_via;
  - QuestionKind Approval.
- Added:
  - Question.topic, what the decision is about;
  - Question.run_id, the run waiting on it;
  - AnswerOption.binds, the content digest the option approves.
- UNMAPPED: a holder that is not the addressee. A rule-held decision is an `authority` Decision
  with decider Rule. An advised decision (the asker decides) has no field.

## G8 A message between sessions

The concept: sessions message each other. The receiver sees a sender. The sending tool call and
the prompt it became cannot be joined.

- Covered: `channel` Message.from and Message.to are parties, and a party can be a harness session
  (PartyKind Session). Channel kind AgentMessage.
- Added: Message.sent_by_call, the sending tool call (`harness` ToolCall, a lower layer).
- Left out: the receiving turn. `harness` Turn holds no prompt or author. The join on the
  receiving side is the message's `to` and `delivered_at`.

## G9 Relayed chat

The concept: a human writes in a web chat. An agent watches it and gets each message as a harness
notification. A heartbeat marks the agent online.

- Covered: the chat is a shared channel. The human's post keeps the human as `from`. The agent
  holds a Subscription.
- Added:
  - Subscription.relays_for, the principal whose posts it carries into the session; Subscribe
    takes it;
  - Subscription.last_beat_at, and command Beat (event HeartbeatRecorded), refused for an ended
    subscription.
- Left out: the online window. Whether a party counts as online is a reader's threshold over
  last_beat_at.

## G10 A work queue with cancel

The concept: a click on a busy route is queued. A dispatcher submits it in click order when a slot
frees. The click can be cancelled.

- Covered: `resource` Pool and Claim. A claim waits Queued until the pool grants it.
- Added:
  - Claim.queued_at, published by the Queue view;
  - state Withdrawn, transition withdraw (Queued to Withdrawn);
  - command WithdrawClaim by the holder, event ClaimWithdrawn.
- This answers the block's open question whether a queued claim can be withdrawn.
- Scenario: synthesis reaches no held claim. `blocks/scenarios/resource/claim-withdrawn-from-queue.yaml`
  withdraws a queued claim and is refused on a held one.
- UNMAPPED: a grant out of queue order is not refused (a row-set guard).
- Left out: a link from a claim to the run it became. The claim's holder is the party.

## G11 Requested model of a sub-agent

The concept: a rule picks the sub-agent's model: a default tier, a higher tier for code, a cap.

- Covered: `agent` AgentRevision.models binds a model or route per use. BudgetLimits holds the
  caps. The revision is the rule.
- Added: HarnessSession.requested_route, the route the spawning session asked for. SpawnSubAgent
  takes it.
- Left out: a named model policy other than the revision. A grant cannot be named from `harness`:
  `authority` is layer 6.

## G12 Session name and role

The concept: sessions are addressed by name, and rules name roles. A session works in a directory.

- Added: HarnessSession.name. OpenSession and SpawnSubAgent take it.
- Covered:
  - the role is AgentRevision.role, which the session runs;
  - HarnessSession.role (Main, SubAgent) is its place in the tree;
  - the working directory is a `workspace` Lease whose holder is the session. Turned around:
    `harness` cannot name a workspace.

## G13 Audience of an outbound post

The concept: a post's audience (outside readers, internal, unknown), the identity it posts as (a
person or a bot), and the pick that allowed its exact text.

- Added: Channel.audience (Audience: Internal, External, Private, Unknown). OpenChannel and
  OpenSharedChannel take it.
- Posting as a person is a post under a delegation from that person (Post.delegation_id, from G1).
  Without one, it posts in its own name.
- The pick: the option's `binds` digest equals the post's content digest. Turned around: `channel`
  cannot name a question, because `question` depends on `channel`.

## G14 A human act outside a session

The concept: the human clicks Send, Pick, Run, Retry or Dispatch on a web page. Each click starts a
run or answers a decision.

- Covered: in the blocks a principal acts without a session:
  - `work` Requester creates tasks;
  - `question` Answerer answers (a party can be a principal);
  - `authority` Steward approves;
  - `resource` Claimant queues.
- Added: ChannelKind Control (a page where a human acts by a button, each act a Request message
  or post), and `proof` on messages and posts (G1).

## G15 Trigger rules

The concept: entities of one kind in one state with a condition start protocol P with intake
mapping M, offered or automatic, deduplicated per state entry.

- Covered by `trigger`:
  - TriggerKind OnEvent with event_name (a state entry is an event);
  - the agent revision it wakes;
  - ConcurrencyPolicy;
  - Firing records each firing and what became of it.
- Added on Trigger: `condition`, `process_id` (relation to `process` Process), `intake`, and
  `start_mode` (StartMode: Auto, Offer). Absent means Auto.
- Added: Firing.subject_key, which FireTrigger takes.
- UNMAPPED: the grammar of `condition` and `intake`. A duplicate firing for one subject key is
  not refused; that needs a row-set guard, whose cost RecordStart's note records.
