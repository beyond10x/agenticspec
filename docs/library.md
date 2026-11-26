# Records library

The Rust crate `sessionspec-records` holds what the `sessionspec` command uses: the transcript
reader, redaction, the store, the KPIs and the queries. A program that sees model traffic, such as
an LLM gateway, uses it to write the same records into a store that `sessionspec` reads.

```toml
[dependencies]
sessionspec-records = { git = "https://github.com/beyond10x/sessionspec", tag = "<version>" }
sessionspec-types = { git = "https://github.com/beyond10x/sessionspec", tag = "<version>" }
```

## Record gateway traffic

Per finished model call, lift the call into steps and hand them to the appender:

```rust
use std::path::Path;
use sessionspec_records::append::Appender;
use sessionspec_records::wire::{self, Answer, GatewayFacts, Wire, WireCall, WireSession};
use sessionspec_types::session::Harness;

let (appender, writer) = Appender::spawn(Path::new("sessions.db"), 1024)?;
let mut session = WireSession::new(); // keep one per gateway session

let call = WireCall {
    wire: Wire::Messages,
    request: request_body,
    answer: Answer::Events(sse_payloads),
    facts: GatewayFacts { /* call id, session hash, route, timings, ... */ },
};
let steps = wire::lift_call(&mut session, &call);
appender.append(&call.facts.session_hash, Harness::ClaudeCode, steps);
```

| item | what it does |
|---|---|
| `Wire` | the API the client spoke: `Messages`, `Chat` or `Responses` |
| `Answer` | the answer as relayed: a whole JSON `Body`, or the `Events` of a stream |
| `GatewayFacts` | what the gateway knows itself: call id, session hash, route, upstream, attempts, queue wait, time to first token, declared cost, start and end |
| `WireSession` | what is kept per session between calls, so each call is attributed to the right agent |
| `wire::lift_call` | turns one finished call into steps |
| `wire::session_hash(source, value)` | the 12-hex-digit session id the gateway derives from a session header |
| `Appender::spawn(path, capacity)` | opens the store and starts a writer thread behind a queue of `capacity` batches |
| `AppenderHandle::append` | queues one call's steps. It never blocks: when the queue is full it returns `false` and counts a drop |
| `AppenderHandle::stats` | counts of batches queued, dropped, written and failed |

The writer extends each session's views as steps arrive, so `sessionspec serve` on the same store
shows gateway sessions live. The writer thread ends when every handle is dropped and the queue is
drained. Join it with the returned handle.

## Join gateway facts to transcripts

When the same session is also recorded on disk, `wire::join(&mut disk_steps, &wire_steps)` copies
the gateway's facts onto the transcript's model turns with the same message id. It returns how many
turns it joined. Message ids match for Anthropic traffic.

## Privacy

The wire lift keeps lengths and SHA-256 digests of content, never the text. It does not scan
gateway traffic for secrets and personal data yet, so gateway sessions report no exposures. See
[Privacy](privacy.md).
