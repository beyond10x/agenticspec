//! agenticspec-sim, the library behind `agenticspec sim`: checks an architecture document,
//! simulates it under a behaviour profile and searches its marked places for the best variant.

pub mod best;
pub mod check;
pub mod cli;
pub mod contract;
pub mod corpus;
pub mod doc;
pub mod event_log;
pub mod events;
pub mod ladder;
pub mod marked;
pub mod nest;
pub mod profile;
pub mod reader;
pub mod rng;
pub mod rows;
pub mod scenario;
pub mod search;
pub mod sim;
pub mod whole;

pub use search::{Experiment, Shown, run, run_shown};
