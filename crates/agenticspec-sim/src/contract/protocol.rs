//! The protocol of a port: a regular expression over the short names of its commands, intent
//! first, such as `SendMessage (MarkDelivered MarkRead | MarkLost)`.
//!
//! Written form: names are words of letters and digits; a space concatenates; `|` separates
//! alternatives (it binds weakest); `*`, `+` and `?` follow what they repeat; parentheses group.
//! [`Protocol::parse`] reads it, [`Protocol`]'s `Display` writes it in one canonical form
//! (alternatives sorted by their written text), and [`Protocol::compile`] turns it into a
//! [`regex::Regex`] over a trace written as each command's short name followed by one space.

use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A regular expression over command names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Protocol {
    /// The empty sequence.
    Empty,
    Name(String),
    Seq(Vec<Protocol>),
    Alt(Vec<Protocol>),
    Star(Box<Protocol>),
    Plus(Box<Protocol>),
    Opt(Box<Protocol>),
}

use Protocol::*;

impl Protocol {
    pub fn name(n: &str) -> Protocol {
        Name(n.to_owned())
    }

    /// `a` then `b`.
    pub fn then(a: Protocol, b: Protocol) -> Protocol {
        match (a, b) {
            (Empty, x) | (x, Empty) => x,
            (Seq(mut xs), Seq(ys)) => {
                xs.extend(ys);
                Seq(xs)
            }
            (Seq(mut xs), y) => {
                xs.push(y);
                Seq(xs)
            }
            (x, Seq(mut ys)) => {
                ys.insert(0, x);
                Seq(ys)
            }
            (x, y) => Seq(vec![x, y]),
        }
    }

    /// `a` or `b`; an alternative that is empty makes the rest optional.
    pub fn or(a: Protocol, b: Protocol) -> Protocol {
        let mut items: Vec<Protocol> = Vec::new();
        let mut empty = false;
        for p in [a, b] {
            match p {
                Alt(xs) => items.extend(xs),
                Opt(x) => {
                    empty = true;
                    items.push(*x);
                }
                Empty => empty = true,
                x => items.push(x),
            }
        }
        let mut seen = BTreeMap::new();
        for p in items {
            seen.entry(p.to_string()).or_insert(p);
        }
        let items: Vec<Protocol> = seen.into_values().collect();
        let body = match items.len() {
            0 => Empty,
            1 => items.into_iter().next().unwrap_or(Empty),
            _ => Alt(items),
        };
        if empty { Protocol::opt(body) } else { body }
    }

    pub fn star(p: Protocol) -> Protocol {
        match p {
            Empty => Empty,
            Star(x) | Plus(x) | Opt(x) => Star(x),
            x => Star(Box::new(x)),
        }
    }

    pub fn opt(p: Protocol) -> Protocol {
        match p {
            Empty => Empty,
            Star(x) => Star(x),
            Plus(x) => Star(x),
            Opt(x) => Opt(x),
            x => Opt(Box::new(x)),
        }
    }

    /// Every command name the protocol mentions.
    pub fn names(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        self.collect(&mut out);
        out
    }

    fn collect(&self, out: &mut BTreeSet<String>) {
        match self {
            Empty => {}
            Name(n) => {
                out.insert(n.clone());
            }
            Seq(xs) | Alt(xs) => xs.iter().for_each(|x| x.collect(out)),
            Star(x) | Plus(x) | Opt(x) => x.collect(out),
        }
    }

    /// The first names a trace of this protocol can start with.
    pub fn first(&self) -> BTreeSet<String> {
        match self {
            Empty => BTreeSet::new(),
            Name(n) => BTreeSet::from([n.clone()]),
            Alt(xs) => xs.iter().flat_map(Protocol::first).collect(),
            Star(x) | Plus(x) | Opt(x) => x.first(),
            Seq(xs) => {
                let mut out = BTreeSet::new();
                for x in xs {
                    out.extend(x.first());
                    if !x.nullable() {
                        break;
                    }
                }
                out
            }
        }
    }

    fn nullable(&self) -> bool {
        match self {
            Empty | Star(_) | Opt(_) => true,
            Name(_) => false,
            Plus(x) => x.nullable(),
            Seq(xs) => xs.iter().all(Protocol::nullable),
            Alt(xs) => xs.iter().any(Protocol::nullable),
        }
    }

