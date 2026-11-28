//! The generated documentation pages: deterministic, complete, and held by `--check`.

use agenticspec_docs::{Catalogue, Generated, OWNED_DIRS, OWNED_FILES, generate, layer_edges};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A catalogue with KPIs on several blocks of several layers and every kind, so the KPI tests
/// hold whatever the repository's own catalogue holds today.
const FIXTURE_CATALOGUE: &str = include_str!("fixtures/catalogue.yaml");

fn fixture() -> Catalogue {
    Catalogue::parse(FIXTURE_CATALOGUE).expect("the fixture catalogue parses")
}

fn generated_with_fixture() -> Generated {
    generate(&root(), Some(fixture())).expect("the pages generate")
}

fn domains() -> Vec<String> {
    let out = Command::new("ess")
        .args(["specify", "compile", "--format", "json", "--path"])
        .arg(root().join("blocks"))
        .output()
        .expect("ess runs");
    assert!(out.status.success(), "ess specify compile succeeds");
    let ir: serde_json::Value = serde_json::from_slice(&out.stdout).expect("IR JSON");
    ir["domains"]
        .as_object()
        .expect("domains")
        .keys()
        .cloned()
        .collect()
}

#[test]
fn two_generations_are_the_same_bytes() {
    let a = generated_with_fixture();
    let b = generated_with_fixture();
    assert_eq!(
        a.files.keys().collect::<Vec<_>>(),
        b.files.keys().collect::<Vec<_>>()
    );
    for (path, text) in &a.files {
        assert!(b.files[path] == *text, "{path} differs between two runs");
    }
}

#[test]
fn every_domain_has_a_page_linked_from_the_layer_map() {
    let g = generated_with_fixture();
    let index = &g.files["docs/blocks/index.md"];
    let ds = domains();
    assert_eq!(ds.len(), 23, "the blocks system has 23 domains");
    for domain in &ds {
        let block = domain.strip_prefix("agentic.").expect("agentic prefix");
        let page = g
            .files
            .keys()
            .find(|p| p.starts_with("docs/blocks/") && p.ends_with(&format!("/{block}.md")))
            .unwrap_or_else(|| panic!("{domain} has a page"));
        let rel = page.strip_prefix("docs/blocks/").unwrap();
        assert!(
            index.contains(&format!("({rel})")),
            "the layer map links {rel}"
        );
        let text = &g.files[page];
        assert!(
            text.contains(&format!("title: {block}\n")),
            "{page} is titled"
        );
    }
}

#[test]
fn every_kpi_of_the_catalogue_is_on_its_block_page_and_in_the_catalogue_page() {
    let mut catalogues = vec![fixture()];
    let own = root().join("kpis/catalogue.yaml");
    if own.exists() {
        let text = std::fs::read_to_string(&own).expect("kpis/catalogue.yaml");
        catalogues.push(Catalogue::parse(&text).expect("kpis/catalogue.yaml parses"));
    }
    for catalogue in catalogues {
        assert!(!catalogue.kpis.is_empty());
        let g = generate(&root(), Some(catalogue.clone())).expect("the pages generate");
        let table = &g.files["docs/kpis-catalogue.md"];
        let data = &g.files["website/src/generated/kpis.json"];
        for kpi in &catalogue.kpis {
            let page = g
                .files
                .iter()
                .find(|(p, _)| {
                    p.starts_with("docs/blocks/") && p.ends_with(&format!("/{}.md", kpi.block))
                })
                .map(|(_, t)| t.as_str())
                .unwrap_or_else(|| {
                    // A block the catalogue names before blocks/ specifies it: the planned page.
                    let planned = &g.files["docs/blocks/planned.md"];
                    let at = planned
                        .find(&format!("\n## {}\n", kpi.block))
                        .unwrap_or_else(|| panic!("planned block {} has a section", kpi.block));
                    let rest = &planned[at + 1..];
                    &rest[..rest[3..].find("\n## ").map_or(rest.len(), |e| e + 3)]
                });
            assert!(
                page.contains(&format!("`{}`", kpi.id)),
                "KPI {} is on the page of {}",
                kpi.id,
                kpi.block
            );
            assert!(
                data.contains(&format!("\"id\": \"{}\"", kpi.id)),
                "KPI {} is in the catalogue data",
                kpi.id
            );
            assert!(
                table.contains(&format!("`{}`", kpi.id)),
                "KPI {} is listed on the catalogue page",
                kpi.id
            );
        }
    }
}

