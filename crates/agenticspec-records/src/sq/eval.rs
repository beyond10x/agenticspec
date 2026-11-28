//! The sq evaluator: rules compiled to slots, ordered so every filter sees bound variables,
//! stratified by negation and aggregation, and solved depth first per session.
//!
//! A row is definite or possible. A row is possible when a fact it rests on is possible (an
//! inferred author, a decision not recorded, an inferred spawn link, an open interval) or when a
//! negation holds only because the facts that would refute it are possible. Possible rows carry
//! the sites they rest on.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use regex::Regex;

use super::facts::{FactBase, SCHEMA, Table, Val};
use super::syntax::{AggOp, Atom, CmpOp, Expect, FieldPat, Lit, Module, Scope, Term};

const MAX_SITES: usize = 8;

#[derive(Debug, Clone)]
enum Ct {
    Var(usize),
    Const(Val),
    Wild,
}

#[derive(Debug, Clone)]
enum Pat {
    T(Ct),
    Re(Regex),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rel {
    Base(&'static str),
    Derived(usize),
}

#[derive(Debug, Clone)]
enum CLit {
    Atom {
        rel: Rel,
        pats: Vec<(usize, Pat)>,
    },
    Not(Box<CLit>),
    Cmp(Ct, CmpOp, Ct),
    Match(Ct, Regex),
    In(Ct, Arc<HashSet<String>>),
    Agg {
        out: Ct,
        op: AggOp,
        of: Option<Ct>,
        locals: Vec<usize>,
        body: Vec<CLit>,
    },
    Before(Ct, Ct),
    Seq {
        nodes: Vec<Ct>,
        scope: Scope,
        within: Option<i64>,
        gap: Option<i64>,
        nearest: bool,
    },
}

#[derive(Debug, Clone)]
struct Clause {
    head: Vec<Ct>,
    body: Vec<CLit>,
    slots: usize,
}

#[derive(Debug, Clone)]
pub struct Derived {
    pub name: String,
    pub arity: usize,
    clauses: Vec<Clause>,
}

#[derive(Debug, Clone)]
pub struct CompiledQuery {
    pub text: String,
    pub columns: Vec<String>,
    clause: Clause,
}

#[derive(Debug, Clone)]
pub struct CompiledCheck {
    pub name: String,
    pub about: String,
    pub severity: String,
    pub expect: Expect,
    find: usize,
}

/// A compiled module: derived relations in evaluation order, queries and checks.
#[derive(Debug, Clone)]
pub struct Program {
    pub derived: Vec<Derived>,
    /// Strata: each a list of derived relations evaluated together to a fixpoint.
    strata: Vec<Vec<usize>>,
    pub queries: Vec<CompiledQuery>,
    pub checks: Vec<CompiledCheck>,
}

/// The relations a fact base holds and their columns: the session relations ([`SCHEMA`]) or those
/// of an architecture (`super::architecture::SCHEMA`).
pub type Schema = &'static [(&'static str, &'static [&'static str])];

fn base_cols(schema: Schema, name: &str) -> Option<(&'static str, &'static [&'static str])> {
    schema
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(n, c)| (*n, *c))
}

struct Scoper<'a> {
    schema: Schema,
    slots: HashMap<String, usize>,
    derived: &'a HashMap<String, (usize, usize)>,
    names: &'a BTreeMap<String, Vec<String>>,
    /// Inside an aggregate a wildcard is a column of its own: `count{ r(_, _) }` counts rows.
    in_agg: bool,
}

