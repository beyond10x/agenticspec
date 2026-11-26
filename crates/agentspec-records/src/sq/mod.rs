//! sq: a query language over the session IR (`design/query-design.md`). A module of rules,
//! queries and checks is compiled once and run over the fact base of each session; rows are
//! definite or possible, and a check gives one verdict per session.

pub mod eval;
pub mod facts;
pub mod syntax;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agentspec_types::cli as c;
use agentspec_types::session as s;
use rayon::prelude::*;

pub use eval::{Answers, Program};
pub use facts::FactBase;

pub const REPORT_FORMAT: &str = "agentspec-query-report/1";

pub fn compile(src: &str) -> Result<Program, String> {
    eval::compile(&syntax::parse(src)?)
}

/// The fact base of one main transcript and its sub-agents, lifted from the file.
pub fn fact_base(main: &Path) -> std::io::Result<FactBase> {
    fact_base_with(main, &crate::claude::lift)
}

/// The fact base of one main transcript, its lifted commands taken from `load` (the file, or the
/// local store).
pub fn fact_base_with(main: &Path, load: crate::Load<'_>) -> std::io::Result<FactBase> {
    let lifted = load(main)?;
    let replayed = crate::replay::replay(&lifted.steps);
    let project = main
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");
    Ok(facts::build(&lifted, &replayed, project))
}

pub struct SessionAnswers {
    pub session_id: String,
    pub answers: Answers,
}

/// Runs a program over each main transcript, in parallel.
pub fn run_all(p: &Program, mains: &[PathBuf]) -> Vec<SessionAnswers> {
    run_all_with(p, mains, &crate::claude::lift)
}

/// One check's verdict on one session: name, about, severity, verdict, rows and possible rows.
pub struct SessionCheck {
    pub check: String,
    pub about: String,
    pub severity: String,
    pub verdict: s::Verdict,
    pub rows: i64,
    pub possible: i64,
}

/// The verdict of every check of `p` on one session's answers.
pub fn session_checks(p: &Program, answers: &Answers) -> Vec<SessionCheck> {
    p.checks
        .iter()
        .map(|ch| {
            let found = answers.check_rows(ch);
            SessionCheck {
                check: ch.name.clone(),
                about: ch.about.clone(),
                severity: ch.severity.clone(),
                verdict: verdict(ch.expect, found),
                rows: found.rows.len() as i64,
                possible: found.rows.iter().filter(|(_, st)| !st.is_empty()).count() as i64,
            }
        })
        .collect()
}

/// Runs a program over each main transcript, in parallel, its commands taken from `load`.
pub fn run_all_with(p: &Program, mains: &[PathBuf], load: crate::Load<'_>) -> Vec<SessionAnswers> {
    let mut out: Vec<SessionAnswers> = mains
        .par_iter()
        .filter_map(|m| {
            let fb = fact_base_with(m, load).ok()?;
            let answers = eval::run(p, &fb);
            Some(SessionAnswers {
                session_id: fb.session.clone(),
                answers,
            })
        })
        .collect();
    out.sort_by(|a, b| a.session_id.cmp(&b.session_id));
    out
}

fn verdict(expect: syntax::Expect, rows: &eval::Rows) -> s::Verdict {
    let definite = rows.rows.iter().any(|(_, st)| st.is_empty());
    let any = !rows.rows.is_empty();
    match expect {
        syntax::Expect::None if definite => s::Verdict::Gap,
        syntax::Expect::None if any => s::Verdict::Unk,
        syntax::Expect::None => s::Verdict::Ok,
        syntax::Expect::Some if definite => s::Verdict::Ok,
        syntax::Expect::Some if any => s::Verdict::Unk,
        syntax::Expect::Some => s::Verdict::Gap,
    }
}

pub fn report(p: &Program, results: &[SessionAnswers]) -> c::QueryReport {
    let mut rows = Vec::new();
    for (qi, _) in p.queries.iter().enumerate() {
        for r in results {
            let mut qrows: Vec<&(Vec<facts::Val>, Vec<Arc<str>>)> =
                r.answers.queries[qi].rows.iter().collect();
            qrows.sort_by(|a, b| a.0.cmp(&b.0));
            for (vals, sites) in qrows {
                rows.push(c::QueryRow {
                    query: qi as i64,
                    session_id: r.session_id.clone(),
                    values: vals.iter().map(ToString::to_string).collect(),
                    certainty: if sites.is_empty() {
                        c::Certainty::Definite
                    } else {
                        c::Certainty::Possible
                    },
                    unknown: sites.iter().map(|s| s.to_string()).collect(),
                });
            }
        }
    }
    let mut checks = Vec::new();
    let mut verdicts = Vec::new();
    for ch in &p.checks {
        let (mut ok, mut gap, mut unk) = (0, 0, 0);
        for r in results {
            let found = r.answers.check_rows(ch);
            let v = verdict(ch.expect, found);
            match v {
                s::Verdict::Ok => ok += 1,
                s::Verdict::Gap => gap += 1,
                s::Verdict::Unk => unk += 1,
            }
            if v != s::Verdict::Ok {
                verdicts.push(c::CheckVerdict {
                    check: ch.name.clone(),
                    session_id: r.session_id.clone(),
                    verdict: v,
                    rows: found.rows.len() as i64,
                });
            }
        }
        checks.push(c::CheckFigures {
            check: ch.name.clone(),
            about: ch.about.clone(),
            severity: ch.severity.clone(),
            ok,
            gap,
            unk,
        });
    }
    c::QueryReport {
        format: REPORT_FORMAT.to_owned(),
        sessions: results.len() as i64,
        queries: p
            .queries
            .iter()
            .map(|q| format!("{}  [{}]", q.text, q.columns.join(", ")))
            .collect(),
        rows,
        checks,
        verdicts,
    }
}