#[test]
fn a_kpi_of_a_block_not_yet_specified_is_on_the_planned_page() {
    let text = FIXTURE_CATALOGUE.replacen("block: question", "block: verification", 1);
    let catalogue = Catalogue::parse(&text).expect("parses");
    let g = generate(&root(), Some(catalogue)).expect("the pages generate");
    let planned = &g.files["docs/blocks/planned.md"];
    let at = planned
        .find("\n## verification\n")
        .expect("a section for the planned block");
    assert!(planned[at..].contains("`questions_defaulted`"));
    assert!(
        !g.files.contains_key("docs/blocks/planned/verification.md"),
        "a planned block has no page of its own"
    );
    let index = &g.files["docs/blocks/index.md"];
    assert!(
        index.contains("(planned.md)"),
        "the layer map links the planned blocks"
    );
}

#[test]
fn every_architecture_has_a_page_and_a_rendered_html() {
    let g = generated_with_fixture();
    let dir = root().join("architectures");
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("architectures/") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "yaml") {
            names.push(path.file_stem().unwrap().to_string_lossy().into_owned());
        }
    }
    assert!(!names.is_empty());
    let index = &g.files["docs/architectures/index.md"];
    for name in names {
        let page = &g.files[&format!("docs/architectures/{name}.md")];
        let html = &g.files[&format!("website/static/architectures/{name}.html")];
        assert!(html.starts_with("<!doctype html>") || html.starts_with("<!DOCTYPE html>"));
        // Without `.html`: docusaurus serve redirects `<name>.html` off the base path (SUF-001).
        assert!(
            page.contains(&format!("src={{\"/architectures/{name}\"}}")),
            "{name}: the page embeds its HTML by the path without .html"
        );
        assert!(index.contains(&format!("({name}.md)")), "{name} is listed");
        assert!(
            !page.contains("hide_table_of_contents"),
            "{name}: the page has its contents list"
        );
    }
    a_nested_architecture_links_to_the_page_of_the_one_it_nests(&g);
}

/// Every domain the root ess-inputs.yaml adds to the core is a block of a pack, with a page under
/// docs/packs/<pack>/ that the packs index links and that links the core blocks it depends on; the
/// layer map of the core links the packs index; every architecture a field lists has its page and
/// graph under the field, accepted by the check.
#[test]
fn every_pack_block_and_field_architecture_has_a_linked_page() {
    let g = generated_with_fixture();
    let out = Command::new("ess")
        .args(["specify", "compile", "--format", "json", "--path"])
        .arg(root())
        .output()
        .expect("ess runs");
    assert!(out.status.success(), "the whole system compiles");
    let ir: serde_json::Value = serde_json::from_slice(&out.stdout).expect("IR JSON");
    let core = domains();
    let packs = &g.files["docs/packs/index.md"];
    let mut seen = 0;
    for domain in ir["domains"].as_object().expect("domains").keys() {
        if core.contains(domain) {
            continue;
        }
        seen += 1;
        let name = domain.strip_prefix("agentic.").expect("agentic prefix");
        let last = name.rsplit('.').next().unwrap();
        let page = g
            .files
            .keys()
            .find(|p| {
                p.starts_with("docs/packs/")
                    && p.ends_with(&format!("/{last}.md"))
                    && g.files[*p].contains(&format!("title: \"{name}\"\n"))
            })
            .unwrap_or_else(|| panic!("{domain} has a page under docs/packs/"));
        let rel = page.strip_prefix("docs/packs/").unwrap();
        assert!(
            packs.contains(&format!("({rel})")),
            "the packs index links {rel}"
        );
        assert!(
            g.files[page].contains("](../../blocks/"),
            "{page} links the core blocks it depends on"
        );
    }
    assert!(seen >= 9, "the packs add {seen} domains");
    assert!(g.files["docs/blocks/index.md"].contains("(../packs/index.md)"));
    let field = &g.files["docs/fields/coding/index.md"];
    assert!(
        field.contains("(auto-merger.md)"),
        "the field lists its architecture"
    );
    let page = &g.files["docs/fields/coding/auto-merger.md"];
    assert!(page.contains("src={\"/fields/coding/auto-merger\"}"));
    assert!(
        page.contains("as-tag--accepted"),
        "the check accepts the auto-merger"
    );
    assert!(page.contains("](../../packs/source-control/change.md)"));
    assert!(
        g.files
            .contains_key("website/static/fields/coding/auto-merger.html")
    );
}

