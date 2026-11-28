//! The routes of `agenticstudio serve`, answered from the studio without a socket.
//!
//! | method | path | answer |
//! |---|---|---|
//! | GET | `/api/views/<view>`, POST `/api/commands/<command>` | the routes of the generated app (`app`) |
//! | GET | any path outside `/api` | a file of the built app (`app::file`); `/` is its `index.html` |
//! | GET | `/api/architectures` | the architecture documents a draft opens |
//! | GET | `/api/drafts` | the drafts |
//! | POST | `/api/drafts` | open a draft: `{"source": "architectures/<name>.yaml", "name"?}` |
//! | GET | `/api/drafts/<id>` | the draft: its document, revision, parts, check and applications |
//! | GET | `/api/drafts/<id>/graph` | the graph page of its current revision |
//! | POST | `/api/drafts/<id>/patches` | apply a patch: `{"text", "dry_run"?}` or a patch document |

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use agenticspec_patch::{Change, Patch, Refused};
use agenticspec_records::sq::facts::Val;
use agenticspec_sim::check::{self, Refusal};
use serde_json::{Value, json};

use crate::http::{HTML, Request, Response};
use crate::studio::{Draft, Studio, StudioError};

pub(crate) fn lock(studio: &Mutex<Studio>) -> MutexGuard<'_, Studio> {
    studio.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn error(status: u16, name: &str, message: &str) -> Response {
    Response::json(status, &json!({"error": name, "message": message}))
}

pub(crate) fn studio_error(e: &StudioError) -> Response {
    let status = match e {
        StudioError::NoSuchDraft(_) | StudioError::NoSuchSource(_) => 404,
        StudioError::DraftNameTaken(_) => 409,
        StudioError::NotAnArchitecture(_) => 422,
    };
    error(status, e.name(), &e.message())
}

pub(crate) fn to_json<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

pub(crate) fn val(v: &Val) -> Value {
    match v {
        Val::Null => Value::Null,
        Val::Bool(b) => Value::from(*b),
        Val::Int(n) => Value::from(*n),
        Val::Str(s) => Value::from(s.as_ref()),
    }
}

fn check_json(refusals: &[Refusal]) -> Value {
    json!({
        "passed": refusals.is_empty(),
        "refusals": refusals
            .iter()
            .map(|r| json!({"rule": r.rule(), "part": r.part(), "message": r.to_string()}))
            .collect::<Vec<_>>(),
    })
}

fn diff_json(diff: &[Change]) -> Value {
    Value::from(
        diff.iter()
            .map(|c| json!({"path": c.path, "before": c.before, "after": c.after}))
            .collect::<Vec<_>>(),
    )
}

fn refusal_json(r: &Refused) -> Value {
    json!({"rule": r.rule, "error": r.error.name(), "message": r.message})
}

/// A draft as the page reads it.
pub fn draft_json(d: &Draft) -> Value {
    let current = d.current();
    let facts = agenticspec_patch::facts(&current.document);
    let id = &d.row.draft_id.0;
    json!({
        "draft": to_json(&d.row),
        "revision": to_json(&current.row),
        "document": current.document,
        "parts": facts
            .parts
            .iter()
            .map(|p| json!({"id": p.id, "block": val(&p.block), "count": p.count}))
            .collect::<Vec<_>>(),
        "check": check_json(&check::check(&current.document)),
        "graph": format!("/api/drafts/{id}/graph"),
        "applications": d.applications.iter().map(to_json).collect::<Vec<_>>(),
    })
}

fn body(request: &Request) -> Result<Value, Response> {
    let v: Value = serde_json::from_slice(&request.body)
        .map_err(|e| error(400, "InvalidRequest", &format!("the body is no JSON: {e}")))?;
    if v.is_object() {
        Ok(v)
    } else {
        Err(error(400, "InvalidRequest", "the body is a JSON object"))
    }
}

fn open(studio: &Mutex<Studio>, request: &Request) -> Response {
    let v = match body(request) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let Some(source) = v["source"].as_str() else {
        return error(
            400,
            "InvalidRequest",
            "open takes {\"source\": \"architectures/<name>.yaml\"}",
        );
    };
    let name = match &v["name"] {
        Value::Null => None,
        Value::String(n) if !n.trim().is_empty() => Some(n.trim()),
        _ => {
            return error(
                400,
                "InvalidRequest",
                "a draft's name is a non-empty string",
            );
        }
    };
    let mut s = lock(studio);
    match s.open(source, name) {
        Ok(d) => Response::json(201, &draft_json(d)),
        Err(e) => studio_error(&e),
    }
}

