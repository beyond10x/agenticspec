//! Pattern mining over the session IR (`design/query-design.md` § 6) and the lift of one pattern
//! into a procedure (§ 6.3).
//!
//! Each agent turn becomes a sequence of tokens, one per tool call, never holding content. A run
//! of the same token becomes `T+`. Patterns are contiguous runs of 2 to 8 tokens; support counts
//! distinct sessions and projects; a pattern is kept when no pattern one token longer has the same
//! session support (closed).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use agenticspec_types::cli as c;
use rayon::prelude::*;

use crate::sq::facts::{FactBase, Val};

pub const PATTERNS_FORMAT: &str = "agenticspec-patterns/1";
pub const PROCEDURE_FORMAT: &str = "agenticspec-procedure/1";
const MIN_LEN: usize = 2;
const MAX_LEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Coarse,
    Tool,
}

impl Level {
    pub fn parse(s: Option<&str>) -> Result<Level, String> {
        match s.unwrap_or("tool") {
            "coarse" => Ok(Level::Coarse),
            "tool" => Ok(Level::Tool),
            other => Err(format!("level `{other}`: coarse or tool")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Level::Coarse => "coarse",
            Level::Tool => "tool",
        }
    }
}

/// One tool call as mining sees it.
#[derive(Debug, Clone)]
pub struct CallFact {
    pub node: Arc<str>,
    pub tool: String,
    pub category: String,
    pub ok: Option<bool>,
    pub refused_by: Option<String>,
    pub dur_ms: i64,
    pub decided_by_human: bool,
    pub shape: Option<String>,
}

/// One token of a turn and the calls it stands for (several for `T+`).
#[derive(Debug, Clone)]
pub struct Tok {
    pub token: String,
    pub calls: Vec<CallFact>,
}

pub struct Turn {
    pub tokens: Vec<Tok>,
}

fn s(v: &Val) -> String {
    v.to_string()
}

fn call_token(level: Level, c: &CallFact) -> String {
    let base = match level {
        Level::Coarse => c.category.to_lowercase(),
        Level::Tool => match &c.shape {
            Some(sh) => format!("exec:{sh}"),
            None => c.tool.clone(),
        },
    };
    match &c.refused_by {
        Some(d) => format!("{base}/refused:{d}"),
        None => base,
    }
}

/// The turns of every agent of a session as token sequences, in agent order.
pub fn turns(fb: &FactBase, level: Level) -> Vec<Turn> {
    let t = |name: &str| fb.tables.get(name);
    let mut completed: HashMap<String, (bool, i64)> = HashMap::new();
    if let Some(tb) = t("completed") {
        for r in &tb.rows {
            let ok = matches!(r.vals[2], Val::Bool(true));
            let dur = match r.vals[5] {
                Val::Int(n) => n,
                _ => 0,
            };
            completed.insert(s(&r.vals[1]), (ok, dur));
        }
    }
    let mut decided: HashMap<String, (String, String)> = HashMap::new();
    if let Some(tb) = t("decided") {
        for r in &tb.rows {
            decided.insert(s(&r.vals[1]), (s(&r.vals[2]), s(&r.vals[3])));
        }
    }
    let mut calls: HashMap<Arc<str>, CallFact> = HashMap::new();
    if let Some(tb) = t("call") {
        for r in &tb.rows {
            let Val::Str(node) = &r.vals[0] else { continue };
            let id = s(&r.vals[1]);
            let (decision, decider) = decided.get(&id).cloned().unwrap_or_default();
            let done = completed.get(&id).copied();
            calls.insert(
                node.clone(),
                CallFact {
                    node: node.clone(),
                    tool: s(&r.vals[3]),
                    category: s(&r.vals[4]),
                    ok: done.map(|d| d.0),
                    refused_by: (decision == "Refuse").then_some(decider.clone()),
                    dur_ms: done.map_or(0, |d| d.1),
                    decided_by_human: decider == "Human",
                    shape: fb.shapes.get(node).cloned(),
                },
            );
        }
    }
    let mut turn_of: HashMap<Arc<str>, Arc<str>> = HashMap::new();
    if let Some(tb) = t("turn_of") {
        for r in &tb.rows {
            if let (Val::Str(n), Val::Str(p)) = (&r.vals[0], &r.vals[1]) {
                turn_of.insert(n.clone(), p.clone());
            }
        }
    }
    let mut out = Vec::new();
    let mut agents: Vec<&Arc<str>> = fb.agent_order.keys().collect();
    agents.sort();
    for a in agents {
        let mut cur: Option<Arc<str>> = None;
        let mut toks: Vec<Tok> = Vec::new();
        for n in &fb.agent_order[a] {
            let Some(cf) = calls.get(n) else { continue };
            let turn = turn_of.get(n).cloned();
            if turn != cur {
                if toks.len() >= MIN_LEN {
                    out.push(Turn {
                        tokens: std::mem::take(&mut toks),
                    });
                }
                toks.clear();
                cur = turn;
            }
            let token = call_token(level, cf);
            match toks.last_mut() {
                Some(last) if last.token == token || last.token == format!("{token}+") => {
                    if !last.token.ends_with('+') {
                        last.token.push('+');
                    }
                    last.calls.push(cf.clone());
                }
                _ => toks.push(Tok {
                    token,
                    calls: vec![cf.clone()],
                }),
            }
        }
        if toks.len() >= MIN_LEN {
            out.push(Turn { tokens: toks });
        }
    }
    out
}

#[derive(Default)]
struct Stat {
    sessions: BTreeSet<usize>,
    projects: BTreeSet<Arc<str>>,
    occurrences: i64,
    ok: i64,
    done: i64,
}

fn project_of(p: &std::path::Path) -> Arc<str> {
    Arc::from(
        p.parent()
            .and_then(|d| d.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(""),
    )
}

/// The sq query that finds a pattern again: one node per token, in one agent's order, within
/// 15 minutes. It is an over-approximation: the nodes need not be adjacent.
pub fn pattern_query(level: Level, tokens: &[String]) -> String {
    let mut atoms = Vec::new();
    let mut nodes = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        let n = format!("N{}", i + 1);
        let t = t.trim_end_matches('+');
        let (base, refused) = match t.split_once("/refused:") {
            Some((b, d)) => (b, Some(d)),
            None => (t, None),
        };
        let field = match level {
            Level::Coarse => {
                let mut ch = base.chars();
                let cat: String = ch
                    .next()
                    .map(|f| f.to_uppercase().chain(ch).collect())
                    .unwrap_or_default();
                format!("category: \"{cat}\"")
            }
            Level::Tool => match base.strip_prefix("exec:") {
                Some(_) => "tool: \"Bash\"".to_owned(),
                None => format!("tool: \"{base}\""),
            },
        };
        match refused {
            Some(d) => {
                atoms.push(format!("call{{node: {n}, id: I{}, {field}}}", i + 1));
                atoms.push(format!(
                    "decided{{call: I{}, decision: \"Refuse\", decider: \"{d}\"}}",
                    i + 1
                ));
            }
            None => atoms.push(format!("call{{node: {n}, {field}}}")),
        }
        nodes.push(n);
    }
    format!(
        "?- {}, seq({}, scope: agent, within: 15m).",
        atoms.join(", "),
        nodes.join(", ")
    )
}

fn pattern_id(tokens: &[String]) -> String {
    let d = crate::redact::digest_hex(tokens.join(" ").as_bytes());
    format!("p-{}", &d[..10])
}

/// One session as mining reads it: its transcript, id and token turns.
pub struct Mined {
    pub path: PathBuf,
    pub session: String,
    pub turns: Vec<Turn>,
}

pub fn corpus(mains: &[PathBuf], level: Level) -> Vec<Mined> {
    corpus_with(mains, level, &crate::claude::lift)
}

/// The corpus of `mains`, their lifted commands taken from `load`.
pub fn corpus_with(mains: &[PathBuf], level: Level, load: crate::Load<'_>) -> Vec<Mined> {
    let mut v: Vec<Mined> = mains
        .par_iter()
        .filter_map(|m| {
            let fb = crate::sq::fact_base_with(m, load).ok()?;
            Some(Mined {
                path: m.clone(),
                session: fb.session.clone(),
                turns: turns(&fb, level),
            })
        })
        .collect();
    v.sort_by(|a, b| a.path.cmp(&b.path));
    v
}

pub fn mine(
    bases: &[Mined],
    level: Level,
    min_sessions: usize,
    min_projects: usize,
    top: usize,
) -> c::MineReport {
    let per: Vec<(usize, Arc<str>, &Vec<Turn>)> = bases
        .iter()
        .enumerate()
        .map(|(i, m)| (i, project_of(&m.path), &m.turns))
        .collect();
    let mut stats: HashMap<Vec<String>, Stat> = HashMap::new();
    let mut turn_count = 0;
    for (si, project, turns) in &per {
        turn_count += turns.len();
        for t in turns.iter() {
            let toks: Vec<&str> = t.tokens.iter().map(|x| x.token.as_str()).collect();
            for len in MIN_LEN..=MAX_LEN.min(toks.len()) {
                for start in 0..=toks.len() - len {
                    let key: Vec<String> = toks[start..start + len]
                        .iter()
                        .map(|x| (*x).to_owned())
                        .collect();
                    let st = stats.entry(key).or_default();
                    st.sessions.insert(*si);
                    st.projects.insert(project.clone());
                    st.occurrences += 1;
                    for tk in &t.tokens[start..start + len] {
                        for cf in &tk.calls {
                            if let Some(ok) = cf.ok {
                                st.done += 1;
                                st.ok += i64::from(ok);
                            }
                        }
                    }
                }
            }
        }
    }
    stats.retain(|_, st| st.sessions.len() >= min_sessions && st.projects.len() >= min_projects);
    // Closed: drop a pattern when a pattern one token longer holds it with the same support.
    let support: HashMap<&Vec<String>, usize> =
        stats.iter().map(|(k, v)| (k, v.sessions.len())).collect();
    let mut covered: BTreeSet<Vec<String>> = BTreeSet::new();
    for (k, n) in &support {
        if k.len() <= MIN_LEN {
            continue;
        }
        for sub in [k[1..].to_vec(), k[..k.len() - 1].to_vec()] {
            if support.get(&sub) == Some(n) {
                covered.insert(sub);
            }
        }
    }
    let mut kept: Vec<(Vec<String>, Stat)> = stats
        .into_iter()
        .filter(|(k, _)| !covered.contains(k))
        .collect();
    kept.sort_by(|a, b| {
        b.1.sessions
            .len()
            .cmp(&a.1.sessions.len())
            .then(b.0.len().cmp(&a.0.len()))
            .then(a.0.cmp(&b.0))
    });
    kept.truncate(top);
    c::MineReport {
        format: PATTERNS_FORMAT.to_owned(),
        level: level.name().to_owned(),
        sessions: bases.len() as i64,
        turns: turn_count as i64,
        patterns: kept
            .into_iter()
            .map(|(k, st)| c::Pattern {
                id: pattern_id(&k),
                query: pattern_query(level, &k),
                sessions: st.sessions.len() as i64,
                projects: st.projects.len() as i64,
                occurrences: st.occurrences,
                ok_permille: if st.done == 0 {
                    0
                } else {
                    st.ok * 1000 / st.done
                },
                tokens: k,
            })
            .collect(),
    }
}

fn effect(category: &str) -> &'static str {
    match category {
        "Read" | "Search" | "Plan" => "read",
        "Edit" => "write",
        "Network" | "Message" => "external",
        _ => "none",
    }
}

