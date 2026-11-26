//! KPIs over replayed sessions. Every figure is computed from the events the generated component
//! published; nothing here reads a transcript.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use agentspec_system::SystemEvent;
use agentspec_types::cli as c;
use agentspec_types::session as s;
use rayon::prelude::*;

use crate::time::{epoch_ms, rfc3339};

const MINUTE: i64 = 60_000;
const ACTIVE_GAP: i64 = 5 * MINUTE;
const WORK_GAP: i64 = 10 * MINUTE;
const WAIT_SPLIT: i64 = 60 * MINUTE;
const BUST_FLOOR: i64 = 20_000;

fn scope_ix(sc: c::AgentScope) -> usize {
    match sc {
        c::AgentScope::Main => 0,
        c::AgentScope::Sub => 1,
    }
}
const SCOPES: [c::AgentScope; 2] = [c::AgentScope::Main, c::AgentScope::Sub];
const CAUSES: [c::BustCause; 4] = [
    c::BustCause::TtlExpired,
    c::BustCause::ModelChanged,
    c::BustCause::Compacted,
    c::BustCause::Unexplained,
];

fn cause_ix(cause: c::BustCause) -> usize {
    CAUSES.iter().position(|x| *x == cause).unwrap_or(3)
}

type ModelRow = (i64, i64, Vec<i64>, i64, i64);

/// Mergeable totals; one per session, folded into one per run.
#[derive(Default, Clone)]
pub struct Acc {
    pub transcripts: i64,
    pub sessions: i64,
    pub sub_agents: i64,
    pub refused: i64,
    /// Per scope and model: messages, output tokens, context sizes, cost in nanodollars, messages
    /// with no price.
    models: BTreeMap<(usize, String), ModelRow>,
    cache: [[i64; 3]; 2],
    busts: [[(i64, i64); 4]; 2],
    minutes: [Vec<u32>; 2],
    working_ms: i64,
    wait_ms: i64,
    away_ms: i64,
    prompts: Vec<(s::PromptAuthor, s::Attribution, i64)>,
    decisions: Vec<(s::Decider, s::Decision, i64)>,
    exposures: Vec<(
        s::ExposureChannel,
        s::ExposureClass,
        String,
        s::Verdict,
        i64,
        i64,
    )>,
    /// Per session only; `merge` leaves them out.
    pub marks: Vec<c::TimelineMark>,
    pub tools: Vec<c::ToolFigures>,
    pub first_ms: Option<i64>,
    pub last_ms: Option<i64>,
}

impl Acc {
    fn new(window_minutes: usize) -> Self {
        Acc {
            minutes: [vec![0; window_minutes], vec![0; window_minutes]],
            ..Default::default()
        }
    }

    pub fn merge(&mut self, o: Acc) {
        self.first_ms = match (self.first_ms, o.first_ms) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        self.last_ms = self.last_ms.max(o.last_ms);
        self.transcripts += o.transcripts;
        self.sessions += o.sessions;
        self.sub_agents += o.sub_agents;
        self.refused += o.refused;
        for (k, (m, out, ctx, cost, unpriced)) in o.models {
            let e = self.models.entry(k).or_default();
            e.0 += m;
            e.1 += out;
            e.2.extend(ctx);
            e.3 += cost;
            e.4 += unpriced;
        }
        for i in 0..2 {
            for j in 0..3 {
                self.cache[i][j] += o.cache[i][j];
            }
            for j in 0..4 {
                self.busts[i][j].0 += o.busts[i][j].0;
                self.busts[i][j].1 += o.busts[i][j].1;
            }
            if self.minutes[i].len() < o.minutes[i].len() {
                self.minutes[i].resize(o.minutes[i].len(), 0);
            }
            for (a, b) in self.minutes[i].iter_mut().zip(&o.minutes[i]) {
                *a += b;
            }
        }
        self.working_ms += o.working_ms;
        self.wait_ms += o.wait_ms;
        self.away_ms += o.away_ms;
        for (a, b, n) in o.prompts {
            match self.prompts.iter_mut().find(|r| r.0 == a && r.1 == b) {
                Some(r) => r.2 += n,
                None => self.prompts.push((a, b, n)),
            }
        }
        for (a, b, n) in o.decisions {
            match self.decisions.iter_mut().find(|r| r.0 == a && r.1 == b) {
                Some(r) => r.2 += n,
                None => self.decisions.push((a, b, n)),
            }
        }
        for (ch, cl, k, v, n, ss) in o.exposures {
            match self
                .exposures
                .iter_mut()
                .find(|r| r.0 == ch && r.1 == cl && r.2 == k && r.3 == v)
            {
                Some(r) => {
                    r.4 += n;
                    r.5 += ss;
                }
                None => self.exposures.push((ch, cl, k, v, n, ss)),
            }
        }
    }

