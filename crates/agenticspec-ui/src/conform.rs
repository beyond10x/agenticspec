//! The conformance suite of the runtime interface (`ui/runtime.openapi.yaml`), run against a
//! source by its root URL: the fixture source first, then any runtime's adapter. A plan
//! (`agenticspec-runtime-conformance/1`, the fixture source's is `ui/views/source/plan.json`)
//! names what the source holds: the architecture, the scopes and their views, a channel that
//! sends events, commands with the answer expected, and the principal's grants.
//!
//! The checks, each named by what it requested:
//!
//! - `unauthenticated R1|R2|R4|R5 …`: without credentials the answer is 401 with no row.
//! - `R1 <view> at <scope>`: every row is in scope of the placements to that scope, and no row
//!   of the architecture's read that is in scope is left out. A failure names the view and the
//!   row's key.
//! - `R3 <view> at <scope>`: the placement views answer `{"rows": [...]}`.
//! - `R2 resume …`: a reconnect with the cursor of event k gets exactly the events after it, in
//!   order, and none at or before it. `R2 scope …`: every event is in scope.
//! - `R4 <command> at <scope>`: an accepted command answers `{"row"}`, a refused one 422 with the
//!   refusal's row.
//! - `R5 at <scope>`: every allowed command names a grant of the principal that covers it, and
//!   no covered command is withheld. `R5 hidden <command> at <scope>`: a command R5 does not
//!   allow is refused by R4.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::time::{Duration, Instant};

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value, json};

use crate::interface;
use crate::source::Grant;
use crate::wire;

/// What the source holds, as the tester set it up.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub format: String,
    pub architecture_id: String,
    /// The bearer token; `AGENTICSPEC_UI_TOKEN` replaces it in `agenticspec ui conform`.
    #[serde(default)]
    pub token: Option<String>,
    pub scopes: Vec<ScopePlan>,
    #[serde(default)]
    pub feed: Option<FeedPlan>,
    #[serde(default)]
    pub commands: Vec<CommandPlan>,
    #[serde(default)]
    pub actions: Option<ActionsPlan>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopePlan {
    pub part: String,
    #[serde(default)]
    pub instance: Option<u32>,
    pub views: Vec<String>,
}

fn five() -> usize {
    5
}
fn three() -> usize {
    3
}
fn wait() -> u64 {
    5000
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedPlan {
    #[serde(default)]
    pub part: Option<String>,
    #[serde(default)]
    pub instance: Option<u32>,
    pub channel: String,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    /// How many events the first connection waits for.
    #[serde(default = "five")]
    pub events: usize,
    /// The event whose cursor the reconnect names, counted from 1.
    #[serde(default = "three")]
    pub resume_after: usize,
    /// How long a connection waits for its events, in milliseconds.
    #[serde(default = "wait")]
    pub wait_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expect {
    Accepted,
    Refused,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandPlan {
    #[serde(default)]
    pub part: Option<String>,
    #[serde(default)]
    pub instance: Option<u32>,
    pub command: String,
    pub input: Value,
    pub expect: Expect,
    /// The error a refusal names.
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionsPlan {
    pub principal_id: String,
    /// The action class of each command (standing in for the core action class catalog).
    pub action_classes: BTreeMap<String, String>,
    pub grants: Vec<Grant>,
    /// An input per command, posted through R4 where R5 does not allow the command.
    #[serde(default)]
    pub probes: BTreeMap<String, Value>,
}

impl Plan {
    pub fn parse(text: &str) -> Result<Plan, String> {
        let plan: Plan = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if plan.format != "agenticspec-runtime-conformance/1" {
            return Err(format!(
                "format {} is not agenticspec-runtime-conformance/1",
                plan.format
            ));
        }
        Ok(plan)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Passed,
    Failed,
    Skipped,
}

impl Status {
    pub fn word(self) -> &'static str {
        match self {
            Status::Passed => "passed",
            Status::Failed => "failed",
            Status::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: String,
    pub status: Status,
    pub detail: String,
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.status.word(), self.name, self.detail)
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub source: String,
    pub checks: Vec<Check>,
}

impl Report {
    pub fn count(&self, s: Status) -> usize {
        self.checks.iter().filter(|c| c.status == s).count()
    }

    pub fn failed(&self) -> Vec<&Check> {
        self.checks
            .iter()
            .filter(|c| c.status == Status::Failed)
            .collect()
    }

    /// The checks whose name starts with `prefix`.
    pub fn named(&self, prefix: &str) -> Vec<&Check> {
        self.checks
            .iter()
            .filter(|c| c.name.starts_with(prefix))
            .collect()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "source": self.source,
            "passed": self.count(Status::Passed),
            "failed": self.count(Status::Failed),
            "skipped": self.count(Status::Skipped),
            "checks": self.checks.iter().map(|c| json!({
                "name": c.name, "status": c.status.word(), "detail": c.detail
            })).collect::<Vec<_>>(),
        })
    }
}

/// How long the suite waits.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// For one request or connection.
    pub timeout: Duration,
    /// With no event after a reconnect's last expected one, before it is judged.
    pub idle: Duration,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            timeout: Duration::from_secs(5),
            idle: Duration::from_millis(300),
        }
    }
}

