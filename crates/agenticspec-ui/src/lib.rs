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
//!
//! The output is a function of the document alone: the same document gives the same bytes.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

mod check;
pub mod conform;
pub mod interface;
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
