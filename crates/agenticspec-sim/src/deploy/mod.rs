//! The deploy side of an architecture: the binding (`agenticspec-binding/1`) with its `packing:`
//! section (`binding`, `packing`), the admission of a packing into a plan of units or a list of
//! named refusals (`admit`), and the deployment bundle (`agenticspec-deployment/1`) a translator
//! reads (`bundle`); the fixture bundles of `bundles/` (`fixtures`). The formats are on
//! `docs/deployment.md`.
//!
//! Nothing here changes a simulation result: the catalog digest leaves this directory out.

pub mod admit;
pub mod binding;
pub mod bundle;
pub mod fixtures;
pub mod packing;

use serde_json::Value;
use sha2::{Digest, Sha256};

/// `sha256:<hex>` of some bytes.
pub fn sha256(bytes: &[u8]) -> String {
    let hex: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

/// The canonical JSON of a value: keys sorted, no space. `serde_json` keeps the keys of an
/// object sorted in this workspace (no `preserve_order`), so its compact form is canonical.
pub fn canonical(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// `sha256:<hex>` of the canonical JSON of a value.
pub fn digest(v: &Value) -> String {
    sha256(canonical(v).as_bytes())
}
