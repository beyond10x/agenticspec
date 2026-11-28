//! Reading a document into the model the emitter writes: parse, refuse by node path, merge each
//! page over its kind, resolve every reference and compile every expression.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::Refusal;
use crate::doc::{
    self, Action, Component, Document, Effect, Field, FieldAs, Labels, Layout, Node, Store, Style,
    Tone, Width,
};

// ------------------------------------------------------------------------------------------
// The model the emitter writes.

pub struct App {
    pub name: String,
    pub title: String,
    pub home: String,
    pub pages: Vec<PageModel>,
    pub widgets: Vec<WidgetModel>,
    pub reads: BTreeMap<String, ReadSig>,
    pub commands: BTreeMap<String, CommandSig>,
    pub channels: BTreeMap<String, ChannelSig>,
    pub hosts: BTreeMap<String, HostSig>,
    pub theme: Option<ThemeModel>,
    pub shell_state: Vec<(String, StateModel)>,
    /// The navigation, when a shell has a region of kind `navigation`.
    pub nav: Option<Vec<NavModel>>,
}

/// One section of the navigation: its label and its pages, each by name and label.
pub struct NavModel {
    pub label: String,
    pub pages: Vec<(String, String)>,
}

/// The parameters a widget reads its scope from (D7 of the view layer): the base URL of the
/// adapter that answers the scope, or the scope under the renderer's own base.
pub const SCOPE_PARAMS: [&str; 4] = ["source", "architecture", "part", "instance"];

/// A widget: one single-file component, named by the widget alone.
pub struct WidgetModel {
    pub name: String,
    pub component: String,
    pub summary: String,
    pub params: Vec<ParamModel>,
    pub body: Vec<SectionModel>,
}

pub struct ParamModel {
    pub name: String,
    /// `string`, `number` or `boolean`.
    pub ts_type: &'static str,
    pub required: bool,
}

#[derive(Default)]
pub struct ReadSig {
    pub placeholder: bool,
    pub params: BTreeSet<String>,
    pub fields: BTreeSet<String>,
}

#[derive(Default)]
pub struct CommandSig {
    pub input: BTreeSet<String>,
    pub answer: BTreeSet<String>,
}

pub struct ChannelSig {
    pub events: Vec<String>,
    pub view: Option<String>,
    pub per: Option<String>,
}

pub struct HostSig {
    pub at: String,
    pub field: bool,
    pub stands_in_for: String,
    pub reason: String,
    pub reads: Option<String>,
}

pub struct ThemeModel {
    pub default: String,
    pub themes: Vec<String>,
    pub state: Option<String>,
}

pub struct StateModel {
    pub store: Store,
    pub initial: Value,
}

pub struct PageModel {
    pub name: String,
    pub component: String,
    pub segments: Vec<String>,
    pub params: Vec<String>,
    pub state: Vec<(String, StateModel)>,
    pub title: String,
    pub title_from: Option<(String, String)>,
    pub bar: Option<SectionModel>,
    pub main: Vec<SectionModel>,
    pub side: Vec<SectionModel>,
}

pub struct SectionModel {
    pub name: String,
    pub var: String,
    pub title: Option<String>,
    /// What a table or list is called for a reader: the title, else the name.
    pub label: String,
    /// Drawn without a card of its own: a node of a widget's body, framed where it is used.
    pub bare: bool,
    pub collapsed: bool,
    pub kind: SectionKind,
    pub read: Option<ReadUse>,
    pub empty: Option<(Option<String>, String)>,
    pub failed: Option<String>,
}

pub enum SectionKind {
    Collection {
        columns: Vec<FieldModel>,
        list: bool,
        row_actions: Vec<ActionModel>,
        search: Option<String>,
    },
    Record {
        fields: Vec<FieldModel>,
        actions: Vec<ActionModel>,
    },
    Form {
        command: String,
        fields: Vec<FormField>,
        submit: String,
        result: Option<(String, Vec<FieldModel>)>,
    },
    FilterBar {
        choices: Vec<ChoiceModel>,
        search: Option<(String, Option<String>)>,
        actions: Vec<ActionModel>,
    },
    Host {
        host: String,
    },
    /// A use of a widget: its component and each argument as an expression.
    Widget {
        component: String,
        args: Vec<(String, String)>,
    },
}

pub struct ReadUse {
    pub var: String,
    pub view: String,
    pub params: Vec<(String, String)>,
    pub key: Option<String>,
    pub filter: Option<(String, String)>,
    pub live: Option<LiveUse>,
}

/// How a read takes the events of a channel: re-read (coalesced over `coalesce` milliseconds) or
/// patch the row whose `match` field the event carries, inserting it when no row has it.
pub struct LiveUse {
    /// The channel as an expression: a name, or a widget parameter.
    pub channel: String,
    pub on: Vec<String>,
    pub effect: Effect,
    pub match_: Option<String>,
    /// `only_if: matches(params)`: an event whose field of a read parameter differs is dropped.
    pub only_params: bool,
    pub coalesce: Option<u64>,
}

pub struct FieldModel {
    pub field: String,
    pub label: String,
    pub as_: FieldAs,
    pub labels: Option<BTreeMap<String, String>>,
    pub tones: Option<BTreeMap<String, Tone>>,
    pub tone: Option<Tone>,
}

pub enum FormInput {
    Host(String),
    Toggle,
    Text,
    Number,
}

pub struct FormField {
    pub field: String,
    pub label: String,
    pub input: FormInput,
}

pub struct ChoiceModel {
    pub label: String,
    pub binds: String,
    pub read: Option<ReadUse>,
    pub value: String,
    pub shows: String,
    pub placeholder: Option<String>,
    pub options: Vec<(Value, String)>,
}

pub struct ActionModel {
    pub label: String,
    pub kind: ActionKind,
    /// `visible`, compiled: shown only while it holds, beyond the action set.
    pub visible: Option<String>,
}

pub enum ActionKind {
    Does {
        command: String,
        var: String,
        input: Vec<(String, String)>,
    },
    Navigate {
        page: String,
        params: Vec<(String, String)>,
    },
    Copy(String),
}

// ------------------------------------------------------------------------------------------
// Parsing, with refusals by node path.

fn refusal(path: impl Into<String>, message: impl Into<String>) -> Refusal {
    Refusal {
        path: path.into(),
        message: message.into(),
    }
}

/// The node path of a serde path, read alongside the document: a map key is its key, a list
/// entry its `name` (else `field`, `view`, its text), never its index.
fn node_path(raw: &Value, path: &serde_path_to_error::Path) -> String {
    use serde_path_to_error::Segment;
    let mut at = Some(raw);
    let mut out: Vec<String> = Vec::new();
    for seg in path.iter() {
        match seg {
            Segment::Map { key } => {
                out.push(key.clone());
                at = at.and_then(|v| v.get(key));
            }
            Segment::Seq { index } => {
                let elem = at.and_then(|v| v.get(*index));
                let name = elem.and_then(|e| {
                    ["name", "field", "view"]
                        .iter()
                        .find_map(|k| e.get(k).and_then(Value::as_str))
                        .or_else(|| e.as_str())
                        .map(str::to_string)
                });
                out.push(name.unwrap_or_else(|| index.to_string()));
                at = elem;
            }
            Segment::Enum { .. } | Segment::Unknown => {}
        }
    }
    out.join("/")
}

