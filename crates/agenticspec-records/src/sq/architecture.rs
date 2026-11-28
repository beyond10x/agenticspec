//! The fact base of an architecture (`agenticspec-architecture/1`): the relations a selector of a
//! patch reads. The facts come from the reader of the document (`agenticspec-patch`); this module
//! holds their relations and builds the fact base the evaluator runs over, beside the session
//! fact base and apart from it: a session relation is unknown here, and these are unknown to a
//! session query.

use super::eval::{self, Program, Schema};
use super::facts::{FactBase, Table, Val, vs};
use super::syntax;

/// The relations of an architecture and their fields, in the order positional atoms match them.
pub const SCHEMA: Schema = &[
    ("part", &["id", "block", "count"]),
    ("setting", &["part", "name", "value"]),
    ("link", &["index", "from", "to", "carries", "over"]),
    ("marker", &["path", "value"]),
];

/// A part: its key under `parts`, its `block` and its `count` (1 when absent).
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub id: String,
    pub block: Val,
    pub count: i64,
}

/// One other key of a part and its value; a list gives one setting per element.
#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub part: String,
    pub name: String,
    pub value: Val,
}

/// A link at its 0-based position in `links`; `over` is null when absent.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub index: i64,
    pub from: Val,
    pub to: Val,
    pub carries: Val,
    pub over: Val,
}

/// A marked place: its path (`parts.worker.count`) and its value as built.
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub path: String,
    pub value: Val,
}

/// The facts of one architecture document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Facts {
    /// The architecture's name, held where a session fact base holds its session id.
    pub name: String,
    pub parts: Vec<Part>,
    pub settings: Vec<Setting>,
    pub links: Vec<Link>,
    pub markers: Vec<Marker>,
}

/// Compiles sq over the relations of an architecture.
pub fn compile(src: &str) -> Result<Program, String> {
    eval::compile_in(&syntax::parse(src)?, SCHEMA)
}

/// The fact base of `facts`, one table per relation of [`SCHEMA`]. Every fact is definite.
pub fn fact_base(facts: &Facts) -> FactBase {
    let mut fb = FactBase {
        session: facts.name.clone(),
        ..FactBase::default()
    };
    let mut tables: Vec<Table> = SCHEMA
        .iter()
        .map(|(_, cols)| Table::new(cols.to_vec()))
        .collect();
    for p in &facts.parts {
        tables[0].push(vec![vs(&p.id), p.block.clone(), Val::Int(p.count)], vec![]);
    }
    for s in &facts.settings {
        tables[1].push(vec![vs(&s.part), vs(&s.name), s.value.clone()], vec![]);
    }
    for l in &facts.links {
        tables[2].push(
            vec![
                Val::Int(l.index),
                l.from.clone(),
                l.to.clone(),
                l.carries.clone(),
                l.over.clone(),
            ],
            vec![],
        );
    }
    for m in &facts.markers {
        tables[3].push(vec![vs(&m.path), m.value.clone()], vec![]);
    }
    for ((name, _), table) in SCHEMA.iter().zip(tables) {
        fb.tables.insert(name, table);
    }
    fb
}
