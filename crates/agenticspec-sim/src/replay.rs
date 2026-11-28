//! A simulated run as the runtime interface answers it, for `agenticspec ui serve --sim`: the
//! architecture run once at the values it is built with, its event log ([`crate::event_log`]) and
//! the state that log leaves in the views of the building blocks.
//!
//! The state is read from the blocks built into the binary ([`crate::rows::SOURCES`]): an event's
//! key field names its entity (the entity whose identity is that field, of the event's own domain
//! first); the command outcome that emits the event says what it does to the entity: `creates`
//! puts it in the initial state of its lifecycle, `moves` (also under `affects`) in the state its
//! transition goes to, `updates` leaves its state. A view over the entity answers one row per
//! entity, its identity first and its `state` when the view declares one and the run decided it,
//! kept by the view's `state ==` and `state !=` filters; a view whose first field is not the
//! entity's identity answers none. The simulator knows no other field of an entity, so a row holds
//! no other.
//!
//! Writing the replay changes no result: it reads a recorded run and draws nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use agenticspec_ui::interface::Ordered;
use agenticspec_ui::serve::{At, Replay, ReplayEvent, ReplayRow};
use serde_json::{Value, json};

use crate::event_log::Event;

/// A filter term of a view on the state of its entity.
#[derive(Debug, Clone, PartialEq)]
enum StateTerm {
    Is(String),
    Not(String),
}

#[derive(Debug, Clone)]
struct View {
    name: String,
    source: String,
    fields: Vec<String>,
    state: Vec<StateTerm>,
}

/// What the replay reads of the blocks.
#[derive(Debug, Default)]
struct Model {
    /// Each entity's identity field.
    identity: BTreeMap<String, String>,
    /// What each event does, by the outcomes that emit it in the order of the blocks: the entity
    /// and the state it leaves, `None` for one that leaves the state.
    emitted: BTreeMap<String, Vec<(String, Option<String>)>>,
    views: Vec<View>,
    /// The domains of the blocks' events, the block channels.
    domains: BTreeSet<String>,
}

fn domain(name: &str) -> Option<&str> {
    name.split('.').nth(1)
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(String::from))
        .collect()
}

/// The `state == X` and `state != X` terms of a view's filter: one term, or the terms of `all`.
fn state_terms(filter: &Value) -> Vec<StateTerm> {
    let terms: Vec<String> = match filter {
        Value::String(s) => vec![s.clone()],
        Value::Object(_) => strings(&filter["all"]),
        _ => Vec::new(),
    };
    terms
        .iter()
        .filter_map(|t| {
            let t = t.trim();
            if let Some(v) = t.strip_prefix("state ==") {
                Some(StateTerm::Is(v.trim().to_string()))
            } else {
                t.strip_prefix("state !=")
                    .map(|v| StateTerm::Not(v.trim().to_string()))
            }
        })
        .collect()
}