fn between_backticks(message: &str) -> Option<&str> {
    let start = message.find('`')? + 1;
    let len = message[start..].find('`')?;
    Some(&message[start..start + len])
}

const COMPONENTS: [&str; 5] = ["collection", "record", "form", "filter_bar", "choice"];

/// Refuses a node whose `component` this generator does not draw, by the path of that key, before
/// the node's other keys are read (the keys of a node are read in their order of names, so an
/// unknown key of a chart would otherwise be named first).
fn components(raw: &Value, path: &str, widgets: &BTreeSet<String>, out: &mut Vec<Refusal>) {
    let Some(map) = raw.as_object() else { return };
    if let Some(Value::String(c)) = map.get("component")
        && !COMPONENTS.contains(&c.as_str())
        && !widgets.contains(c)
    {
        out.push(refusal(
            format!("{path}/component"),
            format!(
                "`{c}` is no component this generator draws (it draws {}) and no widget of the document",
                COMPONENTS.join(", ")
            ),
        ));
    }
    for key in ["sections", "choices", "body"] {
        for n in map.get(key).and_then(Value::as_array).into_iter().flatten() {
            let name = n.get("name").and_then(Value::as_str).unwrap_or("");
            components(n, &format!("{path}/{key}/{name}"), widgets, out);
        }
    }
    if let Some(r) = map.get("result") {
        components(r, &format!("{path}/result"), widgets, out);
    }
}

pub fn parse(text: &str) -> Result<(Document, Value), Vec<Refusal>> {
    let raw: Value = serde_yaml_ng::from_str(text).map_err(|e| {
        vec![refusal(
            "",
            format!("the document is no YAML this generator reads: {e}"),
        )]
    })?;
    let mut early = Vec::new();
    let widgets: BTreeSet<String> = raw
        .get("widgets")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    for group in ["pages", "page_kinds", "widgets"] {
        for (name, v) in raw
            .get(group)
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            components(v, &format!("{group}/{name}"), &widgets, &mut early);
        }
    }
    if !early.is_empty() {
        return Err(early);
    }
    let doc: Document = match serde_path_to_error::deserialize(&raw) {
        Ok(d) => d,
        Err(e) => {
            let mut path = node_path(&raw, e.path());
            let inner = e.inner().to_string();
            let message = if inner.starts_with("unknown field") {
                let name = between_backticks(&inner).unwrap_or("");
                if path.rsplit('/').next() != Some(name) {
                    if !path.is_empty() {
                        path.push('/');
                    }
                    path.push_str(name);
                }
                format!("unknown property `{name}`: this generator does not draw it ({inner})")
            } else if inner.starts_with("unknown variant") {
                let name = between_backticks(&inner).unwrap_or("");
                format!("`{name}` is not a value this generator draws here ({inner})")
            } else {
                inner
            };
            return Err(vec![refusal(path, message)]);
        }
    };
    if doc.format != "ess-ui/1" {
        return Err(vec![refusal(
            "format",
            format!(
                "the format is `{}`; this generator reads ess-ui/1",
                doc.format
            ),
        )]);
    }
    Ok((doc, raw))
}

// ------------------------------------------------------------------------------------------
// Resolution.

/// The name of a node as a TypeScript identifier.
pub fn ident(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// `embed.session` → `EmbedSessionPage`.
pub fn component_name(page: &str) -> String {
    let mut out = String::new();
    for part in page.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut cs = part.chars();
        if let Some(c) = cs.next() {
            out.push(c.to_ascii_uppercase());
            out.extend(cs);
        }
    }
    out.push_str("Page");
    out
}

fn humanize(name: &str) -> String {
    let s = name.replace(['_', '.', '-'], " ");
    let mut cs = s.chars();
    match cs.next() {
        Some(c) => c.to_uppercase().chain(cs).collect(),
        None => s,
    }
}

/// A TypeScript string literal in single quotes, safe in a template attribute.
pub fn ts_str(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\u0022"),
            '\n' => out.push_str("\\n"),
            '<' => out.push_str("\\u003c"),
            '&' => out.push_str("\\u0026"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// A JSON value as a TypeScript literal, safe in a template attribute.
pub fn ts_value(v: &Value) -> String {
    match v {
        Value::String(s) => ts_str(s),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(ts_value).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(o) => format!(
            "{{ {} }}",
            o.iter()
                .map(|(k, v)| format!("{}: {}", ts_str(k), ts_value(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => other.to_string(),
    }
}

/// `ActiveDelegations` → `Active delegations`.
fn humanize_camel(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push(' ');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// A widget parameter as its body reads it.
#[derive(Clone)]
struct Arg {
    required: bool,
    channel: bool,
}

#[derive(Clone, Default)]
struct Scope {
    state: BTreeSet<String>,
    params: BTreeSet<String>,
    shell: BTreeSet<String>,
    row: bool,
    /// Inside a widget's body: its parameters, read as `args.<name>`.
    args: Option<BTreeMap<String, Arg>>,
}

/// A row field an expression reads, to type the read it reads.
struct Compiled {
    ts: String,
    row_field: Option<String>,
}

fn compile(e: &str, scope: &Scope, path: &str) -> Result<Compiled, Refusal> {
    let e = e.trim();
    let plain = |ts: String| Compiled {
        ts,
        row_field: None,
    };
    if let Some(rest) = e.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        return Ok(plain(ts_str(rest)));
    }
    if let Some(rest) = e.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        return Ok(plain(ts_str(rest)));
    }
    if e == "true" || e == "false" || e == "null" || e.parse::<f64>().is_ok() {
        return Ok(plain(e.to_string()));
    }
    // `actor.may(<command>)`: whether the action set of the scope allows the command (R5).
    if let Some(cmd) = e
        .strip_prefix("actor.may(")
        .and_then(|r| r.strip_suffix(')'))
    {
        let cmd = cmd.trim();
        if !cmd.is_empty()
            && cmd
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            return Ok(plain(format!("may.allows({})", ts_str(cmd))));
        }
    }
    let word = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if let Some((root, name)) = e.split_once('.') {
        if word(root) && word(name) {
            let found = match root {
                "state" if scope.state.contains(name) => {
                    Some(format!("page.state[{}]", ts_str(name)))
                }
                "params" if scope.params.contains(name) => {
                    Some(format!("page.params[{}]", ts_str(name)))
                }
                "shell" if scope.shell.contains(name) => {
                    Some(format!("shell.state[{}]", ts_str(name)))
                }
                "row" if scope.row => {
                    return Ok(Compiled {
                        ts: format!("row[{}]", ts_str(name)),
                        row_field: Some(name.to_string()),
                    });
                }
                "args" => match scope.args.as_ref().and_then(|a| a.get(name)) {
                    Some(a) if a.required => Some(format!("args[{}]", ts_str(name))),
                    Some(_) => Some(format!("(args[{}] ?? null)", ts_str(name))),
                    None => None,
                },
                _ => None,
            };
            return found.map(plain).ok_or_else(|| {
                refusal(
                    path,
                    format!("expression `{e}` reads `{root}.{name}`, which is not declared where it stands"),
                )
            });
        }
        return Err(refusal(
            path,
            format!(
                "expression `{e}` is not one this generator compiles (state.<name>, params.<name>, shell.<name>, row.<field>, args.<name> in a widget, actor.may(<command>) or a literal)"
            ),
        ));
    }
    if e.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ' ' || c == '-')
        && !e.is_empty()
    {
        return Ok(plain(ts_str(e)));
    }
    Err(refusal(
        path,
        format!(
            "expression `{e}` is not one this generator compiles (state.<name>, params.<name>, shell.<name>, row.<field>, args.<name> in a widget, actor.may(<command>) or a literal)"
        ),
    ))
}

/// The properties a node of each kind takes besides `name`, `title`, `collapsed` and `states`.
fn allowed(component: Option<Component>, host: bool, widget: bool) -> &'static [&'static str] {
    if host {
        return &["host", "reason", "stands_in_for", "reads", "live"];
    }
    if widget {
        return &["component", "args"];
    }
    match component {
        Some(Component::Collection) => &[
            "component",
            "reads",
            "live",
            "columns",
            "style",
            "row_actions",
        ],
        Some(Component::Record) => &["component", "reads", "live", "fields", "actions"],
        Some(Component::Form) => &["component", "does", "fields", "submit", "result"],
        Some(Component::FilterBar) => &["component", "binds", "choices", "search", "actions"],
        Some(Component::Choice) => &[
            "component",
            "reads",
            "binds",
            "value",
            "label",
            "placeholder",
            "options",
        ],
        None => &[],
    }
}

fn present(n: &Node) -> Vec<&'static str> {
    let mut out = Vec::new();
    let mut add = |k: &'static str, b: bool| {
        if b {
            out.push(k)
        }
    };
    add("component", n.component.is_some());
    add("args", n.args.is_some());
    add("host", n.host.is_some());
    add("reason", n.reason.is_some());
    add("stands_in_for", n.stands_in_for.is_some());
    add("commands", n.commands.is_some());
    add("reads", n.reads.is_some());
    add("live", n.live.is_some());
    add("columns", n.columns.is_some());
    add("style", n.style.is_some());
    add("row_actions", n.row_actions.is_some());
    add("fields", n.fields.is_some());
    add("actions", n.actions.is_some());
    add("does", n.does.is_some());
    add("submit", n.submit.is_some());
    add("result", n.result.is_some());
    add("binds", n.binds.is_some());
    add("choices", n.choices.is_some());
    add("search", n.search.is_some());
    add("value", n.value.is_some());
    add("label", n.label.is_some());
    add("placeholder", n.placeholder.is_some());
    add("options", n.options.is_some());
    out
}

