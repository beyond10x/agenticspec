//! The contract of an architecture: the ports of the building blocks, derived from their compiled
//! ESS system (`ports`), and, per architecture document, the ports its parts use with their
//! settings, instances, routing invariants and border (`derive`), whose invariants are written
//! in a small predicate language (`predicate`). A contract cuts the blocks' conformance suite to
//! its ports (`suite`) and has its routing invariants checked on a trace in port terms
//! (`trace`). The ingresses of a document have each interface resolved into operations
//! (`ingress`). The formats are on `docs/contract.md`.
//!
//! Nothing here changes a simulation result: the catalog digest leaves this directory out.

pub mod derive;
pub mod ingress;
pub mod ports;
pub mod predicate;
pub mod protocol;
pub mod suite;
pub mod trace;
