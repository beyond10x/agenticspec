//! A trace in port terms (`agenticspec-trace/1`) and the check of a contract's routing invariants
//! on it. A trace lists the ports its producer records and, ordered by time, the begin and the end
//! of each use of a port by an instance of a part; a begin names the use it lies inside and its
//! counterparty. The simulator writes one (`agenticspec sim trace`); a runtime can write the same.
//! `docs/contract.md` states the format and the check.

use super::derive::Contract;
use super::predicate::{Clause, Predicate, Use};
use crate::sim::PortUse;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const FORMAT: &str = "agenticspec-trace/1";

/// The begin or the end of a use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Begin,
    End,
}

/// One event of a trace.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// Its place in the trace, from 0.
    pub index: usize,
    /// Seconds from the start of the run.
    pub time: f64,
    pub part: String,
    pub instance: u64,
    pub port: String,
    pub phase: Phase,
    /// On a begin: the index of the begin of the use this one lies inside.
    pub within: Option<usize>,
    /// On an end: the index of the begin it ends.
    pub begin: Option<usize>,
    /// On a begin: the part on the other side of the use.
    pub to: Option<String>,
    /// On a begin: the channel part the use is carried over.
    pub over: Option<String>,
}

/// A trace: the architecture it is of, what wrote it, the ports it records, and its events.
#[derive(Debug, Clone, PartialEq)]
pub struct Trace {
    pub architecture: String,
    /// What wrote the trace, as the producer describes itself.
    pub source: Value,
    /// The ports whose every use is in the trace.
    pub records: Vec<String>,
    pub events: Vec<Event>,
}

impl Trace {
    /// The trace of a recorded run: a begin per use and an end per use whose end is known, by
    /// time; at one moment ends of earlier uses come first, then begins (an enclosing use before
    /// the uses inside it), then ends of uses of no duration, innermost first.
    pub fn from_uses(
        architecture: &str,
        source: Value,
        records: &[&str],
        uses: &[PortUse],
    ) -> Trace {
        // (time, rank, order key, use, phase)
        let mut order: Vec<(f64, u8, i64, usize, Phase)> = Vec::new();
        for (n, u) in uses.iter().enumerate() {
            order.push((u.begin, 1, n as i64, n, Phase::Begin));
            if u.end.is_finite() {
                let rank = if u.end > u.begin { 0 } else { 2 };
                order.push((u.end, rank, -(n as i64), n, Phase::End));
            }
        }
        order.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut begin_at = vec![0usize; uses.len()];
        for (index, (_, _, _, n, phase)) in order.iter().enumerate() {
            if *phase == Phase::Begin {
                begin_at[*n] = index;
            }
        }
        let events = order
            .iter()
            .enumerate()
            .map(|(index, (time, _, _, n, phase))| {
                let u = &uses[*n];
                let begin = *phase == Phase::Begin;
                Event {
                    index,
                    time: *time,
                    part: u.part.clone(),
                    instance: u.instance as u64,
                    port: u.port.to_owned(),
                    phase: *phase,
                    within: if begin {
                        u.within.map(|w| begin_at[w])
                    } else {
                        None
                    },
                    begin: (!begin).then_some(begin_at[*n]),
                    to: if begin { u.to.clone() } else { None },
                    over: if begin { u.over.clone() } else { None },
                }
            })
            .collect();
        Trace {
            architecture: architecture.to_owned(),
            source,
            records: records.iter().map(|r| (*r).to_owned()).collect(),
            events,
        }
    }

    /// The trace as written: JSON with one event per line, the same bytes for the same trace.
    pub fn text(&self) -> String {
        let mut s = String::from("{\n");
        s.push_str(&format!("  \"format\": {},\n", json!(FORMAT)));
        s.push_str(&format!(
            "  \"architecture\": {},\n",
            json!(self.architecture)
        ));
        s.push_str(&format!("  \"source\": {},\n", self.source));
        s.push_str(&format!("  \"records\": {},\n", json!(self.records)));
        if self.events.is_empty() {
            s.push_str("  \"events\": []\n}\n");
            return s;
        }
        s.push_str("  \"events\": [\n");
        for (n, e) in self.events.iter().enumerate() {
            let mut v = json!({
                "index": e.index,
                "time": (e.time * 1000.0).round() / 1000.0,
                "part": e.part,
                "instance": e.instance,
                "port": e.port,
                "phase": match e.phase { Phase::Begin => "begin", Phase::End => "end" },
            });
            for (k, x) in [("within", e.within), ("begin", e.begin)] {
                if let Some(x) = x {
                    v[k] = json!(x);
                }
            }
            for (k, x) in [("to", &e.to), ("over", &e.over)] {
                if let Some(x) = x {
                    v[k] = json!(x);
                }
            }
            let comma = if n + 1 < self.events.len() { "," } else { "" };
            s.push_str(&format!("    {v}{comma}\n"));
        }
        s.push_str("  ]\n}\n");
        s
    }