struct Resolver<'a> {
    doc: &'a Document,
    refusals: Vec<Refusal>,
    reads: BTreeMap<String, ReadSig>,
    commands: BTreeMap<String, CommandSig>,
    hosts: BTreeMap<String, HostSig>,
    /// The events each channel parameter of the widget being resolved is taken for.
    channel_uses: BTreeMap<String, BTreeSet<String>>,
    /// The same, by widget, for the check of each use.
    widget_channels: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
}

/// `500ms`, `1s`, `2m` in milliseconds.
fn duration_ms(d: &str) -> Option<u64> {
    let d = d.trim();
    let (n, unit) = if let Some(n) = d.strip_suffix("ms") {
        (n, 1)
    } else if let Some(n) = d.strip_suffix('s') {
        (n, 1000)
    } else {
        let n = d.strip_suffix('m')?;
        (n, 60_000)
    };
    n.parse::<u64>().ok().map(|n| n * unit)
}

/// `agentic.harness.Turns` → `AgenticHarnessTurns`: the file of a widget, by its name alone.
pub fn widget_component(name: &str) -> String {
    let mut out = String::new();
    for part in name.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut cs = part.chars();
        if let Some(c) = cs.next() {
            out.push(c.to_ascii_uppercase());
            out.extend(cs);
        }
    }
    out
}

/// A parameter's type as a prop: (TypeScript type, optional, a channel), or why it is refused.
fn param_type(ty: &Value) -> Result<(&'static str, bool, bool), String> {
    match ty {
        Value::String(t) => Ok((
            match t.as_str() {
                "integer" | "number" | "Integer" | "Float" => "number",
                "boolean" | "Boolean" => "boolean",
                _ => "string",
            },
            false,
            false,
        )),
        Value::Object(m) if m.len() == 1 => {
            if let Some(inner) = m.get("optional") {
                let (t, _, channel) = param_type(inner)?;
                return Ok((t, true, channel));
            }
            if m.get("ref").and_then(Value::as_str) == Some("channel") {
                return Ok(("string", false, true));
            }
            Err(format!(
                "a parameter of type {} is not drawn here (a scalar, {{optional: ...}} or {{ref: channel}})",
                serde_json::to_string(ty).unwrap_or_default()
            ))
        }
        _ => Err(format!(
            "a parameter of type {} is not drawn here (a scalar, {{optional: ...}} or {{ref: channel}})",
            serde_json::to_string(ty).unwrap_or_default()
        )),
    }
}

impl<'a> Resolver<'a> {
    fn refuse(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.refusals.push(refusal(path, message));
    }

    /// The parameters of a widget as its body reads them, with their prop types.
    fn params_of(&self, w: &doc::Widget) -> BTreeMap<String, (Arg, &'static str)> {
        w.params
            .iter()
            .filter_map(|(p, def)| {
                let (ts, optional, channel) = param_type(&def.ty).ok()?;
                Some((
                    p.clone(),
                    (
                        Arg {
                            required: def.required == Some(true) && !optional,
                            channel,
                        },
                        ts,
                    ),
                ))
            })
            .collect()
    }

    fn widget(&mut self, name: &str, w: &doc::Widget) -> Option<WidgetModel> {
        let path = format!("widgets/{name}");
        let mut params = Vec::new();
        for (p, def) in &w.params {
            let ppath = format!("{path}/params/{p}");
            let ident_ok = !p.is_empty()
                && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !p.starts_with(|c: char| c.is_ascii_digit());
            if !ident_ok {
                self.refuse(
                    &ppath,
                    "a parameter's name is a word of letters, digits and _",
                );
            }
            if def.default.is_some() {
                self.refuse(
                    format!("{ppath}/default"),
                    "this generator takes no default for a widget parameter; give the value at each use",
                );
            }
            match param_type(&def.ty) {
                Ok((ts, optional, channel)) => {
                    let required = def.required == Some(true) && !optional;
                    if channel && !required {
                        self.refuse(
                            format!("{ppath}/required"),
                            "a channel parameter is required, since the live updates of the body take it",
                        );
                    }
                    params.push(ParamModel {
                        name: p.clone(),
                        ts_type: ts,
                        required,
                    });
                }
                Err(m) => self.refuse(format!("{ppath}/type"), m),
            }
        }
        if w.body.is_empty() {
            self.refuse(
                format!("{path}/body"),
                "a widget's body holds at least one node",
            );
            return None;
        }
        let scope = Scope {
            args: Some(
                self.params_of(w)
                    .into_iter()
                    .map(|(k, (a, _))| (k, a))
                    .collect(),
            ),
            ..Scope::default()
        };
        self.channel_uses.clear();
        let mut body = Vec::new();
        for n in &w.body {
            let npath = format!("{path}/body/{}", n.name.as_deref().unwrap_or(""));
            if let Some(m) = self.node(n, &npath, &scope, None) {
                body.push(m);
            }
        }
        if let [only] = body.as_mut_slice()
            && only.title.is_none()
        {
            only.label = humanize_camel(name.rsplit('.').next().unwrap_or(name));
        }
        let uses = std::mem::take(&mut self.channel_uses);
        self.widget_channels.insert(name.to_string(), uses);
        Some(WidgetModel {
            name: name.to_string(),
            component: widget_component(name),
            summary: w.summary.clone(),
            params,
            body,
        })
    }

