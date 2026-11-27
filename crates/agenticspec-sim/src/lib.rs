//! agenticspec-sim: checks an architecture document, simulates it under a behaviour profile and
//! searches its marked places for the best variant.

pub mod check;
pub mod doc;
pub mod ladder;
pub mod marked;
pub mod profile;
pub mod rng;
pub mod search;
pub mod sim;

pub use search::{Experiment, run};
