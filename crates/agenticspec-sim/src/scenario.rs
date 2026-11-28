//! Scenarios (`agenticspec-scenario/1`): inputs at the border of an architecture, fixtures for
//! its external parts and the events expected in block terms. A scenario is checked against its
//! architecture, run on the simulator with a virtual clock and its own seed, and its expectations
//! are evaluated on the trace of block events the run records (`agenticspec-scenario-result/1`).

use crate::check::{block, check as check_architecture};
use crate::cli::Refusal;
use crate::corpus::Class;
use crate::doc::{load_yaml, parse_yaml};
use crate::events::{self, REFUSAL};
use crate::marked::resolve;
use crate::profile::Profile;
use crate::reader::{DocRef, Reader, sha256_hex};
use crate::rows::{Row, blocks, check_row, read_row};
use crate::search::kind as kpi_kind;
use crate::sim::{Arrival, Model, RUN_KPIS, RunOut, RunOutcome, Script, TaskScript, simulate};
use crate::unavailable::{self, Mode, PartThen, Plan, Unavailable};
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "agenticspec-scenario/1";
pub const RESULT_FORMAT: &str = "agenticspec-scenario-result/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Prompt,
    Order,
    Message,
    Control,
}

pub const INPUT_KINDS: &[(&str, InputKind)] = &[
    ("prompt", InputKind::Prompt),
    ("order", InputKind::Order),
    ("message", InputKind::Message),
    ("control", InputKind::Control),
];

impl InputKind {
    pub fn name(self) -> &'static str {
        INPUT_KINDS
            .iter()
            .find(|(_, k)| *k == self)
            .map_or("", |(n, _)| n)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Send,
    Pick,
    Run,
    Retry,
    Dispatch,
    Approve,
    Reject,
}

pub const ACTS: &[(&str, Act)] = &[
    ("send", Act::Send),
    ("pick", Act::Pick),
    ("run", Act::Run),
    ("retry", Act::Retry),
    ("dispatch", Act::Dispatch),
    ("approve", Act::Approve),
    ("reject", Act::Reject),
];

impl Act {
    pub fn name(self) -> &'static str {
        ACTS.iter().find(|(_, a)| *a == self).map_or("", |(n, _)| n)
    }
}

/// One input at a border part.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub at: String,
    pub kind: InputKind,
    pub author: String,
    pub proof: Option<String>,
    pub text: Option<String>,
    pub act: Act,
    /// Seconds after the input before it (the first: after the start).
    pub after: f64,
}

