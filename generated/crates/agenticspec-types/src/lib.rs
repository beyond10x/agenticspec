// generated from agenticspec v1
// contract digest 7d11d25d7fef32d7b76d91e8b03a6eb8fea1f6c0915288d49fac9326766d8d6d
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
