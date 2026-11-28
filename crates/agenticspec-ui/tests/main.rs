//! Tests of the UI generator. One binary (workspace rule).
//!
//! - `golden`: each document under `tests/cases/<case>/input.ui.yaml` generates exactly the files
//!   of `tests/cases/<case>/expected/`. `UPDATE_GOLDEN=1` rewrites them; review the diff.
//! - `refusals`: a construct the generator does not draw is refused by its node path.
//! - `output`: the properties every generation holds (deterministic, only kit components and host
//!   slots, no look of its own), on the cases and on the studio's document.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use agenticspec_ui::{Refusal, generate};

fn cases() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases")
}

fn input(case: &str) -> String {
    std::fs::read_to_string(cases().join(case).join("input.ui.yaml")).unwrap()
}

fn files_under(dir: &Path) -> BTreeMap<String, String> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                let rel = path
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, std::fs::read_to_string(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

fn generated(case: &str) -> BTreeMap<String, String> {
    match generate(&input(case)) {
        Ok(files) => files,
        Err(refusals) => panic!("{case} refused: {refusals:?}"),
    }
}

/// The refusals of `text`, which must be refused.
fn refused(text: &str) -> Vec<Refusal> {
    match generate(text) {
        Ok(_) => panic!("generated, and should have been refused:\n{text}"),
        Err(r) => r,
    }
}

/// A minimal document with one page whose one section is `section` (YAML, indented by six).
fn with_section(section: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: t\nplacement_profile: fat\nshells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\nnavigation: {{home: p, sections: [{{name: s, pages: [p]}}]}}\npage_kinds: {{plain: {{purpose: plain}}}}\npages:\n  p:\n    kind: plain\n    sections:\n{section}"
    )
}

fn assert_refused_at(text: &str, path: &str, words: &str) {
    let r = refused(text);
    assert!(
        r.iter()
            .any(|f| f.path == path && f.message.contains(words)),
        "no refusal at {path} containing {words:?}: {r:?}"
    );
}

mod golden {
    use super::*;

    fn check(case: &str) {
        let got = generated(case);
        let dir = cases().join(case).join("expected");
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            let _ = std::fs::remove_dir_all(&dir);
            agenticspec_ui::write(&got, &dir).unwrap();
        }
        let want = files_under(&dir);
        let got_names: Vec<&String> = got.keys().collect();
        let want_names: Vec<&String> = want.keys().collect();
        assert_eq!(got_names, want_names, "the files of {case}");
        for (name, text) in &got {
            assert_eq!(
                text, &want[name],
                "{case}/{name} differs from tests/cases/{case}/expected (UPDATE_GOLDEN=1 rewrites)"
            );
        }
    }

    #[test]
    fn a_list_and_a_record_over_a_path_parameter() {
        check("list");
    }

    #[test]
    fn a_form_with_host_slots_a_filter_bar_columns_and_a_channel() {
        check("form");
    }
}

mod refusals {
    use super::*;

    #[test]
    fn an_unknown_property_is_refused_by_the_path_of_its_node() {
        let text = with_section(
            "      - {name: list, component: collection, reads: {view: t.V}, colour: red}\n",
        );
        assert_refused_at(&text, "pages/p/sections/list/colour", "unknown property");
    }

    #[test]
    fn a_component_the_generator_does_not_draw_is_refused_by_its_path() {
        let text = with_section(
            "      - {name: trend, component: chart, chart: line, reads: {view: t.V}, x: at, series: [n]}\n",
        );
        assert_refused_at(&text, "pages/p/sections/trend/component", "chart");
    }

    #[test]
    fn a_property_of_another_component_is_refused_by_its_path() {
        let text = with_section("      - {name: f, component: form, does: t.Do, columns: [a]}\n");
        assert_refused_at(&text, "pages/p/sections/f/columns", "form");
    }

    #[test]
    fn a_nested_unknown_property_is_named_through_list_entries_by_their_names() {
        let text = with_section(
            "      - name: list\n        component: collection\n        reads: {view: t.V}\n        columns:\n          - {field: a}\n          - {field: b, width: 3}\n",
        );
        assert_refused_at(
            &text,
            "pages/p/sections/list/columns/b/width",
            "unknown property",
        );
    }

    #[test]
    fn a_theme_that_overrides_tokens_is_refused_since_the_look_is_the_kits() {
        let text =
            with_section("      - {name: list, component: collection, reads: {view: t.V}}\n")
                + "themes:\n  dark: {color: {bg: \"#000000\"}}\n";
        assert_refused_at(&text, "themes/dark/color", "unknown property");
        let text =
            with_section("      - {name: list, component: collection, reads: {view: t.V}}\n")
                + "tokens: {color: {accent: \"#ff0000\"}}\n";
        assert_refused_at(&text, "tokens", "unknown property");
    }

    #[test]
    fn an_expression_the_generator_does_not_compile_is_refused_by_its_path() {
        let text = with_section(
            "      - {name: r, component: record, reads: {view: t.V, params: {id: section.x.selection}}}\n",
        );
        assert_refused_at(&text, "pages/p/sections/r/reads/params/id", "expression");
    }

    #[test]
    fn an_action_kind_the_generator_does_not_carry_out_is_refused() {
        let text = with_section(
            "      - name: list\n        component: collection\n        reads: {view: t.V}\n        row_actions: [{name: x, export: {reads: t.V, as: csv}}]\n",
        );
        assert_refused_at(
            &text,
            "pages/p/sections/list/row_actions/x/export",
            "unknown property",
        );
    }

