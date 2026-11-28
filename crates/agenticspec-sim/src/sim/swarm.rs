//! What wakes an agent and how the copies of an agent part coordinate: schedules, triggers and
//! their concurrency policies, shared channels (topic, queue, board), an orchestrator, mesh
//! messages, a scope partition with its merges, and a start stagger.

use super::{Sim, Task, TaskScript, Who, WorkerKind, position, span};
use crate::check;
use crate::corpus::Class;
use crate::doc::num;
use crate::events;
use crate::rng::Rng;
use serde_json::Value;
use std::collections::VecDeque;

/// How a shared channel hands a post to its subscribers (agentic.channel.Distribution).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Distribution {
    Direct,
    /// Every subscribed trigger fires for each post; each run works its own copy.
    Topic,
    /// One subscribed trigger fires; the run takes the oldest untaken post.
    Queue,
    /// Every subscribed trigger fires; every idle copy reads the post and races for the claim.
    Board,
}

/// How the copies of an agent part coordinate (agentic.architecture.Coordination).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Coordination {
    None,
    Orchestrator,
    Board,
    Mesh,
}

/// Where the copies of a worker kind take their work from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Input {
    /// The backlog of arrivals, or under a scope partition the copy's own scope.
    Backlog,
    /// Only the runs its triggers start.
    Woken,
    /// The tasks another kind hands it over an `order` link.
    Handed,
}

/// Where a task goes when a worker kind is done with it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Output {
    Ship,
    Hand(usize),
    Post(usize),
}

/// What a trigger does when it fires at its limit (agentic.trigger.ConcurrencyPolicy).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Policy {
    Skip,
    Queue,
    Parallel,
}

/// What a trigger fires on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Source {
    Schedule(usize),
    Channel(usize),
    /// An event of the simulation, by its simulator kind (a `trigger` row of `events::TABLE`).
    Event(&'static str),
    /// A source the check refuses: it never fires.
    Never,
}

#[derive(Debug, Clone)]
pub(super) struct Schedule {
    pub(super) part: String,
    pub(super) every: f64,
    pub(super) first: f64,
}

#[derive(Debug, Clone)]
pub(super) struct Trigger {
    pub(super) part: String,
    pub(super) source: Source,
    pub(super) policy: Policy,
    pub(super) max_parallel: usize,
    /// A scheduled firing makes a new task from its template instead of taking one.
    pub(super) template: bool,
    /// A run goes on with the next item of its source until the source is empty.
    pub(super) takes_all: bool,
    /// The worker kinds it wakes.
    pub(super) wakes: Vec<usize>,
    /// For an `on_event` trigger, the part whose events alone fire it (`source`).
    pub(super) on: Option<String>,
}

impl Trigger {
    /// Whether an event of timeline kind `kind` at part `part` fires this trigger.
    pub(super) fn fires_on(&self, kind: &str, part: &str) -> bool {
        match self.source {
            Source::Event(e) => e == kind && self.on.as_deref().is_none_or(|on| on == part),
            _ => false,
        }
    }
}

/// A firing waiting for a free copy or for a run to end: when it fired, and the topic copy it
/// carries.
type Waiting = (f64, Option<usize>);

#[derive(Debug, Default, Clone)]
pub(super) struct TrigAcc {
    pub(super) running: usize,
    /// Firings within the limit, waiting for a free copy.
    pub(super) waiting: VecDeque<Waiting>,
    /// Firings at the limit under Queue, waiting for a run to end.
    pub(super) queued: VecDeque<Waiting>,
    pub(super) waits: Vec<f64>,
    pub(super) fired: f64,
    pub(super) started: f64,
    pub(super) skipped: f64,
    pub(super) queued_n: f64,
    pub(super) empty: f64,
    pub(super) failed: f64,
}

pub(super) fn stagger_of(p: &Value) -> f64 {
    num(&p["start_stagger"]).unwrap_or(0.0).max(0.0)
}

