//! The UIs of a bundle: one unit per UI the binding asks for under `ui:`, a `service` unit that
//! serves static files (`code: {kind: static, archive, digest}`) and reads the adapter of an
//! upstream unit (`upstream: {unit, port, path}`). The archive is built in-process with the
//! library `agenticspec ui build` uses, the same bytes as `ui build --archive` writes, and is
//! carried inline in the bundle's `files` member, base64, by its file name. The UI unit serves
//! its export of an ingress (the operator's `ui`); the other exports stay where they were.
//!
//! A UI is one row of [`KINDS`]: its key under `ui:`, its role, the ingress and export it serves
//! and how its files are built. The operator UI and the control-plane UI (the operator UI with the
//! pages across parts, `ui build --control-plane`) serve the same export: the first one the binding
//! asks for serves it, and the other listens beside it on the port `ui.<ui>.port` gives it. A
//! binding without `ui:` gives the bundle it gave before.

use super::admit::{Accepts, Finding};
use super::binding::{BIND, is_port, is_text};
use super::bundle::CONTROL;
use super::packing::Packing;
use super::sha256;
use serde_json::{Map, Value, json};
use std::path::Path;

/// The files of a UI from the architecture's file and its contract: by name, sorted.
pub type Build = fn(&Path, &Value, &str) -> Result<Vec<(String, Vec<u8>)>, String>;

/// One UI a binding may ask for.
pub struct Kind {
    /// Its key under the binding's `ui:`.
    pub key: &'static str,
    /// The role of its unit, which is also the unit's name.
    pub role: &'static str,
    /// The ingress whose export it serves.
    pub ingress: &'static str,
    /// The export of that ingress it serves.
    pub export: &'static str,
    /// The export of that ingress whose unit is its default upstream.
    pub api: &'static str,
    pub build: Build,
}

/// The UIs a binding may ask for.
pub const KINDS: &[Kind] = &[
    Kind {
        key: "operator",
        role: "operator-ui",
        ingress: "operator",
        export: "ui",
        api: "http",
        build: operator_files,
    },
    Kind {
        key: "control-plane",
        role: "control-plane-ui",
        ingress: "operator",
        export: "ui",
        api: "http",
        build: control_plane_files,
    },
];

/// The base path of the runtime's adapter a UI reads, written into its `config.json`.
pub const API_BASE: &str = agenticspec_ui::operator::API_BASE;

/// The keys of one UI's entry under `ui:`.
const KEYS: &[&str] = &["port", "bind", "upstream"];

/// The keys of its `upstream:`.
const UPSTREAM_KEYS: &[&str] = &["unit", "port"];

/// The refusals of a written bundle that a build cannot reach: the archive a UI unit names is not
/// the archive the bundle carries.
pub const CHECK_REFUSALS: &[&str] = &["ui_archive_mismatch"];

fn operator_files(
    architecture: &Path,
    contract: &Value,
    api_base: &str,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    ui_files(architecture, contract, api_base, None)
}

fn control_plane_files(
    architecture: &Path,
    contract: &Value,
    api_base: &str,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    let control = agenticspec_ui::derive::ControlInputs::default();
    ui_files(architecture, contract, api_base, Some(&control))
}

/// The files of `agenticspec ui build` (with `--control-plane` when `control` is given).
fn ui_files(
    architecture: &Path,
    contract: &Value,
    api_base: &str,
    control: Option<&agenticspec_ui::derive::ControlInputs<'_>>,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    use agenticspec_ui::derive::Failure;
    let root = architecture.parent().unwrap_or(Path::new("."));
    agenticspec_ui::operator::build_ui(root, architecture, contract, "contract", api_base, control)
        .map_err(|f| match f {
            Failure::Unreadable(e) => e,
            Failure::Refused(rs) => rs
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        })
}

/// Whether the unit of `k` serves its export: the first UI the binding asks for that serves that
/// export of that ingress does; another one beside it listens on a port of its own.
pub fn serves_export(k: &Kind, binding: &Value) -> bool {
    asked(binding)
        .into_iter()
        .find(|x| x.ingress == k.ingress && x.export == k.export)
        .is_some_and(|x| x.key == k.key)
}

/// The name of a UI's unit: its role.
pub fn unit_name(k: &Kind) -> &'static str {
    k.role
}

/// The file name of a UI's archive among the bundle's `files`.
pub fn archive_name(k: &Kind) -> String {
    format!("{}.tar", k.role)
}

