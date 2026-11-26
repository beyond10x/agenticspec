//! The local store, its views and the server, over the synthetic fixture project, each test in a
//! database of its own under cargo's test temporary directory.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use agentspec::serve::{self, Hub, Seen, Shared, StoreView};
use agentspec::store::Store;
use agentspec_types::session as s;
use agentspec_types::view as v;
use agentspec_types::view::obligations::{
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
    let from_file = agentspec::claude::lift(&main).unwrap();
    let from_store = st.load(&main).unwrap();
    assert_eq!(from_store.session_id, from_file.session_id);
    assert_eq!(from_store.steps.len(), from_file.steps.len());
    let a = agentspec::replay::replay(&from_file.steps);
    let b = agentspec::replay::replay(&from_store.steps);
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
    let later = agentspec_types::primitives::Timestamp("2100-01-01T00:00:00Z".into());
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
        agentspec::time::epoch_ms(&total.last_at.0),
        agentspec::time::epoch_ms(&events[0].at.0)
    );
    let synced = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    assert!(agentspec::time::epoch_ms(&total.last_at.0).unwrap() < synced - 3_600_000);
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
    let last = agentspec::time::epoch_ms(&total.last_at.0).unwrap();
    let hour = last / 3_600_000 * 3_600_000;
    let since = agentspec_types::primitives::Timestamp(agentspec::time::rfc3339(hour));
    let figures = view.overview_figures(since, 3600).unwrap();
    assert_eq!(figures.len(), 1);
    assert_eq!(figures[0].sessions, 1);
    assert_eq!(figures[0].tool_calls, total.tool_calls);
    assert_eq!(figures[0].cost_usd, total.cost_usd);

    let checks = view.session_checks(id).unwrap();
    let program = agentspec::sq::compile(agentspec::store::CHECKS).unwrap();
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

/// `n` copies of the fixture session `s1` with its sub-agents under `<dir>/root/project`; returns
/// the project directory.
fn copies(dir: &std::path::Path, n: usize) -> PathBuf {
    let src = root().join("project");
    let project = dir.join("root/project");
    std::fs::create_dir_all(&project).unwrap();
    for k in 0..n {
        std::fs::copy(src.join("s1.jsonl"), project.join(format!("s{k}.jsonl"))).unwrap();
        let subs = project.join(format!("s{k}/subagents"));
        std::fs::create_dir_all(&subs).unwrap();
        for f in std::fs::read_dir(src.join("s1/subagents")).unwrap() {
            let f = f.unwrap().path();
            std::fs::copy(&f, subs.join(f.file_name().unwrap())).unwrap();
        }
    }
    project
}

/// Marks every transcript of `project` as changed, so the next sync lifts each again.
fn touch_all(project: &std::path::Path, n: usize, round: u64) {
    let at = std::time::SystemTime::now() + Duration::from_secs(round + 1);
    for k in 0..n {
        let f = std::fs::File::options()
            .write(true)
            .open(project.join(format!("s{k}.jsonl")))
            .unwrap();
        f.set_modified(at).unwrap();
    }
}

/// How many processes whose parent is `pid` run with `arg` on their command line, from `/proc`.
#[cfg(target_os = "linux")]
fn children_running(pid: u32, arg: &str) -> usize {
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return 0;
    };
    dir.filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.parse::<u32>().is_ok())
        })
        .filter(|e| {
            let stat = std::fs::read_to_string(e.path().join("stat")).unwrap_or_default();
            // Field 4, after the command name in parentheses, is the parent's pid.
            let ppid = stat
                .rsplit_once(')')
                .and_then(|(_, rest)| rest.split_whitespace().nth(1))
                .and_then(|p| p.parse::<u32>().ok());
            ppid == Some(pid)
        })
        .filter(|e| {
            let cmd = std::fs::read(e.path().join("cmdline")).unwrap_or_default();
            cmd.split(|b| *b == 0).any(|a| a == arg.as_bytes())
        })
        .count()
}

/// The server's sync child: `sync` of the server's state directory, root and window, two lifts
/// at a time, its output closed.
#[test]
fn the_server_syncs_through_a_child_process() {
    let syncer = serve::Syncer {
        exe: PathBuf::from("/bin/agentspec"),
        state_dir: PathBuf::from("/state"),
        root: PathBuf::from("/root/projects"),
        since_hours: 24,
    };
    let c = syncer.command();
    assert_eq!(c.get_program(), "/bin/agentspec");
    let args: Vec<_> = c.get_args().map(|a| a.to_str().unwrap()).collect();
    assert_eq!(
        args,
        [
            "sync",
            "--state-dir",
            "/state",
            "--root",
            "/root/projects",
            "--since-hours",
            "24"
        ]
    );
    let threads = c
        .get_envs()
        .find(|(k, _)| *k == "RAYON_NUM_THREADS")
        .and_then(|(_, v)| v);
    assert_eq!(threads, Some(std::ffi::OsStr::new("2")));
}