    /// A section that uses a widget: every required parameter given, each argument one the
    /// widget declares, and a channel argument a channel that carries what the body takes.
    fn widget_use(&mut self, n: &Node, widget: &str, path: &str, scope: &Scope) -> SectionKind {
        let component = widget_component(widget);
        let Some(w) = self.doc.widgets.get(widget) else {
            self.refuse(format!("{path}/component"), format!("no widget `{widget}`"));
            return SectionKind::Widget {
                component,
                args: Vec::new(),
            };
        };
        let params = self.params_of(w);
        let given = n.args.clone().unwrap_or_default();
        for (p, (a, _)) in &params {
            if a.required && !given.contains_key(p) {
                self.refuse(
                    format!("{path}/args"),
                    format!("the widget `{widget}` takes `{p}`, which is required and not given"),
                );
            }
        }
        let uses = self
            .widget_channels
            .get(widget)
            .cloned()
            .unwrap_or_default();
        let mut args = Vec::new();
        for (k, v) in &given {
            let apath = format!("{path}/args/{k}");
            let Some((a, _)) = params.get(k) else {
                self.refuse(
                    &apath,
                    format!("the widget `{widget}` declares no parameter `{k}`"),
                );
                continue;
            };
            let ts = if a.channel {
                match v
                    .as_str()
                    .and_then(|c| self.doc.channels.get(c).map(|ch| (c, ch)))
                {
                    Some((c, ch)) => {
                        for e in uses.get(k).into_iter().flatten() {
                            if !ch.carries.events.is_empty() && !ch.carries.events.contains(e) {
                                self.refuse(
                                    &apath,
                                    format!("channel `{c}` does not carry `{e}`, which the widget's body takes"),
                                );
                            }
                        }
                        ts_str(c)
                    }
                    None => {
                        self.refuse(
                            &apath,
                            "a channel parameter takes the name of a channel of the document",
                        );
                        continue;
                    }
                }
            } else {
                match v {
                    Value::String(e) => self.expr(e, scope, &apath, None),
                    Value::Number(_) | Value::Bool(_) | Value::Null => v.to_string(),
                    _ => {
                        self.refuse(&apath, "a list or a map is not passed to a widget here");
                        continue;
                    }
                }
            };
            args.push((k.clone(), ts));
        }
        SectionKind::Widget { component, args }
    }

    fn expr(&mut self, e: &str, scope: &Scope, path: &str, read: Option<&str>) -> String {
        match compile(e, scope, path) {
            Ok(c) => {
                if let (Some(f), Some(view)) = (c.row_field, read) {
                    self.reads
                        .entry(view.to_string())
                        .or_default()
                        .fields
                        .insert(f);
                }
                c.ts
            }
            Err(r) => {
                self.refusals.push(r);
                "null".into()
            }
        }
    }

    fn labels(&mut self, labels: &Option<Labels>, path: &str) -> Option<BTreeMap<String, String>> {
        match labels {
            None => None,
            Some(Labels::Map(m)) => Some(m.clone()),
            Some(Labels::Name(n)) => match self.doc.label_maps.get(n) {
                Some(m) => Some(m.clone()),
                None => {
                    self.refuse(format!("{path}/labels"), format!("no label map `{n}`"));
                    None
                }
            },
        }
    }

    fn field(&mut self, f: &Field, path: &str, read: Option<&str>) -> FieldModel {
        let fpath = format!("{path}/{}", f.name.as_deref().unwrap_or(&f.field));
        if f.host.is_some() {
            self.refuse(
                format!("{fpath}/host"),
                "a host slot stands only on a form's input here",
            );
        }
        let tones = match &f.tone_by {
            None => None,
            Some(n) => match self.doc.tone_maps.get(n) {
                Some(m) => Some(m.clone()),
                None => {
                    self.refuse(format!("{fpath}/tone_by"), format!("no tone map `{n}`"));
                    None
                }
            },
        };
        if let Some(view) = read {
            self.reads
                .entry(view.to_string())
                .or_default()
                .fields
                .insert(f.field.clone());
        }
        FieldModel {
            field: f.field.clone(),
            label: f.label.clone().unwrap_or_else(|| humanize(&f.field)),
            as_: f.as_.unwrap_or(FieldAs::Text),
            labels: self.labels(&f.labels, &fpath),
            tones,
            tone: f.tone,
        }
    }

