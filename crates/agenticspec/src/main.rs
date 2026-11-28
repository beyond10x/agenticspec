//! The `agenticspec` command line. Parser, help and process adapter come from the generated
//! `agenticspec-cli-contract` (`ess generate cli` over `spec/cli.yaml`); this file supplies the
//! handler behind each command.

use std::ffi::OsString;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

use agenticspec::claude::{Actor, Cmd};
use agenticspec_server::wire;
use agenticspec_sim::cli as sim;
use cli_contract::{Handler, HandlerReply, Invocation};
use serde_json::{Value, json};

const VERSION_LINE: &str = concat!("agenticspec ", env!("CARGO_PKG_VERSION"), "\n");

/// The handler. `stdout` is what a command prints before the generated adapter's own output: the
/// document of `sim check --json` when the check refuses the architecture.
#[derive(Default)]
struct Commands {
    stdout: String,
}

fn refusal(code: &str, detail: String) -> HandlerReply {
    HandlerReply::Error {
        code: code.to_owned(),
        data: json!({ "detail": detail }),
    }
}

fn encoded(out: String) -> Value {
    serde_json::from_str(&out).unwrap_or(Value::Null)
}

fn command_json(cmd: &Cmd) -> Value {
    let mut out = String::new();
    match cmd {
        Cmd::OpenSession(i) => wire::encode_command_agenticspec_session_open_session(i, &mut out),
        Cmd::StartAgent(i) => wire::encode_command_agenticspec_session_start_agent(i, &mut out),
        Cmd::SubmitPrompt(i) => wire::encode_command_agenticspec_session_submit_prompt(i, &mut out),
        Cmd::EndTurn(i) => wire::encode_command_agenticspec_session_end_turn(i, &mut out),
        Cmd::RecordModelTurn(i) => {
            wire::encode_command_agenticspec_session_record_model_turn(i, &mut out)
        }
        Cmd::CompactContext(i) => {
            wire::encode_command_agenticspec_session_compact_context(i, &mut out)
        }
        Cmd::RequestToolCall(i) => {
            wire::encode_command_agenticspec_session_request_tool_call(i, &mut out)
        }
        Cmd::DecideToolCall(i) => {
            wire::encode_command_agenticspec_session_decide_tool_call(i, &mut out)
        }
        Cmd::CompleteToolCall(i) => {
            wire::encode_command_agenticspec_session_complete_tool_call(i, &mut out)
        }
        Cmd::RecordExposure(i) => {
            wire::encode_command_agenticspec_session_record_exposure(i, &mut out)
        }
    }
    encoded(out)
}

fn actor_name(a: Actor) -> &'static str {
    match a {
        Actor::HarnessRuntime => "HarnessRuntime",
        Actor::Human => "Human",
        Actor::Delegate => "Delegate",
        Actor::PeerSession => "PeerSession",
        Actor::AgentLoop => "AgentLoop",
        Actor::PermissionAuthority => "PermissionAuthority",
        Actor::ToolProvider => "ToolProvider",
        Actor::Redactor => "Redactor",
    }
}

/// The projects directory and the window `[now - since_hours, now]` in milliseconds.
fn window(input: &Value) -> (PathBuf, i64, i64) {
    let root = input
        .get("root")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".claude/projects")
        });
    let hours = input
        .get("since_hours")
        .and_then(Value::as_i64)
        .unwrap_or(72);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    (root, now - hours * 3_600_000, now)
}

