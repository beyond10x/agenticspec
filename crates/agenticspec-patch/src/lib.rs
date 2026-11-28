//! agenticspec-patch: the patch language of agenticspec architectures.
//!
//! A patch (`agenticspec-patch/1`) is an ordered list of rules. A rule's body is a selector in sq
//! over the facts of an architecture (`part`, `setting`, `link`, `marker`); its head is a
//! transform, one of a closed set of verbs. This applier applies `set` and `add`. A patch applies
//! atomically: every rule in order, each over the facts the rules before it left, then the
//! structural check of `agenticspec-sim` on the result. A refusal changes nothing and names the
//! 1-based position of the refusing rule (none when the check refused) and the error.

mod diff;
pub mod facts;
mod text;

use std::collections::HashMap;
use std::fmt;

use agenticspec_records::sq::architecture;
use agenticspec_records::sq::eval::{self, Program};
use agenticspec_records::sq::facts::Val;
use agenticspec_records::sq::syntax;
use agenticspec_sim::check::{self, Refusal};
use agenticspec_sim::doc::{format_duration, is_duration};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub use diff::Change;
pub use facts::{fact_base, facts};
use text::{Arg, Head};

/// The format of a patch document.
pub const FORMAT: &str = "agenticspec-patch/1";

/// The verbs of `agenticspec-patch/1` (`agentic.architecture.PatchVerb`).
pub const VERBS: &[&str] = &[
    "set",
    "add",
    "remove",
    "connect",
    "replicate",
    "wrap",
    "prepend",
    "append",
    "mark",
    "unmark",
];

/// The verbs this applier applies.
pub const APPLIED_VERBS: &[&str] = &["set", "add"];

/// The fields of a link a patch may write.
const LINK_FIELDS: &[&str] = &["from", "to", "carries", "over"];

/// The name of the error a patch is refused with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorName {
    /// The patch has no rule.
    EmptyPatch,
    /// The document's `format` is not `agenticspec-patch/1`.
    UnknownFormat,
    /// The rule is not `head :- body.` or `head.`, or its head or body does not parse.
    UnparsableRule,
    /// The head's verb is not a verb of `agenticspec-patch/1`.
    UnknownVerb,
    /// The verb is one of `agenticspec-patch/1` that this applier does not apply.
    UnsupportedVerb,
    /// The head's arguments do not fit its verb.
    InvalidHead,
    /// The body names a relation or field an architecture does not have.
    InvalidSelector,
    /// A variable of the head is bound by no atom of the body.
    UnboundVariable,
    /// `set` names a part id that is no part.
    NoSuchPart,
    /// `set` names a link index that is no link.
    NoSuchLink,
    /// `set` writes a part's `id` or a link's `index`.
    ReadOnlyField,
    /// A link has no such field.
    UnknownField,
    /// `add` leaves out a field the part or link needs.
    MissingField,
    /// A count is not a whole number of at least 1.
    InvalidCount,
    /// One rule gives one place two values.
    ConflictingValues,
    /// `add` names a part id that is taken.
    PartNameTaken,
    /// `add` names a link that exists.
    LinkExists,
    /// The structural check refuses the result.
    CheckRefused,
}

impl ErrorName {
    /// Every error, in the order of the enum.
    pub const ALL: [ErrorName; 18] = [
        ErrorName::EmptyPatch,
        ErrorName::UnknownFormat,
        ErrorName::UnparsableRule,
        ErrorName::UnknownVerb,
        ErrorName::UnsupportedVerb,
        ErrorName::InvalidHead,
        ErrorName::InvalidSelector,
        ErrorName::UnboundVariable,
        ErrorName::NoSuchPart,
        ErrorName::NoSuchLink,
        ErrorName::ReadOnlyField,
        ErrorName::UnknownField,
        ErrorName::MissingField,
        ErrorName::InvalidCount,
        ErrorName::ConflictingValues,
        ErrorName::PartNameTaken,
        ErrorName::LinkExists,
        ErrorName::CheckRefused,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ErrorName::EmptyPatch => "EmptyPatch",
            ErrorName::UnknownFormat => "UnknownFormat",
            ErrorName::UnparsableRule => "UnparsableRule",
            ErrorName::UnknownVerb => "UnknownVerb",
            ErrorName::UnsupportedVerb => "UnsupportedVerb",
            ErrorName::InvalidHead => "InvalidHead",
            ErrorName::InvalidSelector => "InvalidSelector",
            ErrorName::UnboundVariable => "UnboundVariable",
            ErrorName::NoSuchPart => "NoSuchPart",
            ErrorName::NoSuchLink => "NoSuchLink",
            ErrorName::ReadOnlyField => "ReadOnlyField",
            ErrorName::UnknownField => "UnknownField",
            ErrorName::MissingField => "MissingField",
            ErrorName::InvalidCount => "InvalidCount",
            ErrorName::ConflictingValues => "ConflictingValues",
            ErrorName::PartNameTaken => "PartNameTaken",
            ErrorName::LinkExists => "LinkExists",
            ErrorName::CheckRefused => "CheckRefused",
        }
    }
}

