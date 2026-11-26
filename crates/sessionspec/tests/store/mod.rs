//! The local store, its views and the server, over the synthetic fixture project, each test in a
//! database of its own under cargo's test temporary directory.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sessionspec::serve::{self, Hub, Seen, Shared, StoreView};
use sessionspec::store::Store;
use sessionspec_types::session as s;
use sessionspec_types::view::obligations::{
    SessionAgentsQuery, SessionByIdQuery, SessionChecksQuery, SessionEventsQuery, SessionListQuery,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn store(name: &str) -> Store {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("store")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    Store::open(&dir.join("sessions.db")).unwrap()
}

#[test]
fn a_sync_writes_once_and_a_second_finds_nothing_changed() {
    let st = store("sync");
    let first = st.sync(&root(), 0).unwrap().result;
    assert_eq!(
        (first.scanned, first.lifted, first.unchanged, first.failed),
        (1, 1, 0, 0)
    );
    let second = st.sync(&root(), 0).unwrap().result;
    assert_eq!(
        (second.lifted, second.unchanged, second.sessions),
        (0, 1, 1)
    );
}

#[test]
fn stored_commands_replay_as_the_transcript_does() {
    let st = store("load");
    st.sync(&root(), 0).unwrap();
    let main = root().join("project/s1.jsonl");
    let from_file = sessionspec::claude::lift(&main).unwrap();
    let from_store = st.load(&main).unwrap();
    assert_eq!(from_store.session_id, from_file.session_id);
    assert_eq!(from_store.steps.len(), from_file.steps.len());
    let a = sessionspec::replay::replay(&from_file.steps);
    let b = sessionspec::replay::replay(&from_store.steps);
    assert_eq!(format!("{:?}", a.events), format!("{:?}", b.events));
}

#[test]
fn the_views_answer_from_the_store() {
    let st = Arc::new(store("views"));
    st.sync(&root(), 0).unwrap();
    let view = StoreView::new(Arc::clone(&st));
    let all = view.session_list(None, None, None).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].rank, 1);
    let id = all[0].session_id.clone();
    // The search ignores case; a word the session does not hold finds nothing.
    assert_eq!(
        view.session_list(None, Some("BASH".into()), None)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        view.session_list(None, Some("no-such-tool".into()), None)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(view.session_list(None, None, Some(0)).unwrap().len(), 0);
    let later = sessionspec_types::primitives::Timestamp("2100-01-01T00:00:00Z".into());
    assert_eq!(view.session_list(Some(later), None, None).unwrap().len(), 0);

    let one = view.session_by_id(id.clone()).unwrap();
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].kpis.sessions, 1);
    assert!(
        view.session_by_id(s::SessionId("other".into()))
            .unwrap()
            .is_empty()
    );
    assert!(!view.session_agents(id.clone()).unwrap().is_empty());
    let events = view.session_events(id.clone()).unwrap();
    assert!(events.iter().enumerate().all(|(i, e)| e.seq == i as i64));
    let checks = view.session_checks(id).unwrap();
    let program = sessionspec::sq::compile(sessionspec::store::CHECKS).unwrap();
    assert_eq!(checks.len(), program.checks.len());
}

fn get(port: u16, path: &str) -> (String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(s, "GET {path} HTTP/1.1\r\nhost: x\r\n\r\n").unwrap();
    let mut text = String::new();
    s.read_to_string(&mut text).unwrap();
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    (head.to_owned(), body.to_owned())
}

#[test]
fn the_server_answers_views_and_streams_events() {
    let st = Arc::new(store("serve"));
    st.sync(&root(), 0).unwrap();
    let id = st.sessions().unwrap()[0].session_id.0.clone();
    let shared = Arc::new(Shared {
        system: Mutex::new(serve::system(Arc::clone(&st))),
        hub: Hub::default(),
        store: Arc::clone(&st),
        ui: None,
    });
    let listener = serve::bind(0).unwrap();
    let port = listener.local_addr().unwrap().port();
    let runner = Arc::clone(&shared);
    std::thread::spawn(move || serve::run(listener, runner, root(), 1));

    let (head, body) = get(port, "/view/views/session-list?search=bash");
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    assert!(body.contains(&id), "{body}");
    let (head, _) = get(port, "/view/views/session-checks?session_id=x");
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    // No UI built: a page path says how to build it.
    let (head, body) = get(port, "/");
    assert!(head.starts_with("HTTP/1.1 404"), "{head}");
    assert!(body.contains("ui:build"));

    let mut sse = TcpStream::connect(("127.0.0.1", port)).unwrap();
    sse.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    write!(
        sse,
        "GET /view/channels/session_events?session_id={id} HTTP/1.1\r\nhost: x\r\n\r\n"
    )
    .unwrap();
    let mut reader = BufReader::new(sse);
    let mut line = String::new();
    loop {
        line.clear();
        reader.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
    }
    // From an empty `Seen` every row of the session is new.
    let sent = serve::publish_changes(&shared, &mut Seen::default(), std::slice::from_ref(&id));
    assert!(sent > 1);
    let mut events = Vec::new();
    while events.len() < 2 {
        line.clear();
        reader.read_line(&mut line).unwrap();
        if let Some(name) = line.strip_prefix("event: ") {
            events.push(name.trim().to_owned());
        }
    }
    assert_eq!(events[0], "sessionspec.view.SessionUpdated");
    assert_eq!(events[1], "sessionspec.view.AgentUpdated");
}