impl Handler for Commands {
    fn call(&mut self, inv: &Invocation<'_>) -> HandlerReply {
        match inv.callable {
            "ui_generate" => {
                let s = |k: &str| inv.input.get(k).and_then(Value::as_str).unwrap_or("");
                let (path, out) = (s("path"), s("out"));
                let text = match std::fs::read_to_string(path) {
                    Ok(t) => t,
                    Err(e) => return refusal("document_unreadable", format!("{path}: {e}")),
                };
                let files = match agenticspec_ui::generate(&text) {
                    Ok(f) => f,
                    Err(refusals) => {
                        let lines: Vec<String> = refusals.iter().map(ToString::to_string).collect();
                        return refusal(
                            "document_refused",
                            format!("{path}:\n{}", lines.join("\n")),
                        );
                    }
                };
                if let Err(e) = agenticspec_ui::write(&files, std::path::Path::new(out)) {
                    return refusal("out_unwritable", format!("{out}: {e}"));
                }
                HandlerReply::Success(
                    json!({"out": out, "files": files.keys().collect::<Vec<_>>()}),
                )
            }
            "lift" => {
                let path = PathBuf::from(
                    inv.input
                        .get("transcript")
                        .and_then(Value::as_str)
                        .unwrap_or(""),
                );
                let lifted = match agenticspec::claude::lift(&path) {
                    Ok(l) => l,
                    Err(e) => {
                        return refusal(
                            "transcript_unreadable",
                            format!("{}: {e}", path.display()),
                        );
                    }
                };
                let replayed = agenticspec::replay::replay(&lifted.steps);
                let commands: Vec<Value> = lifted
                    .steps
                    .iter()
                    .zip(&replayed.log)
                    .enumerate()
                    .map(|(n, (step, logged))| {
                        json!({
                            "seq": n as i64,
                            "at": agenticspec::time::rfc3339(step.at_ms),
                            "actor": actor_name(step.actor),
                            "command": logged.command,
                            "outcome": logged.outcome,
                            "input": command_json(&step.cmd),
                        })
                    })
                    .collect();
                HandlerReply::Success(json!({
                    "session_id": lifted.session_id,
                    "agents": lifted.agents as i64,
                    "commands": commands,
                    "refused": replayed.refused as i64,
                }))
            }
            "analyze" => {
                let (root, from, now) = window(&inv.input);
                // Without --out the file goes beside the store, out of any checkout.
                let out = inv.input.get("out").and_then(Value::as_str).map_or_else(
                    || {
                        agenticspec::store::path_in(inv.context.state_dir.as_deref())
                            .with_file_name("agenticspec-dashboard.html")
                    },
                    PathBuf::from,
                );
                let store = match synced(&inv.context, &root, from) {
                    Ok(s) => s,
                    Err(r) => return r,
                };
                let load = |p: &std::path::Path| store.load(p);
                let dashboard = match agenticspec::kpi::dashboard_with(&root, from, now, now, &load)
                {
                    Ok(d) => d,
                    Err(e) => {
                        return refusal("root_unreadable", format!("{}: {e}", root.display()));
                    }
                };
                if let Err(e) = std::fs::write(&out, agenticspec::html::render(&dashboard)) {
                    return refusal("out_unwritable", format!("{}: {e}", out.display()));
                }
                let path = std::fs::canonicalize(&out).unwrap_or(out);
                HandlerReply::Success(json!({
                    "path": path.display().to_string(),
                    "sessions": dashboard.sessions.len() as i64,
                }))
            }
            "query" => {
                let mut src = String::new();
                if let Some(m) = inv.input.get("module").and_then(Value::as_str) {
                    match std::fs::read_to_string(m) {
                        Ok(t) => src.push_str(&t),
                        Err(e) => return refusal("module_invalid", format!("{m}: {e}")),
                    }
                }
                if let Some(t) = inv.input.get("text").and_then(Value::as_str) {
                    src.push('\n');
                    src.push_str(t);
                }
                let program = match agenticspec::sq::compile(&src) {
                    Ok(p) => p,
                    Err(e) => return refusal("module_invalid", e),
                };
                let results = if let Some(t) = inv.input.get("transcript").and_then(Value::as_str) {
                    agenticspec::sq::run_all(&program, &[PathBuf::from(t)])
                } else {
                    let (root, from, _) = window(&inv.input);
                    let store = match synced(&inv.context, &root, from) {
                        Ok(s) => s,
                        Err(r) => return r,
                    };
                    let mains: Vec<PathBuf> = match agenticspec::kpi::sessions_in(&root, from) {
                        Ok(v) => v.into_iter().map(|(p, _)| p).collect(),
                        Err(e) => {
                            return refusal("root_unreadable", format!("{}: {e}", root.display()));
                        }
                    };
                    let load = |p: &std::path::Path| store.load(p);
                    agenticspec::sq::run_all_with(&program, &mains, &load)
                };
                let report = agenticspec::sq::report(&program, &results);
                let mut out = String::new();
                wire::encode_agenticspec_cli_query_report(&report, &mut out);
                HandlerReply::Success(encoded(out))
            }
            "mine" | "procedure" => {
                use agenticspec::mine;
                let level = match mine::Level::parse(inv.input.get("level").and_then(Value::as_str))
                {
                    Ok(l) => l,
                    Err(e) => return refusal("root_unreadable", e),
                };
                let (root, from, _) = window(&inv.input);
                let store = match synced(&inv.context, &root, from) {
                    Ok(s) => s,
                    Err(r) => return r,
                };
                let mains: Vec<PathBuf> = match agenticspec::kpi::sessions_in(&root, from) {
                    Ok(v) => v.into_iter().map(|(p, _)| p).collect(),
                    Err(e) => {
                        return refusal("root_unreadable", format!("{}: {e}", root.display()));
                    }
                };
                let load = |p: &std::path::Path| store.load(p);
                let bases = mine::corpus_with(&mains, level, &load);
                let int = |k: &str, d: i64| inv.input.get(k).and_then(Value::as_i64).unwrap_or(d);
                if inv.callable == "mine" {
                    let report = mine::mine(
                        &bases,
                        level,
                        int("min_sessions", 5).max(1) as usize,
                        int("min_projects", 2).max(1) as usize,
                        int("top", 50).max(1) as usize,
                    );
                    let mut out = String::new();
                    wire::encode_agenticspec_cli_mine_report(&report, &mut out);
                    return HandlerReply::Success(encoded(out));
                }
                let pattern = inv
                    .input
                    .get("pattern")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let tokens: Vec<String> = if pattern.starts_with("p-") {
                    let all = mine::mine(&bases, level, 1, 1, usize::MAX);
                    match all.patterns.into_iter().find(|p| p.id == pattern) {
                        Some(p) => p.tokens,
                        None => {
                            return refusal(
                                "no_match",
                                format!("no pattern {pattern} at level {}", level.name()),
                            );
                        }
                    }
                } else {
                    pattern.split_whitespace().map(str::to_owned).collect()
                };
                let Some(proc_) = mine::procedure(&bases, level, &tokens) else {
                    return refusal("no_match", format!("no turn holds `{}`", tokens.join(" ")));
                };
                let mut out = String::new();
                wire::encode_agenticspec_cli_procedure(&proc_, &mut out);
                if let Some(path) = inv.input.get("out").and_then(Value::as_str)
                    && let Err(e) = std::fs::write(path, &out)
                {
                    return refusal("out_unwritable", format!("{path}: {e}"));
                }
                HandlerReply::Success(encoded(out))
            }
            "export" => {
                let path = inv
                    .input
                    .get("procedure")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let out =
                    PathBuf::from(inv.input.get("out").and_then(Value::as_str).unwrap_or("."));
                let text = match std::fs::read_to_string(path) {
                    Ok(t) => t,
                    Err(e) => return refusal("procedure_invalid", format!("{path}: {e}")),
                };
                let value: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(e) => return refusal("procedure_invalid", format!("{path}: {e}")),
                };
                // A file `procedure --out` wrote, or the printed envelope `{ok, result}`.
                let doc = value.get("result").cloned().unwrap_or(value);
                let parsed = match agenticspec_server::json::parse(&doc.to_string()) {
                    Ok(v) => v,
                    Err(e) => return refusal("procedure_invalid", format!("{path}: {e:?}")),
                };
                let proc_ = match wire::decode_agenticspec_cli_procedure(&parsed, "$") {
                    Ok(p) => p,
                    Err(e) => return refusal("procedure_invalid", format!("{path}: {e:?}")),
                };
                match agenticspec::export::export(&proc_, &out) {
                    Ok(r) => {
                        let mut o = String::new();
                        wire::encode_agenticspec_cli_export_result(&r, &mut o);
                        HandlerReply::Success(encoded(o))
                    }
                    Err(e) => refusal("out_unwritable", e),
                }
            }
            "kpi" => {
                let (root, from, now) = window(&inv.input);
                let store = match synced(&inv.context, &root, from) {
                    Ok(s) => s,
                    Err(r) => return r,
                };
                let load = |p: &std::path::Path| store.load(p);
                match agenticspec::kpi::run_with(&root, from, now, &load) {
                    Ok(report) => {
                        let mut out = String::new();
                        wire::encode_agenticspec_cli_kpi_report(&report, &mut out);
                        HandlerReply::Success(encoded(out))
                    }
                    Err(e) => refusal("root_unreadable", format!("{}: {e}", root.display())),
                }
            }
            "sync" => {
                let (root, from, _) = window(&inv.input);
                let store = match open_store(&inv.context) {
                    Ok(s) => s,
                    Err(r) => return r,
                };
                match store.sync(&root, from) {
                    Ok(done) => {
                        let mut out = String::new();
                        wire::encode_agenticspec_cli_sync_result(&done.result, &mut out);
                        HandlerReply::Success(encoded(out))
                    }
                    Err(e) => refusal("root_unreadable", e),
                }
            }
            "serve" => {
                let (root, _, _) = window(&inv.input);
                let hours = inv
                    .input
                    .get("since_hours")
                    .and_then(Value::as_i64)
                    .unwrap_or(72);
                let port = inv
                    .input
                    .get("port")
                    .and_then(Value::as_i64)
                    .and_then(|p| u16::try_from(p).ok())
                    .unwrap_or(agenticspec::serve::PORT);
                let path = agenticspec::store::path_in(inv.context.state_dir.as_deref());
                let syncer = agenticspec::serve::Syncer {
                    exe: std::env::current_exe().unwrap_or_else(|_| "agenticspec".into()),
                    state_dir: path
                        .parent()
                        .map(std::path::Path::to_path_buf)
                        .unwrap_or_default(),
                    root,
                    since_hours: hours,
                };
                // The first sync runs in a child too, before the server opens the store: an
                // upgrade or a first fill lifts every session, and that memory is the child's.
                syncer.sync_once();
                let store = match open_store(&inv.context) {
                    Ok(s) => std::sync::Arc::new(s),
                    Err(r) => return r,
                };
                let listener = match agenticspec::serve::bind(port) {
                    Ok(l) => l,
                    Err(e) => return refusal("port_unavailable", format!("127.0.0.1:{port}: {e}")),
                };
                let state = store
                    .path()
                    .parent()
                    .map(std::path::Path::to_path_buf)
                    .unwrap_or_default();
                let ui = agenticspec::serve::ui_dir(&state);
                let url = format!("http://127.0.0.1:{port}/");
                let mut err = std::io::stderr().lock();
                let _ = writeln!(
                    err,
                    "agenticspec: serving {url} from {} (UI: {})",
                    store.path().display(),
                    ui.as_ref()
                        .map_or("not built; run `task ui:build`".to_owned(), |d| d
                            .display()
                            .to_string())
                );
                drop(err);
                let shared = std::sync::Arc::new(agenticspec::serve::Shared {
                    system: std::sync::Mutex::new(agenticspec::serve::system(
                        std::sync::Arc::clone(&store),
                    )),
                    hub: agenticspec::serve::Hub::default(),
                    store,
                    ui,
                });
                agenticspec::serve::run(listener, shared, syncer);
                HandlerReply::Success(json!({ "url": url }))
            }
            "sim_check" => {
                let path = |field: &str| {
                    inv.input
                        .get(field)
                        .and_then(Value::as_str)
                        .map(PathBuf::from)
                };
                let (ports, kpis) = (path("ports"), path("kpis"));
                let architecture = path_of(&inv.input, "architecture");
                let options = sim::CheckOptions {
                    architecture: &architecture,
                    ports: ports.as_deref(),
                    kpis: kpis.as_deref(),
                };
                if !inv
                    .input
                    .get("json")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    return sim_reply(sim::check_with(&options));
                }
                match sim::check_json(&options) {
                    Ok(sim::Checked {
                        document,
                        refusal: None,
                    }) => HandlerReply::Success(document),
                    Ok(sim::Checked {
                        document,
                        refusal: Some(r),
                    }) => {
                        self.stdout = format!("{document}\n");
                        refusal(r.code, r.detail)
                    }
                    Err(r) => refusal(r.code, r.detail),
                }
            }
            "sim_digest" => sim_reply(sim::digest(&path_of(&inv.input, "architecture"))),
            "sim_run" => {
                let threads = inv
                    .input
                    .get("threads")
                    .and_then(Value::as_i64)
                    .and_then(|t| usize::try_from(t).ok());
                let mut err = std::io::stderr().lock();
                sim_reply(sim::run(
                    &path_of(&inv.input, "experiment"),
                    &path_of(&inv.input, "out"),
                    threads,
                    inv.input
                        .get("events")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    &mut err,
                ))
            }
            "sim_render" => sim_reply(sim::render(
                &path_of(&inv.input, "architecture"),
                inv.input
                    .get("result")
                    .and_then(Value::as_str)
                    .map(std::path::Path::new),
                &path_of(&inv.input, "out"),
            )),
            "sim_scenario" => sim_reply(sim::scenario(
                inv.input
                    .get("action")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                &path_of(&inv.input, "scenario"),
                inv.input
                    .get("out")
                    .and_then(Value::as_str)
                    .map(std::path::Path::new),
            )),
            "sim_corpus" => {
                let mut err = std::io::stderr().lock();
                sim_reply(sim::corpus(
                    inv.input
                        .get("action")
                        .and_then(Value::as_str)
                        .unwrap_or(""),
                    &path_of(&inv.input, "corpus"),
                    &mut err,
                ))
            }
            "blocks_lift" => {
                draft_reply(agenticspec::draft::lift(&path_of(&inv.input, "transcript")))
            }
            "blocks_scenario" => draft_reply(agenticspec::draft::draft(&inv.input)),
            "sim_ladder" => sim_reply(sim::ladder(
                &path_of(&inv.input, "ladder"),
                &path_of(&inv.input, "out"),
                inv.input
                    .get("check")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            )),
            "ui_panels" => {
                let root = inv.input.get("root").and_then(Value::as_str).unwrap_or(".");
                let check = inv
                    .input
                    .get("check")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let root = std::path::Path::new(root);
                match agenticspec_views::run(root, check) {
                    Ok(result) => match agenticspec_ui::serve::write_bundle(root, check) {
                        Ok(()) => HandlerReply::Success(result),
                        Err(e) if check => refusal("panels_stale", e),
                        Err(e) => refusal("out_unwritable", e),
                    },
                    Err(r) => refusal(r.code, r.detail),
                }
            }
            "sim_ports" => {
                let path = |field: &str| {
                    inv.input
                        .get(field)
                        .and_then(Value::as_str)
                        .map(PathBuf::from)
                };
                let (blocks, out, check, expect) =
                    (path("blocks"), path("out"), path("check"), path("expect"));
                let mut err = std::io::stderr().lock();
                sim_reply(sim::ports(
                    &sim::PortsOptions {
                        blocks: blocks.as_deref(),
                        out: out.as_deref(),
                        check: check.as_deref(),
                        expect: expect.as_deref(),
                    },
                    &mut err,
                ))
            }
            "sim_contract" => {
                let path = |field: &str| {
                    inv.input
                        .get(field)
                        .and_then(Value::as_str)
                        .map(PathBuf::from)
                };
                let (ports, out, check, kpis) =
                    (path("ports"), path("out"), path("check"), path("kpis"));
                let (suite, trace) = (path("suite"), path("trace"));
                let mut err = std::io::stderr().lock();
                sim_reply(sim::contract(
                    &sim::ContractOptions {
                        architecture: &path_of(&inv.input, "architecture"),
                        ports: ports.as_deref(),
                        out: out.as_deref(),
                        check: check.as_deref(),
                        suite: suite.as_deref(),
                        trace: trace.as_deref(),
                        kpis: kpis.as_deref(),
                    },
                    &mut err,
                ))
            }
            "sim_trace" => {
                let out = inv
                    .input
                    .get("out")
                    .and_then(Value::as_str)
                    .map(PathBuf::from);
                sim_reply(sim::trace(&sim::TraceOptions {
                    architecture: &path_of(&inv.input, "architecture"),
                    profile: &path_of(&inv.input, "profile"),
                    seed: inv
                        .input
                        .get("seed")
                        .and_then(Value::as_i64)
                        .and_then(|s| u64::try_from(s).ok()),
                    horizon: inv.input.get("horizon").and_then(Value::as_str),
                    out: out.as_deref(),
                }))
            }
            "ui_conform" => ui_conform(&inv.input),
            "packing_default" => sim_reply(sim::packing_default(
                &path_of(&inv.input, "architecture"),
                inv.input.get("strategy").and_then(Value::as_str),
            )),
            "deploy_bundle" => {
                let path = |field: &str| {
                    inv.input
                        .get(field)
                        .and_then(Value::as_str)
                        .map(PathBuf::from)
                };
                let (out, ports, accepts, kpis) =
                    (path("out"), path("ports"), path("accepts"), path("kpis"));
                sim_reply(sim::deploy_bundle(&sim::BundleOptions {
                    architecture: &path_of(&inv.input, "architecture"),
                    binding: &path_of(&inv.input, "binding"),
                    out: out.as_deref(),
                    ports: ports.as_deref(),
                    accepts: accepts.as_deref(),
                    kpis: kpis.as_deref(),
                }))
            }
            "ingress_default" => {
                sim_reply(sim::ingress_default(&path_of(&inv.input, "architecture")))
            }
            "ingress_project" => {
                let path = |field: &str| {
                    inv.input
                        .get(field)
                        .and_then(Value::as_str)
                        .map(PathBuf::from)
                };
                let (out, check, blocks, ports, kpis) = (
                    path("out"),
                    path("check"),
                    path("blocks"),
                    path("ports"),
                    path("kpis"),
                );
                sim_reply(sim::ingress_project(&sim::ProjectOptions {
                    architecture: &path_of(&inv.input, "architecture"),
                    out: out.as_deref(),
                    check: check.as_deref(),
                    blocks: blocks.as_deref(),
                    ports: ports.as_deref(),
                    kpis: kpis.as_deref(),
                }))
            }
            "ui_derive" => ui_derive(&inv.input),
            "ui_build" => ui_build(&inv.input),
            "ui_serve" => ui_serve(&inv.input),
            "deploy_check" => {
                let accepts = inv
                    .input
                    .get("accepts")
                    .and_then(Value::as_str)
                    .map(PathBuf::from);
                sim_reply(sim::deploy_check(
                    &path_of(&inv.input, "bundle"),
                    accepts.as_deref(),
                ))
            }
            _ => HandlerReply::Unavailable,
        }
    }
}