impl Input {
    /// Whether the input, once admitted, makes a task.
    pub fn makes_task(&self) -> bool {
        matches!(
            (self.kind, self.act),
            (InputKind::Prompt, Act::Send)
                | (InputKind::Order, Act::Send | Act::Dispatch)
                | (InputKind::Control, Act::Run | Act::Retry | Act::Dispatch)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GivenTask {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GivenQuestion {
    pub id: String,
    pub asked_by: String,
    pub addressee: String,
    pub options: Vec<String>,
    pub default: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Given {
    pub tasks: Vec<GivenTask>,
    pub questions: Vec<GivenQuestion>,
    /// Rows of block entities, checked against the blocks; the simulator does not read them.
    pub rows: Vec<Row>,
}

pub const OUTCOMES: &[(&str, RunOutcome)] = &[
    ("correct", RunOutcome::Correct),
    ("wrong", RunOutcome::Wrong),
    ("false_completion", RunOutcome::FalseCompletion),
    ("question", RunOutcome::Question),
];

/// A model's answer to one run: how the run ends.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelAnswer {
    pub part: Option<String>,
    pub outcome: RunOutcome,
    pub turns: Option<usize>,
    pub builds: usize,
    pub options: Vec<String>,
    pub default: Option<String>,
    pub needs_owner: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    pub tool: String,
    pub seconds: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub part: String,
    pub option: Option<String>,
    pub after: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fixtures {
    pub model: Vec<ModelAnswer>,
    pub tools: Vec<ToolResult>,
    pub principals: Vec<Reply>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Sequence,
    Set,
}

/// One expected event, by its full block name (`agentic.work.RunStarted`), or one expected
/// refusal: the block command refused (`agentic.resource.RequestClaim`), its block error, the
/// rule of the run that refused it, or any of them.
#[derive(Debug, Clone, PartialEq)]
pub struct Expect {
    pub event: Option<String>,
    pub refusal: Option<String>,
    pub error: Option<String>,
    pub part: Option<String>,
    pub task: Option<String>,
    pub rule: Option<String>,
    pub options: Option<Vec<String>>,
    pub option: Option<String>,
    pub channel: Option<String>,
    pub author: Option<String>,
    pub process: Option<String>,
    pub absent: bool,
    /// The expectation as written.
    pub written: Value,
}

impl Expect {
    /// Whether it expects a refusal rather than an event.
    pub fn is_refusal(&self) -> bool {
        self.event.is_none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bound {
    pub kpi: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Then {
    pub order: Order,
    pub events: Vec<Expect>,
    pub kpis: Vec<Bound>,
    /// What each named part shows while something is unavailable.
    pub parts: Vec<PartThen>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub name: String,
    pub summary: Option<String>,
    pub architecture: DocRef,
    pub profile: DocRef,
    pub seed: u64,
    pub horizon: f64,
    pub given: Given,
    pub unavailable: Vec<Unavailable>,
    pub when: Vec<Input>,
    pub fixtures: Fixtures,
    pub then: Then,
}

const TOP: &[&str] = &[
    "format",
    "scenario",
    "summary",
    "architecture",
    "profile",
    "seed",
    "horizon",
    "given",
    "unavailable",
    "when",
    "fixtures",
    "then",
];
const EXPECT_KEYS: &[&str] = &[
    "event", "refusal", "error", "part", "task", "rule", "options", "option", "channel", "author",
    "process", "absent",
];

/// Reads a scenario document; every problem is named by its path.
pub fn parse(doc: &Value) -> Result<Scenario, Vec<String>> {
    let mut r = Reader::default();
    let Some(m) = r.obj(
        doc,
        "",
        TOP,
        &["format", "scenario", "architecture", "profile", "when"],
    ) else {
        return Err(r.errors);
    };
    r.format(m, FORMAT);
    let name = r.text(m, "scenario", "");
    let summary = r.text(m, "summary", "");
    let architecture = r.doc_ref(m, "architecture", "");
    let profile = r.doc_ref(m, "profile", "");
    let seed = r.uint(m, "seed", "").unwrap_or(0);
    let horizon = r.duration(m, "horizon", "").unwrap_or(4.0 * 3600.0);
    if horizon <= 0.0 {
        r.err("horizon", "must be longer than 0s");
    }
    let given = read_given(&mut r, m.get("given"));
    let unavailable = unavailable::read_unavailable(&mut r, m.get("unavailable"));
    let mut when = Vec::new();
    for (i, v) in r.list(m, "when", "").iter().enumerate() {
        if let Some(input) = read_input(&mut r, v, &format!("when.{i}")) {
            when.push(input);
        }
    }
    let fixtures = read_fixtures(&mut r, m.get("fixtures"));
    let then = read_then(&mut r, m.get("then"));
    match (name, architecture, profile) {
        (Some(name), Some(architecture), Some(profile)) if r.errors.is_empty() => Ok(Scenario {
            name,
            summary,
            architecture,
            profile,
            seed,
            horizon,
            given,
            unavailable,
            when,
            fixtures,
            then,
        }),
        _ => Err(r.errors),
    }
}

fn read_given(r: &mut Reader, v: Option<&Value>) -> Given {
    let mut g = Given::default();
    let Some(v) = v.filter(|v| !v.is_null()) else {
        return g;
    };
    let Some(m) = r.obj(v, "given", &["tasks", "questions", "rows"], &[]) else {
        return g;
    };
    for (i, t) in r.list(m, "tasks", "given").iter().enumerate() {
        let p = format!("given.tasks.{i}");
        if let Some(o) = r.obj(t, &p, &["id"], &["id"])
            && let Some(id) = r.text(o, "id", &p)
        {
            g.tasks.push(GivenTask { id });
        }
    }
    for (i, q) in r.list(m, "questions", "given").iter().enumerate() {
        let p = format!("given.questions.{i}");
        let keys = ["id", "asked_by", "addressee", "options", "default"];
        let Some(o) = r.obj(q, &p, &keys, &keys) else {
            continue;
        };
        let (id, asked_by, addressee) = (
            r.text(o, "id", &p),
            r.text(o, "asked_by", &p),
            r.text(o, "addressee", &p),
        );
        let options = r.texts(o, "options", &p).unwrap_or_default();
        let default = r.text(o, "default", &p);
        if options.is_empty() {
            r.err(
                &format!("{p}.options"),
                "a question offers at least one option",
            );
        }
        if let Some(d) = &default
            && !options.contains(d)
        {
            r.err(
                &format!("{p}.default"),
                format!("`{d}` is not one of the options"),
            );
        }
        if let (Some(id), Some(asked_by), Some(addressee), Some(default)) =
            (id, asked_by, addressee, default)
        {
            g.questions.push(GivenQuestion {
                id,
                asked_by,
                addressee,
                options,
                default,
            });
        }
    }
    for (i, row) in r.list(m, "rows", "given").iter().enumerate() {
        if let Some(row) = read_row(r, row, &format!("given.rows.{i}")) {
            g.rows.push(row);
        }
    }
    g
}

fn read_input(r: &mut Reader, v: &Value, p: &str) -> Option<Input> {
    let keys = ["at", "kind", "author", "proof", "text", "act", "after"];
    let o = r.obj(v, p, &keys, &["at", "kind", "author"])?;
    let at = r.text(o, "at", p);
    let kind = r.one_of(o, "kind", p, INPUT_KINDS);
    let author = r.text(o, "author", p);
    let proof = r.text(o, "proof", p);
    let text = o.get("text").and_then(|t| match t {
        Value::String(s) => Some(s.clone()),
        _ => {
            r.err(&format!("{p}.text"), "expected a string");
            None
        }
    });
    let act = r.one_of(o, "act", p, ACTS);
    let after = r.duration(o, "after", p).unwrap_or(0.0);
    let kind = kind?;
    let act = match (act, kind) {
        (Some(a), _) => a,
        (None, InputKind::Order) => Act::Dispatch,
        (None, InputKind::Control) => {
            r.err(&format!("{p}.act"), "a control input names its act");
            return None;
        }
        (None, _) => Act::Send,
    };
    Some(Input {
        at: at?,
        kind,
        author: author?,
        proof,
        text,
        act,
        after,
    })
}

fn read_fixtures(r: &mut Reader, v: Option<&Value>) -> Fixtures {
    let mut f = Fixtures::default();
    let Some(v) = v.filter(|v| !v.is_null()) else {
        return f;
    };
    let Some(m) = r.obj(v, "fixtures", &["model", "tools", "principals"], &[]) else {
        return f;
    };
    for (i, a) in r.list(m, "model", "fixtures").iter().enumerate() {
        let p = format!("fixtures.model.{i}");
        let keys = [
            "for",
            "outcome",
            "turns",
            "builds",
            "options",
            "default",
            "needs_owner",
        ];
        let Some(o) = r.obj(a, &p, &keys, &["outcome"]) else {
            continue;
        };
        let part = r.text(o, "for", &p);
        let outcome = r.one_of(o, "outcome", &p, OUTCOMES);
        let turns = r.uint(o, "turns", &p).map(|n| n as usize);
        if turns == Some(0) {
            r.err(&format!("{p}.turns"), "a run makes at least one model call");
        }
        let builds = r.uint(o, "builds", &p).unwrap_or(0) as usize;
        let options = r.texts(o, "options", &p).unwrap_or_default();
        let default = r.text(o, "default", &p);
        let needs_owner = r.boolean(o, "needs_owner", &p).unwrap_or(false);
        if outcome != Some(RunOutcome::Question)
            && (!options.is_empty() || default.is_some() || o.contains_key("needs_owner"))
        {
            r.err(
                &p,
                "options, default and needs_owner belong to the outcome question",
            );
        }
        if let Some(d) = &default
            && !options.contains(d)
        {
            r.err(
                &format!("{p}.default"),
                format!("`{d}` is not one of the options"),
            );
        }
        if let Some(outcome) = outcome {
            f.model.push(ModelAnswer {
                part,
                outcome,
                turns,
                builds,
                options,
                default,
                needs_owner,
            });
        }
    }
    for (i, t) in r.list(m, "tools", "fixtures").iter().enumerate() {
        let p = format!("fixtures.tools.{i}");
        let Some(o) = r.obj(t, &p, &["tool", "seconds"], &["tool", "seconds"]) else {
            continue;
        };
        let tool = r.one_of(o, "tool", &p, &[("build", "build")]);
        let seconds = r.duration(o, "seconds", &p);
        if let (Some(tool), Some(seconds)) = (tool, seconds) {
            f.tools.push(ToolResult {
                tool: tool.to_string(),
                seconds,
            });
        }
    }
    for (i, a) in r.list(m, "principals", "fixtures").iter().enumerate() {
        let p = format!("fixtures.principals.{i}");
        let Some(o) = r.obj(a, &p, &["part", "option", "after"], &["part", "after"]) else {
            continue;
        };
        let part = r.text(o, "part", &p);
        let option = r.text(o, "option", &p);
        let after = r.duration(o, "after", &p);
        if let (Some(part), Some(after)) = (part, after) {
            f.principals.push(Reply {
                part,
                option,
                after,
            });
        }
    }
    f
}

fn read_then(r: &mut Reader, v: Option<&Value>) -> Then {
    let mut t = Then {
        order: Order::Sequence,
        events: Vec::new(),
        kpis: Vec::new(),
        parts: Vec::new(),
    };
    let Some(v) = v.filter(|v| !v.is_null()) else {
        return t;
    };
    let Some(m) = r.obj(v, "then", &["order", "events", "kpis", "parts"], &[]) else {
        return t;
    };
    t.parts = unavailable::read_parts(r, m.get("parts"));
    t.order = r
        .one_of(
            m,
            "order",
            "then",
            &[("sequence", Order::Sequence), ("set", Order::Set)],
        )
        .unwrap_or(Order::Sequence);
    for (i, e) in r.list(m, "events", "then").iter().enumerate() {
        let p = format!("then.events.{i}");
        let Some(o) = r.obj(e, &p, EXPECT_KEYS, &[]) else {
            continue;
        };
        let event = r.text(o, "event", &p);
        let refusal = r.text(o, "refusal", &p);
        let error = r.text(o, "error", &p);
        let rule = r.text(o, "rule", &p);
        let of_refusal = ["refusal", "error", "rule"]
            .into_iter()
            .filter(|k| o.contains_key(*k))
            .collect::<Vec<_>>();
        match (o.contains_key("event"), of_refusal.is_empty()) {
            (true, false) => r.err(
                &p,
                format!(
                    "an expected event has no {}: a refusal is expected by `refusal`, `error` or `rule` alone",
                    of_refusal.join(", ")
                ),
            ),
            (false, true) => r.err(
                &p,
                "an expectation names an `event`, or a refusal by `refusal`, `error` or `rule`",
            ),
            _ => {}
        }
        if let Some(rl) = &rule
            && !events::RULES.iter().any(|(n, _, _, _)| n == rl)
            && rl != unavailable::RULE
        {
            let mut names: Vec<&str> = events::RULES.iter().map(|(n, _, _, _)| *n).collect();
            names.push(unavailable::RULE);
            r.err(
                &format!("{p}.rule"),
                format!("`{rl}` is no rule; expected one of {}", names.join(", ")),
            );
        }
        let expect = Expect {
            event,
            refusal,
            error,
            part: r.text(o, "part", &p),
            task: r.text(o, "task", &p),
            rule,
            options: r.texts(o, "options", &p),
            option: r.text(o, "option", &p),
            channel: r.text(o, "channel", &p),
            author: r.text(o, "author", &p),
            process: r.text(o, "process", &p),
            absent: r.boolean(o, "absent", &p).unwrap_or(false),
            written: e.clone(),
        };
        t.events.push(expect);
    }
    if let Some(k) = m.get("kpis")
        && let Some(o) = r.obj(k, "then.kpis", RUN_KPIS, &[])
    {
        for (name, b) in o {
            let p = format!("then.kpis.{name}");
            let Some(bo) = r.obj(b, &p, &["min", "max"], &[]) else {
                continue;
            };
            let min = r.number(bo, "min", &p);
            let max = r.number(bo, "max", &p);
            if min.is_none() && max.is_none() {
                r.err(&p, "a bound names min, max or both");
            }
            t.kpis.push(Bound {
                kpi: name.clone(),
                min,
                max,
            });
        }
    }
    t
}

/// A scenario with the documents it names, read relative to it.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub path: PathBuf,
    pub scenario: Scenario,
    /// The architecture as written, markers included.
    pub architecture: Value,
    pub architecture_path: PathBuf,
    pub architecture_digest: String,
    pub profile: Profile,
    pub profile_path: PathBuf,
    pub profile_digest: String,
}

fn refused(code: &'static str, detail: impl Into<String>) -> Refusal {
    Refusal {
        code,
        detail: detail.into(),
    }
}

/// Reads a scenario file and the architecture and profile it names.
pub fn load(path: &Path) -> Result<Loaded, Refusal> {
    let doc = load_yaml(path).map_err(|e| refused("document_unreadable", e))?;
    let scenario = parse(&doc).map_err(|e| {
        refused(
            "scenario_invalid",
            format!("{}: {}", path.display(), e.join("; ")),
        )
    })?;
    load_with(path, scenario)
}

/// Reads the architecture and profile `scenario` names, relative to the directory of `path`.
pub fn load_with(path: &Path, scenario: Scenario) -> Result<Loaded, Refusal> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let read = |r: &DocRef| -> Result<(PathBuf, Vec<u8>), Refusal> {
        let p = dir.join(&r.reference);
        let bytes = std::fs::read(&p)
            .map_err(|e| refused("document_unreadable", format!("{}: {e}", p.display())))?;
        Ok((p, bytes))
    };
    let (architecture_path, abytes) = read(&scenario.architecture)?;
    let (profile_path, pbytes) = read(&scenario.profile)?;
    let text = |p: &Path, b: &[u8]| {
        String::from_utf8(b.to_vec())
            .map_err(|e| refused("document_unreadable", format!("{}: {e}", p.display())))
    };
    let architecture = parse_yaml(&text(&architecture_path, &abytes)?).map_err(|e| {
        refused(
            "document_unreadable",
            format!("{}: {e}", architecture_path.display()),
        )
    })?;
    let pdoc = parse_yaml(&text(&profile_path, &pbytes)?).map_err(|e| {
        refused(
            "document_unreadable",
            format!("{}: {e}", profile_path.display()),
        )
    })?;
    let mut profile =
        Profile::parse(&pdoc).map_err(|e| refused("scenario_refused", format!("profile: {e}")))?;
    profile
        .read_corpus(profile_path.parent().unwrap_or(Path::new(".")))
        .map_err(|e| refused("scenario_refused", format!("profile: {e}")))?;
    Ok(Loaded {
        path: path.to_path_buf(),
        scenario,
        architecture,
        architecture_path,
        architecture_digest: sha256_hex(&abytes),
        profile,
        profile_path,
        profile_digest: sha256_hex(&pbytes),
    })
}

fn parts(arch: &Value) -> Vec<&str> {
    arch["parts"]
        .as_object()
        .map(|o| o.keys().map(String::as_str).collect())
        .unwrap_or_default()
}

fn border(arch: &Value) -> Vec<&str> {
    match &arch["border"] {
        Value::String(s) => vec![s.as_str()],
        Value::Array(xs) => xs.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

/// The code of a problem that has no code of its own.
pub const REFUSED: &str = "scenario_refused";

/// The codes a problem of [`problems`] can have, besides [`REFUSED`]: each is also the code of the
/// refusal of a scenario whose problems all have it.
pub const CODES: &[(&str, &str)] = &[
    (
        "architecture_digest_mismatch",
        "`architecture.digest` is not the sha256 of the architecture file",
    ),
    (
        "profile_digest_mismatch",
        "`profile.digest` is not the sha256 of the profile file",
    ),
    ("architecture_refused", "the architecture itself is refused"),
    (
        "input_not_at_border",
        "an input is at a part the architecture's `border` does not list",
    ),
    (
        "invalid_row",
        "a `given` row does not fit the entity the blocks declare",
    ),
    (
        "fixture_without_part",
        "a fixture names a part the architecture does not have",
    ),
    (
        "unknown_event",
        "an expected `event` is no event the blocks declare",
    ),
    (
        "unknown_refusal",
        "an expected `refusal` is no command the blocks declare, or its `error` is no error that command is refused with",
    ),
];

/// One problem of a scenario against its architecture and profile, with its code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub code: &'static str,
    pub text: String,
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.code == REFUSED {
            f.write_str(&self.text)
        } else {
            write!(f, "{}: {}", self.code, self.text)
        }
    }
}

/// Every problem of a scenario against its architecture and profile, as text.
pub fn check(l: &Loaded) -> Vec<String> {
    problems(l).iter().map(Problem::to_string).collect()
}

/// The refusal of a scenario with `problems`: the code they share, or [`REFUSED`] when their codes
/// differ, and every problem.
pub fn refusal(path: &Path, problems: &[Problem]) -> Refusal {
    let code = match problems.first() {
        Some(p) if problems.iter().all(|q| q.code == p.code) => p.code,
        _ => REFUSED,
    };
    let text: Vec<String> = problems.iter().map(Problem::to_string).collect();
    refused(code, format!("{}: {}", path.display(), text.join("; ")))
}

/// Every problem of a scenario against its architecture and profile: a digest that does not
/// match, an architecture the check refuses, an input at a part outside the border, an author
/// or a fixture's part that is no part of the architecture, a row that does not fit its entity,
/// an expected event or refusal the blocks do not declare.
pub fn problems(l: &Loaded) -> Vec<Problem> {
    let s = &l.scenario;
    let a = &l.architecture;
    let mut out: Vec<Problem> = Vec::new();
    let mut push = |code: &'static str, text: String| out.push(Problem { code, text });
    for (key, code, given, actual, path) in [
        (
            "architecture",
            "architecture_digest_mismatch",
            &s.architecture.digest,
            &l.architecture_digest,
            &l.architecture_path,
        ),
        (
            "profile",
            "profile_digest_mismatch",
            &s.profile.digest,
            &l.profile_digest,
            &l.profile_path,
        ),
    ] {
        if given != actual {
            push(
                code,
                format!(
                    "{key}.digest: {given} is not the sha256 of {}, {actual}",
                    path.display()
                ),
            );
        }
    }
    for refusal in check_architecture(a) {
        push("architecture_refused", format!("architecture: {refusal}"));
    }
    let names = parts(a);
    let edge = border(a);
    let is_part = |n: &str| names.contains(&n);
    for (i, input) in s.when.iter().enumerate() {
        if !edge.contains(&input.at.as_str()) {
            push(
                "input_not_at_border",
                format!(
                    "when.{i}.at: `{}` is not a border part (border: {})",
                    input.at,
                    if edge.is_empty() {
                        "none".to_string()
                    } else {
                        edge.join(", ")
                    }
                ),
            );
        }
        if !is_part(&input.author) {
            push(
                REFUSED,
                format!(
                    "when.{i}.author: `{}` is no part of the architecture",
                    input.author
                ),
            );
        }
    }
    let mut ids = BTreeSet::new();
    for (i, t) in s.given.tasks.iter().enumerate() {
        if !ids.insert(t.id.clone()) {
            push(
                REFUSED,
                format!("given.tasks.{i}.id: `{}` is given twice", t.id),
            );
        }
    }
    for (i, q) in s.given.questions.iter().enumerate() {
        if !ids.insert(q.id.clone()) {
            push(
                REFUSED,
                format!("given.questions.{i}.id: `{}` is given twice", q.id),
            );
        }
        if block(a, &q.asked_by) != Some("agent") {
            push(
                REFUSED,
                format!(
                    "given.questions.{i}.asked_by: `{}` is no agent part",
                    q.asked_by
                ),
            );
        }
        if !matches!(block(a, &q.addressee), Some("agent" | "principal")) {
            push(
                REFUSED,
                format!(
                    "given.questions.{i}.addressee: `{}` is no agent or principal part",
                    q.addressee
                ),
            );
        }
    }
    let mut rows = BTreeSet::new();
    for (i, row) in s.given.rows.iter().enumerate() {
        let p = format!("given.rows.{i}");
        if !rows.insert((row.entity.as_str(), row.id.as_str())) {
            push(
                "invalid_row",
                format!("{p}.id: {} `{}` is given twice", row.entity, row.id),
            );
        }
        for problem in check_row(blocks(), row, &p) {
            push("invalid_row", problem);
        }
    }
    for (i, m) in s.fixtures.model.iter().enumerate() {
        if let Some(p) = &m.part {
            match block(a, p) {
                None => push(
                    "fixture_without_part",
                    format!("fixtures.model.{i}.for: `{p}` is no part of the architecture"),
                ),
                Some("agent") => {}
                Some(_) => push(
                    REFUSED,
                    format!("fixtures.model.{i}.for: `{p}` is no agent part"),
                ),
            }
        }
    }
    for (i, rp) in s.fixtures.principals.iter().enumerate() {
        match block(a, &rp.part) {
            None => push(
                "fixture_without_part",
                format!(
                    "fixtures.principals.{i}.part: `{}` is no part of the architecture",
                    rp.part
                ),
            ),
            Some("principal") => {}
            Some(_) => push(
                REFUSED,
                format!(
                    "fixtures.principals.{i}.part: `{}` is no principal part",
                    rp.part
                ),
            ),
        }
    }
    let declared = unavailable::names();
    for (i, e) in s.then.events.iter().enumerate() {
        let p = format!("then.events.{i}");
        if let Some(ev) = &e.event
            && !declared.events.contains(ev)
        {
            let hint = match ev == REFUSAL {
                true => format!(
                    "; a refusal is expected by `refusal: <command>` (an input: `refusal: {}`), `error` or `rule`",
                    events::INPUT_COMMAND
                ),
                false => near_event(ev, &declared.events)
                    .map(|n| format!("; did you mean {n}?"))
                    .unwrap_or_default(),
            };
            push(
                "unknown_event",
                format!("{p}.event: `{ev}` is no event of the blocks{hint}"),
            );
        }
        match (&e.refusal, &e.error) {
            (Some(c), _) if !declared.refusals.contains_key(c) => push(
                "unknown_refusal",
                format!("{p}.refusal: `{c}` is no command of the blocks"),
            ),
            (Some(c), Some(err)) if !events::refusal_known(c, err, &declared.refusals[c]) => {
                let mut known: Vec<&str> =
                    declared.refusals[c].iter().map(String::as_str).collect();
                for (_, rc, re, _) in events::RULES {
                    if let Some(re) = re
                        && rc == c
                        && !known.contains(re)
                    {
                        known.push(re);
                    }
                }
                push(
                    "unknown_refusal",
                    format!(
                        "{p}.error: `{c}` is not refused with `{err}` (it is refused with {})",
                        if known.is_empty() {
                            "no error".to_string()
                        } else {
                            known.join(", ")
                        }
                    ),
                )
            }
            (None, Some(err)) if !declared.errors.contains(err) => push(
                "unknown_refusal",
                format!("{p}.error: `{err}` is no error of the blocks"),
            ),
            _ => {}
        }
        for (key, v) in [
            ("part", &e.part),
            ("author", &e.author),
            ("channel", &e.channel),
        ] {
            if let Some(n) = v
                && !is_part(n)
            {
                push(
                    REFUSED,
                    format!("{p}.{key}: `{n}` is no part of the architecture"),
                );
            }
        }
    }
    for problem in unavailable::check(a, s.horizon, &s.unavailable, &s.then.parts) {
        push(REFUSED, problem);
    }
    out
}

/// The declared event whose short name (after the last `.`) is that of `name` but for case and
/// `_` (`run_started`), or one edit away from it in the same domain; one the simulator records
/// first, else the first in order.
fn near_event<'a>(name: &str, declared: &'a BTreeSet<String>) -> Option<&'a str> {
    let (domain, short) = name.rsplit_once('.').unwrap_or(("", name));
    let snake = short.replace('_', "").to_lowercase();
    let near: Vec<&str> = declared
        .iter()
        .filter_map(|d| {
            let (dd, ds) = d.rsplit_once('.')?;
            let same = dd == domain || domain.is_empty();
            let close = ds.to_lowercase() == snake || (dd == domain && one_edit(ds, short));
            (same && close).then_some(d.as_str())
        })
        .collect();
    near.iter()
        .find(|d| events::modeled(d))
        .or(near.first())
        .copied()
}

/// Whether `a` becomes `b` by inserting, deleting or replacing one character.
fn one_edit(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if long.len() - short.len() > 1 {
        return false;
    }
    let mut i = 0;
    while i < short.len() && short[i] == long[i] {
        i += 1;
    }
    if short.len() == long.len() {
        short[i..].iter().skip(1).eq(long[i..].iter().skip(1))
    } else {
        short[i..] == long[i + 1..]
    }
}

/// Loads and checks a scenario; refuses it with every problem, by the code they share.
pub fn load_checked(path: &Path) -> Result<Loaded, Refusal> {
    let l = load(path)?;
    let problems = problems(&l);
    if !problems.is_empty() {
        return Err(refusal(path, &problems));
    }
    Ok(l)
}

/// How the runner decided one input.
#[derive(Debug, Clone, PartialEq)]
pub struct Admission {
    pub t: f64,
    /// The rule that refused it.
    pub rule: Option<&'static str>,
    /// The channel the link carries it over.
    pub channel: Option<String>,
}

/// Admits one input: a link from its author to its part carries its kind (any link for a
/// message or a control input), and a control input comes, with proof, from the principal the
/// part acts for.
pub fn admit(arch: &Value, input: &Input) -> Result<Option<String>, &'static str> {
    let links = arch["links"].as_array().cloned().unwrap_or_default();
    let carries = match input.kind {
        InputKind::Prompt => Some("prompt"),
        InputKind::Order => Some("order"),
        InputKind::Message | InputKind::Control => None,
    };
    let link = links.iter().find(|l| {
        l["from"] == input.author.as_str()
            && l["to"] == input.at.as_str()
            && carries.is_none_or(|c| l["carries"] == c)
    });
    let Some(link) = link else {
        return Err("input_without_link");
    };
    if input.kind == InputKind::Control {
        if arch["parts"][&input.at]["acts_for"] != input.author.as_str() {
            return Err("control_without_authority");
        }
        if input.proof.as_deref().is_none_or(|p| p == "none") {
            return Err("control_without_proof");
        }
    }
    Ok(link["over"].as_str().map(str::to_string))
}

/// One event of a scenario's trace, in block terms.
#[derive(Debug, Clone, PartialEq)]
pub struct Traced {
    pub t: f64,
    /// The block event, or `refusal`.
    pub event: &'static str,
    /// The block command a refusal refuses.
    pub command: Option<&'static str>,
    pub part: String,
    /// The simulator's own kind it was mapped from.
    pub sim: &'static str,
    pub task: Option<String>,
    pub author: Option<String>,
    pub channel: Option<String>,
    pub rule: Option<String>,
    pub error: Option<String>,
    pub options: Option<Vec<String>>,
    pub default: Option<String>,
    pub option: Option<String>,
    pub input: Option<usize>,
}

impl Traced {
    fn new(t: f64, event: &'static str, part: &str, sim: &'static str) -> Self {
        Traced {
            t,
            event,
            command: None,
            part: part.to_string(),
            sim,
            task: None,
            author: None,
            channel: None,
            rule: None,
            error: None,
            options: None,
            default: None,
            option: None,
            input: None,
        }
    }

    pub fn json(&self) -> Value {
        let mut m = Map::new();
        m.insert("t_seconds".into(), json!(self.t));
        m.insert("event".into(), json!(self.event));
        if let Some(c) = self.command {
            m.insert("command".into(), json!(c));
        }
        m.insert("part".into(), json!(self.part));
        m.insert("sim".into(), json!(self.sim));
        let opt = |m: &mut Map<String, Value>, k: &str, v: &Option<String>| {
            if let Some(v) = v {
                m.insert(k.into(), json!(v));
            }
        };
        opt(&mut m, "task", &self.task);
        opt(&mut m, "author", &self.author);
        opt(&mut m, "channel", &self.channel);
        opt(&mut m, "rule", &self.rule);
        opt(&mut m, "error", &self.error);
        if let Some(o) = &self.options {
            m.insert("options".into(), json!(o));
        }
        opt(&mut m, "default", &self.default);
        opt(&mut m, "option", &self.option);
        if let Some(i) = self.input {
            m.insert("input".into(), json!(i));
        }
        Value::Object(m)
    }
}

/// The status of one expectation or bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Held,
    Failed,
    NotChecked,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Held => "held",
            Status::Failed => "failed",
            Status::NotChecked => "not_checked",
        }
    }
}

