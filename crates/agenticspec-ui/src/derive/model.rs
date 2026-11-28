//! What the derivation reads: the architecture tree (the document and the documents its nested
//! parts reference), the block panels of `ui/views/blocks.ui.yaml` (each widget's text, copied
//! byte for byte), the compiled blocks, and, when given, the contract and the port catalogue.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

/// A part of one document.
#[derive(Clone, Debug)]
pub struct Part {
    /// Its name in its document.
    pub name: String,
    /// Its path from the top document, `team.lead`.
    pub path: String,
    pub block: String,
    pub kind: Option<String>,
    /// How many copies run; 1 when the part names no count.
    pub count: u64,
    pub per: Option<String>,
    pub acts_for: Option<String>,
    /// Every other setting, by name, as written (a marked value as its `value`).
    pub settings: Vec<(String, Value)>,
    /// The level a nested architecture part opens, by its index in [`Tree::levels`].
    pub nested: Option<usize>,
}

/// A link of one document.
#[derive(Clone, Debug)]
pub struct Link {
    pub from: String,
    pub to: String,
    pub carries: String,
    pub over: Option<String>,
}

/// One document of the tree: the top one or a nested one.
#[derive(Clone, Debug)]
pub struct Level {
    /// The path of the nested part that opens it; empty for the top document.
    pub path: String,
    pub name: String,
    pub summary: String,
    /// The file, relative to the repository root where it lies below it.
    pub file: String,
    pub doc: Value,
    pub parts: Vec<Part>,
    pub links: Vec<Link>,
    pub border: Vec<String>,
    /// The copies of the nested part that opens it; 1 for the top document.
    pub count: u64,
}

impl Level {
    pub fn part(&self, name: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.name == name)
    }
}

/// The architecture tree, top document first, each nested document after its parent.
pub struct Tree {
    pub levels: Vec<Level>,
    /// Every file read, relative to the root, with its bytes.
    pub files: Vec<(String, Vec<u8>)>,
}

/// A marked value (`{ value, vary }`) as its value; any other value as it is.
pub fn plain(v: &Value) -> Value {
    match v {
        Value::Object(o) if o.contains_key("vary") => {
            o.get("value").cloned().unwrap_or(Value::Null)
        }
        other => other.clone(),
    }
}

