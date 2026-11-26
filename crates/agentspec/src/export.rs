//! Export of a `agentspec-procedure/1` as a canon `protocol/1` file and loom bindings
//! (`design/query-design.md` § 7).
//!
//! The protocol holds one action per step, one evidence kind and one claim per step, the order of
//! the steps as preconditions over claims, and the outcomes `done` and `not_done`. The bindings
//! hold one executor per action: a placeholder program for a program step (the procedure holds no
//! command text), Runbook's model step `sdc-node` for a judgement step, a person for a person step,
//! a child case for a child step. Both files are written by hand, so one procedure gives the same
//! bytes every time.
//!
//! Every step's evidence is about one artifact, `work`, the tree the procedure acts on. A step that
//! reads it holds per revision of the tree, so it runs again after a write. A step that writes it
//! (a judgement step, or a step whose effect is `write`) holds once it has written at any revision:
//! the next write moves the tree on, and a claim bound to the current revision would send every
//! earlier writer back.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use agentspec_types::cli as c;
use sha2::Digest as _;

/// The one artifact every step's evidence is about.
const WORK: &str = "work";
/// Where the bindings place it in the case's workspace.
const WORK_PATH: &str = "tree";
/// The longest protocol id: loom's registration stem is at most 100 bytes.
const MAX_ID: usize = 100;
/// The longest `timeout_seconds` of a program executor (loom 0.61.0).
const MAX_PROGRAM_TIMEOUT: i64 = 14_400;
/// The token prefix of a hook refusal: reported beside the export, never a step (§ 7.2).
const REFUSAL: &str = "refuse:";

pub fn export(procedure: &c::Procedure, out: &Path) -> Result<c::ExportResult, String> {
    let model = Model::of(procedure)?;
    let protocol_text = model.protocol();
    let bindings_text = model.bindings();
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let protocol = out.join(format!("{}.protocol.yaml", model.id));
    let bindings = out.join(format!("{}.yaml", model.id));
    std::fs::write(&protocol, protocol_text).map_err(|e| format!("{}: {e}", protocol.display()))?;
    std::fs::write(&bindings, bindings_text).map_err(|e| format!("{}: {e}", bindings.display()))?;
    let (validated, detail) = validate(&protocol);
    Ok(c::ExportResult {
        protocol: protocol.display().to_string(),
        bindings: bindings.display().to_string(),
        validated,
        detail,
    })
}

/// Runs `canon validate --path <protocol>` when `canon` is on `PATH`: true only on exit 0, with the
/// first line canon printed.
fn validate(protocol: &Path) -> (bool, String) {
    let Some(canon) = on_path("canon") else {
        return (false, "canon not installed".to_owned());
    };
    let run = Command::new(&canon)
        .arg("validate")
        .arg("--path")
        .arg(protocol)
        .output();
    match run {
        Err(e) => (false, format!("{}: {e}", canon.display())),
        Ok(output) => {
            let first = |bytes: &[u8]| {
                String::from_utf8_lossy(bytes)
                    .lines()
                    .find(|line| !line.trim().is_empty())
                    .map(str::to_owned)
            };
            let line = if output.status.success() {
                first(&output.stdout)
            } else {
                first(&output.stderr).or_else(|| first(&output.stdout))
            };
            let line = line.unwrap_or_else(|| format!("canon validate: {}", output.status));
            (output.status.success(), line)
        }
    }
}

/// `program` on `PATH`, else in cargo's `bin` directory (`$CARGO_HOME/bin`, `~/.cargo/bin`), where
/// `cargo install` puts canon; a build job's `PATH` may lack it.
fn on_path(program: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let cargo_bin = std::env::var_os("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".cargo")))
        .map(|home| home.join("bin"));
    std::env::split_paths(&path)
        .chain(cargo_bin)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

/// The procedure with every identifier checked and named as the two files write it.
struct Model<'a> {
    procedure: &'a c::Procedure,
    id: String,
    steps: Vec<Step<'a>>,
    inputs: Vec<Input<'a>>,
    refusals: usize,
}

struct Step<'a> {
    src: &'a c::ProcedureStep,
    id: String,
    after: Vec<String>,
    kind: String,
    claim: String,
    /// The result that makes the claim true; `None` for a child step, whose opening has none.
    ok: Option<&'static str>,
    /// The result a failed run records, which `retry_when` and `not_done` read.
    failed: Option<&'static str>,
    /// Whether the step moves `work` to a new revision.
    revises: bool,
    effect: Option<String>,
    free: Vec<(String, &'a c::ProcedureParameter)>,
}

struct Input<'a> {
    id: String,
    step: String,
    parameter: &'a c::ProcedureParameter,
    values: Vec<&'a str>,
}

