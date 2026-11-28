//! `agenticspec ui derive`: the operator UI of an architecture as one `ess-ui/1` composition.
//!
//! The top document is the app and its overview the navigation home. A nested architecture part
//! (`{ block: architecture, ref }`) is one overview page and one navigation section, recursively,
//! and one row in its parent's parts table. Every other part is one part page: first the panel of
//! its instances, then the panels of each domain it uses in a collapsed frame of its own, all
//! scoped to the part. A link is a row of the reach table; policies and marked places are shown
//! read-only on the overview. One channel per level carries the events of the panels on its pages.
//!
//! The domains of a part come from its contract when one is given (the blocks of the ports it
//! uses); with none, from the architecture alone: its block, a setting named like a block (an
//! agent's `harness`) and what its links carry when that names a block (`question`, `claim`).
//! Actions come from the contract and the port catalogue only: a principal port of a principal
//! part becomes a row action on the panels its intent command acts on, and a link into the border
//! from a principal part or from outside becomes a header action on the border part's page, with
//! the intent command of the link's port. Every action is visible by `actor.may(<command>)`.
//! Nothing here names a block, a view or a command.
//!
//! Panels are copied from `ui/views/blocks.ui.yaml` byte for byte; a panel that takes a row
//! action is drawn as a collection of its own with the panel's columns. The document's `view:`
//! key adjusts labels, order, what is hidden, display groups and the home page.
//!
//! Beside the document: `<name>.sources.sha256` (the architecture documents, the panels, the
//! contract and catalogue when given, and the generator's version), `<name>.fixtures/` (the
//! parts, reach and graph of each level, a script per channel, and an index that points at the
//! panels' own fixtures) and `<name>.layout.json` (the same composition for the view package).

pub mod model;
pub mod view_key;
mod write;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::Refusal;
use model::{Contract, Level, Model, Panels, Port, Tree};
use view_key::ViewKey;

/// The generator's version, a line of every digest file.
pub const VERSION: &str = concat!("agenticspec ui derive ", env!("CARGO_PKG_VERSION"));

/// The panels document every derivation copies from, relative to the root.
pub const PANELS: &str = "ui/views/blocks.ui.yaml";
/// The digest of the panels' own sources, which the derivation's digest covers.
pub const PANELS_DIGEST: &str = "ui/views/blocks.sources.sha256";
/// The scripts of the panels' channels.
pub const SCRIPTS: &str = "ui/views/fixtures/scripts";
/// The panels' fixtures.
pub const FIXTURES: &str = "ui/views/fixtures";
/// The port catalogue a derivation reads when it exists and none is named.
pub const PORTS: &str = "blocks/ports.json";

/// What `agenticspec ui derive` reads and where it writes.
pub struct Inputs {
    /// The repository root that holds `blocks/` and `ui/views/`.
    pub root: PathBuf,
    pub architecture: PathBuf,
    pub contract: Option<PathBuf>,
    pub ports: Option<PathBuf>,
    /// The document to write, `<dir>/<name>.ui.yaml`.
    pub out: PathBuf,
}

/// Why a derivation did not produce a document.
#[derive(Debug)]
pub enum Failure {
    /// An input could not be read or parsed.
    Unreadable(String),
    /// The architecture or its `view:` key is refused, each entry by its node path.
    Refused(Vec<Refusal>),
}

/// A derivation: every file by its path, and what it holds.
pub struct Derivation {
    pub files: Vec<(PathBuf, String)>,
    pub pages: usize,
    pub actions: usize,
    pub channels: usize,
    /// The directory of the fixtures, which holds nothing but generated files.
    pub fixtures_dir: PathBuf,
}

/// One action on a page: a command, the row fields it binds and its label.
#[derive(Clone, Debug)]
pub struct Action {
    pub name: String,
    pub command: String,
    pub label: String,
    pub bind: Vec<(String, String)>,
}

