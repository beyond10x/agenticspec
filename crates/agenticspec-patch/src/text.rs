//! The text of a patch: rules split at their final `.`, a rule split into its head and its body,
//! and the head parsed into its verb and arguments. The body is sq and parsed by sq.

/// An argument of a rule's head.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Var(String),
    Wild,
    /// A lower-case name: a field (`count`) or a relation of a record.
    Name(String),
    Str(String),
    Int(i64),
    /// A duration literal (`10m`), in milliseconds.
    Duration(i64),
    Bool(bool),
    /// `part{id: "x", block: "agent"}`: a relation and its fields, in the order written.
    Record(String, Vec<(String, Arg)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Head {
    pub verb: String,
    pub args: Vec<Arg>,
}

fn ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The rules of a patch's text form, each ending at a `.` outside a string and a comment that no
/// name character follows. Comments between rules are dropped; a comment inside a rule stays in
/// its text. A last rule without its `.` is kept as written.
pub fn split(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut rules = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let started = !current.trim().is_empty();
        let comment = c == '#' || c == '%' || (c == '/' && chars.get(i + 1) == Some(&'/'));
        if comment {
            while i < chars.len() && chars[i] != '\n' {
                if started {
                    current.push(chars[i]);
                }
                i += 1;
            }
            continue;
        }
        if c == '"' {
            current.push(c);
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                current.push(d);
                i += 1;
                if d == '\\' {
                    if let Some(&e) = chars.get(i) {
                        current.push(e);
                        i += 1;
                    }
                } else if d == '"' {
                    break;
                }
            }
            continue;
        }
        current.push(c);
        i += 1;
        if c == '.' && !chars.get(i).copied().is_some_and(ident_char) {
            rules.push(current.trim().to_string());
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        rules.push(current.trim().to_string());
    }
    rules
}

/// Byte positions in `text` outside strings and comments, with the nesting depth of `(`, `{`
/// and `[` before each.
fn code_positions(text: &str) -> Vec<(usize, char, i32)> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut it = text.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        match c {
            '"' => {
                while let Some((_, d)) = it.next() {
                    if d == '\\' {
                        it.next();
                    } else if d == '"' {
                        break;
                    }
                }
            }
            '#' | '%' => {
                for (_, d) in it.by_ref() {
                    if d == '\n' {
                        break;
                    }
                }
            }
            '/' if it.peek().is_some_and(|(_, d)| *d == '/') => {
                for (_, d) in it.by_ref() {
                    if d == '\n' {
                        break;
                    }
                }
            }
            _ => {
                out.push((i, c, depth));
                match c {
                    '(' | '{' | '[' => depth += 1,
                    ')' | '}' | ']' => depth -= 1,
                    _ => {}
                }
            }
        }
    }
    out
}

/// A rule's head text and its body text (none for a rule without `:-`), without the final `.`.
pub fn split_rule(rule: &str) -> Result<(&str, Option<&str>), String> {
    let rule = rule.trim();
    let Some(text) = rule.strip_suffix('.') else {
        return Err("a rule ends with `.`".into());
    };
    let code = code_positions(text);
    let neck = code
        .windows(2)
        .find(|w| w[0].1 == ':' && w[1].1 == '-' && w[0].2 == 0 && w[1].0 == w[0].0 + 1)
        .map(|w| w[0].0);
    Ok(match neck {
        Some(at) => (text[..at].trim(), Some(text[at + 2..].trim())),
        None => (text.trim(), None),
    })
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Name(String),
    Var(String),
    Str(String),
    Int(i64),
    Duration(i64),
    Punct(char),
}