/// The UIs the binding asks for, in the order of [`KINDS`].
pub fn asked(binding: &Value) -> Vec<&'static Kind> {
    let Some(ui) = binding["ui"].as_object() else {
        return Vec::new();
    };
    KINDS.iter().filter(|k| ui.contains_key(k.key)).collect()
}

/// The problems of the form of the binding's `ui:`: `{<ui>: {port?, bind?, upstream?: {unit?,
/// port?}}}`, one line each.
pub fn problems(binding: &Value, out: &mut Vec<String>) {
    let ui = match &binding["ui"] {
        Value::Null => return,
        Value::Object(m) => m,
        _ => {
            out.push("`ui:` is a mapping from a UI to where it listens and what it reads".into());
            return;
        }
    };
    for (key, v) in ui {
        if !KINDS.iter().any(|k| k.key == key) {
            let known: Vec<&str> = KINDS.iter().map(|k| k.key).collect();
            out.push(format!("ui: `{key}` is not one of {}", known.join(", ")));
            continue;
        }
        let entry = match v {
            Value::Null => continue,
            Value::Object(m) => m,
            _ => {
                out.push(format!("ui: `{key}` is a mapping"));
                continue;
            }
        };
        for (k, x) in entry {
            match k.as_str() {
                "port" if is_port(x) => {}
                "port" => out.push(format!("ui: {key}: port is a whole number from 1 to 65535")),
                "bind" if x.as_str().is_some_and(|b| BIND.contains(&b)) => {}
                "bind" => out.push(format!("ui: {key}: bind is one of {}", BIND.join(", "))),
                "upstream" => upstream_problems(key, x, out),
                other => out.push(format!(
                    "ui: {key}: unknown key `{other}` (one of {})",
                    KEYS.join(", ")
                )),
            }
        }
    }
}

fn upstream_problems(key: &str, v: &Value, out: &mut Vec<String>) {
    let Some(m) = v.as_object() else {
        out.push(format!("ui: {key}: upstream is {{unit, port}}"));
        return;
    };
    for (k, x) in m {
        match k.as_str() {
            "unit" if x.as_str().is_some_and(is_text) => {}
            "unit" => out.push(format!("ui: {key}: upstream: unit names a unit")),
            "port" if is_port(x) => {}
            "port" => out.push(format!(
                "ui: {key}: upstream: port is a whole number from 1 to 65535"
            )),
            other => out.push(format!(
                "ui: {key}: upstream: unknown key `{other}` (one of {})",
                UPSTREAM_KEYS.join(", ")
            )),
        }
    }
}

/// The problems of the binding's `ui:` that need the contract and the packing: an ingress of the
/// contract that does not export what the UI serves, a unit of the packing that takes the UI
/// unit's name, a `part` for the export the UI unit serves, and a port or bind given twice,
/// differently, under `ui:` and under the export.
pub fn served_problems(
    binding: &Value,
    contract_ingress: &Value,
    packing: &Packing,
) -> Vec<String> {
    let mut out = Vec::new();
    for k in asked(binding) {
        let exported = contract_ingress
            .as_array()
            .into_iter()
            .flatten()
            .filter(|i| i["name"] == k.ingress)
            .flat_map(|i| i["exports"].as_array().into_iter().flatten())
            .any(|p| p == k.export);
        if !exported {
            out.push(format!(
                "ui: {}: the contract's ingress {} exports no `{}` for the UI to serve",
                k.key, k.ingress, k.export
            ));
        }
        if packing.units.iter().any(|u| u.name == unit_name(k)) {
            out.push(format!(
                "ui: {}: the packing has a unit {}, the name of the UI's unit",
                k.key,
                unit_name(k)
            ));
        }
        if !serves_export(k, binding) {
            continue;
        }
        let e = &binding["ingress"][k.ingress]["exports"][k.export];
        if !e["part"].is_null() {
            out.push(format!(
                "ingress: {}: exports: {}: part: the {} UI's unit serves this export",
                k.ingress, k.export, k.key
            ));
        }
        let ui = &binding["ui"][k.key];
        for field in ["port", "bind"] {
            if !ui[field].is_null() && !e[field].is_null() && ui[field] != e[field] {
                out.push(format!(
                    "ui: {}: {field} {} is not the {field} of ingress {}'s {} export, {}; give it in one place",
                    k.key, ui[field], k.ingress, k.export, e[field]
                ));
            }
        }
    }
    out
}

