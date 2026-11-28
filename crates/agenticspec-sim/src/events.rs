//! The one table from the simulator's own event kinds to the events of the building blocks
//! (`blocks/`), which of them an `on_event` trigger can wait for, the block commands a refusal
//! stands for, and the rules a scenario run refuses an input by.

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

/// The block command an input stands for: every input is a message to its border part
/// (`input_delivered` records `agentic.channel.MessageSent`), so a refused input is a refusal of
/// this command.
pub const INPUT_COMMAND: &str = "agentic.channel.SendMessage";

/// The rules a scenario run refuses by, with the block command each refuses, the block error that
/// refuses the same (none where no block declares one) and what each means.
pub const RULES: &[(&str, &str, Option<&str>, &str)] = &[
    (
        "input_without_link",
        INPUT_COMMAND,
        Some("agentic.channel.NotAParty"),
        "no link from the author to the part carries the input's kind",
    ),
    (
        "control_without_authority",
        INPUT_COMMAND,
        Some("agentic.delegation.NotTheDelegator"),
        "a control input from an author that is not the principal the part acts for",
    ),
    (
        "control_without_proof",
        INPUT_COMMAND,
        None,
        "a control input without proof that it comes from its author",
    ),
    (
        "claim_conflict",
        "agentic.channel.TakePost",
        Some("agentic.channel.PostAlreadyTaken"),
        "a copy claims a board entry another copy claimed first",
    ),
    (
        "corpus_refused",
        INPUT_COMMAND,
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

/// Every block event the simulator records, in table order, each once.
pub fn recorded() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for b in TABLE.iter().flat_map(|k| k.events.iter().copied()) {
        if b != REFUSAL && !out.contains(&b) {
            out.push(b);
        }
    }
    out
}

/// Whether the simulator records the block event `block_event`.
pub fn modeled(block_event: &str) -> bool {
    block_event != REFUSAL && TABLE.iter().any(|k| k.events.contains(&block_event))
}

/// Whether the simulator refuses the block command `command` by one of its rules.
pub fn refuses(command: &str) -> bool {
    RULES.iter().any(|(_, c, _, _)| *c == command)
}

/// The block command a rule refuses; an input's command for a rule of no row (an input refused
/// because a part is unavailable).
pub fn rule_command(rule: &str) -> &'static str {
    RULES
        .iter()
        .find(|(r, _, _, _)| *r == rule)
        .map_or(INPUT_COMMAND, |(_, c, _, _)| *c)
}

/// The block error of a rule, if a block declares one.
pub fn rule_error(rule: &str) -> Option<&'static str> {
    RULES
        .iter()
        .find(|(r, _, _, _)| *r == rule)
        .and_then(|(_, _, e, _)| *e)
}

/// Whether a refusal of `command` with `error` can occur: the command's outcomes name the error,
/// or a rule of the simulator refuses that command with it.
pub fn refusal_known(
    command: &str,
    error: &str,
    declared: &std::collections::BTreeSet<String>,
) -> bool {
    declared.contains(error)
        || RULES
            .iter()
            .any(|(_, c, e, _)| *c == command && *e == Some(error))
}