/// The page of an architecture, its catalog entry and the catalog index show the KPIs of the
/// architecture as a whole with the numbers `agenticspec sim check` reports.
#[test]
fn the_architecture_as_a_whole_shows_what_check_reports() {
    let g = generated_with_fixture();
    let checked =
        agenticspec_sim::cli::check(&root().join("architectures/workstation.yaml")).unwrap();
    let page = &g.files["docs/architectures/workstation.md"];
    let entry = &g.files["docs/catalog/workstation.md"];
    let index = &g.files["docs/catalog/index.md"];
    for text in [page, entry] {
        assert!(text.contains("\n## The architecture as a whole\n"));
    }
    for m in agenticspec_sim::whole::ARCHITECTURE_KPIS {
        let v = checked["kpis"][m.id].as_f64().expect("a number");
        let shown = if v.fract() == 0.0 {
            format!("{}", v as i64)
        } else {
            format!("{v}")
        };
        let row = format!("| `{}` | {shown}", m.id);
        assert!(page.contains(&row), "the page shows {row}");
        assert!(entry.contains(&row), "the catalog entry shows {row}");
        assert!(
            index.contains(&format!("\"id\":\"{}\"", m.id)),
            "the index has {}",
            m.id
        );
    }
    assert!(page.contains("| `single_points` | 2 (`gateway`, `model`) |"));
    assert!(page.contains("[KPI catalogue](../kpis-catalogue.md)"));
    assert!(
        g.files["docs/fields/coding/auto-merger.md"]
            .contains("[KPI catalogue](../../kpis-catalogue.md)"),
        "a page of a field links the catalogue two levels up"
    );
    assert!(index.contains("<ArchitectureKpis data={"));
    assert!(
        index.contains("\"name\":\"solo\""),
        "an unsearched architecture is in the table"
    );
}

/// The catalog groups its entries by the field their experiment names, the field's page lists its
/// profile, its KPIs under the core block each reads and its rows of core catalogs, and the core
/// KPI catalogue holds none of the field's KPIs.
#[test]
fn the_catalog_groups_by_field_and_the_field_page_holds_its_data() {
    let g = generate(&root(), None).expect("the pages generate");
    let index = &g.files["docs/catalog/index.md"];
    let coding = index.find("\n## Coding\n").expect("a Coding group");
    for (_, name) in experiments() {
        let card = index
            .find(&format!("\"to\":\"/docs/catalog/{name}\""))
            .unwrap_or_else(|| panic!("{name} has a card"));
        assert!(card > coding, "{name} is under Coding");
    }
    let field = &g.files["docs/fields/coding/index.md"];
    assert!(field.contains("fields/coding/profiles/coding.yaml"));
    assert!(field.contains("### [workspace](../../blocks/8-place/workspace.md)"));
    assert!(field.contains("`merge_conflicts`"));
    assert!(field.contains("`accept-edits`"));
    let core = &g.files["docs/kpis-catalogue.md"];
    for id in [
        "merge_conflicts",
        "false_completions_shipped",
        "revert_rate",
    ] {
        assert!(!core.contains(&format!("`{id}`")), "{id} is not a core KPI");
    }
}

