//! `agenticspec ui serve` as a library: the default control plane read from the operator UI of
//! the workstation, and the server checked by `ui conform --architecture` with one fault per
//! obligation. The healthy server and the story's acceptance run on the command line, against
//! the one server that test starts (`crates/agenticspec/tests/ui`).

use std::path::{Path, PathBuf};
use std::time::Duration;

use agenticspec_ui::conform::{self, Status};
use agenticspec_ui::operator;
use agenticspec_ui::plane::Plane;
use agenticspec_ui::serve::{self, Faults, Fixtures, Served};

const TOKEN: &str = "serve-test";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The files of the workstation's operator UI with the API base of `ui serve`.
fn files() -> Vec<(String, Vec<u8>)> {
    let root = root();
    let contract =
        operator::read_doc(&root.join("ui/views/examples/workstation.contract.yaml")).unwrap();
    operator::build_with(
        &root,
        &root.join("architectures/workstation.yaml"),
        &contract,
        "workstation.contract.yaml",
        serve::ROOT,
    )
    .unwrap()
}

/// The plane holds every part of the layout with the reads of its page, the level's channel,
/// the commands of the operator ingress and of the layout's actions, and each view's shape.
#[test]
fn the_plane_of_the_workstation_is_read_from_its_operator_ui() {
    let plane = Plane::from_files(&files()).unwrap();
    assert_eq!(plane.architecture, "workstation");
    let parts: Vec<&str> = plane.parts.iter().map(|p| p.path.as_str()).collect();
    for p in ["owner", "operator", "worker", "gateway", "builds"] {
        assert!(parts.contains(&p), "{parts:?}");
    }
    assert_eq!(plane.part("worker").unwrap().count, 12);
    let gateway = plane.part("gateway").unwrap();
    assert!(
        gateway
            .reads
            .iter()
            .any(|r| r.view == "agentic.gateway.Gateways"),
        "{gateway:?}"
    );
    let queue = plane
        .part("builds")
        .unwrap()
        .reads
        .iter()
        .find(|r| r.view == "agentic.resource.Queue")
        .unwrap();
    assert!(queue.params.contains_key("pool_id"), "{queue:?}");
    assert!(!queue.params.contains_key("channel"), "{queue:?}");
    assert_eq!(plane.channels, ["level"]);
    for c in [
        "agentic.gateway.DrainGateway",
        "agentic.question.Answer",
        "agentic.channel.SendMessage",
    ] {
        assert!(plane.commands.contains(c), "{:?}", plane.commands);
    }
    assert!(plane.feeds.contains("agentic.agent.AgentDefined"));
    assert!(plane.views.contains("agentic.architecture.Placements"));
    let shape = &plane.shapes["agentic.agent.Agents"];
    assert_eq!(shape.key, "agent_id");
    assert!(shape.fields.contains("state"), "{shape:?}");
}

/// A server that adds an undeclared field to one view, resends the event of the cursor on a
/// reconnect and withholds a command in R5 that R4 still carries out fails exactly those checks,
/// each naming what broke; the rest still pass.
#[test]
fn a_server_with_one_fault_per_obligation_fails_the_check_that_names_it() {
    let files = files();
    let plane = Plane::from_files(&files).unwrap();
    let served = Served::start(
        plane.clone(),
        files,
        Fixtures::load(&root()).unwrap(),
        serve::Options {
            port: 0,
            token: TOKEN.into(),
            faults: Faults {
                undeclared_field: Some("agentic.gateway.Gateways".into()),
                resend_on_resume: true,
                withhold: Some("agentic.gateway.DrainGateway".into()),
            },
        },
    )
    .unwrap();
    let report = conform::run_plane(
        &served.interface_root(),
        &plane,
        Some(TOKEN),
        conform::Options {
            timeout: Duration::from_secs(5),
            idle: Duration::from_millis(200),
        },
    );
    let failed = report.failed();
    let names: Vec<&str> = failed.iter().map(|c| c.name.as_str()).collect();
    let read = failed
        .iter()
        .find(|c| c.name == "R1 agentic.gateway.Gateways at workstation/gateway")
        .unwrap_or_else(|| panic!("{names:?}"));
    assert!(
        read.detail
            .contains("gateway_id=00000000-0000-4000-8000-9121f2990000 with the field undeclared"),
        "{}",
        read.detail
    );
    let resume = failed
        .iter()
        .find(|c| c.name == "R2 resume level at workstation")
        .unwrap_or_else(|| panic!("{names:?}"));
    assert!(resume.detail.contains("again"), "{}", resume.detail);
    let withheld = failed
        .iter()
        .find(|c| c.name == "R4 withheld agentic.gateway.DrainGateway at workstation")
        .unwrap_or_else(|| panic!("{names:?}"));
    assert!(
        withheld.detail.starts_with(
            "R5 withholds agentic.gateway.DrainGateway at workstation, and R4 answered 200"
        ),
        "{}",
        withheld.detail
    );
    assert!(
        failed
            .iter()
            .all(|c| c.name.contains("agentic.gateway.Gateways")
                || c.name.starts_with("R2 resume")
                || c.name
                    .starts_with("R4 withheld agentic.gateway.DrainGateway")),
        "{names:?}"
    );
    assert!(report.count(Status::Passed) > 50, "{names:?}");
}

