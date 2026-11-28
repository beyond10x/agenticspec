//! `agenticstudio serve --root <dir> --port <n> --www <dir>`: the studio over the architecture
//! documents of `<dir>/architectures`, on 127.0.0.1 only, with the built app of `ui/studio` from
//! the www directory. Port 0 takes a free port; the first line printed names the address.

use std::io::Write as _;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use agenticstudio::http;
use agenticstudio::routes::serve;
use agenticstudio::studio::Studio;

const USAGE: &str = "usage: agenticstudio serve [--root <dir>] [--port <n>] [--www <dir>]

  --root <dir>   the directory whose architectures/ holds the documents a draft opens (default .)
  --port <n>     the port on 127.0.0.1 (default 7431; 0 takes a free port)
  --www <dir>    the built app of ui/studio (default <root>/target/studio/www; `task studio:build`)";

struct Args {
    root: PathBuf,
    port: u16,
    www: Option<PathBuf>,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut it = args.iter();
    match it.next().map(String::as_str) {
        Some("serve") => {}
        Some("-h" | "--help") | None => return Err(USAGE.into()),
        Some(other) => return Err(format!("unknown command {other}\n{USAGE}")),
    }
    let mut a = Args {
        root: PathBuf::from("."),
        port: 7431,
        www: None,
    };
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--root" => a.root = PathBuf::from(value()?),
            "--www" => a.www = Some(PathBuf::from(value()?)),
            "--port" => {
                let v = value()?;
                a.port = v.parse().map_err(|_| format!("--port {v} is no port"))?;
            }
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("unknown flag {other}\n{USAGE}")),
        }
    }
    Ok(a)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a = match parse(&args) {
        Ok(a) => a,
        Err(m) => {
            eprintln!("{m}");
            return ExitCode::from(2);
        }
    };
    if !a.root.join("architectures").is_dir() {
        eprintln!(
            "agenticstudio: {} has no architectures/ directory",
            a.root.display()
        );
        return ExitCode::from(2);
    }
    let listener = match TcpListener::bind(("127.0.0.1", a.port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("agenticstudio: cannot listen on 127.0.0.1:{}: {e}", a.port);
            return ExitCode::from(1);
        }
    };
    let port = listener.local_addr().map_or(a.port, |s| s.port());
    println!("agenticstudio: serving http://127.0.0.1:{port}/");
    let _ = std::io::stdout().flush();
    let www = Arc::new(
        a.www
            .clone()
            .unwrap_or_else(|| a.root.join("target/studio/www")),
    );
    let studio = Arc::new(Mutex::new(Studio::new(a.root)));
    for conn in listener.incoming() {
        let Ok(mut stream) = conn else { continue };
        let studio = Arc::clone(&studio);
        let www = Arc::clone(&www);
        std::thread::spawn(move || {
            let _ = stream.set_read_timeout(Some(http::READ_TIMEOUT));
            let _ = stream.set_write_timeout(Some(http::WRITE_TIMEOUT));
            let response = match http::read(&stream) {
                Ok(request) => serve(&studio, &www, &request),
                Err(e) => http::Response::new(400, http::TEXT, format!("{e}\n")),
            };
            let _ = http::write(&mut stream, &response);
        });
    }
    ExitCode::SUCCESS
}