/// Lifts a pattern into a procedure over every contiguous match of its tokens.
pub fn procedure(bases: &[Mined], level: Level, tokens: &[String]) -> Option<c::Procedure> {
    let k = tokens.len();
    // Per step: the calls of each match.
    let mut matches: Vec<Vec<Vec<CallFact>>> = Vec::new();
    let mut sessions: BTreeSet<String> = BTreeSet::new();
    for mb in bases {
        for t in &mb.turns {
            let toks: Vec<&str> = t.tokens.iter().map(|x| x.token.as_str()).collect();
            if toks.len() < k {
                continue;
            }
            for start in 0..=toks.len() - k {
                if toks[start..start + k]
                    .iter()
                    .zip(tokens)
                    .all(|(a, b)| *a == b.as_str())
                {
                    matches.push(
                        t.tokens[start..start + k]
                            .iter()
                            .map(|x| x.calls.clone())
                            .collect(),
                    );
                    sessions.insert(mb.session.clone());
                }
            }
        }
    }
    if matches.is_empty() {
        return None;
    }
    let mut steps = Vec::new();
    let mut parameters = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let id = format!("s{}", i + 1);
        let calls: Vec<&CallFact> = matches.iter().flat_map(|m| m[i].iter()).collect();
        let mut tools: BTreeMap<&str, usize> = BTreeMap::new();
        let mut cats: BTreeMap<&str, usize> = BTreeMap::new();
        let mut shapes: BTreeSet<Option<&str>> = BTreeSet::new();
        let (mut ok, mut done, mut max_dur, mut retry) = (0i64, 0i64, 0i64, 0usize);
        let mut human = 0usize;
        for cf in &calls {
            *tools.entry(cf.tool.as_str()).or_default() += 1;
            *cats.entry(cf.category.as_str()).or_default() += 1;
            shapes.insert(cf.shape.as_deref());
            if let Some(o) = cf.ok {
                done += 1;
                ok += i64::from(o);
            }
            max_dur = max_dur.max(cf.dur_ms);
            human += usize::from(cf.decided_by_human);
        }
        for m in &matches {
            // A retry is a call of the run that follows a failed one.
            let again = m[i].windows(2).filter(|w| w[0].ok == Some(false)).count();
            retry = retry.max(again);
        }
        let top = |m: &BTreeMap<&str, usize>| {
            m.iter()
                .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                .map(|(k, _)| (*k).to_owned())
                .unwrap_or_default()
        };
        let tool = top(&tools);
        let category = top(&cats);
        if tools.len() > 1 {
            parameters.push(c::ProcedureParameter {
                name: format!("{id}_tool"),
                step: id.clone(),
                kind: "varying".to_owned(),
                values: tools.keys().map(|t| (*t).to_owned()).collect(),
            });
        }
        let kind = if human * 2 > calls.len() {
            c::StepKind::Person
        } else {
            match category.as_str() {
                "Execute" => c::StepKind::Program,
                "Spawn" => c::StepKind::Child,
                _ => c::StepKind::Judgement,
            }
        };
        let shape = match shapes.len() {
            1 => shapes.into_iter().next().flatten().map(str::to_owned),
            _ => None,
        };
        steps.push(c::ProcedureStep {
            id: id.clone(),
            kind,
            token: token.clone(),
            tool,
            effect: effect(&category).to_owned(),
            category,
            shape,
            after: if i == 0 {
                Vec::new()
            } else {
                vec![format!("s{i}")]
            },
            retry_max: retry as i64,
            timeout_seconds: ((max_dur + 59_999) / 60_000).max(1) * 60,
            ok_permille: if done == 0 { 0 } else { ok * 1000 / done },
        });
    }
    let id = pattern_id(tokens);
    Some(c::Procedure {
        format: PROCEDURE_FORMAT.to_owned(),
        description: format!(
            "Mined from {} matches in {} sessions: {}",
            matches.len(),
            sessions.len(),
            tokens.join(" → ")
        ),
        id,
        pattern: tokens.to_vec(),
        matches: matches.len() as i64,
        sessions: sessions.into_iter().collect(),
        parameters,
        steps,
        query: pattern_query(level, tokens),
    })
}