/// The fixtures built into the binary are the fixtures of the checkout, and the committed
/// bundle is the one its sources give now.
#[test]
fn the_embedded_fixtures_are_the_fixtures_of_the_checkout() {
    assert_eq!(Fixtures::embedded(), Fixtures::load(&root()).unwrap());
    let held = std::fs::read_to_string(root().join(serve::BUNDLE_FILE)).unwrap();
    assert_eq!(
        held,
        serve::BUNDLE,
        "the binary is built from another bundle than the tree"
    );
    assert_eq!(held, serve::bundle(&root()).unwrap(), "run task ui:panels");
    serve::write_bundle(&root(), true).unwrap();
}

/// A directory of `ui serve --fixtures`: the views, `scripts/` and `outcomes.json` read as the
/// checkout's fixtures; scripts and outcomes may be absent; a directory with no view is refused.
#[test]
fn a_fixtures_directory_reads_views_scripts_and_outcomes() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("serve-fixtures");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    let from = root().join(serve::FIXTURES);
    for sub in ["", "scripts"] {
        for e in std::fs::read_dir(from.join(sub)).unwrap() {
            let p = e.unwrap().path();
            if p.is_file() {
                std::fs::copy(&p, dir.join(sub).join(p.file_name().unwrap())).unwrap();
            }
        }
    }
    std::fs::copy(
        root().join(serve::OUTCOMES_FILE),
        dir.join(serve::DIR_OUTCOMES),
    )
    .unwrap();
    assert_eq!(Fixtures::from_dir(&dir).unwrap(), Fixtures::embedded());

    std::fs::remove_dir_all(dir.join("scripts")).unwrap();
    std::fs::remove_file(dir.join(serve::DIR_OUTCOMES)).unwrap();
    let bare = Fixtures::from_dir(&dir).unwrap();
    assert_ne!(bare, Fixtures::embedded());

    let empty = dir.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let err = Fixtures::from_dir(&empty).unwrap_err();
    assert!(err.contains("holds no view fixture"), "{err}");
    let err = Fixtures::from_dir(&dir.join("absent")).unwrap_err();
    assert!(err.contains("absent"), "{err}");
}

/// The UI of one ingress of the workstation with ingresses, with the base path `ui serve` gives
/// it, and its plane.
fn ingress_ui(name: &str) -> (Plane, Vec<(String, Vec<u8>)>) {
    let root = root();
    let contract =
        operator::read_doc(&root.join("ui/views/examples/workstation-ingress.contract.yaml"))
            .unwrap();
    let files = operator::build_ingress(
        &root,
        &root.join("architectures/workstation-ingress.yaml"),
        &contract,
        "workstation-ingress.contract.yaml",
        &serve::root_of(name),
        None,
        Some(name),
    )
    .unwrap_or_else(|_| panic!("the UI of {name}"));
    (Plane::from_files(&files).unwrap(), files)
}

fn get(url: &str, token: Option<&str>) -> agenticspec_ui::wire::Response {
    let auth = token.map(|t| format!("Bearer {t}"));
    let headers: Vec<(&str, &str)> = auth.iter().map(|a| ("Authorization", a.as_str())).collect();
    agenticspec_ui::wire::request("GET", url, &headers, None, Duration::from_secs(5)).unwrap()
}

fn json(r: &agenticspec_ui::wire::Response) -> serde_json::Value {
    serde_json::from_slice(&r.body).unwrap()
}