    /// Read a trace and check its shape: indices in order from 0, an end ends an earlier begin of
    /// the same port, part and instance, at most once and not before it, and a use lies inside
    /// an earlier begin.
    pub fn parse(text: &str) -> Result<Trace, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if v["format"] != FORMAT {
            return Err(format!("not {FORMAT}"));
        }
        let field = |e: &Value, k: &str| -> Option<String> { e[k].as_str().map(str::to_owned) };
        let records = v["records"]
            .as_array()
            .ok_or("no `records`")?
            .iter()
            .map(|r| {
                r.as_str()
                    .map(str::to_owned)
                    .ok_or("`records` holds a non-string")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut events: Vec<Event> = Vec::new();
        let mut ended = BTreeSet::new();
        for (n, e) in v["events"]
            .as_array()
            .ok_or("no `events`")?
            .iter()
            .enumerate()
        {
            let at = |what: &str| format!("event {n}: {what}");
            if e["index"].as_u64() != Some(n as u64) {
                return Err(at(&format!("index is {}, not {n}", e["index"])));
            }
            let phase = match e["phase"].as_str() {
                Some("begin") => Phase::Begin,
                Some("end") => Phase::End,
                _ => return Err(at("phase is neither `begin` nor `end`")),
            };
            let event = Event {
                index: n,
                time: e["time"].as_f64().ok_or_else(|| at("no `time`"))?,
                part: field(e, "part").ok_or_else(|| at("no `part`"))?,
                instance: e["instance"].as_u64().ok_or_else(|| at("no `instance`"))?,
                port: field(e, "port").ok_or_else(|| at("no `port`"))?,
                phase,
                within: e["within"].as_u64().map(|x| x as usize),
                begin: e["begin"].as_u64().map(|x| x as usize),
                to: field(e, "to"),
                over: field(e, "over"),
            };
            let earlier_begin = |i: usize| events.get(i).filter(|b| b.phase == Phase::Begin);
            match phase {
                Phase::Begin => {
                    if let Some(w) = event.within
                        && earlier_begin(w).is_none()
                    {
                        return Err(at(&format!("`within` {w} is no earlier begin")));
                    }
                }
                Phase::End => {
                    let b = event.begin.ok_or_else(|| at("an end names no `begin`"))?;
                    let Some(start) = earlier_begin(b) else {
                        return Err(at(&format!("`begin` {b} is no earlier begin")));
                    };
                    if (&start.port, &start.part, start.instance)
                        != (&event.port, &event.part, event.instance)
                    {
                        return Err(at(&format!(
                            "ends begin {b} of another use ({}@{} instance {})",
                            start.port, start.part, start.instance
                        )));
                    }
                    if !ended.insert(b) {
                        return Err(at(&format!("begin {b} is ended twice")));
                    }
                }
            }
            events.push(event);
        }
        Ok(Trace {
            architecture: field(&v, "architecture").ok_or("no `architecture`")?,
            source: v["source"].clone(),
            records,
            events,
        })
    }

    /// The end of each begin, by the begin's index.
    fn ends(&self) -> Vec<Option<&Event>> {
        let mut out = vec![None; self.events.len()];
        for e in &self.events {
            if let Some(b) = e.begin {
                out[b] = Some(e);
            }
        }
        out
    }
}

/// What the check found for one invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Held,
    Violated,
    /// A port the predicate names is not recorded by the trace: absent uses prove nothing.
    NotChecked,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Held => "held",
            Status::Violated => "violated",
            Status::NotChecked => "not_checked",
        }
    }
}

/// The check of one invariant on a trace.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub link: String,
    pub predicate: String,
    pub status: Status,
    /// The uses of the predicate's ports the check read.
    pub uses: usize,
    /// The index of the event of the first violation.
    pub event: Option<usize>,
    /// What was violated, or why the invariant was not checked.
    pub detail: Option<String>,
}

impl Check {
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "link": self.link,
            "predicate": self.predicate,
            "status": self.status.as_str(),
            "uses": self.uses,
        });
        if let Some(e) = self.event {
            v["event"] = json!(e);
        }
        if let Some(d) = &self.detail {
            v["detail"] = json!(d);
        }
        v
    }
}

/// The outcome of one clause: the uses it read, and its first violation (event, text).
type ClauseOutcome = (usize, Option<(usize, String)>);