/// An experiment that names no field, or a field that does not exist, is refused.
#[test]
fn an_experiment_without_a_field_is_refused() {
    let tmp = copy_inputs("docs-no-field");
    let exp = tmp.join("experiments/shared-board-tuning.yaml");
    let text = std::fs::read_to_string(&exp)
        .unwrap()
        .replace("field: coding\n", "field: nowhere\n");
    std::fs::write(&exp, text).unwrap();
    let err = generate(&tmp, Some(fixture())).expect_err("an unknown field is refused");
    assert!(
        err.contains("experiments/shared-board-tuning.yaml"),
        "{err}"
    );
    assert!(err.contains("needs `field`"), "{err}");
}

/// SUF-002: on an architecture with an experiment, the search result (the marked places with their
/// best values and the KPIs that moved) comes before the graph.
#[test]
fn an_architecture_page_shows_the_search_result_before_the_graph() {
    let g = generated_with_fixture();
    let page = &g.files["docs/architectures/author-reviewer.md"];
    let at = |needle: &str| {
        page.find(needle)
            .unwrap_or_else(|| panic!("author-reviewer has {needle}"))
    };
    let frame = at("<ArchitectureFrame ");
    for section in [
        "## Search result\n",
        "## Marked places\n",
        "## KPIs that moved\n",
    ] {
        assert!(at(section) < frame, "{section} comes before the graph");
    }
    assert!(at("| place | as built | best value |") < frame);
    assert!(at("## Optimization\n") > frame);
}

#[test]
fn the_dependencies_are_those_of_the_layer_check() {
    let g = generated_with_fixture();
    let out = Command::new("sh")
        .arg(root().join("scripts/check-layers.sh"))
        .arg("--graph")
        .output()
        .expect("check-layers runs");
    assert!(out.status.success(), "the layer check passes");
    let text = String::from_utf8(out.stdout).expect("utf-8");
    let mut want = BTreeMap::new();
    for line in text.lines() {
        // "     19 1 agentic.delegation -> 1 agentic.principal"
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.len() == 6 && w[3] == "->" {
            want.insert(
                (w[2].to_string(), w[5].to_string()),
                w[0].parse::<usize>().unwrap(),
            );
        }
    }
    assert!(!want.is_empty());
    assert_eq!(layer_edges(&g.ir), want);
}

#[test]
fn no_predicate_is_printed_as_raw_json() {
    let g = generated_with_fixture();
    for (path, text) in g.files.iter().filter(|(p, _)| p.ends_with(".md")) {
        for line in text.lines() {
            let line = line.replace("\\`", "");
            for (i, span) in line.split('`').enumerate() {
                assert!(
                    i % 2 == 0 || !span.contains("{\""),
                    "{path}: a raw JSON predicate in `{span}`"
                );
            }
        }
    }
}

#[test]
fn no_generated_file_carries_a_machine_path() {
    let g = generated_with_fixture();
    let root = std::fs::canonicalize(root()).unwrap();
    let root = root.to_string_lossy();
    for (path, text) in &g.files {
        assert!(
            !text.contains(root.as_ref()),
            "{path} holds the checkout path"
        );
        assert!(!text.contains("/home/"), "{path} holds a home directory");
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn run(bin_root: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_agenticspec-docs"))
        .arg("--root")
        .arg(bin_root)
        .args(args)
        .output()
        .expect("agenticspec-docs runs");
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), text)
}

/// A copy of the generator's inputs under the target's test directory: the sources, the committed
/// results of the catalog with their digest files and what those digest, and the fixture catalogue.
fn copy_inputs(name: &str) -> PathBuf {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&tmp);
    for dir in [
        "blocks",
        "architectures",
        "experiments",
        "kpis",
        "crates/agenticspec-sim/src",
        "packs",
        "fields",
    ] {
        if root().join(dir).exists() {
            copy_dir(&root().join(dir), &tmp.join(dir));
        }
    }
    for file in ["Cargo.lock", "rust-toolchain.toml", "ess-inputs.yaml"] {
        std::fs::copy(root().join(file), tmp.join(file)).unwrap();
    }
    for (_, name) in experiments() {
        let dir = format!("website/static/catalog/{name}");
        std::fs::create_dir_all(tmp.join(&dir)).unwrap();
        for f in ["result.json", "sources.sha256"] {
            std::fs::copy(root().join(&dir).join(f), tmp.join(&dir).join(f)).unwrap();
        }
    }
    std::fs::write(tmp.join("kpis/catalogue.yaml"), FIXTURE_CATALOGUE).unwrap();
    tmp
}