    pub fn report(mut self, from_ms: i64, to_ms: i64) -> c::KpiReport {
        let pct = |v: &[i64], q: f64| -> i64 {
            if v.is_empty() {
                return 0;
            }
            v[((q * v.len() as f64) as usize).min(v.len() - 1)]
        };
        let mut models: Vec<c::ModelFigures> = Vec::new();
        let (mut cost_total, mut unpriced_total) = (0i64, 0i64);
        for ((sc, model), (messages, out, mut ctx, cost, unpriced)) in
            std::mem::take(&mut self.models)
        {
            cost_total += cost;
            unpriced_total += unpriced;
            ctx.sort_unstable();
            models.push(c::ModelFigures {
                scope: SCOPES[sc],
                model,
                messages,
                output_tokens: out,
                context_p50: pct(&ctx, 0.5),
                context_p90: pct(&ctx, 0.9),
                context_max: ctx.last().copied().unwrap_or(0),
                cost_microdollars: cost / 1000,
            });
        }
        models.sort_by(|a, b| {
            scope_ix(a.scope)
                .cmp(&scope_ix(b.scope))
                .then(b.messages.cmp(&a.messages))
        });
        let cache = SCOPES
            .iter()
            .map(|sc| {
                let [read, write, unc] = self.cache[scope_ix(*sc)];
                let total = read + write + unc;
                c::CacheFigures {
                    scope: *sc,
                    read_tokens: read,
                    write_tokens: write,
                    uncached_tokens: unc,
                    hit_permille: if total > 0 { read * 1000 / total } else { 0 },
                }
            })
            .collect();
        let mut busts = Vec::new();
        for sc in SCOPES {
            for cause in CAUSES {
                let (n, t) = self.busts[scope_ix(sc)][cause_ix(cause)];
                if n > 0 {
                    busts.push(c::BustFigures {
                        scope: sc,
                        cause,
                        count: n,
                        tokens_written_again: t,
                    });
                }
            }
        }
        let concurrency = SCOPES
            .iter()
            .map(|sc| {
                let mut active: Vec<i64> = self.minutes[scope_ix(*sc)]
                    .iter()
                    .filter(|n| **n > 0)
                    .map(|n| *n as i64)
                    .collect();
                active.sort_unstable();
                c::ConcurrencyFigures {
                    scope: *sc,
                    p50: pct(&active, 0.5),
                    p90: pct(&active, 0.9),
                    max: active.last().copied().unwrap_or(0),
                }
            })
            .collect();
        let mut prompts: Vec<c::PromptFigures> = self
            .prompts
            .iter()
            .map(|(a, b, n)| c::PromptFigures {
                author: *a,
                attribution: *b,
                count: *n,
            })
            .collect();
        prompts.sort_by_key(|f| std::cmp::Reverse(f.count));
        let mut decisions: Vec<c::DecisionFigures> = self
            .decisions
            .iter()
            .map(|(a, b, n)| c::DecisionFigures {
                decider: *a,
                decision: *b,
                count: *n,
            })
            .collect();
        decisions.sort_by_key(|f| std::cmp::Reverse(f.count));
        let mut exposures: Vec<c::ExposureFigures> = self
            .exposures
            .iter()
            .map(|(ch, cl, k, v, n, ss)| c::ExposureFigures {
                channel: *ch,
                class: *cl,
                kind: k.clone(),
                verdict: *v,
                count: *n,
                sessions: *ss,
            })
            .collect();
        exposures.sort_by_key(|f| std::cmp::Reverse(f.count));
        c::KpiReport {
            window_from: rfc3339(from_ms),
            window_to: rfc3339(to_ms),
            transcripts: self.transcripts,
            sessions: self.sessions,
            sub_agents: self.sub_agents,
            refused_commands: self.refused,
            cost_microdollars: cost_total / 1000,
            unpriced_messages: unpriced_total,
            models,
            cache,
            busts,
            concurrency,
            autonomy: c::AutonomyFigures {
                working_seconds: self.working_ms / 1000,
                waiting_on_human_seconds: self.wait_ms / 1000,
                away_seconds: self.away_ms / 1000,
                prompts,
                decisions,
            },
            exposures,
        }
    }
}