/// A value as text for a table cell.
pub fn text(v: &Value) -> String {
    match plain(v) {
        Value::String(s) => s,
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// `path` relative to `root`, with `/`, when it lies below it; else as given.
pub fn relative_to_root(root: &Path, path: &Path) -> String {
    let abs = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let (r, p) = (abs(root), abs(path));
    p.strip_prefix(&r)
        .map(|x| x.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn read_doc(path: &Path) -> Result<(Value, Vec<u8>), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = String::from_utf8_lossy(&bytes);
    let doc: Value =
        serde_yaml_ng::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if doc.get("format").and_then(Value::as_str) != Some("agenticspec-architecture/1") {
        return Err(format!(
            "{}: not an agenticspec-architecture/1 document",
            path.display()
        ));
    }
    Ok((doc, bytes))
}

fn names(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

impl Tree {
    /// Reads the document at `path` and every document its nested parts reference, relative to
    /// the document that references it. A cycle of references is refused.
    pub fn read(root: &Path, path: &Path) -> Result<Tree, String> {
        let mut tree = Tree {
            levels: Vec::new(),
            files: Vec::new(),
        };
        let mut open = Vec::new();
        tree.level(root, path, String::new(), 1, &mut open)?;
        Ok(tree)
    }

    fn level(
        &mut self,
        root: &Path,
        path: &Path,
        prefix: String,
        count: u64,
        open: &mut Vec<PathBuf>,
    ) -> Result<usize, String> {
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if open.contains(&canonical) {
            return Err(format!(
                "{}: the nested architectures reference each other in a cycle",
                path.display()
            ));
        }
        let (doc, bytes) = read_doc(path)?;
        let file = relative_to_root(root, path);
        if !self.files.iter().any(|(f, _)| *f == file) {
            self.files.push((file.clone(), bytes));
        }
        let index = self.levels.len();
        let name = doc["architecture"]
            .as_str()
            .unwrap_or("architecture")
            .to_string();
        let mut parts = Vec::new();
        for (pname, p) in doc["parts"].as_object().into_iter().flatten() {
            let block = p["block"].as_str().unwrap_or("").to_string();
            let mut settings = Vec::new();
            for (k, v) in p.as_object().into_iter().flatten() {
                if !matches!(
                    k.as_str(),
                    "block" | "kind" | "count" | "per" | "acts_for" | "ref" | "digest"
                ) {
                    settings.push((k.clone(), plain(v)));
                }
            }
            let count = plain(&p["count"]).as_u64().unwrap_or(1).max(1);
            parts.push(Part {
                name: pname.clone(),
                path: if prefix.is_empty() {
                    pname.clone()
                } else {
                    format!("{prefix}.{pname}")
                },
                block,
                kind: p["kind"].as_str().map(str::to_string),
                count,
                per: p["per"].as_str().map(str::to_string),
                acts_for: p["acts_for"].as_str().map(str::to_string),
                settings,
                nested: None,
            });
        }
        let links = doc["links"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|l| Link {
                from: l["from"].as_str().unwrap_or("").to_string(),
                to: l["to"].as_str().unwrap_or("").to_string(),
                carries: l["carries"].as_str().unwrap_or("").to_string(),
                over: l["over"].as_str().map(str::to_string),
            })
            .collect();
        self.levels.push(Level {
            path: prefix.clone(),
            name,
            summary: doc["summary"].as_str().unwrap_or("").trim().to_string(),
            file,
            border: names(&doc["border"]),
            doc: doc.clone(),
            parts,
            links,
            count,
        });
        open.push(canonical);
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let nested: Vec<(usize, String, u64)> = self.levels[index]
            .parts
            .iter()
            .enumerate()
            .filter(|(_, p)| p.block == "architecture")
            .filter_map(|(i, p)| {
                doc["parts"][&p.name]["ref"]
                    .as_str()
                    .map(|r| (i, r.to_string(), p.count))
            })
            .collect();
        for (i, r, n) in nested {
            let child_prefix = self.levels[index].parts[i].path.clone();
            let child = self.level(root, &dir.join(&r), child_prefix, n, open)?;
            self.levels[index].parts[i].nested = Some(child);
        }
        open.pop();
        Ok(index)
    }
}

/// One block panel of `ui/views/blocks.ui.yaml`.
#[derive(Clone, Debug)]
pub struct Panel {
    /// The widget, named as its view: `agentic.harness.Turns`.
    pub widget: String,
    /// Its domain: `agentic.harness`.
    pub domain: String,
    /// The widget's text in the document, every line of it, copied byte for byte.
    pub text: String,
    /// The events its body takes; it then takes a `channel` argument.
    pub events: Vec<String>,
    /// The title the gallery gives it.
    pub title: String,
    /// The row key of its read.
    pub key: String,
    /// The columns of its collection as written.
    pub columns: Vec<Value>,
    /// The arguments of its view's required parameters in the gallery, as expressions.
    pub args: Vec<(String, String)>,
    /// The parameters its read binds, each as the expression written (`args.addressee`).
    pub read_params: Vec<(String, String)>,
}

/// The block panels, in the order of the document (by layer, then block, then view).
pub struct Panels {
    pub list: Vec<Panel>,
}

impl Panels {
    pub fn get(&self, widget: &str) -> Option<&Panel> {
        self.list.iter().find(|p| p.widget == widget)
    }

    pub fn of_domain<'a>(&'a self, domain: &'a str) -> impl Iterator<Item = &'a Panel> + 'a {
        self.list.iter().filter(move |p| p.domain == domain)
    }

    /// The domains that have panels, in the document's order.
    pub fn domains(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for p in &self.list {
            if !out.contains(&p.domain) {
                out.push(p.domain.clone());
            }
        }
        out
    }

    /// Reads the panels document: its structure through YAML, each widget's text by its lines.
    pub fn read(text: &str) -> Result<Panels, String> {
        let doc: Value =
            serde_yaml_ng::from_str(text).map_err(|e| format!("ui/views/blocks.ui.yaml: {e}"))?;
        let texts = widget_texts(text);
        let mut args: BTreeMap<String, (String, Vec<(String, String)>)> = BTreeMap::new();
        for (_, page) in doc["pages"].as_object().into_iter().flatten() {
            for s in page["sections"].as_array().into_iter().flatten() {
                let Some(w) = s["component"].as_str() else {
                    continue;
                };
                let title = s["title"].as_str().unwrap_or(w).to_string();
                let a: Vec<(String, String)> = s["args"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(k, _)| k.as_str() != "channel")
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            v.as_str().map_or_else(|| v.to_string(), str::to_string),
                        )
                    })
                    .collect();
                args.insert(w.to_string(), (title, a));
            }
        }
        let mut list = Vec::new();
        for (name, body) in &texts {
            let w = &doc["widgets"][name.as_str()];
            let rows = &w["body"][0];
            let events = rows["live"]["on"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            let (title, a) = args
                .get(name)
                .cloned()
                .unwrap_or_else(|| (name.clone(), Vec::new()));
            list.push(Panel {
                widget: name.clone(),
                domain: name
                    .rsplit_once('.')
                    .map_or(name.as_str(), |(d, _)| d)
                    .to_string(),
                text: body.clone(),
                events,
                title,
                key: rows["reads"]["key"].as_str().unwrap_or("").to_string(),
                columns: rows["columns"].as_array().cloned().unwrap_or_default(),
                args: a,
                read_params: rows["reads"]["params"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect(),
            });
        }
        Ok(Panels { list })
    }
}

/// Each widget of the document's `widgets:` map with its lines: from its name line (two spaces of
/// indent) to the line before the next name line or the first line indented less than four.
fn widget_texts(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if !inside {
            inside = bare == "widgets:";
            continue;
        }
        if let Some(rest) = bare.strip_prefix("  ")
            && !rest.starts_with(' ')
            && let Some(name) = rest.strip_suffix(':')
        {
            out.push((name.to_string(), line.to_string()));
            continue;
        }
        if bare.starts_with("    ") {
            if let Some((_, t)) = out.last_mut() {
                t.push_str(line);
            }
            continue;
        }
        break;
    }
    out
}

/// The compiled blocks, read where the derivation needs them.
pub struct Model<'a> {
    pub ir: &'a Value,
}

