// generated from agenticspec v1
// contract digest 1c85283ae85dcd279c75a78ac6655a0487a7fc753ad961048719d83148ee6045
// do not edit: regenerate with `ess synthesize`

//! The HTTP surface of `agenticspec` v1, synthesised.
//!
//! One module per component the specification declares is reached over a network, each holding
//! that component's route table, its listener and the two documents it publishes about itself.
//! The routes are the ones the committed `OpenAPI` document declares, from the same mapping, so a
//! path served here and a path published there cannot be two different answers.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent is absent by
//! decision — no framework, no runtime, no second protocol, no concurrency, no authentication —
//! and each absence is argued in the `TARGET.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod entry;
pub mod http;
pub mod json;
pub mod wire;
pub mod session_ir;
pub mod session_view;
pub mod memory;
mod static_assets;