#[derive(Default)]
struct AgentTrack {
    scope: usize,
    last_ms: Option<i64>,
    active_in_window: bool,
    working: bool,
    prev_turn: Option<(i64, i64, String, bool)>,
    compacted: bool,
    minutes: HashSet<u32>,
}

fn agent_of(e: &SystemEvent, tools: &HashMap<String, String>) -> Option<(String, String)> {
    use SystemEvent as E;
    let (agent, at) = match e {
        E::AgentStarted(x) => (x.agent_id.0.clone(), &x.at),
        E::PromptSubmitted(x) => (x.agent_id.0.clone(), &x.at),
        E::TurnEnded(x) => (x.agent_id.0.clone(), &x.at),
        E::ModelTurnRecorded(x) => (x.agent_id.0.clone(), &x.at),
        E::ContextCompacted(x) => (x.agent_id.0.clone(), &x.at),
        E::ToolCallRequested(x) => (x.agent_id.0.clone(), &x.at),
        E::ExposureDetected(x) => (x.agent_id.0.clone(), &x.at),
        E::ToolCallDecided(x) => (tools.get(&x.tool_call_id.0)?.clone(), &x.at),
        E::ToolCallCompleted(x) => (tools.get(&x.tool_call_id.0)?.clone(), &x.at),
        _ => return None,
    };
    Some((agent, at.0.clone()))
}

/// The totals of one replayed session, counting only events inside the window.
pub fn session_facts(events: &[SystemEvent], refused: usize, from_ms: i64, to_ms: i64) -> Acc {
    let mut f = Facts::new(from_ms, to_ms);
    for e in events {
        f.push(e);
    }
    f.into_acc(refused, to_ms)
}

/// [`session_facts`] as a fold: one published event at a time, so a live source counts only what
/// a session appended. A window that ends at `i64::MAX` counts every event from `from_ms` on, and
/// [`Facts::acc`] then closes it at the session's last event, as `session_facts` over the whole
/// session with that end does.
pub struct Facts {
    from_ms: i64,
    to_ms: i64,
    window_minutes: usize,
    acc: Acc,
    agents: HashMap<String, AgentTrack>,
    tools: HashMap<String, String>,
    categories: HashMap<String, s::ToolCategory>,
    exposures_seen: HashSet<(String, String, String, String)>,
    any_in_window: bool,
    // The session's state over time: the agents Working, and the time in which none was.
    busy: HashSet<String>,
    quiet_since: Option<i64>,
    quiet_total: i64,
    main_quiet_mark: Option<i64>,
    session_last: Option<i64>,
}

impl Facts {
    pub fn new(from_ms: i64, to_ms: i64) -> Facts {
        Facts {
            from_ms,
            to_ms,
            window_minutes: (to_ms.saturating_sub(from_ms) / MINUTE + 1).max(1) as usize,
            acc: Acc::new(0),
            agents: HashMap::new(),
            tools: HashMap::new(),
            categories: HashMap::new(),
            exposures_seen: HashSet::new(),
            any_in_window: false,
            busy: HashSet::new(),
            quiet_since: None,
            quiet_total: 0,
            main_quiet_mark: None,
            session_last: None,
        }
    }

    /// The start of the window.
    pub fn from_ms(&self) -> i64 {
        self.from_ms
    }

    /// Drops the timeline marks kept so far; a reader of [`Facts::acc`]'s report needs none.
    pub fn drop_marks(&mut self) {
        self.acc.marks.clear();
    }

    /// The totals so far, the window closed at `window_to`.
    pub fn acc(&self, refused: usize, window_to: i64) -> Acc {
        close(
            self.acc.clone(),
            &self.agents,
            self.any_in_window,
            self.from_ms,
            refused,
            window_to,
        )
    }

    /// The totals, the window closed at `window_to`.
    pub fn into_acc(self, refused: usize, window_to: i64) -> Acc {
        close(
            self.acc,
            &self.agents,
            self.any_in_window,
            self.from_ms,
            refused,
            window_to,
        )
    }

