//! The event channel of each draft: every client open on a draft subscribes to it, and the studio
//! publishes on it each new revision of the draft and each answered or failed tool call of a
//! session on it. Over HTTP the channel is a server-sent-events stream.

use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use serde_json::Value;

/// The draft moved to a new revision: an applied patch, or its source changed on disk.
pub const DRAFT_REVISED: &str = "studio.draft.DraftRevised";
/// A tool call of a session on the draft was answered.
pub const CALL_ANSWERED: &str = "studio.session.ToolCallAnswered";
/// A tool call of a session on the draft could not be carried out.
pub const CALL_FAILED: &str = "studio.session.ToolCallFailed";

/// The longest a stream stays silent: then it writes a comment, which also finds a client that
/// went away.
pub const KEEPALIVE: Duration = Duration::from_secs(15);

/// One event on a draft's channel: its name (an event of the `studio` model) and its data.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub name: &'static str,
    pub data: Value,
}

impl Event {
    /// The event as one server-sent-events frame.
    pub fn frame(&self) -> String {
        format!("event: {}\ndata: {}\n\n", self.name, self.data)
    }
}

/// The subscribers of every draft's channel. A subscriber whose receiver is gone is dropped at
/// the next event of its draft.
#[derive(Debug, Default)]
pub struct Hub {
    subscribers: Vec<(String, Sender<Event>)>,
}

impl Hub {
    /// A new subscriber of the channel of `draft`.
    pub fn subscribe(&mut self, draft: &str) -> Receiver<Event> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push((draft.to_string(), tx));
        rx
    }

    /// Sends `event` to every subscriber of `draft`.
    pub fn publish(&mut self, draft: &str, event: &Event) {
        self.subscribers
            .retain(|(d, tx)| d != draft || tx.send(event.clone()).is_ok());
    }

    /// How many subscribers the channel of `draft` holds.
    pub fn subscribers(&self, draft: &str) -> usize {
        self.subscribers.iter().filter(|(d, _)| d == draft).count()
    }
}

/// The head of a server-sent-events response. It has no length: the stream ends when the
/// connection closes.
pub const HEAD: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n";

/// Writes the head, a first comment `hello`, then every event `events` receives, each flushed at
/// once, and a comment after `keepalive` of silence. Ends when the sender side is gone, or with
/// the error of a write to a client that went away.
pub fn stream<W: Write>(
    out: &mut W,
    hello: &str,
    events: &Receiver<Event>,
    keepalive: Duration,
) -> io::Result<()> {
    out.write_all(HEAD.as_bytes())?;
    out.write_all(format!("retry: 2000\n: {hello}\n\n").as_bytes())?;
    out.flush()?;
    loop {
        match events.recv_timeout(keepalive) {
            Ok(event) => out.write_all(event.frame().as_bytes())?,
            Err(RecvTimeoutError::Timeout) => out.write_all(b": keep-alive\n\n")?,
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
        out.flush()?;
    }
}
