//! The routes the generated app (`ui/studio`, from `studio/studio.ui.yaml`) calls, and the files
//! of the built app.
//!
//! | method | path | answer |
//! |---|---|---|
//! | GET | `/api/views/<view>?<params>` | `{"rows": [...]}` of one view of the document |
//! | POST | `/api/commands/<command>` | `{"row": ...}`; a refused patch is status 422 with its row |
//! | GET | any other path | a file of the www directory; `/` is its `index.html` |
//!
//! The views beside `studio.draft.Drafts` are placeholders of the document until the studio
//! model holds them; their rows are the fields the document reads.

use std::path::{Component, Path};
use std::sync::Mutex;

use agenticspec_patch::{Change, Patch, Refused};
use agenticspec_sim::check::{self, Refusal};
use serde_json::{Value, json};

use crate::http::{HTML, Request, Response, TEXT};
use crate::routes::{error, lock, studio_error, to_json, val};
use crate::studio::{Draft, Studio};

/// The value of `key` in a query string, percent-decoded.
fn param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        (decode(k) == key).then(|| decode(v))
    })
}

/// `%xx` and `+` decoded; a broken escape is kept as written.
fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                let hex = |c: u8| (c as char).to_digit(16);
                match (hex(b[i + 1]), hex(b[i + 2])) {
                    (Some(h), Some(l)) => {
                        out.push((h * 16 + l) as u8);
                        i += 2;
                    }
                    _ => out.push(b'%'),
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn rows(rows: Vec<Value>) -> Response {
    Response::json(200, &json!({ "rows": rows }))
}

fn draft_param<'a>(studio: &'a Studio, request: &Request) -> Result<&'a Draft, Response> {
    let id = param(&request.query, "draft_id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| error(400, "InvalidRequest", "this view takes draft_id"))?;
    studio.draft(&id).map_err(|e| studio_error(&e))
}

fn refusal_rows(refusals: &[Refusal]) -> Vec<Value> {
    refusals
        .iter()
        .enumerate()
        .map(|(i, r)| {
            json!({"id": (i + 1).to_string(), "rule": r.rule(), "part": r.part(), "message": r.to_string()})
        })
        .collect()
}

/// The rows of one view.
pub fn view(studio: &Mutex<Studio>, name: &str, request: &Request) -> Response {
    let s = lock(studio);
    match name {
        "studio.Architectures" => rows(
            s.architectures()
                .into_iter()
                .map(|source| {
                    let name = source
                        .trim_start_matches("architectures/")
                        .trim_end_matches(".yaml")
                        .to_string();
                    json!({"name": name, "source": source})
                })
                .collect(),
        ),
        "studio.draft.Drafts" => rows(s.drafts().iter().map(|d| to_json(&d.row)).collect()),
        "studio.DraftSummary"
        | "studio.DraftParts"
        | "studio.DraftRefusals"
        | "studio.DraftGraph" => {
            let d = match draft_param(&s, request) {
                Ok(d) => d,
                Err(r) => return r,
            };
            let current = d.current();
            let document = &current.document;
            match name {
                "studio.DraftSummary" => rows(vec![json!({
                    "draft_id": d.row.draft_id.0,
                    "name": d.row.name.0,
                    "revision": current.row.number,
                    "digest": current.row.digest.0,
                    "passed": check::check(document).is_empty(),
                    "description": document["summary"].as_str().unwrap_or_default(),
                })]),
                "studio.DraftParts" => rows(
                    agenticspec_patch::facts(document)
                        .parts
                        .iter()
                        .map(|p| json!({"id": p.id, "block": val(&p.block), "count": p.count}))
                        .collect(),
                ),
                "studio.DraftRefusals" => rows(refusal_rows(&check::check(document))),
                _ => {
                    let mut g = agenticspec_render::graph(document);
                    g["draft_id"] = Value::from(d.row.draft_id.0.as_str());
                    g["revision"] = Value::from(current.row.number.clone());
                    rows(vec![g])
                }
            }
        }
        _ => error(404, "NoSuchView", &format!("the studio has no view {name}")),
    }
}

fn shown(v: Option<&Value>) -> String {
    match v {
        None => "(none)".into(),
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
    }
}

fn diff_lines(diff: &[Change]) -> Vec<String> {
    diff.iter()
        .map(|c| {
            format!(
                "{}: {} → {}",
                c.path,
                shown(c.before.as_ref()),
                shown(c.after.as_ref())
            )
        })
        .collect()
}