impl Model<'_> {
    pub fn system(&self) -> &str {
        self.ir["system"].as_str().unwrap_or("")
    }

    /// The domain a block names: the domain of that name, else the domain of the entity of that
    /// name (`pool` is the `Pool` of `resource`).
    pub fn domain_of(&self, word: &str) -> Option<String> {
        let system = self.system();
        let direct = format!("{system}.{word}");
        if self.ir["domains"].get(&direct).is_some() {
            return Some(direct);
        }
        let want: String = word
            .chars()
            .filter(|c| *c != '_')
            .collect::<String>()
            .to_lowercase();
        self.ir["entities"]
            .as_object()
            .into_iter()
            .flatten()
            .find(|(name, _)| {
                name.rsplit('.')
                    .next()
                    .is_some_and(|n| n.to_lowercase() == want)
            })
            .and_then(|(name, _)| name.rsplit_once('.').map(|(d, _)| d.to_string()))
    }

    /// The entity a word names, qualified: `agent` is `agentic.agent.Agent`.
    pub fn entity_of(&self, word: &str) -> Option<String> {
        let want: String = word
            .chars()
            .filter(|c| *c != '_')
            .collect::<String>()
            .to_lowercase();
        self.ir["entities"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(n, _)| n)
            .find(|n| {
                n.rsplit('.')
                    .next()
                    .is_some_and(|x| x.to_lowercase() == want)
            })
            .cloned()
    }

    pub fn view_source(&self, view: &str) -> Option<&str> {
        self.ir["views"][view]["source"].as_str()
    }

    pub fn domain_label(&self, domain: &str) -> String {
        self.ir["domains"][domain]["naming"]["display"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| domain.rsplit('.').next().unwrap_or(domain).to_string())
    }

    pub fn command_label(&self, command: &str) -> String {
        self.ir["commands"][command]["naming"]["display"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| command.rsplit('.').next().unwrap_or(command).to_string())
    }

    pub fn command_inputs(&self, command: &str) -> Vec<String> {
        self.ir["commands"][command]["input"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|f| f["name"].as_str())
            .map(str::to_string)
            .collect()
    }

    pub fn has_command(&self, command: &str) -> bool {
        self.ir["commands"].get(command).is_some()
    }

    /// The enum variants of a declared type, when it is an enum.
    pub fn variants(&self, ty: &str) -> BTreeSet<String> {
        self.ir["types"][ty]["body"]["variants"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    }
}

