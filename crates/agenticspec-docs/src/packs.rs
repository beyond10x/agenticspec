//! The pages of the packs and the fields: one page per block of a pack, the index of the packs,
//! the index of the fields and one page per field with the architectures it lists.
//!
//! Made from the whole system, the root `ess-inputs.yaml` compiled, so a pack's page can link the
//! core blocks it depends on. A repository without that file or without `packs/` has none.

use crate::SOURCE_BASE;
use crate::arch;
use crate::blocks::{self, Ctx, GENERATED_NOTE, front};
use crate::kpis::Catalogue;
use crate::md::{first_sentence, text, yaml};
use crate::model::{self, Block, Field, Pack, layer_edges};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// The files of the packs and fields, and their items of the sidebar.
pub struct Output {
    pub files: BTreeMap<String, String>,
    pub sidebar_packs: Vec<Value>,
    pub sidebar_fields: Vec<Value>,
}

fn tree(path: &str) -> String {
    format!("{}{path}", SOURCE_BASE.replace("/blob/", "/tree/"))
}

fn note(sources: &str) -> String {
    GENERATED_NOTE.replace("the specification in blocks/", sources)
}

fn count(ir: &Value, b: &Block, kind: &str) -> usize {
    ir["domains"][&b.domain][kind]
        .as_array()
        .map_or(0, |a| a.len())
}

/// The core catalogue and every `packs/<pack>/kpis.yaml`, in one.
fn catalogue(root: &Path, core: &Catalogue, packs: &[Pack]) -> Result<Catalogue, String> {
    let mut all = Catalogue {
        kpis: core.kpis.clone(),
    };
    for p in packs {
        let rel = format!("packs/{}/kpis.yaml", p.name);
        let path = root.join(&rel);
        if !path.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{rel}: {e}"))?;
        all.kpis.extend(
            Catalogue::parse(&text)
                .map_err(|e| format!("{rel}: {e}"))?
                .kpis,
        );
    }
    Ok(all)
}