fn apply(studio: &Mutex<Studio>, id: &str, request: &Request) -> Response {
    let v = match body(request) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let dry_run = v["dry_run"].as_bool().unwrap_or(false);
    let patch = if v.get("format").is_some() {
        Patch::from_document(&v)
    } else if let Some(text) = v["text"].as_str() {
        Patch::from_text(text).map(|mut p| {
            p.name = v["name"].as_str().map(str::to_string);
            p
        })
    } else {
        return error(
            400,
            "InvalidRequest",
            "apply takes {\"text\": \"<rules>\"} or a patch document",
        );
    };
    let mut s = lock(studio);
    let patch = match patch {
        Ok(p) => p,
        Err(refused) => {
            if let Err(e) = s.draft(id) {
                return studio_error(&e);
            }
            return Response::json(
                422,
                &json!({"refusal": refusal_json(&refused), "application": null, "patch": null,
                        "diff": [], "check": null, "revision": null}),
            );
        }
    };
    let outcome = match s.apply(id, &patch, dry_run) {
        Ok(o) => o,
        Err(e) => return studio_error(&e),
    };
    let draft = s.draft(id).map(draft_json).unwrap_or(Value::Null);
    let status = if outcome.refusal.is_some() { 422 } else { 200 };
    let check = match &outcome.refusal {
        Some(r) if r.check.is_empty() => Value::Null,
        _ => check_json(&outcome.check),
    };
    Response::json(
        status,
        &json!({
            "refusal": outcome.refusal.as_ref().map(refusal_json),
            "application": outcome.application.as_ref().map(to_json),
            "patch": outcome.patch.as_ref().map(to_json),
            "diff": diff_json(&outcome.diff),
            "check": check,
            "revision": outcome.revision.as_ref().map(to_json),
            "draft": draft,
        }),
    )
}

/// The answer to one request: an `/api` route, or a file of the built app under `www`.
pub fn serve(studio: &Mutex<Studio>, www: &Path, request: &Request) -> Response {
    let api = request.path == "/api" || request.path.starts_with("/api/");
    if !api && request.method == "GET" {
        return crate::app::file(www, &request.path);
    }
    answer(studio, request)
}

/// The answer to one `/api` request.
pub fn answer(studio: &Mutex<Studio>, request: &Request) -> Response {
    let method = request.method.as_str();
    let segments: Vec<&str> = request.path.trim_matches('/').split('/').collect();
    match (method, segments.as_slice()) {
        ("GET", ["api", "views", name]) => crate::app::view(studio, name, request),
        ("POST", ["api", "commands", name]) => crate::app::command(studio, name, request),
        ("GET", ["api", "architectures"]) => {
            let list: Vec<Value> = lock(studio)
                .architectures()
                .into_iter()
                .map(|source| {
                    let name = source
                        .trim_start_matches("architectures/")
                        .trim_end_matches(".yaml")
                        .to_string();
                    json!({"name": name, "source": source})
                })
                .collect();
            Response::json(200, &json!({"architectures": list}))
        }
        ("GET", ["api", "drafts"]) => {
            let rows: Vec<Value> = lock(studio)
                .drafts()
                .iter()
                .map(|d| to_json(&d.row))
                .collect();
            Response::json(200, &json!({"drafts": rows}))
        }
        ("POST", ["api", "drafts"]) => open(studio, request),
        ("GET", ["api", "drafts", id]) => match lock(studio).draft(id) {
            Ok(d) => Response::json(200, &draft_json(d)),
            Err(e) => studio_error(&e),
        },
        ("GET", ["api", "drafts", id, "graph"]) => match lock(studio).draft(id) {
            Ok(d) => Response::new(
                200,
                HTML,
                agenticspec_render::page(&d.current().document, None),
            ),
            Err(e) => studio_error(&e),
        },
        ("POST", ["api", "drafts", id, "patches"]) => apply(studio, id, request),
        (_, ["api", "architectures"] | ["api", "drafts"] | ["api", "drafts", _])
        | (_, ["api", "drafts", _, "graph" | "patches"])
        | (_, ["api", "views" | "commands", _]) => error(
            405,
            "MethodNotAllowed",
            &format!("{method} is not allowed on {}", request.path),
        ),
        _ => error(404, "NoSuchRoute", &format!("no route {}", request.path)),
    }
}
