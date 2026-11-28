//! The UI of an ingress (`design/ingress.md` I10): an operator UI is the `ui` export of an
//! ingress, so an architecture whose contract gives a `ui` export to an ingress other than
//! `operator` gets one composition per such ingress, each scoped to that ingress's interface:
//!
//! - its pages show only the panels of the interface's views; a part page left without a panel
//!   is dropped, with its entry in the navigation;
//! - its actions are the interface's commands: an action the derivation places is kept when its
//!   command is one of them, and a command of the interface no action carries yet becomes a row
//!   action on each panel of its domain whose row key is one of its inputs (as the control plane
//!   places the administration commands), or else a header action on the first page that shows
//!   a panel of its domain;
//! - its channels carry only the interface's feeds.
//!
//! An architecture whose only `ui` export is the operator ingress's keeps its one operator UI,
//! byte for byte: it is not scoped.

use std::collections::BTreeSet;

use serde_json::Value;

use super::model::{Model, Panels};
use super::{Action, Entry, LevelPlan, row_actions, snake};
use crate::Refusal;

/// The ingress of the default control plane, whose UI is the operator UI.
pub const OPERATOR: &str = super::control::OPERATOR;
/// The protocol of the export that gives an ingress a UI.
pub const UI_EXPORT: &str = "ui";

/// An ingress with a `ui` export: its name and the operations of its interface.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ingress {
    pub name: String,
    /// The commands, in the contract's order.
    pub commands: Vec<String>,
    pub views: BTreeSet<String>,
    pub feeds: BTreeSet<String>,
}

/// The ingresses of a contract (`agenticspec-contract/1`) with a `ui` export, in its order.
pub fn ui_ingresses(contract: &Value) -> Vec<Ingress> {
    let mut out = Vec::new();
    for i in contract["ingress"].as_array().into_iter().flatten() {
        let ui = i["exports"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|e| e.as_str() == Some(UI_EXPORT));
        let Some(name) = i["name"].as_str().filter(|_| ui) else {
            continue;
        };
        let mut ingress = Ingress {
            name: name.to_string(),
            ..Ingress::default()
        };
        for op in i["operations"].as_array().into_iter().flatten() {
            let Some(n) = op["name"].as_str() else {
                continue;
            };
            match op["kind"].as_str() {
                Some("command") => ingress.commands.push(n.to_string()),
                Some("view") => {
                    ingress.views.insert(n.to_string());
                }
                Some("feed") => {
                    ingress.feeds.insert(n.to_string());
                }
                _ => {}
            }
        }
        out.push(ingress);
    }
    out
}

/// Whether the contract gives a UI per ingress: an ingress other than `operator` has a `ui`
/// export.
pub fn per_ingress(contract: &Value) -> bool {
    ui_ingresses(contract).iter().any(|i| i.name != OPERATOR)
}

fn refusal(message: String) -> Refusal {
    Refusal {
        path: "contract/ingress".into(),
        message,
    }
}

/// The ingress the UI named `name` (`operator` when none is named) is scoped to: none for the
/// operator UI of a contract that gives no other ingress a UI (or of no contract), which is
/// derived unscoped.
///
/// # Errors
///
/// The named ingress has no `ui` export in the contract, or an ingress other than `operator` is
/// named without a contract.
pub fn scope(contract: Option<&Value>, name: Option<&str>) -> Result<Option<Ingress>, Refusal> {
    let name = name.unwrap_or(OPERATOR);
    let Some(contract) = contract else {
        if name == OPERATOR {
            return Ok(None);
        }
        return Err(refusal(format!(
            "the UI of ingress {name} is derived from the contract: name it"
        )));
    };
    let uis = ui_ingresses(contract);
    if !per_ingress(contract) && name == OPERATOR {
        return Ok(None);
    }
    if let Some(i) = uis.iter().find(|i| i.name == name) {
        return Ok(Some(i.clone()));
    }
    let names: Vec<&str> = uis.iter().map(|i| i.name.as_str()).collect();
    Err(refusal(format!(
        "no ingress {name} with a ui export; the contract gives a UI to: {}",
        if names.is_empty() {
            "none".to_string()
        } else {
            names.join(", ")
        }
    )))
}