/// Point each export a UI serves at the UI's unit and the reserved part [`CONTROL`], with the
/// port and bind of `ui.<ui>` where the binding's export gives none.
pub fn serve(entries: &mut [Value], binding: &Value) {
    for k in asked(binding)
        .into_iter()
        .filter(|k| serves_export(k, binding))
    {
        let ui = &binding["ui"][k.key];
        let e = &binding["ingress"][k.ingress]["exports"][k.export];
        for entry in entries
            .iter_mut()
            .filter(|x| x["ingress"] == k.ingress && x["protocol"] == k.export)
        {
            entry["unit"] = json!(unit_name(k));
            entry["part"] = json!(CONTROL);
            if e["port"].is_null()
                && let Some(p) = ui["port"].as_u64()
            {
                entry["port"] = json!(p);
            }
            if e["bind"].is_null()
                && let Some(b) = ui["bind"].as_str()
            {
                entry["bind"] = json!(b);
            }
        }
    }
}

/// The upstream of a UI: the unit the binding names, else the unit that serves the UI's
/// ingress's API export (`ingress` the bundle's entries); its port, the binding's, else that
/// export's when it is served by the same unit; and the base path written into `config.json`.
pub fn upstream(k: &Kind, binding: &Value, ingress: &[Value]) -> Value {
    let u = &binding["ui"][k.key]["upstream"];
    let api = ingress
        .iter()
        .find(|e| e["ingress"] == k.ingress && e["protocol"] == k.api);
    let unit = u["unit"]
        .as_str()
        .or_else(|| api.and_then(|e| e["unit"].as_str()))
        .unwrap_or("core");
    let mut out = json!({ "unit": unit, "path": API_BASE });
    let port = u["port"].as_u64().or_else(|| {
        api.filter(|e| e["unit"] == unit)
            .and_then(|e| e["port"].as_u64())
    });
    if let Some(p) = port {
        out["port"] = json!(p);
    }
    out
}

/// The findings of the UIs the binding asks for that do not need their files: an upstream that
/// names no unit of the packing (`ui_upstream_unknown`), a UI role or the `service` kind the
/// accepted list does not hold (`unit_not_hosted`).
pub fn findings(
    binding: &Value,
    ingress: &[Value],
    packing: &Packing,
    accepts: &Accepts,
) -> Vec<Finding> {
    let mut out = Vec::new();
    for k in asked(binding) {
        let up = upstream(k, binding, ingress);
        let unit = up["unit"].as_str().unwrap_or("");
        if !packing.units.iter().any(|u| u.name == unit) {
            out.push(Finding {
                refusal: "ui_upstream_unknown",
                detail: format!(
                    "ui: {}: upstream unit {unit} is no unit of the bundle",
                    k.key
                ),
            });
        }
        out.extend(hosted(k, accepts));
    }
    out
}

fn hosted(k: &Kind, accepts: &Accepts) -> Vec<Finding> {
    let mut out = Vec::new();
    if !accepts.unit_kinds.contains("service") {
        out.push(Finding {
            refusal: "unit_not_hosted",
            detail: format!(
                "unit {}: kind service is not in the accepted list",
                unit_name(k)
            ),
        });
    }
    if !accepts.roles.contains(k.role) {
        out.push(Finding {
            refusal: "unit_not_hosted",
            detail: format!(
                "unit {}: role {} is not in the accepted list",
                unit_name(k),
                k.role
            ),
        });
    }
    out
}

/// The UI units of a bundle and its `files` member.
pub type Built = (Vec<Value>, Map<String, Value>);

