//! The two inputs, read leniently: every field is optional and a value of the wrong shape is
//! skipped, never a panic.

use crate::fmt;
use serde_json::{Map, Value};

/// A place marked for optimization: `{ value, vary }`.
pub struct Place {
    /// `parts.<part>.<field>` or `policies.<name>`.
    pub path: String,
    /// The field as shown on a chip: `capacity`, `limits.cpu`.
    pub field: String,
    pub value: Value,
    pub vary: Value,
}

impl Place {
    /// The search range as short text: `8…32`, `5m | 15m | 60m`.
    pub fn range(&self) -> String {
        range_text(&self.vary)
    }
}

pub fn range_text(vary: &Value) -> String {
    match vary {
        Value::Array(xs) => {
            let nums: Vec<f64> = xs.iter().filter_map(Value::as_f64).collect();
            if !xs.is_empty() && nums.len() == xs.len() {
                let lo = nums.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = nums.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                format!("{}…{}", fmt::num(lo), fmt::num(hi))
            } else {
                xs.iter().map(fmt::value).collect::<Vec<_>>().join(" | ")
            }
        }
        Value::Object(o) => {
            let lo = o.get("min").map(fmt::value).unwrap_or_default();
            let hi = o.get("max").map(fmt::value).unwrap_or_default();
            format!("{lo}…{hi}")
        }
        Value::Null => String::new(),
        other => fmt::value(other),
    }
}

/// The value of a possibly marked place.
pub fn plain(v: &Value) -> &Value {
    match marked(v) {
        Some((value, _)) => value,
        None => v,
    }
}

fn marked(v: &Value) -> Option<(&Value, &Value)> {
    let o = v.as_object()?;
    Some((o.get("value")?, o.get("vary")?))
}

fn places_in(prefix: &str, field: &str, v: &Value, out: &mut Vec<Place>) {
    if let Some((value, vary)) = marked(v) {
        out.push(Place {
            path: format!("{prefix}.{field}"),
            field: field.to_string(),
            value: value.clone(),
            vary: vary.clone(),
        });
    } else if let Some(o) = v.as_object() {
        for (k, inner) in o {
            places_in(prefix, &format!("{field}.{k}"), inner, out);
        }
    }
}

pub struct Part {
    pub name: String,
    pub block: String,
    pub kind: Option<String>,
    pub count: Option<String>,
    pub per: Option<String>,
    pub acts_for: Option<String>,
    /// Plain properties, shown as `key value`.
    pub props: Vec<(String, String)>,
    pub places: Vec<Place>,
    /// For a part of block `architecture`: the architecture it nests.
    pub nested: Option<Nested>,
}

/// The architecture a part of block `architecture` nests, as far as the document tells: its name
/// and part count when the simulator resolved it beside the part (`inner`), its border parts as
/// ports, addressed `<part>.<border part>`.
pub struct Nested {
    pub name: Option<String>,
    pub parts: Option<usize>,
    pub reference: Option<String>,
    pub digest: Option<String>,
    /// Full addresses, `console.steward`: the nested border, or without it the addresses the
    /// document writes.
    pub ports: Vec<String>,
}

/// Whether link end `end` lands on part `name`: the part itself or one of its ports.
pub fn lands_on(end: &str, name: &str) -> bool {
    end == name
        || end
            .strip_prefix(name)
            .is_some_and(|rest| rest.starts_with('.'))
}

pub struct Link {
    pub from: String,
    pub to: String,
    pub carries: String,
    pub over: Option<String>,
}

pub struct Architecture {
    pub name: String,
    pub summary: String,
    pub format: String,
    pub parts: Vec<Part>,
    pub links: Vec<Link>,
    pub border: Vec<String>,
    pub policies: Vec<(String, String)>,
    pub policy_places: Vec<Place>,
}

fn text(v: Option<&Value>) -> Option<String> {
    match v.map(plain)? {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        other => Some(fmt::value(other)),
    }
}

const SPECIAL: [&str; 7] = [
    "block", "kind", "count", "per", "acts_for", "inner", "digest",
];

