//! The `ess-cli/2` binding of a `cli` export: one command per command and per view of the
//! interface, at the path `<domain> <wire name>`, each forwarded to the component that serves
//! the blocks (`service_forward`), with one `--option` per input field or view parameter. It maps
//! no error codes: a callable's error is a type of the model, and an ESS error is none.
//!
//! The binding names each operation's input as `<command>.Input` or `<view>.Params`, a view's
//! rows as `List<<view>.Row>` and a command's answer as `<command>.Outcome`. `ess` 0.99.2 declares
//! none of these types, and a `service_forward` callable cannot name a command's input or a view's
//! parameters and rows in their place (`design/ess-repro/cli-service-forward/`), so `ess specify
//! cli` refuses the binding until ESS can; it is written, not validated.

use super::{
    Contract, ContractIngress, OpKind, declared, described, domain_short, header_of, kebab, short,
};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// The format of the binding.
pub const FORMAT: &str = "ess-cli/2";

/// The `ess-cli/2` binding of an ingress's `cli` export. `ir` is the compiled blocks and
/// `component` the component that serves them. Refused, naming every one, when the blocks lack
/// a command or view of the interface, and when two operations take the same command path.
pub fn cli_binding(
    contract: &Contract,
    ingress: &ContractIngress,
    ir: &Value,
    component: &str,
) -> Result<String, String> {
    let mut callables = Map::new();
    let mut commands: Vec<Vec<(&str, Value)>> = Vec::new();
    let mut paths: BTreeMap<String, String> = BTreeMap::new();
    let mut missing = Vec::new();
    for o in ingress
        .operations
        .iter()
        .filter(|o| matches!(o.kind, OpKind::Command | OpKind::View))
    {
        let Some(entry) = declared(ir, o.kind, &o.name) else {
            missing.push(o.name.as_str());
            continue;
        };
        let wire = kebab(entry["naming"]["wire"].as_str().unwrap_or(short(&o.name)));
        let domain = domain_short(&o.name);
        let path = format!("{domain} {wire}");
        if let Some(prev) = paths.insert(path.clone(), o.name.clone()) {
            return Err(format!(
                "ingress {}: the command path `{path}` is the path of both {prev} and {}",
                ingress.name, o.name
            ));
        }
        let id = format!("{domain}_{}", wire.replace('-', "_"));
        let fields = match o.kind {
            OpKind::Command => &entry["input"],
            _ => &entry["params"],
        };
        let names: Vec<&str> = fields
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|f| f["name"].as_str())
            .collect();
        let input = match (o.kind, names.is_empty()) {
            (_, true) => Value::Null,
            (OpKind::Command, false) => json!(format!("{}.Input", o.name)),
            _ => json!(format!("{}.Params", o.name)),
        };
        let mut callable = Map::new();
        callable.insert(
            "target".into(),
            json!({"kind": "service_forward", "owner": component, "operation": o.name}),
        );
        callable.insert("input".into(), input);
        if o.kind == OpKind::Command {
            callable.insert("result".into(), json!(format!("{}.Outcome", o.name)));
        } else {
            callable.insert("result".into(), json!(format!("List<{}.Row>", o.name)));
        }
        callables.insert(id.clone(), Value::Object(callable));
        let arguments: Vec<Value> = names
            .iter()
            .map(|n| json!({"field": n, "source": {"kind": "option", "long": kebab(n)}}))
            .collect();
        commands.push(vec![
            ("path", json!([domain, wire])),
            ("callable", json!(id)),
            ("about", json!(described(entry, &o.name))),
            ("arguments", Value::Array(arguments)),
        ]);
    }
    if !missing.is_empty() {
        return Err(format!(
            "ingress {}: the compiled blocks have no operation {}",
            ingress.name,
            missing.join(", ")
        ));
    }
    let mut s = header_of(contract, ingress, "cli", "The ess-cli/2 binding");
    s.push_str(
        "# Each command is forwarded to the component that serves the blocks. ess 0.99.2 cannot yet\n\
         # resolve the input, row and outcome types it names (design/ess-repro/cli-service-forward).\n",
    );
    s.push_str(&format!("format: {FORMAT}\n"));
    s.push_str(&super::field(
        "binary",
        &json!(format!("{}-{}", contract.architecture, ingress.name)),
    ));
    s.push_str(&super::field(
        "about",
        &json!(format!(
            "The {} ingress of architecture {}",
            ingress.name, contract.architecture
        )),
    ));
    s.push_str("globals:\n  state: state-dir\n");
    s.push_str(&super::field("callables", &Value::Object(callables)));
    s.push_str(&super::list("commands", &commands));
    Ok(s)
}