    /// The protocol as a regular expression of the `regex` crate over a trace in which each
    /// command's short name is followed by one space, anchored at both ends.
    pub fn compile(&self) -> Result<Regex, String> {
        fn re(p: &Protocol, out: &mut String) {
            match p {
                Empty => {}
                Name(n) => {
                    out.push_str("(?:");
                    out.push_str(n);
                    out.push_str(" )");
                }
                Seq(xs) => xs.iter().for_each(|x| re(x, out)),
                Alt(xs) => {
                    out.push_str("(?:");
                    for (i, x) in xs.iter().enumerate() {
                        if i > 0 {
                            out.push('|');
                        }
                        re(x, out);
                    }
                    out.push(')');
                }
                Star(x) | Plus(x) | Opt(x) => {
                    out.push_str("(?:");
                    re(x, out);
                    out.push(')');
                    out.push(match p {
                        Star(_) => '*',
                        Plus(_) => '+',
                        _ => '?',
                    });
                }
            }
        }
        let mut text = String::from("^");
        re(self, &mut text);
        text.push('$');
        Regex::new(&text).map_err(|e| e.to_string())
    }

    /// Whether the commands of `trace`, in order, are a whole trace of this protocol.
    pub fn accepts(&self, trace: &[&str]) -> Result<bool, String> {
        let text: String = trace.iter().map(|c| format!("{c} ")).collect();
        Ok(self.compile()?.is_match(&text))
    }

    /// Read the written form. A refusal names the byte offset where the text stops parsing.
    pub fn parse(text: &str) -> Result<Protocol, String> {
        let mut p = Parser {
            text: text.as_bytes(),
            at: 0,
        };
        let out = p.alt()?;
        p.space();
        if p.at < p.text.len() {
            return Err(format!(
                "protocol `{text}`: unexpected `{}` at offset {}",
                p.text[p.at] as char, p.at
            ));
        }
        if out == Empty {
            return Err(format!("protocol `{text}`: empty"));
        }
        Ok(out)
    }
}

struct Parser<'a> {
    text: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.at < self.text.len() && self.text[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.space();
        self.text.get(self.at).copied()
    }

    fn alt(&mut self) -> Result<Protocol, String> {
        let mut out = self.seq()?;
        while self.peek() == Some(b'|') {
            self.at += 1;
            let next = self.seq()?;
            if next == Empty {
                return Err(self.expected("a name or `(` after `|`"));
            }
            out = Protocol::or(out, next);
        }
        Ok(out)
    }

    fn seq(&mut self) -> Result<Protocol, String> {
        let mut out = Empty;
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_alphanumeric() || c == b'(' => {
                    let item = self.post()?;
                    out = Protocol::then(out, item);
                }
                _ => return Ok(out),
            }
        }
    }

    fn post(&mut self) -> Result<Protocol, String> {
        let mut out = self.atom()?;
        loop {
            match self.text.get(self.at) {
                Some(b'*') => out = Protocol::star(out),
                Some(b'?') => out = Protocol::opt(out),
                Some(b'+') => {
                    out = match out {
                        Star(x) => Star(x),
                        Plus(x) => Plus(x),
                        Opt(x) => Star(x),
                        x => Plus(Box::new(x)),
                    }
                }
                _ => return Ok(out),
            }
            self.at += 1;
        }
    }

    fn atom(&mut self) -> Result<Protocol, String> {
        match self.peek() {
            Some(b'(') => {
                self.at += 1;
                let inner = self.alt()?;
                if inner == Empty {
                    return Err(self.expected("a name inside `(`"));
                }
                if self.peek() != Some(b')') {
                    return Err(self.expected("`)`"));
                }
                self.at += 1;
                Ok(inner)
            }
            Some(c) if c.is_ascii_alphanumeric() => {
                let start = self.at;
                while self
                    .text
                    .get(self.at)
                    .is_some_and(|c| c.is_ascii_alphanumeric())
                {
                    self.at += 1;
                }
                Ok(Name(
                    String::from_utf8_lossy(&self.text[start..self.at]).into_owned(),
                ))
            }
            _ => Err(self.expected("a name or `(`")),
        }
    }

    fn expected(&self, what: &str) -> String {
        let text = String::from_utf8_lossy(self.text);
        match self.text.get(self.at) {
            Some(c) => format!(
                "protocol `{text}`: expected {what} at offset {}, found `{}`",
                self.at, *c as char
            ),
            None => format!(
                "protocol `{text}`: expected {what} at offset {}, found the end",
                self.at
            ),
        }
    }
}

