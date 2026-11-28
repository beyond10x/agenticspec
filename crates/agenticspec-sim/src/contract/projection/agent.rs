//! The agent card of an `agent` export, for an agent-to-agent protocol: the architecture's name,
//! one skill per command of the interface with its input schema and the ESS errors it refuses
//! with, and the events of its feeds as the updates of a conversation. The address is left open:
//! the runtime binding gives it, and the work inputs travel over the audience's channel part (the
//! check refuses an `agent` export without one, `ingress_agent_without_channel`).

use super::tool::entry;
use super::{
    Contract, ContractIngress, OpKind, Pin, callables, declared, header_of, ingress_text, list,
    pin_text,
};
use serde_json::{Value, json};

/// The format of an agent card.
pub const FORMAT: &str = "agenticspec-agent-card/1";

/// The agent card of an ingress's `agent` export. `ir` is the compiled blocks, `full` ESS's
/// OpenAPI document of the component that serves them. Refused when the document lacks a
/// command of the interface, when two commands have the same wire name, and when the blocks
/// lack an event of a feed.
pub fn agent_card(
    contract: &Contract,
    ingress: &ContractIngress,
    pin: &Pin,
    ir: &Value,
    full: &Value,
) -> Result<String, String> {
    let skills = callables(ingress, &[OpKind::Command], ir, full, "skill")?;
    let updates: Vec<&str> = ingress
        .operations
        .iter()
        .filter(|o| o.kind == OpKind::Feed)
        .map(|o| o.name.as_str())
        .collect();
    let missing: Vec<&str> = updates
        .iter()
        .copied()
        .filter(|e| declared(ir, OpKind::Feed, e).is_none())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "ingress {}: the compiled blocks have no event {}",
            ingress.name,
            missing.join(", ")
        ));
    }
    let mut s = header_of(contract, ingress, "agent", "The agent card");
    s.push_str(
        "# One skill per command; the events of its feeds are the updates of a conversation.\n\
         # The address is open: the runtime binding gives it.\n",
    );
    s.push_str(&format!("format: {FORMAT}\n"));
    s.push_str(&pin_text(pin));
    s.push_str(&super::field("name", &json!(contract.architecture)));
    s.push_str(&super::field(
        "description",
        &json!(format!(
            "The {} ingress of architecture {}.",
            ingress.name, contract.architecture
        )),
    ));
    s.push_str(&ingress_text(contract, ingress));
    s.push_str(&super::field(
        "capabilities",
        &json!({"streaming": !updates.is_empty()}),
    ));
    s.push_str("defaultInputModes: [application/json]\n");
    s.push_str("defaultOutputModes: [application/json]\n");
    let items: Vec<Vec<(&str, Value)>> = skills
        .iter()
        .map(|c| {
            let mut item = entry(c, "id");
            item.insert(1, ("name", json!(c.display)));
            item.push(("inputSchema", c.input.clone()));
            item
        })
        .collect();
    s.push_str(&list("skills", &items));
    s.push_str(&super::field("updates", &json!(updates)));
    Ok(s)
}
