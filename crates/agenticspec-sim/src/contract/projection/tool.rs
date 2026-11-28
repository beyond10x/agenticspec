//! The tool list of a `tool` export, for a model-context tool protocol: one tool per command and
//! one read tool per view of the interface, each named by its ESS wire name, described by its
//! `naming.summary` (else its display name), with its input as one JSON Schema. A command's
//! refusal is a tool error naming the ESS error, so each command tool lists the errors its
//! outcomes refuse with.

use super::{
    Callable, Contract, ContractIngress, OpKind, Pin, callables, header_of, ingress_text, list,
    pin_text,
};
use serde_json::{Value, json};

/// The format of a tool list.
pub const FORMAT: &str = "agenticspec-tools/1";

/// One tool or skill of a callable, its keys in the order written.
pub(super) fn entry<'a>(c: &Callable, name_key: &'a str) -> Vec<(&'a str, Value)> {
    let mut item = vec![
        (name_key, json!(c.wire)),
        ("description", json!(c.description)),
        ("operation", json!(c.name)),
    ];
    if c.kind == OpKind::Command {
        item.push(("errors", json!(c.errors)));
    }
    item
}

/// The tool list of an ingress's `tool` export. `ir` is the compiled blocks (`ess specify
/// compile --format json`), `full` ESS's OpenAPI document of the component that serves them.
/// Refused when the document lacks an operation of the interface, and when two operations
/// have the same wire name, so no tool name is ambiguous.
pub fn tools(
    contract: &Contract,
    ingress: &ContractIngress,
    pin: &Pin,
    ir: &Value,
    full: &Value,
) -> Result<String, String> {
    let all = callables(ingress, &[OpKind::Command, OpKind::View], ir, full, "tool")?;
    let mut s = header_of(contract, ingress, "tool", "The tool list");
    s.push_str(
        "# One tool per command and one read tool per view, each named by its ESS wire name; a\n\
         # refused command answers a tool error naming one of its `errors`.\n",
    );
    s.push_str(&format!("format: {FORMAT}\n"));
    s.push_str(&pin_text(pin));
    s.push_str(&ingress_text(contract, ingress));
    let items: Vec<Vec<(&str, Value)>> = all
        .iter()
        .map(|c| {
            let mut item = entry(c, "name");
            item.push((
                "annotations",
                json!({"readOnlyHint": c.kind == OpKind::View}),
            ));
            item.push(("inputSchema", c.input.clone()));
            item
        })
        .collect();
    s.push_str(&list("tools", &items));
    Ok(s)
}