/// The pages of the packs and the fields, or `None` without a root `ess-inputs.yaml` and `packs/`.
pub fn generate(root: &Path, core_catalogue: &Catalogue) -> Result<Option<Output>, String> {
    if !root.join("ess-inputs.yaml").exists() || !root.join("packs").is_dir() {
        return Ok(None);
    }
    let ir = crate::compile_at(root)?;
    let (core, packs) = model::system(root, &ir)?;
    let catalogue = catalogue(root, core_catalogue, &packs)?;
    let all: Vec<Block> = core
        .iter()
        .chain(packs.iter().flat_map(|p| p.blocks.iter()))
        .cloned()
        .collect();
    let edges = layer_edges(&ir);
    let ctx = Ctx {
        ir: &ir,
        blocks: &all,
        edges: &edges,
        catalogue: &catalogue,
        planned: &[],
    };
    let fields = model::fields(root)?;

    let mut files = BTreeMap::new();
    for b in packs.iter().flat_map(|p| p.blocks.iter()) {
        files.insert(format!("{}.md", b.doc_id()), blocks::page(&ctx, b));
    }
    files.insert(
        "docs/packs/index.md".to_string(),
        packs_index(&ir, &catalogue, &packs, &fields),
    );
    files.insert("docs/fields/index.md".to_string(), fields_index(&fields));

    let mut sidebar_fields = Vec::new();
    for f in &fields {
        let archs = arch::load_field(&f.name, &f.architectures, root)?;
        files.insert(
            format!("docs/fields/{}/index.md", f.name),
            field_page(f, &packs, &core, &archs, root)?,
        );
        let mut items = Vec::new();
        for a in &archs {
            files.insert(format!("docs/{}.md", a.slug), arch::page(a, &all));
            files.insert(format!("website/static/{}.html", a.slug), arch::html(a));
            if let Ok(shown) = &a.contract {
                files.insert(
                    format!("website/static/{}.contract.yaml", a.slug),
                    shown.contract.text(),
                );
            }
            items.push(json!(format!("docs/{}", a.slug)));
        }
        sidebar_fields.push(json!({
            "type": "category",
            "label": f.name,
            "collapsed": true,
            "link": {"type": "doc", "id": format!("docs/fields/{}/index", f.name)},
            "items": items,
        }));
    }
    let sidebar_packs = packs
        .iter()
        .map(|p| {
            json!({
                "type": "category",
                "label": p.name,
                "collapsed": true,
                "items": p.blocks.iter().map(Block::doc_id).collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(Some(Output {
        files,
        sidebar_packs,
        sidebar_fields,
    }))
}

fn packs_index(ir: &Value, catalogue: &Catalogue, packs: &[Pack], fields: &[Field]) -> String {
    let mut s = String::new();
    front(
        &mut s,
        &[
            ("title", yaml("Packs")),
            ("sidebar_label", yaml("Overview")),
            ("slug", "/packs".into()),
            (
                "description",
                yaml(
                    "Blocks more than one field of work reuses, above the core and named without any product.",
                ),
            ),
            (
                "lede",
                yaml(
                    "Blocks more than one field of work reuses: source control, tracking, file tools, a shell. Each pack depends on the core and on the packs it declares, never on a field.",
                ),
            ),
            ("source", yaml("packs/")),
            ("source_url", yaml(&tree("packs"))),
            ("hide_table_of_contents", "true".into()),
        ],
    );
    s.push_str(&note("packs/ and the root ess-inputs.yaml"));
    s.push_str(
        "Three tiers, each in its own directories, with dependencies pointing down only:\n\n\
- the [core](../blocks/index.md) (`blocks/`): what every agentic system has, whatever its work;\n\
- packs (`packs/<pack>/`): concepts more than one field reuses; a pack depends on the core and on the packs its `pack.yaml` lists under `uses:`;\n\
- [fields](../fields/index.md) (`fields/<field>/`): one kind of work; a field lists the packs it uses and adds a block only for a concept no other field shares.\n\n\
`task check:layers` refuses a core block that depends on a pack or a field, a pack that depends on a field or on a pack it does not list, and a field that depends on another field. \
The whole system validates from the root `ess-inputs.yaml`; the core still validates alone from `blocks/ess-inputs.yaml`.\n\n",
    );
    for p in packs {
        let _ = writeln!(s, "## {}\n", p.name);
        let _ = writeln!(s, "{}\n", text(&p.summary));
        let uses = if p.uses.is_empty() {
            "the core".to_string()
        } else {
            format!(
                "the core and {}",
                p.uses
                    .iter()
                    .map(|u| format!("[{u}](#{u})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let users: Vec<String> = fields
            .iter()
            .filter(|f| f.packs.contains(&p.name))
            .map(|f| format!("[{0}](../fields/{0}/index.md)", f.name))
            .collect();
        let _ = writeln!(
            s,
            "Namespace `{}`. Depends on {uses}. Used by {}.\n",
            p.namespace,
            if users.is_empty() {
                "no field yet".to_string()
            } else {
                format!(
                    "the field{} {}",
                    if users.len() == 1 { "" } else { "s" },
                    users.join(", ")
                )
            }
        );
        s.push_str("| block | what it is | entities | commands | KPIs |\n|---|---|---|---|---|\n");
        for b in &p.blocks {
            let _ = writeln!(
                s,
                "| [{}]({}/{}.md) | {} | {} | {} | {} |",
                b.name,
                b.dir,
                b.page,
                text(&first_sentence(&b.summary)),
                count(ir, b, "entities"),
                count(ir, b, "commands"),
                catalogue.of(&b.name).count()
            );
        }
        s.push('\n');
        if !p.scenarios.is_empty() {
            s.push_str("Authored scenarios, for the paths synthesis does not reach: ");
            let links: Vec<String> = p
                .scenarios
                .iter()
                .map(|f| {
                    let name = f.rsplit('/').next().unwrap_or(f).trim_end_matches(".yaml");
                    format!("[{}]({SOURCE_BASE}{f})", text(name))
                })
                .collect();
            let _ = writeln!(s, "{}.\n", links.join(", "));
        }
    }
    s
}

fn fields_index(fields: &[Field]) -> String {
    let mut s = String::new();
    front(
        &mut s,
        &[
            ("title", yaml("Fields")),
            ("sidebar_label", yaml("Overview")),
            ("slug", "/fields".into()),
            (
                "description",
                yaml(
                    "Kinds of agentic work: the packs each uses, its KPIs and its example architectures.",
                ),
            ),
            ("source", yaml("fields/")),
            ("source_url", yaml(&tree("fields"))),
        ],
    );
    s.push_str(&note("fields/"));
    s.push_str(
        "A field is one kind of work. It lists the [packs](../packs/index.md) it uses and holds its example architectures; \
it adds a block only for a concept no other field shares, and never depends on another field.\n\n",
    );
    s.push_str("| field | what it is | packs | architectures |\n|---|---|---|---|\n");
    for f in fields {
        let _ = writeln!(
            s,
            "| [{0}]({0}/index.md) | {1} | {2} | {3} |",
            f.name,
            text(&first_sentence(&f.summary)),
            f.packs
                .iter()
                .map(|p| format!("[{p}](../packs/index.md#{p})"))
                .collect::<Vec<_>>()
                .join(", "),
            f.architectures.len()
        );
    }
    s
}

/// A `## title` after one blank line, whatever the text before it ends with.
fn heading(s: &mut String, title: &str) {
    while s.ends_with('\n') {
        s.pop();
    }
    let _ = write!(s, "\n\n## {title}\n\n");
}

fn field_page(
    f: &Field,
    packs: &[Pack],
    core: &[Block],
    archs: &[arch::Architecture],
    root: &Path,
) -> Result<String, String> {
    let mut s = String::new();
    let source = format!("fields/{}/", f.name);
    front(
        &mut s,
        &[
            ("title", yaml(&f.name)),
            ("sidebar_label", yaml("Overview")),
            ("slug", format!("/fields/{}", f.name)),
            ("description", yaml(&first_sentence(&f.summary))),
            ("kicker", yaml("Field")),
            ("source", yaml(&source)),
            ("source_url", yaml(&tree(&format!("fields/{}", f.name)))),
        ],
    );
    s.push_str(&note(&source));
    let _ = writeln!(s, "{}\n", text(&f.summary));
    s.push_str("## Packs\n\n");
    for name in &f.packs {
        let blocks: Vec<String> = packs
            .iter()
            .filter(|p| &p.name == name)
            .flat_map(|p| p.blocks.iter())
            .map(|b| format!("[{}](../../packs/{}/{}.md)", b.name, b.dir, b.page))
            .collect();
        let _ = writeln!(
            s,
            "- [{name}](../../packs/index.md#{name}): {}",
            if blocks.is_empty() {
                "no such pack".to_string()
            } else {
                blocks.join(", ")
            }
        );
    }
    s.push_str("\n## Architectures\n\n");
    if archs.is_empty() {
        s.push_str("None yet.\n");
    }
    for a in archs {
        let summary = a.doc["summary"].as_str().unwrap_or("");
        let _ = writeln!(
            s,
            "- [{0}]({0}.md): {1}",
            a.name,
            text(&first_sentence(summary))
        );
    }
    if !f.profiles.is_empty() {
        heading(&mut s, "Behaviour profiles");
        s.push_str("The simulator's input for this field: how long turns and answers take, how often work fails and what model calls cost. The experiments of the [catalog](../../catalog/index.md) that name this field run under them.\n\n");
        for p in &f.profiles {
            let name = p.rsplit('/').next().unwrap_or(p);
            let _ = writeln!(s, "- [{}]({SOURCE_BASE}{p})", text(name));
        }
    }
    if let Some(rel) = &f.kpis {
        let text_of = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
        let catalogue = Catalogue::parse(&text_of).map_err(|e| format!("{rel}: {e}"))?;
        heading(&mut s, "KPIs");
        let _ = writeln!(
            s,
            "The field's own catalogue, [{rel}]({SOURCE_BASE}{rel}). Each KPI names the core block whose events it reads; the core's [KPI catalogue](../../kpis-catalogue.md) holds none of them.\n"
        );
        let mut names: Vec<&str> = Vec::new();
        for k in &catalogue.kpis {
            if !names.contains(&k.block.as_str()) {
                names.push(&k.block);
            }
        }
        for name in names {
            match core.iter().find(|b| b.name == name) {
                Some(b) => {
                    let _ = writeln!(s, "### [{name}](../../blocks/{}/{}.md)\n", b.dir, b.page);
                }
                None => {
                    let _ = writeln!(s, "### {name}\n");
                }
            }
            let kpis: Vec<_> = catalogue.kpis.iter().filter(|k| k.block == name).collect();
            blocks::kpi_table(&mut s, &kpis);
        }
    }
    if let Some(rel) = &f.rows {
        let doc = model::read_yaml(root, rel)?;
        heading(&mut s, "Rows of core catalogs");
        let _ = writeln!(
            s,
            "What the field registers in catalogs of the core when it is installed ([{rel}]({SOURCE_BASE}{rel})); the core itself names none of these.\n"
        );
        s.push_str("| command | row |\n|---|---|\n");
        for r in doc["rows"].as_array().into_iter().flatten() {
            let command = r["command"].as_str().unwrap_or("");
            let name = r["input"]["name"].as_str().unwrap_or("");
            let summary = r["input"]["summary"].as_str().unwrap_or("");
            let _ = writeln!(
                s,
                "| `{}` | `{}`: {} |",
                command.strip_prefix("agentic.").unwrap_or(command),
                text(name),
                text(summary)
            );
        }
    }
    Ok(s)
}