impl Scoper<'_> {
    fn term(&mut self, t: &Term) -> Ct {
        match t {
            Term::Wild if self.in_agg => {
                let n = self.slots.len();
                Ct::Var(*self.slots.entry(format!("_#{n}")).or_insert(n))
            }
            Term::Wild => Ct::Wild,
            Term::Var(v) => {
                let n = self.slots.len();
                Ct::Var(*self.slots.entry(v.clone()).or_insert(n))
            }
            Term::Str(s) => Ct::Const(Val::Str(Arc::from(s.as_str()))),
            Term::Int(n) => Ct::Const(Val::Int(*n)),
            Term::Bool(b) => Ct::Const(Val::Bool(*b)),
        }
    }

    fn regex(re: &str) -> Result<Regex, String> {
        Regex::new(re).map_err(|e| format!("regex {re:?}: {e}"))
    }

    fn atom(&mut self, a: &Atom) -> Result<CLit, String> {
        if a.rel == "before" && !a.record {
            if a.positional.len() != 2 {
                return Err("before/2 takes two nodes".to_owned());
            }
            return Ok(CLit::Before(
                self.term(&a.positional[0]),
                self.term(&a.positional[1]),
            ));
        }
        if let Some((name, cols)) = base_cols(self.schema, &a.rel) {
            let mut pats = Vec::new();
            if a.record {
                for (f, p) in &a.named {
                    let Some(c) = cols.iter().position(|x| x == f) else {
                        return Err(format!(
                            "{}: no field `{f}`; fields are {}",
                            a.rel,
                            cols.join(", ")
                        ));
                    };
                    let pat = match p {
                        FieldPat::Term(t) => Pat::T(self.term(t)),
                        FieldPat::Regex(re) => Pat::Re(Self::regex(re)?),
                    };
                    pats.push((c, pat));
                }
            } else {
                if a.positional.len() != cols.len() {
                    return Err(format!(
                        "{}/{} is used with {} arguments; fields are {}",
                        a.rel,
                        cols.len(),
                        a.positional.len(),
                        cols.join(", ")
                    ));
                }
                for (c, t) in a.positional.iter().enumerate() {
                    pats.push((c, Pat::T(self.term(t))));
                }
            }
            return Ok(CLit::Atom {
                rel: Rel::Base(name),
                pats,
            });
        }
        if let Some(&(ix, arity)) = self.derived.get(&a.rel) {
            if a.record {
                return Err(format!(
                    "{}: a rule is matched by position, not by field",
                    a.rel
                ));
            }
            if a.positional.len() != arity {
                return Err(format!(
                    "{}/{arity} is used with {} arguments",
                    a.rel,
                    a.positional.len()
                ));
            }
            let pats = a
                .positional
                .iter()
                .enumerate()
                .map(|(c, t)| (c, Pat::T(self.term(t))))
                .collect();
            return Ok(CLit::Atom {
                rel: Rel::Derived(ix),
                pats,
            });
        }
        Err(format!("unknown relation `{}`", a.rel))
    }

    fn lit(&mut self, l: &Lit) -> Result<CLit, String> {
        Ok(match l {
            Lit::Atom(a) => self.atom(a)?,
            Lit::Not(inner) => CLit::Not(Box::new(self.lit(inner)?)),
            Lit::Cmp(a, op, b) => CLit::Cmp(self.term(a), *op, self.term(b)),
            Lit::Match(t, re) => CLit::Match(self.term(t), Self::regex(re)?),
            Lit::In(t, list) => {
                let Some(items) = self.names.get(list) else {
                    return Err(format!("no names list `{list}`"));
                };
                CLit::In(self.term(t), Arc::new(items.iter().cloned().collect()))
            }
            Lit::Agg { out, op, of, body } => {
                let out = self.term(out);
                let of = of.as_ref().map(|t| self.term(t));
                let was = std::mem::replace(&mut self.in_agg, true);
                let body = body
                    .iter()
                    .map(|l| self.lit(l))
                    .collect::<Result<Vec<_>, _>>()?;
                self.in_agg = was;
                CLit::Agg {
                    out,
                    op: *op,
                    of,
                    locals: Vec::new(),
                    body,
                }
            }
            Lit::Seq {
                nodes,
                scope,
                within,
                gap,
                nearest,
            } => CLit::Seq {
                nodes: nodes.iter().map(|t| self.term(t)).collect(),
                scope: *scope,
                within: *within,
                gap: *gap,
                nearest: *nearest,
            },
        })
    }
}

fn vars_of_ct(t: &Ct, out: &mut Vec<usize>) {
    if let Ct::Var(v) = t {
        out.push(*v);
    }
}

fn vars_of(l: &CLit, out: &mut Vec<usize>) {
    match l {
        CLit::Atom { pats, .. } => {
            for (_, p) in pats {
                if let Pat::T(t) = p {
                    vars_of_ct(t, out);
                }
            }
        }
        CLit::Not(i) => vars_of(i, out),
        CLit::Cmp(a, _, b) | CLit::Before(a, b) => {
            vars_of_ct(a, out);
            vars_of_ct(b, out);
        }
        CLit::Match(t, _) | CLit::In(t, _) => vars_of_ct(t, out),
        CLit::Agg {
            out: o, of, body, ..
        } => {
            vars_of_ct(o, out);
            if let Some(t) = of {
                vars_of_ct(t, out);
            }
            for b in body {
                vars_of(b, out);
            }
        }
        CLit::Seq { nodes, .. } => {
            for n in nodes {
                vars_of_ct(n, out);
            }
        }
    }
}