/// The units of the UIs the binding asks for and the files they carry (the `files` member: per
/// archive name its `digest`, its size and its bytes in base64), or the finding of a UI whose
/// files cannot be built (`ui_build_refused`).
pub fn units(
    architecture: &Path,
    contract: &Value,
    binding: &Value,
    ingress: &[Value],
) -> Result<Built, Vec<Finding>> {
    let mut units = Vec::new();
    let mut files = Map::new();
    let mut findings = Vec::new();
    for k in asked(binding) {
        let tar = (k.build)(architecture, contract, API_BASE)
            .and_then(|f| agenticspec_ui::operator::archive(&f));
        let tar = match tar {
            Ok(t) => t,
            Err(e) => {
                findings.push(Finding {
                    refusal: "ui_build_refused",
                    detail: format!("ui: {}: {e}", k.key),
                });
                continue;
            }
        };
        let digest = sha256(&tar);
        let archive = archive_name(k);
        let name = unit_name(k);
        // The export's port when the unit serves the export, else the port `ui.<ui>` gives it.
        let port = if serves_export(k, binding) {
            ingress
                .iter()
                .find(|e| e["ingress"] == k.ingress && e["protocol"] == k.export)
                .and_then(|e| e["port"].as_u64())
        } else {
            binding["ui"][k.key]["port"].as_u64()
        };
        // Each check on the export's port, unless `checks.<unit>.<check>.port` gives its own.
        let given = &binding["checks"][name];
        let check = |c: &str| {
            let mut v = json!({ "kind": "http", "path": "/" });
            if let Some(p) = given[c]["port"].as_u64().or(port) {
                v["port"] = json!(p);
            }
            v
        };
        // The claims `claims.<unit>` gives, as for any other unit.
        let c = &binding["claims"][name];
        units.push(json!({
            "name": name,
            "kind": "service",
            "parts": [],
            "replicas": 1,
            "code": { "kind": "static", "archive": archive, "digest": digest },
            "upstream": upstream(k, binding, ingress),
            "claims": {
                "cpu_millis": c["cpu_millis"].as_u64().unwrap_or(super::bundle::DEFAULT_CPU_MILLIS),
                "memory_mib": c["memory_mib"].as_u64().unwrap_or(super::bundle::DEFAULT_MEMORY_MIB),
                "accelerators": c["accelerators"].as_u64().unwrap_or(0),
                "volumes": [],
            },
            "isolation": "none",
            "checks": { "ready": check("ready"), "alive": check("alive") },
            "secrets": [],
            "egress": [],
            "roles": [k.role],
            "ports": [],
            "source": { "unit": name, "parts": [] },
        }));
        files.insert(
            archive,
            json!({ "digest": digest, "size": tar.len(), "base64": base64(&tar) }),
        );
    }
    if findings.is_empty() {
        Ok((units, files))
    } else {
        Err(findings)
    }
}

/// The findings of the UI units of a written bundle: an archive the bundle does not carry, or
/// whose bytes are not the digest the unit names (`ui_archive_mismatch`); an upstream that names
/// no other unit of the bundle (`ui_upstream_unknown`); a UI role the accepted list does not hold
/// (`unit_not_hosted`).
pub fn check(bundle: &Value, accepts: &Accepts) -> Vec<Finding> {
    let mut out = Vec::new();
    let units: Vec<&Value> = bundle["units"].as_array().into_iter().flatten().collect();
    for u in &units {
        if u["code"]["kind"] != "static" {
            continue;
        }
        let name = u["name"].as_str().unwrap_or("");
        let archive = u["code"]["archive"].as_str().unwrap_or("");
        let named = u["code"]["digest"].as_str().unwrap_or("");
        let file = &bundle["files"][archive];
        let mismatch = match file["base64"].as_str().and_then(unbase64) {
            None => Some(format!(
                "unit {name}: the bundle carries no archive `{archive}` in `files`"
            )),
            Some(bytes) => {
                let d = sha256(&bytes);
                if d != named {
                    Some(format!(
                        "unit {name}: archive {archive} is {d}, and the unit names {named}"
                    ))
                } else if file["digest"] != named {
                    Some(format!(
                        "unit {name}: files: {archive}: digest is not the digest of its bytes"
                    ))
                } else {
                    None
                }
            }
        };
        if let Some(detail) = mismatch {
            out.push(Finding {
                refusal: "ui_archive_mismatch",
                detail,
            });
        }
        let up = u["upstream"]["unit"].as_str().unwrap_or("");
        if up == name || !units.iter().any(|x| x["name"] == up) {
            out.push(Finding {
                refusal: "ui_upstream_unknown",
                detail: format!("unit {name}: upstream unit {up} is no other unit of the bundle"),
            });
        }
        for r in u["roles"].as_array().into_iter().flatten() {
            if let Some(k) = KINDS.iter().find(|k| r == k.role) {
                out.extend(hosted(k, accepts));
            }
        }
    }
    out
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding.
pub fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The bytes of standard base64 with padding; `None` for anything else.
pub fn unbase64(text: &str) -> Option<Vec<u8>> {
    let s = text.as_bytes();
    if !s.len().is_multiple_of(4) {
        return None;
    }
    let value = |c: u8| ALPHABET.iter().position(|a| *a == c).map(|p| p as u32);
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    for (i, chunk) in s.chunks(4).enumerate() {
        let last = i + 1 == s.len() / 4;
        let pad = chunk.iter().rev().take_while(|c| **c == b'=').count();
        if pad > 2 || (pad > 0 && !last) {
            return None;
        }
        let mut n = 0u32;
        for c in &chunk[..4 - pad] {
            n = (n << 6) | value(*c)?;
        }
        n <<= 6 * pad as u32;
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}