/// A row of a read with the name of its first field, the view's key.
struct Row {
    first: Option<String>,
    map: Map<String, Value>,
}

impl<'de> Deserialize<'de> for Row {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Row, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Row;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a row object")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Row, M::Error> {
                let mut row = Row {
                    first: None,
                    map: Map::new(),
                };
                while let Some((k, v)) = m.next_entry::<String, Value>()? {
                    if row.first.is_none() {
                        row.first = Some(k.clone());
                    }
                    row.map.insert(k, v);
                }
                Ok(row)
            }
        }
        d.deserialize_map(V)
    }
}

impl Row {
    /// `field=value` of the key.
    fn key(&self) -> String {
        match &self.first {
            Some(k) => format!("{k}={}", text(&self.map[k])),
            None => "an empty row".into(),
        }
    }

    fn key_value(&self) -> String {
        self.first
            .as_ref()
            .map(|k| text(&self.map[k]))
            .unwrap_or_default()
    }
}

fn text(v: &Value) -> String {
    v.as_str().map_or_else(|| v.to_string(), String::from)
}

#[derive(Deserialize)]
struct Rows {
    rows: Vec<Row>,
}

struct Suite<'a> {
    root: &'a str,
    plan: &'a Plan,
    token: Option<&'a str>,
    options: Options,
    checks: Vec<Check>,
}

fn query(params: &BTreeMap<String, String>) -> String {
    if params.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", wire::percent_encode(k), wire::percent_encode(v)))
        .collect();
    format!("?{}", parts.join("&"))
}

