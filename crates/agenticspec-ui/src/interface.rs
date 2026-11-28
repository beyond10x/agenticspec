//! The runtime interface of the generated UI as a document, `ui/runtime.openapi.yaml` (OpenAPI
//! 3.1): R1 read, R2 feed, R3 placements, R4 command and R5 action set at
//! `{root}/scopes/{scope}`. The fixture source, the conformance suite and the test that holds the
//! generated client to the document all read it from here.

use serde_json::Value;

/// The document, byte for byte.
pub const INTERFACE: &str = include_str!("../../../ui/runtime.openapi.yaml");

/// One operation of the document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Operation {
    /// `GET`, `POST`.
    pub method: String,
    /// The path below the scope, such as `/api/views/{view}`.
    pub path: String,
    pub id: String,
    /// The statuses the document declares, ascending.
    pub statuses: Vec<u16>,
}

/// The document, parsed once per process.
fn document() -> &'static Value {
    static DOC: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    DOC.get_or_init(|| serde_yaml_ng::from_str(INTERFACE).expect("ui/runtime.openapi.yaml is YAML"))
}

/// Every operation, ordered by path and method.
pub fn operations() -> Vec<Operation> {
    let doc = document();
    let mut out = Vec::new();
    if let Some(paths) = doc["paths"].as_object() {
        for (path, item) in paths {
            for method in ["get", "post", "put", "patch", "delete"] {
                let Some(op) = item.get(method) else { continue };
                let mut statuses: Vec<u16> = op["responses"]
                    .as_object()
                    .map(|r| r.keys().filter_map(|k| k.parse().ok()).collect())
                    .unwrap_or_default();
                statuses.sort_unstable();
                out.push(Operation {
                    method: method.to_ascii_uppercase(),
                    path: path.clone(),
                    id: op["operationId"].as_str().unwrap_or("").to_string(),
                    statuses,
                });
            }
        }
    }
    out.sort();
    out
}

fn runtime_ext(key: &str) -> Value {
    document()["x-agentic-runtime"][key].clone()
}

/// The views R3 reads at a scope.
pub fn r3_views() -> Vec<String> {
    runtime_ext("r3_views")
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// The path below the root that every scope starts with, `/scopes/`.
pub fn scope_prefix() -> String {
    runtime_ext("scope_prefix")
        .as_str()
        .unwrap_or("/scopes/")
        .to_string()
}

/// The header a reconnect names its cursor in.
pub fn cursor_header() -> String {
    runtime_ext("cursor_header")
        .as_str()
        .unwrap_or("Last-Event-ID")
        .to_string()
}

/// The name of the session cookie a browser sends.
pub fn session_cookie() -> String {
    runtime_ext("session_cookie")
        .as_str()
        .unwrap_or("agentic_session")
        .to_string()
}

/// The URL of a scope below `root`: `{root}/scopes/{architecture}[/{part}[/{instance}]]`, each
/// segment encoded as the generated runtime encodes it.
pub fn scope_url(
    root: &str,
    architecture: &str,
    part: Option<&str>,
    instance: Option<u32>,
) -> String {
    let mut url = format!(
        "{}{}{}",
        root.trim_end_matches('/'),
        scope_prefix(),
        crate::wire::percent_encode(architecture)
    );
    if let Some(p) = part {
        url.push('/');
        url.push_str(&crate::wire::percent_encode(p));
        if let Some(i) = instance {
            url.push_str(&format!("/{i}"));
        }
    }
    url
}

/// The scope as text, as a grant's `scope` names it: `{architecture}[/{part}[/{instance}]]`.
pub fn scope_path(architecture: &str, part: Option<&str>, instance: Option<u32>) -> String {
    let mut s = architecture.to_string();
    if let Some(p) = part {
        s.push('/');
        s.push_str(p);
        if let Some(i) = instance {
            s.push_str(&format!("/{i}"));
        }
    }
    s
}

/// Whether a grant on `grant_scope` covers `scope`: the same scope or one above it.
pub fn covers(grant_scope: &str, scope: &str) -> bool {
    scope == grant_scope
        || scope
            .strip_prefix(grant_scope)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// The top-level field a row or a change signal names its scope in (`x-agentic-runtime.scope_field`).
pub fn scope_field() -> String {
    runtime_ext("scope_field")
        .as_str()
        .unwrap_or("scope")
        .to_string()
}

/// The error a source answers a view read with, with 404, when it does not serve the view
/// (`x-agentic-runtime.view_not_served`).
pub fn view_not_served() -> String {
    runtime_ext("view_not_served")
        .as_str()
        .unwrap_or("view_not_served")
        .to_string()
}

/// Whether a 404 body is the declared "not served" answer for `view`:
/// `{"error": "view_not_served", "view": "<view>"}`. Any other 404 is not.
pub fn is_view_not_served(body: &[u8], view: &str) -> bool {
    let v: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
    v["error"].as_str() == Some(view_not_served().as_str()) && v["view"].as_str() == Some(view)
}

/// The scope a row (or an event's data) names in its top-level [`scope_field`]: its value when
/// that is a scope of `architecture`, `{architecture}` or `{architecture}/…`. A field of that name
/// holding anything else (a view's own `scope` such as `Part`) names no scope.
pub fn named_scope<'a>(field: Option<&'a Value>, architecture: &str) -> Option<&'a str> {
    field
        .and_then(Value::as_str)
        .filter(|s| covers(architecture, s))
}

