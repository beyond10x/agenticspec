//! agenticspec: the command line over the interaction records (`agenticspec-records`): pattern
//! mining (`mine`), procedure export (`export`), the dashboard file (`html`) and the local UI
//! server (`serve`).
//!
//! The records modules are re-exported under their own names, so `agenticspec::claude`,
//! `agenticspec::kpi` and the others keep their paths.

pub use agenticspec_records::{
    Load, blocks, claude, kpi, pricing, redact, replay, sq, store, time, view,
};

pub mod draft;
pub mod export;
pub mod html;
pub mod mine;
pub mod serve;
