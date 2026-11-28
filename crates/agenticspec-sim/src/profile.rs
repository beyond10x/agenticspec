//! The behaviour profile (`agenticspec-profile/1`): distributions the simulation draws from.

use crate::corpus::Class;
use crate::doc::num;
use crate::rng::Rng;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// The z-score of the 90th percentile of a standard normal distribution.
const Z90: f64 = 1.281_551_565_5;

#[derive(Debug, Clone, PartialEq)]
pub enum Dist {
    Fixed(f64),
    /// Given by its median and 90th percentile.
    Lognormal {
        mu: f64,
        sigma: f64,
    },
    Poisson {
        mean: f64,
    },
    Exponential {
        mean: f64,
    },
    Uniform {
        min: f64,
        max: f64,
    },
}

impl Dist {
    pub fn parse(path: &str, v: &Value) -> Result<Dist, String> {
        if let Some(x) = num(v) {
            return Ok(Dist::Fixed(x));
        }
        let field = |k: &str| {
            v.get(k)
                .and_then(num)
                .ok_or_else(|| format!("{path}: needs a numeric `{k}`"))
        };
        match v.get("dist").and_then(Value::as_str) {
            Some("fixed") => Ok(Dist::Fixed(field("value")?)),
            Some("lognormal") => {
                let (median, p90) = (field("median")?, field("p90")?);
                if median <= 0.0 || p90 < median {
                    return Err(format!("{path}: lognormal needs 0 < median <= p90"));
                }
                Ok(Dist::Lognormal {
                    mu: median.ln(),
                    sigma: (p90 / median).ln() / Z90,
                })
            }
            Some("poisson") => Ok(Dist::Poisson {
                mean: field("mean")?,
            }),
            Some("exponential") => Ok(Dist::Exponential {
                mean: field("mean")?,
            }),
            Some("uniform") => Ok(Dist::Uniform {
                min: field("min")?,
                max: field("max")?,
            }),
            Some(other) => Err(format!("{path}: unknown dist `{other}`")),
            None => Err(format!(
                "{path}: expected a number, a duration or {{ dist: ... }}"
            )),
        }
    }

    pub fn sample(&self, rng: &mut Rng) -> f64 {
        match *self {
            Dist::Fixed(x) => x,
            Dist::Lognormal { mu, sigma } => (mu + sigma * rng.normal()).exp(),
            Dist::Poisson { mean } => rng.poisson(mean) as f64,
            Dist::Exponential { mean } => rng.exponential(1.0 / mean.max(1e-12)),
            Dist::Uniform { min, max } => min + (max - min) * rng.unit(),
        }
    }
}

/// Token classes, in the order of every `[f64; 4]` in this crate.
pub const TOKEN_CLASSES: [&str; 4] = ["input", "cache_read", "cache_write", "output"];

/// The behaviour of one role, named by a part's `role`. A part that works tasks needs
/// `task_turns` and `turn_seconds`, a reviewer `turn_seconds`, an agent that answers questions
/// `answer_seconds`; the simulation refuses a part whose role lacks what it uses.
#[derive(Debug, Clone)]
pub struct Role {
    /// Turns of a task; for a runtime, the steps of a run.
    pub task_turns: Option<Dist>,
    /// Duration of one turn or step.
    pub turn_seconds: Option<Dist>,
    /// How long an agent of this role takes to answer or pass on a question.
    pub answer_seconds: Option<Dist>,
    pub tokens_per_turn: [Dist; 4],
    pub builds_per_task: Dist,
    pub build_seconds: Dist,
    pub questions_per_task: Dist,
    pub question_needs_owner: f64,
    /// Turns per step of a task's chain; `None` makes the whole task one step.
    pub step_turns: Option<Dist>,
    /// Pass chance of a task's first step.
    pub step_pass_first: f64,
    pub step_pass_after_pass: f64,
    pub step_pass_after_fail: f64,
    /// Rework turns after a failed step, and per round of a task that went back.
    pub rework_turns: Dist,
    /// Chance that a question that took its default rests on a wrong assumption.
    pub default_wrong: f64,
    /// Share of the task's turns a wrong assumption costs again.
    pub rework_share: f64,
    /// Chance that a task that is not correct is still reported done.
    pub false_completion: f64,
}