impl<'a> Model<'a> {
    fn of(procedure: &'a c::Procedure) -> Result<Self, String> {
        let mut id = ident(&procedure.id, "procedure-");
        id.truncate(MAX_ID);
        let by_id: std::collections::BTreeMap<&str, &c::ProcedureStep> = procedure
            .steps
            .iter()
            .map(|step| (step.id.as_str(), step))
            .collect();
        if by_id.len() != procedure.steps.len() {
            return Err("procedure: two steps share one id".to_owned());
        }
        let mut steps: Vec<Step<'a>> = Vec::new();
        let mut refusals = 0;
        for src in &procedure.steps {
            if src.token.starts_with(REFUSAL) {
                refusals += 1;
                continue;
            }
            let sid = ident(&src.id, "step-");
            if steps.iter().any(|s| s.id == sid) {
                return Err(format!(
                    "step `{}`: its id `{sid}` is another step's",
                    src.id
                ));
            }
            let mut after = Vec::new();
            for target in effective_after(src, &by_id, 0)? {
                let name = ident(target, "step-");
                if !after.contains(&name) {
                    after.push(name);
                }
            }
            let (kind, ok, failed) = match src.kind {
                c::StepKind::Program => (format!("{sid}_run"), Some("ok"), Some("failed")),
                c::StepKind::Judgement => {
                    (format!("{sid}_written"), Some("written"), Some("failed"))
                }
                c::StepKind::Person => (
                    format!("{sid}_decision"),
                    Some("approved"),
                    Some("rejected"),
                ),
                c::StepKind::Child => (format!("{sid}_opened"), None, None),
            };
            let effect = Some(src.effect.as_str())
                .filter(|e| !e.is_empty() && *e != "none" && plain(e))
                .map(str::to_owned);
            let revises = src.kind == c::StepKind::Judgement || src.effect == "write";
            steps.push(Step {
                src,
                claim: format!("{sid}.done"),
                id: sid,
                after,
                kind,
                ok,
                failed,
                revises,
                effect,
                free: Vec::new(),
            });
        }
        if steps.is_empty() {
            return Err("procedure: no step to export".to_owned());
        }
        let mut inputs: Vec<Input<'a>> = Vec::new();
        for parameter in &procedure.parameters {
            let Some(src) = by_id.get(parameter.step.as_str()) else {
                return Err(format!(
                    "parameter `{}`: names step `{}`, which the procedure does not hold",
                    parameter.name, parameter.step
                ));
            };
            let sid = ident(&src.id, "step-");
            let name = ident(&parameter.name, "p-");
            let mut values: Vec<&str> = Vec::new();
            for value in &parameter.values {
                if value_ok(value) && !values.contains(&value.as_str()) {
                    values.push(value);
                }
            }
            if !values.is_empty() {
                if inputs.iter().any(|i| i.id == name) {
                    return Err(format!(
                        "parameter `{}`: its id `{name}` is another's",
                        parameter.name
                    ));
                }
                inputs.push(Input {
                    id: name,
                    step: sid,
                    parameter,
                    values,
                });
            } else if let Some(step) = steps.iter_mut().find(|s| s.id == sid) {
                if step.free.iter().any(|(n, _)| *n == name) {
                    return Err(format!(
                        "parameter `{}`: its id `{name}` is another's",
                        parameter.name
                    ));
                }
                step.free.push((name, parameter));
            }
        }
        Ok(Model {
            procedure,
            id,
            steps,
            inputs,
            refusals,
        })
    }

    fn registration(&self) -> String {
        format!("{}@1", self.id.replace(['.', '_'], "-"))
    }

    fn description(&self) -> String {
        let p = self.procedure;
        let tokens: Vec<String> = p.pattern.iter().map(|t| clean(t)).collect();
        let mut text = format!(
            "Mined by agentspec from the pattern `{}`: {} matches in {} sessions",
            tokens.join(" "),
            p.matches,
            p.sessions.len()
        );
        if !p.query.is_empty() {
            let digest = sha2::Sha256::digest(p.query.as_bytes());
            let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
            let _ = write!(text, " (sq query sha256:{hex})");
        }
        text.push_str(
            ". A draft to harden: each program step runs a placeholder argv that fails until a \
             person writes its command, since the procedure holds no command text; each judgement \
             step runs the model step sdc-node with an instruction file still to write.",
        );
        if self.refusals > 0 {
            let _ = write!(
                text,
                " {} hook refusals inside the matches are not exported as steps.",
                self.refusals
            );
        }
        text
    }

