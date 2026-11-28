//! `agenticspec ui build`: the static files of an architecture's operator UI, from the binary
//! alone.
//!
//! The operator app (`ui/operator/`) is built once in the repository into `ui/operator/built/`
//! and embedded here with what a derivation reads: the panels document, the port catalogue and
//! the part of the compiled blocks [`crate::derive::model::Model`] reads. A build writes the app's
//! files and its data files: `views.json` (the layout `agenticspec ui derive` writes as
//! `<name>.layout.json`), `contract.json`, `ports.json` and `config.json` (the architecture, the
//! base path of the runtime's adapter, the title). The same inputs give the same bytes, and
//! [`archive`] packs them into a tar that is the same bytes each time too.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::derive::{self, ControlInputs, Failure};

/// The files of the built operator app, by name, sorted.
pub const APP: &[(&str, &[u8])] = &[
    (
        "app.css",
        include_bytes!("../../../ui/operator/built/app.css"),
    ),
    (
        "app.js",
        include_bytes!("../../../ui/operator/built/app.js"),
    ),
    (
        "index.html",
        include_bytes!("../../../ui/operator/built/index.html"),
    ),
];

/// The panels document a derivation copies from.
pub const PANELS: &str = include_str!("../../../ui/views/blocks.ui.yaml");
/// The port catalogue of the blocks this binary was built with.
pub const PORTS: &str = include_str!("../../../blocks/ports.json");
/// The part of the compiled blocks a derivation reads (`agenticspec ui panels` writes it).
const MODEL: &str = include_str!("../../../ui/views/blocks.model.json");

/// `text`, a JSON file built into the binary, parsed once per process.
fn parsed(
    cell: &'static OnceLock<Result<Value, String>>,
    text: &str,
    what: &str,
) -> Result<&'static Value, Failure> {
    cell.get_or_init(|| serde_json::from_str(text).map_err(|e| format!("the embedded {what}: {e}")))
        .as_ref()
        .map_err(|e| Failure::Unreadable(e.clone()))
}

/// [`MODEL`], the part of the compiled blocks a derivation reads, parsed once per process.
///
/// # Errors
///
/// The embedded model is not JSON.
pub fn built_in_model() -> Result<&'static Value, Failure> {
    static CELL: OnceLock<Result<Value, String>> = OnceLock::new();
    parsed(&CELL, MODEL, "model")
}

/// [`PORTS`], the port catalogue this binary was built with, parsed once per process.
///
/// # Errors
///
/// The embedded port catalogue is not JSON.
pub fn built_in_ports() -> Result<&'static Value, Failure> {
    static CELL: OnceLock<Result<Value, String>> = OnceLock::new();
    parsed(&CELL, PORTS, "ports")
}

/// The format of `config.json`.
pub const CONFIG_FORMAT: &str = "agenticspec-operator-config/1";
/// The `mode` of `config.json` of a control-plane UI.
pub const CONTROL_PLANE: &str = "control-plane";
/// The base path of the runtime's adapter when none is named.
pub const API_BASE: &str = "/api/";

/// What `agenticspec ui build` reads.
pub struct Inputs {
    /// The directory the names of the architecture documents are relative to.
    pub root: PathBuf,
    pub architecture: PathBuf,
    pub contract: PathBuf,
    pub api_base: String,
    /// Build the control-plane UI: the operator UI with the pages across parts.
    pub control_plane: bool,
    /// A simulator result whose predictions the Metrics page shows; read with `control_plane`.
    pub prediction: Option<PathBuf>,
}

/// An architecture document or contract read from `path`, as JSON when it ends in `.json` and as
/// YAML otherwise.
///
/// # Errors
///
/// The file cannot be read or parsed.
pub fn read_doc(path: &Path) -> Result<Value, Failure> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| Failure::Unreadable(format!("{}: {e}", path.display())))?;
    let parsed = if path.extension().is_some_and(|x| x == "json") {
        serde_json::from_str(&text).map_err(|e| e.to_string())
    } else {
        serde_yaml_ng::from_str(&text).map_err(|e| e.to_string())
    };
    parsed.map_err(|e| Failure::Unreadable(format!("{}: {e}", path.display())))
}

/// A contract held as YAML text (what `agenticspec sim contract` prints), as JSON.
///
/// # Errors
///
/// The text is not YAML.
pub fn parse_contract(text: &str) -> Result<Value, Failure> {
    serde_yaml_ng::from_str(text).map_err(|e| Failure::Unreadable(format!("the contract: {e}")))
}