/// `agenticspec ui conform`: the conformance suite of the runtime interface against `--source`,
/// or against the fixture source started here when there is none.
fn ui_conform(input: &Value) -> HandlerReply {
    use agenticspec_ui::{conform, source};
    let s = |k: &str| input.get(k).and_then(Value::as_str);
    if s("architecture").is_some() {
        return ui_conform_plane(input);
    }
    let root = std::path::Path::new(s("root").unwrap_or("."));
    let plan_path = s("plan")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(source::PLAN_FILE));
    let plan = match std::fs::read_to_string(&plan_path)
        .map_err(|e| e.to_string())
        .and_then(|t| conform::Plan::parse(&t))
    {
        Ok(p) => p,
        Err(e) => return refusal("plan_unreadable", format!("{}: {e}", plan_path.display())),
    };
    let token = std::env::var("AGENTICSPEC_UI_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| plan.token.clone());
    let fixture;
    let url = match s("source") {
        Some(u) => u.trim_end_matches('/').to_string(),
        None => {
            let started = source::Data::load(root)
                .map_err(|e| e.to_string())
                .and_then(|d| {
                    source::FixtureSource::start(d, source::Faults::default())
                        .map_err(|e| e.to_string())
                });
            fixture = match started {
                Ok(f) => f,
                Err(e) => return refusal("source_unavailable", e),
            };
            fixture.url().to_string()
        }
    };
    let report = conform::run(&url, &plan, token.as_deref(), conform::Options::default());
    let failed = report.failed();
    if !failed.is_empty() {
        let lines: Vec<String> = failed.iter().map(ToString::to_string).collect();
        return refusal(
            "conformance_failed",
            format!(
                "{url}: {} of {} checks failed\n{}",
                failed.len(),
                report.checks.len(),
                lines.join("\n")
            ),
        );
    }
    HandlerReply::Success(report.to_json())
}