/// The pages are made from the committed results; a result whose sources changed is refused with
/// the command that writes it again.
#[test]
fn a_result_whose_sources_changed_is_refused() {
    let tmp = copy_inputs("docs-stale");
    let exp = tmp.join("experiments/shared-board-tuning.yaml");
    let text = std::fs::read_to_string(&exp)
        .unwrap()
        .replace("seeds: 8", "seeds: 7");
    std::fs::write(&exp, text).unwrap();
    let err = generate(&tmp, Some(fixture())).expect_err("a stale result is refused");
    assert!(
        err.contains("experiments/shared-board-tuning.yaml"),
        "{err}"
    );
    assert!(err.contains("task catalog:generate"), "{err}");

    let tmp = copy_inputs("docs-stale-sim");
    let sim = tmp.join("crates/agenticspec-sim/src/search.rs");
    let mut text = std::fs::read_to_string(&sim).unwrap();
    text.push_str("\n// changed\n");
    std::fs::write(&sim, text).unwrap();
    let err = generate(&tmp, Some(fixture())).expect_err("a change of the simulator is refused");
    assert!(
        err.contains("crates/agenticspec-sim/src/search.rs"),
        "{err}"
    );
}

/// The contract module of the simulator (`crates/agenticspec-sim/src/contract/`) changes no
/// simulation result, so a change there leaves every committed result current.
#[test]
fn a_change_of_the_contract_module_keeps_every_result_current() {
    let tmp = copy_inputs("docs-contract-change");
    let file = tmp.join("crates/agenticspec-sim/src/contract/ports.rs");
    let mut text = std::fs::read_to_string(&file).unwrap();
    text.push_str("\n// changed\n");
    std::fs::write(&file, text).unwrap();
    generate(&tmp, Some(fixture()))
        .expect("a change of the contract module is no change of a result");
}

#[test]
fn check_fails_after_a_generated_file_is_edited_or_added() {
    let tmp = copy_inputs("docs-check");

    let (code, text) = run(&tmp, &[]);
    assert_eq!(code, 0, "{text}");
    for p in OWNED_FILES {
        assert!(tmp.join(p).exists(), "{p} is written");
    }
    let (code, text) = run(&tmp, &["--check"]);
    assert_eq!(code, 0, "fresh pages pass: {text}");

    let page = tmp.join("docs/blocks/7-collaboration/question.md");
    let mut body = std::fs::read_to_string(&page).unwrap();
    body.push_str("\nedited by hand\n");
    std::fs::write(&page, body).unwrap();
    let html = tmp.join("website/static/architectures/workstation.html");
    std::fs::remove_file(&html).unwrap();
    let contract = tmp.join("website/static/architectures/workstation.contract.yaml");
    let mut yaml = std::fs::read_to_string(&contract).unwrap();
    yaml.push_str("# edited by hand\n");
    std::fs::write(&contract, yaml).unwrap();
    let stray = tmp.join(OWNED_DIRS[0]).join("stray.md");
    std::fs::write(&stray, "not generated\n").unwrap();
    let (code, text) = run(&tmp, &["--check"]);
    assert_eq!(
        code, 1,
        "an edited, a missing and a stray file fail: {text}"
    );
    for named in [
        "changed: docs/blocks/7-collaboration/question.md",
        "missing: website/static/architectures/workstation.html",
        "changed: website/static/architectures/workstation.contract.yaml",
        "not generated: docs/blocks/stray.md",
    ] {
        assert!(text.contains(named), "{named} in {text}");
    }
    assert!(stray.exists() && !html.exists(), "--check writes nothing");

    let (code, _) = run(&tmp, &[]);
    assert_eq!(code, 0);
    assert!(!stray.exists(), "a write removes the stray file");
    assert!(html.exists(), "a write restores the missing file");
}