/// The verdict of one expectation: its status, the trace entry it matched, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub status: Status,
    pub matched: Option<usize>,
    pub reason: Option<String>,
}

/// What a scenario run gives.
#[derive(Debug, Clone)]
pub struct Ran {
    pub admissions: Vec<Admission>,
    pub trace: Vec<Traced>,
    pub verdicts: Vec<Verdict>,
    pub bounds: Vec<(Status, f64)>,
    pub predicted: Class,
    pub run: RunOut,
    /// How the run made each `unavailable` entry unavailable.
    pub plan: Plan,
    /// The verdict of each observable of `then.parts`, in the order of
    /// [`unavailable::observables`].
    pub observed: Vec<Verdict>,
}

impl Ran {
    pub fn count(&self, s: Status) -> usize {
        self.verdicts.iter().filter(|v| v.status == s).count()
            + self.bounds.iter().filter(|(b, _)| *b == s).count()
            + self.observed.iter().filter(|v| v.status == s).count()
    }
}

/// What a task of the run is: a given row or the task an input made.
#[derive(Debug, Clone)]
enum Origin {
    GivenTask(String),
    GivenQuestion(usize),
    Input(usize),
}

/// Runs a checked scenario on the simulator: the inputs at their times, the fixtures in place of
/// the model, the build tool and the principals, the scenario's seed; then its expectations.
pub fn run(l: &Loaded) -> Result<Ran, Refusal> {
    let s = &l.scenario;
    let invalid = |e: String| refused("scenario_refused", format!("{}: {e}", l.path.display()));
    let arch = resolve(&l.architecture, &Map::new()).map_err(invalid)?;
    let mut admissions = Vec::new();
    let mut t = 0.0;
    let mut arrivals = Vec::new();
    let mut origins = Vec::new();
    let home = border(&arch)
        .first()
        .map(|b| b.to_string())
        .or_else(|| parts(&arch).first().map(|p| p.to_string()))
        .unwrap_or_default();
    for g in &s.given.tasks {
        arrivals.push(Arrival {
            at: 0.0,
            part: home.clone(),
            script: None,
        });
        origins.push(Origin::GivenTask(g.id.clone()));
    }
    for (q, g) in s.given.questions.iter().enumerate() {
        arrivals.push(Arrival {
            at: 0.0,
            part: g.asked_by.clone(),
            script: Some(TaskScript {
                outcome: Some(RunOutcome::Question),
                turns: None,
                builds: Some(0),
                needs_owner: Some(block(&arch, &g.addressee) == Some("principal")),
                question_first: true,
            }),
        });
        origins.push(Origin::GivenQuestion(q));
    }
    let mut plan = unavailable::plan(
        &arch,
        s.horizon,
        &s.unavailable,
        &s.then.parts,
        !s.given.tasks.is_empty() || !s.given.questions.is_empty(),
    );
    for (i, input) in s.when.iter().enumerate() {
        t += input.after;
        let decided = admit(&arch, input).and_then(|over| {
            if plan.refuses(&input.at, &input.author, over.as_deref(), t) {
                Err(unavailable::RULE)
            } else {
                Ok(over)
            }
        });
        admissions.push(Admission {
            t,
            rule: decided.as_ref().err().copied(),
            channel: decided.clone().ok().flatten(),
        });
        if decided.is_ok() && input.makes_task() {
            arrivals.push(Arrival {
                at: t,
                part: input.at.clone(),
                script: None,
            });
            origins.push(Origin::Input(i));
        }
    }
    let script = Script {
        arrivals,
        runs: s
            .fixtures
            .model
            .iter()
            .map(|m| {
                (
                    m.part.clone(),
                    TaskScript {
                        outcome: Some(m.outcome),
                        turns: m.turns,
                        builds: Some(m.builds),
                        needs_owner: Some(m.needs_owner),
                        question_first: false,
                    },
                )
            })
            .collect(),
        builds: s.fixtures.tools.iter().map(|t| t.seconds).collect(),
        replies: s
            .fixtures
            .principals
            .iter()
            .map(|r| (r.part.clone(), r.after))
            .collect(),
    };
    let model = Model::build(&arch, &l.profile, s.horizon, &plan.faults, &[])
        .and_then(|m| m.with_script(script))
        .map_err(invalid)?;
    let run = simulate(&model, s.seed, true);
    let trace = trace(&arch, s, &admissions, &origins, &run);
    // An entry that refuses inputs holds only while the simulation leaves its part quiet.
    for (mode, &(from, until)) in plan.modes.iter_mut().zip(&plan.windows) {
        let broken = trace.iter().find(|e| {
            !e.sim.starts_with("input_")
                && mode.broken_by(from, until, e.t, &e.part, e.channel.as_deref())
        });
        if let Some(e) = broken {
            *mode = Mode::NotModeled(format!(
                "the simulator records {} by `{}` at {}s while it is unavailable",
                e.command
                    .map_or(e.event.to_string(), |c| format!("a refusal of {c}")),
                e.part,
                e.t
            ));
        }
    }
    let mut verdicts = evaluate(&s.then, &trace);
    let mut bounds: Vec<(Status, f64)> = s
        .then
        .kpis
        .iter()
        .map(|b| {
            let v = run.kpis.get(&b.kpi).copied().unwrap_or(0.0);
            let ok = b.min.is_none_or(|x| v >= x) && b.max.is_none_or(|x| v <= x);
            (if ok { Status::Held } else { Status::Failed }, v)
        })
        .collect();
    // A run that cannot make an entry unavailable checks nothing of what it expects.
    if let Some(why) = &unmodeled(&plan) {
        for v in &mut verdicts {
            *v = Verdict {
                status: Status::NotChecked,
                matched: None,
                reason: Some(why.clone()),
            };
        }
        for b in &mut bounds {
            b.0 = Status::NotChecked;
        }
    }
    let observed = unavailable::observables(&s.then.parts)
        .iter()
        .map(|(_, what, _)| Verdict {
            status: Status::NotChecked,
            matched: None,
            reason: Some(unavailable::not_checked_reason(what).to_string()),
        })
        .collect();
    let predicted = predict(&admissions, &trace, &run);
    Ok(Ran {
        admissions,
        trace,
        verdicts,
        bounds,
        predicted,
        run,
        plan,
        observed,
    })
}

