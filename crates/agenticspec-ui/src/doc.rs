//! The constructs of `ess-ui/1` this generator draws, typed from the schema
//! (`schemas/ui/ess-ui.schema.yaml` of the ESS release `blocks/ess-inputs.yaml` requires).
//!
//! Every struct refuses a key it does not declare, so a construct the generator does not draw is
//! refused by its node path rather than ignored. Where the schema accepts a short form (a bare
//! field name for a field, a view name for a read, a string for an option), the type reads both.

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// A type that has a short form written as one string.
pub trait FromShort: Sized {
    fn from_short(s: String) -> Self;
}

/// Reads `T` from a string (its short form) or from a map, keeping the path of every key of the
/// map for the refusal of an unknown one.
fn short_or_full<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromShort + Deserialize<'de>,
{
    struct V<T>(PhantomData<T>);
    impl<'de, T: FromShort + Deserialize<'de>> Visitor<'de> for V<T> {
        type Value = T;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a name or a map")
        }
        fn visit_str<E: de::Error>(self, s: &str) -> Result<T, E> {
            Ok(T::from_short(s.to_string()))
        }
        fn visit_string<E: de::Error>(self, s: String) -> Result<T, E> {
            Ok(T::from_short(s))
        }
        fn visit_map<M: MapAccess<'de>>(self, m: M) -> Result<T, M::Error> {
            T::deserialize(de::value::MapAccessDeserializer::new(m))
        }
    }
    d.deserialize_any(V(PhantomData))
}

