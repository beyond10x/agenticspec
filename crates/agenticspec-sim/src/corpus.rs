//! Task corpora (`agenticspec-corpus/1`): inputs taken from real sessions, each a scenario
//! reference or an inline entry with its input kind, the digest of its input and the outcome
//! class it had. A corpus is run entry by entry (`sim corpus run`), and a profile draws its
//! tasks from one (`load.corpus`).

use crate::cli::Refusal;
use crate::doc::load_yaml;
use crate::reader::{DocRef, Reader, sha256_hex};
use crate::scenario::{self, ACTS, Act, INPUT_KINDS, Input, InputKind, Loaded, Scenario, Then};
use crate::sim::RunOutcome;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "agenticspec-corpus/1";

/// How an input ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// A task was completed, and it is correct.
    Correct,
    /// A run ended failed.
    Wrong,
    /// A task was reported done and is not correct.
    FalseCompletion,
    /// A question was asked.
    Question,
    /// The input was refused and made no task.
    Refused,
    /// Nothing ended within the horizon.
    Open,
}

pub const CLASSES: &[(&str, Class)] = &[
    ("correct", Class::Correct),
    ("wrong", Class::Wrong),
    ("false_completion", Class::FalseCompletion),
    ("question", Class::Question),
    ("refused", Class::Refused),
    ("open", Class::Open),
];

impl Class {
    pub fn name(self) -> &'static str {
        CLASSES
            .iter()
            .find(|(_, c)| *c == self)
            .map_or("", |(n, _)| n)
    }

    /// The outcome of the first run a task of this class has; `None` for refused and open.
    pub fn outcome(self) -> Option<RunOutcome> {
        match self {
            Class::Correct => Some(RunOutcome::Correct),
            Class::Wrong => Some(RunOutcome::Wrong),
            Class::FalseCompletion => Some(RunOutcome::FalseCompletion),
            Class::Question => Some(RunOutcome::Question),
            Class::Refused | Class::Open => None,
        }
    }
}