fn pretty(v: &Value) -> Vec<u8> {
    let mut text = serde_json::to_string_pretty(v).unwrap_or_default();
    text.push('\n');
    text.into_bytes()
}

fn refused(path: &str, message: String) -> Failure {
    Failure::Refused(vec![crate::Refusal {
        path: path.to_string(),
        message,
    }])
}

/// Every file of the operator UI of `inputs.architecture`, by name, sorted.
///
/// # Errors
///
/// An input that cannot be read, a contract of another architecture or of other ports, or an
/// architecture or `view:` key the derivation refuses.
pub fn build(inputs: &Inputs) -> Result<Vec<(String, Vec<u8>)>, Failure> {
    build_for(inputs, None)
}

/// Every file of the UI of the ingress `ingress` of `inputs.architecture` (`operator` when none is
/// named), by name, sorted: as [`build`], scoped to the ingress's interface when the contract
/// gives a UI per ingress (see [`crate::derive::ingress`]).
///
/// # Errors
///
/// As [`build`], and an ingress with no `ui` export.
pub fn build_for(
    inputs: &Inputs,
    ingress: Option<&str>,
) -> Result<Vec<(String, Vec<u8>)>, Failure> {
    let contract = read_doc(&inputs.contract)?;
    let result = match (&inputs.prediction, inputs.control_plane) {
        (Some(p), true) => Some(read_doc(p)?),
        _ => None,
    };
    let control = inputs.control_plane.then_some(ControlInputs {
        result: result.as_ref(),
    });
    build_ingress(
        &inputs.root,
        &inputs.architecture,
        &contract,
        &inputs.contract.display().to_string(),
        &inputs.api_base,
        control.as_ref(),
        ingress,
    )
}

/// Every file of the operator UI of `architecture`, by name, sorted, from a contract held in
/// memory (`shown` names it in a refusal): the files [`build`] writes for the same contract.
///
/// # Errors
///
/// A contract of another architecture or of other ports, an architecture that cannot be read, or
/// an architecture or `view:` key the derivation refuses.
pub fn build_with(
    root: &Path,
    architecture: &Path,
    contract: &Value,
    shown: &str,
    api_base: &str,
) -> Result<Vec<(String, Vec<u8>)>, Failure> {
    build_ui(root, architecture, contract, shown, api_base, None)
}

/// Every file of the operator UI of `architecture` as [`build_with`] writes them, or with `control`
/// of its control-plane UI: the layout holds the pages across parts (`control`), and `config.json`
/// says `mode: control-plane`.
///
/// # Errors
///
/// As [`build_with`], and a simulator result of another revision.
pub fn build_ui(
    root: &Path,
    architecture: &Path,
    contract: &Value,
    shown: &str,
    api_base: &str,
    control: Option<&ControlInputs<'_>>,
) -> Result<Vec<(String, Vec<u8>)>, Failure> {
    build_ingress(root, architecture, contract, shown, api_base, control, None)
}