    fn protocol(&self) -> String {
        let mut y = String::new();
        y.push_str("format: protocol/1\n\nprotocol:\n");
        let _ = writeln!(y, "  id: {}", scalar(&self.id));
        y.push_str("  revision: 1\n");
        let _ = writeln!(y, "  description: {}", scalar(&self.description()));
        y.push_str("\nartifacts:\n");
        let _ = writeln!(y, "  {WORK}:");
        let _ = writeln!(
            y,
            "    description: {}",
            scalar("the tree the procedure acts on; its revision is its content digest")
        );
        if !self.inputs.is_empty() {
            y.push_str("\ncase:\n  inputs:\n");
            for input in &self.inputs {
                let _ = writeln!(y, "    {}:", scalar(&input.id));
                let what = format!(
                    "{} of step {}, one of the values observed across the matches",
                    clean(&input.parameter.kind),
                    input.step
                );
                let _ = writeln!(y, "      description: {}", scalar(&what));
                let values: Vec<String> = input.values.iter().map(|v| quoted(v)).collect();
                let _ = writeln!(y, "      values: [{}]", values.join(", "));
            }
        }
        y.push_str("\nevidence_kinds:\n");
        for step in &self.steps {
            let what = match step.src.kind {
                c::StepKind::Program => format!(
                    "the exit of step {}'s program on the work tree: ok on exit 0, failed otherwise",
                    step.id
                ),
                c::StepKind::Judgement => format!(
                    "the record that step {}'s writer ran and left the work tree at a new revision: \
                     written, or failed",
                    step.id
                ),
                c::StepKind::Person => {
                    format!(
                        "a person's decision on step {}: approved or rejected",
                        step.id
                    )
                }
                c::StepKind::Child => {
                    format!("the record that step {} opened its child case", step.id)
                }
            };
            let _ = writeln!(y, "  {}:", scalar(&step.kind));
            let _ = writeln!(y, "    description: {}", scalar(&what));
        }
        y.push_str("\nclaims:\n");
        for step in &self.steps {
            let (what, when) = if step.revises {
                let ok = step.ok.unwrap_or("written");
                (
                    format!(
                        "step {} has written the work tree, at any revision",
                        step.id
                    ),
                    format!(
                        "{{any: [{{first: {{kind: {k}, result: {r}, subject: {WORK}}}}}, \
                         {{after: {{first: {k}, then: {{kind: {k}, result: {r}}}, subject: {WORK}}}}}]}}",
                        k = scalar(&step.kind),
                        r = scalar(ok)
                    ),
                )
            } else {
                let result = step
                    .ok
                    .map(|r| format!(", result: {}", scalar(r)))
                    .unwrap_or_default();
                (
                    format!(
                        "step {} is done on the current revision of the work tree",
                        step.id
                    ),
                    format!(
                        "{{evidence: {{kind: {}{result}, subject: {WORK}}}}}",
                        scalar(&step.kind)
                    ),
                )
            };
            let _ = writeln!(y, "  {}:", scalar(&step.claim));
            let _ = writeln!(y, "    description: {}", scalar(&what));
            let _ = writeln!(y, "    true_when: {when}");
        }
        y.push_str("\nactions:\n");
        for step in &self.steps {
            let _ = writeln!(y, "  {}:", scalar(&step.id));
            let _ = writeln!(y, "    description: {}", scalar(&action_description(step)));
            let author = match step.src.kind {
                c::StepKind::Judgement | c::StepKind::Person => "judgement",
                c::StepKind::Program | c::StepKind::Child => "program",
            };
            let _ = writeln!(y, "    author: {author}");
            if let Some(effect) = &step.effect {
                let _ = writeln!(y, "    effect: {}", scalar(effect));
            }
            if step.revises {
                let _ = writeln!(y, "    revises: [{WORK}]");
            }
            let _ = writeln!(y, "    precondition: {}", precondition(step));
            if step.src.kind == c::StepKind::Person {
                y.push_str("    requires:\n      - capability: owner.approve\n");
            }
            if step.src.kind == c::StepKind::Child {
                let _ = writeln!(
                    y,
                    "    case_open: {{protocol: {}, subject: {WORK}}}",
                    scalar(&self.child_protocol(step))
                );
            }
            if !step.free.is_empty() {
                let _ = writeln!(y, "    arguments: {}", arguments(step));
            }
            let _ = writeln!(
                y,
                "    may_produce:\n      - evidence: {}",
                scalar(&step.kind)
            );
        }
        y.push_str("\noutcomes:\n  done:\n");
        let _ = writeln!(
            y,
            "    description: {}",
            scalar("every step of the procedure is done")
        );
        let all: Vec<String> = self
            .steps
            .iter()
            .map(|s| format!("{{claim: {}}}", scalar(&s.claim)))
            .collect();
        let _ = writeln!(y, "    requires: {{all: [{}]}}", all.join(", "));
        let failing: Vec<String> = self
            .steps
            .iter()
            .filter(|s| s.failed.is_some())
            .map(|s| format!("{{claim: {}, is: \"false\"}}", scalar(&s.claim)))
            .collect();
        if !failing.is_empty() {
            y.push_str("  not_done:\n");
            let _ = writeln!(
                y,
                "    description: {}",
                scalar("a step failed or was rejected (ends the case once its runs are spent)")
            );
            let _ = writeln!(y, "    requires: {{any: [{}]}}", failing.join(", "));
        }
        y
    }