fn uses_of<'t>(trace: &'t Trace, u: &'t Use) -> impl Iterator<Item = &'t Event> + 't {
    trace
        .events
        .iter()
        .filter(move |e| e.phase == Phase::Begin && e.port == u.port && e.part == u.part)
}

fn end_time(end: Option<&Event>) -> f64 {
    end.map_or(f64::INFINITY, |e| e.time)
}

fn inside(trace: &Trace, ends: &[Option<&Event>], every: &Use, within: &Use) -> ClauseOutcome {
    let mut n = 0;
    for e in uses_of(trace, every) {
        n += 1;
        let mut at = e.within;
        let outer = loop {
            match at.map(|w| &trace.events[w]) {
                None => break None,
                Some(w) if w.port == within.port && w.part == within.part => break Some(w),
                Some(w) => at = w.within,
            }
        };
        let Some(w) = outer else {
            return (
                n,
                Some((e.index, format!("lies inside no use of {within}"))),
            );
        };
        if e.time < w.time {
            return (
                n,
                Some((
                    e.index,
                    format!(
                        "begins at {} s, before the use of {within} it lies inside (event {}) begins at {} s",
                        e.time, w.index, w.time
                    ),
                )),
            );
        }
        let (inner_end, outer_end) = (ends[e.index], ends[w.index]);
        if end_time(inner_end) > end_time(outer_end) {
            let index = inner_end.map_or(e.index, |x| x.index);
            return (
                n,
                Some((
                    index,
                    format!(
                        "ends after the use of {within} it lies inside (event {}) ends; the use begins at event {}",
                        w.index, e.index
                    ),
                )),
            );
        }
    }
    (n, None)
}

fn to(trace: &Trace, every: &Use, to: &str, over: Option<&str>) -> ClauseOutcome {
    let mut n = 0;
    for e in uses_of(trace, every) {
        n += 1;
        if e.to.as_deref() != Some(to) {
            let found =
                e.to.as_deref()
                    .map_or("no counterparty".to_owned(), |t| format!("`{t}`"));
            return (n, Some((e.index, format!("goes to {found}, not `{to}`"))));
        }
        if let Some(c) = over
            && e.over.as_deref() != Some(c)
        {
            let found = e
                .over
                .as_deref()
                .map_or("no channel".to_owned(), |t| format!("`{t}`"));
            return (
                n,
                Some((e.index, format!("is carried over {found}, not `{c}`"))),
            );
        }
    }
    (n, None)
}

/// Check every invariant of the contract on the trace, in the contract's order. Refused when the
/// trace is of another architecture.
pub fn check(contract: &Contract, trace: &Trace) -> Result<Vec<Check>, String> {
    if trace.architecture != contract.architecture {
        return Err(format!(
            "the trace is of architecture `{}`, the contract of `{}`",
            trace.architecture, contract.architecture
        ));
    }
    let ends = trace.ends();
    let recorded = |port: &str| trace.records.iter().any(|r| r == port);
    let mut out = Vec::new();
    for inv in &contract.invariants {
        let mut c = Check {
            link: inv.link.clone(),
            predicate: inv.predicate.clone(),
            status: Status::Held,
            uses: 0,
            event: None,
            detail: None,
        };
        let predicate = match Predicate::parse(&inv.predicate) {
            Ok(p) => p,
            Err(e) => {
                c.status = Status::NotChecked;
                c.detail = Some(format!("the predicate does not parse: {e}"));
                out.push(c);
                continue;
            }
        };
        let mut unrecorded = Vec::new();
        for clause in &predicate.clauses {
            let ports: Vec<&str> = match clause {
                Clause::Inside { every, within } => vec![&every.port, &within.port],
                Clause::To { every, .. } => vec![&every.port],
            };
            let missing: Vec<&str> = ports.into_iter().filter(|p| !recorded(p)).collect();
            if !missing.is_empty() {
                unrecorded.extend(missing);
                continue;
            }
            let (n, violation) = match clause {
                Clause::Inside { every, within } => inside(trace, &ends, every, within),
                Clause::To { every, to: t, over } => to(trace, every, t, over.as_deref()),
            };
            c.uses += n;
            if let Some((event, text)) = violation
                && c.status != Status::Violated
            {
                c.status = Status::Violated;
                c.event = Some(event);
                c.detail = Some(format!("{}: event {event} {text}", clause.every()));
            }
        }
        if c.status == Status::Held && !unrecorded.is_empty() {
            unrecorded.sort_unstable();
            unrecorded.dedup();
            c.status = Status::NotChecked;
            c.detail = Some(format!(
                "the trace does not record {}",
                unrecorded.join(", ")
            ));
        }
        out.push(c);
    }
    Ok(out)
}
