//! The row predicate of a read (`reads.filter`) and the condition of a confirm (`confirm.show`):
//! the grammar of released `ess-ui/1` (ESS 0.99.2, `ess_ui::filter`), parsed and compiled into a
//! TypeScript expression over the helpers of the runtime (`same`, `member`, `contains`, `truthy`,
//! `at`).
//!
//! The grammar: `or` of `and` of a unary form; a unary form is `not <unary>` or a primary, then
//! optionally `==`, `!=`, `in [..]` or `contains` and a primary. A primary is `( .. )`, a list
//! `[ .. ]`, a quoted text, a number, `true`, `false`, `null`, a dotted path, or a bare word, which
//! is text. `in` takes a literal list only. A filter's top-level form is a comparison, `not`, `and`
//! or `or`; a bare path or literal is no predicate. Paths read `row`, `params`, `state`, `shell` and
//! `args` only.

/// A parsed expression of the grammar.
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// Text, a number (as written), a boolean or null, as a TypeScript literal.
    Literal(String),
    Path(Vec<String>),
    List(Vec<Predicate>),
    Not(Box<Predicate>),
    Binary(Op, Box<Predicate>, Box<Predicate>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Equal,
    NotEqual,
    In,
    Contains,
    And,
    Or,
}

impl Predicate {
    /// Whether the form always gives a boolean: what a filter's top level must be.
    pub fn boolean(&self) -> bool {
        matches!(self, Predicate::Not(_) | Predicate::Binary(..))
    }

    /// Every path the expression reads.
    pub fn paths(&self) -> Vec<&[String]> {
        let mut out = Vec::new();
        self.visit(&mut out);
        out
    }

    fn visit<'a>(&'a self, out: &mut Vec<&'a [String]>) {
        match self {
            Predicate::Path(p) => out.push(p),
            Predicate::List(items) => items.iter().for_each(|i| i.visit(out)),
            Predicate::Not(inner) => inner.visit(out),
            Predicate::Binary(_, l, r) => {
                l.visit(out);
                r.visit(out);
            }
            Predicate::Literal(_) => {}
        }
    }

    /// The states the right side of a `contains` reads, each a whole state (`state.<name>`), in
    /// order: a section draws a search box for each.
    pub fn searches(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        self.visit_searches(&mut out);
        out
    }

    fn visit_searches(&self, out: &mut Vec<String>) {
        match self {
            Predicate::Binary(Op::Contains, _, r) => {
                if let Predicate::Path(p) = r.as_ref()
                    && let [root, name] = p.as_slice()
                    && root == "state"
                    && !out.contains(name)
                {
                    out.push(name.clone());
                }
            }
            Predicate::Binary(_, l, r) => {
                l.visit_searches(out);
                r.visit_searches(out);
            }
            Predicate::Not(inner) => inner.visit_searches(out),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Text(String),
    Open,
    Close,
    List,
    EndList,
    Comma,
    Equal,
    NotEqual,
}

fn tokenize(text: &str) -> Option<Vec<Token>> {
    let mut chars = text.char_indices().peekable();
    let mut tokens = Vec::new();
    while let Some((start, c)) = chars.next() {
        tokens.push(match c {
            c if c.is_whitespace() => continue,
            '(' => Token::Open,
            ')' => Token::Close,
            '[' => Token::List,
            ']' => Token::EndList,
            ',' => Token::Comma,
            '=' | '!' => {
                chars.next_if(|(_, c)| *c == '=')?;
                if c == '=' {
                    Token::Equal
                } else {
                    Token::NotEqual
                }
            }
            '\'' | '"' => {
                let (end, _) = chars.find(|(_, next)| *next == c)?;
                Token::Text(text[start + 1..end].to_owned())
            }
            c if c.is_ascii_alphanumeric() || "_-".contains(c) => {
                let mut end = start + c.len_utf8();
                while let Some((at, c)) =
                    chars.next_if(|(_, c)| c.is_ascii_alphanumeric() || "_.-".contains(*c))
                {
                    end = at + c.len_utf8();
                }
                Token::Word(text[start..end].to_owned())
            }
            _ => return None,
        });
    }
    Some(tokens)
}

/// Parses the whole expression, or `None` when it is not one of the grammar.
pub fn parse(text: &str) -> Option<Predicate> {
    let tokens = tokenize(text)?;
    let mut p = Parser {
        tokens: &tokens,
        at: 0,
    };
    let out = p.or()?;
    (p.at == tokens.len()).then_some(out)
}

struct Parser<'a> {
    tokens: &'a [Token],
    at: usize,
}