/// Whether a row is in scope at `scope` when it names one: `Some(true)` when `scope` is the
/// row's named scope or one above it, `Some(false)` when not, `None` when the row names no scope
/// and the entity-id rule ([`in_scope`]) decides.
pub fn named_in_scope(field: Option<&Value>, architecture: &str, scope: &str) -> Option<bool> {
    named_scope(field, architecture).map(|named| covers(scope, named))
}

/// The scope rule of R1 and R2 at `scope` (text, `{architecture}[/{part}[/{instance}]]`): the
/// row's named scope ([`named_scope`]) decides when it has one; else the entity-id rule over the
/// entities `placed` to that scope.
pub fn in_scope_at(
    row: &serde_json::Map<String, Value>,
    placed: &std::collections::BTreeSet<String>,
    architecture: &str,
    scope: &str,
) -> bool {
    named_in_scope(row.get(&scope_field()), architecture, scope)
        .unwrap_or_else(|| in_scope(row, placed))
}

/// Whether a row (or an event's data) is in scope of the placed entities `placed`: its key or a
/// top-level field that holds an entity's identifier equals one of them. One hop only.
pub fn in_scope(
    row: &serde_json::Map<String, Value>,
    placed: &std::collections::BTreeSet<String>,
) -> bool {
    in_scope_values(row.values(), placed)
}

/// [`in_scope`] over the values of the row's top-level fields.
pub fn in_scope_values<'a>(
    mut values: impl Iterator<Item = &'a Value>,
    placed: &std::collections::BTreeSet<String>,
) -> bool {
    values.any(|v| v.as_str().is_some_and(|s| placed.contains(s)))
}

/// A JSON object that keeps the order of its fields, so a row keeps its key first.
#[derive(Debug, Clone, PartialEq)]
pub struct Ordered(pub Vec<(String, Value)>);

impl Ordered {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.0.iter().map(|(_, v)| v)
    }

    /// [`in_scope_at`] of this row.
    pub fn in_scope_at(
        &self,
        placed: &std::collections::BTreeSet<String>,
        architecture: &str,
        scope: &str,
    ) -> bool {
        named_in_scope(self.get(&scope_field()), architecture, scope)
            .unwrap_or_else(|| in_scope_values(self.values(), placed))
    }
}

impl<'de> serde::Deserialize<'de> for Ordered {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Ordered, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Ordered;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut m: M,
            ) -> Result<Ordered, M::Error> {
                let mut out = Vec::new();
                while let Some(entry) = m.next_entry::<String, Value>()? {
                    out.push(entry);
                }
                Ok(Ordered(out))
            }
        }
        d.deserialize_map(V)
    }
}

impl serde::Serialize for Ordered {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut m = s.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            m.serialize_entry(k, v)?;
        }
        m.end()
    }
}