/// Orders a body so each literal's inputs are bound before it runs. `outside` are the variables
/// used outside the body (the head, other literals of an enclosing body).
fn plan(
    body: Vec<CLit>,
    bound: &HashSet<usize>,
    outside: &HashSet<usize>,
) -> Result<Vec<CLit>, String> {
    let mut bound = bound.clone();
    let mut left: Vec<Option<CLit>> = body.into_iter().map(Some).collect();
    let mut out = Vec::new();
    // Occurrence counts: a variable inside a negation or an aggregate is local to it when it
    // occurs nowhere else.
    let occurs = |left: &[Option<CLit>], skip: usize| -> HashSet<usize> {
        let mut s: HashSet<usize> = outside.clone();
        for (i, l) in left.iter().enumerate() {
            if i == skip {
                continue;
            }
            if let Some(l) = l {
                let mut v = Vec::new();
                vars_of(l, &mut v);
                s.extend(v);
            }
        }
        s
    };
    // Variables already placed count as outside too.
    let mut placed_vars: HashSet<usize> = HashSet::new();
    while left.iter().any(Option::is_some) {
        // Among the literals that can run, filters first, then a binding comparison or an
        // aggregate, then `seq` (it generates few candidates near a bound node), then an atom with a
        // bound or constant column (an index lookup), then any atom; ties keep the written order.
        let mut pick: Option<(u8, usize)> = None;
        for (i, l) in left.iter().enumerate() {
            let Some(l) = l else { continue };
            let mut shared = occurs(&left, i);
            shared.extend(placed_vars.iter().copied());
            let is_bound = |t: &Ct| match t {
                Ct::Var(v) => bound.contains(v),
                _ => true,
            };
            let ready = match l {
                CLit::Atom { .. } => true,
                CLit::Not(inner) => {
                    let mut v = Vec::new();
                    vars_of(inner, &mut v);
                    v.iter().all(|x| !shared.contains(x) || bound.contains(x))
                }
                CLit::Cmp(a, CmpOp::Eq, b) => is_bound(a) || is_bound(b),
                CLit::Cmp(a, _, b) | CLit::Before(a, b) => is_bound(a) && is_bound(b),
                CLit::Match(t, _) | CLit::In(t, _) => is_bound(t),
                CLit::Agg { body, .. } => {
                    let mut v = Vec::new();
                    for b in body {
                        vars_of(b, &mut v);
                    }
                    v.iter().all(|x| !shared.contains(x) || bound.contains(x))
                }
                CLit::Seq { nodes, .. } => {
                    nodes.iter().any(|n| !matches!(n, Ct::Wild) && is_bound(n))
                }
            };
            if !ready {
                continue;
            }
            let prio = match l {
                CLit::Not(_) | CLit::Match(..) | CLit::In(..) | CLit::Before(..) => 0,
                CLit::Cmp(a, _, b) if is_bound(a) && is_bound(b) => 0,
                CLit::Cmp(..) | CLit::Agg { .. } => 1,
                CLit::Seq { .. } => 2,
                // Every named column bound: a lookup that only keeps or drops rows, so it runs
                // before a `seq` multiplies them (2026-11-26: `at(P, T)` after `seq(P, B)` made
                // the idle-bust check time out on sessions over 37 MB).
                CLit::Atom { pats, .. }
                    if !pats.is_empty()
                        && pats.iter().all(|(_, p)| match p {
                            Pat::T(t) => is_bound(t),
                            Pat::Re(_) => true,
                        }) =>
                {
                    0
                }
                CLit::Atom { pats, .. } => {
                    if pats.iter().any(
                        |(_, p)| matches!(p, Pat::T(t) if !matches!(t, Ct::Wild) && is_bound(t)),
                    ) {
                        3
                    } else {
                        4
                    }
                }
            };
            if pick.is_none_or(|(p, _)| prio < p) {
                pick = Some((prio, i));
            }
        }
        let Some((_, i)) = pick else {
            return Err(
                "a literal can never run: a comparison, negation, `~`, `in`, `before` or `seq` \
                 uses a variable no positive atom binds"
                    .to_owned(),
            );
        };
        let shared = {
            let mut s = occurs(&left, i);
            s.extend(placed_vars.iter().copied());
            s
        };
        let mut l = left[i].take().expect("picked a present literal");
        if let CLit::Agg {
            of, locals, body, ..
        } = &mut l
        {
            let mut v = Vec::new();
            for b in body.iter() {
                vars_of(b, &mut v);
            }
            let mut inner_outside: HashSet<usize> = shared.clone();
            if let Some(t) = of {
                let mut w = Vec::new();
                vars_of_ct(t, &mut w);
                inner_outside.extend(w);
            }
            let mut loc: Vec<usize> = v.iter().copied().filter(|x| !bound.contains(x)).collect();
            loc.sort_unstable();
            loc.dedup();
            *locals = loc;
            let b = std::mem::take(body);
            *body = plan(b, &bound, &inner_outside)?;
        }
        let mut v = Vec::new();
        vars_of(&l, &mut v);
        match &l {
            CLit::Not(_) | CLit::Match(..) | CLit::In(..) | CLit::Before(..) => {}
            CLit::Cmp(_, op, _) if *op != CmpOp::Eq => {}
            CLit::Agg { out: o, .. } => {
                let mut w = Vec::new();
                vars_of_ct(o, &mut w);
                bound.extend(w);
            }
            _ => bound.extend(v.iter().copied()),
        }
        placed_vars.extend(v);
        out.push(l);
    }
    Ok(out)
}

/// Compiles a module over the session relations.
pub fn compile(m: &Module) -> Result<Program, String> {
    compile_in(m, SCHEMA)
}