macro_rules! short_form {
    ($outer:ident, $inner:ident) => {
        impl<'de> Deserialize<'de> for $outer {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                short_or_full::<D, $inner>(d).map($outer)
            }
        }
        impl std::ops::Deref for $outer {
            type Target = $inner;
            fn deref(&self) -> &$inner {
                &self.0
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

impl Tone {
    pub fn word(self) -> &'static str {
        match self {
            Tone::Neutral => "neutral",
            Tone::Info => "info",
            Tone::Success => "success",
            Tone::Warning => "warning",
            Tone::Danger => "danger",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub format: String,
    pub app: String,
    #[serde(default)]
    pub title: Option<String>,
    pub model: String,
    #[serde(default)]
    pub actor: Option<Actor>,
    pub placement_profile: Profile,
    #[serde(default)]
    pub fixtures: Option<Fixtures>,
    #[serde(default)]
    pub unmapped: Vec<String>,
    #[serde(default)]
    pub tone_maps: BTreeMap<String, BTreeMap<String, Tone>>,
    #[serde(default)]
    pub label_maps: BTreeMap<String, BTreeMap<String, String>>,
    pub shells: BTreeMap<String, Shell>,
    #[serde(default)]
    pub themes: BTreeMap<String, Theme>,
    #[serde(default)]
    pub theme: Option<ThemeChoice>,
    #[serde(default)]
    pub channels: BTreeMap<String, Channel>,
    pub navigation: Navigation,
    #[serde(default)]
    pub page_kinds: BTreeMap<String, PageKind>,
    #[serde(default)]
    pub widgets: BTreeMap<String, Widget>,
    pub pages: BTreeMap<String, Page>,
}

/// An app-defined composite with typed parameters. The generator writes one single-file component
/// per widget, named by the widget, so the same widget gives the same file in every document.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Widget {
    pub summary: String,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, WidgetParam>,
    #[serde(default)]
    pub arrange: Option<Arrange>,
    pub body: Vec<Node>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WidgetParam {
    #[serde(rename = "type")]
    pub ty: Value,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub note: Option<String>,
}

/// How a widget's body nodes are arranged; this generator stacks them in a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrange {
    Column,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    FromSession,
    Anonymous,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Thin,
    Fat,
    Hybrid,
}

/// Sample data for renderers without a backend; the generated app reads its server, so the
/// generator reads the index only to accept it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixtures {
    #[serde(default)]
    pub dir: Option<String>,
    #[serde(default)]
    pub index: Option<String>,
    #[serde(default)]
    pub views: BTreeMap<String, String>,
    #[serde(default)]
    pub derived: BTreeMap<String, Value>,
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shell {
    pub regions: BTreeMap<String, Region>,
    #[serde(default)]
    pub state: BTreeMap<String, State>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub kind: RegionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    PageOutlet,
    Notifications,
    Navigation,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[serde(rename = "type")]
    pub ty: Value,
    pub class: StateClass,
    #[serde(default)]
    pub store: Option<Store>,
    #[serde(default)]
    pub pinned: Option<bool>,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateClass {
    PageState,
    Selection,
    ComponentState,
    Draft,
    Preference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Store {
    Memory,
    Url,
    SessionStorage,
    LocalStorage,
}

/// A theme of the document. The look of every theme is ui/kit's, so a theme names no override:
/// a token it overrides is refused as an unknown property.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeChoice {
    pub default: String,
    #[serde(default)]
    pub chosen_by: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    pub carries: Carries,
    pub direction: Direction,
    pub delivery: Delivery,
    pub resume: Resume,
    #[serde(default)]
    pub scope: Option<ChannelScope>,
    #[serde(default)]
    pub session: Option<ChannelSession>,
    #[serde(default)]
    pub lifecycle: Vec<String>,
    #[serde(default)]
    pub reconnect: Option<Value>,
    #[serde(default)]
    pub stale_after: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Carries {
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub view: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    ServerToClient,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    LatestValue,
    EveryEvent,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resume {
    FromLastSeen,
    Refetch,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelScope {
    Account,
    User,
    Session,
    Param,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSession {
    pub per: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigation {
    pub home: String,
    pub sections: Vec<NavSection>,
    #[serde(default)]
    pub hidden: Vec<String>,
    #[serde(default)]
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavSection {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    pub pages: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageKind {
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub state: BTreeMap<String, State>,
    #[serde(default)]
    pub sections: Vec<Node>,
    #[serde(default)]
    pub header: Option<Header>,
    #[serde(default)]
    pub layout: Option<Layout>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub kind: String,
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub nav: Option<NavEntry>,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    #[serde(default)]
    pub layout: Option<Layout>,
    #[serde(default)]
    pub state: BTreeMap<String, State>,
    #[serde(default)]
    pub header: Option<Header>,
    #[serde(default)]
    pub sections: Vec<Node>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavEntry {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_from: Option<TitleFrom>,
    #[serde(default)]
    pub filters: Option<String>,
    #[serde(default)]
    pub live: Vec<String>,
    /// Primary actions, drawn above the page's sections.
    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TitleFrom {
    pub section: String,
    pub field: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Layout {
    Stack(StackWord),
    Columns(Columns),
    Areas(AreasLayout),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AreasLayout {
    pub areas: Areas,
}

/// A grid of area names row by row, and the sections each area holds.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Areas {
    pub grid: Vec<Vec<String>>,
    pub place: BTreeMap<String, Place>,
}

/// The sections of an area: a list (each in a frame of its own), or a record that frames them.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Place {
    Sections(Vec<String>),
    Framed(Framed),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Framed {
    pub sections: Vec<String>,
    #[serde(default)]
    pub frame: Option<FrameKind>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub collapsed: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameKind {
    Each,
    Shared,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackWord {
    Stack,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Columns {
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub name: String,
    pub sections: Vec<String>,
    #[serde(default)]
    pub width: Option<Width>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Width {
    Narrow,
    Normal,
    Wide,
}

/// The composites this generator draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Component {
    Collection,
    Record,
    Form,
    FilterBar,
    Choice,
}

impl Component {
    pub fn word(self) -> &'static str {
        match self {
            Component::Collection => "collection",
            Component::Record => "record",
            Component::Form => "form",
            Component::FilterBar => "filter_bar",
            Component::Choice => "choice",
        }
    }

    pub fn from_word(word: &str) -> Option<Component> {
        [
            Component::Collection,
            Component::Record,
            Component::Form,
            Component::FilterBar,
            Component::Choice,
        ]
        .into_iter()
        .find(|c| c.word() == word)
    }
}

/// What a node's `component` names: a composite this generator draws, or a widget of the
/// document (any other name; `check::parse` refuses one the document does not declare).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentRef {
    Kind(Component),
    Widget(String),
}

impl<'de> Deserialize<'de> for ComponentRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match Component::from_word(&s) {
            Some(c) => ComponentRef::Kind(c),
            None => ComponentRef::Widget(s),
        })
    }
}

/// A section, a nested node or a host slot: every property any of them takes. Which properties
/// a node may carry is decided by its `component` (or `host`) after reading, by
/// `check::allowed`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub component: Option<ComponentRef>,
    #[serde(default)]
    pub args: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub stands_in_for: Option<String>,
    #[serde(default)]
    pub commands: Option<Vec<String>>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub collapsed: Option<bool>,
    #[serde(default)]
    pub reads: Option<Reads>,
    #[serde(default)]
    pub live: Option<Live>,
    #[serde(default)]
    pub states: Option<States>,
    #[serde(default)]
    pub columns: Option<Vec<Field>>,
    #[serde(default)]
    pub style: Option<Style>,
    #[serde(default)]
    pub row_actions: Option<Vec<Action>>,
    #[serde(default)]
    pub fields: Option<Vec<Field>>,
    #[serde(default)]
    pub actions: Option<Vec<Action>>,
    #[serde(default)]
    pub does: Option<String>,
    #[serde(default)]
    pub submit: Option<Submit>,
    #[serde(default)]
    pub result: Option<Box<Node>>,
    #[serde(default)]
    pub binds: Option<Binds>,
    #[serde(default)]
    pub choices: Option<Vec<Node>>,
    #[serde(default)]
    pub search: Option<Search>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<ChoiceOption>>,
}

impl Node {
    /// The composite the node draws, when it names one rather than a widget.
    pub fn kind(&self) -> Option<Component> {
        match &self.component {
            Some(ComponentRef::Kind(c)) => Some(*c),
            _ => None,
        }
    }

    /// The widget the node uses, when it names one.
    pub fn widget(&self) -> Option<&str> {
        match &self.component {
            Some(ComponentRef::Widget(w)) => Some(w),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    Table,
    List,
}

/// `binds`: a list of expressions on a filter bar, one expression on a choice.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Binds {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldFull {
    pub field: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default, rename = "as")]
    pub as_: Option<FieldAs>,
    #[serde(default)]
    pub labels: Option<Labels>,
    #[serde(default)]
    pub tone_by: Option<String>,
    #[serde(default)]
    pub tone: Option<Tone>,
    #[serde(default)]
    pub sortable: Option<bool>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub stands_in_for: Option<String>,
}

impl FromShort for FieldFull {
    fn from_short(s: String) -> Self {
        FieldFull {
            field: s,
            name: None,
            label: None,
            as_: None,
            labels: None,
            tone_by: None,
            tone: None,
            sortable: None,
            host: None,
            reason: None,
            stands_in_for: None,
        }
    }
}

/// A column of a collection, a field of a record or an input of a form.
#[derive(Debug, Clone)]
pub struct Field(pub FieldFull);
short_form!(Field, FieldFull);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldAs {
    Text,
    Number,
    Badge,
    Toggle,
    List,
    Timestamp,
    Date,
    Time,
    Percent,
    Json,
}

impl FieldAs {
    pub fn word(self) -> &'static str {
        match self {
            FieldAs::Text => "text",
            FieldAs::Number => "number",
            FieldAs::Badge => "badge",
            FieldAs::Toggle => "toggle",
            FieldAs::List => "list",
            FieldAs::Timestamp => "timestamp",
            FieldAs::Date => "date",
            FieldAs::Time => "time",
            FieldAs::Percent => "percent",
            FieldAs::Json => "json",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Labels {
    Name(String),
    Map(BTreeMap<String, String>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub does: Option<String>,
    #[serde(default)]
    pub bind: BTreeMap<String, String>,
    #[serde(default)]
    pub navigate: Option<Navigate>,
    #[serde(default)]
    pub copy: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default, rename = "as")]
    pub as_: Option<ActionAs>,
    /// Shows the action only while it holds, beyond the action set of the scope.
    #[serde(default)]
    pub visible: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAs {
    Button,
    Link,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigate {
    pub to: String,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadsFull {
    #[serde(default)]
    pub view: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub fixture: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    #[serde(default)]
    pub paging: Option<Paging>,
    #[serde(default)]
    pub filter: Option<String>,
}

impl FromShort for ReadsFull {
    fn from_short(s: String) -> Self {
        ReadsFull {
            view: Some(s),
            placeholder: None,
            fixture: None,
            key: None,
            params: BTreeMap::new(),
            paging: None,
            filter: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reads(pub ReadsFull);
short_form!(Reads, ReadsFull);

impl Reads {
    /// The view, or the placeholder that stands for one.
    pub fn name(&self) -> &str {
        self.view
            .as_deref()
            .or(self.placeholder.as_deref())
            .unwrap_or("")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Paging {
    None,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Live {
    pub channel: String,
    #[serde(default)]
    pub on: Vec<String>,
    pub effect: Effect,
    #[serde(default, rename = "match")]
    pub match_: Option<String>,
    #[serde(default)]
    pub only_if: Option<String>,
    #[serde(default)]
    pub coalesce: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Refetch,
    InsertOrPatch,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct States {
    #[serde(default)]
    pub loading: Option<Loading>,
    #[serde(default)]
    pub empty: Option<Empty>,
    #[serde(default)]
    pub failed: Option<Failed>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Loading {
    Skeleton,
    Spinner,
    None,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Empty {
    #[serde(default)]
    pub title: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failed {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub retry: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submit {
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Search {
    pub binds: String,
    #[serde(default)]
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionFull {
    pub value: Value,
    pub label: String,
}

impl FromShort for OptionFull {
    fn from_short(s: String) -> Self {
        OptionFull {
            value: Value::String(s.clone()),
            label: s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChoiceOption(pub OptionFull);
short_form!(ChoiceOption, OptionFull);