/// The false-accept and false-reject rate of a reviewer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strictness {
    pub false_accept: f64,
    pub false_reject: f64,
}

/// What a review takes and how strict a reviewer is, by name.
#[derive(Debug, Clone)]
pub struct Review {
    /// Model calls of one review.
    pub turns: Dist,
    pub strictness: BTreeMap<String, Strictness>,
}

/// The strictness table used when a profile has none. Lenient false accept and strict false
/// reject are the middles of the ranges of 2611.02207; the other two values are estimates and
/// `balanced` lies between them.
pub fn default_strictness() -> BTreeMap<String, Strictness> {
    [
        ("lenient", 0.86, 0.05),
        ("balanced", 0.58, 0.27),
        ("strict", 0.30, 0.495),
    ]
    .into_iter()
    .map(|(n, a, r)| {
        (
            n.to_string(),
            Strictness {
                false_accept: a,
                false_reject: r,
            },
        )
    })
    .collect()
}

/// A window `[from, from + for)` repeated every `every` seconds when given.
#[derive(Debug, Clone)]
pub struct Away {
    pub from: f64,
    pub length: f64,
    pub every: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Principal {
    pub answer_seconds: Dist,
    pub away: Vec<Away>,
}

/// What coordinating copies of an agent part costs.
#[derive(Debug, Clone)]
pub struct Coordination {
    /// Delay before a copy reads a new post on a board.
    pub board_read_seconds: Dist,
    /// A claim on a board entry is seen by the other readers only after this many seconds.
    pub claim_window_seconds: f64,
    /// Model calls a copy spends reading one mesh message.
    pub mesh_read_calls: f64,
    /// Chance that a task of a scope partition also touches one given other scope.
    pub scope_overlap: f64,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub roles: BTreeMap<String, Role>,
    pub principals: BTreeMap<String, Principal>,
    pub load: Load,
    /// Price per token of each class, by model route.
    pub prices: BTreeMap<String, [f64; 4]>,
    /// Price of one agent instance alive for one hour.
    pub agent_hour_usd: f64,
    pub review: Review,
    pub coordination: Coordination,
    /// `load.corpus`: a corpus file, relative to the profile, whose entries the arrivals draw.
    pub corpus_ref: Option<String>,
    /// The entries of that corpus by weight and expected class, once read ([`Profile::read_corpus`]).
    pub corpus: Vec<(f64, Class)>,
}

/// Where tasks come from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Load {
    /// Tasks per hour for the whole architecture.
    Total(f64),
    /// The older form: tasks per hour for each worker instance, summed over the instances.
    PerWorker(f64),
}

fn share(path: &str, v: &Value, default: f64) -> Result<f64, String> {
    match v {
        Value::Null => Ok(default),
        _ => v
            .as_f64()
            .filter(|x| (0.0..=1.0).contains(x))
            .ok_or_else(|| format!("{path}: expected a share from 0 to 1")),
    }
}

fn map<'a>(v: &'a Value, key: &str) -> impl Iterator<Item = (&'a String, &'a Value)> {
    v[key].as_object().into_iter().flatten()
}

fn dist_or(path: &str, v: &Value, default: Dist) -> Result<Dist, String> {
    if v.is_null() {
        Ok(default)
    } else {
        Dist::parse(path, v)
    }
}

fn optional(path: &str, v: &Value) -> Result<Option<Dist>, String> {
    if v.is_null() {
        Ok(None)
    } else {
        Dist::parse(path, v).map(Some)
    }
}