/// A section of a part page: a copied panel, or a panel with row actions drawn as a collection.
#[derive(Clone, Debug)]
pub struct Section {
    pub name: String,
    pub title: String,
    pub widget: String,
    /// The widget's arguments, each as an expression.
    pub args: Vec<(String, String)>,
    pub row_actions: Vec<Action>,
}

/// A frame of a part page: one area of its layout.
#[derive(Clone, Debug)]
pub struct Frame {
    pub area: String,
    /// The heading of a shared frame; none draws each section in a frame of its own.
    pub title: Option<String>,
    pub collapsed: bool,
    pub sections: Vec<Section>,
}

/// One part page.
#[derive(Clone, Debug)]
pub struct PartPage {
    pub part: String,
    pub page: String,
    pub label: String,
    pub block: String,
    pub frames: Vec<Frame>,
    pub actions: Vec<Action>,
}

/// One entry of a level's navigation: a part page, or the overview of a nested level.
#[derive(Clone, Debug)]
pub enum Entry {
    Page(String),
    Nested(usize),
}

/// One level of the composition.
pub struct LevelPlan {
    pub level: usize,
    pub overview: String,
    pub title: String,
    pub channel: Option<String>,
    pub events: BTreeSet<String>,
    /// Part names in the order of the parts table: nested first, then block, then name.
    pub order: Vec<String>,
    pub pages: Vec<PartPage>,
    /// Navigation sections of the level: name, label and entries.
    pub sections: Vec<(String, String, Vec<Entry>)>,
    pub key: ViewKey,
    pub instances: u64,
}

/// The whole composition, before it is written.
pub struct Plan<'a> {
    pub tree: &'a Tree,
    pub model: &'a Model<'a>,
    pub panels: &'a Panels,
    pub levels: Vec<LevelPlan>,
    pub home: String,
}

fn refusal(path: impl Into<String>, message: impl Into<String>) -> Refusal {
    Refusal {
        path: path.into(),
        message: message.into(),
    }
}

/// `AgentRevisions` is `agent_revisions`; a dot or a dash is `_`.
pub fn snake(s: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if prev_lower {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else if c == '.' || c == '-' || c == ' ' {
            out.push('_');
            prev_lower = false;
        } else {
            out.push(c);
            prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        }
    }
    out
}

