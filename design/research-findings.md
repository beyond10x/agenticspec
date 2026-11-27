# What the blocks take from the research (2026-11-27)

Source: the paper distillations of the research repository (`literature/papers/<arxiv-id>.md`,
86 papers from 2609 to 2611 read), its `literature/DISTILLATION.md`, and its study of one owner's
Claude Code sessions (`out/REPORT.md`, a 2026-08 corpus with unvalidated labels). Each number
carries its paper id; `E<n>` grades in the distillations say how strong the evidence is, and many
E4 results come from authors who are not independent. A number here is a starting value for a
behaviour profile or a KPI, not a fact about this system.

## New blocks

Each fits a layer without an upward dependency.

- `verification` (layer 6, governance): a check that a deliverable is correct.
  - `Check`: expected, ran, result, evidence. A pass rate counts expected checks, so a check that
    did not run stays in the total (2610.08651). A check needs a negative case: 62.3% of accepted
    compliance checks pass on commits that touch nothing relevant (2611.13709).
  - `Verdict` of a critic, reviewer, oracle or auditor, typed by evidence (typed 0.533 vs untyped
    0.457 F1, 2608.18167). A critic has a false-accept and a false-reject rate: lenient reviewers
    accept 77-95% of wrong patches, strict ones reject 41-58% of correct ones (2611.02207).
  - `Receipt`: signed outside the sandbox, valid for a bounded time; forged, stale and rebound
    receipts are refused (2611.06946).
  - Execution beats judgment: module pass by execution tests 93.2 vs LLM-as-judge 73.6
    (2610.15708). An oracle is independent of the agent's own tests (2611.07166).
- `memory` (layer 3, capability, beside knowledge): entries an agent proposes and a principal
  approves before they are kept (2610.08651, 2611.05094: 3 of 8 proposed entries approved). Entries
  carry scope, supersedes and stale (task success 0.93 with both, 0.67 without supersession,
  2610.36188). Misapplied memory is a named failure: 115 misapplications, 81 by transfer
  (2610.27524).
- `plan` (layer 5, work): a plan exists before the first source change (PAUSE: 50.2% to 65.0%
  resolved, planner under 2% of executor tokens, 2610.29870; plan precedes every first change in
  23 of 23 runs, 2610.35992).
- `monitor` (layer 6): watches a run. It states what it observes (the outcome or a proxy), shows
  its last run, and has a bounded authority: an unbounded monitor repeatedly repaired legitimate
  work (2610.08651). A per-step router answers allow, redirect or pause (30% to 44% resolved for
  one model, beneficial in 109 of 242 interventions, cost +15% to +75%, 2609.39957).

## Changes to existing blocks

- `harness`: limits are enforced outside the prompt (67 iterations against a limit of 30 when the
  limit was only in the prompt, none over it when enforced, 2610.09995). A revision budget
  (B=0: 92.2, B=3: 97.0, 2610.07832). Approvals never live inside a compaction summary: a forged
  approval in a summary executed in 5 of 5 runs (2610.29869). The harness is an experiment factor
  next to the model: a harness change moved one model's pass rate by 14.06 points (2610.35971) and
  reversed model rankings in 6 of 9 comparisons (2610.35348).
- `channel`: a message is a request or an inform; 12-16% of informs got no action within 4 h,
  at most 3% of requests (2611.00689). Wake rules on intake cut total coordination failure from
  21 of 50 to 4 of 32 (2611.00689). A change notice names its scope; a vague warning equals no
  coordination (2610.07261). A typed board (observed, fact, fail, claim, patch summary) with
  length limits serves teams without an orchestrator (2609.26781).
- `question`: ask or assume before acting on an ambiguous task (23 to 33 resolved for one model,
  2609.39957); elicitation before implementation +4 to +6 pass@1 at about 20% more steps
  (2611.16289).
- `authority`: the submitter and the decider of an approval are recorded apart: 71 of 263 merges
  rested on approvals that a session submitted under a person's account (2610.28683). Work with
  no named human principal cannot be submitted (2609.07919). Human approval drifts upward over a
  review sequence, 30.5% to 36.6% (2609.06213), and 64% of people approved actions without close
  review (2611.10058).