/// One port of the catalogue (`agenticspec-ports/1`, `blocks/ports.json`).
#[derive(Clone, Debug)]
pub struct Port {
    pub name: String,
    pub block: String,
    pub kind: String,
    /// The commands the port's intent is carried out by, qualified. Read from `intent_command`
    /// (a name or a list), else the first command of `protocol` in the port's block.
    pub intent: Vec<String>,
    /// The catalogue names the intent command itself (`intent_command`), or the principal's
    /// answers (`reports` the principal actor may issue); a principal port becomes a row action
    /// only then, since the first command of a protocol is the asker's.
    pub explicit: bool,
    /// The entity the port opens: the entity of its waiting state.
    pub opens: Option<String>,
}

/// The actor of `actors` in the port catalogue whose commands a principal part may send.
const PRINCIPAL_ACTOR: &str = "agentic.architecture.Principal";

/// The port catalogue, by port name.
pub fn ports(doc: &Value, model: &Model<'_>) -> BTreeMap<String, Port> {
    let mut out = BTreeMap::new();
    let principal_may: BTreeSet<&str> = doc["actors"][PRINCIPAL_ACTOR]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    for p in doc["ports"].as_array().into_iter().flatten() {
        let name = p["name"].as_str().unwrap_or("").to_string();
        let block = p["block"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| name.split('.').next().unwrap_or("").to_string());
        let domain = model.domain_of(&block).unwrap_or_default();
        let qualify = |c: &str| {
            if c.contains('.') {
                c.to_string()
            } else {
                format!("{domain}.{c}")
            }
        };
        let kind = p["kind"].as_str().unwrap_or("").to_string();
        // A principal port of `agenticspec sim ports` names the commands that answer it in
        // `reports`; those the principal actor may issue are the principal's to send.
        let answers: Vec<String> = if kind == "Principal" {
            p["reports"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(qualify)
                .filter(|c| principal_may.contains(c.as_str()))
                .collect()
        } else {
            Vec::new()
        };
        let explicit = p.get("intent_command").is_some_and(|v| !v.is_null()) || !answers.is_empty();
        let intent: Vec<String> = match &p["intent_command"] {
            Value::String(s) => vec![qualify(s)],
            Value::Array(xs) => xs.iter().filter_map(Value::as_str).map(qualify).collect(),
            _ if !answers.is_empty() => answers,
            _ => p["protocol"]
                .as_str()
                .and_then(|s| {
                    s.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .find(|w| !w.is_empty())
                })
                .map(|c| vec![qualify(c)])
                .unwrap_or_default(),
        };
        out.insert(
            name.clone(),
            Port {
                name,
                block,
                kind,
                explicit,
                intent,
                opens: p["waiting_state"]
                    .as_str()
                    .and_then(|s| s.rsplit_once('.'))
                    .map(|(e, _)| e.to_string()),
            },
        );
    }
    out
}

/// The contract of an architecture (`agenticspec-contract/1`): the ports each part uses, in the
/// contract's order, and the port each link stands for (`owner -> operator carries prompt over
/// chat`).
#[derive(Default)]
pub struct Contract {
    pub uses: Vec<(String, String)>,
    pub links: BTreeMap<String, String>,
}

impl Contract {
    pub fn read(doc: &Value) -> Contract {
        let mut c = Contract::default();
        for p in doc["ports"].as_array().into_iter().flatten() {
            if let (Some(port), Some(part)) = (p["port"].as_str(), p["part"].as_str()) {
                c.uses.push((part.to_string(), port.to_string()));
            }
        }
        for i in doc["invariants"].as_array().into_iter().flatten() {
            if let (Some(link), Some(port)) = (i["link"].as_str(), i["port"].as_str()) {
                c.links.insert(link.to_string(), port.to_string());
            }
        }
        c
    }

    pub fn ports_of<'a>(&'a self, part: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.uses
            .iter()
            .filter(move |(p, _)| p == part)
            .map(|(_, port)| port.as_str())
    }
}

/// A link written as the contract names it.
pub fn link_text(l: &Link) -> String {
    let mut s = format!("{} -> {} carries {}", l.from, l.to, l.carries);
    if let Some(o) = &l.over {
        s.push_str(&format!(" over {o}"));
    }
    s
}