impl Profile {
    pub fn parse(doc: &Value) -> Result<Profile, String> {
        if let Some(f) = doc["format"].as_str()
            && f != "agenticspec-profile/1"
        {
            return Err(format!(
                "profile: format `{f}`, expected agenticspec-profile/1"
            ));
        }
        let mut roles = BTreeMap::new();
        for (name, r) in map(doc, "roles") {
            let p = |k: &str| format!("roles.{name}.{k}");
            let t = &r["tokens_per_turn"];
            let tokens = [
                dist_or(&p("tokens_per_turn.input"), &t["input"], Dist::Fixed(0.0))?,
                dist_or(
                    &p("tokens_per_turn.cache_read"),
                    &t["cache_read"],
                    Dist::Fixed(0.0),
                )?,
                dist_or(
                    &p("tokens_per_turn.cache_write"),
                    &t["cache_write"],
                    Dist::Fixed(0.0),
                )?,
                dist_or(&p("tokens_per_turn.output"), &t["output"], Dist::Fixed(0.0))?,
            ];
            roles.insert(
                name.clone(),
                Role {
                    task_turns: optional(&p("task_turns"), &r["task_turns"])?,
                    turn_seconds: optional(&p("turn_seconds"), &r["turn_seconds"])?,
                    answer_seconds: optional(&p("answer_seconds"), &r["answer_seconds"])?,
                    tokens_per_turn: tokens,
                    builds_per_task: dist_or(
                        &p("builds_per_task"),
                        &r["builds_per_task"],
                        Dist::Fixed(0.0),
                    )?,
                    build_seconds: dist_or(
                        &p("build_seconds"),
                        &r["build_seconds"],
                        Dist::Fixed(0.0),
                    )?,
                    questions_per_task: dist_or(
                        &p("questions_per_task"),
                        &r["questions_per_task"],
                        Dist::Fixed(0.0),
                    )?,
                    question_needs_owner: r["question_needs_owner"]
                        .as_f64()
                        .unwrap_or(0.0)
                        .clamp(0.0, 1.0),
                    step_turns: match &r["step_turns"] {
                        Value::Null => None,
                        v => Some(Dist::parse(&p("step_turns"), v)?),
                    },
                    step_pass_first: share(
                        &p("step_pass_first"),
                        &r["step_pass_first"],
                        share(&p("step_pass_after_pass"), &r["step_pass_after_pass"], 1.0)?,
                    )?,
                    step_pass_after_pass: share(
                        &p("step_pass_after_pass"),
                        &r["step_pass_after_pass"],
                        1.0,
                    )?,
                    step_pass_after_fail: share(
                        &p("step_pass_after_fail"),
                        &r["step_pass_after_fail"],
                        1.0,
                    )?,
                    rework_turns: dist_or(
                        &p("rework_turns"),
                        &r["rework_turns"],
                        Dist::Fixed(0.0),
                    )?,
                    default_wrong: share(&p("default_wrong"), &r["default_wrong"], 0.0)?,
                    rework_share: r["rework_share"].as_f64().unwrap_or(0.0).max(0.0),
                    false_completion: share(&p("false_completion"), &r["false_completion"], 1.0)?,
                },
            );
        }
        let mut principals = BTreeMap::new();
        for (kind, pr) in map(doc, "principals") {
            let p = |k: &str| format!("principals.{kind}.{k}");
            let mut away = Vec::new();
            for (i, w) in pr["away"].as_array().into_iter().flatten().enumerate() {
                let get = |k: &str| {
                    w.get(k)
                        .and_then(num)
                        .ok_or_else(|| format!("{}: needs `{k}`", p(&format!("away.{i}"))))
                };
                away.push(Away {
                    from: get("from")?,
                    length: get("for")?,
                    every: w.get("every").and_then(num).filter(|e| *e > 0.0),
                });
            }
            principals.insert(
                kind.clone(),
                Principal {
                    answer_seconds: Dist::parse(&p("answer_seconds"), &pr["answer_seconds"])?,
                    away,
                },
            );
        }
        if !doc["agents"].is_null() {
            return Err(
                "profile: `agents` names answer delays by part name; give each on the role \
                 of the part that answers instead, roles.<role>.answer_seconds"
                    .into(),
            );
        }
        let mut prices = BTreeMap::new();
        for (route, pr) in map(doc, "prices") {
            if !pr.is_object() {
                continue;
            }
            let per = match pr["per"].as_str().unwrap_or("1M_tokens_usd") {
                "1M_tokens_usd" => 1e6,
                "1K_tokens_usd" => 1e3,
                "token_usd" => 1.0,
                other => {
                    return Err(format!(
                        "prices.{route}.per: `{other}`, expected 1M_tokens_usd, 1K_tokens_usd or token_usd"
                    ));
                }
            };
            let mut row = [0.0; 4];
            for (i, class) in TOKEN_CLASSES.iter().enumerate() {
                row[i] = pr[*class].as_f64().unwrap_or(0.0) / per;
            }
            prices.insert(route.clone(), row);
        }
        let load = match (
            doc["load"]["tasks_per_hour"].as_f64(),
            doc["load"]["tasks_per_hour_per_worker"].as_f64(),
        ) {
            (Some(_), Some(_)) => {
                return Err(
                    "load: give tasks_per_hour or tasks_per_hour_per_worker, not both".into(),
                );
            }
            (Some(t), None) => Load::Total(t.max(0.0)),
            (None, Some(w)) => Load::PerWorker(w.max(0.0)),
            (None, None) => Load::Total(0.0),
        };
        let agent_hour_usd = match &doc["prices"]["agent_hour_usd"] {
            Value::Null => 0.0,
            v => v
                .as_f64()
                .filter(|x| *x >= 0.0)
                .ok_or("prices.agent_hour_usd: expected a number of at least 0")?,
        };
        let rv = &doc["review"];
        let mut strictness = default_strictness();
        for (name, s) in map(rv, "strictness") {
            let p = |k: &str| format!("review.strictness.{name}.{k}");
            strictness.insert(
                name.clone(),
                Strictness {
                    false_accept: share(&p("false_accept"), &s["false_accept"], 0.0)?,
                    false_reject: share(&p("false_reject"), &s["false_reject"], 0.0)?,
                },
            );
        }
        let review = Review {
            turns: dist_or("review.turns", &rv["turns"], Dist::Fixed(1.0))?,
            strictness,
        };
        let c = &doc["coordination"];
        let coordination = Coordination {
            board_read_seconds: dist_or(
                "coordination.board_read_seconds",
                &c["board_read_seconds"],
                Dist::Fixed(0.0),
            )?,
            claim_window_seconds: match &c["claim_window_seconds"] {
                Value::Null => 0.0,
                v => num(v).filter(|x| *x >= 0.0).ok_or(
                    "coordination.claim_window_seconds: expected a duration of at least 0",
                )?,
            },
            mesh_read_calls: match &c["mesh_read_calls"] {
                Value::Null => 1.0,
                v => v
                    .as_f64()
                    .filter(|x| *x >= 0.0)
                    .ok_or("coordination.mesh_read_calls: expected a number of at least 0")?,
            },
            scope_overlap: share("coordination.scope_overlap", &c["scope_overlap"], 0.0)?,
        };
        let corpus_ref = match &doc["load"]["corpus"] {
            Value::Null => None,
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => {
                return Err(
                    "load.corpus: expected the path of an agenticspec-corpus/1 file".into(),
                );
            }
        };
        Ok(Profile {
            roles,
            principals,
            load,
            prices,
            agent_hour_usd,
            review,
            coordination,
            corpus_ref,
            corpus: Vec::new(),
        })
    }

    /// Reads the corpus `load.corpus` names, relative to `dir`, the directory of the profile.
    pub fn read_corpus(&mut self, dir: &Path) -> Result<(), String> {
        if let Some(r) = &self.corpus_ref {
            let corpus =
                crate::corpus::load(&dir.join(r)).map_err(|e| format!("load.corpus: {e}"))?;
            self.corpus = corpus
                .entries
                .iter()
                .map(|e| (e.weight, e.expected))
                .collect();
        }
        Ok(())
    }
}
