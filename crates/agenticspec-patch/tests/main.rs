//! Tests of agenticspec-patch. One binary (workspace rule); the architectures are the documents
//! under `architectures/`, read as they are.

use std::path::{Path, PathBuf};

use agenticspec_patch::{Applied, Patch, Refused};
use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn architecture(name: &str) -> Value {
    let path = root().join("architectures").join(format!("{name}.yaml"));
    agenticspec_sim::doc::load_yaml(&path).unwrap()
}

fn patch(text: &str) -> Patch {
    Patch::from_text(text).unwrap()
}

fn applied(doc: &Value, text: &str) -> Applied {
    match agenticspec_patch::apply(doc, &patch(text)) {
        Ok(a) => a,
        Err(r) => panic!("refused: {r}"),
    }
}

fn refused(doc: &Value, text: &str) -> Refused {
    match agenticspec_patch::apply(doc, &patch(text)) {
        Ok(a) => panic!("applied: {:?}", a.diff),
        Err(r) => r,
    }
}

mod facts {
    use super::*;
    use agenticspec_patch::facts;
    use agenticspec_records::sq::architecture::{Link, Marker, Part, Setting};
    use agenticspec_records::sq::facts::{Val, vs};

    #[test]
    fn a_part_is_its_key_block_and_count_one_when_absent() {
        let f = facts(&architecture("workstation"));
        let part = |id: &str| f.parts.iter().find(|p| p.id == id).unwrap().clone();
        assert_eq!(
            part("worker"),
            Part {
                id: "worker".into(),
                block: vs("agent"),
                count: 12
            }
        );
        assert_eq!(part("operator").count, 1);
        assert_eq!(part("owner").block, vs("principal"));
        assert_eq!(f.parts.len(), 9);
    }

    #[test]
    fn every_other_key_is_a_setting_a_list_one_per_element_a_duration_in_ms() {
        let f = facts(&architecture("workstation"));
        let has = |part: &str, name: &str, value: Val| {
            f.settings.contains(&Setting {
                part: part.into(),
                name: name.into(),
                value,
            })
        };
        assert!(has("operator", "acts_for", vs("owner")));
        assert!(has("operator", "harness", vs("coding")));
        assert!(has("owner", "kind", vs("human")));
        assert!(has("box", "egress", vs("gateway")));
        assert!(has("builds", "capacity", Val::Int(16)));
        assert!(has("builds", "job_limit", Val::Int(600_000)));
        assert!(has("tree", "per", vs("worker")));
        assert!(
            !f.settings
                .iter()
                .any(|s| s.name == "block" || s.name == "count"),
            "block and count are fields of part"
        );
    }

    #[test]
    fn a_link_is_numbered_from_zero_and_over_is_null_when_absent() {
        let f = facts(&architecture("workstation"));
        assert_eq!(
            f.links[0],
            Link {
                index: 0,
                from: vs("owner"),
                to: vs("operator"),
                carries: vs("prompt"),
                over: vs("chat"),
            }
        );
        assert_eq!(f.links[2].over, Val::Null);
        assert_eq!(f.links.len(), 8);
    }

    #[test]
    fn a_marker_is_every_marked_place_with_its_value() {
        let f = facts(&architecture("workstation"));
        let paths: Vec<&str> = f.markers.iter().map(|m| m.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "parts.builds.capacity",
                "parts.gateway.rate_limit_per_minute",
                "parts.worker.count",
                "policies.operator_answers_alone",
                "policies.question_default_after",
            ]
        );
        assert!(f.markers.contains(&Marker {
            path: "policies.question_default_after".into(),
            value: Val::Int(900_000),
        }));
        assert!(f.markers.contains(&Marker {
            path: "policies.operator_answers_alone".into(),
            value: Val::Bool(false),
        }));
    }

    #[test]
    fn every_architecture_gives_facts() {
        let dir = root().join("architectures");
        let mut n = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|x| x == "yaml") {
                let doc = agenticspec_sim::doc::load_yaml(&path).unwrap();
                let f = facts(&doc);
                assert!(!f.parts.is_empty(), "{}", path.display());
                n += 1;
            }
        }
        assert!(n >= 10);
    }
}

mod apply {
    use super::*;