    /// Counts one published event.
    pub fn push(&mut self, e: &SystemEvent) {
        fold_one(
            e,
            &mut Fold {
                from_ms: self.from_ms,
                to_ms: self.to_ms,
                window_minutes: self.window_minutes,
                acc: &mut self.acc,
                agents: &mut self.agents,
                tools: &mut self.tools,
                categories: &mut self.categories,
                exposures_seen: &mut self.exposures_seen,
                any_in_window: &mut self.any_in_window,
                busy: &mut self.busy,
                quiet_since: &mut self.quiet_since,
                quiet_total: &mut self.quiet_total,
                main_quiet_mark: &mut self.main_quiet_mark,
                session_last: &mut self.session_last,
            },
        );
    }
}

/// Closes a window: the per-minute concurrency counts over `[from_ms, window_to]`, the refused
/// commands, and whether the session counts as one.
fn close(
    mut acc: Acc,
    agents: &HashMap<String, AgentTrack>,
    any_in_window: bool,
    from_ms: i64,
    refused: usize,
    window_to: i64,
) -> Acc {
    let window_minutes = ((window_to - from_ms) / MINUTE + 1).max(1) as usize;
    acc.minutes = [vec![0; window_minutes], vec![0; window_minutes]];
    acc.refused = refused as i64;
    for tr in agents.values() {
        for m in &tr.minutes {
            acc.minutes[tr.scope][*m as usize] += 1;
        }
    }
    if any_in_window {
        acc.sessions = 1;
        acc.sub_agents = agents
            .values()
            .filter(|a| a.scope == 1 && a.active_in_window)
            .count() as i64;
    }
    acc
}

/// The state [`fold_one`] reads and changes.
struct Fold<'a> {
    from_ms: i64,
    to_ms: i64,
    window_minutes: usize,
    acc: &'a mut Acc,
    agents: &'a mut HashMap<String, AgentTrack>,
    tools: &'a mut HashMap<String, String>,
    categories: &'a mut HashMap<String, s::ToolCategory>,
    exposures_seen: &'a mut HashSet<(String, String, String, String)>,
    any_in_window: &'a mut bool,
    busy: &'a mut HashSet<String>,
    quiet_since: &'a mut Option<i64>,
    quiet_total: &'a mut i64,
    main_quiet_mark: &'a mut Option<i64>,
    session_last: &'a mut Option<i64>,
}