    fn child_protocol(&self, step: &Step) -> String {
        format!("{}.{}.child", self.id, step.id)
    }

    fn bindings(&self) -> String {
        let mut y = String::new();
        y.push_str("format: loom.bindings/1\n");
        // Lowercase letters, digits, `-` and `@1`: plain YAML, as Runbook writes it.
        let _ = writeln!(y, "protocol: {}", self.registration());
        let _ = writeln!(y, "artifacts:\n  {WORK}: {WORK_PATH}");
        y.push_str("actions:\n");
        for step in &self.steps {
            let _ = writeln!(y, "  - action: {}", scalar(&step.id));
            y.push_str("    executor:\n");
            let (producer, from, results) = match step.src.kind {
                c::StepKind::Program => {
                    let argv = placeholder_argv(step);
                    let _ = writeln!(y, "      kind: program\n      argv: [{argv}]");
                    y.push_str("    exits: {error: [126, 127]}\n");
                    (
                        "agentspec-todo",
                        "exit",
                        "{\"0\": ok, timeout: failed, \"*\": failed}",
                    )
                }
                c::StepKind::Judgement => {
                    let instructions =
                        quoted(&format!("{{\"task\":\"instructions/{}.md\"}}", step.id));
                    let _ = writeln!(
                        y,
                        "      kind: program\n      argv: [sdc-node, {}, --adapter, claude-cli, \
                         --max-turns, \"40\", --instructions, {instructions}]",
                        scalar(&step.id)
                    );
                    y.push_str("    exits: {error: [75, 126, 127], not_called: [79]}\n");
                    (
                        "sdc-node",
                        "exit",
                        "{\"0\": written, timeout: failed, \"*\": failed}",
                    )
                }
                c::StepKind::Person => {
                    y.push_str("      kind: person\n      principal: person:owner\n");
                    (
                        "person:owner",
                        "effect",
                        "{performed: approved, rejected: rejected}",
                    )
                }
                c::StepKind::Child => {
                    let file = format!(
                        "{}.yaml",
                        self.child_protocol(step).replace(['.', '_'], "-")
                    );
                    let _ = writeln!(
                        y,
                        "      kind: open\n      name: {}\n      bindings: {}",
                        scalar(&step.id),
                        scalar(&file)
                    );
                    ("runbook", "effect", "{performed: null}")
                }
            };
            let runs_program =
                matches!(step.src.kind, c::StepKind::Program | c::StepKind::Judgement);
            if step.src.kind != c::StepKind::Child {
                let max_runs = step.src.retry_max.max(0) + 1;
                let _ = writeln!(y, "    budget: {{max_runs: {max_runs}}}");
            }
            if runs_program && step.src.timeout_seconds > 0 {
                let seconds = step.src.timeout_seconds.min(MAX_PROGRAM_TIMEOUT);
                let _ = writeln!(y, "    timeout_seconds: {seconds}");
            }
            if let Some(failed) = step.failed.filter(|_| step.src.retry_max > 0) {
                let _ = writeln!(
                    y,
                    "    retry_when:\n      {}: [{}]",
                    scalar(&step.kind),
                    scalar(failed)
                );
            }
            y.push_str("    evidence:\n");
            let _ = writeln!(y, "      - kind: {}", scalar(&step.kind));
            let _ = writeln!(y, "        subject: {WORK}");
            let _ = writeln!(y, "        producer: {}", scalar(producer));
            let _ = writeln!(y, "        from: {from}");
            let _ = writeln!(y, "        results: {results}");
        }
        y
    }
}