    #[test]
    fn a_set_on_a_marked_count_sets_its_value_and_keeps_its_vary() {
        let doc = architecture("workstation");
        let a = applied(
            &doc,
            r#"set(P, count, 3) :- part{id: P, block: "agent", count: 12}."#,
        );
        let worker = &a.document["parts"]["worker"];
        assert_eq!(worker["count"]["value"], 3);
        assert_eq!(
            worker["count"]["vary"],
            doc["parts"]["worker"]["count"]["vary"]
        );
        assert_eq!(a.document["parts"]["operator"], doc["parts"]["operator"]);
        assert!(a.check.is_empty());
        assert_eq!(a.changed_rows, 1);
        assert_eq!(a.diff.len(), 1);
        assert_eq!(a.diff[0].path, "parts.worker.count.value");
        assert_eq!(a.diff[0].before, Some(Value::from(12)));
        assert_eq!(a.diff[0].after, Some(Value::from(3)));
    }

    #[test]
    fn add_puts_in_a_part_and_its_links_and_the_check_passes() {
        let doc = architecture("workstation");
        let a = applied(
            &doc,
            r#"
            add(part{id: "reviewer", block: "agent", harness: "coding", role: "reviewer"}).
            add(link{from: "reviewer", to: "gateway", carries: "model_call"}).
            add(link{from: W, to: "reviewer", carries: "review"}) :- part{id: W, block: "agent", count: 12}.
            "#,
        );
        let reviewer = &a.document["parts"]["reviewer"];
        assert_eq!(reviewer["block"], "agent");
        assert_eq!(reviewer["role"], "reviewer");
        let links = a.document["links"].as_array().unwrap();
        assert_eq!(links.len(), 10);
        assert_eq!(links[9]["from"], "worker");
        assert_eq!(links[9]["to"], "reviewer");
        assert_eq!(links[9]["carries"], "review");
        assert!(links[8].get("over").is_none());
        assert!(a.check.is_empty());
        assert_eq!(a.changed_rows, 3);
    }

    #[test]
    fn a_rule_selects_on_what_the_rules_before_it_did() {
        let doc = architecture("workstation");
        let a = applied(
            &doc,
            r#"
            add(part{id: "spare", block: "model", route: "frontier"}).
            add(link{from: "gateway", to: "spare", carries: "model_call"}).
            set(P, route, "local") :- part{id: P, block: "model"}, setting{part: P, name: "route", value: "frontier"}.
            "#,
        );
        assert_eq!(a.document["parts"]["spare"]["route"], "local");
        assert_eq!(a.document["parts"]["model"]["route"], "local");
    }

    #[test]
    fn a_set_on_a_link_by_its_index() {
        let doc = architecture("workstation");
        let a = applied(
            &doc,
            r#"set(L, over, "chat") :- link{index: L, from: "worker", to: "operator", carries: "question"}."#,
        );
        assert_eq!(a.document["links"][2]["over"], "chat");
        assert_eq!(a.diff.len(), 1);
        assert_eq!(a.diff[0].path, "links.2.over");
        assert_eq!(a.diff[0].before, None);
    }

    #[test]
    fn a_duration_is_written_as_a_duration() {
        let doc = architecture("workstation");
        let a = applied(
            &doc,
            r#"set(P, job_limit, 5m) :- part{id: P, block: "pool"}, setting{part: P, name: "job_limit", value: V}, V > 5m."#,
        );
        assert_eq!(a.document["parts"]["builds"]["job_limit"], "5m");
    }

    #[test]
    fn a_selector_without_answers_changes_nothing_and_applies() {
        let doc = architecture("workstation");
        let a = applied(
            &doc,
            r#"set(P, count, 3) :- part{id: P, block: "agent", count: 99}."#,
        );
        assert_eq!(a.document, doc);
        assert!(a.diff.is_empty());
        assert_eq!(a.changed_rows, 0);
    }

