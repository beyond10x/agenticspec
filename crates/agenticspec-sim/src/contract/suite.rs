//! The conformance suite of the building blocks cut to one contract: the scenarios a runtime of
//! the architecture must pass, which are those about a command of a port the contract lists.
//! `docs/contract.md` states the rule.
//!
//! A scenario is about the command its id names: `<command>/outcome/<o>`,
//! `<entity>/transition/<t>/by/<command>/<o>`, `<entity>/state/<s>/refuses/<command>`,
//! `<entity>/invariant/after/<command>/<o>`, or for `<binding>/binding/<case>` the command the
//! binding invokes. An id that names no command (an authored scenario, an invariant at a view) is
//! about the command of the scenario's last `execute_command` step. Each scenario carries its own
//! setup steps, so a kept scenario keeps them.

use super::derive::Contract;
use super::ports::{Port, PortsFile};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// The prefix of the suite version of a conformance suite ESS writes.
pub const SUITE_FORMAT: &str = "ess-conformance/";

/// The domain of a port's commands: the domain of its intent event, waiting state or first
/// reporting command.
fn domain(port: &Port) -> Option<String> {
    let named = port
        .intent_event
        .as_deref()
        .or(port.waiting_state.as_deref())
        .or(port.reports.first().map(String::as_str))?;
    let mut parts: Vec<&str> = named.split('.').collect();
    // A waiting state names `<domain>.<Entity>.<State>`; an event or command `<domain>.<Name>`.
    let drop = if port.intent_event.is_none() && port.waiting_state.is_some() {
        2
    } else {
        1
    };
    if parts.len() <= drop {
        return None;
    }
    parts.truncate(parts.len() - drop);
    Some(parts.join("."))
}

/// The commands of every port the contract lists, by their qualified names.
pub fn commands(contract: &Contract, ports: &PortsFile) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for used in &contract.ports {
        let Some(port) = ports.port(&used.port) else {
            continue;
        };
        let Some(domain) = domain(port) else {
            continue;
        };
        out.extend(port.commands().iter().map(|c| format!("{domain}.{c}")));
        out.extend(port.reports.iter().cloned());
    }
    out
}

/// The command of a scenario's last `execute_command` step.
fn last_command(scenario: &Value) -> Option<String> {
    scenario["steps"]
        .as_array()?
        .iter()
        .rev()
        .find(|s| s["step"] == "execute_command")
        .and_then(|s| s["command"].as_str())
        .map(str::to_owned)
}

/// The command a scenario is about (see the module text), or none when it names none and
/// executes none.
pub fn about(id: &str, scenario: &Value, bindings: &BTreeMap<String, String>) -> Option<String> {
    let segs: Vec<&str> = id.split('/').collect();
    match segs.as_slice() {
        [binding, "binding", ..] => bindings.get(*binding).cloned(),
        [command, "outcome", ..] => Some((*command).to_owned()),
        [_, "transition", _, "by", command, ..]
        | [_, "state", _, "refuses", command, ..]
        | [_, "invariant" | "invariant-wrapped", "after", command, ..] => {
            Some((*command).to_owned())
        }
        _ => last_command(scenario),
    }
}

/// A suite cut to a contract.
#[derive(Debug, Clone)]
pub struct Cut {
    /// The cut suite: the whole suite's document with only the kept scenarios.
    pub suite: Value,
    /// The ids of the kept scenarios, in the suite's order.
    pub kept: Vec<String>,
    /// The number of scenarios of the whole suite.
    pub total: usize,
}

/// Keep the scenarios of `suite` about one of `commands`, with everything else of the document
/// (provenance, events) unchanged. Refused when the document is no conformance suite.
pub fn cut(
    suite: &Value,
    commands: &BTreeSet<String>,
    bindings: &BTreeMap<String, String>,
) -> Result<Cut, String> {
    let version = suite["provenance"]["suite_version"].as_str().unwrap_or("");
    if !version.starts_with(SUITE_FORMAT) {
        return Err(format!(
            "not a conformance suite: provenance.suite_version is `{version}`, not {SUITE_FORMAT}<N>"
        ));
    }
    let scenarios = suite["scenarios"]
        .as_object()
        .ok_or("not a conformance suite: no `scenarios` object")?;
    let mut kept_map = Map::new();
    let mut kept = Vec::new();
    for (id, scenario) in scenarios {
        if about(id, scenario, bindings).is_some_and(|c| commands.contains(&c)) {
            kept.push(id.clone());
            kept_map.insert(id.clone(), scenario.clone());
        }
    }
    let mut out = suite.clone();
    out["scenarios"] = Value::Object(kept_map);
    Ok(Cut {
        suite: out,
        kept,
        total: scenarios.len(),
    })
}