impl Suite<'_> {
    fn push(&mut self, name: String, status: Status, detail: impl Into<String>) {
        self.checks.push(Check {
            name,
            status,
            detail: detail.into(),
        });
    }

    fn url(&self, part: Option<&str>, instance: Option<u32>) -> String {
        interface::scope_url(self.root, &self.plan.architecture_id, part, instance)
    }

    fn scope_name(&self, part: Option<&str>, instance: Option<u32>) -> String {
        interface::scope_path(&self.plan.architecture_id, part, instance)
    }

    fn auth(&self) -> Option<String> {
        self.token.map(|t| format!("Bearer {t}"))
    }

    fn get(&self, url: &str, authed: bool) -> Result<wire::Response, String> {
        let auth = self.auth();
        let headers: Vec<(&str, &str)> = match (&auth, authed) {
            (Some(a), true) => vec![("Authorization", a.as_str())],
            _ => vec![],
        };
        wire::request("GET", url, &headers, None, self.options.timeout)
            .map_err(|e| format!("GET {url}: {e}"))
    }

    fn post(&self, url: &str, body: &Value, authed: bool) -> Result<wire::Response, String> {
        let auth = self.auth();
        let mut headers: Vec<(&str, &str)> = vec![("Content-Type", "application/json")];
        if let (Some(a), true) = (&auth, authed) {
            headers.push(("Authorization", a.as_str()));
        }
        let text = body.to_string();
        wire::request(
            "POST",
            url,
            &headers,
            Some(text.as_bytes()),
            self.options.timeout,
        )
        .map_err(|e| format!("POST {url}: {e}"))
    }

    /// `{"rows": [...]}` of a view at a scope.
    fn rows(
        &self,
        view: &str,
        part: Option<&str>,
        instance: Option<u32>,
    ) -> Result<Vec<Row>, String> {
        let url = format!("{}/api/views/{view}", self.url(part, instance));
        let r = self.get(&url, true)?;
        if r.status != 200 {
            return Err(format!("GET {url} answered {}: {}", r.status, r.text()));
        }
        serde_json::from_slice::<Rows>(&r.body)
            .map(|b| b.rows)
            .map_err(|e| format!("GET {url} is not {{\"rows\": [...]}}: {e}"))
    }

    /// The entity ids placed to a scope, read from `Placements` at the architecture.
    fn placed(
        &self,
        part: Option<&str>,
        instance: Option<u32>,
    ) -> Result<BTreeSet<String>, String> {
        let rows = self.rows("agentic.architecture.Placements", None, None)?;
        Ok(rows
            .iter()
            .filter(|r| {
                part.is_none_or(|p| r.map.get("part_path").and_then(Value::as_str) == Some(p))
                    && instance.is_none_or(|i| {
                        r.map.get("instance").and_then(Value::as_u64) == Some(u64::from(i))
                    })
            })
            .filter_map(|r| {
                r.map
                    .get("entity_id")
                    .and_then(Value::as_str)
                    .map(String::from)
            })
            .collect())
    }

    fn unauthenticated(&mut self) {
        let Some(first) = self.plan.scopes.first() else {
            return;
        };
        let (part, instance) = (Some(first.part.as_str()), first.instance);
        let scope = self.scope_name(part, instance);
        let base = self.url(part, instance);
        let judge = |r: Result<wire::Response, String>| -> (Status, String) {
            match r {
                Err(e) => (Status::Failed, e),
                Ok(r) if r.status != 401 => (
                    Status::Failed,
                    format!("answered {} without credentials, not 401", r.status),
                ),
                Ok(r) => {
                    let body: Value = serde_json::from_slice(&r.body).unwrap_or(Value::Null);
                    if body.get("rows").is_some() || body.get("row").is_some() {
                        (Status::Failed, "answered 401 with a row".into())
                    } else {
                        (Status::Passed, "401, no row".into())
                    }
                }
            }
        };
        let view = first
            .views
            .first()
            .cloned()
            .unwrap_or_else(|| "agentic.architecture.Parts".into());
        let (s, d) = judge(self.get(&format!("{base}/api/views/{view}"), false));
        self.push(format!("unauthenticated R1 {view} at {scope}"), s, d);
        if let Some(feed) = &self.plan.feed {
            let url = format!("{base}/api/channels/{}", feed.channel);
            let (s, d) = match wire::subscribe(&url, &[], self.options.timeout) {
                Err(e) => (Status::Failed, format!("GET {url}: {e}")),
                Ok(st) => {
                    let got = st.next(Duration::from_millis(100));
                    if st.status != 401 {
                        (
                            Status::Failed,
                            format!("answered {} without credentials, not 401", st.status),
                        )
                    } else if got.is_some() {
                        (
                            Status::Failed,
                            "streamed an event without credentials".into(),
                        )
                    } else {
                        (Status::Passed, "401, no event".into())
                    }
                }
            };
            self.push(
                format!("unauthenticated R2 {} at {scope}", feed.channel),
                s,
                d,
            );
        }
        if let Some(c) = self.plan.commands.first() {
            let url = format!("{base}/api/commands/{}", c.command);
            let (s, d) = judge(self.post(&url, &c.input, false));
            self.push(format!("unauthenticated R4 {} at {scope}", c.command), s, d);
        }
        if self.plan.actions.is_some() {
            let (s, d) = judge(self.get(&format!("{base}/api/actions"), false));
            self.push(format!("unauthenticated R5 at {scope}"), s, d);
        }
    }

    fn scoped_reads(&mut self) {
        let arch = self.plan.architecture_id.clone();
        let all = match self.placed(None, None) {
            Ok(p) => p,
            Err(e) => {
                self.push(format!("R1 placements at {arch}"), Status::Failed, e);
                return;
            }
        };
        let mut seen_arch = BTreeSet::new();
        for sp in self.plan.scopes.clone() {
            let (part, instance) = (Some(sp.part.as_str()), sp.instance);
            let scope = self.scope_name(part, instance);
            let placed = match self.placed(part, instance) {
                Ok(p) => p,
                Err(e) => {
                    self.push(format!("R1 placements at {scope}"), Status::Failed, e);
                    continue;
                }
            };
            for view in &sp.views {
                let arch_rows = self.rows(view, None, None);
                if seen_arch.insert(view.clone()) {
                    let name = format!("R1 {view} at {arch}");
                    match &arch_rows {
                        Err(e) => self.push(name, Status::Failed, e.clone()),
                        Ok(rows) => match rows.iter().find(|r| !interface::in_scope(&r.map, &all)) {
                            Some(r) => self.push(
                                name,
                                Status::Failed,
                                format!(
                                    "{view} returned the row {}, which no placement in {arch} reaches",
                                    r.key()
                                ),
                            ),
                            None => self.push(name, Status::Passed, format!("{} in scope", count(rows.len(), "row"))),
                        },
                    }
                }
                let name = format!("R1 {view} at {scope}");
                let rows = match self.rows(view, part, instance) {
                    Ok(r) => r,
                    Err(e) => {
                        self.push(name, Status::Failed, e);
                        continue;
                    }
                };
                if let Some(r) = rows.iter().find(|r| !interface::in_scope(&r.map, &placed)) {
                    self.push(
                        name,
                        Status::Failed,
                        format!(
                            "{view} returned the row {}, which no placement to {scope} reaches",
                            r.key()
                        ),
                    );
                    continue;
                }
                if let Ok(arch_rows) = &arch_rows {
                    let got: BTreeSet<String> = rows.iter().map(Row::key_value).collect();
                    if let Some(r) = arch_rows
                        .iter()
                        .filter(|r| interface::in_scope(&r.map, &placed))
                        .find(|r| !got.contains(&r.key_value()))
                    {
                        self.push(
                            name,
                            Status::Failed,
                            format!(
                                "{view} left out the row {}, which a placement to {scope} reaches",
                                r.key()
                            ),
                        );
                        continue;
                    }
                }
                self.push(
                    name,
                    Status::Passed,
                    format!("{}, each in scope", count(rows.len(), "row")),
                );
            }
        }
    }

    fn placement_views(&mut self) {
        let mut scopes: Vec<(Option<String>, Option<u32>)> = vec![(None, None)];
        scopes.extend(
            self.plan
                .scopes
                .iter()
                .map(|s| (Some(s.part.clone()), s.instance)),
        );
        for (part, instance) in scopes {
            let scope = self.scope_name(part.as_deref(), instance);
            for view in interface::r3_views() {
                let name = format!("R3 {view} at {scope}");
                match self.rows(&view, part.as_deref(), instance) {
                    Ok(rows) => self.push(name, Status::Passed, count(rows.len(), "row")),
                    Err(e) => self.push(name, Status::Failed, e),
                }
            }
        }
    }

    fn collect(&self, stream: &wire::EventStream, n: usize, deadline: Instant) -> Vec<wire::Event> {
        let mut got = Vec::new();
        while got.len() < n {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            match stream.next(deadline - now) {
                Some(e) => got.push(e),
                None => break,
            }
        }
        got
    }

    fn feed(&mut self) {
        let Some(feed) = self.plan.feed.clone() else {
            return;
        };
        let (part, instance) = (feed.part.as_deref(), feed.instance);
        let scope = self.scope_name(part, instance);
        let name = format!("R2 resume {} at {scope}", feed.channel);
        let url = format!(
            "{}/api/channels/{}{}",
            self.url(part, instance),
            feed.channel,
            query(&feed.params)
        );
        let auth = self.auth();
        let mut headers: Vec<(&str, &str)> = Vec::new();
        if let Some(a) = &auth {
            headers.push(("Authorization", a.as_str()));
        }
        let wait = Duration::from_millis(feed.wait_ms);
        let first = match wire::subscribe(&url, &headers, self.options.timeout) {
            Err(e) => return self.push(name, Status::Failed, format!("GET {url}: {e}")),
            Ok(s) if s.status != 200 => {
                let d = format!(
                    "GET {url} answered {}: {}",
                    s.status,
                    String::from_utf8_lossy(&s.body)
                );
                return self.push(name, Status::Failed, d);
            }
            Ok(s) => {
                let got = self.collect(&s, feed.events, Instant::now() + wait);
                drop(s);
                got
            }
        };
        let k = feed.resume_after;
        if first.len() <= k || k == 0 {
            let d = format!(
                "the channel sent {} events within {} ms; the check needs more than {k}",
                first.len(),
                feed.wait_ms
            );
            return self.push(name, Status::Failed, d);
        }
        for (i, e) in first.iter().enumerate() {
            let bad = if e.id.as_deref().is_none_or(str::is_empty) {
                Some("has no id:")
            } else if !e.event.contains('.') {
                Some("is not named by a qualified event name")
            } else if serde_json::from_str::<Value>(&e.data).is_err() {
                Some("carries data that is not JSON")
            } else {
                None
            };
            if let Some(why) = bad {
                return self.push(
                    name,
                    Status::Failed,
                    format!("event {} ({}) {why}", i + 1, e.event),
                );
            }
        }
        if let Some(part) = part {
            let sname = format!("R2 scope {} at {scope}", feed.channel);
            match self.placed(Some(part), instance) {
                Err(e) => self.push(sname, Status::Failed, e),
                Ok(placed) => {
                    let out = first.iter().enumerate().find(|(_, e)| {
                        serde_json::from_str::<Value>(&e.data)
                            .ok()
                            .and_then(|v| v.as_object().cloned())
                            .is_none_or(|o| !interface::in_scope(&o, &placed))
                    });
                    match out {
                        Some((i, e)) => self.push(
                            sname,
                            Status::Failed,
                            format!(
                                "event {} ({}, id {}) is out of scope",
                                i + 1,
                                e.event,
                                e.id.clone().unwrap_or_default()
                            ),
                        ),
                        None => self.push(
                            sname,
                            Status::Passed,
                            format!("{}, each in scope", count(first.len(), "event")),
                        ),
                    }
                }
            }
        }
        let cursor = first[k - 1].id.clone().unwrap_or_default();
        let header = interface::cursor_header();
        let mut again_headers = headers.clone();
        again_headers.push((header.as_str(), cursor.as_str()));
        let want = &first[k..];
        let again = match wire::subscribe(&url, &again_headers, self.options.timeout) {
            Err(e) => return self.push(name, Status::Failed, format!("GET {url}: {e}")),
            Ok(s) if s.status != 200 => {
                let d = format!(
                    "a reconnect answered {}: {}",
                    s.status,
                    String::from_utf8_lossy(&s.body)
                );
                return self.push(name, Status::Failed, d);
            }
            Ok(s) => {
                let mut got = self.collect(&s, want.len(), Instant::now() + wait);
                // Anything that follows at once is judged too: a resent event comes first.
                while let Some(e) = s.next(self.options.idle) {
                    got.push(e);
                    if got.len() > want.len() + 8 {
                        break;
                    }
                }
                got
            }
        };
        match judge_resume(&first, k, &again) {
            Ok(d) => self.push(name, Status::Passed, d),
            Err(d) => self.push(name, Status::Failed, d),
        }
    }

    fn commands(&mut self) {
        for c in self.plan.commands.clone() {
            let (part, instance) = (c.part.as_deref(), c.instance);
            let scope = self.scope_name(part, instance);
            let name = format!("R4 {} at {scope}", c.command);
            let url = format!("{}/api/commands/{}", self.url(part, instance), c.command);
            let r = match self.post(&url, &c.input, true) {
                Ok(r) => r,
                Err(e) => {
                    self.push(name, Status::Failed, e);
                    continue;
                }
            };
            let body: Value = serde_json::from_slice(&r.body).unwrap_or(Value::Null);
            let (s, d) = match c.expect {
                Expect::Accepted => match (r.status, body.get("row")) {
                    (200, Some(Value::Object(_) | Value::Null)) => {
                        (Status::Passed, "200 {\"row\"}".to_string())
                    }
                    (200, _) => (
                        Status::Failed,
                        format!("200 without {{\"row\"}}: {}", r.text()),
                    ),
                    (s, _) => (
                        Status::Failed,
                        format!("answered {s}, not 200: {}", r.text()),
                    ),
                },
                Expect::Refused => refusal(&r, &body, c.error.as_deref()),
            };
            self.push(name, s, d);
        }
    }

    fn actions(&mut self) {
        let Some(ap) = self.plan.actions.clone() else {
            return;
        };
        for sp in self.plan.scopes.clone() {
            let (part, instance) = (Some(sp.part.as_str()), sp.instance);
            let scope = self.scope_name(part, instance);
            let name = format!("R5 at {scope}");
            let url = format!("{}/api/actions", self.url(part, instance));
            let r = match self.get(&url, true) {
                Ok(r) => r,
                Err(e) => {
                    self.push(name, Status::Failed, e);
                    continue;
                }
            };
            if r.status == 404 {
                self.push(name, Status::Skipped, "the source has no R5 (404)");
                continue;
            }
            if r.status != 200 {
                self.push(
                    name,
                    Status::Failed,
                    format!("answered {}: {}", r.status, r.text()),
                );
                continue;
            }
            let list = match parse_actions(&r.body) {
                Ok(l) => l,
                Err(e) => {
                    self.push(name, Status::Failed, e);
                    continue;
                }
            };
            let covering = |cmd: &str| -> Vec<&Grant> {
                let class = ap.action_classes.get(cmd);
                let all: Vec<&Grant> = ap
                    .grants
                    .iter()
                    .filter(|g| {
                        g.grantee_id == ap.principal_id
                            && g.state == "Active"
                            && Some(&g.action_class) == class
                            && interface::covers(&g.scope, &scope)
                    })
                    .collect();
                if all.iter().any(|g| g.effect == "Deny") {
                    return Vec::new();
                }
                all.into_iter()
                    .filter(|g| g.effect == "Allow" || g.effect == "ApprovalRequired")
                    .collect()
            };
            // The faults by weight: an allowance no grant covers, a wrong grant, a withheld one.
            let mut faults: [Option<String>; 3] = [None, None, None];
            for (cmd, allowed, grant) in &list {
                if !ap.action_classes.contains_key(cmd) {
                    continue;
                }
                let cov = covering(cmd);
                let (i, d) = if *allowed && cov.is_empty() {
                    (
                        0,
                        format!(
                            "R5 allows {cmd} at {scope}, which no grant of the principal covers"
                        ),
                    )
                } else if *allowed && !cov.iter().any(|g| Some(&g.grant_id) == grant.as_ref()) {
                    let g = grant.clone().unwrap_or_else(|| "(none named)".into());
                    (
                        1,
                        format!("R5 allows {cmd} at {scope} by grant {g}, which does not cover it"),
                    )
                } else if !*allowed && !cov.is_empty() {
                    (
                        2,
                        format!(
                            "R5 withholds {cmd} at {scope}, which grant {} covers",
                            cov[0].grant_id
                        ),
                    )
                } else {
                    continue;
                };
                faults[i].get_or_insert(d);
            }
            let mut fault = faults.into_iter().flatten().next();
            if fault.is_none() {
                for cmd in ap.action_classes.keys() {
                    if !list.iter().any(|(c, _, _)| c == cmd) && !covering(cmd).is_empty() {
                        fault = Some(format!(
                            "R5 does not list {cmd} at {scope}, which a grant of the principal covers"
                        ));
                        break;
                    }
                }
            }
            match fault {
                Some(d) => self.push(name, Status::Failed, d),
                None => {
                    let n = list.iter().filter(|(_, a, _)| *a).count();
                    self.push(
                        name,
                        Status::Passed,
                        format!(
                            "{n} of {} commands allowed, each by a grant that covers it",
                            list.len()
                        ),
                    );
                }
            }
            for (cmd, input) in &ap.probes {
                let allowed = list.iter().any(|(c, a, _)| c == cmd && *a);
                if allowed {
                    continue;
                }
                let hname = format!("R5 hidden {cmd} at {scope}");
                let url = format!("{}/api/commands/{cmd}", self.url(part, instance));
                match self.post(&url, input, true) {
                    Err(e) => self.push(hname, Status::Failed, e),
                    Ok(r) => {
                        let body: Value = serde_json::from_slice(&r.body).unwrap_or(Value::Null);
                        let (s, d) = refusal(&r, &body, None);
                        self.push(hname, s, d);
                    }
                }
            }
        }
    }
}