/// Where an entry's input comes from.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Scenario(DocRef),
    /// One input, at `at` (the architecture's border part when there is one) from `author` (the
    /// principal the part acts for when not given).
    Inline {
        at: Option<String>,
        author: Option<String>,
        act: Option<Act>,
        proof: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    pub source: Source,
    pub kind: Option<InputKind>,
    pub input_digest: Option<String>,
    pub expected: Class,
    pub weight: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Corpus {
    pub name: String,
    pub summary: Option<String>,
    pub architecture: Option<DocRef>,
    pub profile: Option<DocRef>,
    pub seed: u64,
    pub horizon: f64,
    pub entries: Vec<Entry>,
}

/// Reads a corpus document; every problem is named by its path.
pub fn parse(doc: &Value) -> Result<Corpus, Vec<String>> {
    let mut r = Reader::default();
    let top = [
        "format",
        "corpus",
        "summary",
        "architecture",
        "profile",
        "seed",
        "horizon",
        "entries",
    ];
    let Some(m) = r.obj(doc, "", &top, &["format", "corpus", "entries"]) else {
        return Err(r.errors);
    };
    r.format(m, FORMAT);
    let name = r.text(m, "corpus", "");
    let summary = r.text(m, "summary", "");
    let architecture = r.doc_ref(m, "architecture", "");
    let profile = r.doc_ref(m, "profile", "");
    let seed = r.uint(m, "seed", "").unwrap_or(0);
    let horizon = r.duration(m, "horizon", "").unwrap_or(4.0 * 3600.0);
    let mut entries = Vec::new();
    let list = r.list(m, "entries", "");
    if list.is_empty() && m.contains_key("entries") {
        r.err("entries", "a corpus has at least one entry");
    }
    for (i, v) in list.iter().enumerate() {
        let p = format!("entries.{i}");
        let keys = [
            "id",
            "scenario",
            "kind",
            "input_digest",
            "expected",
            "weight",
            "at",
            "author",
            "act",
            "proof",
        ];
        let Some(o) = r.obj(v, &p, &keys, &["expected"]) else {
            continue;
        };
        let id = r.text(o, "id", &p);
        let scenario = r.doc_ref(o, "scenario", &p);
        let kind = r.one_of(o, "kind", &p, INPUT_KINDS);
        let input_digest = r.digest(o, "input_digest", &p);
        let expected = r.one_of(o, "expected", &p, CLASSES);
        let weight = r.number(o, "weight", &p).unwrap_or(1.0);
        if weight < 0.0 {
            r.err(&format!("{p}.weight"), "expected a number of at least 0");
        }
        let source = match scenario {
            Some(s) => {
                for k in ["at", "author", "act", "proof"] {
                    if o.contains_key(k) {
                        r.err(
                            &format!("{p}.{k}"),
                            "belongs to an inline entry; a scenario names its inputs",
                        );
                    }
                }
                Source::Scenario(s)
            }
            None if o.contains_key("scenario") => continue,
            None => {
                for k in ["id", "kind", "input_digest"] {
                    if !o.contains_key(k) {
                        r.err(&format!("{p}.{k}"), "missing; an inline entry names it");
                    }
                }
                Source::Inline {
                    at: r.text(o, "at", &p),
                    author: r.text(o, "author", &p),
                    act: r.one_of(o, "act", &p, ACTS),
                    proof: r.text(o, "proof", &p),
                }
            }
        };
        let id = id.or_else(|| match &source {
            Source::Scenario(s) => Path::new(&s.reference)
                .file_name()
                .and_then(|f| f.to_str())
                .map(|f| f.split('.').next().unwrap_or(f).to_string()),
            Source::Inline { .. } => None,
        });
        if let (Some(id), Some(expected)) = (id, expected) {
            entries.push(Entry {
                id,
                source,
                kind,
                input_digest,
                expected,
                weight,
            });
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for (i, e) in entries.iter().enumerate() {
        if !seen.insert(e.id.clone()) {
            r.err(
                &format!("entries.{i}.id"),
                format!("`{}` is used twice", e.id),
            );
        }
    }
    match name {
        Some(name) if r.errors.is_empty() => Ok(Corpus {
            name,
            summary,
            architecture,
            profile,
            seed,
            horizon,
            entries,
        }),
        _ => Err(r.errors),
    }
}

/// Reads a corpus file.
pub fn load(path: &Path) -> Result<Corpus, String> {
    let doc = load_yaml(path)?;
    parse(&doc).map_err(|e| format!("{}: {}", path.display(), e.join("; ")))
}

fn refused(code: &'static str, detail: impl Into<String>) -> Refusal {
    Refusal {
        code,
        detail: detail.into(),
    }
}

/// One row of a corpus run.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub entry: String,
    pub kind: String,
    pub expected: Class,
    pub predicted: Class,
    pub held: usize,
    pub failed: usize,
    pub not_checked: usize,
}

impl Row {
    /// An entry holds when none of its expectations failed and it predicts its class.
    pub fn holds(&self) -> bool {
        self.failed == 0 && self.expected == self.predicted
    }

    pub fn json(&self) -> Value {
        json!({
            "entry": self.entry,
            "kind": self.kind,
            "expected": self.expected.name(),
            "predicted": self.predicted.name(),
            "held": self.held,
            "failed": self.failed,
            "not_checked": self.not_checked,
            "verdict": if self.holds() { "held" } else { "failed" },
        })
    }
}

/// The scenario of an inline entry: its one input at the corpus's architecture, no fixtures and
/// no expectations, so the profile and the architecture decide what happens.
fn inline(c: &Corpus, dir: &Path, e: &Entry) -> Result<Loaded, Refusal> {
    let Source::Inline {
        at,
        author,
        act,
        proof,
    } = &e.source
    else {
        unreachable!("only inline entries")
    };
    let (Some(a), Some(p)) = (&c.architecture, &c.profile) else {
        return Err(refused(
            "corpus_invalid",
            format!(
                "entries: `{}` is inline: the corpus names its architecture and profile to run it",
                e.id
            ),
        ));
    };
    let arch = load_yaml(&dir.join(&a.reference)).map_err(|x| refused("document_unreadable", x))?;
    let at = match at {
        Some(at) => at.clone(),
        None => match arch["border"].as_array().map(Vec::as_slice) {
            Some([b]) => b.as_str().unwrap_or_default().to_string(),
            _ => {
                return Err(refused(
                    "corpus_invalid",
                    format!(
                        "entries: `{}` names no `at` and the architecture has not one border part",
                        e.id
                    ),
                ));
            }
        },
    };
    let author = match author {
        Some(x) => x.clone(),
        None => arch["parts"][&at]["acts_for"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| {
                refused(
                    "corpus_invalid",
                    format!(
                        "entries: `{}` names no `author` and `{at}` acts for no principal",
                        e.id
                    ),
                )
            })?,
    };
    let kind = e.kind.unwrap_or(InputKind::Prompt);
    let act = match (act, kind) {
        (Some(a), _) => *a,
        (None, InputKind::Order) => Act::Dispatch,
        (None, InputKind::Control) => Act::Run,
        (None, _) => Act::Send,
    };
    let scenario = Scenario {
        name: e.id.clone(),
        summary: None,
        architecture: a.clone(),
        profile: p.clone(),
        seed: c.seed,
        horizon: c.horizon,
        given: Default::default(),
        when: vec![Input {
            at,
            kind,
            author,
            proof: proof.clone(),
            text: None,
            act,
            after: 0.0,
        }],
        fixtures: Default::default(),
        then: Then {
            order: scenario::Order::Sequence,
            events: Vec::new(),
            kpis: Vec::new(),
        },
    };
    // The loader reads the documents; the scenario is the one built here.
    let mut l = scenario::load_with(&dir.join("inline"), scenario)?;
    let problems = scenario::check(&l);
    if !problems.is_empty() {
        return Err(refused(
            "scenario_refused",
            format!("entries: `{}`: {}", e.id, problems.join("; ")),
        ));
    }
    l.path = PathBuf::from(format!("{}#{}", dir.display(), e.id));
    Ok(l)
}

/// Runs every entry of a corpus: a reference runs its scenario with its fixtures and
/// expectations; an inline entry runs its one input with no fixtures. Each row compares the class
/// the run predicts with the class the entry expects.
pub fn run(path: &Path) -> Result<(Corpus, Vec<Row>), Refusal> {
    let doc = load_yaml(path).map_err(|e| refused("document_unreadable", e))?;
    let c = parse(&doc).map_err(|e| {
        refused(
            "corpus_invalid",
            format!("{}: {}", path.display(), e.join("; ")),
        )
    })?;
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut rows = Vec::new();
    for e in &c.entries {
        let l = match &e.source {
            Source::Scenario(r) => {
                let p = dir.join(&r.reference);
                let bytes = std::fs::read(&p)
                    .map_err(|x| refused("document_unreadable", format!("{}: {x}", p.display())))?;
                let actual = sha256_hex(&bytes);
                if actual != r.digest {
                    return Err(refused(
                        "corpus_invalid",
                        format!(
                            "entries: `{}`: scenario.digest {} is not the sha256 of {}, {actual}",
                            e.id,
                            r.digest,
                            p.display()
                        ),
                    ));
                }
                let l = scenario::load_checked(&p)?;
                let first = l.scenario.when.first();
                if let (Some(k), Some(f)) = (e.kind, first)
                    && k != f.kind
                {
                    return Err(refused(
                        "corpus_invalid",
                        format!(
                            "entries: `{}`: kind {} is not the kind of the scenario's first input, {}",
                            e.id,
                            k.name(),
                            f.kind.name()
                        ),
                    ));
                }
                if let Some(d) = &e.input_digest {
                    let text = first.and_then(|f| f.text.clone()).unwrap_or_default();
                    let actual = sha256_hex(text.as_bytes());
                    if *d != actual {
                        return Err(refused(
                            "corpus_invalid",
                            format!(
                                "entries: `{}`: input_digest {d} is not the sha256 of the text of the scenario's first input, {actual}",
                                e.id
                            ),
                        ));
                    }
                }
                l
            }
            Source::Inline { .. } => inline(&c, dir, e)?,
        };
        let ran = scenario::run(&l)?;
        let kind = e
            .kind
            .or(l.scenario.when.first().map(|i| i.kind))
            .map_or("-", InputKind::name);
        rows.push(Row {
            entry: e.id.clone(),
            kind: kind.to_string(),
            expected: e.expected,
            predicted: ran.predicted,
            held: ran.count(scenario::Status::Held),
            failed: ran.count(scenario::Status::Failed),
            not_checked: ran.count(scenario::Status::NotChecked),
        });
    }
    Ok((c, rows))
}

/// The table `sim corpus run` prints.
pub fn table(rows: &[Row]) -> String {
    let mut out = format!(
        "{:<28} {:<8} {:<17} {:<17} {:>4} {:>6} {:>11}  {}\n",
        "entry", "kind", "expected", "predicted", "held", "failed", "not_checked", "verdict"
    );
    for r in rows {
        out.push_str(&format!(
            "{:<28} {:<8} {:<17} {:<17} {:>4} {:>6} {:>11}  {}\n",
            r.entry,
            r.kind,
            r.expected.name(),
            r.predicted.name(),
            r.held,
            r.failed,
            r.not_checked,
            if r.holds() { "held" } else { "failed" }
        ));
    }
    out
}