/// The steps `step` waits on, with each hook refusal replaced by the steps it waited on.
fn effective_after<'s>(
    step: &'s c::ProcedureStep,
    by_id: &std::collections::BTreeMap<&str, &'s c::ProcedureStep>,
    depth: usize,
) -> Result<Vec<&'s str>, String> {
    if depth > by_id.len() {
        return Err(format!("step `{}`: its `after` runs in a cycle", step.id));
    }
    let mut out = Vec::new();
    for target in &step.after {
        let Some(other) = by_id.get(target.as_str()) else {
            return Err(format!(
                "step `{}`: waits on step `{target}`, which the procedure does not hold",
                step.id
            ));
        };
        if other.token.starts_with(REFUSAL) {
            out.extend(effective_after(other, by_id, depth + 1)?);
        } else {
            out.push(other.id.as_str());
        }
    }
    Ok(out)
}

/// The steps it waits on are done, and its own claim is not yet true.
fn precondition(step: &Step) -> String {
    let claim = scalar(&step.claim);
    let open =
        format!("{{any: [{{claim: {claim}, is: \"false\"}}, {{claim: {claim}, is: unknown}}]}}");
    if step.after.is_empty() {
        return open;
    }
    let mut all: Vec<String> = step
        .after
        .iter()
        .map(|a| format!("{{claim: {}}}", scalar(&format!("{a}.done"))))
        .collect();
    all.push(open);
    format!("{{all: [{}]}}", all.join(", "))
}

fn action_description(step: &Step) -> String {
    let src = step.src;
    let mut text = format!("step {}: {}", step.id, clean(&src.token));
    let mut facts = Vec::new();
    if !src.tool.is_empty() {
        facts.push(format!("tool {}", clean(&src.tool)));
    }
    if !src.category.is_empty() {
        facts.push(format!("category {}", clean(&src.category)));
    }
    if let Some(shape) = src.shape.as_deref().filter(|s| !s.is_empty()) {
        facts.push(format!("shape `{}`", clean(shape)));
    }
    if !facts.is_empty() {
        let _ = write!(text, " ({})", facts.join(", "));
    }
    let _ = write!(text, "; ok in {} of 1000 matched runs", src.ok_permille);
    if src.kind == c::StepKind::Program {
        text.push_str("; its argv in the bindings is a placeholder to replace");
    }
    if src.kind == c::StepKind::Child {
        text.push_str("; the child's protocol and bindings are placeholders to replace");
    }
    text
}

/// The argument schema of a step's free parameters: each an optional string, so `{}` still holds.
fn arguments(step: &Step) -> String {
    let mut properties = serde_json::Map::new();
    for (name, parameter) in &step.free {
        properties.insert(
            name.clone(),
            serde_json::json!({
                "type": "string",
                "description": format!("{}; varies across the matches", clean(&parameter.kind)),
            }),
        );
    }
    serde_json::json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false,
    })
    .to_string()
}

/// An argv that says what is missing and fails: the procedure holds no command text.
fn placeholder_argv(step: &Step) -> String {
    let what = step
        .src
        .shape
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(&step.src.token);
    let message = format!(
        "TODO: write the command of step {} ({}); agentspec holds no command text",
        step.id,
        clean(what)
    );
    let script = format!("echo '{}' >&2; exit 1", message.replace('\'', "'\\''"));
    format!("sh, -c, {}", quoted(&script))
}