    fn read(&mut self, n: &Node, var: &str, scope: &Scope, path: &str) -> Option<ReadUse> {
        let r = n.reads.as_ref()?;
        let rpath = format!("{path}/reads");
        let view = match (&r.view, &r.placeholder) {
            (Some(v), None) => v.clone(),
            (None, Some(p)) => {
                if r.fixture.is_none() {
                    self.refuse(&rpath, "a placeholder read names its fixture");
                }
                p.clone()
            }
            _ => {
                self.refuse(&rpath, "a read names exactly one of view and placeholder");
                return None;
            }
        };
        {
            let sig = self.reads.entry(view.clone()).or_default();
            sig.placeholder = r.placeholder.is_some();
            if let Some(k) = &r.key {
                sig.fields.insert(k.clone());
            }
        }
        let mut params = Vec::new();
        for (k, e) in &r.params {
            let ts = self.expr(e, scope, &format!("{rpath}/params/{k}"), None);
            self.reads
                .entry(view.clone())
                .or_default()
                .params
                .insert(k.clone());
            params.push((k.clone(), ts));
        }
        let filter = r.filter.as_ref().and_then(|f| {
            let words: Vec<&str> = f.split_whitespace().collect();
            match words.as_slice() {
                [a, "contains", b] if a.starts_with("row.") && b.starts_with("state.") => {
                    let field = a["row.".len()..].to_string();
                    let state = b["state.".len()..].to_string();
                    if !scope.state.contains(&state) {
                        self.refuse(
                            format!("{rpath}/filter"),
                            format!("expression `{f}` reads `{b}`, which is not declared where it stands"),
                        );
                        return None;
                    }
                    self.reads.entry(view.clone()).or_default().fields.insert(field.clone());
                    Some((field, state))
                }
                _ => {
                    self.refuse(
                        format!("{rpath}/filter"),
                        format!("expression `{f}` is not one this generator compiles as a filter (row.<field> contains state.<name>)"),
                    );
                    None
                }
            }
        });
        let live = n.live.as_ref().and_then(|l| {
            let lpath = format!("{path}/live");
            let channel = match l.channel.strip_prefix("args.") {
                Some(p) => match scope.args.as_ref().and_then(|a| a.get(p)) {
                    Some(a) if a.channel && a.required => format!("args[{}]", ts_str(p)),
                    Some(_) => {
                        self.refuse(
                            format!("{lpath}/channel"),
                            format!("`{}` is no required widget parameter of type {{ref: channel}}", l.channel),
                        );
                        return None;
                    }
                    None => {
                        self.refuse(
                            format!("{lpath}/channel"),
                            format!("expression `{}` reads a parameter the widget does not declare", l.channel),
                        );
                        return None;
                    }
                },
                None => match self.doc.channels.get(&l.channel) {
                    Some(c) => {
                        for e in &l.on {
                            if !c.carries.events.is_empty() && !c.carries.events.contains(e) {
                                self.refuse(
                                    format!("{lpath}/on"),
                                    format!("channel `{}` does not carry `{e}`", l.channel),
                                );
                            }
                        }
                        ts_str(&l.channel)
                    }
                    None => {
                        self.refuse(
                            format!("{lpath}/channel"),
                            format!("no channel `{}`", l.channel),
                        );
                        return None;
                    }
                },
            };
            if let Some(p) = l.channel.strip_prefix("args.") {
                self.channel_uses
                    .entry(p.to_string())
                    .or_default()
                    .extend(l.on.iter().cloned());
            }
            let coalesce = match &l.coalesce {
                None => None,
                Some(d) => match duration_ms(d) {
                    Some(ms) => Some(ms),
                    None => {
                        self.refuse(
                            format!("{lpath}/coalesce"),
                            format!("`{d}` is no duration this generator reads (such as 500ms, 1s or 2m)"),
                        );
                        None
                    }
                },
            };
            let match_ = match l.effect {
                Effect::Refetch => {
                    if l.match_.is_some() {
                        self.refuse(
                            format!("{lpath}/match"),
                            "`match` picks the row an event patches; a refetch reads every row again",
                        );
                    }
                    None
                }
                Effect::InsertOrPatch => {
                    if l.coalesce.is_some() {
                        self.refuse(
                            format!("{lpath}/coalesce"),
                            "this generator coalesces re-reads; a patch is applied as its event comes",
                        );
                    }
                    let m = l
                        .match_
                        .clone()
                        .or_else(|| r.key.clone())
                        .unwrap_or_else(|| "id".into());
                    self.reads
                        .entry(view.clone())
                        .or_default()
                        .fields
                        .insert(m.clone());
                    Some(m)
                }
            };
            let only_params = match l.only_if.as_deref().map(str::trim) {
                None => false,
                Some("matches(params)") => true,
                Some(e) => {
                    self.refuse(
                        format!("{lpath}/only_if"),
                        format!("expression `{e}` is not one this generator compiles here (matches(params))"),
                    );
                    false
                }
            };
            Some(LiveUse {
                channel,
                on: l.on.clone(),
                effect: l.effect,
                match_,
                only_params,
                coalesce,
            })
        });
        Some(ReadUse {
            var: var.to_string(),
            view,
            params,
            key: r.key.clone(),
            filter,
            live,
        })
    }

    fn action(
        &mut self,
        a: &Action,
        path: &str,
        scope: &Scope,
        owner: &str,
        read: Option<&str>,
    ) -> Option<ActionModel> {
        let kinds = [a.does.is_some(), a.navigate.is_some(), a.copy.is_some()];
        let name = a.name.clone().unwrap_or_else(|| {
            a.does
                .as_deref()
                .map(|d| d.rsplit('.').next().unwrap_or(d).to_string())
                .or_else(|| a.navigate.as_ref().map(|n| n.to_string_name()))
                .unwrap_or_else(|| "action".into())
        });
        let apath = format!("{path}/{name}");
        if kinds.iter().filter(|b| **b).count() != 1 {
            self.refuse(
                &apath,
                "an action carries exactly one of does, navigate and copy",
            );
            return None;
        }
        let label = a.label.clone().unwrap_or_else(|| humanize(&name));
        let kind = if let Some(cmd) = &a.does {
            let mut input = Vec::new();
            for (k, e) in &a.bind {
                let ts = self.expr(e, scope, &format!("{apath}/bind/{k}"), read);
                input.push((k.clone(), ts));
            }
            let sig = self.commands.entry(cmd.clone()).or_default();
            sig.input.extend(a.bind.keys().cloned());
            ActionKind::Does {
                command: cmd.clone(),
                var: format!("c_{}_{}", ident(owner), ident(&name)),
                input,
            }
        } else if let Some(nav) = &a.navigate {
            if !self.doc.pages.contains_key(&nav.to) {
                self.refuse(
                    format!("{apath}/navigate/to"),
                    format!("no page `{}`", nav.to),
                );
                return None;
            }
            let mut params = Vec::new();
            for (k, e) in &nav.params {
                let ts = self.expr(e, scope, &format!("{apath}/navigate/params/{k}"), read);
                params.push((k.clone(), ts));
            }
            ActionKind::Navigate {
                page: nav.to.clone(),
                params,
            }
        } else {
            let e = a.copy.as_deref().unwrap_or("");
            ActionKind::Copy(self.expr(e, scope, &format!("{apath}/copy"), read))
        };
        let visible = a
            .visible
            .as_ref()
            .map(|e| self.expr(e, scope, &format!("{apath}/visible"), read));
        Some(ActionModel {
            label,
            kind,
            visible,
        })
    }

    fn check_keys(&mut self, n: &Node, path: &str) -> bool {
        let host = n.host.is_some();
        if host && n.component.is_some() {
            self.refuse(path, "a host slot carries no component");
            return false;
        }
        if !host && n.component.is_none() {
            self.refuse(path, "a node names its component (or is a host slot)");
            return false;
        }
        let ok = allowed(n.kind(), host, n.widget().is_some());
        let mut good = true;
        for k in present(n) {
            if !ok.contains(&k) {
                let what = if host {
                    "host slot"
                } else if n.widget().is_some() {
                    "widget instance"
                } else {
                    n.kind().map_or("", Component::word)
                };
                self.refuse(
                    format!("{path}/{k}"),
                    format!("`{k}` is no property of a {what} this generator draws"),
                );
                good = false;
            }
        }
        good
    }

    fn states(n: &Node) -> (Option<(Option<String>, String)>, Option<String>) {
        let s = n.states.clone().unwrap_or_default();
        (
            s.empty.map(|e| (e.title, e.message)),
            s.failed.and_then(|f| f.message),
        )
    }