impl Model {
    fn read<'a>(docs: impl IntoIterator<Item = &'a str>) -> Result<Model, String> {
        let docs: Vec<Value> = docs
            .into_iter()
            .map(|t| serde_yaml_ng::from_str(t).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let mut m = Model::default();
        // Each entity's initial state and the state each transition goes to.
        let mut initial: BTreeMap<String, String> = BTreeMap::new();
        let mut to: BTreeMap<(String, String), String> = BTreeMap::new();
        for doc in &docs {
            for e in doc["entities"].as_array().into_iter().flatten() {
                let Some(name) = e["name"].as_str() else {
                    continue;
                };
                if let Some(id) = e["identity"]["name"].as_str() {
                    m.identity.insert(name.to_string(), id.to_string());
                }
                let life = &e["lifecycle"];
                if let Some(s) = life["initial"].as_str() {
                    initial.insert(name.to_string(), s.to_string());
                }
                for t in life["transitions"].as_array().into_iter().flatten() {
                    if let (Some(t_name), Some(state)) = (t["name"].as_str(), t["to"].as_str()) {
                        to.insert((name.to_string(), t_name.to_string()), state.to_string());
                    }
                }
            }
            for e in doc["events"].as_array().into_iter().flatten() {
                if let Some(d) = e["name"].as_str().and_then(domain) {
                    m.domains.insert(d.to_string());
                }
            }
            for v in doc["views"].as_array().into_iter().flatten() {
                let (Some(name), Some(source)) = (v["name"].as_str(), v["source"].as_str()) else {
                    continue;
                };
                let fields = v["fields"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|f| f["name"].as_str().map(String::from))
                    .collect();
                m.views.push(View {
                    name: name.to_string(),
                    source: source.to_string(),
                    fields,
                    state: state_terms(&v["filter"]),
                });
            }
        }
        let moved = |path: &str| -> Option<(String, Option<String>)> {
            let (entity, t) = path.rsplit_once('.')?;
            let state = to.get(&(entity.to_string(), t.to_string())).cloned();
            Some((entity.to_string(), state))
        };
        for doc in &docs {
            for c in doc["commands"].as_array().into_iter().flatten() {
                for o in c["outcomes"].as_array().into_iter().flatten() {
                    let mut effects: Vec<(String, Option<String>)> = Vec::new();
                    if let Some(x) = o["creates"].as_str() {
                        effects.push((x.to_string(), initial.get(x).cloned()));
                    }
                    if let Some(x) = o["moves"].as_str().and_then(moved) {
                        effects.push(x);
                    }
                    if let Some(x) = o["updates"].as_str() {
                        effects.push((x.to_string(), None));
                    }
                    for a in o["affects"].as_array().into_iter().flatten() {
                        if let Some(x) = a["moves"].as_str().and_then(moved) {
                            effects.push(x);
                        }
                    }
                    for event in strings(&o["emits"]) {
                        m.emitted
                            .entry(event)
                            .or_default()
                            .extend(effects.iter().cloned());
                    }
                }
            }
        }
        Ok(m)
    }

    /// The entity an event of the log is about, and the state it leaves the entity in.
    fn effect(&self, e: &Event) -> Option<(String, Option<String>)> {
        let keyed: Vec<&String> = self
            .identity
            .iter()
            .filter(|(_, id)| id.as_str() == e.key)
            .map(|(name, _)| name)
            .collect();
        let entity = keyed
            .iter()
            .find(|n| domain(n) == domain(e.event))
            .or_else(|| keyed.first())?;
        let state = self
            .emitted
            .get(e.event)
            .into_iter()
            .flatten()
            .filter(|(x, _)| x == *entity)
            .find_map(|(_, s)| s.clone());
        Some(((*entity).clone(), state))
    }
}

/// One entity the run's events touched.
struct Entity {
    id: String,
    state: Option<String>,
    at: Vec<At>,
}

fn at_of(e: &Event) -> At {
    At {
        part: e.part.clone(),
        instance: e.instance.and_then(|c| u32::try_from(c + 1).ok()),
    }
}

