//! `agenticstudio-mcp [--studio <url>] [--draft <id> | --open <source>]`: the agent channel of a
//! running `agenticstudio serve` as an MCP server on stdin and stdout.

use std::process::ExitCode;

use agenticstudio_mcp::{DEFAULT_STUDIO, Http, Server, Target};

const USAGE: &str = "usage: agenticstudio-mcp [--studio <url>] [--draft <id> | --open <source>]

An MCP server over stdio whose tools query, apply, check and simulate drive one draft of a
running `agenticstudio serve`. Each call is a tool call with caller Agent in a studio session,
and every client open on the draft sees each change live.

  --studio <url>     the studio (default http://127.0.0.1:7431)
  --draft <id>       the draft of this id
  --open <source>    a new draft of architectures/<name>.yaml, opened at the first call
                     (without --draft or --open: the draft opened last of those still open)";

fn parse(args: &[String]) -> Result<(String, Target), String> {
    let mut studio = DEFAULT_STUDIO.to_string();
    let mut target = Target::Newest;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let mut value = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--studio" => studio = value()?,
            "--draft" => target = Target::Draft(value()?),
            "--open" => target = Target::Open(value()?),
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("unknown flag {other}\n{USAGE}")),
        }
    }
    Ok((studio, target))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (studio, target) = match parse(&args) {
        Ok(a) => a,
        Err(m) => {
            eprintln!("{m}");
            return ExitCode::from(2);
        }
    };
    let http = match Http::parse(&studio) {
        Ok(h) => h,
        Err(m) => {
            eprintln!("agenticstudio-mcp: {m}");
            return ExitCode::from(2);
        }
    };
    let mut server = Server::new(http, target);
    let stdin = std::io::stdin();
    match server.run(stdin.lock(), std::io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("agenticstudio-mcp: {e}");
            ExitCode::from(1)
        }
    }
}