    fn node(
        &mut self,
        n: &Node,
        path: &str,
        scope: &Scope,
        owner: Option<&str>,
    ) -> Option<SectionModel> {
        let Some(name) = n.name.clone() else {
            self.refuse(path, "a node in a list carries a name");
            return None;
        };
        if !self.check_keys(n, path) {
            return None;
        }
        let in_widget = scope.args.is_some();
        if in_widget {
            let what = if n.host.is_some() {
                Some("a host slot")
            } else if n.widget().is_some() {
                Some("a widget instance")
            } else {
                match n.kind() {
                    Some(Component::Collection | Component::Record) => None,
                    Some(c) => Some(c.word()),
                    None => None,
                }
            };
            if let Some(what) = what {
                self.refuse(
                    path,
                    format!("{what} stands in a page section here; a widget's body holds collections and records"),
                );
                return None;
            }
        }
        let var = match owner {
            Some(o) => format!("{}_{}", ident(o), ident(&name)),
            None => ident(&name),
        };
        let label = n.title.clone().unwrap_or_else(|| name.clone());
        if let Some(widget) = n.widget() {
            if n.states.is_some() {
                self.refuse(
                    format!("{path}/states"),
                    "a widget instance reads nothing itself; the states are its body's",
                );
            }
            let kind = self.widget_use(n, widget, path, scope);
            return Some(SectionModel {
                name,
                var,
                title: n.title.clone(),
                label,
                bare: false,
                collapsed: n.collapsed.unwrap_or(false),
                kind,
                read: None,
                empty: None,
                failed: None,
            });
        }
        let (empty, failed) = Self::states(n);
        let read = self.read(n, &format!("r_{var}"), scope, path);
        let view = read.as_ref().map(|r| r.view.clone());
        let row_scope = Scope {
            row: true,
            ..scope.clone()
        };
        let kind = if let Some(host) = &n.host {
            let reason = n.reason.clone().unwrap_or_default();
            if reason.trim().is_empty() {
                self.refuse(
                    format!("{path}/reason"),
                    "a host slot says why the generator cannot draw it",
                );
            }
            if self.hosts.contains_key(host) {
                self.refuse(
                    format!("{path}/host"),
                    format!("host `{host}` is declared twice"),
                );
            }
            self.hosts.insert(
                host.clone(),
                HostSig {
                    at: path.to_string(),
                    field: false,
                    stands_in_for: n.stands_in_for.clone().unwrap_or_default(),
                    reason,
                    reads: view.clone(),
                },
            );
            SectionKind::Host { host: host.clone() }
        } else {
            match n.kind().unwrap_or(Component::Record) {
                Component::Collection => {
                    let columns = n
                        .columns
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .map(|c| self.field(c, &format!("{path}/columns"), view.as_deref()))
                        .collect();
                    let row_actions = n
                        .row_actions
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|a| {
                            self.action(
                                a,
                                &format!("{path}/row_actions"),
                                &row_scope,
                                &var,
                                view.as_deref(),
                            )
                        })
                        .collect();
                    SectionKind::Collection {
                        columns,
                        list: n.style == Some(Style::List),
                        row_actions,
                        search: read
                            .as_ref()
                            .and_then(|r| r.filter.as_ref().map(|f| f.1.clone())),
                    }
                }
                Component::Record => {
                    let fields = n
                        .fields
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .map(|f| self.field(f, &format!("{path}/fields"), view.as_deref()))
                        .collect();
                    let actions = n
                        .actions
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|a| {
                            self.action(
                                a,
                                &format!("{path}/actions"),
                                &row_scope,
                                &var,
                                view.as_deref(),
                            )
                        })
                        .collect();
                    SectionKind::Record { fields, actions }
                }
                Component::Form => self.form(n, path, scope)?,
                Component::FilterBar => self.filter_bar(n, path, scope, &var),
                Component::Choice => {
                    self.refuse(path, "a choice stands in a filter bar or a field here");
                    return None;
                }
            }
        };
        Some(SectionModel {
            name,
            var,
            title: n.title.clone(),
            label,
            bare: in_widget,
            collapsed: n.collapsed.unwrap_or(false),
            kind,
            read,
            empty,
            failed,
        })
    }

    /// A form: its command's input is the page's path parameters, then its fields.
    fn form(&mut self, n: &Node, path: &str, scope: &Scope) -> Option<SectionKind> {
        let Some(command) = n.does.clone() else {
            self.refuse(format!("{path}/does"), "a form names the command it sends");
            return None;
        };
        self.commands
            .entry(command.clone())
            .or_default()
            .input
            .extend(scope.params.iter().cloned());
        let mut fields = Vec::new();
        for f in n.fields.clone().unwrap_or_default() {
            let fpath = format!("{path}/fields/{}", f.name.as_deref().unwrap_or(&f.field));
            let input = if let Some(host) = &f.host {
                let reason = f.reason.clone().unwrap_or_default();
                if reason.trim().is_empty() {
                    self.refuse(
                        format!("{fpath}/reason"),
                        "a host slot says why the generator cannot draw it",
                    );
                }
                if self.hosts.contains_key(host) {
                    self.refuse(
                        format!("{fpath}/host"),
                        format!("host `{host}` is declared twice"),
                    );
                }
                self.hosts.insert(
                    host.clone(),
                    HostSig {
                        at: fpath.clone(),
                        field: true,
                        stands_in_for: f.stands_in_for.clone().unwrap_or_default(),
                        reason,
                        reads: None,
                    },
                );
                FormInput::Host(host.clone())
            } else {
                for (k, set) in [
                    ("labels", f.labels.is_some()),
                    ("tone_by", f.tone_by.is_some()),
                ] {
                    if set {
                        self.refuse(
                            format!("{fpath}/{k}"),
                            format!("`{k}` is no property of a form input this generator draws"),
                        );
                    }
                }
                match f.as_.unwrap_or(FieldAs::Text) {
                    FieldAs::Toggle => FormInput::Toggle,
                    FieldAs::Text => FormInput::Text,
                    FieldAs::Number => FormInput::Number,
                    other => {
                        self.refuse(
                            format!("{fpath}/as"),
                            format!(
                                "a form input `as: {}` is not one this generator draws",
                                other.word()
                            ),
                        );
                        FormInput::Text
                    }
                }
            };
            self.commands
                .entry(command.clone())
                .or_default()
                .input
                .insert(f.field.clone());
            fields.push(FormField {
                field: f.field.clone(),
                label: f.label.clone().unwrap_or_else(|| humanize(&f.field)),
                input,
            });
        }
        let result = match &n.result {
            None => None,
            Some(r) => {
                let rname = r.name.clone().unwrap_or_else(|| "result".into());
                let rpath = format!("{path}/result");
                if r.kind() != Some(Component::Record) || r.host.is_some() {
                    self.refuse(&rpath, "a form's result is a record here");
                    None
                } else if !self.check_keys(r, &rpath) {
                    None
                } else if r.reads.is_some() || r.actions.is_some() || r.live.is_some() {
                    self.refuse(
                        &rpath,
                        "a form's result shows the command's answer and reads nothing",
                    );
                    None
                } else {
                    let fields: Vec<FieldModel> = r
                        .fields
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .map(|f| self.field(f, &format!("{rpath}/fields"), None))
                        .collect();
                    let sig = self.commands.entry(command.clone()).or_default();
                    sig.answer.extend(fields.iter().map(|f| f.field.clone()));
                    Some((rname, fields))
                }
            }
        };
        Some(SectionKind::Form {
            command,
            fields,
            submit: n
                .submit
                .as_ref()
                .and_then(|s| s.label.clone())
                .unwrap_or_else(|| "Submit".into()),
            result,
        })
    }

    fn filter_bar(&mut self, n: &Node, path: &str, scope: &Scope, var: &str) -> SectionKind {
        if let Some(doc::Binds::One(b)) = &n.binds {
            self.refuse(
                format!("{path}/binds"),
                format!("a filter bar binds a list of state, not `{b}`"),
            );
        }
        let mut choices = Vec::new();
        for c in n.choices.clone().unwrap_or_default() {
            let Some(cname) = c.name.clone() else {
                self.refuse(
                    format!("{path}/choices"),
                    "a choice in a list carries a name",
                );
                continue;
            };
            let cpath = format!("{path}/choices/{cname}");
            if c.kind() != Some(Component::Choice) {
                self.refuse(&cpath, "a filter bar's choices are choices");
                continue;
            }
            if !self.check_keys(&c, &cpath) {
                continue;
            }
            let binds = match &c.binds {
                Some(doc::Binds::One(b)) => match b.strip_prefix("state.") {
                    Some(s) if scope.state.contains(s) => s.to_string(),
                    _ => {
                        self.refuse(
                            format!("{cpath}/binds"),
                            format!("expression `{b}` is not page state declared here"),
                        );
                        continue;
                    }
                },
                _ => {
                    self.refuse(format!("{cpath}/binds"), "a choice binds one page state");
                    continue;
                }
            };
            let cvar = format!("{var}_{}", ident(&cname));
            let read = self.read(&c, &format!("r_{cvar}"), scope, &cpath);
            let value = c
                .value
                .clone()
                .or_else(|| read.as_ref().and_then(|r| r.key.clone()))
                .unwrap_or_else(|| "id".into());
            let shows = c.label.clone().unwrap_or_else(|| "label".into());
            if let Some(r) = &read {
                let sig = self.reads.entry(r.view.clone()).or_default();
                sig.fields.insert(value.clone());
                sig.fields.insert(shows.clone());
            }
            choices.push(ChoiceModel {
                label: humanize(&cname),
                binds,
                read,
                value,
                shows,
                placeholder: c.placeholder.clone(),
                options: c
                    .options
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|o| (o.value.clone(), o.label.clone()))
                    .collect(),
            });
        }
        let search = n
            .search
            .as_ref()
            .and_then(|s| match s.binds.strip_prefix("state.") {
                Some(st) if scope.state.contains(st) => {
                    Some((st.to_string(), s.placeholder.clone()))
                }
                _ => {
                    self.refuse(
                        format!("{path}/search/binds"),
                        format!("expression `{}` is not page state declared here", s.binds),
                    );
                    None
                }
            });
        let actions = n
            .actions
            .clone()
            .unwrap_or_default()
            .iter()
            .filter_map(|a| self.action(a, &format!("{path}/actions"), scope, var, None))
            .collect();
        SectionKind::FilterBar {
            choices,
            search,
            actions,
        }
    }

    fn state_models(
        &mut self,
        states: &BTreeMap<String, doc::State>,
        path: &str,
    ) -> Vec<(String, StateModel)> {
        let mut out = Vec::new();
        for (name, s) in states {
            let store = s.store.unwrap_or(match s.class {
                doc::StateClass::Preference => Store::LocalStorage,
                doc::StateClass::PageState => Store::Url,
                _ => Store::Memory,
            });
            let initial = s.default.clone().unwrap_or_else(|| match &s.ty {
                Value::String(t) if t == "boolean" => Value::Bool(false),
                Value::String(t) if t == "integer" || t == "number" => Value::Null,
                _ => Value::String(String::new()),
            });
            if s.store == Some(Store::SessionStorage) {
                self.refuse(
                    format!("{path}/{name}/store"),
                    "this generator keeps state in memory, the URL or local storage",
                );
            }
            out.push((name.clone(), StateModel { store, initial }));
        }
        out
    }

    fn page(
        &mut self,
        name: &str,
        page: &doc::Page,
        shell: &BTreeSet<String>,
    ) -> Option<PageModel> {
        let path = format!("pages/{name}");
        let Some(kind) = self.doc.page_kinds.get(&page.kind) else {
            self.refuse(
                format!("{path}/kind"),
                format!("no page kind `{}` in page_kinds", page.kind),
            );
            return None;
        };
        if let Some(s) = &page.shell
            && !self.doc.shells.contains_key(s)
        {
            self.refuse(format!("{path}/shell"), format!("no shell `{s}`"));
        }
        // State: the kind's, then the page's.
        let mut states = kind.state.clone();
        states.extend(page.state.clone());
        let state = self.state_models(&states, &format!("{path}/state"));
        let scope = Scope {
            state: states.keys().cloned().collect(),
            params: page.params.keys().cloned().collect(),
            shell: shell.clone(),
            row: false,
            args: None,
        };
        // Sections: the kind's in their order, a page's of the same name in its place, then the
        // page's new ones.
        let mut sections: Vec<(String, &Node)> = Vec::new();
        for s in &kind.sections {
            let sname = s.name.clone().unwrap_or_default();
            let own = page
                .sections
                .iter()
                .find(|p| p.name.as_deref() == Some(&sname));
            match own {
                Some(p) => sections.push((format!("{path}/sections/{sname}"), p)),
                None => sections.push((format!("page_kinds/{}/sections/{sname}", page.kind), s)),
            }
        }
        for p in &page.sections {
            let pname = p.name.clone().unwrap_or_default();
            if !kind
                .sections
                .iter()
                .any(|s| s.name.as_deref() == Some(&pname))
            {
                sections.push((format!("{path}/sections/{pname}"), p));
            }
        }
        let mut header = kind.header.clone().unwrap_or_default();
        if let Some(h) = &page.header {
            if h.title.is_some() {
                header.title = h.title.clone();
            }
            if h.title_from.is_some() {
                header.title_from = h.title_from.clone();
            }
            if h.filters.is_some() {
                header.filters = h.filters.clone();
            }
            if !h.live.is_empty() {
                header.live = h.live.clone();
            }
        }
        let mut models: Vec<SectionModel> = Vec::new();
        for (spath, s) in sections {
            if let Some(m) = self.node(s, &spath, &scope, None) {
                models.push(m);
            }
        }
        let names: BTreeSet<String> = models.iter().map(|m| m.name.clone()).collect();
        let bar = match &header.filters {
            Some(f) => match models.iter().position(|m| &m.name == f) {
                Some(i) if matches!(models[i].kind, SectionKind::FilterBar { .. }) => {
                    Some(models.remove(i))
                }
                _ => {
                    self.refuse(
                        format!("{path}/header/filters"),
                        format!("`{f}` is no filter bar section of the page"),
                    );
                    None
                }
            },
            None => None,
        };
        let title_from = header.title_from.as_ref().and_then(|t| {
            match models
                .iter()
                .find(|m| m.name == t.section)
                .and_then(|m| m.read.as_ref())
            {
                Some(r) => {
                    self.reads
                        .entry(r.view.clone())
                        .or_default()
                        .fields
                        .insert(t.field.clone());
                    Some((r.var.clone(), t.field.clone()))
                }
                None => {
                    self.refuse(
                        format!("{path}/header/title_from/section"),
                        format!("`{}` is no section of the page that reads", t.section),
                    );
                    None
                }
            }
        });
        let layout = page.layout.clone().or_else(|| kind.layout.clone());
        let (main, side) = match layout {
            None | Some(Layout::Stack(_)) => (models, Vec::new()),
            Some(Layout::Columns(c)) => {
                if c.columns.len() > 2 {
                    self.refuse(
                        format!("{path}/layout/columns"),
                        "this generator lays out at most two columns, a wide and a narrow one",
                    );
                }
                let narrow: Vec<&doc::Column> = c
                    .columns
                    .iter()
                    .filter(|c| c.width == Some(Width::Narrow))
                    .collect();
                let mut side_names: Vec<String> = Vec::new();
                for col in &c.columns {
                    for s in &col.sections {
                        if !names.contains(s) && bar.as_ref().map(|b| &b.name) != Some(s) {
                            self.refuse(
                                format!("{path}/layout/columns/{}", col.name),
                                format!("no section `{s}` on the page"),
                            );
                        }
                    }
                }
                if c.columns.len() == 2 {
                    match narrow.as_slice() {
                        [one] => side_names = one.sections.clone(),
                        _ => self.refuse(
                            format!("{path}/layout/columns"),
                            "of two columns, one is narrow",
                        ),
                    }
                }
                let mut main = Vec::new();
                let mut side: Vec<(usize, SectionModel)> = Vec::new();
                for m in models {
                    match side_names.iter().position(|s| s == &m.name) {
                        Some(i) => side.push((i, m)),
                        None => main.push(m),
                    }
                }
                side.sort_by_key(|(i, _)| *i);
                (main, side.into_iter().map(|(_, m)| m).collect())
            }
        };
        let segments = if name == self.doc.navigation.home {
            Vec::new()
        } else {
            name.split('.').map(str::to_string).collect()
        };
        Some(PageModel {
            name: name.to_string(),
            component: component_name(name),
            segments,
            params: page.params.keys().cloned().collect(),
            state,
            title: header
                .title
                .filter(|t| t != "from_page")
                .or_else(|| page.title.clone())
                .unwrap_or_else(|| humanize(name)),
            title_from,
            bar,
            main,
            side,
        })
    }
}