    #[test]
    fn a_dry_run_returns_the_diff_and_the_check_and_no_document() {
        let doc = architecture("workstation");
        let p = patch(r#"set(P, count, 4) :- part{id: P, count: 12}."#);
        let d = agenticspec_patch::dry_run(&doc, &p).unwrap();
        assert_eq!(d.diff.len(), 1);
        assert!(d.check.is_empty());
        assert_eq!(d.changed_rows, 1);
        assert_eq!(doc, architecture("workstation"));
    }

    #[test]
    fn the_digest_is_the_same_for_the_same_document_whatever_its_key_order() {
        let a: Value = serde_json::from_str(r#"{"b": 1, "a": {"y": [1, 2], "x": "s"}}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"a": {"x": "s", "y": [1, 2]}, "b": 1}"#).unwrap();
        assert_eq!(agenticspec_patch::digest(&a), agenticspec_patch::digest(&b));
        assert_eq!(agenticspec_patch::digest(&a).len(), 64);
        let c: Value = serde_json::from_str(r#"{"a": {"x": "s", "y": [2, 1]}, "b": 1}"#).unwrap();
        assert_ne!(agenticspec_patch::digest(&a), agenticspec_patch::digest(&c));
    }
}

mod refusal {
    use super::*;
    use agenticspec_patch::ErrorName;

    fn refused_with(text: &str) -> (Option<usize>, ErrorName) {
        let r = refused(&architecture("workstation"), text);
        (r.rule, r.error)
    }

    #[test]
    fn an_unknown_verb_refuses_the_whole_patch_at_its_rule() {
        let doc = architecture("workstation");
        let r = refused(
            &doc,
            r#"
            set(P, count, 2) :- part{id: P, block: "agent"}.
            frobnicate(P) :- part{id: P}.
            "#,
        );
        assert_eq!(r.rule, Some(2));
        assert_eq!(r.error, ErrorName::UnknownVerb);
        assert_eq!(r.error.name(), "UnknownVerb");
        assert!(r.to_string().contains("rule 2"), "{r}");
        assert!(r.to_string().contains("frobnicate"), "{r}");
        assert_eq!(doc, architecture("workstation"));
    }

    #[test]
    fn a_result_the_check_refuses_is_refused_without_a_rule() {
        let doc = architecture("workstation");
        let r = refused(&doc, r#"add(part{id: "lonely", block: "model"})."#);
        assert_eq!(r.rule, None);
        assert_eq!(r.error, ErrorName::CheckRefused);
        assert_eq!(r.check.len(), 1);
        assert_eq!(r.check[0].rule(), "unreachable_parts");
        assert_eq!(r.diff.len(), 1, "a refused check keeps the diff it judged");
    }

    #[test]
    fn a_dry_run_of_a_result_the_check_refuses_returns_its_diff_and_check() {
        let doc = architecture("workstation");
        let p = patch(r#"add(part{id: "lonely", block: "model"})."#);
        let r = agenticspec_patch::dry_run(&doc, &p).unwrap_err();
        assert_eq!(r.error, ErrorName::CheckRefused);
        assert_eq!(r.diff.len(), 1);
        assert_eq!(r.check.len(), 1);
    }

    #[test]
    fn each_error_at_its_rule() {
        let cases: &[(&str, ErrorName)] = &[
            ("set(P, count, 3 :- part{id: P}.", ErrorName::UnparsableRule),
            ("set(P, count, 3) :- part{id: P.", ErrorName::UnparsableRule),
            ("remove(P) :- part{id: P}.", ErrorName::UnsupportedVerb),
            ("set(P, count) :- part{id: P}.", ErrorName::InvalidHead),
            ("add(P) :- part{id: P}.", ErrorName::InvalidHead),
            (
                "set(P, count, 3) :- agent{id: P}.",
                ErrorName::InvalidSelector,
            ),
            (
                "set(P, count, 3) :- part{id: P, role: R}.",
                ErrorName::InvalidSelector,
            ),
            (
                "set(P, count, N) :- part{id: P}.",
                ErrorName::UnboundVariable,
            ),
            ("set(\"nobody\", count, 3).", ErrorName::NoSuchPart),
            ("set(99, carries, \"order\").", ErrorName::NoSuchLink),
            (
                "set(P, id, \"x\") :- part{id: P, block: \"model\"}.",
                ErrorName::ReadOnlyField,
            ),
            (
                "set(L, index, 4) :- link{index: L, carries: \"claim\"}.",
                ErrorName::ReadOnlyField,
            ),
            (
                "set(L, colour, \"red\") :- link{index: L, carries: \"claim\"}.",
                ErrorName::UnknownField,
            ),
            (
                "add(link{from: \"worker\", to: \"model\"}).",
                ErrorName::MissingField,
            ),
            ("add(part{id: \"x\"}).", ErrorName::MissingField),
            (
                "add(link{from: \"worker\", to: \"model\", carries: \"model_call\", via: \"x\"}).",
                ErrorName::UnknownField,
            ),
            (
                "set(P, count, 0) :- part{id: P, block: \"agent\"}.",
                ErrorName::InvalidCount,
            ),
            (
                "set(P, count, \"many\") :- part{id: P, block: \"agent\"}.",
                ErrorName::InvalidCount,
            ),
            (
                "add(part{id: \"worker\", block: \"agent\"}).",
                ErrorName::PartNameTaken,
            ),
            (
                "add(link{from: \"worker\", to: \"builds\", carries: \"claim\"}).",
                ErrorName::LinkExists,
            ),
        ];
        for (text, error) in cases {
            let two = format!("set(P, count, 2) :- part{{id: P, block: \"agent\"}}.\n{text}");
            let (rule, got) = refused_with(&two);
            assert_eq!((rule, got), (Some(2), *error), "{text}");
        }
    }

    #[test]
    fn two_values_for_one_place_in_one_rule_conflict() {
        let one = applied(
            &architecture("workstation"),
            r#"set("worker", harness, V) :- part{id: P}, setting{part: P, name: "harness", value: V}, P != "worker"."#,
        );
        assert!(
            one.diff.is_empty(),
            "one value from many answers is no conflict"
        );
        let (rule, error) =
            refused_with(r#"set("worker", role, B) :- part{id: P, block: B}, P != "worker"."#);
        assert_eq!((rule, error), (Some(1), ErrorName::ConflictingValues));
    }

    #[test]
    fn an_empty_patch_is_refused() {
        let r = Patch::from_text("  // nothing\n").unwrap_err();
        assert_eq!((r.rule, r.error), (None, ErrorName::EmptyPatch));
    }

    #[test]
    fn every_error_name_is_documented() {
        let page = std::fs::read_to_string(root().join("docs/studio.md")).unwrap();
        for e in ErrorName::ALL {
            assert!(
                page.contains(&format!("`{}`", e.name())),
                "{} is not in docs/studio.md",
                e.name()
            );
        }
    }
}

mod format {
    use super::*;
    use agenticspec_patch::ErrorName;

    #[test]
    fn the_text_form_splits_at_the_end_of_each_rule() {
        let p = patch(
            "set(P, route, \"a.b\") :- part{id: P, block: \"model\"}.\n// a comment.\nadd(part{id: \"x.y\", block: \"model\"}).",
        );
        assert_eq!(p.rules.len(), 2);
        assert_eq!(
            p.rules[0],
            "set(P, route, \"a.b\") :- part{id: P, block: \"model\"}."
        );
        assert_eq!(p.rules[1], "add(part{id: \"x.y\", block: \"model\"}).");
    }

    #[test]
    fn the_document_names_its_format_and_lists_its_rules() {
        let doc: Value = serde_json::json!({
            "format": "agenticspec-patch/1",
            "name": "fewer workers",
            "rules": ["set(P, count, 3) :- part{id: P, block: \"agent\", count: 12}."],
        });
        let p = Patch::from_document(&doc).unwrap();
        assert_eq!(p.name.as_deref(), Some("fewer workers"));
        assert_eq!(p.rules.len(), 1);
        assert_eq!(Patch::from_document(&p.to_document()).unwrap(), p);

        let other = serde_json::json!({"format": "agenticspec-patch/2", "rules": ["x(1)."]});
        let r = Patch::from_document(&other).unwrap_err();
        assert_eq!((r.rule, r.error), (None, ErrorName::UnknownFormat));
    }

    #[test]
    fn a_rule_without_a_final_dot_is_unparsable_at_its_position() {
        let p =
            patch("set(P, count, 3) :- part{id: P, count: 12}.\nset(P, count, 4) :- part{id: P}");
        assert_eq!(p.rules.len(), 2);
        let r = agenticspec_patch::apply(&architecture("workstation"), &p).unwrap_err();
        assert_eq!((r.rule, r.error), (Some(2), ErrorName::UnparsableRule));
    }
}
