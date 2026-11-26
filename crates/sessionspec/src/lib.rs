//! sessionspec: the command line over the interaction records (`sessionspec-records`): pattern
//! mining (`mine`), procedure export (`export`), the dashboard file (`html`) and the local UI
//! server (`serve`).
//!
//! The records modules are re-exported under their own names, so `sessionspec::claude`,
//! `sessionspec::kpi` and the others keep their paths.

pub use sessionspec_records::{Load, claude, kpi, pricing, redact, replay, sq, store, time, view};

pub mod export;
pub mod html;
pub mod mine;
pub mod serve;