trait NavName {
    fn to_string_name(&self) -> String;
}

impl NavName for doc::Navigate {
    fn to_string_name(&self) -> String {
        self.to.clone()
    }
}

pub fn resolve(doc: &Document) -> Result<App, Vec<Refusal>> {
    let mut r = Resolver {
        doc,
        refusals: Vec::new(),
        reads: BTreeMap::new(),
        commands: BTreeMap::new(),
        hosts: BTreeMap::new(),
        channel_uses: BTreeMap::new(),
        widget_channels: BTreeMap::new(),
    };
    if !doc.pages.contains_key(&doc.navigation.home) {
        r.refuse(
            "navigation/home",
            format!("no page `{}`", doc.navigation.home),
        );
    }
    for s in &doc.navigation.sections {
        for p in &s.pages {
            if !doc.pages.contains_key(p) {
                r.refuse(
                    format!("navigation/sections/{}/pages", s.name),
                    format!("no page `{p}`"),
                );
            }
        }
    }
    for p in &doc.navigation.hidden {
        if !doc.pages.contains_key(p) {
            r.refuse("navigation/hidden", format!("no page `{p}`"));
        }
    }
    let mut shell_states: BTreeMap<String, doc::State> = BTreeMap::new();
    for (sname, s) in &doc.shells {
        if !s
            .regions
            .values()
            .any(|reg| reg.kind == doc::RegionKind::PageOutlet)
        {
            r.refuse(
                format!("shells/{sname}/regions"),
                "a shell has a region of kind page_outlet",
            );
        }
        shell_states.extend(s.state.clone());
    }
    let shell_names: BTreeSet<String> = shell_states.keys().cloned().collect();
    let shell_state = r.state_models(&shell_states, "shells/app/state");
    let theme = doc.theme.as_ref().map(|t| {
        if !doc.themes.contains_key(&t.default) {
            r.refuse("theme/default", format!("no theme `{}`", t.default));
        }
        let state = t
            .chosen_by
            .as_ref()
            .and_then(|c| match c.strip_prefix("shell.") {
                Some(s) if shell_names.contains(s) => Some(s.to_string()),
                _ => {
                    r.refuse(
                        "theme/chosen_by",
                        format!("expression `{c}` is no shell state"),
                    );
                    None
                }
            });
        ThemeModel {
            default: t.default.clone(),
            themes: doc.themes.keys().cloned().collect(),
            state,
        }
    });
    let mut widgets: Vec<WidgetModel> = Vec::new();
    let mut files: BTreeMap<String, String> = BTreeMap::new();
    for (name, w) in &doc.widgets {
        if let Some(model) = r.widget(name, w) {
            if let Some(other) = files.insert(model.component.clone(), name.clone()) {
                r.refuse(
                    format!("widgets/{name}"),
                    format!(
                        "the widget writes `widgets/{}.vue`, the file of widget `{other}` too",
                        model.component
                    ),
                );
            }
            widgets.push(model);
        }
    }
    let mut pages = Vec::new();
    for (name, page) in &doc.pages {
        if let Some(p) = r.page(name, page, &shell_names) {
            pages.push(p);
        }
    }
    let nav = doc
        .shells
        .values()
        .any(|s| {
            s.regions
                .values()
                .any(|reg| reg.kind == doc::RegionKind::Navigation)
        })
        .then(|| {
            doc.navigation
                .sections
                .iter()
                .map(|s| NavModel {
                    label: s.label.clone().unwrap_or_else(|| humanize(&s.name)),
                    pages: s
                        .pages
                        .iter()
                        .map(|p| {
                            let page = doc.pages.get(p);
                            let label = page
                                .and_then(|pg| pg.nav.as_ref().and_then(|n| n.label.clone()))
                                .or_else(|| page.and_then(|pg| pg.title.clone()))
                                .unwrap_or_else(|| humanize(p));
                            (p.clone(), label)
                        })
                        .collect(),
                })
                .collect()
        });
    let channels = doc
        .channels
        .iter()
        .map(|(n, c)| {
            (
                n.clone(),
                ChannelSig {
                    events: c.carries.events.clone(),
                    view: c.carries.view.clone(),
                    per: c.session.as_ref().map(|s| s.per.clone()),
                },
            )
        })
        .collect();
    if !r.refusals.is_empty() {
        return Err(r.refusals);
    }
    Ok(App {
        name: doc.app.clone(),
        title: doc.title.clone().unwrap_or_else(|| doc.app.clone()),
        home: doc.navigation.home.clone(),
        pages,
        widgets,
        reads: r.reads,
        commands: r.commands,
        channels,
        hosts: r.hosts,
        theme,
        shell_state,
        nav,
    })
}