impl Parser<'_> {
    fn eat(&mut self, t: &Token) -> bool {
        if self.tokens.get(self.at) == Some(t) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn word(&mut self, w: &str) -> bool {
        self.eat(&Token::Word(w.to_owned()))
    }

    fn or(&mut self) -> Option<Predicate> {
        let mut left = self.and()?;
        while self.word("or") {
            left = Predicate::Binary(Op::Or, Box::new(left), Box::new(self.and()?));
        }
        Some(left)
    }

    fn and(&mut self) -> Option<Predicate> {
        let mut left = self.unary()?;
        while self.word("and") {
            left = Predicate::Binary(Op::And, Box::new(left), Box::new(self.unary()?));
        }
        Some(left)
    }

    fn unary(&mut self) -> Option<Predicate> {
        if self.word("not") {
            return Some(Predicate::Not(Box::new(self.unary()?)));
        }
        let left = self.primary()?;
        let op = if self.eat(&Token::Equal) {
            Op::Equal
        } else if self.eat(&Token::NotEqual) {
            Op::NotEqual
        } else if self.word("in") {
            Op::In
        } else if self.word("contains") {
            Op::Contains
        } else {
            return Some(left);
        };
        let right = self.primary()?;
        if op == Op::In && !matches!(right, Predicate::List(_)) {
            return None;
        }
        Some(Predicate::Binary(op, Box::new(left), Box::new(right)))
    }

    fn primary(&mut self) -> Option<Predicate> {
        let token = self.tokens.get(self.at)?.clone();
        self.at += 1;
        match token {
            Token::Open => {
                let inner = self.or()?;
                self.eat(&Token::Close).then_some(inner)
            }
            Token::List => {
                let mut items = Vec::new();
                if !self.eat(&Token::EndList) {
                    loop {
                        items.push(self.or()?);
                        if self.eat(&Token::EndList) {
                            break;
                        }
                        if !self.eat(&Token::Comma) {
                            return None;
                        }
                    }
                }
                Some(Predicate::List(items))
            }
            Token::Text(t) => Some(Predicate::Literal(crate::check::ts_str(&t))),
            Token::Word(w) => {
                if ["and", "or", "not", "in", "contains"].contains(&w.as_str()) {
                    return None;
                }
                if w.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
                    let digits = w.strip_prefix('-').unwrap_or(&w);
                    let parts: Vec<&str> = digits.split('.').collect();
                    if parts.len() > 2
                        || parts
                            .iter()
                            .any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()))
                    {
                        return None;
                    }
                }
                match w.as_str() {
                    "true" | "false" | "null" => return Some(Predicate::Literal(w)),
                    _ => {}
                }
                if let Ok(n) = w.parse::<f64>()
                    && n.is_finite()
                {
                    return Some(Predicate::Literal(w));
                }
                if w.contains('.') {
                    let segments: Vec<String> = w.split('.').map(str::to_owned).collect();
                    if segments.iter().any(|s| {
                        s.is_empty()
                            || !s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                    }) {
                        return None;
                    }
                    Some(Predicate::Path(segments))
                } else {
                    Some(Predicate::Literal(crate::check::ts_str(&w)))
                }
            }
            _ => None,
        }
    }
}

/// How a path of each root reads in the generated code; `None` when the name is not declared where
/// the expression stands.
pub trait Roots {
    fn root(&self, root: &str, name: &str) -> Option<String>;
}

/// The TypeScript of an expression whose every path `roots` resolves. A path of more than one
/// segment below its root walks the value as the browser does (`at`).
pub fn compile(p: &Predicate, roots: &dyn Roots) -> String {
    match p {
        Predicate::Literal(l) => l.clone(),
        Predicate::Path(segments) => {
            let base = roots
                .root(&segments[0], &segments[1])
                .unwrap_or_else(|| "null".into());
            if segments.len() == 2 {
                base
            } else {
                let rest: Vec<String> = segments[1..]
                    .iter()
                    .map(|s| crate::check::ts_str(s))
                    .collect();
                // `row` is walked from the row, so `row.a.b` is `at(row, ['a', 'b'])`.
                if segments[0] == "row" {
                    format!("at(row, [{}])", rest.join(", "))
                } else {
                    format!("at({base}, [{}])", rest[1..].join(", "))
                }
            }
        }
        Predicate::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|i| compile(i, roots))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Predicate::Not(inner) => {
            if inner.boolean() {
                format!("!{}", compile(inner, roots))
            } else {
                format!("!truthy({})", compile(inner, roots))
            }
        }
        Predicate::Binary(op, l, r) => match op {
            Op::Equal => format!("same({}, {})", compile(l, roots), compile(r, roots)),
            Op::NotEqual => format!("!same({}, {})", compile(l, roots), compile(r, roots)),
            Op::In => format!("member({}, {})", compile(l, roots), compile(r, roots)),
            Op::Contains => format!("contains({}, {})", compile(l, roots), compile(r, roots)),
            Op::And | Op::Or => format!(
                "({} {} {})",
                operand(l, roots),
                if *op == Op::And { "&&" } else { "||" },
                operand(r, roots)
            ),
        },
    }
}

fn operand(p: &Predicate, roots: &dyn Roots) -> String {
    if p.boolean() {
        compile(p, roots)
    } else {
        format!("truthy({})", compile(p, roots))
    }
}