/// `agentspec serve` never lifts a transcript itself: over 8 syncs that each re-lift 48 changed
/// sessions, a `agentspec sync` child of the server is seen running, and the server's peak
/// resident memory grows by less than 4 MiB after its first sync. Before the sync ran in a
/// child, the server kept each re-lift's peak (847 MiB after 3.5 min on a 117-session store).
#[cfg(target_os = "linux")]
#[test]
fn the_server_keeps_no_lift_in_its_own_memory() {
    const GROWTH_KIB: u64 = 4 * 1024;
    const N: usize = 48;
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("store/child-sync");
    let _ = std::fs::remove_dir_all(&dir);
    let project = copies(&dir, N);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_agentspec"))
        .args(["serve", "--port", "0", "--state-dir"])
        .arg(dir.join("state"))
        .arg("--root")
        .arg(dir.join("root"))
        .env("AGENTSPEC_SYNC_MS", "100")
        .env("AGENTSPEC_UI_DIR", dir.join("no-ui"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    // The first sync ran in a child before the server opened the store.
    let db = dir.join("state/sessions.db");
    let mut held = 0;
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(50));
        if db.exists() {
            held = Store::open(&db).unwrap().sessions().unwrap().len();
            if held == N {
                break;
            }
        }
    }
    std::thread::sleep(Duration::from_millis(300));
    let (start, _) = rss_kib(child.id());
    let mut sync_children = 0;
    for round in 0..8 {
        touch_all(&project, N, round);
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(10));
            sync_children += children_running(child.id(), "sync");
        }
    }
    let (end, now) = rss_kib(child.id());
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(held, N, "the first sync child did not fill the store");
    assert!(start > 0, "no /proc status read");
    assert!(
        sync_children > 0,
        "no `agentspec sync` child of the server seen in 8 syncs"
    );
    assert!(
        end - start < GROWTH_KIB,
        "serve's peak grew from {start} to {end} KiB (now {now} KiB) over 8 re-lifts"
    );
}

/// `agentspec serve` re-lifts every changed transcript on each sync. Over 48 fixture sessions,
/// each changed 6 times, its peak resident memory stays under 96 MiB. Measured 2026-11-26: about
/// 20 MiB; before the sync lifted in chunks and returned freed pages, a copy of a 116-session
/// store grew to 1.9 GiB within 30 s.
#[cfg(target_os = "linux")]
#[test]
fn serve_holds_its_memory_while_it_re_lifts() {
    const BOUND_KIB: u64 = 96 * 1024;
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("store/rss");
    let _ = std::fs::remove_dir_all(&dir);
    let project = copies(&dir, 48);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_agentspec"))
        .args(["serve", "--port", "0", "--state-dir"])
        .arg(dir.join("state"))
        .arg("--root")
        .arg(dir.join("root"))
        .env("AGENTSPEC_SYNC_MS", "100")
        .env("AGENTSPEC_UI_DIR", dir.join("no-ui"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut peak = 0;
    for round in 0..6 {
        std::thread::sleep(Duration::from_millis(250));
        touch_all(&project, 48, round);
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
    let syncer = serve::Syncer {
        exe: PathBuf::from(env!("CARGO_BIN_EXE_agentspec")),
        state_dir: st.path().parent().unwrap().to_path_buf(),
        root: root(),
        since_hours: 1,
    };
    std::thread::spawn(move || serve::run(listener, runner, syncer));

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
    assert_eq!(events[0], "agentspec.view.SessionUpdated");
    assert_eq!(events[1], "agentspec.view.AgentUpdated");
}

/// Without `--out`, `analyze` writes its file beside the store, not in the working directory.
#[test]
fn analyze_writes_beside_the_store_by_default() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("store/analyze");
    let _ = std::fs::remove_dir_all(&dir);
    let cwd = dir.join("checkout");
    std::fs::create_dir_all(&cwd).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_agentspec"))
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
    assert!(dir.join("state/agentspec-dashboard.html").is_file());
    assert!(!cwd.join("agentspec-dashboard.html").exists());
}