/// Why the run checks none of its expected events and bounds: the first entry the simulator
/// does not model.
fn unmodeled(plan: &Plan) -> Option<String> {
    plan.modes.iter().enumerate().find_map(|(i, m)| match m {
        Mode::NotModeled(why) => Some(format!(
            "the simulator does not model unavailable.{i}: {why}"
        )),
        _ => None,
    })
}

/// The channel a part sends its questions over, if its question link names one.
fn question_channel(arch: &Value, part: &str) -> Option<String> {
    arch["links"].as_array().and_then(|ls| {
        ls.iter()
            .find(|l| l["from"] == part && l["carries"] == "question" && l["over"].is_string())
            .and_then(|l| l["over"].as_str().map(str::to_string))
    })
}

fn trace(
    arch: &Value,
    s: &Scenario,
    admissions: &[Admission],
    origins: &[Origin],
    run: &RunOut,
) -> Vec<Traced> {
    let mut out = Vec::new();
    for (i, (input, a)) in s.when.iter().zip(admissions).enumerate() {
        let (kind, event) = match a.rule {
            Some(_) => ("input_refused", REFUSAL),
            None => ("input_delivered", "agentic.channel.MessageSent"),
        };
        let mut e = Traced::new(a.t, event, &input.at, kind);
        e.author = Some(input.author.clone());
        e.channel = a.channel.clone();
        e.input = Some(i);
        if let Some(rule) = a.rule {
            e.command = Some(events::INPUT_COMMAND);
            e.rule = Some(rule.to_string());
            e.error = events::rule_error(rule).map(str::to_string);
        }
        out.push(e);
    }
    let label = |task: usize| -> Option<String> {
        let rec = run.tasks.get(task)?;
        Some(match rec.arrival.and_then(|a| origins.get(a)) {
            Some(Origin::GivenTask(id)) => id.clone(),
            Some(Origin::GivenQuestion(q)) => s.given.questions[*q].id.clone(),
            Some(Origin::Input(i)) => format!("when.{i}"),
            None => format!("task.{task}"),
        })
    };
    // The question of a task: its options and default, from its fixture or its given row.
    let question = |task: usize| -> (Option<Vec<String>>, Option<String>) {
        let Some(rec) = run.tasks.get(task) else {
            return (None, None);
        };
        if let Some(Origin::GivenQuestion(q)) = rec.arrival.and_then(|a| origins.get(a)) {
            let g = &s.given.questions[*q];
            return (Some(g.options.clone()), Some(g.default.clone()));
        }
        match rec.fixture.map(|f| &s.fixtures.model[f]) {
            Some(m) if !m.options.is_empty() => (Some(m.options.clone()), m.default.clone()),
            _ => (None, None),
        }
    };
    let given_question = |task: usize| -> Option<&GivenQuestion> {
        match run.tasks.get(task)?.arrival.and_then(|a| origins.get(a)) {
            Some(Origin::GivenQuestion(q)) => Some(&s.given.questions[*q]),
            _ => None,
        }
    };
    let is_given = |task: usize| {
        matches!(
            run.tasks
                .get(task)
                .and_then(|r| r.arrival)
                .and_then(|a| origins.get(a)),
            Some(Origin::GivenTask(_) | Origin::GivenQuestion(_))
        )
    };
    // The events that made a given row: they happened before the scenario starts.
    let mut skipped: BTreeSet<(usize, &'static str)> = BTreeSet::new();
    let mut sim = Vec::new();
    for st in &run.timeline {
        let Some(blocks) = events::block_events(st.kind) else {
            continue;
        };
        if let Some(task) = st.task
            && is_given(task)
        {
            let initial: &[&str] = match given_question(task) {
                Some(g) if block(arch, &g.addressee) == Some("principal") => &[
                    "task_arrived",
                    "task_started",
                    "question_asked",
                    "question_forwarded",
                ],
                Some(_) => &["task_arrived", "task_started", "question_asked"],
                None => &["task_arrived"],
            };
            if initial.contains(&st.kind) && skipped.insert((task, st.kind)) {
                continue;
            }
        }
        for &b in blocks {
            let mut e = Traced::new(st.t, b, &st.part, st.kind);
            e.task = st.task.and_then(label);
            if let Some(task) = st.task {
                match (st.kind, b) {
                    ("question_asked" | "question_forwarded", "agentic.question.QuestionAsked") => {
                        let (options, default) = question(task);
                        e.options = options;
                        e.default = default;
                    }
                    ("question_forwarded", "agentic.channel.MessageSent") => {
                        e.author = Some(st.part.clone());
                        e.channel = question_channel(arch, &st.part);
                    }
                    ("question_answered", _) => {
                        e.option = st
                            .reply
                            .and_then(|r| s.fixtures.principals[r].option.clone())
                            .or_else(|| st.reply.and_then(|_| question(task).1));
                    }
                    ("question_defaulted", _) => e.option = question(task).1,
                    _ => {}
                }
            }
            if b == REFUSAL {
                e.command = Some(events::rule_command(st.kind));
                e.rule = Some(st.kind.to_string());
                e.error = events::rule_error(st.kind).map(str::to_string);
            }
            sim.push(e);
        }
    }
    out.extend(sim);
    // Inputs first at the same time: an input causes what follows it.
    out.sort_by(|a, b| a.t.total_cmp(&b.t));
    out
}

fn matches(e: &Expect, t: &Traced) -> bool {
    let eq = |want: &Option<String>, have: &Option<String>| want.is_none() || want == have;
    let what = match &e.event {
        Some(ev) => t.event != REFUSAL && *ev == t.event,
        None => {
            t.event == REFUSAL
                && e.refusal.as_deref().is_none_or(|c| t.command == Some(c))
                && eq(&e.error, &t.error)
                && eq(&e.rule, &t.rule)
        }
    };
    what && e.part.as_ref().is_none_or(|p| *p == t.part)
        && eq(&e.task, &t.task)
        && eq(&e.option, &t.option)
        && eq(&e.channel, &t.channel)
        && eq(&e.author, &t.author)
        && (e.options.is_none() || e.options == t.options)
}

/// Why the simulator cannot check `e`: an event it does not record, a refusal of a command it
/// never refuses, or a process it does not model.
fn not_simulated(e: &Expect) -> Option<String> {
    match (&e.event, &e.refusal) {
        (Some(ev), _) if !events::modeled(ev) => {
            return Some(format!("the simulator does not record {ev}"));
        }
        (None, Some(c)) if !events::refuses(c) => {
            return Some(format!("the simulator refuses no {c}"));
        }
        _ => {}
    }
    e.process
        .as_ref()
        .map(|_| "the simulator follows no process a task names".to_string())
}

/// Evaluates the expected events on the trace: in order (each after the one before it) or as a
/// set (each on its own entry). An `absent` expectation holds when no entry matches it. An
/// expectation of an event the simulator does not record, of a refusal of a command it never
/// refuses, or with a key it does not model, is not checked.
pub fn evaluate(then: &Then, trace: &[Traced]) -> Vec<Verdict> {
    let mut from = 0;
    let mut used = vec![false; trace.len()];
    then.events
        .iter()
        .map(|e| {
            if let Some(why) = not_simulated(e) {
                return Verdict {
                    status: Status::NotChecked,
                    matched: None,
                    reason: Some(why),
                };
            }
            if e.absent {
                let hit = trace.iter().position(|t| matches(e, t));
                return Verdict {
                    status: if hit.is_none() {
                        Status::Held
                    } else {
                        Status::Failed
                    },
                    matched: hit,
                    reason: hit.map(|i| format!("trace entry {i} matches")),
                };
            }
            let hit = match then.order {
                Order::Sequence => (from..trace.len()).find(|&i| matches(e, &trace[i])),
                Order::Set => (0..trace.len()).find(|&i| !used[i] && matches(e, &trace[i])),
            };
            match hit {
                Some(i) => {
                    used[i] = true;
                    from = i + 1;
                    Verdict {
                        status: Status::Held,
                        matched: Some(i),
                        reason: None,
                    }
                }
                None => Verdict {
                    status: Status::Failed,
                    matched: None,
                    reason: Some(match then.order {
                        Order::Sequence => {
                            format!("no matching trace entry from entry {from} on")
                        }
                        Order::Set => "no matching trace entry left".into(),
                    }),
                },
            }
        })
        .collect()
}

/// The outcome class a run predicts: refused (an input was refused and no task was made),
/// question, false completion, correct, wrong (a run ended failed), or open.
pub fn predict(admissions: &[Admission], trace: &[Traced], run: &RunOut) -> Class {
    let has = |event: &str| trace.iter().any(|t| t.event == event);
    let refused = admissions.iter().any(|a| a.rule.is_some());
    if refused && !has("agentic.work.TaskCreated") && run.tasks.is_empty() {
        Class::Refused
    } else if has("agentic.question.QuestionAsked") {
        Class::Question
    } else if run
        .kpis
        .get("false_completions_shipped")
        .copied()
        .unwrap_or(0.0)
        > 0.0
    {
        Class::FalseCompletion
    } else if run.kpis.get("tasks_correct").copied().unwrap_or(0.0) > 0.0 {
        Class::Correct
    } else if has("agentic.work.RunEnded") {
        Class::Wrong
    } else {
        Class::Open
    }
}

/// The result document of a run, `agenticspec-scenario-result/1`.
pub fn result_json(l: &Loaded, ran: &Ran) -> Value {
    let s = &l.scenario;
    let expectations: Vec<Value> = s
        .then
        .events
        .iter()
        .zip(&ran.verdicts)
        .enumerate()
        .map(|(i, (e, v))| {
            let mut m = Map::new();
            m.insert("index".into(), json!(i));
            m.insert("expect".into(), e.written.clone());
            m.insert("status".into(), json!(v.status.name()));
            if let Some(t) = v.matched {
                m.insert("trace".into(), json!(t));
            }
            if let Some(r) = &v.reason {
                m.insert("reason".into(), json!(r));
            }
            Value::Object(m)
        })
        .collect();
    let bounds: Vec<Value> = s
        .then
        .kpis
        .iter()
        .zip(&ran.bounds)
        .map(|(b, (st, v))| {
            let mut row =
                json!({"kpi": b.kpi, "min": b.min, "max": b.max, "value": v, "status": st.name()});
            if *st == Status::NotChecked {
                row["reason"] = json!(unmodeled(&ran.plan));
            }
            row
        })
        .collect();
    let unavailable: Vec<Value> = s
        .unavailable
        .iter()
        .zip(ran.plan.modes.iter().zip(&ran.plan.windows))
        .enumerate()
        .map(|(i, (u, (mode, window)))| unavailable::entry_json(i, u, mode, *window))
        .collect();
    let observables: Vec<Value> = unavailable::observables(&s.then.parts)
        .into_iter()
        .zip(&ran.observed)
        .map(|((part, what, written), v)| {
            let mut m = Map::new();
            m.insert("part".into(), json!(part));
            m.insert("observable".into(), json!(what));
            m.insert("expect".into(), written);
            m.insert("status".into(), json!(v.status.name()));
            if let Some(r) = &v.reason {
                m.insert("reason".into(), json!(r));
            }
            Value::Object(m)
        })
        .collect();
    let inputs: Vec<Value> = s
        .when
        .iter()
        .zip(&ran.admissions)
        .enumerate()
        .map(|(i, (input, a))| {
            json!({
                "index": i,
                "t_seconds": a.t,
                "at": input.at,
                "kind": input.kind.name(),
                "act": input.act.name(),
                "author": input.author,
                "status": if a.rule.is_some() { "refused" } else { "accepted" },
                "rule": a.rule,
            })
        })
        .collect();
    let kpis: Map<String, Value> = ran
        .run
        .kpis
        .iter()
        .map(|(k, v)| (k.clone(), json!({"value": v, "kind": kpi_kind(k)})))
        .collect();
    let failed = ran.count(Status::Failed);
    json!({
        "format": RESULT_FORMAT,
        "scenario": s.name,
        "architecture": {"ref": s.architecture.reference, "digest": s.architecture.digest},
        "profile": {"ref": s.profile.reference, "digest": s.profile.digest},
        "seed": s.seed,
        "horizon_seconds": s.horizon,
        "verdict": if failed == 0 { "held" } else { "failed" },
        "predicted": ran.predicted.name(),
        "counts": {
            "held": ran.count(Status::Held),
            "failed": failed,
            "not_checked": ran.count(Status::NotChecked),
        },
        "expectations": expectations,
        "kpi_bounds": bounds,
        "unavailable": unavailable,
        "observables": observables,
        "inputs": inputs,
        "trace": ran.trace.iter().map(Traced::json).collect::<Vec<_>>(),
        "kpis": kpis,
    })
}