fn grouped(p: &Protocol) -> String {
    match p {
        Alt(_) | Seq(_) => format!("({p})"),
        _ => p.to_string(),
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Empty => Ok(()),
            Name(n) => f.write_str(n),
            Seq(xs) => {
                let parts: Vec<String> = xs
                    .iter()
                    .map(|x| match x {
                        Alt(_) => format!("({x})"),
                        _ => x.to_string(),
                    })
                    .collect();
                f.write_str(&parts.join(" "))
            }
            Alt(xs) => {
                let mut parts: Vec<String> = xs.iter().map(ToString::to_string).collect();
                parts.sort();
                f.write_str(&parts.join(" | "))
            }
            Star(x) => write!(f, "{}*", grouped(x)),
            Plus(x) => write!(f, "{}+", grouped(x)),
            Opt(x) => write!(f, "{}?", grouped(x)),
        }
    }
}

/// An automaton of commands: from a start through states to an end, each edge one command.
/// [`Automaton::protocol`] turns it into a [`Protocol`] by eliminating its states, the deepest
/// first, so an acyclic automaton gives a nested expression.
#[derive(Default)]
pub struct Automaton {
    edges: BTreeMap<(Node, Node), Protocol>,
}

/// A node of an [`Automaton`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    Start,
    State(String),
    End,
}

impl Automaton {
    pub fn edge(&mut self, from: Node, to: Node, label: Protocol) {
        let key = (from, to);
        let next = match self.edges.remove(&key) {
            Some(old) => Protocol::or(old, label),
            None => label,
        };
        self.edges.insert(key, next);
    }

    /// The protocol of every path from the start to the end.
    pub fn protocol(mut self) -> Protocol {
        let mut depth: BTreeMap<Node, usize> = BTreeMap::new();
        let mut frontier = vec![Node::Start];
        depth.insert(Node::Start, 0);
        while !frontier.is_empty() {
            let mut next = Vec::new();
            for n in frontier {
                let d = depth[&n];
                for (from, to) in self.edges.keys() {
                    if *from == n && !depth.contains_key(to) {
                        depth.insert(to.clone(), d + 1);
                        next.push(to.clone());
                    }
                }
            }
            frontier = next;
        }
        let mut states: Vec<Node> = self
            .edges
            .keys()
            .flat_map(|(a, b)| [a.clone(), b.clone()])
            .filter(|n| matches!(n, Node::State(_)))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        states.sort_by(|a, b| {
            let da = depth.get(a).copied().unwrap_or(usize::MAX);
            let db = depth.get(b).copied().unwrap_or(usize::MAX);
            db.cmp(&da).then_with(|| a.cmp(b))
        });
        for k in states {
            let lp = self
                .edges
                .remove(&(k.clone(), k.clone()))
                .map_or(Empty, Protocol::star);
            let ins: Vec<(Node, Protocol)> = self
                .edges
                .iter()
                .filter(|((_, to), _)| *to == k)
                .map(|((from, _), p)| (from.clone(), p.clone()))
                .collect();
            let outs: Vec<(Node, Protocol)> = self
                .edges
                .iter()
                .filter(|((from, _), _)| *from == k)
                .map(|((_, to), p)| (to.clone(), p.clone()))
                .collect();
            self.edges.retain(|(a, b), _| *a != k && *b != k);
            for (i, pin) in &ins {
                for (j, pout) in &outs {
                    let path =
                        Protocol::then(Protocol::then(pin.clone(), lp.clone()), pout.clone());
                    self.edge(i.clone(), j.clone(), path);
                }
            }
        }
        self.edges
            .remove(&(Node::Start, Node::End))
            .unwrap_or(Empty)
    }
}
