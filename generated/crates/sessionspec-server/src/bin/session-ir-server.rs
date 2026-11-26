// generated from sessionspec v1
// contract digest 7eaa4202b2be444349f069c24e3e8839a7d36b41fbc39fdf08c4042538b65d40
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
let ports = sessionspec_server::memory::MemoryPorts::default();
let mut system = sessionspec_system::System::new(session_ir::SessionIr::new(sessionspec_types::behaviour::Generated::new(ports.clone())), session_view::SessionView::new(sessionspec_types::behaviour::Generated::new(ports.clone())));
let mode = options.callers;
eprintln!("{}", if mode == "actor-header" { r#"{"format":"ess/1","callers":"actor-header","demonstration":true}"# } else { r#"{"format":"ess/1","callers":"none"}"# });
match sessionspec_server::session_ir::serve_with_static(&mut system, &options.listen, |request| { if mode != "actor-header" { return None; } let mut authorization = request.headers.iter().filter(|(key, _)| key.eq_ignore_ascii_case("Authorization")); let header = authorization.next()?.1.strip_prefix("Actor ")?; if authorization.next().is_some() { return None; } let actor = match header { "AgentLoop" => "sessionspec.session.AgentLoop",
"Delegate" => "sessionspec.session.Delegate",
"HarnessRuntime" => "sessionspec.session.HarnessRuntime",
"Human" => "sessionspec.session.Human",
"PeerSession" => "sessionspec.session.PeerSession",
"PermissionAuthority" => "sessionspec.session.PermissionAuthority",
"Redactor" => "sessionspec.session.Redactor",
"ToolProvider" => "sessionspec.session.ToolProvider",
"Viewer" => "sessionspec.view.Viewer",
 other => other }; sessionspec_types::actor::Actor::ALL.iter().find(|candidate| candidate.name() == actor).map(|actor| sessionspec_types::actor::Caller { actor: *actor }) }, options.static_root.as_deref()) { Ok(()) => std::process::ExitCode::SUCCESS, Err(error) => { eprintln!("{error}"); std::process::ExitCode::FAILURE } }
}