/// The files of an operator UI by name.
type OperatorFiles = Vec<(String, Vec<u8>)>;

/// The files of the operator UI of `architecture` with `api_base`: from the contract at
/// `contract`, or from the contract derived here from the architecture with the port and KPI
/// catalogues built into the binary (`sim contract`), so no file of a checkout is read.
fn operator_files(
    architecture: &std::path::Path,
    contract: Option<&str>,
    api_base: &str,
) -> Result<OperatorFiles, HandlerReply> {
    use agenticspec_ui::derive::Failure;
    use agenticspec_ui::operator;
    let failed = |f: Failure| match f {
        Failure::Unreadable(e) => refusal("document_unreadable", e),
        Failure::Refused(rs) => {
            let lines: Vec<String> = rs.iter().map(ToString::to_string).collect();
            refusal(
                "document_refused",
                format!("{}:\n{}", architecture.display(), lines.join("\n")),
            )
        }
    };
    let (doc, shown) = match contract {
        Some(path) => (
            operator::read_doc(std::path::Path::new(path)).map_err(failed)?,
            path.to_string(),
        ),
        None => {
            let text = sim::contract_built_in(architecture).map_err(|r| {
                let code = if r.code == "document_unreadable" {
                    "document_unreadable"
                } else {
                    "document_refused"
                };
                refusal(code, format!("{}: {}", r.code, r.detail))
            })?;
            (
                operator::parse_contract(&text).map_err(failed)?,
                format!("the contract derived from {}", architecture.display()),
            )
        }
    };
    operator::build_with(
        std::path::Path::new("."),
        architecture,
        &doc,
        &shown,
        api_base,
    )
    .map_err(failed)
}