/// Each experiment as `(its name, the file stem of the architecture it names)`.
fn experiments() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root().join("experiments")).expect("experiments/") {
        let path = entry.expect("entry").path();
        if path.extension().is_none_or(|e| e != "yaml") {
            continue;
        }
        let doc = agenticspec_sim::doc::load_yaml(&path).expect("experiment parses");
        let arch = doc["architecture"].as_str().expect("architecture");
        let arch = Path::new(arch)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        out.push((
            doc["experiment"].as_str().expect("experiment").to_string(),
            arch,
        ));
    }
    out.sort();
    out
}

#[test]
fn every_experiment_has_a_catalog_entry_with_its_downloads() {
    let g = generated_with_fixture();
    let index = &g.files["docs/catalog/index.md"];
    let list = experiments();
    assert!(!list.is_empty());
    for (exp, name) in &list {
        let page = &g.files[&format!("docs/catalog/{name}.md")];
        let dir = format!("website/static/catalog/{name}");
        for file in [
            format!("{name}.yaml"),
            format!("{name}.best.yaml"),
            format!("{exp}.yaml"),
            "coding.yaml".to_string(),
            "result.json".to_string(),
        ] {
            let path = format!("{dir}/{file}");
            assert!(
                g.files.contains_key(&path)
                    || (file == "result.json" && root().join(&path).exists()),
                "{path} is written or committed"
            );
            assert!(
                page.contains(&format!("\"file\":\"{file}\"")),
                "{name} offers {file}"
            );
        }
        let html = &g.files[&format!("{dir}/{name}.html")];
        assert!(
            html.contains("architecture and optimization"),
            "{name}: the full result page"
        );
        assert!(
            page.contains(&format!("src={{\"/catalog/{name}/{name}\"}}")),
            "{name}: the page embeds its result page by the path without .html"
        );
        assert!(
            page.contains(&format!("agenticspec sim run {exp}.yaml --out out/")),
            "{name}: the command"
        );
        assert!(page.contains("## Deploy\n"), "{name}: the deploy note");
        assert!(
            index.contains(&format!("\"to\":\"/docs/catalog/{name}\"")),
            "{name} has a card"
        );
        assert_eq!(
            g.files[&format!("{dir}/{name}.yaml")],
            std::fs::read_to_string(root().join(format!("architectures/{name}.yaml"))).unwrap(),
            "{name}: the architecture as written"
        );
    }
    for entry in std::fs::read_dir(root().join("architectures")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        if path.extension().is_some_and(|e| e == "yaml") && !list.iter().any(|(_, a)| *a == name) {
            assert!(
                index.contains(&format!("[{name}](../architectures/{name}.md)")),
                "{name} is listed as not optimized yet"
            );
        }
    }
}

/// The downloaded files alone, read as a user reads them: the experiment names the architecture and
/// the profile beside it, the optimized architecture passes the check, and with it in place of the
/// one as written the experiment scores the best fitness of `result.json`.
#[test]
fn a_downloaded_optimized_architecture_reproduces_the_best_fitness() {
    let g = generated_with_fixture();
    let dir = "website/static/catalog/shared-board";
    let read =
        |f: &str| agenticspec_sim::doc::parse_yaml(&g.files[&format!("{dir}/{f}")]).expect(f);
    let exp = read("shared-board-tuning.yaml");
    assert_eq!(exp["architecture"], "shared-board.yaml");
    assert_eq!(exp["profile"], "coding.yaml");
    let best = read("shared-board.best.yaml");
    assert!(agenticspec_sim::check::check(&best).is_empty());
    let e =
        agenticspec_sim::Experiment::from_docs(exp, read("shared-board.yaml"), read("coding.yaml"))
            .unwrap();
    let text = std::fs::read_to_string(root().join(dir).join("result.json")).unwrap();
    let result: serde_json::Value = serde_json::from_str(&text).unwrap();
    let want = result["best"]["fitness"].as_f64().unwrap();
    let got = agenticspec_sim::best::fitness_as_written(&e, best, 2).unwrap();
    // serde_json reads a float back to within a unit in the last place.
    assert!(
        got.to_bits().abs_diff(want.to_bits()) <= 1,
        "{got} != {want}"
    );
}