pub(super) fn coordination_of(name: &str, p: &Value) -> Result<Coordination, String> {
    match &p["coordination"] {
        Value::Null => Ok(Coordination::None),
        v => match v.as_str() {
            Some("none") => Ok(Coordination::None),
            Some("orchestrator") => Ok(Coordination::Orchestrator),
            Some("board") => Ok(Coordination::Board),
            Some("mesh") => Ok(Coordination::Mesh),
            _ => Err(format!(
                "part `{name}`: coordination {v}, expected {}",
                check::COORDINATIONS.join(", ")
            )),
        },
    }
}

pub(super) fn partition_of(name: &str, p: &Value) -> Result<bool, String> {
    match &p["partition"] {
        Value::Null => Ok(false),
        v if v == "scope" => Ok(true),
        v => Err(format!("part `{name}`: partition {v}, expected scope")),
    }
}

pub(super) fn distribution_of(arch: &Value, name: &str) -> Result<Distribution, String> {
    match check::distribution(arch, name) {
        None | Some("direct") => Ok(Distribution::Direct),
        Some("topic") => Ok(Distribution::Topic),
        Some("queue") => Ok(Distribution::Queue),
        Some("board") => Ok(Distribution::Board),
        Some(other) => Err(format!(
            "part `{name}`: distribution `{other}`, expected {}",
            check::DISTRIBUTIONS.join(", ")
        )),
    }
}

fn parts(arch: &Value) -> impl Iterator<Item = (&String, &Value)> {
    arch["parts"].as_object().into_iter().flatten()
}

fn links(arch: &Value) -> impl Iterator<Item = &Value> {
    arch["links"].as_array().into_iter().flatten()
}

pub(super) fn schedules(arch: &Value) -> Vec<Schedule> {
    parts(arch)
        .filter(|(_, p)| p["block"] == "schedule")
        .map(|(n, p)| Schedule {
            part: n.clone(),
            every: num(&p["every"])
                .filter(|e| *e > 0.0)
                .unwrap_or(f64::INFINITY),
            first: num(&p["next_fire_at"]).unwrap_or(0.0).max(0.0),
        })
        .collect()
}

/// The shared channel a principal posts to: arrivals become posts there.
pub(super) fn intake(arch: &Value, channels: &[String], dist: &[Distribution]) -> Option<usize> {
    links(arch).find_map(|l| {
        let from = l["from"].as_str()?;
        if check::block(arch, from) != Some("principal") || l["carries"] != "message" {
            return None;
        }
        position(channels, l["to"].as_str()?).filter(|c| dist[*c] != Distribution::Direct)
    })
}