impl fmt::Display for ErrorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A patch refused as a whole: nothing changed.
#[derive(Debug, Clone)]
pub struct Refused {
    /// The 1-based position of the refusing rule; none when the patch as a whole or the check of
    /// its result refused.
    pub rule: Option<usize>,
    pub error: ErrorName,
    pub message: String,
    /// When the check refused: the changes it judged. Empty otherwise.
    pub diff: Vec<Change>,
    /// When the check refused: its refusals. Empty otherwise.
    pub check: Vec<Refusal>,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.rule {
            Some(n) => write!(f, "rule {n}: {}: {}", self.error, self.message),
            None => write!(f, "{}: {}", self.error, self.message),
        }
    }
}

impl std::error::Error for Refused {}

fn refuse(rule: Option<usize>, error: ErrorName, message: impl Into<String>) -> Refused {
    Refused {
        rule,
        error,
        message: message.into(),
        diff: Vec::new(),
        check: Vec::new(),
    }
}

/// A patch applied: the new document, what changed, and the check, which passed.
#[derive(Debug, Clone)]
pub struct Applied {
    pub document: Value,
    pub diff: Vec<Change>,
    /// The parts and links the patch changed (a part's settings belong to its part).
    pub changed_rows: usize,
    /// The check of the result: empty, since a refused check refuses the patch.
    pub check: Vec<Refusal>,
}

/// What a patch would change, and the check of the result; nothing is written.
#[derive(Debug, Clone)]
pub struct DryRun {
    pub diff: Vec<Change>,
    pub changed_rows: usize,
    pub check: Vec<Refusal>,
}

/// A patch: an optional name and its rules, in order, each as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub name: Option<String>,
    pub rules: Vec<String>,
}

impl Patch {
    /// The rules of a patch's text form, one after another, each ending with `.`.
    pub fn from_text(text: &str) -> Result<Patch, Refused> {
        let rules = text::split(text);
        if rules.is_empty() {
            return Err(refuse(None, ErrorName::EmptyPatch, "the patch has no rule"));
        }
        Ok(Patch { name: None, rules })
    }

    /// A patch document: `format: agenticspec-patch/1`, an optional `name` and `rules`, a list
    /// of rules as text.
    pub fn from_document(doc: &Value) -> Result<Patch, Refused> {
        if doc["format"].as_str() != Some(FORMAT) {
            return Err(refuse(
                None,
                ErrorName::UnknownFormat,
                format!("the format is {}, not {FORMAT}", doc["format"]),
            ));
        }
        let mut rules = Vec::new();
        for (i, r) in doc["rules"].as_array().into_iter().flatten().enumerate() {
            let Some(text) = r.as_str() else {
                return Err(refuse(
                    Some(i + 1),
                    ErrorName::UnparsableRule,
                    "a rule is a string",
                ));
            };
            rules.push(text.trim().to_string());
        }
        if rules.is_empty() {
            return Err(refuse(None, ErrorName::EmptyPatch, "the patch has no rule"));
        }
        Ok(Patch {
            name: doc["name"].as_str().map(str::to_string),
            rules,
        })
    }

    /// The patch as a document of `agenticspec-patch/1`.
    pub fn to_document(&self) -> Value {
        let mut doc = Map::new();
        doc.insert("format".into(), Value::from(FORMAT));
        if let Some(name) = &self.name {
            doc.insert("name".into(), Value::from(name.as_str()));
        }
        doc.insert("rules".into(), Value::from(self.rules.clone()));
        Value::Object(doc)
    }