/// Compiles a module over the relations of `schema`; an atom of any other relation is refused.
pub fn compile_in(m: &Module, schema: Schema) -> Result<Program, String> {
    // Derived relations: one per rule head.
    let mut derived_ix: HashMap<String, (usize, usize)> = HashMap::new();
    let mut derived: Vec<Derived> = Vec::new();
    for r in &m.rules {
        if base_cols(schema, &r.head).is_some() || r.head == "before" || r.head == "seq" {
            return Err(format!(
                "`{}` is a relation of the fact base; a rule cannot define it",
                r.head
            ));
        }
        match derived_ix.get(&r.head) {
            Some(&(_, arity)) if arity != r.args.len() => {
                return Err(format!(
                    "`{}` is defined with {arity} and with {} arguments",
                    r.head,
                    r.args.len()
                ));
            }
            Some(_) => {}
            None => {
                derived_ix.insert(r.head.clone(), (derived.len(), r.args.len()));
                derived.push(Derived {
                    name: r.head.clone(),
                    arity: r.args.len(),
                    clauses: Vec::new(),
                });
            }
        }
    }
    let clause = |head: &[Term], body: &[Lit]| -> Result<Clause, String> {
        let mut sc = Scoper {
            schema,
            slots: HashMap::new(),
            derived: &derived_ix,
            names: &m.names,
            in_agg: false,
        };
        let head_c: Vec<Ct> = head.iter().map(|t| sc.term(t)).collect();
        let body_c = body
            .iter()
            .map(|l| sc.lit(l))
            .collect::<Result<Vec<_>, _>>()?;
        let mut outside = HashSet::new();
        for h in &head_c {
            if let Ct::Var(v) = h {
                outside.insert(*v);
            }
        }
        let planned = plan(body_c, &HashSet::new(), &outside)?;
        // Every head variable is bound by the body.
        let mut bound = Vec::new();
        for l in &planned {
            match l {
                CLit::Atom { .. } | CLit::Seq { .. } | CLit::Cmp(_, CmpOp::Eq, _) => {
                    vars_of(l, &mut bound)
                }
                CLit::Agg { out, .. } => vars_of_ct(out, &mut bound),
                _ => {}
            }
        }
        for h in &head_c {
            if let Ct::Var(v) = h
                && !bound.contains(v)
            {
                let name = sc
                    .slots
                    .iter()
                    .find(|(_, s)| *s == v)
                    .map(|(n, _)| n.clone())
                    .unwrap_or_default();
                return Err(format!(
                    "`{name}` in the head is bound by no atom of the body"
                ));
            }
        }
        Ok(Clause {
            head: head_c,
            body: planned,
            slots: sc.slots.len(),
        })
    };
    for r in &m.rules {
        let c = clause(&r.args, &r.body).map_err(|e| format!("rule `{}`: {e}", r.head))?;
        let (ix, _) = derived_ix[&r.head];
        derived[ix].clauses.push(c);
    }

    // Strata: strongly connected components of the dependency graph, dependencies first. A
    // negation or an aggregate inside a component is refused.
    let n = derived.len();
    let mut edges: Vec<Vec<(usize, bool)>> = vec![Vec::new(); n];
    fn deps(l: &CLit, neg: bool, out: &mut Vec<(usize, bool)>) {
        match l {
            CLit::Atom {
                rel: Rel::Derived(d),
                ..
            } => out.push((*d, neg)),
            CLit::Not(i) => deps(i, true, out),
            CLit::Agg { body, .. } => {
                for b in body {
                    deps(b, true, out);
                }
            }
            _ => {}
        }
    }
    for (i, d) in derived.iter().enumerate() {
        for c in &d.clauses {
            for l in &c.body {
                deps(l, false, &mut edges[i]);
            }
        }
    }
    let strata = tarjan(&edges);
    for comp in &strata {
        let set: HashSet<usize> = comp.iter().copied().collect();
        for &i in comp {
            for &(j, neg) in &edges[i] {
                if neg && set.contains(&j) {
                    return Err(format!(
                        "`{}` depends on itself through a negation or an aggregate",
                        derived[i].name
                    ));
                }
            }
        }
    }

    let mut queries = Vec::new();
    for q in &m.queries {
        let mut names: Vec<String> = Vec::new();
        collect_var_names(&q.body, &mut names);
        let head: Vec<Term> = names.iter().map(|n| Term::Var(n.clone())).collect();
        let c = clause(&head, &q.body).map_err(|e| format!("query `{}`: {e}", q.text))?;
        queries.push(CompiledQuery {
            text: q.text.clone(),
            columns: names,
            clause: c,
        });
    }
    let mut checks = Vec::new();
    for c in &m.checks {
        let Some(&(find, _)) = derived_ix.get(&c.find) else {
            return Err(format!("check `{}`: no rule `{}`", c.name, c.find));
        };
        checks.push(CompiledCheck {
            name: c.name.clone(),
            about: c.about.clone(),
            severity: c.severity.clone(),
            expect: c.expect,
            find,
        });
    }
    Ok(Program {
        derived,
        strata,
        queries,
        checks,
    })
}

/// Variables of a query in order of first occurrence; `_`-prefixed and those only inside a
/// negation or an aggregate are not columns.
fn collect_var_names(body: &[Lit], out: &mut Vec<String>) {
    fn term(t: &Term, out: &mut Vec<String>) {
        if let Term::Var(v) = t
            && !v.starts_with('_')
            && !out.contains(v)
        {
            out.push(v.clone());
        }
    }
    for l in body {
        match l {
            Lit::Atom(a) => {
                for t in &a.positional {
                    term(t, out);
                }
                for (_, p) in &a.named {
                    if let FieldPat::Term(t) = p {
                        term(t, out);
                    }
                }
            }
            Lit::Cmp(a, CmpOp::Eq, b) => {
                term(a, out);
                term(b, out);
            }
            Lit::Agg { out: o, .. } => term(o, out),
            Lit::Seq { nodes, .. } => {
                for n in nodes {
                    term(n, out);
                }
            }
            _ => {}
        }
    }
}

