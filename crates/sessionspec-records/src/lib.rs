//! sessionspec-records: the interaction records of agent sessions (`docs/interaction-records.md`).
//!
//! Sources lift what they observed into the session IR's commands: `claude` reads Claude Code
//! transcripts on disk, `wire` lifts one finished model call on a gateway. The commands replay
//! through the generated `session-ir` component (`replay`), whose published events every reader
//! takes: the KPIs (`kpi`), the view projection (`view`), the queries (`sq`). `store` keeps each
//! session's commands in SQLite; `append` is the non-blocking writer a gateway appends through.
//!
//! A gateway's adapter calls, per finished call: [`wire::lift_call`] with the session's
//! [`wire::WireSession`], then [`append::AppenderHandle::append`] with the steps.
//!
//! The IR types and the command behaviours are generated from `spec/` (`task generate`); this
//! crate holds what the specification leaves to an implementation.

pub mod append;
pub mod claude;
pub mod kpi;
pub mod pricing;
pub mod redact;
pub mod replay;
pub mod sq;
pub mod store;
pub mod time;
pub mod view;
pub mod wire;

/// Where a reader takes the lifted commands of a main transcript from: the file
/// (`claude::lift`) or the local store (`store::Store::load`).
pub type Load<'a> = &'a (dyn Fn(&std::path::Path) -> std::io::Result<claude::Lifted> + Sync);
