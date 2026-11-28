//! The contract of an architecture: the ports of the building blocks, derived from their compiled
//! ESS system (`ports`), and, per architecture document, the ports its parts use with their
//! settings, instances, routing invariants and border (`derive`), whose invariants are written
//! in a small predicate language (`predicate`). The format of both files is on
//! `docs/contract.md`.
//!
//! Nothing here changes a simulation result: the catalog digest leaves this directory out.

pub mod derive;
pub mod ports;
pub mod predicate;
pub mod protocol;