/// The committed result of the cheapest experiment is the bytes of a fresh run; `task slow:catalog`
/// checks every one.
#[test]
fn a_catalog_result_is_the_bytes_of_a_fresh_run() {
    let e = agenticspec_sim::Experiment::load(&root().join("experiments/shared-board-tuning.yaml"))
        .unwrap();
    let mut fresh = serde_json::to_string_pretty(&agenticspec_sim::run(&e, 2).unwrap()).unwrap();
    fresh.push('\n');
    let committed =
        std::fs::read_to_string(root().join("website/static/catalog/shared-board/result.json"))
            .unwrap();
    assert!(committed == fresh, "run `task catalog:generate`");
}

/// The contract of a document as `agenticspec sim contract` derives it, from the committed ports.
fn contract_of(path: &Path) -> agenticspec_sim::contract::derive::Contract {
    let ports =
        agenticspec_sim::contract::ports::PortsFile::load(&root().join("blocks/ports.json"))
            .expect("blocks/ports.json");
    let doc = agenticspec_sim::doc::load_yaml(path).expect("the document");
    let mut contract = agenticspec_sim::contract::derive::derive(&doc, &ports).expect("a contract");
    let kpis = agenticspec_sim::contract::ingress::Kpis::load(&root().join("kpis/catalogue.yaml"))
        .expect("kpis/catalogue.yaml");
    let (resolved, refusals) =
        agenticspec_sim::contract::ingress::resolve(&doc, &contract, &ports, &kpis);
    assert!(refusals.is_empty(), "{refusals:?}");
    contract.ingress = resolved;
    contract
}

/// The part of a page from its "Contract" section to the next section.
fn contract_section<'a>(page: &'a str, what: &str) -> &'a str {
    let at = page
        .find("\n## Contract\n")
        .unwrap_or_else(|| panic!("{what} has a Contract section"));
    let rest = &page[at + 1..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |e| e + 4);
    &rest[..end]
}

