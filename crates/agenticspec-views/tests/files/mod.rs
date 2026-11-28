//! Writing and checking the generated files under a repository root: the check is clean right
//! after a write, names a hand edit, a stray fixture and a changed input, and the written
//! document passes `ess ui check` against its model with no finding.

use std::process::Command;

use agenticspec_views::{DIGEST, FIXTURES, OUT, run};

use crate::root;

#[test]
fn a_write_then_a_check_is_clean() {
    let root = root("write-check");
    let wrote = run(&root, false).expect("the panels are written");
    assert_eq!(wrote["written"], true);
    assert_eq!(wrote["panels"], 2);
    let checked = run(&root, true).expect("a fresh write is current");
    assert_eq!(checked["written"], false);
    assert_eq!(checked["files"], wrote["files"]);
}

#[test]
fn the_written_document_passes_ess_ui_check_with_no_finding() {
    let root = root("ess-ui-check");
    run(&root, false).expect("the panels are written");
    let out = Command::new("ess")
        .args(["ui", "check", "--path"])
        .arg(root.join(OUT))
        .arg("--model")
        .arg(root.join("blocks"))
        .output()
        .expect("ess runs");
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("0 error(s), 0 warning(s)"), "{text}");
    assert!(text.contains("2 covered"), "{text}");
}

#[test]
fn the_check_names_a_hand_edit_and_a_stray_fixture() {
    let root = root("hand-edit");
    run(&root, false).expect("the panels are written");
    let doc = root.join(OUT);
    let text = std::fs::read_to_string(&doc).expect("the document");
    std::fs::write(&doc, text.replace("style: table", "style: cards")).expect("edit");
    std::fs::write(root.join(FIXTURES).join("extra.json"), "{}").expect("a stray file");
    let r = run(&root, true).expect_err("a hand edit is refused");
    assert_eq!(r.code, "panels_stale");
    assert!(
        r.detail.contains(&format!("{OUT}: differs")),
        "{}",
        r.detail
    );
    assert!(
        r.detail.contains(&format!(
            "{FIXTURES}/extra.json: in the tree, and not generated"
        )),
        "{}",
        r.detail
    );
    assert!(r.detail.contains("task ui:panels"), "{}", r.detail);

    run(&root, false).expect("a write repairs it");
    run(&root, true).expect("and the check is clean again");
    assert!(!root.join(FIXTURES).join("extra.json").exists());
}

#[test]
fn a_changed_input_changes_the_digest_even_when_the_document_does_not() {
    let root = root("digest");
    run(&root, false).expect("the panels are written");
    let before = std::fs::read_to_string(root.join(DIGEST)).expect("the digest file");
    assert!(
        before
            .lines()
            .any(|l| l.ends_with("  blocks/1-things/thing.yaml")),
        "{before}"
    );
    assert!(
        before.lines().any(|l| l.contains("  ess --version: ess ")),
        "{before}"
    );
    assert!(
        before
            .lines()
            .any(|l| l.ends_with("  crates/agenticspec-views/src/panels.rs")),
        "{before}"
    );

    let file = root.join("blocks/1-things/thing.yaml");
    let text = std::fs::read_to_string(&file).expect("a block file");
    std::fs::write(&file, text + "\n# a comment changes no panel\n").expect("edit");
    let r = run(&root, true).expect_err("a changed input is refused");
    assert_eq!(
        r.detail,
        format!(
            "{DIGEST}: differs from a fresh generation. Generated files are never edited: run task ui:panels"
        )
    );
}

#[test]
fn a_system_that_does_not_compile_is_refused() {
    let root = root("broken");
    std::fs::write(root.join("blocks/1-things/thing.yaml"), "domain: [").expect("break a file");
    let r = run(&root, true).expect_err("a broken system is refused");
    assert_eq!(r.code, "blocks_unreadable");
}
