//! The one table from the simulator's own event kinds to the events of the building blocks
//! (`blocks/`), which of them an `on_event` trigger can wait for, the names a scenario expects
//! them by, and the rules a scenario run refuses an input by.

/// A refused command: no block event, a refusal with a rule.
pub const REFUSAL: &str = "refusal";

/// One event kind of the simulator and the block events it stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kind {
    pub kind: &'static str,
    pub events: &'static [&'static str],
    /// Whether an `on_event` trigger can wait for it, by its one block event.
    pub trigger: bool,
}

const fn kind(kind: &'static str, events: &'static [&'static str]) -> Kind {
    Kind {
        kind,
        events,
        trigger: false,
    }
}

const fn trigger(kind: &'static str, event: &'static [&'static str; 1]) -> Kind {
    Kind {
        kind,
        events: event,
        trigger: true,
    }
}

/// Each event kind the simulator or the scenario runner records, and the block events it stands
/// for. An empty list: the kind is bookkeeping of the simulation (a fault window, a schedule's
/// own tick) or the blocks record nothing for it. A `trigger` row is an event the simulation
/// emits for an `on_event` trigger; the order of those rows is the order they are listed in.
pub const TABLE: &[Kind] = &[
    trigger("task_arrived", &["agentic.work.TaskCreated"]),
    kind("task_started", &["agentic.work.RunStarted"]),
    kind("task_reworked", &["agentic.work.RunStarted"]),
    kind("task_failed", &["agentic.work.RunEnded"]),
    kind("task_claimed", &["agentic.work.DeliverableSubmitted"]),
    kind("task_handed", &["agentic.work.AssignmentOffered"]),
    trigger("task_done", &["agentic.work.TaskCompleted"]),
    trigger("task_rejected", &["agentic.work.DeliverableRejected"]),
    kind("review_started", &["agentic.work.RunStarted"]),
    kind("duplicate_dropped", &["agentic.work.TaskCancelled"]),
    kind("rate_limited", &["agentic.gateway.CallDecided"]),
    kind("build_queued", &["agentic.resource.ClaimQueued"]),
    kind("build_started", &["agentic.resource.ClaimGranted"]),
    kind("build_done", &["agentic.resource.ClaimReleased"]),
    trigger("question_asked", &["agentic.question.QuestionAsked"]),
    kind(
        "question_forwarded",
        &[
            "agentic.question.QuestionAsked",
            "agentic.channel.MessageSent",
        ],
    ),
    trigger("question_answered", &["agentic.question.QuestionAnswered"]),
    trigger("question_defaulted", &["agentic.question.DefaultTaken"]),
    trigger("build_killed", &["agentic.resource.ClaimKilled"]),
    kind("posted", &["agentic.channel.Posted"]),
    kind("schedule_ticked", &["agentic.trigger.ScheduleAdvanced"]),
    kind("trigger_fired", &["agentic.trigger.TriggerFired"]),
    kind("firing_queued", &["agentic.trigger.FiringQueued"]),
    kind("firing_skipped", &["agentic.trigger.FiringSkipped"]),
    kind("firing_started", &["agentic.trigger.FiringStarted"]),
    trigger("firing_failed", &["agentic.trigger.FiringFailed"]),
    kind("claim_conflict", &[REFUSAL]),
    kind("merge_conflict", &[]),
    kind("capacity_dropped", &[]),
    kind("capacity_restored", &[]),
    kind("principal_away", &[]),
    kind("principal_back", &[]),
    kind("corpus_refused", &[REFUSAL]),
    kind("input_delivered", &["agentic.channel.MessageSent"]),
    kind("input_refused", &[REFUSAL]),
];

/// The names a scenario's `then` expects events by, each with the block event it means.
pub const EXPECTABLE: &[(&str, &str)] = &[
    ("task_created", "agentic.work.TaskCreated"),
    ("run_started", "agentic.work.RunStarted"),
    ("run_ended", "agentic.work.RunEnded"),
    ("assignment_offered", "agentic.work.AssignmentOffered"),
    ("deliverable_submitted", "agentic.work.DeliverableSubmitted"),
    ("deliverable_rejected", "agentic.work.DeliverableRejected"),
    ("task_completed", "agentic.work.TaskCompleted"),
    ("task_cancelled", "agentic.work.TaskCancelled"),
    ("proposal_opened", "agentic.workspace.ChangeProposed"),
    ("proposal_merged", "agentic.workspace.ProposalMerged"),
    ("question_asked", "agentic.question.QuestionAsked"),
    ("question_answered", "agentic.question.QuestionAnswered"),
    ("default_taken", "agentic.question.DefaultTaken"),
    ("post_sent", "agentic.channel.MessageSent"),
    ("board_posted", "agentic.channel.Posted"),
    ("approval_granted", "agentic.authority.ApprovalGranted"),
    ("approval_rejected", "agentic.authority.ApprovalRejected"),
    ("call_decided", "agentic.gateway.CallDecided"),
    ("claim_queued", "agentic.resource.ClaimQueued"),
    ("claim_granted", "agentic.resource.ClaimGranted"),
    ("claim_released", "agentic.resource.ClaimReleased"),
    ("claim_killed", "agentic.resource.ClaimKilled"),
    ("schedule_advanced", "agentic.trigger.ScheduleAdvanced"),
    ("trigger_fired", "agentic.trigger.TriggerFired"),
    ("firing_queued", "agentic.trigger.FiringQueued"),
    ("firing_skipped", "agentic.trigger.FiringSkipped"),
    ("firing_started", "agentic.trigger.FiringStarted"),
    ("firing_failed", "agentic.trigger.FiringFailed"),
    ("refusal", REFUSAL),
];

/// The rules a scenario run refuses an input by, with the block error that refuses the same
/// (none where no block declares one) and what each means.
pub const RULES: &[(&str, Option<&str>, &str)] = &[
    (
        "input_without_link",
        Some("agentic.channel.NotAParty"),
        "no link from the author to the part carries the input's kind",
    ),
    (
        "control_without_authority",
        Some("agentic.delegation.NotTheDelegator"),
        "a control input from an author that is not the principal the part acts for",
    ),
    (
        "control_without_proof",
        None,
        "a control input without proof that it comes from its author",
    ),
    (
        "claim_conflict",
        Some("agentic.channel.PostAlreadyTaken"),
        "a copy claims a board entry another copy claimed first",
    ),
    (
        "corpus_refused",
        None,
        "a corpus entry whose input was refused makes no task",
    ),
];

/// The block events of a simulator event kind; `None` for a kind the table does not know.
pub fn block_events(kind: &str) -> Option<&'static [&'static str]> {
    TABLE.iter().find(|k| k.kind == kind).map(|k| k.events)
}

/// The events an `on_event` trigger can wait for, `(block event, simulator kind)`, in table
/// order.
pub fn triggers() -> impl Iterator<Item = (&'static str, &'static str)> {
    TABLE
        .iter()
        .filter(|k| k.trigger)
        .map(|k| (k.events[0], k.kind))
}

/// The simulator kind an `on_event` trigger waiting for `block_event` fires on.
pub fn trigger_kind(block_event: &str) -> Option<&'static str> {
    triggers().find(|(e, _)| *e == block_event).map(|(_, k)| k)
}

/// The block event an expected name means.
pub fn expected_event(name: &str) -> Option<&'static str> {
    EXPECTABLE.iter().find(|(n, _)| *n == name).map(|(_, b)| *b)
}

/// The expected name of a block event.
pub fn name_of(block_event: &str) -> Option<&'static str> {
    EXPECTABLE
        .iter()
        .find(|(_, b)| *b == block_event)
        .map(|(n, _)| *n)
}

/// Whether the simulator records the block event an expected name means.
pub fn modeled(name: &str) -> bool {
    expected_event(name).is_some_and(|b| TABLE.iter().any(|k| k.events.contains(&b)))
}

/// The block error of a rule, if a block declares one.
pub fn rule_error(rule: &str) -> Option<&'static str> {
    RULES
        .iter()
        .find(|(r, _, _)| *r == rule)
        .and_then(|(_, e, _)| *e)
}
