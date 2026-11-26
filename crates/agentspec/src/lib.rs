//! agentspec: the command line over the interaction records (`agentspec-records`): pattern
//! mining (`mine`), procedure export (`export`), the dashboard file (`html`) and the local UI
//! server (`serve`).
//!
//! The records modules are re-exported under their own names, so `agentspec::claude`,
//! `agentspec::kpi` and the others keep their paths.

pub use agentspec_records::{Load, claude, kpi, pricing, redact, replay, sq, store, time, view};

pub mod export;
pub mod html;
pub mod mine;
pub mod serve;