/// Builds the triggers and sets each worker kind's input and output from the links.
pub(super) fn wire(
    arch: &Value,
    workers: &mut [WorkerKind],
    channels: &[String],
    dist: &[Distribution],
    schedules: &[Schedule],
) -> Result<Vec<Trigger>, String> {
    let names: Vec<String> = workers.iter().map(|w| w.exec.part.clone()).collect();
    let shared = |c: &str| position(channels, c).filter(|c| dist[*c] != Distribution::Direct);
    let mut triggers = Vec::new();
    for (name, p) in parts(arch).filter(|(_, p)| p["block"] == "trigger") {
        let source = match p["kind"].as_str() {
            Some("on_schedule") => p["schedule"]
                .as_str()
                .and_then(|s| schedules.iter().position(|x| x.part == s))
                .map_or(Source::Never, Source::Schedule),
            Some("on_message") => p["channel"]
                .as_str()
                .and_then(shared)
                .map_or(Source::Never, Source::Channel),
            Some("on_event") => match p["event_name"].as_str().filter(|e| !e.is_empty()) {
                None => Source::Never,
                Some(e) => match events::trigger_kind(e) {
                    Some(k) => Source::Event(k),
                    None => {
                        let known: Vec<&str> = events::triggers().map(|(n, _)| n).collect();
                        return Err(format!(
                            "part `{name}`: event_name `{e}` is not an event the simulation emits ({})",
                            known.join(", ")
                        ));
                    }
                },
            },
            _ => Source::Never,
        };
        let policy = match p["policy"].as_str() {
            Some("queue") => Policy::Queue,
            Some("parallel") => Policy::Parallel,
            _ => Policy::Skip,
        };
        let max_parallel = match policy {
            Policy::Parallel => num(&p["max_parallel"]).unwrap_or(1.0).round().max(1.0) as usize,
            _ => 1,
        };
        triggers.push(Trigger {
            part: name.clone(),
            source,
            policy,
            max_parallel,
            template: !p["task_template"].is_null(),
            takes_all: p["takes"] == "all",
            wakes: Vec::new(),
            on: p["source"].as_str().map(str::to_string),
        });
    }
    for l in links(arch).filter(|l| l["carries"] == "trigger") {
        let (Some(from), Some(to)) = (l["from"].as_str(), l["to"].as_str()) else {
            continue;
        };
        let Some(k) = position(&names, to) else {
            continue;
        };
        let tr = match triggers.iter().position(|t| t.part == from) {
            Some(tr) => tr,
            None => match schedules.iter().position(|s| s.part == from) {
                // A schedule linked straight to an agent is a trigger with the defaults.
                Some(sc) => {
                    triggers.push(Trigger {
                        part: from.to_string(),
                        source: Source::Schedule(sc),
                        policy: Policy::Skip,
                        max_parallel: 1,
                        template: false,
                        takes_all: false,
                        wakes: Vec::new(),
                        on: None,
                    });
                    triggers.len() - 1
                }
                None => continue,
            },
        };
        if !triggers[tr].wakes.contains(&k) {
            triggers[tr].wakes.push(k);
        }
    }
    for (tr, t) in triggers.iter().enumerate() {
        for &k in &t.wakes {
            workers[k].triggers.push(tr);
            workers[k].input = Input::Woken;
        }
    }
    for (k, name) in names.iter().enumerate() {
        let from_here = |carries: &'static str| {
            links(arch).filter(move |l| l["from"] == name.as_str() && l["carries"] == carries)
        };
        let post = from_here("message").find_map(|l| l["to"].as_str().and_then(shared));
        let hand = from_here("order")
            .chain(from_here("run"))
            .find_map(|l| l["to"].as_str().and_then(|to| position(&names, to)))
            .filter(|k2| *k2 != k);
        workers[k].output = match (post, hand) {
            (Some(c), _) => Output::Post(c),
            (None, Some(k2)) => Output::Hand(k2),
            _ => Output::Ship,
        };
    }
    for k in 0..workers.len() {
        if let Output::Hand(k2) = workers[k].output {
            if workers[k2].input == Input::Woken {
                return Err(format!(
                    "part `{}` is woken by a trigger, so the order or run link from `{}` cannot hand it tasks",
                    names[k2], names[k]
                ));
            }
            workers[k2].input = Input::Handed;
            workers[k].orchestrates = workers[k2].coordination == Coordination::Orchestrator;
        }
    }
    Ok(triggers)
}