/// The control plane of an architecture: the files of its operator UI with the API base of
/// `ui serve`, and the plane read from them.
fn plane_of(input: &Value) -> Result<(OperatorFiles, agenticspec_ui::plane::Plane), HandlerReply> {
    let architecture = path_of(input, "architecture");
    let contract = input.get("contract").and_then(Value::as_str);
    let files = operator_files(&architecture, contract, agenticspec_ui::serve::ROOT)?;
    let plane = agenticspec_ui::plane::Plane::from_files(&files)
        .map_err(|e| refusal("document_refused", e))?;
    Ok((files, plane))
}

/// The token of `ui serve` and `ui conform`: `AGENTICSPEC_UI_TOKEN` when set.
fn ui_token() -> Option<String> {
    std::env::var("AGENTICSPEC_UI_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
}

/// `agenticspec ui conform --architecture`: the checks of the default control plane of the
/// architecture against `--source`, or against `ui serve` of it started here, on the fixtures
/// built into the binary, or on those of `--root` when it is named.
fn ui_conform_plane(input: &Value) -> HandlerReply {
    use agenticspec_ui::{conform, serve};
    let s = |k: &str| input.get(k).and_then(Value::as_str);
    if s("plan").is_some() {
        return refusal(
            "plan_unreadable",
            "--plan and --architecture name two plans; name one".to_string(),
        );
    }
    let (files, plane) = match plane_of(input) {
        Ok(x) => x,
        Err(r) => return r,
    };
    let mut token = ui_token();
    let served;
    let url = match s("source") {
        Some(u) => u.trim_end_matches('/').to_string(),
        None => {
            let t = token.clone().unwrap_or_else(serve::random_token);
            let fixtures = match s("root") {
                Some(root) => serve::Fixtures::load(std::path::Path::new(root)),
                None => Ok(serve::Fixtures::embedded()),
            };
            let started = fixtures.and_then(|f| {
                serve::Served::start(
                    plane.clone(),
                    files,
                    f,
                    serve::Options {
                        port: 0,
                        token: t.clone(),
                        faults: serve::Faults::default(),
                    },
                )
                .map_err(|e| e.to_string())
            });
            served = match started {
                Ok(x) => x,
                Err(e) => return refusal("source_unavailable", e),
            };
            token = Some(t);
            served.interface_root()
        }
    };
    let report = conform::run_plane(&url, &plane, token.as_deref(), conform::Options::default());
    let failed = report.failed();
    if !failed.is_empty() {
        let lines: Vec<String> = failed.iter().map(ToString::to_string).collect();
        return refusal(
            "conformance_failed",
            format!(
                "{url}: {} of {} checks failed\n{}",
                failed.len(),
                report.checks.len(),
                lines.join("\n")
            ),
        );
    }
    HandlerReply::Success(report.to_json())
}

/// `agenticspec ui serve`: the operator UI of an architecture and its default control plane on
/// fixtures (those built into the binary, or a directory with `--fixtures`), on 127.0.0.1, until
/// the process is stopped.
fn ui_serve(input: &Value) -> HandlerReply {
    use agenticspec_ui::serve;
    let (files, plane) = match plane_of(input) {
        Ok(x) => x,
        Err(r) => return r,
    };
    let fixtures = match input.get("fixtures").and_then(Value::as_str) {
        None => serve::Fixtures::embedded(),
        Some(dir) => match serve::Fixtures::from_dir(std::path::Path::new(dir)) {
            Ok(f) => f,
            Err(e) => return refusal("fixtures_unreadable", e),
        },
    };
    let port = input
        .get("port")
        .and_then(Value::as_i64)
        .and_then(|p| u16::try_from(p).ok())
        .unwrap_or(serve::PORT);
    let given = ui_token();
    let token = given.clone().unwrap_or_else(serve::random_token);
    let architecture = plane.architecture.clone();
    let served = match serve::Served::start(
        plane,
        files,
        fixtures,
        serve::Options {
            port,
            token: token.clone(),
            faults: serve::Faults::default(),
        },
    ) {
        Ok(s) => s,
        Err(e) => return refusal("port_unavailable", format!("127.0.0.1:{port}: {e}")),
    };
    let url = format!("{}/", served.url());
    let interface = served.interface_root();
    let mut err = std::io::stderr().lock();
    let _ = writeln!(
        err,
        "agenticspec: serving {url} (the operator UI of {architecture} on fixtures; the runtime interface at {interface})"
    );
    if given.is_none() {
        let _ = writeln!(
            err,
            "agenticspec: token {token} (set AGENTICSPEC_UI_TOKEN to choose it; the page sets it as its cookie)"
        );
    }
    drop(err);
    served.wait();
    HandlerReply::Success(json!({ "url": url, "interface": interface }))
}

/// `agenticspec ui derive`: derive, then write the files or with `--check` compare them.
fn ui_derive(input: &Value) -> HandlerReply {
    use agenticspec_ui::derive::{Failure, Inputs, differences, write};
    let opt = |k: &str| input.get(k).and_then(Value::as_str).map(PathBuf::from);
    let root = opt("root").unwrap_or_else(|| PathBuf::from("."));
    let inputs = Inputs {
        architecture: path_of(input, "path"),
        contract: opt("contract"),
        ports: opt("ports"),
        out: path_of(input, "out"),
        root,
    };
    let check = input.get("check").and_then(Value::as_bool).unwrap_or(false);
    let d = match agenticspec_ui::derive_ui(&inputs) {
        Ok(d) => d,
        Err(Failure::Unreadable(e)) => return refusal("document_unreadable", e),
        Err(Failure::Refused(rs)) => {
            let lines: Vec<String> = rs.iter().map(ToString::to_string).collect();
            return refusal(
                "document_refused",
                format!("{}:\n{}", inputs.architecture.display(), lines.join("\n")),
            );
        }
    };
    let answer = |written: bool| {
        json!({
            "out": inputs.out.to_string_lossy(),
            "pages": d.pages,
            "actions": d.actions,
            "channels": d.channels,
            "files": d.files.len(),
            "written": written,
        })
    };
    if check {
        let diff = differences(&d);
        if diff.is_empty() {
            return HandlerReply::Success(answer(false));
        }
        return refusal(
            "derived_stale",
            format!(
                "{}. A derived file is never edited: derive it again",
                diff.join("; ")
            ),
        );
    }
    if let Err(e) = write(&d) {
        return refusal("out_unwritable", format!("{}: {e}", inputs.out.display()));
    }
    HandlerReply::Success(answer(true))
}

/// A path field of the input; the parser has made every required one present.
/// `agenticspec ui build`: the embedded operator app with the data files of one architecture,
/// and with `--archive` their reproducible tar and its sha256.
fn ui_build(input: &Value) -> HandlerReply {
    use agenticspec_ui::derive::Failure;
    use agenticspec_ui::operator::{self, API_BASE, Inputs};
    let architecture = path_of(input, "architecture");
    let out = path_of(input, "out");
    let inputs = Inputs {
        root: PathBuf::from("."),
        architecture: architecture.clone(),
        contract: path_of(input, "contract"),
        api_base: input
            .get("api_base")
            .and_then(Value::as_str)
            .unwrap_or(API_BASE)
            .to_string(),
    };
    let files = match operator::build(&inputs) {
        Ok(f) => f,
        Err(Failure::Unreadable(e)) => return refusal("document_unreadable", e),
        Err(Failure::Refused(rs)) => {
            let lines: Vec<String> = rs.iter().map(ToString::to_string).collect();
            return refusal(
                "document_refused",
                format!("{}:\n{}", architecture.display(), lines.join("\n")),
            );
        }
    };
    if let Err(e) = operator::write(&files, &out) {
        return refusal("out_unwritable", format!("{}: {e}", out.display()));
    }
    let mut answer = json!({
        "out": out.to_string_lossy(),
        "files": files.len(),
        "bytes": files.iter().map(|(_, b)| b.len()).sum::<usize>(),
    });
    if input
        .get("archive")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let tar = match operator::archive(&files) {
            Ok(t) => t,
            Err(e) => return refusal("out_unwritable", e),
        };
        let path = operator::archive_path(&out);
        if let Err(e) = std::fs::write(&path, &tar) {
            return refusal("out_unwritable", format!("{}: {e}", path.display()));
        }
        answer["archive"] = json!(path.to_string_lossy());
        answer["archive_sha256"] = json!(operator::sha256(&tar));
    }
    HandlerReply::Success(answer)
}

