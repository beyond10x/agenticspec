//! The writer a gateway appends through: [`Appender::spawn`] starts one thread that owns the
//! store, and [`AppenderHandle::append`] hands it a session's new steps without blocking. A full
//! queue drops the steps and counts the drop; the caller's connection never waits on SQLite.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::thread::JoinHandle;
use std::time::Duration;

use sessionspec_types::session as s;

use crate::claude::Step;
use crate::store::{CHECKS, Projections, Store};

/// One batch for the writer: a session's hash, its harness and the steps of one call.
struct Batch {
    session: String,
    harness: s::Harness,
    steps: Vec<Step>,
}

/// Counts the writer keeps; read them with [`AppenderHandle::stats`].
#[derive(Debug, Default)]
pub struct Stats {
    /// Batches taken into the queue.
    pub queued: AtomicU64,
    /// Batches dropped because the queue was full.
    pub dropped: AtomicU64,
    /// Batches written.
    pub written: AtomicU64,
    /// Batches the store refused (an SQLite error); the error is not kept.
    pub failed: AtomicU64,
}

/// The writer thread.
pub struct Appender;

/// The side a gateway holds: cheap to clone, never blocks.
#[derive(Clone)]
pub struct AppenderHandle {
    tx: SyncSender<Batch>,
    stats: Arc<Stats>,
}

impl Appender {
    /// Opens the store at `path` and starts the writer behind a queue of `capacity` batches. The
    /// thread ends when every handle is dropped and the queue is drained; join it with the
    /// returned [`JoinHandle`].
    pub fn spawn(path: &Path, capacity: usize) -> Result<(AppenderHandle, JoinHandle<()>), String> {
        let store = Store::open(path)?;
        let program = crate::sq::compile(CHECKS)?;
        let (tx, rx) = sync_channel::<Batch>(capacity.max(1));
        let stats = Arc::new(Stats::default());
        let thread_stats = stats.clone();
        let join = std::thread::Builder::new()
            .name("sessionspec-appender".to_owned())
            .spawn(move || writer(&store, &program, &rx, &thread_stats))
            .map_err(|e| e.to_string())?;
        Ok((AppenderHandle { tx, stats }, join))
    }
}

/// Sessions whose projection the writer keeps; past it the least recently appended is dropped and
/// projected again from the store when it appends again.
const OPEN_SESSIONS: usize = 64;

/// How long the queue stays empty before the writer judges the sessions whose verdicts lag.
const IDLE: Duration = Duration::from_millis(500);

fn writer(store: &Store, program: &crate::sq::Program, rx: &Receiver<Batch>, stats: &Stats) {
    let mut open = Projections::new(OPEN_SESSIONS);
    loop {
        let first = match rx.recv_timeout(IDLE) {
            Ok(b) => b,
            Err(RecvTimeoutError::Timeout) => {
                let _ = store.flush_checks(program, &mut open);
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => {
                let _ = store.flush_checks(program, &mut open);
                return;
            }
        };
        // Take what else is queued, so a session written by several calls is projected once.
        let mut by_session: BTreeMap<String, (s::Harness, Vec<Step>, u64)> = BTreeMap::new();
        let mut add = |b: Batch| {
            let e = by_session
                .entry(b.session)
                .or_insert_with(|| (b.harness, Vec::new(), 0));
            e.1.extend(b.steps);
            e.2 += 1;
        };
        add(first);
        while let Ok(b) = rx.try_recv() {
            add(b);
        }
        for (session, (harness, steps, n)) in by_session {
            match store.append(&session, harness, &steps, program, &mut open) {
                Ok(()) => stats.written.fetch_add(n, Ordering::Relaxed),
                Err(_) => stats.failed.fetch_add(n, Ordering::Relaxed),
            };
        }
    }
}

impl AppenderHandle {
    /// Queues one call's steps for the session named by `session_hash`. Returns false and counts
    /// a drop when the queue is full or the writer has ended; never blocks.
    pub fn append(&self, session_hash: &str, harness: s::Harness, steps: Vec<Step>) -> bool {
        if steps.is_empty() {
            return true;
        }
        match self.tx.try_send(Batch {
            session: session_hash.to_owned(),
            harness,
            steps,
        }) {
            Ok(()) => {
                self.stats.queued.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.stats.dropped.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    /// The writer's counts.
    pub fn stats(&self) -> &Stats {
        &self.stats
    }
}