/// The names written as strings in `v`: one string or a list.
fn strings(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(xs)) => xs
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// The nested architecture of part `name`, when its block is `architecture`.
fn nested_of(doc: &Value, name: &str, body: &Map<String, Value>) -> Option<Nested> {
    if body.get("block").and_then(Value::as_str) != Some("architecture") {
        return None;
    }
    let inner = body.get("inner").filter(|v| v.is_object());
    let ports: Vec<String> = match inner {
        Some(i) => strings(i.get("border"))
            .into_iter()
            .map(|b| format!("{name}.{b}"))
            .collect(),
        None => {
            let mut seen: Vec<String> = Vec::new();
            let links = doc.get("links").and_then(Value::as_array);
            let ends = links.into_iter().flatten().flat_map(|l| {
                strings(l.get("from"))
                    .into_iter()
                    .chain(strings(l.get("to")))
            });
            for end in ends.chain(strings(doc.get("border"))) {
                if end != name && lands_on(&end, name) && !seen.contains(&end) {
                    seen.push(end);
                }
            }
            seen
        }
    };
    Some(Nested {
        name: inner.and_then(|i| text(i.get("architecture"))),
        parts: inner.and_then(|i| i.get("parts")?.as_object().map(Map::len)),
        reference: text(body.get("ref")),
        digest: text(body.get("digest")),
        ports,
    })
}