/// Every architecture page and every catalog entry shows the contract of its document: one row
/// per port with its protocol, the border and the contract file, which is the derived contract.
#[test]
fn every_architecture_page_and_catalog_entry_shows_its_contract() {
    let g = generated_with_fixture();
    let mut pages = 0;
    for entry in std::fs::read_dir(root().join("architectures")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "yaml") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let contract = contract_of(&path);
        let file = format!("website/static/architectures/{name}.contract.yaml");
        assert_eq!(g.files[&file], contract.text(), "{file}");
        for (page_path, link) in [
            (
                format!("docs/architectures/{name}.md"),
                format!("pathname:///architectures/{name}.contract.yaml"),
            ),
            (
                format!("docs/catalog/{name}.md"),
                format!("{name}.contract.yaml"),
            ),
        ] {
            let Some(page) = g.files.get(&page_path) else {
                continue;
            };
            pages += 1;
            let section = contract_section(page, &page_path);
            let rows = section.lines().filter(|l| l.starts_with("| `")).count();
            assert_eq!(rows, contract.ports.len(), "{page_path}: one row per port");
            for p in &contract.ports {
                let protocol = format!("`{}`", p.protocol.replace('|', "\\|"));
                let row = section
                    .lines()
                    .find(|l| {
                        let cells: Vec<&str> = l.split(" | ").collect();
                        l.starts_with(&format!("| `{}` |", p.port))
                            && cells.get(4) == Some(&format!("`{}`", p.part).as_str())
                    })
                    .unwrap_or_else(|| panic!("{page_path}: a row for {}@{}", p.port, p.part));
                assert!(row.contains(&protocol), "{page_path}: {row}");
                assert!(
                    row.ends_with(&format!("| {} |", p.instances)),
                    "{page_path}: {row}"
                );
            }
            let border: Vec<String> = contract.border.iter().map(|b| format!("**{b}**")).collect();
            let border = if border.is_empty() {
                "none".to_owned()
            } else {
                border.join(", ")
            };
            assert!(
                section.contains(&format!("Border: {border}.")),
                "{page_path}: the border"
            );
            assert!(section.contains(&link), "{page_path}: the contract file");
        }
        if g.files.contains_key(&format!("docs/catalog/{name}.md")) {
            let entry_file = format!("website/static/catalog/{name}/{name}.contract.yaml");
            assert_eq!(g.files[&entry_file], contract.text(), "{entry_file}");
            assert!(
                g.files[&format!("docs/catalog/{name}.md")]
                    .contains(&format!("\"file\":\"{name}.contract.yaml\"")),
                "{name}: the downloads list the contract"
            );
        }
    }
    assert_eq!(pages, 17, "11 architecture pages and 6 catalog entries");
    let contracts = g
        .files
        .keys()
        .filter(|k| k.ends_with(".contract.yaml"))
        .count();
    assert_eq!(contracts, 17);
    let wi = contract_section(
        &g.files["docs/architectures/workstation-ingress.md"],
        "workstation-ingress",
    );
    assert!(
        wi.contains("| **operator** | `admin` | yes | `http`, `event_stream`, `ui`, `tool` | 3 |"),
        "{wi}"
    );
    let ws = contract_section(&g.files["docs/architectures/workstation.md"], "workstation");
    assert!(
        ws.contains("| `workspace.lease` | `Outbound` | `workspace` |"),
        "{ws}"
    );
    assert!(
        ws.contains("| `tree` | `sharing: exclusive` | 12 |"),
        "{ws}"
    );
    for absent in ["runtime.instance", "trigger.firing", "channel.post"] {
        assert!(!ws.contains(absent), "{absent}");
    }
}

/// A nested architecture: its page links to the page of the architecture it nests, and the index
/// explains the format. Called from the test of the architecture pages, so the pages are generated
/// once for both.
fn a_nested_architecture_links_to_the_page_of_the_one_it_nests(g: &Generated) {
    let page = &g.files["docs/architectures/team-with-console.md"];
    assert!(page.contains("## Nested architectures\n"), "{page}");
    assert!(page.contains("[console](console.md)"), "{page}");
    assert!(
        page.contains("accepted"),
        "the nested pair passes the check"
    );
    let index = &g.files["docs/architectures/index.md"];
    assert!(index.contains("## Nesting an architecture\n"), "{index}");
    assert!(index.contains("agenticspec sim digest"), "{index}");
    let html = &g.files["website/static/architectures/team-with-console.html"];
    assert!(
        html.contains("data-port=\"console."),
        "the graph draws the ports"
    );
}

/// The `view:` key is presentation only: a catalog result's digest reads the architecture without it,
/// so a label edit leaves the committed result current.
#[test]
fn a_view_key_edit_leaves_the_digest_of_a_catalog_result_as_it_was() {
    let text = std::fs::read_to_string(root().join("architectures/workstation.yaml")).unwrap();
    let at = text
        .find("\nview:")
        .expect("workstation.yaml has a view key");
    let bare = format!("{}\n", text[..at].trim_end_matches('\n'));
    assert_eq!(agenticspec_docs::without_view(&text), bare);
    let edited = text.replace("worker: coding agents", "worker: agents that code");
    assert_ne!(edited, text);
    assert_eq!(agenticspec_docs::without_view(&edited), bare);
    let commented = format!("{bare}\n# How the UI looks.\nview:\n  home: overview\n");
    assert_eq!(agenticspec_docs::without_view(&commented), bare);
    let middle = "a: 1\nview:\n  home: overview\n\nb: 2\n";
    assert_eq!(agenticspec_docs::without_view(middle), "a: 1\nb: 2\n");
}
