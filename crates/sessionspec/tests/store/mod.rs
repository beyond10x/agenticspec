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
use sessionspec_types::view as v;
use sessionspec_types::view::obligations::{
    OverviewFiguresQuery, SessionAgentsQuery, SessionByIdQuery, SessionChecksQuery,
    SessionEventCountsQuery, SessionEventsQuery, SessionListQuery, SessionTimelineQuery,
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
    let all = view.session_list(None, None, None, None).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].rank, 1);
    let id = all[0].session_id.clone();
    // The search ignores case, by the trigram index from three characters and by `names` below
    // that; a word the session does not hold finds nothing.
    for q in ["BASH", "ba", "a"] {
        assert_eq!(
            view.session_list(None, Some(q.into()), None, None)
                .unwrap()
                .len(),
            1,
            "{q}"
        );
    }
    assert_eq!(
        view.session_list(None, Some("no-such-tool".into()), None, None)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        view.session_list(None, Some("\"x".into()), None, None)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        view.session_list(None, None, Some(0), None).unwrap().len(),
        0
    );
    assert_eq!(
        view.session_list(None, None, None, Some(1)).unwrap().len(),
        0
    );
    let later = sessionspec_types::primitives::Timestamp("2100-01-01T00:00:00Z".into());
    assert_eq!(
        view.session_list(Some(later), None, None, None)
            .unwrap()
            .len(),
        0
    );

    let one = view.session_by_id(id.clone()).unwrap();
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].kpis.sessions, 1);
    assert!(
        view.session_by_id(s::SessionId("other".into()))
            .unwrap()
            .is_empty()
    );
    let agents = view.session_agents(id.clone()).unwrap();
    assert!(!agents.is_empty());

    // Events: newest place first; a page goes on from the last place of the one before.
    let events = view
        .session_events(id.clone(), None, None, None, None)
        .unwrap();
    let total = st.session(&id.0).unwrap().unwrap();
    assert!(!events.is_empty());
    assert!(events.windows(2).all(|w| w[0].ord > w[1].ord));
    assert!(events.windows(2).all(|w| w[0].at.0 >= w[1].at.0));
    // Last active is the time of the session's last record, not the time of the sync.
    assert!(events.len() < 500, "the fixture fits one page");
    assert_eq!(
        sessionspec::time::epoch_ms(&total.last_at.0),
        sessionspec::time::epoch_ms(&events[0].at.0)
    );
    let synced = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    assert!(sessionspec::time::epoch_ms(&total.last_at.0).unwrap() < synced - 3_600_000);
    let after = view
        .session_events(id.clone(), None, None, None, Some(events[2].ord))
        .unwrap();
    assert_eq!(after.len(), events.len() - 3);
    assert_eq!(after[0].ord, events[3].ord);
    let calls = view
        .session_events(id.clone(), None, Some(v::EventKind::ToolCall), None, None)
        .unwrap();
    assert_eq!(calls.len() as i64, total.tool_calls);
    let notable = view
        .session_events(id.clone(), None, None, Some(true), None)
        .unwrap();
    assert!(notable.iter().all(|e| e.notable));
    assert!(notable.len() < events.len());
    let main = &agents[0].agent_id;
    let mine = view
        .session_events(id.clone(), Some(main.clone()), None, None, None)
        .unwrap();
    assert!(!mine.is_empty() && mine.iter().all(|e| &e.agent == main));

    // The counts per kind add up to the events; the timeline counts every call once.
    let counts = view.session_event_counts(id.clone(), None).unwrap();
    assert_eq!(
        counts.iter().map(|c| c.count).sum::<i64>() as usize,
        events.len()
    );
    for width in [1, 60, 86_400] {
        let slots = view.session_timeline(id.clone(), width).unwrap();
        assert!(slots.windows(2).all(|w| w[0].start.0 <= w[1].start.0));
        let tools: i64 = slots
            .iter()
            .map(|x| {
                x.read_calls
                    + x.search_calls
                    + x.edit_calls
                    + x.execute_calls
                    + x.network_calls
                    + x.spawn_calls
                    + x.message_calls
                    + x.plan_calls
                    + x.other_calls
            })
            .sum();
        assert_eq!(tools, total.tool_calls, "slot {width} s");
    }

    // The overview's figures: one hour bucket from the session's hour holds it.
    let last = sessionspec::time::epoch_ms(&total.last_at.0).unwrap();
    let hour = last / 3_600_000 * 3_600_000;
    let since = sessionspec_types::primitives::Timestamp(sessionspec::time::rfc3339(hour));
    let figures = view.overview_figures(since, 3600).unwrap();
    assert_eq!(figures.len(), 1);
    assert_eq!(figures[0].sessions, 1);
    assert_eq!(figures[0].tool_calls, total.tool_calls);
    assert_eq!(figures[0].cost_usd, total.cost_usd);

    let checks = view.session_checks(id).unwrap();
    let program = sessionspec::sq::compile(sessionspec::store::CHECKS).unwrap();
    assert_eq!(checks.len(), program.checks.len());
}

