//! The predicate language of routing invariants: what a link of an architecture promises about
//! the trace of a run. Its grammar (also on `docs/contract.md`):
//!
//! ```text
//! predicate = clause { "and" clause }
//! clause    = "every" use "inside" use
//!           | "every" use "to" part [ "over" part ]
//! use       = port "@" part
//! port      = word "." word
//! word      = a lowercase letter, then lowercase letters, digits and "-"
//! part      = a letter or "_", then letters, digits, "_" and "-"
//! ```
//!
//! Words are separated by spaces. `every model.inference@worker inside gateway.call@gateway`:
//! each use of the inference port by part `worker` lies inside a use of the gateway's call port by
//! part `gateway`. `every question.question@operator to owner over chat`: each use of the
//! question port by `operator` has `owner` as its counterparty, carried over the channel part
//! `chat`.
//!
//! [`Predicate::parse`] reads text into the typed form or refuses it naming the offset (in bytes)
//! and what was expected there; printing a predicate ([`std::fmt::Display`]) and parsing the text
//! gives the same form. The evaluation over a trace reads the typed form.

use std::fmt;

/// A port used by a part: `<port>@<part>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Use {
    pub port: String,
    pub part: String,
}

impl fmt::Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.port, self.part)
    }
}

/// One clause of a predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clause {
    /// `every <use> inside <use>`: each use of `every` lies inside a use of `within`.
    Inside { every: Use, within: Use },
    /// `every <use> to <part> [over <part>]`: each use of `every` has `to` as its counterparty,
    /// carried over channel part `over` when given.
    To {
        every: Use,
        to: String,
        over: Option<String>,
    },
}

impl Clause {
    /// The use every clause starts with.
    pub fn every(&self) -> &Use {
        match self {
            Clause::Inside { every, .. } | Clause::To { every, .. } => every,
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Clause::Inside { every, within } => write!(f, "every {every} inside {within}"),
            Clause::To { every, to, over } => {
                write!(f, "every {every} to {to}")?;
                if let Some(c) = over {
                    write!(f, " over {c}")?;
                }
                Ok(())
            }
        }
    }
}

/// A predicate: one or more clauses, all of which hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Predicate {
    pub clauses: Vec<Clause>,
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, c) in self.clauses.iter().enumerate() {
            if i > 0 {
                f.write_str(" and ")?;
            }
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

struct Parser<'a> {
    text: &'a str,
    at: usize,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &'a str {
        &self.text[self.at..]
    }

    fn spaces(&mut self) {
        while self.rest().starts_with(' ') {
            self.at += 1;
        }
    }

    fn expected(&self, what: &str) -> String {
        let found = match self.rest().chars().next() {
            None => "the end".to_owned(),
            Some(c) => format!("`{c}`"),
        };
        format!("expected {what} at offset {}, found {found}", self.at)
    }

    /// A keyword followed by a space or the end.
    fn keyword(&mut self, word: &str) -> bool {
        let rest = self.rest();
        if rest.starts_with(word) && rest[word.len()..].chars().next().is_none_or(|c| c == ' ') {
            self.at += word.len();
            true
        } else {
            false
        }
    }

    fn take(
        &mut self,
        first: impl Fn(char) -> bool,
        more: impl Fn(char) -> bool,
    ) -> Option<&'a str> {
        let rest = self.rest();
        let mut chars = rest.char_indices();
        match chars.next() {
            Some((_, c)) if first(c) => {}
            _ => return None,
        }
        let end = chars
            .find(|(_, c)| !more(*c))
            .map_or(rest.len(), |(i, _)| i);
        self.at += end;
        Some(&rest[..end])
    }

    fn word(&mut self) -> Option<&'a str> {
        self.take(
            |c| c.is_ascii_lowercase(),
            |c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-',
        )
    }

    fn part(&mut self, what: &str) -> Result<String, String> {
        self.take(
            |c| c.is_ascii_alphabetic() || c == '_',
            |c| c.is_ascii_alphanumeric() || c == '_' || c == '-',
        )
        .map(str::to_owned)
        .ok_or_else(|| self.expected(what))
    }

    fn port_use(&mut self) -> Result<Use, String> {
        let port_text = "a port (`block.entity`)";
        let block = self.word().ok_or_else(|| self.expected(port_text))?;
        if !self.rest().starts_with('.') {
            return Err(self.expected("`.`"));
        }
        self.at += 1;
        let entity = self.word().ok_or_else(|| self.expected(port_text))?;
        if !self.rest().starts_with('@') {
            return Err(self.expected("`@`"));
        }
        self.at += 1;
        let part = self.part("a part")?;
        Ok(Use {
            port: format!("{block}.{entity}"),
            part,
        })
    }

    fn clause(&mut self) -> Result<Clause, String> {
        self.spaces();
        if !self.keyword("every") {
            return Err(self.expected("`every`"));
        }
        self.spaces();
        let every = self.port_use()?;
        self.spaces();
        if self.keyword("inside") {
            self.spaces();
            let within = self.port_use()?;
            return Ok(Clause::Inside { every, within });
        }
        if self.keyword("to") {
            self.spaces();
            let to = self.part("a part")?;
            let mark = self.at;
            self.spaces();
            let over = if self.keyword("over") {
                self.spaces();
                Some(self.part("a channel part")?)
            } else {
                self.at = mark;
                None
            };
            return Ok(Clause::To { every, to, over });
        }
        Err(self.expected("`inside` or `to`"))
    }
}

impl Predicate {
    /// Read a predicate, or refuse it naming the byte offset and what was expected there.
    pub fn parse(text: &str) -> Result<Predicate, String> {
        let mut p = Parser { text, at: 0 };
        let mut clauses = vec![p.clause()?];
        loop {
            let mark = p.at;
            p.spaces();
            if p.rest().is_empty() {
                break;
            }
            if p.at == mark {
                return Err(p.expected("a space"));
            }
            if !p.keyword("and") {
                return Err(p.expected("`and` or the end"));
            }
            clauses.push(p.clause()?);
        }
        Ok(Predicate { clauses })
    }
}