#[allow(clippy::too_many_lines)]
fn fold_one(e: &SystemEvent, f: &mut Fold<'_>) {
    let (from_ms, to_ms, window_minutes) = (f.from_ms, f.to_ms, f.window_minutes);
    let acc = &mut *f.acc;
    let agents = &mut *f.agents;
    let tools = &mut *f.tools;
    let categories = &mut *f.categories;
    let exposures_seen = &mut *f.exposures_seen;
    let busy = &mut *f.busy;
    let mut any_in_window = *f.any_in_window;
    let mut quiet_since = *f.quiet_since;
    let mut quiet_total = *f.quiet_total;
    let mut main_quiet_mark = *f.main_quiet_mark;
    let mut session_last = *f.session_last;
    // One pass of the former loop over all events: its `continue`s end this event, and the state
    // below the loop is written back either way.
    #[allow(clippy::never_loop)]
    for e in std::iter::once(e) {
        if let SystemEvent::ToolCallRequested(x) = e {
            tools.insert(x.tool_call_id.0.clone(), x.agent_id.0.clone());
            categories.insert(x.tool_call_id.0.clone(), x.category);
        }
        if let SystemEvent::AgentStarted(x) = e {
            let scope = if x.role == s::AgentRole::Main { 0 } else { 1 };
            agents.insert(
                x.agent_id.0.clone(),
                AgentTrack {
                    scope,
                    ..Default::default()
                },
            );
        }
        let Some((agent, at)) = agent_of(e, tools) else {
            continue;
        };
        let Some(t) = epoch_ms(&at) else { continue };
        let in_window = t >= from_ms && t <= to_ms;
        let Some(tr) = agents.get_mut(&agent) else {
            continue;
        };
        let scope = tr.scope;
        // Working time: at least one agent of the session Working, gaps up to 10 minutes.
        if !busy.is_empty()
            && let Some(p) = session_last
            && in_window
            && t > p
            && t - p <= WORK_GAP
        {
            acc.working_ms += t - p;
        }
        session_last = Some(session_last.map_or(t, |p| p.max(t)));

        // Concurrency: minutes between two records at most 5 minutes apart.
        if let Some(prev) = tr.last_ms
            && t - prev <= ACTIVE_GAP
            && t >= from_ms
            && prev <= to_ms
        {
            let a = ((prev.max(from_ms) - from_ms) / MINUTE) as usize;
            let b = ((t.min(to_ms) - from_ms) / MINUTE) as usize;
            for m in a..=b.min(window_minutes - 1) {
                tr.minutes.insert(m as u32);
            }
        }
        if in_window {
            any_in_window = true;
            tr.active_in_window = true;
        }
        if in_window {
            acc.first_ms = Some(acc.first_ms.map_or(t, |f| f.min(t)));
            acc.last_ms = Some(acc.last_ms.map_or(t, |l| l.max(t)));
            let sc = SCOPES[scope];
            let mut mark = |kind, value| {
                acc.marks.push(c::TimelineMark {
                    at: at.clone(),
                    kind,
                    scope: sc,
                    value,
                })
            };
            match e {
                SystemEvent::PromptSubmitted(x) => mark(
                    match x.author {
                        s::PromptAuthor::Human => c::MarkKind::HumanPrompt,
                        s::PromptAuthor::Delegate => c::MarkKind::DelegatePrompt,
                        s::PromptAuthor::PeerSession => c::MarkKind::PeerPrompt,
                        _ => c::MarkKind::HarnessPrompt,
                    },
                    0,
                ),
                SystemEvent::ModelTurnRecorded(x) => mark(c::MarkKind::Turn, x.context_tokens),
                SystemEvent::ContextCompacted(_) => mark(c::MarkKind::Compaction, 0),
                SystemEvent::ToolCallDecided(x) if x.decision == s::Decision::Refuse => {
                    mark(c::MarkKind::Refusal, 0)
                }
                _ => {}
            }
            let tool_row = |acc: &mut Acc, id: &str| -> Option<usize> {
                let cat = *categories.get(id)?;
                Some(match acc.tools.iter().position(|r| r.category == cat) {
                    Some(i) => i,
                    None => {
                        acc.tools.push(c::ToolFigures {
                            category: cat,
                            calls: 0,
                            refused: 0,
                            failed: 0,
                        });
                        acc.tools.len() - 1
                    }
                })
            };
            match e {
                SystemEvent::ToolCallRequested(x) => {
                    if let Some(i) = tool_row(acc, &x.tool_call_id.0) {
                        acc.tools[i].calls += 1;
                    }
                }
                SystemEvent::ToolCallDecided(x) if x.decision == s::Decision::Refuse => {
                    if let Some(i) = tool_row(acc, &x.tool_call_id.0) {
                        acc.tools[i].refused += 1;
                    }
                }
                SystemEvent::ToolCallCompleted(x) if !x.ok => {
                    if let Some(i) = tool_row(acc, &x.tool_call_id.0) {
                        acc.tools[i].failed += 1;
                    }
                }
                _ => {}
            }
        }

        match e {
            SystemEvent::PromptSubmitted(x) => {
                if scope == 0 {
                    if in_window {
                        match acc
                            .prompts
                            .iter_mut()
                            .find(|r| r.0 == x.author && r.1 == x.attribution)
                        {
                            Some(r) => r.2 += 1,
                            None => acc.prompts.push((x.author, x.attribution, 1)),
                        }
                    }
                    if !tr.working
                        && let Some(mark) = main_quiet_mark
                        && in_window
                        && matches!(x.author, s::PromptAuthor::Human | s::PromptAuthor::Delegate)
                    {
                        let quiet = quiet_total + quiet_since.map_or(0, |q| (t - q).max(0)) - mark;
                        if quiet <= WAIT_SPLIT {
                            acc.wait_ms += quiet;
                        } else {
                            acc.away_ms += quiet;
                        }
                    }
                }
                tr.working = true;
                if busy.is_empty()
                    && let Some(q) = quiet_since.take()
                {
                    quiet_total += (t - q).max(0);
                }
                busy.insert(agent.clone());
            }
            SystemEvent::TurnEnded(_) => {
                tr.working = false;
                busy.remove(&agent);
                if busy.is_empty() && quiet_since.is_none() {
                    quiet_since = Some(t);
                }
                if scope == 0 {
                    main_quiet_mark = Some(quiet_total + quiet_since.map_or(0, |q| (t - q).max(0)));
                }
            }
            SystemEvent::ContextCompacted(_) => tr.compacted = true,
            SystemEvent::ModelTurnRecorded(x) => {
                let u = &x.usage;
                let write = u.cache_write_5m + u.cache_write_1h;
                if in_window {
                    let row = acc.models.entry((scope, x.model.clone())).or_default();
                    row.0 += 1;
                    row.1 += u.output;
                    row.2.push(x.context_tokens);
                    match crate::pricing::cost_nanodollars(&x.model, u) {
                        Some(n) => row.3 += n,
                        None => row.4 += 1,
                    }
                    acc.cache[scope][0] += u.cache_read;
                    acc.cache[scope][1] += write;
                    acc.cache[scope][2] += u.uncached_input;
                }
                if let Some((pctx, pt, pmodel, p1h)) = &tr.prev_turn
                    && *pctx > BUST_FLOOR
                    && u.cache_read * 2 < *pctx
                    && write * 2 > *pctx
                    && in_window
                {
                    let ttl = if *p1h { 60 * MINUTE } else { 5 * MINUTE };
                    let cause = if tr.compacted {
                        c::BustCause::Compacted
                    } else if *pmodel != x.model {
                        c::BustCause::ModelChanged
                    } else if t - pt > ttl {
                        c::BustCause::TtlExpired
                    } else {
                        c::BustCause::Unexplained
                    };
                    let cell = &mut acc.busts[scope][cause_ix(cause)];
                    cell.0 += 1;
                    cell.1 += write;
                    acc.marks.push(c::TimelineMark {
                        at: at.clone(),
                        kind: c::MarkKind::CacheBust,
                        scope: SCOPES[scope],
                        value: write,
                    });
                }
                tr.compacted = false;
                tr.prev_turn = Some((
                    x.context_tokens,
                    t,
                    x.model.clone(),
                    u.cache_write_1h > u.cache_write_5m,
                ));
            }
            SystemEvent::ToolCallDecided(x) if scope == 0 && in_window => {
                match acc
                    .decisions
                    .iter_mut()
                    .find(|r| r.0 == x.decider && r.1 == x.decision)
                {
                    Some(r) => r.2 += 1,
                    None => acc.decisions.push((x.decider, x.decision, 1)),
                }
            }
            SystemEvent::ExposureDetected(x) if in_window => {
                let key = (
                    format!("{:?}", x.channel),
                    format!("{:?}", x.class),
                    x.kind.clone(),
                    format!("{:?}", x.verdict),
                );
                let first = exposures_seen.insert(key);
                match acc.exposures.iter_mut().find(|r| {
                    r.0 == x.channel && r.1 == x.class && r.2 == x.kind && r.3 == x.verdict
                }) {
                    Some(r) => r.4 += 1,
                    None => {
                        acc.exposures
                            .push((x.channel, x.class, x.kind.clone(), x.verdict, 1, 0))
                    }
                }
                if first
                    && let Some(r) = acc.exposures.iter_mut().find(|r| {
                        r.0 == x.channel && r.1 == x.class && r.2 == x.kind && r.3 == x.verdict
                    })
                {
                    r.5 += 1;
                }
            }
            _ => {}
        }
        tr.last_ms = Some(t);
    }
    *f.any_in_window = any_in_window;
    *f.quiet_since = quiet_since;
    *f.quiet_total = quiet_total;
    *f.main_quiet_mark = main_quiet_mark;
    *f.session_last = session_last;
}