    #[test]
    fn a_document_of_another_format_is_refused() {
        let text =
            with_section("      - {name: list, component: collection, reads: {view: t.V}}\n")
                .replace("ess-ui/1", "ess-ui/2");
        assert_refused_at(&text, "format", "ess-ui/1");
    }

    #[test]
    fn a_page_of_an_unknown_kind_and_a_link_to_an_unknown_page_are_refused() {
        let text = with_section(
            "      - name: list\n        component: collection\n        reads: {view: t.V}\n        row_actions: [{name: go, navigate: {to: nowhere}, label: Go}]\n",
        );
        assert_refused_at(
            &text,
            "pages/p/sections/list/row_actions/go/navigate/to",
            "nowhere",
        );
        let text =
            with_section("      - {name: list, component: collection, reads: {view: t.V}}\n")
                .replace("kind: plain\n", "kind: other\n");
        assert_refused_at(&text, "pages/p/kind", "other");
    }
}

mod output {
    use super::*;

    fn studio() -> String {
        std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../studio/studio.ui.yaml"),
        )
        .unwrap()
    }

    fn all() -> Vec<(String, BTreeMap<String, String>)> {
        let mut out: Vec<(String, BTreeMap<String, String>)> = ["list", "form"]
            .iter()
            .map(|c| (c.to_string(), generated(c)))
            .collect();
        out.push((
            "studio".into(),
            generate(&studio()).expect("the studio document"),
        ));
        out
    }

    #[test]
    fn the_same_document_generates_the_same_bytes() {
        for case in ["list", "form"] {
            assert_eq!(generated(case), generated(case), "{case}");
        }
        assert_eq!(generate(&studio()).unwrap(), generate(&studio()).unwrap());
    }

    #[test]
    fn the_studio_document_generates_a_page_per_page_a_router_and_a_client() {
        let files = generate(&studio()).unwrap();
        for name in [
            "App.vue",
            "app.ts",
            "client.ts",
            "hosts.ts",
            "router.ts",
            "runtime.ts",
            "pages/DraftPage.vue",
            "pages/StudioPage.vue",
        ] {
            assert!(files.contains_key(name), "{name} in {:?}", files.keys());
        }
        let client = &files["client.ts"];
        for name in [
            "'studio.Architectures'",
            "'studio.draft.Drafts'",
            "'studio.DraftSummary'",
            "'studio.DraftParts'",
            "'studio.DraftRefusals'",
            "'studio.DraftGraph'",
            "'studio.draft.OpenDraft'",
            "'studio.session.CallTool'",
        ] {
            assert!(client.contains(name), "{name} in client.ts");
        }
        let hosts = &files["hosts.ts"];
        assert!(
            hosts.contains("DraftGraph") && hosts.contains("PatchEditor"),
            "{hosts}"
        );
    }

    /// Every element of a generated template is a component of ui/kit (`Kit…`), a host slot
    /// (`component :is`) or a Vue built-in; no HTML element, so the look is all ui/kit's.
    #[test]
    fn templates_use_only_kit_components_host_slots_and_vue_built_ins() {
        for (case, files) in all() {
            for (name, text) in files.iter().filter(|(n, _)| n.ends_with(".vue")) {
                let template = text
                    .split_once("<template>")
                    .and_then(|(_, rest)| rest.rsplit_once("</template>"))
                    .map(|(t, _)| t)
                    .unwrap_or_else(|| panic!("{case}/{name} has no template"));
                for tag in template.split('<').skip(1) {
                    let tag = tag.trim_start_matches('/');
                    let word: String = tag
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                        .collect();
                    if word.is_empty() {
                        continue;
                    }
                    assert!(
                        word.starts_with("Kit")
                            || ["component", "template", "Teleport"].contains(&word.as_str()),
                        "{case}/{name}: <{word}> is neither a kit component nor a host slot"
                    );
                }
            }
        }
    }

    /// No colour, no style block and no class of its own: a change of a token in ui/kit changes no
    /// generated byte.
    #[test]
    fn generated_files_carry_no_look_of_their_own() {
        for (case, files) in all() {
            for (name, text) in &files {
                for needle in [
                    "<style", "style=", "class=", "#fff", "rgb(", "px;", "var(--",
                ] {
                    assert!(!text.contains(needle), "{case}/{name} holds {needle:?}");
                }
                let hex = text
                    .match_indices('#')
                    .filter(|(i, _)| {
                        let tail: String = text[i + 1..]
                            .chars()
                            .take_while(char::is_ascii_hexdigit)
                            .collect();
                        tail.len() == 6 || tail.len() == 3
                    })
                    .count();
                assert_eq!(hex, 0, "{case}/{name} holds a colour literal");
            }
        }
    }

    #[test]
    fn every_generated_file_says_it_is_generated_and_from_what() {
        for (case, files) in all() {
            for (name, text) in &files {
                assert!(
                    text.contains("Generated by agenticspec ui generate"),
                    "{case}/{name}"
                );
            }
        }
    }

    #[test]
    fn write_replaces_the_directory_with_exactly_the_files() {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("agenticspec-ui-write");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("pages")).unwrap();
        std::fs::write(dir.join("pages/Stale.vue"), "stale").unwrap();
        let files = generated("list");
        agenticspec_ui::write(&files, &dir).unwrap();
        assert_eq!(files_under(&dir), files);
    }
}