fn action(model: &Model<'_>, command: &str) -> Action {
    Action {
        name: snake(command.rsplit('.').next().unwrap_or(command)),
        command: command.to_string(),
        label: model.command_label(command),
        bind: Vec::new(),
    }
}

/// Scopes the levels of a composition to the interface of `ingress` (see the module).
pub(crate) fn apply(
    levels: &mut [LevelPlan],
    ingress: &Ingress,
    model: &Model<'_>,
    panels: &Panels,
) {
    let allowed: BTreeSet<&str> = ingress.commands.iter().map(String::as_str).collect();
    for lp in levels.iter_mut() {
        for page in &mut lp.pages {
            page.actions
                .retain(|a| allowed.contains(a.command.as_str()));
            for frame in &mut page.frames {
                frame.sections.retain(|s| ingress.views.contains(&s.widget));
                for s in &mut frame.sections {
                    s.row_actions
                        .retain(|a| allowed.contains(a.command.as_str()));
                }
            }
            page.frames.retain(|f| !f.sections.is_empty());
        }
    }
    // The commands of the interface no action carries yet.
    let placed: BTreeSet<String> = levels
        .iter()
        .flat_map(|lp| &lp.pages)
        .flat_map(|p| {
            p.actions.iter().chain(
                p.frames
                    .iter()
                    .flat_map(|f| &f.sections)
                    .flat_map(|s| &s.row_actions),
            )
        })
        .map(|a| a.command.clone())
        .collect();
    for command in ingress
        .commands
        .iter()
        .filter(|c| !placed.contains(*c) && model.has_command(c))
    {
        let mut done = false;
        for lp in levels.iter_mut() {
            let hidden = lp.key.hide_actions.clone();
            for page in &mut lp.pages {
                let widgets: Vec<String> = page
                    .frames
                    .iter()
                    .flat_map(|f| &f.sections)
                    .map(|s| s.widget.clone())
                    .collect();
                let fits = row_actions(model, panels, std::slice::from_ref(command), &widgets);
                for s in page.frames.iter_mut().flat_map(|f| &mut f.sections) {
                    for a in fits.get(&s.widget).into_iter().flatten() {
                        done = true;
                        if !hidden.contains(&a.name) {
                            s.row_actions.push(a.clone());
                        }
                    }
                }
            }
        }
        if done {
            continue;
        }
        let domain = command.rsplit_once('.').map_or("", |(d, _)| d);
        'levels: for lp in levels.iter_mut() {
            for page in &mut lp.pages {
                let shows = page
                    .frames
                    .iter()
                    .flat_map(|f| &f.sections)
                    .any(|s| panels.get(&s.widget).is_some_and(|p| p.domain == domain));
                if shows {
                    let a = action(model, command);
                    if !lp.key.hide_actions.contains(&a.name) {
                        page.actions.push(a);
                    }
                    break 'levels;
                }
            }
        }
    }
    for lp in levels.iter_mut() {
        // A frame of a domain with a row action is the part's work to do: it opens unfolded.
        for f in lp.pages.iter_mut().flat_map(|p| &mut p.frames) {
            if f.title.is_some() && f.area != "area_per" {
                f.collapsed = !f.sections.iter().any(|s| !s.row_actions.is_empty());
            }
        }
        lp.pages.retain(|p| !p.frames.is_empty());
        let kept: BTreeSet<String> = lp.pages.iter().map(|p| p.page.clone()).collect();
        for (_, _, entries) in &mut lp.sections {
            entries.retain(|e| match e {
                Entry::Page(p) => kept.contains(p),
                Entry::Nested(_) => true,
            });
        }
        lp.sections.retain(|(_, _, entries)| !entries.is_empty());
        let mut events = BTreeSet::new();
        for s in lp
            .pages
            .iter()
            .flat_map(|p| &p.frames)
            .flat_map(|f| &f.sections)
        {
            if let Some(p) = panels.get(&s.widget) {
                events.extend(
                    p.events
                        .iter()
                        .filter(|e| ingress.feeds.contains(*e))
                        .cloned(),
                );
            }
        }
        if events.is_empty() {
            lp.channel = None;
        }
        lp.events = events;
    }
}