/// The replay of an event log against block documents `docs` (their YAML texts).
///
/// # Errors
///
/// A block document is not YAML.
pub fn replay_in<'a>(
    docs: impl IntoIterator<Item = &'a str>,
    events: &[Event],
) -> Result<Replay, String> {
    let model = Model::read(docs)?;
    // The entities of each block entity, in the order the log first names them.
    let mut entities: BTreeMap<String, Vec<Entity>> = BTreeMap::new();
    for e in events {
        let Some((entity, state)) = model.effect(e) else {
            continue;
        };
        let list = entities.entry(entity).or_default();
        let i = match list.iter().position(|x| x.id == e.subject_key) {
            Some(i) => i,
            None => {
                list.push(Entity {
                    id: e.subject_key.clone(),
                    state: None,
                    at: Vec::new(),
                });
                list.len() - 1
            }
        };
        let x = &mut list[i];
        if state.is_some() {
            x.state = state;
        }
        let at = at_of(e);
        if !x.at.contains(&at) {
            x.at.push(at);
        }
    }
    let mut views = BTreeMap::new();
    for v in &model.views {
        let (Some(list), Some(id)) = (entities.get(&v.source), model.identity.get(&v.source))
        else {
            continue;
        };
        if v.fields.first() != Some(id) {
            continue;
        }
        let keeps = |state: &Option<String>| {
            v.state.iter().all(|t| match t {
                StateTerm::Is(s) => state.as_deref() == Some(s.as_str()),
                StateTerm::Not(s) => state.as_deref() != Some(s.as_str()),
            })
        };
        let rows: Vec<ReplayRow> = list
            .iter()
            .filter(|x| keeps(&x.state))
            .map(|x| {
                let mut row = vec![(id.clone(), json!(x.id))];
                if let Some(s) = x
                    .state
                    .as_ref()
                    .filter(|_| v.fields.iter().any(|f| f == "state"))
                {
                    row.push(("state".to_string(), json!(s)));
                }
                ReplayRow {
                    row: Ordered(row),
                    at: x.at.clone(),
                }
            })
            .collect();
        views.insert(v.name.clone(), rows);
    }
    let events = events
        .iter()
        .map(|e| ReplayEvent {
            seq: e.seq,
            event: e.event.to_string(),
            payload: json!({ e.key: e.subject_key }),
            at: at_of(e),
        })
        .collect();
    Ok(Replay {
        views,
        events,
        channels: model.domains,
    })
}

/// The replay of an event log against the blocks built into the binary.
///
/// # Errors
///
/// A block built in is not YAML.
pub fn replay(events: &[Event]) -> Result<Replay, String> {
    replay_in(crate::rows::SOURCES.iter().map(|(_, t)| *t), events)
}

/// A run of an architecture for `ui serve --sim`.
#[derive(Debug, Clone)]
pub struct Run {
    /// The architecture document as read, nested architectures resolved beside their parts: the
    /// document a simulator result names.
    pub document: Value,
    pub events: Vec<Event>,
    pub replay: Replay,
    /// The KPIs of the run, by id.
    pub kpis: BTreeMap<String, f64>,
    pub seed: u64,
    pub horizon_seconds: f64,
}

/// The seed of a run for `ui serve --sim`.
pub const SEED: u64 = 1;
/// The simulated time of a run for `ui serve --sim`: one day.
pub const HORIZON_SECONDS: f64 = 86400.0;

impl Run {
    /// A simulator result (`agenticspec-sim-result/1`) of the document with this run's KPIs as its
    /// baseline: what the Metrics page of the control-plane UI shows as the prediction of each KPI.
    pub fn result(&self) -> Value {
        let kpis: serde_json::Map<String, Value> = self
            .kpis
            .iter()
            .map(|(k, v)| (k.clone(), json!({ "value": v })))
            .collect();
        json!({
            "format": "agenticspec-sim-result/1",
            "architecture": self.document,
            "baseline": { "kpis": kpis },
        })
    }
}

/// Simulates the architecture at `architecture` once at the values it is built with, under the
/// profile at `profile`, at [`SEED`] for [`HORIZON_SECONDS`], and replays its event log: the same
/// files give the same run.
///
/// # Errors
///
/// A document cannot be read, the architecture is refused by the check, or the simulator refuses
/// the pair.
pub fn run(architecture: &Path, profile: &Path) -> Result<Run, crate::cli::Refusal> {
    let (document, built, model) = crate::cli::built_model(architecture, profile, HORIZON_SECONDS)?;
    let out = crate::sim::simulate(&model, SEED, true);
    let refused = |e: String| crate::cli::refused("simulation_refused", e);
    let events = crate::event_log::log(&built, &out.timeline).map_err(refused)?;
    let replay = replay(&events).map_err(refused)?;
    Ok(Run {
        document,
        events,
        replay,
        kpis: out.kpis,
        seed: SEED,
        horizon_seconds: HORIZON_SECONDS,
    })
}
