// generated from agenticspec v1
// contract digest b3a8e549462bc9e05e03c46df5841f20a02fcb1fe0d494b44b19fdeffb576120
// do not edit: regenerate with `ess synthesize`

//! Semantic types synthesised from the `agenticspec` specification, v1.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent — behaviour, queries,
//! escalations — is listed with reasons in the `PLAN.md` beside this workspace, and every entry
//! there is owed through a typed seam in an `obligations` module here.

// `deny`, not the source workspace's lint set: this crate must hold on its own, and an undocumented
// public item here is an emitter defect worth failing the gate over.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod actor;
pub mod behaviour;
pub mod cli;
pub mod json;
pub mod obligation;
pub mod primitives;
pub mod session;
pub mod view;