fn lex(text: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '#' || c == '%' || (c == '/' && chars.get(i + 1) == Some(&'/')) {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '"' {
            let mut s = String::new();
            i += 1;
            loop {
                let Some(&d) = chars.get(i) else {
                    return Err("unterminated string".into());
                };
                i += 1;
                match d {
                    '"' => break,
                    '\\' => {
                        let e = chars.get(i).copied().unwrap_or('\\');
                        i += 1;
                        s.push(match e {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                    }
                    other => s.push(other),
                }
            }
            toks.push(Tok::Str(s));
            continue;
        }
        if c.is_ascii_digit() || (c == '-' && chars.get(i + 1).is_some_and(char::is_ascii_digit)) {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let digits: String = chars[start..i].iter().collect();
            let n: i64 = digits.parse().map_err(|_| format!("bad number {digits}"))?;
            let rest: String = chars[i..].iter().take(3).collect();
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
            if len > 0 && !chars.get(i + len).copied().is_some_and(ident_char) {
                i += len;
                toks.push(Tok::Duration(n * mult));
            } else {
                toks.push(Tok::Int(n));
            }
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && ident_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            toks.push(if c.is_ascii_uppercase() || c == '_' {
                Tok::Var(word)
            } else {
                Tok::Name(word)
            });
            continue;
        }
        if "(){},:".contains(c) {
            toks.push(Tok::Punct(c));
            i += 1;
            continue;
        }
        return Err(format!("unexpected character {c:?} in the head"));
    }
    Ok(toks)
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn eat(&mut self, p: char) -> bool {
        if self.peek() == Some(&Tok::Punct(p)) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, p: char) -> Result<(), String> {
        if self.eat(p) {
            Ok(())
        } else {
            Err(format!(
                "expected `{p}` in the head, found {}",
                self.found()
            ))
        }
    }

    fn found(&self) -> String {
        match self.peek() {
            None => "its end".into(),
            Some(Tok::Name(s) | Tok::Var(s)) => format!("`{s}`"),
            Some(Tok::Str(s)) => format!("{s:?}"),
            Some(Tok::Int(n) | Tok::Duration(n)) => n.to_string(),
            Some(Tok::Punct(p)) => format!("`{p}`"),
        }
    }

    fn name(&mut self) -> Result<String, String> {
        match self.peek().cloned() {
            Some(Tok::Name(s)) => {
                self.at += 1;
                Ok(s)
            }
            _ => Err(format!(
                "expected a name in the head, found {}",
                self.found()
            )),
        }
    }

    fn arg(&mut self) -> Result<Arg, String> {
        let Some(tok) = self.peek().cloned() else {
            return Err("the head ends inside its arguments".into());
        };
        self.at += 1;
        Ok(match tok {
            Tok::Var(v) if v == "_" => Arg::Wild,
            Tok::Var(v) => Arg::Var(v),
            Tok::Str(s) => Arg::Str(s),
            Tok::Int(n) => Arg::Int(n),
            Tok::Duration(n) => Arg::Duration(n),
            Tok::Name(s) if s == "true" => Arg::Bool(true),
            Tok::Name(s) if s == "false" => Arg::Bool(false),
            Tok::Name(s) if self.eat('{') => {
                let mut fields = Vec::new();
                while !self.eat('}') {
                    let field = self.name()?;
                    self.expect(':')?;
                    fields.push((field, self.arg()?));
                    if !self.eat(',') {
                        self.expect('}')?;
                        break;
                    }
                }
                Arg::Record(s, fields)
            }
            Tok::Name(s) => Arg::Name(s),
            Tok::Punct(p) => return Err(format!("unexpected `{p}` in the head")),
        })
    }
}

/// The verb of a head: the lower-case name it starts with, whatever follows it.
pub fn verb(head: &str) -> Option<&str> {
    let head = head.trim_start();
    let end = head.find(|c: char| !ident_char(c)).unwrap_or(head.len());
    let name = &head[..end];
    name.starts_with(|c: char| c.is_ascii_lowercase())
        .then_some(name)
}

/// The head `verb(arg, ...)`.
pub fn parse_head(head: &str) -> Result<Head, String> {
    let mut p = Parser {
        toks: lex(head)?,
        at: 0,
    };
    let verb = p.name()?;
    p.expect('(')?;
    let mut args = Vec::new();
    if !p.eat(')') {
        loop {
            args.push(p.arg()?);
            if p.eat(')') {
                break;
            }
            p.expect(',')?;
        }
    }
    if p.peek().is_some() {
        return Err(format!("{} after the head", p.found()));
    }
    Ok(Head { verb, args })
}