fn tarjan(edges: &[Vec<(usize, bool)>]) -> Vec<Vec<usize>> {
    struct T<'a> {
        edges: &'a [Vec<(usize, bool)>],
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        on: Vec<bool>,
        stack: Vec<usize>,
        next: usize,
        out: Vec<Vec<usize>>,
    }
    fn visit(t: &mut T<'_>, v: usize) {
        t.index[v] = Some(t.next);
        t.low[v] = t.next;
        t.next += 1;
        t.stack.push(v);
        t.on[v] = true;
        for &(w, _) in &t.edges[v] {
            match t.index[w] {
                None => {
                    visit(t, w);
                    t.low[v] = t.low[v].min(t.low[w]);
                }
                Some(iw) if t.on[w] => t.low[v] = t.low[v].min(iw),
                _ => {}
            }
        }
        if Some(t.low[v]) == t.index[v] {
            let mut comp = Vec::new();
            while let Some(w) = t.stack.pop() {
                t.on[w] = false;
                comp.push(w);
                if w == v {
                    break;
                }
            }
            t.out.push(comp);
        }
    }
    let n = edges.len();
    let mut t = T {
        edges,
        index: vec![None; n],
        low: vec![0; n],
        on: vec![false; n],
        stack: Vec::new(),
        next: 0,
        out: Vec::new(),
    };
    for v in 0..n {
        if t.index[v].is_none() {
            visit(&mut t, v);
        }
    }
    t.out
}

/// The rows of one relation: values and the sites a possible row rests on (empty: definite).
#[derive(Debug, Default, Clone)]
pub struct Rows {
    pub rows: Vec<(Vec<Val>, Vec<Arc<str>>)>,
    seen: HashMap<Vec<Val>, usize>,
}

impl Rows {
    /// Adds a row; a definite row replaces a possible one. True when the set changed.
    fn add(&mut self, vals: Vec<Val>, sites: Vec<Arc<str>>) -> bool {
        match self.seen.get(&vals) {
            Some(&i) => {
                if !self.rows[i].1.is_empty() && sites.is_empty() {
                    self.rows[i].1.clear();
                    true
                } else {
                    false
                }
            }
            None => {
                self.seen.insert(vals.clone(), self.rows.len());
                self.rows.push((vals, sites));
                true
            }
        }
    }

    fn lookup(
        &self,
        col: usize,
        v: &Val,
        index: &mut HashMap<usize, HashMap<Val, Vec<usize>>>,
    ) -> Vec<usize> {
        let by = index.entry(col).or_insert_with(|| {
            let mut m: HashMap<Val, Vec<usize>> = HashMap::new();
            for (i, r) in self.rows.iter().enumerate() {
                m.entry(r.0[col].clone()).or_default().push(i);
            }
            m
        });
        by.get(v).cloned().unwrap_or_default()
    }
}

/// The answers of a program over one session.
pub struct Answers {
    pub derived: Vec<Rows>,
    pub queries: Vec<Rows>,
}

struct Ctx<'a> {
    fb: &'a FactBase,
    derived: &'a [Rows],
    index: std::cell::RefCell<Vec<ColIndex>>,
}

type Env = Vec<Option<Val>>;
type ColIndex = HashMap<usize, HashMap<Val, Vec<usize>>>;

fn resolve(t: &Ct, env: &Env) -> Option<Val> {
    match t {
        Ct::Var(v) => env[*v].clone(),
        Ct::Const(c) => Some(c.clone()),
        Ct::Wild => None,
    }
}

fn cmp(a: &Val, op: CmpOp, b: &Val) -> bool {
    use std::cmp::Ordering::*;
    let ord = match (a, b) {
        (Val::Int(x), Val::Int(y)) => x.cmp(y),
        (Val::Str(x), Val::Str(y)) => x.cmp(y),
        (Val::Bool(x), Val::Bool(y)) => x.cmp(y),
        (Val::Null, Val::Null) => Equal,
        _ => return op == CmpOp::Ne,
    };
    match op {
        CmpOp::Eq => ord == Equal,
        CmpOp::Ne => ord != Equal,
        CmpOp::Lt => ord == Less,
        CmpOp::Le => ord != Greater,
        CmpOp::Gt => ord == Greater,
        CmpOp::Ge => ord != Less,
    }
}

fn add_sites(sites: &mut Vec<Arc<str>>, more: &[Arc<str>]) {
    for s in more {
        if sites.len() >= MAX_SITES {
            break;
        }
        if !sites.contains(s) {
            sites.push(s.clone());
        }
    }
}

/// What a solution callback says: go on looking, or stop.
type Sink<'s> = dyn FnMut(&Env, &[Arc<str>]) -> bool + 's;

