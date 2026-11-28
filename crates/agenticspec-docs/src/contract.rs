//! The "Contract" section of an architecture page and a catalog entry: the ports the document's
//! parts use, from `agenticspec-sim`'s contract derivation over the committed `blocks/ports.json`
//! (no `ess` call), with the kind and block of each port, the routing invariants and the border.

use crate::md::{code, text};
use agenticspec_sim::check::check;
use agenticspec_sim::contract::derive::{self, Contract};
use agenticspec_sim::contract::ports::PortsFile;
use serde_json::Value;
use std::fmt::Write as _;
use std::path::Path;

/// A derived contract with the ports file it was derived from.
pub struct Shown {
    pub contract: Contract,
    pub ports: PortsFile,
}

/// The contract of a document, or why there is none.
pub fn derive(root: &Path, doc: &Value) -> Result<Shown, String> {
    let flat = agenticspec_sim::nest::flatten(doc);
    let doc = &flat;
    if !check(doc).is_empty() {
        return Err("the check refuses the architecture (see Check), so it has no contract".into());
    }
    let ports = PortsFile::load(&root.join("blocks/ports.json"))?;
    let contract = derive::derive(doc, &ports).map_err(|r| r.join("; "))?;
    Ok(Shown { contract, ports })
}

fn setting(name: &str, v: &Value) -> String {
    let value = match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter()
                .map(|x| x.as_str().map_or_else(|| x.to_string(), str::to_owned))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => other.to_string(),
    };
    code(&format!("{name}: {value}"))
}

/// The section, its download given as a Markdown link target (`file`) under the label `label`;
/// `up` is the way from the page up to `docs/`.
pub fn section(shown: &Result<Shown, String>, file: Option<(&str, &str)>, up: &str) -> String {
    let mut s = String::from("## Contract\n\n");
    let shown = match shown {
        Ok(x) => x,
        Err(why) => {
            let _ = writeln!(s, "No contract: {}.\n", text(why));
            return s;
        }
    };
    let c = &shown.contract;
    let _ = write!(
        s,
        "The ports a runtime implements to run this architecture: each port's protocol, the part that uses it with the settings that narrow it, and how many instances run. \
Derived from the document and the ports of the building blocks; [how]({up}contract.md)."
    );
    if let Some((label, target)) = file {
        let _ = write!(s, " The contract file: [{label}]({target}).");
    }
    s.push_str("\n\n| port | kind | block | protocol | part | settings | instances |\n|---|---|---|---|---|---|---|\n");
    for p in &c.ports {
        let (kind, block) = shown
            .ports
            .port(&p.port)
            .map_or(("?", "?"), |x| (x.kind.as_str(), x.block.as_str()));
        let settings: Vec<String> = p.settings.iter().map(|(k, v)| setting(k, v)).collect();
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} |",
            code(&p.port),
            code(kind),
            code(block),
            code(&p.protocol),
            code(&p.part),
            if settings.is_empty() {
                "none".to_owned()
            } else {
                settings.join(", ")
            },
            p.instances
        );
    }
    s.push('\n');
    if c.invariants.is_empty() {
        s.push_str("Routing invariants: none.\n\n");
    } else {
        s.push_str("Routing invariants, one per link whose value travels in a port:\n\n");
        for i in &c.invariants {
            let _ = writeln!(s, "- {}: {}", code(&i.link), code(&i.predicate));
        }
        s.push('\n');
    }
    let border: Vec<String> = c
        .border
        .iter()
        .map(|b| format!("**{}**", text(b)))
        .collect();
    if border.is_empty() {
        s.push_str("Border: none.\n\n");
    } else {
        let _ = writeln!(s, "Border: {}.\n", border.join(", "));
    }
    s
}