/// Every file of the UI of the ingress `ingress` of `architecture` (`operator` when none is named)
/// as [`build_ui`] writes them, scoped to the ingress's interface when the contract gives a UI per
/// ingress (see [`crate::derive::ingress`]); `config.json` then names the ingress and its title
/// says it.
///
/// # Errors
///
/// As [`build_ui`], and an ingress with no `ui` export.
pub fn build_ingress(
    root: &Path,
    architecture: &Path,
    contract: &Value,
    shown: &str,
    api_base: &str,
    control: Option<&ControlInputs<'_>>,
    ingress: Option<&str>,
) -> Result<Vec<(String, Vec<u8>)>, Failure> {
    let ir = built_in_model()?;
    let ports = built_in_ports()?;
    let shown = shown.to_string();
    if contract["format"].as_str() != Some("agenticspec-contract/1") {
        return Err(refused(
            &shown,
            "not an agenticspec-contract/1 document".to_string(),
        ));
    }
    let carried = ports["sources_sha256"].as_str().unwrap_or("");
    if let Some(digest) = contract["ports_digest"].as_str()
        && digest != carried
    {
        return Err(refused(
            &format!("{shown}/ports_digest"),
            format!(
                "{digest} is not the digest of the ports this binary carries, {carried}: derive the contract again with this binary (agenticspec sim contract)"
            ),
        ));
    }
    let layout = derive::layout_for(
        root,
        architecture,
        ir,
        PANELS,
        Some(ports),
        Some(contract),
        control,
        ingress,
    )?;
    let doc: Value = serde_json::from_str(&layout)
        .map_err(|e| Failure::Unreadable(format!("the layout: {e}")))?;
    let architecture = doc["architecture"].as_str().unwrap_or("").to_string();
    if let Some(of) = contract["architecture"].as_str()
        && of != architecture
    {
        return Err(refused(
            &format!("{shown}/architecture"),
            format!("the contract is of {of}, not of {architecture}"),
        ));
    }
    let mut config = json!({
        "format": CONFIG_FORMAT,
        "architecture": architecture,
        "api_base": api_base,
        "title": architecture,
    });
    let scoped = doc["ingress"].as_str();
    if let Some(i) = scoped {
        config["ingress"] = json!(i);
        config["title"] = json!(format!("{architecture} ({i})"));
    }
    if control.is_some() {
        config["mode"] = json!(CONTROL_PLANE);
        config["title"] = json!(format!("{architecture} control plane"));
    }
    let mut files: Vec<(String, Vec<u8>)> = APP
        .iter()
        .map(|(name, bytes)| ((*name).to_string(), bytes.to_vec()))
        .collect();
    files.push(("config.json".to_string(), pretty(&config)));
    files.push(("contract.json".to_string(), pretty(contract)));
    files.push(("ports.json".to_string(), PORTS.as_bytes().to_vec()));
    files.push(("views.json".to_string(), layout.into_bytes()));
    files.sort();
    Ok(files)
}

/// Whether `dir` may be replaced: it does not exist, is empty, or holds an operator UI.
fn replaceable(dir: &Path) -> bool {
    let Ok(mut entries) = std::fs::read_dir(dir) else {
        return !dir.exists();
    };
    if entries.next().is_none() {
        return true;
    }
    std::fs::read_to_string(dir.join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .is_some_and(|c| c["format"].as_str() == Some(CONFIG_FORMAT))
}

/// Replaces `dir` with exactly `files`. A directory that holds anything but an operator UI is
/// refused rather than emptied.
///
/// # Errors
///
/// `dir` holds other files, or a file or directory cannot be written.
pub fn write(files: &[(String, Vec<u8>)], dir: &Path) -> std::io::Result<()> {
    if !replaceable(dir) {
        return Err(std::io::Error::other(
            "holds files and no operator UI; name a new or empty directory",
        ));
    }
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    std::fs::create_dir_all(dir)?;
    for (name, bytes) in files {
        std::fs::write(dir.join(name), bytes)?;
    }
    Ok(())
}

/// The path of the archive of `dir`: `<dir>.tar`.
pub fn archive_path(dir: &Path) -> PathBuf {
    let mut name = dir.file_name().unwrap_or_default().to_os_string();
    name.push(".tar");
    dir.with_file_name(name)
}

fn octal(field: &mut [u8], value: u64) {
    let digits = field.len() - 1;
    let text = format!("{value:0digits$o}");
    field[..digits].copy_from_slice(&text.as_bytes()[text.len() - digits..]);
    field[digits] = 0;
}

/// A ustar archive of `files` (names below 100 bytes, sorted by the caller): every entry a regular
/// file with mode 0644, owner and group 0 with no names, and modification time 0, so the same
/// files give the same bytes.
///
/// # Errors
///
/// A name of 100 bytes or more.
pub fn archive(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for (name, bytes) in files {
        if name.len() >= 100 {
            return Err(format!("{name}: a name of 100 bytes or more"));
        }
        let mut h = [0u8; 512];
        h[..name.len()].copy_from_slice(name.as_bytes());
        octal(&mut h[100..108], 0o644);
        octal(&mut h[108..116], 0);
        octal(&mut h[116..124], 0);
        octal(&mut h[124..136], bytes.len() as u64);
        octal(&mut h[136..148], 0);
        h[148..156].fill(b' ');
        h[156] = b'0';
        h[257..263].copy_from_slice(b"ustar\0");
        h[263..265].copy_from_slice(b"00");
        let sum: u64 = h.iter().map(|b| u64::from(*b)).sum();
        octal(&mut h[148..155], sum);
        h[155] = b' ';
        out.extend_from_slice(&h);
        out.extend_from_slice(bytes);
        out.resize(out.len().div_ceil(512) * 512, 0);
    }
    out.resize(out.len() + 1024, 0);
    Ok(out)
}

/// The sha256 of `bytes`, in hex.
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