/// `build_slots` is `BuildSlots`; `team.lead` is `TeamLead`.
pub fn pascal(s: &str) -> String {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut cs = w.chars();
            cs.next()
                .map(|c| c.to_ascii_uppercase().to_string() + cs.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn capital(s: &str) -> String {
    let mut cs = s.chars();
    cs.next()
        .map(|c| c.to_ascii_uppercase().to_string() + cs.as_str())
        .unwrap_or_default()
        .replace('_', " ")
}

/// An expression literal for a string: `'operator'`.
pub fn lit(s: &str) -> String {
    format!("'{}'", s.replace('\'', ""))
}

/// The page of a part.
pub fn part_page(path: &str) -> String {
    format!("part.{path}")
}

/// The overview page of a level.
pub fn overview_page(level: &Level) -> String {
    if level.path.is_empty() {
        "overview".into()
    } else {
        format!("{}.overview", level.path)
    }
}

/// The channel of a level.
pub fn channel_name(level: &Level) -> String {
    if level.path.is_empty() {
        "level".into()
    } else {
        format!("level_{}", snake(&level.path))
    }
}

/// The domains of a part, its own block's first, then the others in the panels' order.
fn domains_of(
    level: &Level,
    part: &model::Part,
    model: &Model<'_>,
    panels: &Panels,
    contract: Option<&Contract>,
    ports: &BTreeMap<String, Port>,
) -> Vec<String> {
    let own = model.domain_of(&part.block);
    let mut set: BTreeSet<String> = BTreeSet::new();
    match contract {
        Some(c) => {
            for port in c.ports_of(&part.name) {
                let block = ports
                    .get(port)
                    .map(|p| p.block.clone())
                    .unwrap_or_else(|| port.split('.').next().unwrap_or("").to_string());
                if let Some(d) = model.domain_of(&block) {
                    set.insert(d);
                }
            }
        }
        None => {
            for (k, v) in &part.settings {
                if let Some(d) = model.domain_of(k).filter(|_| !v.is_null()) {
                    set.insert(d);
                }
            }
            for l in &level.links {
                if (l.from == part.name || l.to == part.name)
                    && let Some(d) = model.domain_of(&l.carries)
                {
                    set.insert(d);
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Some(o) = &own {
        out.push(o.clone());
    }
    for d in panels.domains() {
        if set.contains(&d) && Some(&d) != own.as_ref() {
            out.push(d);
        }
    }
    out.retain(|d| panels.of_domain(d).next().is_some());
    out
}

/// The panel of a part's instances: the panel over the entity its block names, else the first
/// panel of its own domain.
fn instance_panel(part: &model::Part, model: &Model<'_>, panels: &Panels) -> Option<String> {
    let domain = model.domain_of(&part.block)?;
    let entity = model.entity_of(&part.block);
    let mut own = panels.of_domain(&domain);
    let first = panels.of_domain(&domain).next().map(|p| p.widget.clone());
    own.find(|p| {
        entity
            .as_deref()
            .is_some_and(|e| model.view_source(&p.widget) == Some(e))
    })
    .map(|p| p.widget.clone())
    .or(first)
}

/// The scope arguments of a panel on a part's page, its channel and its gallery arguments.
fn args_of(
    panel: &model::Panel,
    level: &Level,
    part_path: &str,
    channel: &str,
) -> Vec<(String, String)> {
    let mut out = vec![
        ("architecture".to_string(), lit(&level.name)),
        ("part".to_string(), lit(part_path)),
    ];
    if level.count > 1 {
        out.push(("instance".to_string(), "state.instance".to_string()));
    }
    if !panel.events.is_empty() {
        out.push(("channel".to_string(), channel.to_string()));
    }
    for (k, v) in &panel.args {
        if k == "architecture_id" {
            out.push((k.clone(), lit(&level.name)));
        } else {
            out.push((k.clone(), v.clone()));
        }
    }
    out
}

fn section_of(
    panel: &model::Panel,
    level: &Level,
    part_path: &str,
    channel: &str,
    prefix: &str,
) -> Section {
    let short = panel.widget.rsplit('.').next().unwrap_or(&panel.widget);
    let dom = panel.domain.rsplit('.').next().unwrap_or(&panel.domain);
    Section {
        name: format!("{prefix}{}_{}", snake(dom), snake(short)),
        title: panel.title.clone(),
        widget: panel.widget.clone(),
        args: args_of(panel, level, part_path, channel),
        row_actions: Vec::new(),
    }
}

/// The actions of one level, from the contract and the catalogue: row actions by part and
/// widget, header actions by part.
#[allow(clippy::type_complexity)]
fn actions_of(
    level: &Level,
    model: &Model<'_>,
    panels: &Panels,
    contract: Option<&Contract>,
    ports: &BTreeMap<String, Port>,
    placed: &BTreeMap<String, Vec<String>>,
) -> (
    BTreeMap<(String, String), Vec<Action>>,
    BTreeMap<String, Vec<Action>>,
) {
    let mut rows: BTreeMap<(String, String), Vec<Action>> = BTreeMap::new();
    let mut header: BTreeMap<String, Vec<Action>> = BTreeMap::new();
    let Some(c) = contract else {
        return (rows, header);
    };
    let principal = |name: &str| level.part(name).is_some_and(|p| p.block == "principal");
    for part in level.parts.iter().filter(|p| principal(&p.name)) {
        for port in c.ports_of(&part.name).filter_map(|n| ports.get(n)) {
            if port.kind != "Principal" || !port.explicit {
                continue;
            }
            for command in port.intent.iter().filter(|c| model.has_command(c)) {
                let inputs = model.command_inputs(command);
                let domain = command.rsplit_once('.').map_or("", |(d, _)| d);
                let fits: Vec<&String> = placed
                    .get(&part.name)
                    .into_iter()
                    .flatten()
                    .filter(|w| {
                        panels
                            .get(w)
                            .is_some_and(|p| p.domain == domain && inputs.contains(&p.key))
                    })
                    .collect();
                // A panel of the open rows (`OpenQuestions`, `OpenApprovals`) takes the action
                // alone when there is one: the other rows of the domain are already answered.
                let open = |w: &String| w.rsplit('.').next().is_some_and(|v| v.starts_with("Open"));
                let only_open = fits.iter().any(|w| open(w));
                for w in fits.into_iter().filter(|w| !only_open || open(w)) {
                    let Some(p) = panels.get(w) else { continue };
                    let fields: BTreeSet<String> = p
                        .columns
                        .iter()
                        .filter_map(|c| c.as_str().or_else(|| c["field"].as_str()))
                        .map(str::to_string)
                        .collect();
                    // The row binds the command's input of the panel's key; the rest is the
                    // reader's.
                    let bind = inputs
                        .iter()
                        .filter(|i| **i == p.key && fields.contains(*i))
                        .map(|i| (i.clone(), format!("row.{i}")))
                        .collect();
                    rows.entry((part.name.clone(), w.clone()))
                        .or_default()
                        .push(Action {
                            name: snake(command.rsplit('.').next().unwrap_or(command)),
                            command: command.clone(),
                            label: model.command_label(command),
                            bind,
                        });
                }
            }
        }
    }
    for l in &level.links {
        if !level.border.contains(&l.to) {
            continue;
        }
        let from_outside = level.part(&l.from).is_none();
        if !(from_outside || principal(&l.from)) {
            continue;
        }
        let Some(port) = c.links.get(&model::link_text(l)).and_then(|n| ports.get(n)) else {
            continue;
        };
        for command in port.intent.iter().filter(|c| model.has_command(c)) {
            let list = header.entry(l.to.clone()).or_default();
            if list.iter().any(|a| a.command == *command) {
                continue;
            }
            list.push(Action {
                name: snake(command.rsplit('.').next().unwrap_or(command)),
                command: command.clone(),
                label: model.command_label(command),
                bind: Vec::new(),
            });
        }
    }
    (rows, header)
}

/// The level's parts in the order of its parts table: nested architectures first, then by
/// block, then by name; the key's `order.parts` first of all.
fn table_order(level: &Level, key: &ViewKey) -> Vec<String> {
    let mut rest: Vec<&model::Part> = level.parts.iter().collect();
    rest.sort_by(|a, b| {
        (a.nested.is_none(), &a.block, &a.name).cmp(&(b.nested.is_none(), &b.block, &b.name))
    });
    let mut out: Vec<String> = key
        .order_parts
        .iter()
        .filter(|p| level.part(p).is_some())
        .cloned()
        .collect();
    for p in rest {
        if !out.contains(&p.name) {
            out.push(p.name.clone());
        }
    }
    out
}

fn plan_level(
    tree: &Tree,
    index: usize,
    model: &Model<'_>,
    panels: &Panels,
    contract: Option<&Contract>,
    ports: &BTreeMap<String, Port>,
    refusals: &mut Vec<Refusal>,
) -> LevelPlan {
    let level = &tree.levels[index];
    let channel = channel_name(level);
    // The panels each part's page places, before the key is applied.
    let mut placed: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut domains: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for part in level.parts.iter().filter(|p| p.nested.is_none()) {
        let ds = domains_of(level, part, model, panels, contract, ports);
        let mut ws: Vec<String> = Vec::new();
        for d in &ds {
            ws.extend(panels.of_domain(d).map(|p| p.widget.clone()));
        }
        placed.insert(part.name.clone(), ws);
        domains.insert(part.name.clone(), ds);
    }
    let (row_actions, header_actions) = actions_of(level, model, panels, contract, ports, &placed);
    let action_names: BTreeSet<String> = row_actions
        .values()
        .chain(header_actions.values())
        .flatten()
        .map(|a| a.name.clone())
        .collect();

    let part_names: BTreeSet<String> = level.parts.iter().map(|p| p.name.clone()).collect();
    let all_panels: BTreeSet<String> = placed.values().flatten().cloned().collect();
    let panels_of: BTreeMap<String, BTreeSet<String>> = placed
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
        .collect();
    let columns: BTreeMap<String, BTreeSet<String>> = write::TABLE_COLUMNS
        .iter()
        .map(|(t, cols)| {
            (
                t.to_string(),
                cols.iter().map(|(f, _, _)| f.to_string()).collect(),
            )
        })
        .collect();
    let names = view_key::Names {
        parts: &part_names,
        panels: &all_panels,
        panels_of: &panels_of,
        columns: &columns,
    };
    let key = match view_key::read(&level.doc, &names, &action_names) {
        Ok(k) => k,
        Err(rs) => {
            for r in rs {
                let path = if level.path.is_empty() {
                    r.path
                } else {
                    format!("{}/{}", level.path, r.path)
                };
                refusals.push(refusal(path, r.message));
            }
            ViewKey::default()
        }
    };

    let label_of = |name: &str| {
        key.label_parts
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    };
    let mut pages = Vec::new();
    let mut events = BTreeSet::new();
    for part in level.parts.iter().filter(|p| p.nested.is_none()) {
        let ds = &domains[&part.name];
        let instance = instance_panel(part, model, panels).filter(|w| !key.hide_panels.contains(w));
        let mut frames = Vec::new();
        let mut take = |p: &model::Panel, prefix: &str| -> Section {
            events.extend(p.events.iter().cloned());
            let mut s = section_of(p, level, &part.path, &channel, prefix);
            if let Some(l) = key.label_panels.get(&p.widget) {
                s.title = l.clone();
            }
            s.row_actions = row_actions
                .get(&(part.name.clone(), p.widget.clone()))
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|a| !key.hide_actions.contains(&a.name))
                .collect();
            s
        };
        if let Some(w) = &instance
            && let Some(p) = panels.get(w)
        {
            frames.push(Frame {
                area: "area_instances".into(),
                title: None,
                collapsed: false,
                sections: vec![take(p, "")],
            });
        }
        // Parts that run one copy per this part: their instances, beside this part's.
        let per: Vec<&model::Part> = level
            .parts
            .iter()
            .filter(|q| q.per.as_deref() == Some(part.name.as_str()) && q.nested.is_none())
            .collect();
        let mut per_sections = Vec::new();
        for q in &per {
            if let Some(w) =
                instance_panel(q, model, panels).filter(|w| !key.hide_panels.contains(w))
                && let Some(p) = panels.get(&w)
            {
                let mut s = take(p, &format!("per_{}_", snake(&q.name)));
                s.args = args_of(p, level, &q.path, &channel);
                s.title = format!("{} of {}", s.title, label_of(&q.name));
                per_sections.push(s);
            }
        }
        if !per_sections.is_empty() {
            let names: Vec<String> = per.iter().map(|q| label_of(&q.name)).collect();
            frames.push(Frame {
                area: "area_per".into(),
                title: Some(format!("One per {}: {}", part.name, names.join(", "))),
                collapsed: false,
                sections: per_sections,
            });
        }
        let order = key
            .order_panels
            .get(&part.name)
            .cloned()
            .unwrap_or_default();
        for d in ds {
            let mut ws: Vec<&model::Panel> = panels
                .of_domain(d)
                .filter(|p| {
                    Some(&p.widget) != instance.as_ref() && !key.hide_panels.contains(&p.widget)
                })
                .collect();
            ws.sort_by_key(|p| {
                order
                    .iter()
                    .position(|o| *o == p.widget)
                    .unwrap_or(usize::MAX)
            });
            if ws.is_empty() {
                continue;
            }
            let sections: Vec<Section> = ws.into_iter().map(|p| take(p, "")).collect();
            // A frame with a row action is the part's work to do: it opens unfolded.
            let acts = sections.iter().any(|s| !s.row_actions.is_empty());
            frames.push(Frame {
                area: format!("area_{}", snake(d.rsplit('.').next().unwrap_or(d))),
                title: Some(model.domain_label(d)),
                collapsed: !acts,
                sections,
            });
        }
        let actions = header_actions
            .get(&part.name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|a| !key.hide_actions.contains(&a.name))
            .collect();
        pages.push(PartPage {
            part: part.path.clone(),
            page: part_page(&part.path),
            label: label_of(&part.name),
            block: part.block.clone(),
            frames,
            actions,
        });
    }

    // Navigation: the key's groups, then nested levels, then one section per block.
    let order = table_order(level, &key);
    let mut sections: Vec<(String, String, Vec<Entry>)> = Vec::new();
    let mut grouped: BTreeSet<String> = BTreeSet::new();
    let entry = |name: &str| -> Entry {
        let p = level.part(name).expect("a part of the level");
        match p.nested {
            Some(i) => Entry::Nested(i),
            None => Entry::Page(part_page(&p.path)),
        }
    };
    let prefix = if level.path.is_empty() {
        String::new()
    } else {
        format!("{}_", snake(&level.path))
    };
    for g in &key.groups {
        let mut list: Vec<String> = g.parts.clone();
        list.sort_by_key(|p| order.iter().position(|o| o == p));
        grouped.extend(list.iter().cloned());
        sections.push((
            format!("{prefix}group_{}", g.name),
            g.label.clone(),
            list.iter().map(|p| entry(p)).collect(),
        ));
    }
    let mut by_block: Vec<(String, Vec<String>)> = Vec::new();
    for name in &order {
        if grouped.contains(name) {
            continue;
        }
        let p = level.part(name).expect("a part of the level");
        if p.nested.is_some() {
            sections.push((
                format!("{prefix}nested_{}", snake(&p.name)),
                label_of(&p.name),
                vec![entry(name)],
            ));
            continue;
        }
        match by_block.iter_mut().find(|(b, _)| *b == p.block) {
            Some((_, list)) => list.push(name.clone()),
            None => by_block.push((p.block.clone(), vec![name.clone()])),
        }
    }
    for (block, list) in by_block {
        sections.push((
            format!("{prefix}kind_{}", snake(&block)),
            capital(&block),
            list.iter().map(|p| entry(p)).collect(),
        ));
    }

    LevelPlan {
        level: index,
        overview: overview_page(level),
        title: level.name.clone(),
        channel: (!events.is_empty()).then_some(channel),
        events,
        order,
        pages,
        sections,
        key,
        instances: level.count,
    }
}

fn read_json(path: &Path) -> Result<Value, Failure> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| Failure::Unreadable(format!("{}: {e}", path.display())))?;
    if path.extension().is_some_and(|x| x == "json") {
        serde_json::from_str(&text)
            .map_err(|e| Failure::Unreadable(format!("{}: {e}", path.display())))
    } else {
        serde_yaml_ng::from_str(&text)
            .map_err(|e| Failure::Unreadable(format!("{}: {e}", path.display())))
    }
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `path` as an absolute path with no `.` or `..`, without touching the file system.
fn lexical(path: &Path) -> PathBuf {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The path from directory `from` to `to`, with `/`.
pub fn relative(from: &Path, to: &Path) -> String {
    let (a, b) = (lexical(from), lexical(to));
    let ac: Vec<_> = a.components().collect();
    let bc: Vec<_> = b.components().collect();
    let common = ac.iter().zip(&bc).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<String> = vec!["..".to_string(); ac.len() - common];
    parts.extend(
        bc[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

/// The name of the document `out`: its file name without `.ui.yaml`.
pub fn stem(out: &Path) -> String {
    let file = out
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    file.strip_suffix(".ui.yaml")
        .or_else(|| file.strip_suffix(".yaml"))
        .unwrap_or(&file)
        .to_string()
}

/// Derives the composition of `inputs.architecture` with the compiled blocks `ir`.
///
/// # Errors
///
/// An input that cannot be read, or an architecture or `view:` key that is refused.
pub fn derive_with(inputs: &Inputs, ir: &Value) -> Result<Derivation, Failure> {
    let root = &inputs.root;
    let tree = Tree::read(root, &inputs.architecture).map_err(Failure::Unreadable)?;
    let panels_text = std::fs::read_to_string(root.join(PANELS))
        .map_err(|e| Failure::Unreadable(format!("{PANELS}: {e}")))?;
    let panels = Panels::read(&panels_text).map_err(Failure::Unreadable)?;
    let model = Model { ir };
    let ports_path = inputs
        .ports
        .clone()
        .or_else(|| Some(root.join(PORTS)).filter(|p| p.exists()));
    let ports_doc = match &ports_path {
        Some(p) => Some(read_json(p)?),
        None => None,
    };
    let contract_doc = match &inputs.contract {
        Some(p) => Some(read_json(p)?),
        None => None,
    };
    let plan = plan(
        &tree,
        &model,
        &panels,
        ports_doc.as_ref(),
        contract_doc.as_ref(),
    )?;
    derivation(inputs, &plan, ports_path.as_ref())
}

/// The layout (`agenticspec-ui-layout/1`) of `architecture`, from inputs held in memory: the
/// compiled blocks `ir` (or the part of them a derivation reads), the panels document, the port
/// catalogue and the contract. `root` only names the documents read. The same layout as
/// [`derive_with`] writes beside its document.
///
/// # Errors
///
/// An input that cannot be read, or an architecture or `view:` key that is refused.
pub fn layout_with(
    root: &Path,
    architecture: &Path,
    ir: &Value,
    panels_text: &str,
    ports: Option<&Value>,
    contract: Option<&Value>,
) -> Result<String, Failure> {
    let tree = Tree::read(root, architecture).map_err(Failure::Unreadable)?;
    let panels = Panels::read(panels_text).map_err(Failure::Unreadable)?;
    let model = Model { ir };
    let plan = plan(&tree, &model, &panels, ports, contract)?;
    Ok(write::layout(&plan))
}

/// The whole composition of `tree`, or every refusal of the architecture and its `view:` key.
fn plan<'a>(
    tree: &'a Tree,
    model: &'a Model<'a>,
    panels: &'a Panels,
    ports_doc: Option<&Value>,
    contract_doc: Option<&Value>,
) -> Result<Plan<'a>, Failure> {
    let ports = ports_doc
        .map(|d| model::ports(d, model))
        .unwrap_or_default();
    let contract = contract_doc.map(Contract::read);

    let mut refusals = Vec::new();
    for level in &tree.levels {
        for p in &level.parts {
            if p.block == "architecture" && p.nested.is_none() {
                refusals.push(refusal(
                    format!("parts/{}", p.path),
                    "a nested architecture part names its document with `ref`",
                ));
            }
        }
    }
    let levels: Vec<LevelPlan> = (0..tree.levels.len())
        .map(|i| {
            plan_level(
                tree,
                i,
                model,
                panels,
                contract.as_ref(),
                &ports,
                &mut refusals,
            )
        })
        .collect();
    if !refusals.is_empty() {
        return Err(Failure::Refused(refusals));
    }
    let home = match levels[0].key.home.as_deref() {
        None | Some("overview") => levels[0].overview.clone(),
        Some(p) => part_page(p),
    };
    Ok(Plan {
        tree,
        model,
        panels,
        levels,
        home,
    })
}

/// The files of a derivation of `plan`: the document, its fixtures, its layout and its digest.
fn derivation(
    inputs: &Inputs,
    plan: &Plan<'_>,
    ports_path: Option<&PathBuf>,
) -> Result<Derivation, Failure> {
    let root = &inputs.root;
    let tree = plan.tree;
    let name = stem(&inputs.out);
    let dir = inputs.out.parent().unwrap_or(Path::new(".")).to_path_buf();
    let fixtures_dir = dir.join(format!("{name}.fixtures"));
    let to_panels = relative(&fixtures_dir, &root.join(FIXTURES));
    let scripts = write::scripts(&root.join(SCRIPTS));
    let doc = write::document(plan, &name, &format!("{name}.fixtures"));
    let mut files = vec![(inputs.out.clone(), doc)];
    for (rel, text) in write::fixtures(plan, &to_panels, &scripts) {
        files.push((fixtures_dir.join(rel), text));
    }
    files.push((dir.join(format!("{name}.layout.json")), write::layout(plan)));

    // The digest of every input read.
    let mut lines: Vec<String> = Vec::new();
    for (file, bytes) in &tree.files {
        lines.push(format!("{}  {file}", sha(bytes)));
    }
    for rel in [PANELS, PANELS_DIGEST] {
        let bytes = std::fs::read(root.join(rel))
            .map_err(|e| Failure::Unreadable(format!("{rel}: {e}")))?;
        lines.push(format!("{}  {rel}", sha(&bytes)));
    }
    for p in [ports_path, inputs.contract.as_ref()].into_iter().flatten() {
        let bytes =
            std::fs::read(p).map_err(|e| Failure::Unreadable(format!("{}: {e}", p.display())))?;
        lines.push(format!(
            "{}  {}",
            sha(&bytes),
            model::relative_to_root(root, p)
        ));
    }
    lines.push(format!("{}  {VERSION}", sha(VERSION.as_bytes())));
    files.push((
        dir.join(format!("{name}.sources.sha256")),
        lines.join("\n") + "\n",
    ));

    Ok(Derivation {
        pages: plan.levels.iter().map(|l| 1 + l.pages.len()).sum(),
        actions: plan
            .levels
            .iter()
            .flat_map(|l| &l.pages)
            .map(|p| {
                p.actions.len()
                    + p.frames
                        .iter()
                        .flat_map(|f| &f.sections)
                        .map(|s| s.row_actions.len())
                        .sum::<usize>()
            })
            .sum(),
        channels: plan.levels.iter().filter(|l| l.channel.is_some()).count(),
        files,
        fixtures_dir,
    })
}

/// The files under `dir`, sorted.
fn tree_of(dir: &Path) -> Vec<PathBuf> {
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

/// What differs between a derivation and the files on disk: a file that differs or is missing,
/// or one in the fixtures directory that the derivation does not write. Empty when current.
pub fn differences(d: &Derivation) -> Vec<String> {
    let mut out = Vec::new();
    for (path, text) in &d.files {
        match std::fs::read_to_string(path) {
            Ok(t) if t == *text => {}
            Ok(_) => out.push(format!(
                "{}: differs from a fresh derivation",
                path.display()
            )),
            Err(_) => out.push(format!("{}: derived, and not in the tree", path.display())),
        }
    }
    for f in tree_of(&d.fixtures_dir) {
        if !d.files.iter().any(|(p, _)| lexical(p) == lexical(&f)) {
            out.push(format!("{}: in the tree, and not derived", f.display()));
        }
    }
    out
}

/// Writes a derivation: the fixtures directory is replaced, every other file overwritten.
///
/// # Errors
///
/// A file or directory that cannot be written.
pub fn write(d: &Derivation) -> std::io::Result<()> {
    if d.fixtures_dir.exists() {
        std::fs::remove_dir_all(&d.fixtures_dir)?;
    }
    for (path, text) in &d.files {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)?;
    }
    Ok(())
}
