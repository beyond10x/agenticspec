//! The default control plane of one architecture as the runtime interface sees it: what
//! `agenticspec ui serve` answers and what `agenticspec ui conform --architecture` checks a
//! running server for. It is read from the files `agenticspec ui build` writes for the
//! architecture (`views.json`, the layout, and `contract.json`), so the server, the check and the
//! operator app read one derivation.
//!
//! - the scopes: the architecture and every part of every level of the layout, by its path;
//! - the R1 reads of each part: each panel of its page with the parameters the layout binds;
//! - the views, commands and feeds of the contract's `operator` ingress, with the reads and the
//!   actions of the layout and the R3 views;
//! - the channels: the channel of each level of the layout;
//! - the declared shape of each view: its row key and its fields, from the panels document.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::derive::model::Panels;
use crate::interface;

/// One R1 read of a part's page: a view and the parameters the layout binds for it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Read {
    pub view: String,
    pub params: BTreeMap<String, String>,
}

/// A part of the architecture: its path (`team.lead`), its copies and the reads of its page.
#[derive(Debug, Clone)]
pub struct Part {
    pub path: String,
    pub count: u64,
    pub reads: Vec<Read>,
}

/// The declared shape of a view's rows: the key, which is the first field, and every field.
#[derive(Debug, Clone)]
pub struct Shape {
    pub key: String,
    pub fields: BTreeSet<String>,
}

/// The default control plane of one architecture.
#[derive(Debug, Clone)]
pub struct Plane {
    /// The architecture id of the scopes, `{root}/scopes/{architecture}/...`.
    pub architecture: String,
    pub parts: Vec<Part>,
    /// The channel of each level, such as `level`.
    pub channels: Vec<String>,
    /// Every view the plane answers.
    pub views: BTreeSet<String>,
    /// Every command the plane accepts: the ingress's and the layout's actions.
    pub commands: BTreeSet<String>,
    /// The events the ingress carries on its feeds; empty when it names none.
    pub feeds: BTreeSet<String>,
    pub shapes: BTreeMap<String, Shape>,
}

fn file(files: &[(String, Vec<u8>)], name: &str) -> Result<Value, String> {
    let bytes = files
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, b)| b)
        .ok_or_else(|| format!("the operator UI has no {name}"))?;
    serde_json::from_slice(bytes).map_err(|e| format!("{name}: {e}"))
}

fn text(v: &Value) -> String {
    v.as_str().map_or_else(|| v.to_string(), String::from)
}

impl Plane {
    /// The plane of the files `agenticspec ui build` wrote (see [`crate::operator::build_with`]).
    ///
    /// # Errors
    ///
    /// A file is missing or is not the JSON the build writes.
    pub fn from_files(files: &[(String, Vec<u8>)]) -> Result<Plane, String> {
        let layout = file(files, "views.json")?;
        let contract = file(files, "contract.json")?;
        Plane::derive(&layout, &contract)
    }

    /// The plane of a layout (`agenticspec-ui-layout/1`) and a contract (`agenticspec-contract/1`).
    ///
    /// # Errors
    ///
    /// The layout names no architecture, or the embedded panels document cannot be read.
    pub fn derive(layout: &Value, contract: &Value) -> Result<Plane, String> {
        let architecture = layout["architecture"]
            .as_str()
            .filter(|a| !a.is_empty())
            .ok_or("views.json names no architecture")?
            .to_string();
        let mut parts: Vec<Part> = Vec::new();
        let mut channels = Vec::new();
        let mut views: BTreeSet<String> = interface::r3_views().into_iter().collect();
        let mut commands = BTreeSet::new();
        for level in layout["levels"].as_array().into_iter().flatten() {
            if let Some(c) = level["channel"].as_str()
                && !channels.iter().any(|x| x == c)
            {
                channels.push(c.to_string());
            }
            let mut paths = BTreeMap::new();
            for p in level["parts"].as_array().into_iter().flatten() {
                let (Some(name), Some(path)) = (p["name"].as_str(), p["path"].as_str()) else {
                    continue;
                };
                paths.insert(name.to_string(), path.to_string());
                if !parts.iter().any(|x| x.path == path) {
                    parts.push(Part {
                        path: path.to_string(),
                        count: p["count"].as_u64().unwrap_or(1),
                        reads: Vec::new(),
                    });
                }
            }
            for page in level["pages"].as_array().into_iter().flatten() {
                for a in page["actions"].as_array().into_iter().flatten() {
                    if let Some(c) = a["command"].as_str() {
                        commands.insert(c.to_string());
                    }
                }
                for frame in page["frames"].as_array().into_iter().flatten() {
                    for s in frame["sections"].as_array().into_iter().flatten() {
                        for a in s["row_actions"].as_array().into_iter().flatten() {
                            if let Some(c) = a["command"].as_str() {
                                commands.insert(c.to_string());
                            }
                        }
                        let (Some(view), Some(name)) = (s["widget"].as_str(), s["part"].as_str())
                        else {
                            continue;
                        };
                        views.insert(view.to_string());
                        let path = paths.get(name).cloned().unwrap_or_else(|| name.to_string());
                        let params = s["args"]
                            .as_object()
                            .into_iter()
                            .flatten()
                            .filter(|(k, _)| k.as_str() != "channel")
                            .map(|(k, v)| (k.clone(), text(v)))
                            .collect();
                        let read = Read {
                            view: view.to_string(),
                            params,
                        };
                        match parts.iter_mut().find(|p| p.path == path) {
                            Some(p) if !p.reads.contains(&read) => p.reads.push(read),
                            Some(_) => {}
                            None => parts.push(Part {
                                path,
                                count: 1,
                                reads: vec![read],
                            }),
                        }
                    }
                }
            }
        }
        let mut feeds = BTreeSet::new();
        let operator = contract["ingress"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|i| i["name"].as_str() == Some("operator"));
        for op in operator
            .and_then(|i| i["operations"].as_array())
            .into_iter()
            .flatten()
        {
            let Some(name) = op["name"].as_str() else {
                continue;
            };
            match op["kind"].as_str() {
                Some("command") => {
                    commands.insert(name.to_string());
                }
                Some("view") => {
                    views.insert(name.to_string());
                }
                Some("feed") => {
                    feeds.insert(name.to_string());
                }
                _ => {}
            }
        }
        let panels = Panels::read(crate::operator::PANELS)?;
        let shapes = panels
            .list
            .iter()
            .map(|p| {
                let fields = p
                    .columns
                    .iter()
                    .filter_map(|c| c.as_str().or_else(|| c["field"].as_str()))
                    .map(String::from)
                    .collect();
                (
                    p.widget.clone(),
                    Shape {
                        key: p.key.clone(),
                        fields,
                    },
                )
            })
            .collect();
        Ok(Plane {
            architecture,
            parts,
            channels,
            views,
            commands,
            feeds,
            shapes,
        })
    }

    /// The part of a scope's part segment, by its path.
    pub fn part(&self, path: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.path == path)
    }
}
