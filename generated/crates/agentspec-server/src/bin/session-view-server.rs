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
let _ = options;
eprintln!("unmet startup obligations:\nview query: agentspec.view.SessionEventCounts\nview query: agentspec.view.SessionEvents\nview query: agentspec.view.SessionList");
std::process::ExitCode::FAILURE
}
