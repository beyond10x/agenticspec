//! The text of an sq module: lexer and recursive-descent parser.
//!
//! ```text
//! names shim = ["cargo", "go"].
//! head(X, Y) :- rel{field: X, other: ~"re"}, !p(X), N = count{ q(X, _) }, N > 2.
//! ?- head(X, Y), seq(X, Y, scope: agent, within: 10m, gap: 3).
//! check name { about "…" severity low expect none find head }
//! ```

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(String),
    Wild,
    Str(String),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldPat {
    Term(Term),
    Regex(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub rel: String,
    /// `rel(a, b)`: matched by column position.
    pub positional: Vec<Term>,
    /// `rel{field: a}`: matched by column name; absent columns are free.
    pub named: Vec<(String, FieldPat)>,
    pub record: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggOp {
    Count,
    Sum,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Agent,
    Session,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Atom(Atom),
    Not(Box<Lit>),
    Cmp(Term, CmpOp, Term),
    Match(Term, String),
    In(Term, String),
    Agg {
        out: Term,
        op: AggOp,
        of: Option<Term>,
        body: Vec<Lit>,
    },
    Seq {
        nodes: Vec<Term>,
        scope: Scope,
        within: Option<i64>,
        gap: Option<i64>,
        nearest: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub head: String,
    pub args: Vec<Term>,
    pub body: Vec<Lit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub body: Vec<Lit>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    None,
    Some,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub name: String,
    pub about: String,
    pub severity: String,
    pub expect: Expect,
    pub find: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Module {
    pub rules: Vec<Rule>,
    pub queries: Vec<Query>,
    pub checks: Vec<Check>,
    pub names: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Var(String),
    Str(String),
    Int(i64),
    Punct(&'static str),
}

struct Lexed {
    toks: Vec<(Tok, usize)>,
}

fn line_of(src: &str, pos: usize) -> usize {
    src[..pos.min(src.len())].matches('\n').count() + 1
}

const PUNCTS: [&str; 19] = [
    ":-", "?-", "!=", "<=", ">=", "(", ")", "{", "}", "[", "]", ",", ".", ":", "!", "~", "@", "=",
    "<",
];

fn lex(src: &str) -> Result<Lexed, String> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut toks = Vec::new();
    let ident_char = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'#' || (c == b'/' && b.get(i + 1) == Some(&b'/')) || c == b'%' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start = i;
        if c == b'"' {
            let mut s = String::new();
            i += 1;
            loop {
                let Some(&d) = b.get(i) else {
                    return Err(format!("line {}: unterminated string", line_of(src, start)));
                };
                if d == b'"' {
                    i += 1;
                    break;
                }
                if d == b'\\' {
                    let e = b.get(i + 1).copied().unwrap_or(b'\\');
                    s.push(match e {
                        b'n' => '\n',
                        b't' => '\t',
                        other => other as char,
                    });
                    i += 2;
                    continue;
                }
                let ch = src[i..].chars().next().unwrap_or('?');
                s.push(ch);
                i += ch.len_utf8();
            }
            toks.push((Tok::Str(s), start));
            continue;
        }
        if c.is_ascii_digit() || (c == b'-' && b.get(i + 1).is_some_and(u8::is_ascii_digit)) {
            i += 1;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            let n: i64 = src[start..i]
                .parse()
                .map_err(|_| format!("line {}: bad number", line_of(src, start)))?;
            // A duration literal: 30s, 5m, 1h, 250ms; the value is milliseconds.
            let rest = &src[i..];
            let (mult, len) = if rest.starts_with("ms") {
                (1, 2)
            } else if rest.starts_with('s') {
                (1_000, 1)
            } else if rest.starts_with('m') {
                (60_000, 1)
            } else if rest.starts_with('h') {
                (3_600_000, 1)
            } else {
                (1, 0)
            };
            if len > 0 && !b.get(i + len).copied().is_some_and(ident_char) {
                i += len;
                toks.push((Tok::Int(n * mult), start));
            } else {
                toks.push((Tok::Int(n), start));
            }
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            i += 1;
            loop {
                match b.get(i) {
                    Some(&d) if ident_char(d) => i += 1,
                    // A dot or hyphen inside a name joins two name characters: `names.shim`,
                    // `build-outside-slot`; a dot before a space ends a clause.
                    Some(b'.') | Some(b'-') if b.get(i + 1).copied().is_some_and(ident_char) => {
                        i += 2
                    }
                    _ => break,
                }
            }
            let word = &src[start..i];
            let tok = if word.starts_with(|ch: char| ch.is_ascii_uppercase() || ch == '_') {
                Tok::Var(word.to_owned())
            } else {
                Tok::Ident(word.to_owned())
            };
            toks.push((tok, start));
            continue;
        }
        let Some(p) = PUNCTS
            .iter()
            .chain([">"].iter())
            .find(|p| src[i..].starts_with(**p))
        else {
            return Err(format!(
                "line {}: unexpected character {:?}",
                line_of(src, i),
                src[i..].chars().next().unwrap_or(' ')
            ));
        };
        i += p.len();
        toks.push((Tok::Punct(p), start));
    }
    Ok(Lexed { toks })
}

struct Parser<'a> {
    src: &'a str,
    toks: Vec<(Tok, usize)>,
    at: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at).map(|t| &t.0)
    }

    fn peek2(&self) -> Option<&Tok> {
        self.toks.get(self.at + 1).map(|t| &t.0)
    }

    fn pos(&self) -> usize {
        self.toks
            .get(self.at)
            .map(|t| t.1)
            .unwrap_or(self.src.len())
    }

    fn err<T>(&self, what: &str) -> Result<T, String> {
        let found = match self.peek() {
            Some(Tok::Ident(s) | Tok::Var(s)) => s.clone(),
            Some(Tok::Str(s)) => format!("{s:?}"),
            Some(Tok::Int(n)) => n.to_string(),
            Some(Tok::Punct(p)) => (*p).to_owned(),
            None => "end of input".to_owned(),
        };
        Err(format!(
            "line {}: expected {what}, found {found}",
            line_of(self.src, self.pos())
        ))
    }

    fn is(&self, p: &str) -> bool {
        matches!(self.peek(), Some(Tok::Punct(q)) if *q == p)
    }

    fn eat(&mut self, p: &str) -> bool {
        if self.is(p) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, p: &str) -> Result<(), String> {
        if self.eat(p) {
            Ok(())
        } else {
            self.err(&format!("`{p}`"))
        }
    }

    fn ident(&mut self) -> Result<String, String> {
        match self.peek().cloned() {
            Some(Tok::Ident(s)) => {
                self.at += 1;
                Ok(s)
            }
            _ => self.err("a name"),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        match self.peek().cloned() {
            Some(Tok::Str(s)) => {
                self.at += 1;
                Ok(s)
            }
            _ => self.err("a string"),
        }
    }

    fn term(&mut self) -> Result<Term, String> {
        let t = match self.peek().cloned() {
            Some(Tok::Var(v)) if v == "_" => Term::Wild,
            Some(Tok::Var(v)) => Term::Var(v),
            Some(Tok::Str(s)) => Term::Str(s),
            Some(Tok::Int(n)) => Term::Int(n),
            Some(Tok::Ident(s)) if s == "true" => Term::Bool(true),
            Some(Tok::Ident(s)) if s == "false" => Term::Bool(false),
            _ => return self.err("a variable, string, number or boolean"),
        };
        self.at += 1;
        Ok(t)
    }

    fn module(&mut self) -> Result<Module, String> {
        let mut m = Module::default();
        while self.peek().is_some() {
            let start = self.pos();
            if self.eat("?-") {
                let body = self.body()?;
                let end = self.pos();
                self.expect(".")?;
                m.queries.push(Query {
                    body,
                    text: self.src[start..end].trim().to_owned(),
                });
                continue;
            }
            match self.peek().cloned() {
                Some(Tok::Ident(k))
                    if k == "names" && matches!(self.peek2(), Some(Tok::Ident(_))) =>
                {
                    self.at += 1;
                    let name = self.ident()?;
                    self.expect("=")?;
                    self.expect("[")?;
                    let mut items = Vec::new();
                    while !self.is("]") {
                        items.push(self.string()?);
                        if !self.eat(",") {
                            break;
                        }
                    }
                    self.expect("]")?;
                    self.expect(".")?;
                    m.names.insert(name, items);
                }
                Some(Tok::Ident(k))
                    if k == "check" && matches!(self.peek2(), Some(Tok::Ident(_))) =>
                {
                    self.at += 1;
                    m.checks.push(self.check()?);
                    self.eat(".");
                }
                Some(Tok::Ident(_)) => {
                    let head = self.ident()?;
                    self.expect("(")?;
                    let args = self.terms(")")?;
                    let body = if self.eat(":-") {
                        self.body()?
                    } else {
                        Vec::new()
                    };
                    self.expect(".")?;
                    m.rules.push(Rule { head, args, body });
                }
                _ => return self.err("a rule, a query `?-`, `names` or `check`"),
            }
        }
        Ok(m)
    }

    fn check(&mut self) -> Result<Check, String> {
        let name = self.ident()?;
        self.expect("{")?;
        let mut c = Check {
            name,
            about: String::new(),
            severity: "medium".to_owned(),
            expect: Expect::None,
            find: String::new(),
        };
        while !self.eat("}") {
            let key = self.ident()?;
            match key.as_str() {
                "about" => c.about = self.string()?,
                "severity" => c.severity = self.ident()?,
                "expect" => {
                    c.expect = match self.ident()?.as_str() {
                        "none" => Expect::None,
                        "some" => Expect::Some,
                        _ => return self.err("`none` or `some`"),
                    }
                }
                "find" => c.find = self.ident()?,
                "profiles" => {
                    self.expect("[")?;
                    while !self.eat("]") {
                        self.at += 1;
                    }
                }
                _ => return self.err("about, severity, expect, find or profiles"),
            }
        }
        if c.find.is_empty() {
            return self.err("`find <rule>` in the check");
        }
        Ok(c)
    }

    fn terms(&mut self, close: &str) -> Result<Vec<Term>, String> {
        let mut out = Vec::new();
        while !self.is(close) {
            out.push(self.term()?);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(close)?;
        Ok(out)
    }

    fn body(&mut self) -> Result<Vec<Lit>, String> {
        let mut out = vec![self.lit()?];
        while self.eat(",") {
            out.push(self.lit()?);
        }
        Ok(out)
    }

    fn lit(&mut self) -> Result<Lit, String> {
        if self.eat("!") {
            if self.eat("(") {
                let inner = self.lit()?;
                self.expect(")")?;
                return Ok(Lit::Not(Box::new(inner)));
            }
            return Ok(Lit::Not(Box::new(self.lit()?)));
        }
        if let Some(Tok::Ident(name)) = self.peek().cloned()
            && !matches!(name.as_str(), "true" | "false")
        {
            self.at += 1;
            if name == "seq" && self.is("(") {
                return self.seq();
            }
            if self.eat("{") {
                let mut named = Vec::new();
                while !self.is("}") {
                    let field = self.ident()?;
                    self.expect(":")?;
                    let pat = if self.eat("~") {
                        FieldPat::Regex(self.string()?)
                    } else {
                        FieldPat::Term(self.term()?)
                    };
                    named.push((field, pat));
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect("}")?;
                return Ok(Lit::Atom(Atom {
                    rel: name,
                    positional: Vec::new(),
                    named,
                    record: true,
                }));
            }
            self.expect("(")?;
            let positional = self.terms(")")?;
            return Ok(Lit::Atom(Atom {
                rel: name,
                positional,
                named: Vec::new(),
                record: false,
            }));
        }
        let left = self.term()?;
        if self.eat("~") {
            return Ok(Lit::Match(left, self.string()?));
        }
        if matches!(self.peek(), Some(Tok::Ident(k)) if k == "in") {
            self.at += 1;
            self.expect("@")?;
            let name = self.ident()?;
            let name = name.strip_prefix("names.").unwrap_or(&name).to_owned();
            return Ok(Lit::In(left, name));
        }
        let op = match self.peek() {
            Some(Tok::Punct("=")) => CmpOp::Eq,
            Some(Tok::Punct("!=")) => CmpOp::Ne,
            Some(Tok::Punct("<")) => CmpOp::Lt,
            Some(Tok::Punct("<=")) => CmpOp::Le,
            Some(Tok::Punct(">")) => CmpOp::Gt,
            Some(Tok::Punct(">=")) => CmpOp::Ge,
            _ => return self.err("a comparison, `~` or `in`"),
        };
        self.at += 1;
        if op == CmpOp::Eq
            && let Some(Tok::Ident(k)) = self.peek().cloned()
        {
            let agg = match k.as_str() {
                "count" => Some(AggOp::Count),
                "sum" => Some(AggOp::Sum),
                "min" => Some(AggOp::Min),
                "max" => Some(AggOp::Max),
                _ => None,
            };
            if let Some(agg) = agg {
                self.at += 1;
                let of = if agg == AggOp::Count {
                    None
                } else {
                    self.expect("(")?;
                    let t = self.term()?;
                    self.expect(")")?;
                    Some(t)
                };
                self.expect("{")?;
                let body = self.body()?;
                self.expect("}")?;
                return Ok(Lit::Agg {
                    out: left,
                    op: agg,
                    of,
                    body,
                });
            }
        }
        let right = self.term()?;
        Ok(Lit::Cmp(left, op, right))
    }

    fn seq(&mut self) -> Result<Lit, String> {
        self.expect("(")?;
        let mut nodes = Vec::new();
        let (mut scope, mut within, mut gap, mut nearest) = (Scope::Agent, None, None, true);
        while !self.is(")") {
            if matches!(self.peek(), Some(Tok::Ident(_)))
                && matches!(self.peek2(), Some(Tok::Punct(":")))
            {
                let key = self.ident()?;
                self.expect(":")?;
                match key.as_str() {
                    "scope" => {
                        scope = match self.ident()?.as_str() {
                            "agent" => Scope::Agent,
                            "session" => Scope::Session,
                            _ => return self.err("`agent` or `session`"),
                        }
                    }
                    "within" => match self.term()? {
                        Term::Int(n) => within = Some(n),
                        _ => return self.err("a duration"),
                    },
                    "gap" => match self.term()? {
                        Term::Int(n) => gap = Some(n),
                        _ => return self.err("a number"),
                    },
                    "pick" => {
                        nearest = match self.ident()?.as_str() {
                            "nearest" => true,
                            "all" => false,
                            _ => return self.err("`nearest` or `all`"),
                        }
                    }
                    _ => return self.err("scope, within, gap or pick"),
                }
            } else {
                nodes.push(self.term()?);
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        if nodes.len() < 2 {
            return self.err("at least two nodes in seq");
        }
        Ok(Lit::Seq {
            nodes,
            scope,
            within,
            gap,
            nearest,
        })
    }
}

pub fn parse(src: &str) -> Result<Module, String> {
    let lexed = lex(src)?;
    let mut p = Parser {
        src,
        toks: lexed.toks,
        at: 0,
    };
    p.module()
}
