//! The UI generator of agenticspec: an `ess-ui/1` document to Vue 3 single-file components and
//! TypeScript over the components of `ui/kit`.
//!
//! - `doc`: the constructs of `ess-ui/1` the generator draws, typed from the schema; every struct
//!   refuses a key it does not declare.
//! - `check`: parsing with refusals by node path, merging each page over its kind, resolving
//!   references and compiling expressions into the model the emitter writes.
//! - `emit`: the files: one page component per page, `App.vue`, the router, the typed client of
//!   the reads, commands and channels, the host slots' names and props, and the runtime.
//! - `interface`: the runtime interface the generated client calls, `ui/runtime.openapi.yaml`.
//! - `source`: the fixture source, an adapter that answers that interface from fixtures.
//! - `conform`: the conformance suite of that interface, run against a source by its URL.
//! - `wire`: the small HTTP client and server both use.
//! - `operator`: `agenticspec ui build`, the embedded operator app with the data files of one
//!   architecture, and their reproducible archive.
//! - `plane`: the default control plane of an architecture as that interface sees it, read from
//!   the files of its operator UI.
//! - `serve`: `agenticspec ui serve`, the operator app and that plane answered from fixtures,
//!   and the UI of every other ingress with a `ui` export at its own root.
//!
//! The output is a function of the document alone: the same document gives the same bytes.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

mod check;
pub mod conform;
pub mod derive;
pub mod interface;
pub mod operator;
pub mod plane;
pub mod serve;
pub mod source;
pub mod wire;
// The document types hold every property the schema gives a construct the generator accepts; some
// it reads only to accept them (the model, the fixtures, a channel's delivery).
#[allow(dead_code)]
mod doc;
mod emit;

/// A construct the generator refuses, named by its node path (`pages/draft/sections/patch`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub path: String,
    pub message: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = if self.path.is_empty() {
            "/"
        } else {
            &self.path
        };
        write!(f, "{path}: {}", self.message)
    }
}

/// The generated files by their path below the output directory.
pub type Files = BTreeMap<String, String>;

/// The files of the app the document describes, or every construct it refuses.
pub fn generate(text: &str) -> Result<Files, Vec<Refusal>> {
    let (doc, _raw) = check::parse(text)?;
    let app = check::resolve(&doc)?;
    Ok(emit::emit(&app))
}

/// The compiled blocks a derivation of `inputs` reads: the blocks under `inputs.root` compiled,
/// or without a root the part of them built into the binary.
fn derive_ir(inputs: &derive::Inputs) -> Result<Cow<'static, serde_json::Value>, derive::Failure> {
    match &inputs.root {
        Some(root) => agenticspec_views::compile(&root.join("blocks"))
            .map(Cow::Owned)
            .map_err(derive::Failure::Unreadable),
        None => operator::built_in_model().map(Cow::Borrowed),
    }
}

/// `agenticspec ui derive`: compiles the blocks under `inputs.root` (without a root, reads the
/// part of them built into the binary) and derives the composition of the architecture (see
/// [`derive`]).
///
/// # Errors
///
/// The blocks do not compile, an input cannot be read, or the architecture or its `view:` key is
/// refused.
pub fn derive_ui(inputs: &derive::Inputs) -> Result<derive::Derivation, derive::Failure> {
    derive::derive_with(inputs, derive_ir(inputs)?.as_ref())
}

/// `agenticspec ui derive` of every UI of the architecture: compiles the blocks under
/// `inputs.root` once (without a root, reads the part of them built into the binary) and derives
/// one composition per ingress with a `ui` export when the contract gives a UI per ingress, else
/// the one operator UI (see [`derive::derive_all`]).
///
/// # Errors
///
/// As [`derive_ui`], and an ingress with no `ui` export.
pub fn derive_uis(
    inputs: &derive::Inputs,
) -> Result<Vec<(Option<String>, derive::Derivation)>, derive::Failure> {
    derive::derive_all(inputs, derive_ir(inputs)?.as_ref())
}

/// Replaces `out` with exactly `files`: what was in it before is removed.
pub fn write(files: &Files, out: &Path) -> std::io::Result<()> {
    if out.exists() {
        std::fs::remove_dir_all(out)?;
    }
    for (name, text) in files {
        let path = out.join(name);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, text)?;
    }
    Ok(())
}