/// Two ingresses with a `ui` export, `operator` and `user`, are served from one server: each app
/// at its own root with its `config.json` pointing at `/ingress/<ingress>`, each interface below
/// `/ingress/<ingress>/scopes/` answering its own plane, R5 listing only the commands of that
/// ingress, and each plane passing `ui conform` at its root.
#[test]
fn each_ingress_with_a_ui_is_served_at_its_own_root_and_answers_its_own_plane() {
    let (op_plane, op_files) = ingress_ui("operator");
    let (user_plane, user_files) = ingress_ui("user");
    assert_eq!(
        (op_plane.ingress.as_str(), user_plane.ingress.as_str()),
        ("operator", "user")
    );
    let served = Served::start_uis(
        vec![
            (op_plane.clone(), op_files),
            (user_plane.clone(), user_files),
        ],
        Vec::new(),
        Fixtures::embedded(),
        serve::Options {
            port: 0,
            token: TOKEN.into(),
            faults: Faults::default(),
        },
    )
    .unwrap();
    let url = served.url().to_string();

    // Each app at its own root; the operator's at `/` too.
    let moved = get(&format!("{url}/ingress/user"), None);
    assert_eq!(moved.status, 308);
    assert_eq!(moved.header("location"), Some("/ingress/user/"));
    let page = get(&format!("{url}/ingress/user/"), None);
    assert_eq!(page.status, 200);
    assert!(page.header("set-cookie").is_some());
    for (name, at) in [
        ("user", format!("{url}/ingress/user/config.json")),
        ("operator", format!("{url}/ingress/operator/config.json")),
        ("operator", format!("{url}/config.json")),
    ] {
        let config = json(&get(&at, None));
        assert_eq!(config["ingress"], name, "{at}");
        assert_eq!(config["api_base"], format!("/ingress/{name}"), "{at}");
    }

    // The operator UI reads its views below its own ingress.
    let read = agenticspec_ui::interface::scope_url(
        &served.ingress_root("operator"),
        &op_plane.architecture,
        Some("gateway"),
        None,
    );
    assert!(
        read.starts_with(&format!("{url}/ingress/operator/scopes/")),
        "{read}"
    );
    let rows = get(
        &format!("{read}/api/views/agentic.gateway.Gateways"),
        Some(TOKEN),
    );
    assert_eq!(rows.status, 200, "{}", String::from_utf8_lossy(&rows.body));

    // R5 of each ingress lists exactly the commands of its interface.
    let r5 = |name: &str| -> Vec<String> {
        let at = agenticspec_ui::interface::scope_url(
            &served.ingress_root(name),
            &op_plane.architecture,
            None,
            None,
        );
        let list = json(&get(&format!("{at}/api/actions"), Some(TOKEN)));
        list.as_array()
            .unwrap()
            .iter()
            .map(|a| a["command"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        r5("user"),
        ["agentic.channel.SendMessage", "agentic.question.Answer"]
    );
    assert_eq!(
        r5("operator"),
        [
            "agentic.gateway.DetachUpstream",
            "agentic.gateway.DrainGateway",
            "agentic.gateway.StopGateway",
        ]
    );
    // The user's interface holds no view of the gateway; an ingress with no UI answers none.
    let user_root = served.ingress_root("user");
    let gateway = agenticspec_ui::interface::scope_url(
        &user_root,
        &user_plane.architecture,
        Some("gateway"),
        None,
    );
    let refused = get(
        &format!("{gateway}/api/views/agentic.gateway.Gateways"),
        Some(TOKEN),
    );
    assert_eq!(refused.status, 404);
    assert_eq!(json(&refused)["error"], "NoSuchView");
    let none = get(
        &format!("{url}/ingress/metrics/scopes/workstation-ingress/api/actions"),
        Some(TOKEN),
    );
    assert_eq!(none.status, 404);
    assert_eq!(json(&none)["error"], "NoSuchIngress");
    assert_eq!(
        get(
            &format!("{user_root}/scopes/workstation-ingress/api/actions"),
            None
        )
        .status,
        401
    );

    let options = || conform::Options {
        timeout: Duration::from_secs(5),
        idle: Duration::from_millis(200),
    };
    for (plane, at) in [
        (&user_plane, user_root.clone()),
        (&op_plane, served.interface_root()),
    ] {
        let report = conform::run_plane(&at, plane, Some(TOKEN), options());
        let failed: Vec<String> = report.failed().iter().map(ToString::to_string).collect();
        assert!(failed.is_empty(), "{at}: {failed:?}");
        assert!(report.count(Status::Passed) > 5, "{at}");
    }
}

/// With a simulator run (`ui serve --sim`) every ingress's plane answers from the one run, each
/// kept to its own views and feeds: a row of a view only the operator ingress exports is read
/// there and refused at the user's ingress; the level channel of each sends only the events of
/// its own feeds, with the log's `seq` as the cursor.
#[test]
fn a_replayed_run_answers_each_ingress_from_its_own_plane() {
    use agenticspec_ui::interface::Ordered;
    use agenticspec_ui::serve::{At, Replay, ReplayEvent, ReplayRow};
    let (op_plane, op_files) = ingress_ui("operator");
    let (user_plane, user_files) = ingress_ui("user");
    let gateways = "agentic.gateway.Gateways";
    assert!(op_plane.views.contains(gateways) && !user_plane.views.contains(gateways));
    assert!(!op_plane.feeds.is_empty() && !user_plane.feeds.is_empty());
    let op_only = op_plane
        .feeds
        .difference(&user_plane.feeds)
        .next()
        .expect("a feed only the operator ingress carries")
        .clone();
    let user_feed = user_plane
        .feeds
        .iter()
        .next()
        .expect("a feed the user ingress carries")
        .clone();
    assert!(op_plane.feeds.contains(&user_feed));
    let at = |part: &str| At {
        part: part.to_string(),
        instance: None,
    };
    let event = |seq: u64, name: &str, part: &str| ReplayEvent {
        seq,
        event: name.to_string(),
        payload: serde_json::json!({}),
        at: at(part),
    };
    let replay = Replay {
        views: [(
            gateways.to_string(),
            vec![ReplayRow {
                row: Ordered(vec![("id".into(), serde_json::json!("gateway"))]),
                at: vec![at("gateway")],
            }],
        )]
        .into(),
        events: vec![
            event(7, &op_only, "gateway"),
            event(9, &user_feed, "operator"),
        ],
        channels: Default::default(),
    };
    let served = Served::start_uis(
        vec![
            (op_plane.clone(), op_files),
            (user_plane.clone(), user_files),
        ],
        Vec::new(),
        Fixtures::replay(replay),
        serve::Options {
            port: 0,
            token: TOKEN.into(),
            faults: Faults::default(),
        },
    )
    .unwrap();
    let scope = |ingress: &str, plane: &Plane| {
        agenticspec_ui::interface::scope_url(
            &served.ingress_root(ingress),
            &plane.architecture,
            None,
            None,
        )
    };

    let read = get(
        &format!("{}/api/views/{gateways}", scope("operator", &op_plane)),
        Some(TOKEN),
    );
    assert_eq!(read.status, 200, "{}", String::from_utf8_lossy(&read.body));
    assert_eq!(json(&read)["rows"][0]["id"], "gateway");
    let refused = get(
        &format!("{}/api/views/{gateways}", scope("user", &user_plane)),
        Some(TOKEN),
    );
    assert_eq!(refused.status, 404);

    // The first `n` events of the level channel of an ingress, `(event, id)`, read within 5 s:
    // the stream stays open after them.
    let sent = |ingress: &str, plane: &Plane, n: usize| -> Vec<(String, String)> {
        use std::io::{BufRead, BufReader, Write};
        let url = format!(
            "{}/api/channels/{}",
            scope(ingress, plane),
            plane.channels[0]
        );
        let rest = url.strip_prefix("http://").unwrap();
        let (addr, path) = rest.split_at(rest.find('/').unwrap());
        let mut s = std::net::TcpStream::connect(addr).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let head = format!(
            "GET {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {TOKEN}\r\nAccept: text/event-stream\r\n\r\n"
        );
        s.write_all(head.as_bytes()).unwrap();
        let (mut out, mut event) = (Vec::new(), String::new());
        for line in BufReader::new(s).lines().map_while(Result::ok) {
            if let Some(v) = line.strip_prefix("event: ") {
                event = v.to_string();
            } else if let Some(v) = line.strip_prefix("id: ") {
                out.push((event.clone(), v.to_string()));
                if out.len() == n {
                    break;
                }
            }
        }
        out
    };
    // Ordered by `seq`, so the user's first event being 9 shows 7 was kept from it.
    assert_eq!(
        sent("operator", &op_plane, 2),
        [
            (op_only, "7".to_string()),
            (user_feed.clone(), "9".to_string())
        ]
    );
    assert_eq!(sent("user", &user_plane, 1), [(user_feed, "9".to_string())]);
}