/// `raw` as an identifier: lowercase ASCII letters, digits, `.`, `_` and `-`, starting with a
/// letter (`prefix` is put in front otherwise).
fn ident(raw: &str, prefix: &str) -> String {
    let body: String = raw
        .chars()
        .map(|ch| {
            let ch = ch.to_ascii_lowercase();
            if ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect();
    if body.starts_with(|ch: char| ch.is_ascii_lowercase()) {
        body
    } else {
        format!("{prefix}{body}")
    }
}

/// A case input value: an identifier without white space or a control character.
fn value_ok(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}

/// Text with control characters (a line break) replaced by a space.
fn clean(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect()
}

/// Whether `text` can be written as a plain YAML scalar, block or flow, that reads back as itself.
fn plain(text: &str) -> bool {
    const WORDS: [&str; 11] = [
        "true", "false", "null", "yes", "no", "on", "off", "y", "n", "nan", "inf",
    ];
    text.starts_with(|ch: char| ch.is_ascii_alphabetic())
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        && !WORDS.contains(&text.to_ascii_lowercase().as_str())
}

/// `text` as a YAML scalar: plain when that reads back the same, double-quoted otherwise.
fn scalar(text: &str) -> String {
    if plain(text) {
        text.to_owned()
    } else {
        quoted(text)
    }
}

/// A double-quoted YAML scalar: a JSON string is one.
fn quoted(text: &str) -> String {
    serde_json::Value::String(text.to_owned()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn step(
        id: &str,
        kind: c::StepKind,
        token: &str,
        effect: &str,
        after: &[&str],
    ) -> c::ProcedureStep {
        c::ProcedureStep {
            id: id.to_owned(),
            kind,
            token: token.to_owned(),
            tool: token.split(':').next().unwrap_or(token).to_owned(),
            category: if effect == "write" { "edit" } else { "read" }.to_owned(),
            shape: None,
            effect: effect.to_owned(),
            after: after.iter().map(|a| (*a).to_owned()).collect(),
            retry_max: 0,
            timeout_seconds: 0,
            ok_permille: 1000,
        }
    }

    /// Read, Edit, then `exec:test` with a retry and a timeout: the example of the report.
    fn example() -> c::Procedure {
        let mut test = step("s3", c::StepKind::Program, "exec:test", "read", &["s2"]);
        test.tool = "Bash".to_owned();
        test.category = "test".to_owned();
        test.shape = Some("cargo test".to_owned());
        test.retry_max = 2;
        test.timeout_seconds = 240;
        test.ok_permille = 750;
        c::Procedure {
            format: "agentspec-procedure/1".to_owned(),
            id: "read-edit-test".to_owned(),
            description: "SECRET-DESCRIPTION".to_owned(),
            pattern: vec!["Read".into(), "Edit".into(), "exec:test".into()],
            matches: 12,
            sessions: vec![
                "SESSION-ID-A".into(),
                "SESSION-ID-B".into(),
                "SESSION-ID-C".into(),
            ],
            parameters: vec![
                c::ProcedureParameter {
                    name: "runner".into(),
                    step: "s3".into(),
                    kind: "flag_value".into(),
                    values: vec!["nextest".into(), "test".into()],
                },
                c::ProcedureParameter {
                    name: "package".into(),
                    step: "s3".into(),
                    kind: "positional".into(),
                    values: vec![],
                },
            ],
            steps: vec![
                step("s1", c::StepKind::Program, "Read", "read", &[]),
                step("s2", c::StepKind::Judgement, "Edit", "write", &["s1"]),
                test,
            ],
            query: "seq(Read, Edit, exec:test)".to_owned(),
        }
    }

    /// A fresh directory under the build's `target/`, never `/tmp`.
    fn scratch(name: &str) -> PathBuf {
        let exe = std::env::current_exe().expect("test binary path");
        let dir = exe
            .parent()
            .expect("deps directory")
            .join("export-tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn export_writes_both_files_with_steps_order_and_outcome() {
        let out = scratch("files");
        let result = export(&example(), &out).expect("export");
        let protocol = std::fs::read_to_string(&result.protocol).unwrap();
        let bindings = std::fs::read_to_string(&result.bindings).unwrap();
        assert!(result.protocol.ends_with("read-edit-test.protocol.yaml"));
        assert!(result.bindings.ends_with("read-edit-test.yaml"));
        for needed in [
            "format: protocol/1",
            "  id: read-edit-test",
            "pattern `Read Edit exec:test`: 12 matches in 3 sessions",
            "  s1:\n    description: \"step s1: Read",
            "    author: judgement\n    effect: write\n    revises: [work]",
            "precondition: {all: [{claim: s2.done}, {any: [{claim: s3.done, is: \"false\"}",
            "  s1_run:",
            "true_when: {evidence: {kind: s3_run, result: ok, subject: work}}",
            "true_when: {any: [{first: {kind: s2_written, result: written, subject: work}}",
            "requires: {all: [{claim: s1.done}, {claim: s2.done}, {claim: s3.done}]}",
            "values: [\"nextest\", \"test\"]",
            "\"package\":{\"description\":\"positional; varies across the matches\"",
        ] {
            assert!(
                protocol.contains(needed),
                "protocol lacks {needed:?}:\n{protocol}"
            );
        }
        for needed in [
            "protocol: read-edit-test@1",
            "argv: [sh, -c, \"echo 'TODO: write the command of step s3 (cargo test)",
            "argv: [sdc-node, s2, --adapter, claude-cli",
            "    budget: {max_runs: 3}\n    timeout_seconds: 240\n    retry_when:\n      s3_run: [failed]",
            "results: {\"0\": ok, timeout: failed, \"*\": failed}",
        ] {
            assert!(
                bindings.contains(needed),
                "bindings lack {needed:?}:\n{bindings}"
            );
        }
        for text in [&protocol, &bindings] {
            assert!(!text.contains("SESSION-ID"), "a session id leaked");
            assert!(
                !text.contains("SECRET-DESCRIPTION"),
                "the free description leaked"
            );
        }
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn export_is_deterministic() {
        let (a, b) = (scratch("same-a"), scratch("same-b"));
        let ra = export(&example(), &a).expect("export a");
        let rb = export(&example(), &b).expect("export b");
        assert_eq!(
            std::fs::read(&ra.protocol).unwrap(),
            std::fs::read(&rb.protocol).unwrap()
        );
        assert_eq!(
            std::fs::read(&ra.bindings).unwrap(),
            std::fs::read(&rb.bindings).unwrap()
        );
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }

    #[test]
    fn person_child_and_refusal_steps_map_as_section_7_2_says() {
        let mut procedure = example();
        procedure.id = "Mined Proc/1".to_owned();
        procedure.steps.push(step(
            "4",
            c::StepKind::Program,
            "refuse:Hook",
            "none",
            &["s3"],
        ));
        procedure.steps.push(step(
            "s5",
            c::StepKind::Person,
            "prompt:Human",
            "none",
            &["4"],
        ));
        procedure
            .steps
            .push(step("s6", c::StepKind::Child, "Spawn", "none", &["s5"]));
        let out = scratch("kinds");
        let result = export(&procedure, &out).expect("export");
        assert!(result.protocol.ends_with("mined-proc-1.protocol.yaml"));
        let protocol = std::fs::read_to_string(&result.protocol).unwrap();
        let bindings = std::fs::read_to_string(&result.bindings).unwrap();
        assert!(!protocol.contains("step-4"), "the refusal is not a step");
        assert!(protocol.contains("1 hook refusals inside the matches are not exported"));
        assert!(protocol.contains("precondition: {all: [{claim: s3.done}, {any: [{claim: s5.done"));
        assert!(protocol.contains("requires:\n      - capability: owner.approve"));
        assert!(protocol.contains("case_open: {protocol: mined-proc-1.s6.child, subject: work}"));
        assert!(bindings.contains("kind: person\n      principal: person:owner"));
        assert!(bindings.contains("results: {performed: approved, rejected: rejected}"));
        assert!(
            bindings
                .contains("kind: open\n      name: s6\n      bindings: mined-proc-1-s6-child.yaml")
        );
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn a_step_waiting_on_an_unknown_step_is_refused() {
        let mut procedure = example();
        procedure.steps[1].after = vec!["nope".into()];
        let err = export(&procedure, &scratch("unknown")).unwrap_err();
        assert!(err.contains("`nope`"), "{err}");
    }

    #[test]
    fn canon_validates_the_export_when_installed() {
        let mut procedure = example();
        procedure.steps.push(step(
            "s4",
            c::StepKind::Person,
            "prompt:Human",
            "none",
            &["s3"],
        ));
        procedure
            .steps
            .push(step("s5", c::StepKind::Child, "Spawn", "none", &["s4"]));
        let out = scratch("canon");
        let result = export(&procedure, &out).expect("export");
        if on_path("canon").is_none() {
            assert!(!result.validated);
            assert_eq!(result.detail, "canon not installed");
            eprintln!("skipped: canon is not on PATH");
        } else {
            assert!(result.validated, "canon refused: {}", result.detail);
        }
        let _ = std::fs::remove_dir_all(&out);
    }
}
