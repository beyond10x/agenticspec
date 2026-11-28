//! What a scenario or a corpus decides in place of the profile's draws: tasks that arrive at given
//! times, the outcome of a run, the time a build takes, a principal's reply.

/// How the first run of a task ends, decided instead of drawn from the step chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    /// The run is correct and reports done.
    Correct,
    /// The run is not correct and says so: it ends failed and the task is worked on again.
    Wrong,
    /// The run is not correct and reports done anyway.
    FalseCompletion,
    /// The run asks one question, then is correct.
    Question,
}

/// What is decided about one task; a field left `None` is drawn from the profile.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskScript {
    pub outcome: Option<RunOutcome>,
    /// Model calls of the first run.
    pub turns: Option<usize>,
    /// Builds of the first run.
    pub builds: Option<usize>,
    /// Whether its question needs the principal behind the agent it asks.
    pub needs_owner: Option<bool>,
    /// The question comes before the first model call (an open question at the start).
    pub question_first: bool,
}

/// A task that arrives at `at` seconds at part `part`.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrival {
    pub at: f64,
    pub part: String,
    pub script: Option<TaskScript>,
}

/// Inputs and fixtures of one scripted run. With a script, the profile's load makes no tasks:
/// only `arrivals` do.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Script {
    pub arrivals: Vec<Arrival>,
    /// The outcomes of runs, taken in the order runs start; `Some(part)` only by that part.
    pub runs: Vec<(Option<String>, TaskScript)>,
    /// Seconds each build takes, taken in the order builds start; past the end, drawn.
    pub builds: Vec<f64>,
    /// `(principal part, seconds)`: each reply, taken in the order questions reach the principal.
    /// A principal with no reply left does not answer.
    pub replies: Vec<(String, f64)>,
}

/// What a scripted run reports about one task.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskRecord {
    /// The index into `Script::arrivals` it came from.
    pub arrival: Option<usize>,
    /// The index into `Script::runs` its first run took.
    pub fixture: Option<usize>,
    pub correct: bool,
}

/// One event of a run's timeline: its time, the part it is about, its kind, and the task.
#[derive(Debug, Clone, PartialEq)]
pub struct Stamp {
    pub t: f64,
    /// The part it happened at.
    pub part: String,
    /// The moment, a kind of the timeline (`task_arrived`, `build_queued`, ...).
    pub kind: &'static str,
    pub task: Option<usize>,
    /// The index into `Script::replies` of the reply an answered or defaulted question took.
    pub reply: Option<usize>,
    /// The number of the simulated entity it is about, for the kinds whose event is keyed by
    /// one: a task, a question, a build's claim, a firing, a post, a deliverable. Numbers count
    /// per entity from 0 within the run. Kept for the event log (`crate::event_log`).
    pub subject: Option<usize>,
    /// The copy of the part that made it, when a copy of a worker or reviewer did.
    pub copy: Option<usize>,
}