fn refused_row(r: &Refused, diff: &[Change]) -> Value {
    let head = match r.rule {
        Some(n) => format!("Refused at rule {n}: {} — {}", r.error.name(), r.message),
        None => format!("Refused: {} — {}", r.error.name(), r.message),
    };
    let check: Vec<String> = r
        .check
        .iter()
        .map(|c| format!("{}: {c}", c.rule()))
        .collect();
    let message = if check.is_empty() {
        head
    } else {
        format!("{head}; {}", check.join("; "))
    };
    json!({
        "outcome": "Refused",
        "rule": r.rule,
        "error": r.error.name(),
        "message": message,
        "diff": diff_lines(diff),
    })
}

fn call_tool(studio: &Mutex<Studio>, input: &Value) -> Response {
    let (Some(id), Some(text)) = (input["draft_id"].as_str(), input["arguments"].as_str()) else {
        return error(
            400,
            "InvalidRequest",
            "CallTool takes {\"draft_id\", \"arguments\": \"<patch rules>\", \"dry_run\"?}",
        );
    };
    let dry_run = input["dry_run"].as_bool().unwrap_or(false);
    let mut s = lock(studio);
    if let Err(e) = s.draft(id) {
        return studio_error(&e);
    }
    let patch = match Patch::from_text(text) {
        Ok(p) => p,
        Err(r) => return Response::json(422, &json!({"row": refused_row(&r, &[])})),
    };
    let outcome = match s.apply(id, &patch, dry_run) {
        Ok(o) => o,
        Err(e) => return studio_error(&e),
    };
    if let Some(r) = &outcome.refusal {
        return Response::json(422, &json!({"row": refused_row(r, &outcome.diff)}));
    }
    let n = outcome.changed_rows;
    let rows = if n == 1 { "row" } else { "rows" };
    let row = match &outcome.revision {
        Some(rev) => json!({
            "outcome": "Applied",
            "revision": rev.number,
            "message": format!("Applied as revision {}, {n} {rows} changed.", rev.number),
            "diff": diff_lines(&outcome.diff),
        }),
        None => json!({
            "outcome": "DryRun",
            "message": format!("Dry run: {n} {rows} would change; the draft keeps its revision."),
            "diff": diff_lines(&outcome.diff),
        }),
    };
    Response::json(200, &json!({ "row": row }))
}

/// One command.
pub fn command(studio: &Mutex<Studio>, name: &str, request: &Request) -> Response {
    let input: Value = match serde_json::from_slice(&request.body) {
        Ok(v @ Value::Object(_)) => v,
        _ => return error(400, "InvalidRequest", "a command takes a JSON object"),
    };
    match name {
        "studio.draft.OpenDraft" => {
            let Some(source) = input["source"].as_str() else {
                return error(400, "InvalidRequest", "OpenDraft takes {\"source\"}");
            };
            let name = input["name"]
                .as_str()
                .map(str::trim)
                .filter(|n| !n.is_empty());
            match lock(studio).open(source, name) {
                Ok(d) => Response::json(201, &json!({"row": to_json(&d.row)})),
                Err(e) => studio_error(&e),
            }
        }
        "studio.session.CallTool" => call_tool(studio, &input),
        _ => error(
            404,
            "NoSuchCommand",
            &format!("the studio has no command {name}"),
        ),
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => HTML,
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

/// A file of the built app. Only plain names below `www` are read; nothing else leaves it.
pub fn file(www: &Path, path: &str) -> Response {
    let rel = decode(path.trim_start_matches('/'));
    let rel = if rel.is_empty() {
        "index.html".to_string()
    } else {
        rel
    };
    let p = Path::new(&rel);
    let plain = p.components().all(|c| matches!(c, Component::Normal(_)))
        && !rel.contains('\\')
        && !rel.split('/').any(|s| s.is_empty() || s.starts_with('.'));
    let full = www.join(p);
    if plain && let Ok(bytes) = std::fs::read(&full) {
        return Response::new(200, content_type(&full), bytes);
    }
    if rel == "index.html" {
        return Response::new(
            404,
            TEXT,
            format!(
                "the studio app is not built: {} has no index.html. Run `task studio:build`, or pass --www.\n",
                www.display()
            ),
        );
    }
    Response::new(404, TEXT, format!("no file {path}\n"))
}
