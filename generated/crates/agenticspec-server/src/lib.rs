// generated from agenticspec v1
// contract digest 8764a7e4a2c954cdf68fec3a1a5ecaa5de423d1349bbdcf9c033ff55b691f9c2
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