impl Sim<'_> {
    fn new_task(&mut self, now: f64, origin: Option<usize>) -> usize {
        let id = self.tasks.len();
        let arrived = origin.map_or(now, |o| self.tasks[o].arrived);
        self.tasks.push(Task {
            arrived,
            rng: Rng::stream(self.seed, id as u64),
            base_turns: 0,
            correct: true,
            kind: 0,
            origin: origin.map_or(id, |o| self.tasks[o].origin),
            scope: None,
            work_started: now,
            waited: false,
            script: None,
            arrival: None,
            fixture: None,
        });
        self.shipped.push(false);
        if origin.is_none() {
            self.arrived += 1.0;
        }
        id
    }

    /// A task arrives: posted to the intake channel, routed to its scope's owner, or put in the
    /// backlog.
    /// With a corpus in the profile, each arrival draws an entry by weight: its class decides how
    /// the task's first run ends, and an entry that was refused makes no task.
    pub(super) fn arrive(&mut self, t: f64) {
        let m = self.m;
        let part = m.workers.first().map(|w| w.exec.part.as_str());
        let script = match self.draw_class() {
            None | Some(Class::Open) => None,
            Some(Class::Refused) => {
                if let Some(p) = part {
                    self.rec(t, p, "corpus_refused");
                }
                return;
            }
            Some(c) => Some(TaskScript {
                outcome: c.outcome(),
                ..TaskScript::default()
            }),
        };
        self.arrive_as(t, part, script, None);
    }

    /// Arrival `a` of the script arrives at its part.
    pub(super) fn arrive_scripted(&mut self, a: usize, t: f64) {
        let m = self.m;
        let Some(arrival) = m.script.as_ref().map(|s| &s.arrivals[a]) else {
            return;
        };
        self.arrive_as(
            t,
            Some(arrival.part.as_str()),
            arrival.script.clone(),
            Some(a),
        );
    }

    /// The class of the next corpus entry, drawn by weight; `None` without a corpus.
    fn draw_class(&mut self) -> Option<Class> {
        let corpus = &self.m.corpus;
        let total: f64 = corpus.iter().map(|(w, _)| w).sum();
        if corpus.is_empty() || total <= 0.0 {
            return None;
        }
        let mut x = self.corpus.unit() * total;
        for (w, c) in corpus {
            if x < *w {
                return Some(*c);
            }
            x -= w;
        }
        corpus.last().map(|(_, c)| *c)
    }

    fn arrive_as(
        &mut self,
        t: f64,
        part: Option<&str>,
        script: Option<TaskScript>,
        arrival: Option<usize>,
    ) {
        let m = self.m;
        let id = self.new_task(t, None);
        self.tasks[id].script = script;
        self.tasks[id].arrival = arrival;
        if let Some(p) = part {
            self.rec_task(t, p, "task_arrived", Some(id));
        }
        if let Some(c) = m.intake {
            return self.post(c, id, t);
        }
        if let Some(k) = m.partition {
            let n = m.workers[k].exec.count;
            if n > 0 {
                let scope = self.draw_scopes(id, n, None);
                self.tasks[id].scope = Some(scope);
                let i = self.copies[k][scope.0];
                self.insts[i].own.push_back(id);
                return;
            }
        }
        self.backlog.push_back(id);
    }

    /// The scope of task `id` among `n` (drawn unless given) and the other scope it touches: each
    /// other scope with `scope_overlap`, at most one kept. From the task's own scope stream, so a
    /// task's scopes do not depend on the order of events.
    fn draw_scopes(&self, id: usize, n: usize, primary: Option<usize>) -> (usize, Option<usize>) {
        let mut r = Rng::stream(self.seed ^ super::SCOPES, id as u64);
        let s = primary.unwrap_or_else(|| r.below(n));
        let overlap = self.m.coord.scope_overlap;
        let p = if n > 1 {
            1.0 - (1.0 - overlap).powi(n as i32 - 1)
        } else {
            0.0
        };
        let other = r.chance(p).then(|| {
            let o = r.below(n - 1);
            if o >= s { o + 1 } else { o }
        });
        (s, other)
    }

    pub(super) fn post(&mut self, c: usize, id: usize, now: f64) {
        let m = self.m;
        self.posted[c] += 1.0;
        self.rec_task(now, &m.channels[c], "posted", Some(id));
        let subs: Vec<usize> = (0..m.triggers.len())
            .filter(|tr| m.triggers[*tr].source == Source::Channel(c))
            .collect();
        match m.distribution[c] {
            Distribution::Topic => {
                for (n, &tr) in subs.iter().enumerate() {
                    let copy = if n == 0 {
                        id
                    } else {
                        self.new_task(now, Some(id))
                    };
                    self.fire(tr, now, Some(copy));
                }
            }
            Distribution::Queue => {
                self.posts[c].push_back(id);
                let open = subs.iter().copied().find(|tr| !self.at_limit(*tr));
                if let Some(tr) = open.or(subs.first().copied()) {
                    self.fire(tr, now, None);
                }
            }
            Distribution::Board => {
                self.posts[c].push_back(id);
                for tr in subs {
                    self.fire(tr, now, None);
                }
            }
            Distribution::Direct => self.backlog.push_back(id),
        }
    }

    pub(super) fn tick(&mut self, sc: usize, now: f64) {
        let m = self.m;
        self.ticks[sc] += 1.0;
        self.rec(now, &m.schedules[sc].part, "schedule_ticked");
        for tr in 0..m.triggers.len() {
            if m.triggers[tr].source == Source::Schedule(sc) {
                self.fire(tr, now, None);
            }
        }
        self.at(now + m.schedules[sc].every, super::Ev::Tick(sc));
    }

    /// Started runs and firings waiting for a copy fill the trigger's limit.
    fn at_limit(&self, tr: usize) -> bool {
        let a = &self.trig[tr];
        a.running + a.waiting.len() >= self.m.triggers[tr].max_parallel
    }

    fn fire(&mut self, tr: usize, now: f64, item: Option<usize>) {
        let t = &self.m.triggers[tr];
        self.trig[tr].fired += 1.0;
        self.rec(now, &t.part, "trigger_fired");
        if self.at_limit(tr) {
            let a = &mut self.trig[tr];
            if t.policy == Policy::Queue {
                a.queued_n += 1.0;
                a.queued.push_back((now, item));
                self.rec(now, &t.part, "firing_queued");
            } else {
                a.skipped += 1.0;
                self.rec(now, &t.part, "firing_skipped");
            }
            return;
        }
        self.trig[tr].waiting.push_back((now, item));
        self.dispatch_firings(tr, now);
    }

    /// Starts the trigger's waiting firings on idle copies of the kinds it wakes.
    fn dispatch_firings(&mut self, tr: usize, now: f64) {
        let m = self.m;
        while !self.trig[tr].waiting.is_empty() {
            let free = m.triggers[tr]
                .wakes
                .iter()
                .flat_map(|k| self.copies[*k].iter().copied())
                .find(|i| self.idle(*i, now));
            let Some(i) = free else { break };
            let (fired, item) = self.trig[tr].waiting.pop_front().expect("not empty");
            self.start_run(i, tr, fired, item, now);
        }
    }

    /// An idle copy of a woken kind starts the firing that has waited longest.
    pub(super) fn take_firing(&mut self, i: usize, k: usize, now: f64) {
        let mut best: Option<(f64, usize)> = None;
        for &tr in &self.m.workers[k].triggers {
            if let Some(&(fired, _)) = self.trig[tr].waiting.front()
                && best.is_none_or(|(b, _)| fired < b)
            {
                best = Some((fired, tr));
            }
        }
        if let Some((_, tr)) = best {
            let (fired, item) = self.trig[tr].waiting.pop_front().expect("not empty");
            self.start_run(i, tr, fired, item, now);
        }
    }

    fn start_run(&mut self, i: usize, tr: usize, fired: f64, item: Option<usize>, now: f64) {
        let m = self.m;
        let t = &m.triggers[tr];
        let Who::Worker(k) = self.insts[i].who else {
            unreachable!("a trigger wakes workers")
        };
        let a = &mut self.trig[tr];
        a.running += 1;
        a.started += 1.0;
        self.insts[i].run = Some(tr);
        self.rec(now, &t.part, "firing_started");
        if item.is_none()
            && let Source::Channel(c) = t.source
            && m.distribution[c] == Distribution::Board
            && let Some(id) = self.posts[c].pop_front()
        {
            let delay = self.read_board(i, c, now);
            self.trig[tr].waits.push(now + delay - fired);
            if delay > 0.0 {
                self.insts[i].hold = true;
                self.insts[i].begin = Some(id);
                self.at(now + delay, super::Ev::Begin(i));
            } else {
                self.start_task(i, id, now);
            }
            return;
        }
        let item = item.or_else(|| self.take_item(tr, now));
        self.trig[tr].waits.push(now - fired);
        match item {
            Some(id) => self.start_task(i, id, now),
            None => {
                // Nothing to take: the run looks once and ends.
                self.trig[tr].empty += 1.0;
                let dur = self.side_call(k);
                self.kinds[k].busy += span(now, now + dur, m.horizon);
                self.insts[i].hold = true;
                self.at(now + dur, super::Ev::EmptyDone(i));
            }
        }
    }

    /// An event of the simulation fired trigger `tr`.
    pub(super) fn fire_on_event(&mut self, tr: usize, now: f64) {
        self.fire(tr, now, None);
    }

    /// What a run takes: a schedule's or an event's firing makes a task from its template or
    /// takes the oldest of the backlog; a channel's takes the oldest untaken post.
    fn take_item(&mut self, tr: usize, now: f64) -> Option<usize> {
        let t = &self.m.triggers[tr];
        match t.source {
            Source::Schedule(_) | Source::Event(_) if t.template => {
                let id = self.new_task(now, None);
                self.rec_task(now, &t.part, "task_arrived", Some(id));
                Some(id)
            }
            Source::Schedule(_) | Source::Event(_) => self.backlog.pop_front(),
            Source::Channel(c) => self.posts[c].pop_front(),
            Source::Never => None,
        }
    }

    /// Every other idle copy subscribed to board `c` reads the new post with one model call.
    /// The run's copy reads first; a reader within the claim window of it tries the same claim
    /// and conflicts. Returns the delay of the run's own read.
    fn read_board(&mut self, i: usize, c: usize, now: f64) -> f64 {
        let m = self.m;
        let mut seen = vec![false; self.insts.len()];
        seen[i] = true;
        let mut readers = Vec::new();
        for t in m.triggers.iter().filter(|t| t.source == Source::Channel(c)) {
            for &k in &t.wakes {
                for &j in &self.copies[k] {
                    if !seen[j] && self.idle(j, now) {
                        seen[j] = true;
                        readers.push((j, k));
                    }
                }
            }
        }
        let mut delays: Vec<f64> = (0..=readers.len())
            .map(|_| m.coord.board_read_seconds.sample(&mut self.coord).max(0.0))
            .collect();
        let first = delays.iter().copied().fold(f64::INFINITY, f64::min);
        let at = delays.iter().position(|d| *d == first).unwrap_or(0);
        delays.swap(0, at);
        for (n, &(_, k)) in readers.iter().enumerate() {
            self.side_call(k);
            self.kinds[k].coordination_calls += 1.0;
            if delays[n + 1] - first < m.coord.claim_window_seconds {
                self.kinds[k].claim_conflicts += 1.0;
                self.rec(now, &m.workers[k].exec.part, "claim_conflict");
            }
        }
        first
    }

    /// A run released its task. A run that takes all goes on with the next item of its source;
    /// otherwise it ends, a queued firing moves up, and a queue or board offers its oldest
    /// untaken post again.
    pub(super) fn end_run(&mut self, i: usize, id: Option<usize>, now: f64) {
        let m = self.m;
        let Some(tr) = self.insts[i].run.take() else {
            return;
        };
        let t = &m.triggers[tr];
        // A run fails at the first task it releases that is not correct, so a long run that
        // takes all counts its failure before it ends.
        if id.is_some_and(|id| !self.tasks[id].correct) && !self.insts[i].run_failed {
            self.insts[i].run_failed = true;
            self.trig[tr].failed += 1.0;
            self.rec(now, &t.part, "firing_failed");
        }
        if t.takes_all && id.is_some() {
            let next = match t.source {
                Source::Channel(c) => self.posts[c].pop_front(),
                Source::Schedule(_) | Source::Event(_) if !t.template => self.backlog.pop_front(),
                _ => None,
            };
            if let Some(next) = next {
                self.insts[i].run = Some(tr);
                return self.start_task(i, next, now);
            }
        }
        self.insts[i].run_failed = false;
        let a = &mut self.trig[tr];
        a.running -= 1;
        if let Some(q) = a.queued.pop_front() {
            a.waiting.push_back(q);
        } else if let Source::Channel(c) = t.source
            && matches!(m.distribution[c], Distribution::Queue | Distribution::Board)
            && self.posts[c].len() > a.waiting.len()
        {
            self.fire(tr, now, None);
        }
        self.dispatch_firings(tr, now);
    }

    /// What starting a task costs in coordination: a scope for a partition, and a message to
    /// every other started copy of a mesh, each read with `mesh_read_calls` model calls.
    pub(super) fn coordinate_start(&mut self, i: usize, k: usize, id: usize, now: f64) {
        let m = self.m;
        let w = &m.workers[k];
        if w.partition && self.tasks[id].scope.is_none() {
            let scope = self.draw_scopes(id, w.exec.count, Some(self.insts[i].copy));
            self.tasks[id].scope = Some(scope);
        }
        if w.coordination == Coordination::Mesh {
            let others = self.copies[k]
                .iter()
                .filter(|j| **j != i && self.insts[**j].start_at <= now)
                .count();
            if others > 0 {
                self.kinds[k].coordination_messages += others as f64;
                let calls = (others as f64 * m.coord.mesh_read_calls).round() as usize;
                for _ in 0..calls {
                    self.side_call(k);
                }
                self.kinds[k].coordination_calls += calls as f64;
            }
        }
    }

    /// Merges task `id` of partition kind `k`. A merge conflicts when another merge touched one
    /// of its scopes since its work started. Under a mesh the notice turns the conflict into a
    /// round of rework (returns false); without one the merge lands broken.
    pub(super) fn merge(&mut self, k: usize, id: usize, now: f64) -> bool {
        let Some((s, other)) = self.tasks[id].scope else {
            return true;
        };
        let started = self.tasks[id].work_started;
        let touched = [Some(s), other];
        let conflict = touched
            .iter()
            .flatten()
            .any(|x| self.last_merge[k].get(*x).is_some_and(|t| *t > started));
        if conflict {
            let m = self.m;
            self.kinds[k].merge_conflicts += 1.0;
            self.rec_task(now, &m.workers[k].exec.part, "merge_conflict", Some(id));
            if m.workers[k].coordination == Coordination::Mesh {
                return false;
            }
            self.tasks[id].correct = false;
        }
        for x in touched.iter().flatten() {
            if let Some(t) = self.last_merge[k].get_mut(*x) {
                *t = now;
            }
        }
        true
    }

    /// One model call of worker kind `k` outside a task (a read, an empty run): priced and
    /// counted on its model, drawn from the coordination stream; it waits on no rate limit.
    /// Returns its duration.
    fn side_call(&mut self, k: usize) -> f64 {
        let e = &self.m.workers[k].exec;
        let route = e.route(&mut self.coord);
        let dur = e.turn_seconds.sample(&mut self.coord).max(0.0);
        let mut tokens = [0.0; 4];
        for (c, d) in e.role.tokens_per_turn.iter().enumerate() {
            tokens[c] = d.sample(&mut self.coord).max(0.0).round();
        }
        let cost: f64 = route.map_or(0.0, |r| {
            tokens.iter().zip(&r.price).map(|(t, p)| t * p).sum()
        });
        self.kinds[k].cost += cost;
        let Some(r) = route else {
            return dur;
        };
        self.kinds[k].model_calls += 1.0;
        if let Some(g) = r.gateway {
            self.gateways[g].calls += 1.0;
        }
        {
            let ma = &mut self.models[r.model];
            ma.calls += 1.0;
            ma.cost += cost;
            for (sum, t) in ma.tokens.iter_mut().zip(tokens) {
                *sum += t;
            }
        }
        dur
    }
}