    /// The digest of the patch: SHA-256 of its document's canonical JSON.
    pub fn digest(&self) -> String {
        digest(&self.to_document())
    }
}

/// The verb of a rule's head when it is a verb of `agenticspec-patch/1`.
pub fn verb_of(rule: &str) -> Option<&'static str> {
    let v = text::verb(rule)?;
    VERBS.iter().find(|x| **x == v).copied()
}

/// SHA-256, in hex, of the canonical JSON of a document: object keys sorted, no whitespace.
pub fn digest(doc: &Value) -> String {
    let mut text = String::new();
    canonical(doc, &mut text);
    let hash = Sha256::digest(text.as_bytes());
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

fn canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(o) => {
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::from(k.as_str()).to_string());
                out.push(':');
                canonical(&o[k.as_str()], out);
            }
            out.push('}');
        }
        Value::Array(xs) => {
            out.push('[');
            for (i, x) in xs.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonical(x, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verb {
    Set,
    Add,
}

/// One rule, parsed and its selector compiled.
struct Rule {
    position: usize,
    verb: Verb,
    head: Head,
    /// The selector and the names of its columns; none for a rule without a body.
    selector: Option<(Program, Vec<String>)>,
}

fn compile(position: usize, text: &str) -> Result<Rule, Refused> {
    let at = Some(position);
    let unparsable = |m: String| refuse(at, ErrorName::UnparsableRule, m);
    let (head_text, body) = text::split_rule(text).map_err(unparsable)?;
    let verb = match text::verb(head_text) {
        Some("set") => Verb::Set,
        Some("add") => Verb::Add,
        Some(v) if VERBS.contains(&v) => {
            return Err(refuse(
                at,
                ErrorName::UnsupportedVerb,
                format!(
                    "`{v}` is a verb of {FORMAT} that this applier does not apply; it applies {}",
                    APPLIED_VERBS.join(" and ")
                ),
            ));
        }
        Some(v) => {
            return Err(refuse(
                at,
                ErrorName::UnknownVerb,
                format!(
                    "`{v}` is not a verb of {FORMAT}; its verbs are {}",
                    VERBS.join(", ")
                ),
            ));
        }
        None => return Err(unparsable("a rule's head starts with its verb".into())),
    };
    let head = text::parse_head(head_text).map_err(unparsable)?;
    shape(&head, verb).map_err(|m| refuse(at, ErrorName::InvalidHead, m))?;
    let selector = match body {
        None => None,
        Some(body) => {
            let module = syntax::parse(&format!("?- {body}."))
                .map_err(|e| unparsable(format!("the body: {e}")))?;
            if module.queries.len() != 1
                || !module.rules.is_empty()
                || !module.checks.is_empty()
                || !module.names.is_empty()
            {
                return Err(unparsable("the body is one selector".into()));
            }
            let program = eval::compile_in(&module, architecture::SCHEMA)
                .map_err(|e| refuse(at, ErrorName::InvalidSelector, e))?;
            let columns = program.queries[0].columns.clone();
            Some((program, columns))
        }
    };
    let bound: &[String] = selector.as_ref().map_or(&[], |(_, c)| c);
    for v in variables(&head) {
        if !bound.contains(&v) {
            return Err(refuse(
                at,
                ErrorName::UnboundVariable,
                format!("`{v}` in the head is bound by no atom of the body"),
            ));
        }
    }
    Ok(Rule {
        position,
        verb,
        head,
        selector,
    })
}

fn value_arg(a: &Arg) -> bool {
    matches!(
        a,
        Arg::Var(_) | Arg::Str(_) | Arg::Int(_) | Arg::Duration(_) | Arg::Bool(_)
    )
}

/// Whether the head's arguments fit its verb: `set(Target, field, Value)` or `add(part{...})`,
/// `add(link{...})`.
fn shape(head: &Head, verb: Verb) -> Result<(), String> {
    match verb {
        Verb::Set => {
            let [target, field, value] = head.args.as_slice() else {
                return Err(format!(
                    "set takes a part id or link index, a field and a value: set(P, count, 3); this head has {} argument(s)",
                    head.args.len()
                ));
            };
            if !matches!(target, Arg::Var(_) | Arg::Str(_) | Arg::Int(_)) {
                return Err("the first argument of set is a part id or a link index".into());
            }
            if !matches!(field, Arg::Name(_) | Arg::Str(_)) {
                return Err("the second argument of set is a field name".into());
            }
            if !value_arg(value) {
                return Err("the third argument of set is a value or a variable".into());
            }
            Ok(())
        }
        Verb::Add => {
            let [Arg::Record(rel, fields)] = head.args.as_slice() else {
                return Err(
                    "add takes one part{...} or link{...}: add(part{id: \"x\", block: \"model\"})"
                        .into(),
                );
            };
            if rel != "part" && rel != "link" {
                return Err(format!(
                    "add takes part{{...}} or link{{...}}, not {rel}{{...}}"
                ));
            }
            let mut seen: Vec<&str> = Vec::new();
            for (f, a) in fields {
                if seen.contains(&f.as_str()) {
                    return Err(format!("field `{f}` is given twice"));
                }
                seen.push(f);
                if !value_arg(a) {
                    return Err(format!("field `{f}` takes a value or a variable"));
                }
            }
            Ok(())
        }
    }
}

fn variables(head: &Head) -> Vec<String> {
    fn walk(a: &Arg, out: &mut Vec<String>) {
        match a {
            Arg::Var(v) if !out.contains(v) => out.push(v.clone()),
            Arg::Record(_, fields) => fields.iter().for_each(|(_, a)| walk(a, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    head.args.iter().for_each(|a| walk(a, &mut out));
    out
}

/// A value a head writes: a value of the facts, or a duration literal.
#[derive(Debug, Clone, PartialEq)]
enum Lit {
    Val(Val),
    Duration(i64),
}

type Env = HashMap<String, Val>;

fn lit(a: &Arg, env: &Env) -> Lit {
    match a {
        Arg::Var(v) => Lit::Val(env.get(v).cloned().unwrap_or(Val::Null)),
        Arg::Str(s) | Arg::Name(s) => Lit::Val(Val::Str(s.as_str().into())),
        Arg::Int(n) => Lit::Val(Val::Int(*n)),
        Arg::Duration(ms) => Lit::Duration(*ms),
        Arg::Bool(b) => Lit::Val(Val::Bool(*b)),
        Arg::Wild | Arg::Record(..) => Lit::Val(Val::Null),
    }
}

fn duration_text(ms: i64) -> String {
    if ms % 1000 == 0 {
        format_duration(ms as f64 / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

/// The JSON a value is written as. An integer written where a duration stands is milliseconds,
/// as the facts hold durations, and is written as a duration.
fn json(l: &Lit, existing: Option<&Value>) -> Value {
    match l {
        Lit::Duration(ms) => Value::from(duration_text(*ms)),
        Lit::Val(Val::Int(ms)) if existing.is_some_and(is_duration) => {
            Value::from(duration_text(*ms))
        }
        Lit::Val(Val::Int(n)) => Value::from(*n),
        Lit::Val(Val::Str(s)) => Value::from(s.as_ref()),
        Lit::Val(Val::Bool(b)) => Value::from(*b),
        Lit::Val(Val::Null) => Value::Null,
    }
}

/// The answers of a rule's selector over the document, each a binding of its columns; one empty
/// binding for a rule without a body.
fn answers(rule: &Rule, doc: &Value) -> Vec<Env> {
    let Some((program, columns)) = &rule.selector else {
        return vec![Env::new()];
    };
    let answers = eval::run(program, &facts::fact_base(doc));
    answers.queries[0]
        .rows
        .iter()
        .map(|(vals, _)| columns.iter().cloned().zip(vals.iter().cloned()).collect())
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
enum Target {
    Part(String),
    Link(usize),
}

fn is_marker(v: &Value) -> bool {
    v.as_object().is_some_and(|o| o.contains_key("vary"))
}

fn run_set(rule: &Rule, doc: &mut Value) -> Result<(), Refused> {
    let at = Some(rule.position);
    let [target, field, value] = rule.head.args.as_slice() else {
        return Err(refuse(
            at,
            ErrorName::InvalidHead,
            "set takes three arguments",
        ));
    };
    let field = match field {
        Arg::Name(s) | Arg::Str(s) => s.clone(),
        _ => return Err(refuse(at, ErrorName::InvalidHead, "set names a field")),
    };
    let mut actions: Vec<(Target, Lit)> = Vec::new();
    for env in answers(rule, doc) {
        let t = match lit(target, &env) {
            Lit::Val(Val::Str(id)) => Target::Part(id.to_string()),
            Lit::Val(Val::Int(i)) => match usize::try_from(i) {
                Ok(i) => Target::Link(i),
                Err(_) => {
                    return Err(refuse(at, ErrorName::NoSuchLink, format!("no link {i}")));
                }
            },
            other => {
                return Err(refuse(
                    at,
                    ErrorName::InvalidHead,
                    format!("set names a part id or a link index, not {other:?}"),
                ));
            }
        };
        let v = lit(value, &env);
        match actions.iter().find(|(x, _)| *x == t) {
            Some((_, w)) if *w == v => {}
            Some((_, w)) => {
                return Err(refuse(
                    at,
                    ErrorName::ConflictingValues,
                    format!("{t:?}.{field} is given {w:?} and {v:?}"),
                ));
            }
            None => actions.push((t, v)),
        }
    }
    for (t, v) in actions {
        match t {
            Target::Part(id) => set_part(doc, &id, &field, &v, at)?,
            Target::Link(i) => set_link(doc, i, &field, &v, at)?,
        }
    }
    Ok(())
}

fn set_part(
    doc: &mut Value,
    id: &str,
    field: &str,
    v: &Lit,
    at: Option<usize>,
) -> Result<(), Refused> {
    if field == "id" {
        return Err(refuse(
            at,
            ErrorName::ReadOnlyField,
            "a part's id is its key; set does not rename a part",
        ));
    }
    if field == "count" && !matches!(v, Lit::Val(Val::Int(n)) if *n >= 1) {
        return Err(refuse(
            at,
            ErrorName::InvalidCount,
            format!("a count is a whole number of at least 1, not {v:?}"),
        ));
    }
    let Some(part) = doc["parts"].get_mut(id).and_then(Value::as_object_mut) else {
        return Err(refuse(at, ErrorName::NoSuchPart, format!("no part `{id}`")));
    };
    if *v == Lit::Val(Val::Null) {
        part.remove(field);
    } else if let Some(place) = part.get_mut(field).filter(|p| is_marker(p)) {
        // A marked place keeps its `vary`; only its value changes.
        let new = json(v, place.get("value"));
        place["value"] = new;
    } else {
        let new = json(v, part.get(field));
        part.insert(field.to_string(), new);
    }
    Ok(())
}

fn set_link(
    doc: &mut Value,
    i: usize,
    field: &str,
    v: &Lit,
    at: Option<usize>,
) -> Result<(), Refused> {
    if field == "index" {
        return Err(refuse(
            at,
            ErrorName::ReadOnlyField,
            "a link's index is its position in links",
        ));
    }
    if !LINK_FIELDS.contains(&field) {
        return Err(refuse(
            at,
            ErrorName::UnknownField,
            format!(
                "a link has no field `{field}`; its fields are {}",
                LINK_FIELDS.join(", ")
            ),
        ));
    }
    let Some(link) = doc["links"].get_mut(i).and_then(Value::as_object_mut) else {
        return Err(refuse(at, ErrorName::NoSuchLink, format!("no link {i}")));
    };
    if *v == Lit::Val(Val::Null) {
        if field != "over" {
            return Err(refuse(
                at,
                ErrorName::MissingField,
                format!("a link needs `{field}`"),
            ));
        }
        link.remove(field);
    } else {
        let new = json(v, link.get(field));
        link.insert(field.to_string(), new);
    }
    Ok(())
}

fn run_add(rule: &Rule, doc: &mut Value) -> Result<(), Refused> {
    let at = Some(rule.position);
    let [Arg::Record(rel, fields)] = rule.head.args.as_slice() else {
        return Err(refuse(at, ErrorName::InvalidHead, "add takes one record"));
    };
    let mut made: Vec<Map<String, Value>> = Vec::new();
    for env in answers(rule, doc) {
        let mut row = Map::new();
        for (f, a) in fields {
            let v = lit(a, &env);
            if v != Lit::Val(Val::Null) {
                row.insert(f.clone(), json(&v, None));
            }
        }
        if !made.contains(&row) {
            made.push(row);
        }
    }
    for row in made {
        if rel == "part" {
            add_part(doc, row, at)?;
        } else {
            add_link(doc, row, at)?;
        }
    }
    Ok(())
}

fn add_part(
    doc: &mut Value,
    mut row: Map<String, Value>,
    at: Option<usize>,
) -> Result<(), Refused> {
    let id = match row.remove("id") {
        Some(Value::String(id)) => id,
        Some(other) => {
            return Err(refuse(
                at,
                ErrorName::InvalidHead,
                format!("a part's id is a string, not {other}"),
            ));
        }
        None => return Err(refuse(at, ErrorName::MissingField, "a part needs `id`")),
    };
    if !row.contains_key("block") {
        return Err(refuse(
            at,
            ErrorName::MissingField,
            format!("part `{id}` needs `block`"),
        ));
    }
    if let Some(c) = row.get("count")
        && !c.as_i64().is_some_and(|n| n >= 1)
    {
        return Err(refuse(
            at,
            ErrorName::InvalidCount,
            format!("a count is a whole number of at least 1, not {c}"),
        ));
    }
    if !doc["parts"].is_object() {
        doc["parts"] = Value::Object(Map::new());
    }
    let Some(parts) = doc["parts"].as_object_mut() else {
        return Err(refuse(
            at,
            ErrorName::InvalidHead,
            "the document has no parts",
        ));
    };
    if parts.contains_key(&id) {
        return Err(refuse(
            at,
            ErrorName::PartNameTaken,
            format!("part `{id}` exists"),
        ));
    }
    parts.insert(id, Value::Object(row));
    Ok(())
}

fn add_link(doc: &mut Value, row: Map<String, Value>, at: Option<usize>) -> Result<(), Refused> {
    for f in row.keys() {
        if !LINK_FIELDS.contains(&f.as_str()) {
            return Err(refuse(
                at,
                ErrorName::UnknownField,
                format!(
                    "a link has no field `{f}`; its fields are {}",
                    LINK_FIELDS.join(", ")
                ),
            ));
        }
    }
    for f in ["from", "to", "carries"] {
        if !row.contains_key(f) {
            return Err(refuse(
                at,
                ErrorName::MissingField,
                format!("a link needs `{f}`"),
            ));
        }
    }
    if !doc["links"].is_array() {
        doc["links"] = Value::Array(Vec::new());
    }
    let link = Value::Object(row);
    let Some(links) = doc["links"].as_array_mut() else {
        return Err(refuse(
            at,
            ErrorName::InvalidHead,
            "the document has no links",
        ));
    };
    let same = |l: &Value| LINK_FIELDS.iter().all(|f| l.get(*f) == link.get(*f));
    if links.iter().any(same) {
        return Err(refuse(
            at,
            ErrorName::LinkExists,
            format!("the link {link} exists"),
        ));
    }
    links.push(link);
    Ok(())
}

/// The rules applied in order to a copy of `doc`, and the check of the result.
fn run(doc: &Value, patch: &Patch) -> Result<(Value, Vec<Change>, usize), Refused> {
    if patch.rules.is_empty() {
        return Err(refuse(None, ErrorName::EmptyPatch, "the patch has no rule"));
    }
    let rules = patch
        .rules
        .iter()
        .enumerate()
        .map(|(i, r)| compile(i + 1, r))
        .collect::<Result<Vec<_>, _>>()?;
    let mut next = doc.clone();
    for rule in &rules {
        match rule.verb {
            Verb::Set => run_set(rule, &mut next)?,
            Verb::Add => run_add(rule, &mut next)?,
        }
    }
    let changes = diff::diff(doc, &next);
    let rows = diff::rows(&changes);
    let refusals = check::check(&next);
    if !refusals.is_empty() {
        return Err(Refused {
            rule: None,
            error: ErrorName::CheckRefused,
            message: refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
            diff: changes,
            check: refusals,
        });
    }
    Ok((next, changes, rows))
}

/// Applies `patch` to `doc` as one change: every rule, then the check, or a refusal and nothing.
pub fn apply(doc: &Value, patch: &Patch) -> Result<Applied, Refused> {
    let (document, diff, changed_rows) = run(doc, patch)?;
    Ok(Applied {
        document,
        diff,
        changed_rows,
        check: Vec::new(),
    })
}

/// What `apply` would do, without the new document: the diff and the check. A result the check
/// refuses is refused with its diff and its check.
pub fn dry_run(doc: &Value, patch: &Patch) -> Result<DryRun, Refused> {
    let (_, diff, changed_rows) = run(doc, patch)?;
    Ok(DryRun {
        diff,
        changed_rows,
        check: Vec::new(),
    })
}