- `work`: a delay state for a task whose files overlap a running task (91.7% fewer overlaps at
  65% makespan increase, against 167% for first-in-first-out, 2610.04779). A forbidden-file list in
  the assignment (scope leakage 0.00, 2610.07261). Interleaving tasks costs 11.5 points
  (2611.02017).
- `workspace`: a worktree isolates code only; ports and databases are shared state that needs a
  lease (2609.00252). A wrong workspace description cost 242 messages and $34.21, a correct one
  28-51 messages and $1.23-3.72 (2611.00689).
- `sandbox`: an exec scope checked before each command let 3 of 75 attacks through, against 47
  of 75 without it, at 68 against 72 of 75 tasks done (2609.08371).
- `tool`: an error states the next step: 45% recovered with the command alone, 82% with the cause
  (2609.35381).
- `knowledge`: a written precedence rule beats conflicting code, follow-the-code 4.2% to 91.4%
  (2610.30571); a stale line is corrected, not deleted (2611.09802); wording is tested per model
  (2611.04367).

## KPIs: correct, complete, quality

- correct: false completion claims (failed runs reported as done: 665 of 949, 70.1%,
  2610.04375); completion reports that name no failing check (52%, 2611.07166); functional passes
  that fail a constraint test (221 of 644, 34.3%, 2609.04167); revert rate (6.1% to 14.5% per
  agent, 11.5% human, 2609.17598); verified fix within 30 days of a merge (3.68% agent, 2.34%
  human, 2609.26847); regressions of the agent's own earlier work (27.8% of failures, 2611.02207);
  escape rate of fixed issues (5.6%, 2610.36254).
- complete: coverage against claim (32.1% of runs touched every file; runs that over-claim miss
  58.2%, 2609.20812); expected checks that ran (2610.08651); documentation changed with code
  (63.2% of agent pull requests change code and no documentation, 2610.27381).
- quality: owner wait share; merges per week and open-to-merge time (45.4 to 128.8 merges per
  week, 1.0 h to 21.7 h median, 2610.28683); cost per task; selection gap (oracle minus
  self-select, 2609.40303).

## Behaviour profile: starting values

- From the owner's sessions (`out/REPORT.md`): median agent idle before the human replies 122 s;
  34% of wall time idle on the human (229 h); 669 of 1,567 human turns (43%) removable; 48% of
  labelled human turns carry no information (approvals); counts of human turns: start a task 324,
  approve 190, ask 130, correct 123.
- Step pass after a pass 70.2%, after a fail 14.1% (2611.07166).
- Turns per task: half of runs end by turn 17 for one model, by turn 35 for another (2610.13668).
- Compactions 9.5 per run, each discarding about 76% of context (2609.40303).
- First human review of an agent's pull request: median 12.6 h for one agent, 1-4 h for others
  (2609.17598). Human reply p90 0.5-4.2 h; a resident operator's median 1 min (2611.00689).
- Interventions: 65% because the task was misunderstood (2611.10058).
- Cost: a cheaper model reached 78% of a stronger one's recall at 21% of the cost (2610.13668);
  reasoning effort raised detection from 36% to 57% at 23.22 times the cost (2610.13668).

## Topologies (architecture templates)

- One agent with both tasks: 62%; two parallel agents that do not see each other: 16%; a
  sequential hand-off: 54% at the same cost; more alternating turns: no gain at 1.59 times the
  cost (2611.00314).
- An upfront scope partition with a merge order: clean integration 9 of 9, against 4 of 9 when
  reacting and 1 of 9 without coordination (2610.07261).
- A broadcast channel between parallel agents lowered results (58.91 against 66.51 percentile,
  2609.40303).
- Author-reviewer loop: $0.95-1.07 per file, against $1.98-2.91 for a multi-agent workflow
  (2610.09995). Required review sub-agents: files touched 86.9% to 97.3%, defects reported 49.9%
  to 69.6% (2609.20812). Reviewer seats did not stop a planted backdoor (2611.00689).
- Task-specific sub-agents helped (+5.91, +4.46); general stage sub-agents hurt (-0.70, -2.31)
  (2609.32459). Sub-agent concurrency helped only on the largest tasks (2610.10263).
- A team without an orchestrator on a shared repository: 19.31% at 1 agent, 28.78% at 128
  (2609.26781). Scheduled agent loops: 2.2 to about 35 merged pull requests per day (2610.36254).

These become architecture documents under `architectures/` and mutations of experiments: the
simulator compares them on the same profile.
