//! The stream list of an `event_stream` export: one stream per architecture level, the `ess-ui/1`
//! channel shape. It carries every event of the interface's feeds from server to client, each
//! event once (`every_event`), resumed from the last cursor seen; a cursor older than the log's
//! retention is refused by name (`CursorExpired`), never skipped. Its scope is a parameter (the
//! part path a subscriber reads), and a session is per architecture instance.

use super::{Contract, ContractIngress, OpKind, Pin, declared, header_of, ingress_text, pin_text};
use serde_json::{Value, json};

/// The format of a stream list.
pub const FORMAT: &str = "agenticspec-streams/1";

/// The stream list of an ingress's `event_stream` export. `ir` is the compiled blocks. Refused,
/// naming every one, when the blocks lack an event of a feed.
pub fn streams(
    contract: &Contract,
    ingress: &ContractIngress,
    pin: &Pin,
    ir: &Value,
) -> Result<String, String> {
    let carries: Vec<&str> = ingress
        .operations
        .iter()
        .filter(|o| o.kind == OpKind::Feed)
        .map(|o| o.name.as_str())
        .collect();
    let missing: Vec<&str> = carries
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
    let mut s = header_of(contract, ingress, "event_stream", "The stream list");
    s.push_str(
        "# One stream per architecture level, resumed from the last cursor seen; a cursor older\n\
         # than the retention is refused as `CursorExpired`.\n",
    );
    s.push_str(&format!("format: {FORMAT}\n"));
    s.push_str(&pin_text(pin));
    s.push_str(&ingress_text(contract, ingress));
    s.push_str(&super::list(
        "streams",
        &[vec![
            ("name", json!(contract.architecture)),
            ("direction", json!("server_to_client")),
            ("delivery", json!("every_event")),
            ("resume", json!("from_last_seen")),
            ("expired", json!("CursorExpired")),
            ("scope", json!("param")),
            ("session", json!({"per": "architecture_id"})),
            ("carries", json!(carries)),
        ]],
    ));
    Ok(s)
}
