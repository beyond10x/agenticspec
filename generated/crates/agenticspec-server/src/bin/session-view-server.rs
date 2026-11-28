// generated from agenticspec v1
// contract digest d9e4737ab51a3edd599c55c42bcc927804e20ba72238060de431753acc4d4ab1
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
eprintln!("unmet startup obligations:\nview query: agenticspec.view.SessionEventCounts\nview query: agenticspec.view.SessionEvents\nview query: agenticspec.view.SessionList");
std::process::ExitCode::FAILURE
}