impl Ctx<'_> {
    /// Solves `lits[i..]`. Returns (found a solution, stopped).
    fn solve(
        &self,
        lits: &[CLit],
        i: usize,
        env: &mut Env,
        sites: &mut Vec<Arc<str>>,
        sink: &mut Sink<'_>,
    ) -> (bool, bool) {
        let Some(lit) = lits.get(i) else {
            let go_on = sink(env, sites);
            return (true, !go_on);
        };
        match lit {
            CLit::Atom { rel, pats } => self.atom(*rel, pats, lits, i, env, sites, sink),
            CLit::Not(inner) => {
                let mut definite = false;
                let mut possible: Vec<Arc<str>> = Vec::new();
                let mut scratch = env.clone();
                let mut s0 = Vec::new();
                self.solve(
                    std::slice::from_ref(inner.as_ref()),
                    0,
                    &mut scratch,
                    &mut s0,
                    &mut |_, st| {
                        if st.is_empty() {
                            definite = true;
                            false
                        } else {
                            add_sites(&mut possible, st);
                            true
                        }
                    },
                );
                if definite {
                    return (false, false);
                }
                let len = sites.len();
                add_sites(sites, &possible);
                let r = self.solve(lits, i + 1, env, sites, sink);
                sites.truncate(len);
                r
            }
            CLit::Cmp(a, op, b) => {
                let (va, vb) = (resolve(a, env), resolve(b, env));
                match (va, vb) {
                    (Some(x), Some(y)) => {
                        if cmp(&x, *op, &y) {
                            self.solve(lits, i + 1, env, sites, sink)
                        } else {
                            (false, false)
                        }
                    }
                    (Some(x), None) | (None, Some(x)) if *op == CmpOp::Eq => {
                        let slot = match (a, b) {
                            (Ct::Var(v), _) if env[*v].is_none() => *v,
                            (_, Ct::Var(v)) => *v,
                            _ => return (false, false),
                        };
                        env[slot] = Some(x);
                        let r = self.solve(lits, i + 1, env, sites, sink);
                        env[slot] = None;
                        r
                    }
                    _ => (false, false),
                }
            }
            CLit::Match(t, re) => match resolve(t, env) {
                Some(Val::Str(s)) if re.is_match(&s) => self.solve(lits, i + 1, env, sites, sink),
                _ => (false, false),
            },
            CLit::In(t, set) => match resolve(t, env) {
                Some(v) if set.contains(&v.to_string()) => {
                    self.solve(lits, i + 1, env, sites, sink)
                }
                _ => (false, false),
            },
            CLit::Before(a, b) => {
                let (Some(Val::Str(x)), Some(Val::Str(y))) = (resolve(a, env), resolve(b, env))
                else {
                    return (false, false);
                };
                let (Some(nx), Some(ny)) = (self.fb.nodes.get(&x), self.fb.nodes.get(&y)) else {
                    return (false, false);
                };
                let len = sites.len();
                let holds = if nx.agent == ny.agent {
                    nx.agent_pos < ny.agent_pos
                } else if nx.at == ny.at {
                    add_sites(sites, &[Arc::from(format!("{x} (same instant as {y})"))]);
                    true
                } else {
                    nx.at < ny.at
                };
                let r = if holds {
                    self.solve(lits, i + 1, env, sites, sink)
                } else {
                    (false, false)
                };
                sites.truncate(len);
                r
            }
            CLit::Agg {
                out,
                op,
                of,
                locals,
                body,
            } => {
                let mut seen: HashSet<Vec<Option<Val>>> = HashSet::new();
                let mut acc: Option<i64> = None;
                let mut possible: Vec<Arc<str>> = Vec::new();
                let mut scratch = env.clone();
                let mut s0 = Vec::new();
                self.solve(body, 0, &mut scratch, &mut s0, &mut |e, st| {
                    let key: Vec<Option<Val>> = locals.iter().map(|l| e[*l].clone()).collect();
                    if !seen.insert(key) {
                        return true;
                    }
                    add_sites(&mut possible, st);
                    let x = match of.as_ref().and_then(|t| resolve(t, e)) {
                        Some(Val::Int(n)) => n,
                        _ => 1,
                    };
                    acc = Some(match (*op, acc) {
                        (AggOp::Count, a) => a.unwrap_or(0) + 1,
                        (AggOp::Sum, a) => a.unwrap_or(0) + x,
                        (AggOp::Min, a) => a.map_or(x, |a| a.min(x)),
                        (AggOp::Max, a) => a.map_or(x, |a| a.max(x)),
                    });
                    true
                });
                let value = match (*op, acc) {
                    (AggOp::Count | AggOp::Sum, a) => Val::Int(a.unwrap_or(0)),
                    (_, Some(a)) => Val::Int(a),
                    (_, None) => return (false, false),
                };
                let len = sites.len();
                add_sites(sites, &possible);
                let r = match out {
                    Ct::Var(v) if env[*v].is_none() => {
                        env[*v] = Some(value);
                        let r = self.solve(lits, i + 1, env, sites, sink);
                        env[*v] = None;
                        r
                    }
                    other => match resolve(other, env) {
                        Some(x) if x == value => self.solve(lits, i + 1, env, sites, sink),
                        _ => (false, false),
                    },
                };
                sites.truncate(len);
                r
            }
            CLit::Seq {
                nodes,
                scope,
                within,
                gap,
                nearest,
            } => self.seq(
                nodes, *scope, *within, *gap, *nearest, lits, i, env, sites, sink,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn atom(
        &self,
        rel: Rel,
        pats: &[(usize, Pat)],
        lits: &[CLit],
        i: usize,
        env: &mut Env,
        sites: &mut Vec<Arc<str>>,
        sink: &mut Sink<'_>,
    ) -> (bool, bool) {
        // The first bound column picks the rows through an index.
        let key = pats.iter().find_map(|(c, p)| match p {
            Pat::T(t) => resolve(t, env).map(|v| (*c, v)),
            Pat::Re(_) => None,
        });
        let (table, rows): (Option<&Table>, Option<&Rows>) = match rel {
            Rel::Base(name) => (self.fb.tables.get(name), None),
            Rel::Derived(d) => (None, Some(&self.derived[d])),
        };
        let candidates: Vec<usize> = match (&key, table, rows) {
            (Some((c, v)), Some(t), _) => t.lookup(*c, v),
            (Some((c, v)), None, Some(r)) => {
                let mut idx = self.index.borrow_mut();
                let d = match rel {
                    Rel::Derived(d) => d,
                    Rel::Base(_) => 0,
                };
                r.lookup(*c, v, &mut idx[d])
            }
            (None, Some(t), _) => (0..t.rows.len()).collect(),
            (None, None, Some(r)) => (0..r.rows.len()).collect(),
            _ => Vec::new(),
        };
        let mut found = false;
        for ix in candidates {
            let (vals, row_sites): (&[Val], Vec<Arc<str>>) = match (table, rows) {
                (Some(t), _) => {
                    let r = &t.rows[ix];
                    let mut st = Vec::new();
                    for u in &r.unk {
                        let used = match u.col {
                            None => true,
                            Some(c) => pats
                                .iter()
                                .any(|(pc, p)| *pc == c && !matches!(p, Pat::T(Ct::Wild))),
                        };
                        if used {
                            st.push(u.site.clone());
                        }
                    }
                    (&r.vals, st)
                }
                (None, Some(r)) => (&r.rows[ix].0, r.rows[ix].1.clone()),
                _ => continue,
            };
            let mut set: Vec<usize> = Vec::new();
            let mut ok = true;
            for (c, p) in pats {
                let v = &vals[*c];
                match p {
                    Pat::T(Ct::Wild) => {}
                    Pat::T(Ct::Const(k)) => {
                        if k != v {
                            ok = false;
                        }
                    }
                    Pat::T(Ct::Var(s)) => match &env[*s] {
                        Some(b) => {
                            if b != v {
                                ok = false;
                            }
                        }
                        None => {
                            env[*s] = Some(v.clone());
                            set.push(*s);
                        }
                    },
                    Pat::Re(re) => match v {
                        Val::Str(s) if re.is_match(s) => {}
                        _ => ok = false,
                    },
                }
                if !ok {
                    break;
                }
            }
            if ok {
                let len = sites.len();
                add_sites(sites, &row_sites);
                let (f, stop) = self.solve(lits, i + 1, env, sites, sink);
                sites.truncate(len);
                found |= f;
                if stop {
                    for s in set {
                        env[s] = None;
                    }
                    return (found, true);
                }
            }
            for s in set {
                env[s] = None;
            }
        }
        (found, false)
    }

    #[allow(clippy::too_many_arguments)]
    fn seq(
        &self,
        nodes: &[Ct],
        scope: Scope,
        within: Option<i64>,
        gap: Option<i64>,
        nearest: bool,
        lits: &[CLit],
        i: usize,
        env: &mut Env,
        sites: &mut Vec<Arc<str>>,
        sink: &mut Sink<'_>,
    ) -> (bool, bool) {
        // The anchor: the first node already bound.
        let Some((anchor_k, anchor)) =
            nodes
                .iter()
                .enumerate()
                .find_map(|(k, n)| match resolve(n, env) {
                    Some(Val::Str(s)) => Some((k, s)),
                    _ => None,
                })
        else {
            return (false, false);
        };
        let Some(info) = self.fb.nodes.get(&anchor) else {
            return (false, false);
        };
        let order: &[Arc<str>] = match scope {
            Scope::Agent => self
                .fb
                .agent_order
                .get(&info.agent)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            Scope::Session => &self.fb.session_order,
        };
        let pos_of = |n: &str| -> Option<usize> {
            let ni = self.fb.nodes.get(n)?;
            match scope {
                Scope::Agent if ni.agent == info.agent => Some(ni.agent_pos),
                Scope::Agent => None,
                Scope::Session => Some(ni.session_pos),
            }
        };
        let at_of = |p: usize| self.fb.nodes.get(&order[p]).map_or(0, |n| n.at);
        let max_step = gap.map_or(usize::MAX, |g| g.max(0) as usize + 1);
        let a_pos = match pos_of(&anchor) {
            Some(p) => p,
            None => return (false, false),
        };
        // Nodes before the anchor are placed backwards from it, nodes after it forwards; each
        // choice is the nearest first.
        let mut found = 0usize;
        let mut chosen: Vec<Option<usize>> = vec![None; nodes.len()];
        chosen[anchor_k] = Some(a_pos);
        #[allow(clippy::too_many_arguments)]
        fn place(
            me: &Ctx<'_>,
            k: usize,
            dir_back: bool,
            nodes: &[Ct],
            chosen: &mut Vec<Option<usize>>,
            anchor_k: usize,
            order: &[Arc<str>],
            pos_of: &dyn Fn(&str) -> Option<usize>,
            at_of: &dyn Fn(usize) -> i64,
            max_step: usize,
            within: Option<i64>,
            nearest: bool,
            lits: &[CLit],
            i: usize,
            env: &mut Env,
            sites: &mut Vec<Arc<str>>,
            sink: &mut Sink<'_>,
            found: &mut usize,
        ) -> bool {
            // Returns true to stop.
            let n = nodes.len();
            if dir_back && k == usize::MAX {
                return place(
                    me,
                    anchor_k + 1,
                    false,
                    nodes,
                    chosen,
                    anchor_k,
                    order,
                    pos_of,
                    at_of,
                    max_step,
                    within,
                    nearest,
                    lits,
                    i,
                    env,
                    sites,
                    sink,
                    found,
                );
            }
            if !dir_back && k >= n {
                let first = chosen[0].expect("placed");
                let last = chosen[n - 1].expect("placed");
                if let Some(w) = within
                    && at_of(last) - at_of(first) > w
                {
                    return false;
                }
                let (f, stop) = me.solve(lits, i + 1, env, sites, sink);
                if f {
                    *found += 1;
                }
                return stop;
            }
            let neighbour = if dir_back {
                chosen[k + 1]
            } else {
                chosen[k - 1]
            }
            .expect("placed");
            let candidates: Vec<usize> = match resolve(&nodes[k], env) {
                Some(Val::Str(s)) => pos_of(&s).into_iter().collect(),
                _ if dir_back => (neighbour.saturating_sub(max_step)..neighbour)
                    .rev()
                    .collect(),
                _ => (neighbour + 1
                    ..order
                        .len()
                        .min(neighbour.saturating_add(max_step).saturating_add(1)))
                    .collect(),
            };
            for p in candidates {
                let ordered = if dir_back {
                    p < neighbour
                } else {
                    p > neighbour
                };
                if !ordered || p.abs_diff(neighbour) > max_step {
                    continue;
                }
                let bind = match &nodes[k] {
                    Ct::Var(v) if env[*v].is_none() => Some(*v),
                    _ => None,
                };
                if let Some(v) = bind {
                    env[v] = Some(Val::Str(order[p].clone()));
                }
                chosen[k] = Some(p);
                let before = *found;
                let next_k = if dir_back {
                    if k == 0 { usize::MAX } else { k - 1 }
                } else {
                    k + 1
                };
                let stop = place(
                    me, next_k, dir_back, nodes, chosen, anchor_k, order, pos_of, at_of, max_step,
                    within, nearest, lits, i, env, sites, sink, found,
                );
                chosen[k] = None;
                if let Some(v) = bind {
                    env[v] = None;
                }
                if stop {
                    return true;
                }
                if nearest && *found > before {
                    return false;
                }
            }
            false
        }
        let stop = if anchor_k == 0 {
            place(
                self,
                1,
                false,
                nodes,
                &mut chosen,
                anchor_k,
                order,
                &pos_of,
                &at_of,
                max_step,
                within,
                nearest,
                lits,
                i,
                env,
                sites,
                sink,
                &mut found,
            )
        } else {
            place(
                self,
                anchor_k - 1,
                true,
                nodes,
                &mut chosen,
                anchor_k,
                order,
                &pos_of,
                &at_of,
                max_step,
                within,
                nearest,
                lits,
                i,
                env,
                sites,
                sink,
                &mut found,
            )
        };
        (found > 0, stop)
    }
}

fn head_vals(head: &[Ct], env: &Env) -> Vec<Val> {
    head.iter()
        .map(|t| resolve(t, env).unwrap_or(Val::Null))
        .collect()
}

pub fn run(p: &Program, fb: &FactBase) -> Answers {
    let mut derived: Vec<Rows> = vec![Rows::default(); p.derived.len()];
    for comp in &p.strata {
        loop {
            let mut changed = false;
            for &d in comp {
                let mut new: Vec<(Vec<Val>, Vec<Arc<str>>)> = Vec::new();
                {
                    let ctx = Ctx {
                        fb,
                        derived: &derived,
                        index: std::cell::RefCell::new(vec![HashMap::new(); p.derived.len()]),
                    };
                    for c in &p.derived[d].clauses {
                        let mut env: Env = vec![None; c.slots];
                        let mut sites = Vec::new();
                        ctx.solve(&c.body, 0, &mut env, &mut sites, &mut |e, st| {
                            new.push((head_vals(&c.head, e), st.to_vec()));
                            true
                        });
                    }
                }
                for (v, s) in new {
                    changed |= derived[d].add(v, s);
                }
            }
            let recursive = comp.len() > 1
                || comp.iter().any(|&d| {
                    p.derived[d].clauses.iter().any(|c| {
                        c.body.iter().any(
                            |l| matches!(l, CLit::Atom { rel: Rel::Derived(x), .. } if *x == d),
                        )
                    })
                });
            if !changed || !recursive {
                break;
            }
        }
    }
    let mut queries = Vec::new();
    {
        let ctx = Ctx {
            fb,
            derived: &derived,
            index: std::cell::RefCell::new(vec![HashMap::new(); p.derived.len()]),
        };
        for q in &p.queries {
            let mut rows = Rows::default();
            let mut env: Env = vec![None; q.clause.slots];
            let mut sites = Vec::new();
            ctx.solve(&q.clause.body, 0, &mut env, &mut sites, &mut |e, st| {
                rows.add(head_vals(&q.clause.head, e), st.to_vec());
                true
            });
            queries.push(rows);
        }
    }
    Answers { derived, queries }
}

impl Answers {
    pub fn check_rows(&self, ch: &CompiledCheck) -> &Rows {
        &self.derived[ch.find]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fully bound atom runs before a `seq` that would multiply rows (the idle-bust rule).
    #[test]
    fn a_bound_lookup_runs_before_seq() {
        let m = crate::sq::syntax::parse(
            "r(P, B) :- interval{agent: A, to: T}, prompt{node: P, agent: A}, at(P, T), seq(P, B, scope: agent).",
        )
        .unwrap();
        let p = compile(&m).unwrap();
        let body = &p.derived[0].clauses[0].body;
        let at = body
            .iter()
            .position(|l| {
                matches!(
                    l,
                    CLit::Atom {
                        rel: Rel::Base("at"),
                        ..
                    }
                )
            })
            .unwrap();
        let seq = body
            .iter()
            .position(|l| matches!(l, CLit::Seq { .. }))
            .unwrap();
        assert!(at < seq, "{body:?}");
    }
}