impl Architecture {
    pub fn read(doc: &Value) -> Self {
        let empty = Map::new();
        let parts_map = doc
            .get("parts")
            .and_then(Value::as_object)
            .unwrap_or(&empty);
        let mut parts = Vec::new();
        for (name, body) in parts_map {
            let body_map = body.as_object().unwrap_or(&empty);
            let mut props = Vec::new();
            let mut places = Vec::new();
            for (k, v) in body_map {
                if k == "inner" {
                    continue;
                }
                places_in(&format!("parts.{name}"), k, v, &mut places);
                if !SPECIAL.contains(&k.as_str()) && marked(v).is_none() {
                    props.push((k.clone(), fmt::value(v)));
                }
            }
            parts.push(Part {
                name: name.clone(),
                block: text(body_map.get("block")).unwrap_or_else(|| "part".to_string()),
                kind: text(body_map.get("kind")),
                count: text(body_map.get("count")),
                per: text(body_map.get("per")),
                acts_for: text(body_map.get("acts_for")),
                props,
                places,
                nested: nested_of(doc, name, body_map),
            });
        }
        let links = doc
            .get("links")
            .and_then(Value::as_array)
            .map(|xs| {
                xs.iter()
                    .filter_map(|l| {
                        Some(Link {
                            from: text(l.get("from"))?,
                            to: text(l.get("to"))?,
                            carries: text(l.get("carries")).unwrap_or_else(|| "link".to_string()),
                            over: text(l.get("over")),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let border = match doc.get("border") {
            Some(Value::Array(xs)) => xs.iter().filter_map(|x| text(Some(x))).collect(),
            Some(Value::String(s)) => vec![s.clone()],
            _ => Vec::new(),
        };
        let mut policies = Vec::new();
        let mut policy_places = Vec::new();
        if let Some(o) = doc.get("policies").and_then(Value::as_object) {
            for (k, v) in o {
                places_in("policies", k, v, &mut policy_places);
                if marked(v).is_none() {
                    policies.push((k.clone(), fmt::value(v)));
                }
            }
        }
        Architecture {
            name: text(doc.get("architecture")).unwrap_or_else(|| "architecture".to_string()),
            summary: text(doc.get("summary")).unwrap_or_default(),
            format: text(doc.get("format")).unwrap_or_default(),
            parts,
            links,
            border,
            policies,
            policy_places,
        }
    }

    pub fn part(&self, name: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.name == name)
    }

    pub fn places(&self) -> impl Iterator<Item = &Place> {
        self.parts
            .iter()
            .flat_map(|p| p.places.iter())
            .chain(self.policy_places.iter())
    }
}

/// One evaluated point: baseline, best or a variant.
#[derive(Default)]
pub struct Run {
    pub id: Option<String>,
    pub params: Vec<(String, Value)>,
    pub kpis: Vec<(String, f64)>,
    pub per_part: Vec<(String, Vec<(String, f64)>)>,
    pub fitness: Option<f64>,
    pub feasible: Option<bool>,
}

impl Run {
    fn read(v: &Value, kinds: &mut Vec<(String, String)>) -> Self {
        let pairs = |key: &str| -> Vec<(String, Value)> {
            v.get(key)
                .and_then(Value::as_object)
                .map(|o| o.iter().map(|(k, x)| (k.clone(), x.clone())).collect())
                .unwrap_or_default()
        };
        let mut kpis = Vec::new();
        for (k, x) in pairs("kpis") {
            let n = x
                .as_f64()
                .or_else(|| x.get("value").and_then(Value::as_f64));
            if let Some(kind) = x.get("kind").and_then(Value::as_str)
                && !kinds.iter().any(|(n, _)| n == &k)
            {
                kinds.push((k.clone(), kind.to_string()));
            }
            if let Some(n) = n {
                kpis.push((k, n));
            }
        }
        let per_part = pairs("per_part")
            .into_iter()
            .map(|(part, m)| {
                let figures = m
                    .as_object()
                    .map(|o| {
                        o.iter()
                            .filter_map(|(k, x)| {
                                let n = x.as_f64().or_else(|| x.get("value")?.as_f64())?;
                                Some((k.clone(), n))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (part, figures)
            })
            .collect();
        Run {
            id: v.get("id").or_else(|| v.get("variant")).map(|x| {
                x.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| x.to_string())
            }),
            params: pairs("params"),
            kpis,
            per_part,
            fitness: v.get("fitness").and_then(Value::as_f64),
            feasible: v.get("feasible").and_then(Value::as_bool),
        }
    }

    pub fn kpi(&self, name: &str) -> Option<f64> {
        self.kpis.iter().find(|(k, _)| k == name).map(|(_, v)| *v)
    }

    /// The parameter at a marked place, matching `parts.a.b`, `a.b` and `/parts/a/b`.
    pub fn param(&self, path: &str) -> Option<&Value> {
        let want = norm(path);
        self.params
            .iter()
            .find(|(k, _)| norm(k) == want)
            .map(|(_, v)| v)
    }

    pub fn part_figures(&self, part: &str) -> &[(String, f64)] {
        self.per_part
            .iter()
            .find(|(p, _)| p == part)
            .map(|(_, f)| f.as_slice())
            .unwrap_or(&[])
    }
}

/// A parameter path without its `parts.` prefix, with `.` as separator.
pub fn norm(path: &str) -> String {
    let p = path.trim_start_matches('/').replace('/', ".");
    p.strip_prefix("parts.").map(str::to_string).unwrap_or(p)
}

pub struct Trace {
    pub eval: f64,
    pub variant: Option<String>,
    pub fitness: Option<f64>,
    pub best_so_far: Option<f64>,
}

pub struct Event {
    pub t: f64,
    pub part: String,
    pub kind: String,
}

pub struct Marked {
    pub path: String,
    pub value: Value,
    pub range: Value,
}

#[derive(Default)]
pub struct Outcome {
    pub marked: Vec<Marked>,
    pub baseline: Option<Run>,
    pub best: Option<Run>,
    pub variants: Vec<Run>,
    pub trace: Vec<Trace>,
    pub timeline: Vec<Event>,
    /// KPI name to `Correctness`, `Completeness` or `Quality`.
    pub kinds: Vec<(String, String)>,
    /// KPI name to the sign of its weight: positive means higher is better.
    pub directions: Vec<(String, f64)>,
    /// The weights of the fitness, `fitness = Σ weight × KPI`.
    pub weights: Vec<(String, f64)>,
    /// The KPIs that must be 0 for a variant to be feasible.
    pub gates: Vec<String>,
    /// `fit` or `exhaustive`, when the result names it.
    pub mode: Option<String>,
}

/// A marked place and every value of its range, in range order.
pub struct Axis {
    pub path: String,
    pub values: Vec<Value>,
}

/// The values of a search range: a list as written, `{min, max, step}` enumerated.
fn range_values(range: &Value) -> Vec<Value> {
    match range {
        Value::Array(xs) => xs.clone(),
        Value::Object(o) => {
            let (Some(lo), Some(hi)) = (
                o.get("min").and_then(Value::as_f64),
                o.get("max").and_then(Value::as_f64),
            ) else {
                return Vec::new();
            };
            let step = o
                .get("step")
                .and_then(Value::as_f64)
                .filter(|s| *s > 0.0)
                .unwrap_or(1.0);
            let mut out = Vec::new();
            let mut v = lo;
            while v <= hi + 1e-9 && out.len() < 200 {
                out.push(serde_json::json!(v));
                v += step;
            }
            out
        }
        _ => Vec::new(),
    }
}

/// The search settings, wherever the result keeps them.
fn search_field<'a>(doc: &'a Value, key: &str) -> Option<&'a Value> {
    doc.get(key)
        .or_else(|| doc.get("search").and_then(|s| s.get(key)))
        .or_else(|| {
            doc.get("experiment")
                .and_then(|e| e.get("search"))
                .and_then(|s| s.get(key))
        })
}

impl Outcome {
    pub fn read(doc: &Value) -> Self {
        let mut kinds: Vec<(String, String)> = Vec::new();
        for key in ["kpi_kinds", "kinds"] {
            if let Some(o) = doc.get(key).and_then(Value::as_object) {
                for (k, v) in o {
                    if let Some(s) = v.as_str() {
                        kinds.push((k.clone(), s.to_string()));
                    }
                }
            }
        }
        let weights: Vec<(String, f64)> = search_field(doc, "weights")
            .and_then(Value::as_object)
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_f64()?)))
                    .collect()
            })
            .unwrap_or_default();
        let directions = weights
            .iter()
            .map(|(k, w)| (k.clone(), w.signum()))
            .collect();
        let gates = search_field(doc, "gates")
            .and_then(Value::as_array)
            .map(|xs| {
                xs.iter()
                    .filter_map(|g| g.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let mode = search_field(doc, "mode")
            .and_then(Value::as_str)
            .map(str::to_string);
        let baseline = doc.get("baseline").map(|v| Run::read(v, &mut kinds));
        let best = doc.get("best").map(|v| Run::read(v, &mut kinds));
        let list = |key: &str| {
            doc.get(key)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        };
        let variants = list("variants")
            .iter()
            .map(|v| Run::read(v, &mut kinds))
            .collect();
        let trace = list("trace")
            .iter()
            .enumerate()
            .map(|(i, t)| Trace {
                eval: t
                    .get("eval")
                    .and_then(Value::as_f64)
                    .unwrap_or(i as f64 + 1.0),
                variant: t.get("variant").map(|x| {
                    x.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| x.to_string())
                }),
                fitness: t.get("fitness").and_then(Value::as_f64),
                best_so_far: t.get("best_so_far").and_then(Value::as_f64),
            })
            .collect();
        let timeline = list("timeline")
            .iter()
            .filter_map(|e| {
                Some(Event {
                    t: e.get("t_seconds")?.as_f64()?,
                    part: text(e.get("part")).unwrap_or_default(),
                    kind: text(e.get("kind")).unwrap_or_default(),
                })
            })
            .collect();
        let marked = list("marked")
            .iter()
            .filter_map(|m| {
                Some(Marked {
                    path: text(m.get("path"))?,
                    value: m.get("value").cloned().unwrap_or(Value::Null),
                    range: m.get("range").cloned().unwrap_or(Value::Null),
                })
            })
            .collect();
        Outcome {
            marked,
            baseline,
            best,
            variants,
            trace,
            timeline,
            kinds,
            directions,
            weights,
            gates,
            mode,
        }
    }

    /// The marked places of the search with their values: from the result's `marked`, then the
    /// architecture's places, each extended by values only the variants hold.
    pub fn axes(&self, arch: &Architecture) -> Vec<Axis> {
        let mut axes: Vec<Axis> = self
            .marked
            .iter()
            .map(|m| Axis {
                path: m.path.clone(),
                values: range_values(&m.range),
            })
            .collect();
        for p in arch.places() {
            if !axes.iter().any(|a| norm(&a.path) == norm(&p.path)) {
                axes.push(Axis {
                    path: p.path.clone(),
                    values: range_values(&p.vary),
                });
            }
        }
        for a in &mut axes {
            for v in &self.variants {
                if let Some(x) = v.param(&a.path)
                    && !a.values.iter().any(|y| fmt::value(y) == fmt::value(x))
                {
                    a.values.push(x.clone());
                }
            }
        }
        axes.retain(|a| !a.values.is_empty());
        axes
    }

    pub fn weight(&self, kpi: &str) -> Option<f64> {
        self.weights.iter().find(|(k, _)| k == kpi).map(|(_, w)| *w)
    }

    pub fn kind(&self, kpi: &str) -> Option<&str> {
        self.kinds
            .iter()
            .find(|(k, _)| k == kpi)
            .map(|(_, v)| v.as_str())
    }

    pub fn direction(&self, kpi: &str) -> Option<f64> {
        self.directions
            .iter()
            .find(|(k, _)| k == kpi)
            .map(|(_, v)| *v)
    }

    /// Baseline and best value at a marked place.
    pub fn at(&self, place: &Place) -> (Option<&Value>, Option<&Value>) {
        let marked = self
            .marked
            .iter()
            .find(|m| norm(&m.path) == norm(&place.path))
            .map(|m| &m.value);
        let base = self
            .baseline
            .as_ref()
            .and_then(|b| b.param(&place.path))
            .or(marked);
        let best = self.best.as_ref().and_then(|b| b.param(&place.path));
        (base, best)
    }
}