fn mtime_ms(p: &Path) -> i64 {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Main transcripts under `root` that have a file of theirs modified at or after `from_ms`.
pub fn sessions_in(root: &Path, from_ms: i64) -> std::io::Result<Vec<(PathBuf, i64)>> {
    let mut out = Vec::new();
    for project in fs::read_dir(root)?.flatten() {
        let Ok(entries) = fs::read_dir(project.path()) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("jsonl") {
                continue;
            }
            let mut recent = i64::from(mtime_ms(&p) >= from_ms);
            for sub in crate::claude::sub_agent_files(&p) {
                if mtime_ms(&sub.path) >= from_ms {
                    recent += 1;
                }
            }
            if recent > 0 {
                out.push((p, recent));
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn run(root: &Path, from_ms: i64, to_ms: i64) -> std::io::Result<c::KpiReport> {
    run_with(root, from_ms, to_ms, &crate::claude::lift)
}

/// The KPIs of the window, the lifted commands of each session taken from `load`.
pub fn run_with(
    root: &Path,
    from_ms: i64,
    to_ms: i64,
    load: crate::Load<'_>,
) -> std::io::Result<c::KpiReport> {
    let sessions = sessions_in(root, from_ms)?;
    let window_minutes = ((to_ms - from_ms) / MINUTE + 1).max(1) as usize;
    let acc = sessions
        .par_iter()
        .map(|(main, recent)| {
            let mut acc = match load(main) {
                Ok(lifted) => {
                    let replayed = crate::replay::replay(&lifted.steps);
                    session_facts(&replayed.events, replayed.refused, from_ms, to_ms)
                }
                Err(_) => Acc::new(window_minutes),
            };
            acc.transcripts = *recent;
            acc
        })
        .reduce(
            || Acc::new(window_minutes),
            |mut a, b| {
                a.merge(b);
                a
            },
        );
    Ok(acc.report(from_ms, to_ms))
}

/// The largest number of sub-agent turn marks one session keeps; the rest are sampled evenly.
const SUB_TURN_MARKS: usize = 1500;

fn thin(marks: Vec<c::TimelineMark>) -> Vec<c::TimelineMark> {
    let sub_turns = marks
        .iter()
        .filter(|m| m.kind == c::MarkKind::Turn && m.scope == c::AgentScope::Sub)
        .count();
    if sub_turns <= SUB_TURN_MARKS {
        return marks;
    }
    let step = sub_turns.div_ceil(SUB_TURN_MARKS);
    let mut n = 0usize;
    marks
        .into_iter()
        .filter(|m| {
            if m.kind == c::MarkKind::Turn && m.scope == c::AgentScope::Sub {
                n += 1;
                n.is_multiple_of(step)
            } else {
                true
            }
        })
        .collect()
}

/// Every session with an event in the window, each with its own KPI report, and the overall one.
pub fn dashboard(
    root: &Path,
    from_ms: i64,
    to_ms: i64,
    now_ms: i64,
) -> std::io::Result<c::Dashboard> {
    dashboard_with(root, from_ms, to_ms, now_ms, &crate::claude::lift)
}

/// The dashboard of the window, the lifted commands of each session taken from `load`.
pub fn dashboard_with(
    root: &Path,
    from_ms: i64,
    to_ms: i64,
    now_ms: i64,
    load: crate::Load<'_>,
) -> std::io::Result<c::Dashboard> {
    let sessions = sessions_in(root, from_ms)?;
    let window_minutes = ((to_ms - from_ms) / MINUTE + 1).max(1) as usize;
    let parts: Vec<(Acc, Option<c::SessionSummary>)> = sessions
        .par_iter()
        .map(|(main, recent)| {
            let Ok(lifted) = load(main) else {
                return (Acc::new(window_minutes), None);
            };
            let replayed = crate::replay::replay(&lifted.steps);
            let mut acc = session_facts(&replayed.events, replayed.refused, from_ms, to_ms);
            acc.transcripts = *recent;
            if acc.sessions == 0 {
                return (acc, None);
            }
            let mut own = acc.clone();
            let marks = thin(std::mem::take(&mut own.marks));
            let mut tools = std::mem::take(&mut own.tools);
            tools.sort_by_key(|t| std::cmp::Reverse(t.calls));
            let (first, last) = (
                own.first_ms.unwrap_or(from_ms),
                own.last_ms.unwrap_or(to_ms),
            );
            let project = main
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_owned();
            let summary = c::SessionSummary {
                session_id: lifted.session_id.clone(),
                project,
                title: lifted.title.clone(),
                first_at: rfc3339(first),
                last_at: rfc3339(last),
                kpis: own.report(first, last),
                tools,
                timeline: marks,
            };
            acc.marks.clear();
            (acc, Some(summary))
        })
        .collect();
    let mut overall = Acc::new(window_minutes);
    let mut summaries = Vec::new();
    for (acc, summary) in parts {
        overall.merge(acc);
        summaries.extend(summary);
    }
    summaries.sort_by(|a, b| b.last_at.cmp(&a.last_at));
    Ok(c::Dashboard {
        generated_at: rfc3339(now_ms),
        overall: overall.report(from_ms, to_ms),
        sessions: summaries,
    })
}