/// The obligation of R2's cursor: `first` is what a connection received (events with a cursor),
/// `again` what a reconnect with the cursor of event `k` (counted from 1) received. It holds when
/// `again` repeats no event at or before that cursor and starts with exactly the events of
/// `first` after it, in order (by cursor, name and data). Events `again` holds beyond those are
/// newer and not judged. `Ok` says what held; `Err` names the event that broke it.
pub fn judge_resume(
    first: &[wire::Event],
    k: usize,
    again: &[wire::Event],
) -> Result<String, String> {
    if k == 0 || first.len() <= k {
        return Err(format!(
            "{} events received; the check needs more than {k}",
            first.len()
        ));
    }
    let cursor = first[k - 1].id.clone().unwrap_or_default();
    let before: BTreeSet<&str> = first[..k].iter().filter_map(|e| e.id.as_deref()).collect();
    if let Some(e) = again
        .iter()
        .find(|e| e.id.as_deref().is_some_and(|id| before.contains(id)))
    {
        return Err(format!(
            "after a reconnect from the cursor of event {k} ({cursor}) the source sent event {} again ({}, id {})",
            first.iter().position(|f| f.id == e.id).map_or(0, |p| p + 1),
            e.event,
            e.id.clone().unwrap_or_default()
        ));
    }
    for (i, w) in first[k..].iter().enumerate() {
        let n = k + i + 1;
        match again.get(i) {
            None => {
                return Err(format!(
                    "after a reconnect from the cursor of event {k} event {n} ({}) did not come",
                    w.event
                ));
            }
            Some(g) if g.id != w.id || g.event != w.event || !same_json(&g.data, &w.data) => {
                return Err(format!(
                    "after a reconnect from the cursor of event {k} the source sent {} (id {}) where event {n} {} (id {}) followed",
                    g.event,
                    g.id.clone().unwrap_or_default(),
                    w.event,
                    w.id.clone().unwrap_or_default()
                ));
            }
            Some(_) => {}
        }
    }
    Ok(format!(
        "events {}..{} after the cursor of event {k}, in order",
        k + 1,
        first.len()
    ))
}

