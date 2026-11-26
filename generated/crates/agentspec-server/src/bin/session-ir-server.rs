// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! Ephemeral generated server. Durable storage and production authentication remain ports.
use clap::Parser;

#[derive(Parser)]
struct Options {
#[arg(long, default_value = "127.0.0.1:8080")]
listen: String,
#[arg(long, default_value = "none", value_parser = ["none", "actor-header"])]
callers: String,
#[arg(long = "static")]
static_root: Option<std::path::PathBuf>,
}

fn main() -> std::process::ExitCode {
let options = Options::parse();
let ports = agentspec_server::memory::MemoryPorts::default();
let mut system = agentspec_system::System::new(session_ir::SessionIr::new(agentspec_types::behaviour::Generated::new(ports.clone())), session_view::SessionView::new(agentspec_types::behaviour::Generated::new(ports.clone())));
let mode = options.callers;
eprintln!("{}", if mode == "actor-header" { r#"{"format":"ess/1","callers":"actor-header","demonstration":true}"# } else { r#"{"format":"ess/1","callers":"none"}"# });
match agentspec_server::session_ir::serve_with_static(&mut system, &options.listen, |request| { if mode != "actor-header" { return None; } let mut authorization = request.headers.iter().filter(|(key, _)| key.eq_ignore_ascii_case("Authorization")); let header = authorization.next()?.1.strip_prefix("Actor ")?; if authorization.next().is_some() { return None; } let actor = match header { "AgentLoop" => "agentspec.session.AgentLoop",
"Delegate" => "agentspec.session.Delegate",
"HarnessRuntime" => "agentspec.session.HarnessRuntime",
"Human" => "agentspec.session.Human",
"PeerSession" => "agentspec.session.PeerSession",
"PermissionAuthority" => "agentspec.session.PermissionAuthority",
"Redactor" => "agentspec.session.Redactor",
"ToolProvider" => "agentspec.session.ToolProvider",
"Viewer" => "agentspec.view.Viewer",
 other => other }; agentspec_types::actor::Actor::ALL.iter().find(|candidate| candidate.name() == actor).map(|actor| agentspec_types::actor::Caller { actor: *actor }) }, options.static_root.as_deref()) { Ok(()) => std::process::ExitCode::SUCCESS, Err(error) => { eprintln!("{error}"); std::process::ExitCode::FAILURE } }
}