/// The peak and current resident memory of a process, in KiB, from `/proc/<pid>/status`.
#[cfg(target_os = "linux")]
fn rss_kib(pid: u32) -> (u64, u64) {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
    let field = |name: &str| {
        status
            .lines()
            .find_map(|l| l.strip_prefix(name))
            .and_then(|v| v.split_whitespace().next())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    (field("VmHWM:"), field("VmRSS:"))
}

/// `sessionspec serve` re-lifts every changed transcript on each sync. Over 48 fixture sessions,
/// each changed 6 times, its peak resident memory stays under 96 MiB. Measured 2026-11-26: about
/// 20 MiB; before the sync lifted in chunks and returned freed pages, a copy of a 116-session
/// store grew to 1.9 GiB within 30 s.
#[cfg(target_os = "linux")]
#[test]
fn serve_holds_its_memory_while_it_re_lifts() {
    const BOUND_KIB: u64 = 96 * 1024;
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("store/rss");
    let _ = std::fs::remove_dir_all(&dir);
    let src = root().join("project");
    let project = dir.join("root/project");
    std::fs::create_dir_all(&project).unwrap();
    for k in 0..48 {
        std::fs::copy(src.join("s1.jsonl"), project.join(format!("s{k}.jsonl"))).unwrap();
        let subs = project.join(format!("s{k}/subagents"));
        std::fs::create_dir_all(&subs).unwrap();
        for f in std::fs::read_dir(src.join("s1/subagents")).unwrap() {
            let f = f.unwrap().path();
            std::fs::copy(&f, subs.join(f.file_name().unwrap())).unwrap();
        }
    }
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_sessionspec"))
        .args(["serve", "--port", "0", "--state-dir"])
        .arg(dir.join("state"))
        .arg("--root")
        .arg(dir.join("root"))
        .env("SESSIONSPEC_SYNC_MS", "100")
        .env("SESSIONSPEC_UI_DIR", dir.join("no-ui"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut peak = 0;
    for round in 0..6 {
        std::thread::sleep(Duration::from_millis(250));
        let at = std::time::SystemTime::now() + Duration::from_secs(round + 1);
        for k in 0..48 {
            let f = std::fs::File::options()
                .write(true)
                .open(project.join(format!("s{k}.jsonl")))
                .unwrap();
            f.set_modified(at).unwrap();
        }
        peak = peak.max(rss_kib(child.id()).0);
    }
    std::thread::sleep(Duration::from_millis(300));
    let (hwm, now) = rss_kib(child.id());
    let held = Store::open(&dir.join("state/sessions.db"))
        .unwrap()
        .sessions()
        .unwrap()
        .len();
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(held, 48, "the server's syncs did not lift the root");
    let peak = peak.max(hwm);
    assert!(peak > 0, "no /proc status read");
    assert!(
        peak < BOUND_KIB,
        "serve peaked at {peak} KiB (now {now} KiB); the bound is {BOUND_KIB} KiB"
    );
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
    let sent = serve::publish_changes(&shared, &mut Seen::default());
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

/// Without `--out`, `analyze` writes its file beside the store, not in the working directory.
#[test]
fn analyze_writes_beside_the_store_by_default() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("store/analyze");
    let _ = std::fs::remove_dir_all(&dir);
    let cwd = dir.join("checkout");
    std::fs::create_dir_all(&cwd).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sessionspec"))
        .args(["analyze", "--since-hours", "100000", "--state-dir"])
        .arg(dir.join("state"))
        .arg("--root")
        .arg(root())
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dir.join("state/sessionspec-dashboard.html").is_file());
    assert!(!cwd.join("sessionspec-dashboard.html").exists());
}
