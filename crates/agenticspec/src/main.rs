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

struct Commands;

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
            "sim_check" => sim_reply(sim::check(&path_of(&inv.input, "architecture"))),
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
                match agenticspec_views::run(std::path::Path::new(root), check) {
                    Ok(result) => HandlerReply::Success(result),
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
                let (ports, out, check) = (path("ports"), path("out"), path("check"));
                sim_reply(sim::contract(&sim::ContractOptions {
                    architecture: &path_of(&inv.input, "architecture"),
                    ports: ports.as_deref(),
                    out: out.as_deref(),
                    check: check.as_deref(),
                }))
            }
            _ => HandlerReply::Unavailable,
        }
    }
}

/// A path field of the input; the parser has made every required one present.
fn path_of(input: &Value, field: &str) -> PathBuf {
    PathBuf::from(input.get(field).and_then(Value::as_str).unwrap_or(""))
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
    let output = cli_contract::run(args, &mut cli_contract::OsSources, &mut Commands, None);
    std::process::exit(emit(&output.stdout, &output.stderr, output.exit_code));
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