/// `1 row`, `2 rows`.
fn count(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn same_json(a: &str, b: &str) -> bool {
    match (
        serde_json::from_str::<Value>(a),
        serde_json::from_str::<Value>(b),
    ) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

/// Whether an answer is a refusal: 422 with `{"row": {"error": …}}` (naming `error` when given).
fn refusal(r: &wire::Response, body: &Value, error: Option<&str>) -> (Status, String) {
    if r.status != 422 {
        return (
            Status::Failed,
            format!("answered {}, not 422: {}", r.status, r.text()),
        );
    }
    match body["row"]["error"].as_str() {
        None => (
            Status::Failed,
            format!("422 without the refusal's row: {}", r.text()),
        ),
        Some(e) if error.is_some_and(|w| w != e) => (
            Status::Failed,
            format!("refused with {e}, not {}", error.unwrap_or_default()),
        ),
        Some(e) => (Status::Passed, format!("422, refused with {e}")),
    }
}

/// `[{command, allowed, grant_id?}]`.
fn parse_actions(body: &[u8]) -> Result<Vec<(String, bool, Option<String>)>, String> {
    let v: Value = serde_json::from_slice(body).map_err(|e| format!("R5 is not JSON: {e}"))?;
    let arr = v
        .as_array()
        .ok_or_else(|| "R5 is not an array of {command, allowed, grant_id?}".to_string())?;
    arr.iter()
        .map(|e| {
            let cmd = e["command"].as_str();
            let allowed = e["allowed"].as_bool();
            match (cmd, allowed) {
                (Some(c), Some(a)) => {
                    Ok((c.to_string(), a, e["grant_id"].as_str().map(String::from)))
                }
                _ => Err(format!(
                    "R5 holds an entry without command and allowed: {e}"
                )),
            }
        })
        .collect()
}

/// Runs every check of `plan` against the source at `root` (`http://host:port`, scopes below
/// `/scopes/`), with the bearer `token`.
pub fn run(root: &str, plan: &Plan, token: Option<&str>, options: Options) -> Report {
    let mut s = Suite {
        root,
        plan,
        token,
        options,
        checks: Vec::new(),
    };
    s.unauthenticated();
    s.scoped_reads();
    s.placement_views();
    s.feed();
    s.commands();
    s.actions();
    Report {
        source: root.to_string(),
        checks: s.checks,
    }
}