fn path_of(input: &Value, field: &str) -> PathBuf {
    PathBuf::from(input.get(field).and_then(Value::as_str).unwrap_or(""))
}

/// The reply of an `agenticspec blocks` command.
fn draft_reply(outcome: Result<Value, agenticspec::draft::Refused>) -> HandlerReply {
    match outcome {
        Ok(result) => HandlerReply::Success(result),
        Err((code, detail)) => refusal(code, detail),
    }
}

/// The reply of an `agenticspec sim` command.
fn sim_reply(outcome: Result<Value, sim::Refusal>) -> HandlerReply {
    match outcome {
        Ok(result) => HandlerReply::Success(result),
        Err(r) => refusal(r.code, r.detail),
    }
}

fn open_store(ctx: &cli_contract::Context) -> Result<agenticspec::store::Store, HandlerReply> {
    let path = agenticspec::store::path_in(ctx.state_dir.as_deref());
    agenticspec::store::Store::open(&path)
        .map_err(|e| refusal("store_unavailable", format!("{}: {e}", path.display())))
}

/// The store, synced over the window first: readers read what the last sync wrote.
fn synced(
    ctx: &cli_contract::Context,
    root: &std::path::Path,
    from: i64,
) -> Result<agenticspec::store::Store, HandlerReply> {
    let store = open_store(ctx)?;
    store
        .sync(root, from)
        .map_err(|e| refusal("root_unreadable", format!("{}: {e}", root.display())))?;
    Ok(store)
}

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    if args.len() == 2 && args[1] == "--version" {
        std::process::exit(emit(VERSION_LINE, "", 0));
    }
    let mut commands = Commands::default();
    let output = cli_contract::run(args, &mut cli_contract::OsSources, &mut commands, None);
    let stdout = commands.stdout + &output.stdout;
    std::process::exit(emit(&stdout, &output.stderr, output.exit_code));
}

fn emit(stdout: &str, stderr: &str, exit_code: i32) -> i32 {
    let mut out = std::io::stdout().lock();
    let written = out.write_all(stdout.as_bytes()).and_then(|()| out.flush());
    let mut err = std::io::stderr().lock();
    let _ = err.write_all(stderr.as_bytes());
    match written {
        Ok(()) => exit_code,
        Err(error) if error.kind() == ErrorKind::BrokenPipe => exit_code,
        Err(error) => {
            let _ = writeln!(err, "agenticspec: standard output: {error}");
            if exit_code == 0 { 1 } else { exit_code }
        }
    }
}
