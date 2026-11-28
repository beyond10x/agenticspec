//! `agenticspec ui derive` per ingress (`design/ingress.md` I10): the committed example of
//! `architectures/workstation-ingress.yaml`, whose `user` and `operator` ingresses each have a
//! `ui` export, gives two compositions, each scoped to its ingress's interface; a contract whose
//! only `ui` export is the operator ingress's keeps its one unscoped operator UI.

use std::collections::BTreeSet;

use agenticspec_ui::derive::ingress;
use serde_json::Value as Json;

use super::{Derivation, actions, derive, document, inputs, root};

const EXAMPLES: &str = "ui/views/examples";

fn contract(name: &str) -> Json {
    let text = std::fs::read_to_string(root().join(EXAMPLES).join(format!("{name}.contract.yaml")))
        .unwrap();
    serde_yaml_ng::from_str(&text).unwrap()
}

/// Every composition of the workstation with ingresses, derived twice from one compilation of
/// the blocks.
/// The compositions of one derivation, each with its ingress.
type All = Vec<(Option<String>, Derivation)>;

fn derived_twice() -> (All, All) {
    let r = root();
    let i = inputs(
        &r.join("architectures/workstation-ingress.yaml"),
        Some("ui/views/examples/workstation-ingress.contract.yaml"),
        Some(r.join(EXAMPLES).join("ports.json")),
        r.join(EXAMPLES).join("workstation-ingress.ui.yaml"),
    );
    let ir = agenticspec_views::compile(&r.join("blocks")).unwrap();
    let a = derive::derive_all(&i, &ir).unwrap_or_else(|_| panic!("derived"));
    let b = derive::derive_all(&i, &ir).unwrap_or_else(|_| panic!("derived"));
    (a, b)
}

/// Each action as `page command`, sorted.
fn commands(d: &Derivation, out: &std::path::Path) -> Vec<String> {
    let (_, doc) = document(d, out);
    let mut list: Vec<String> = actions(&doc)
        .into_iter()
        .map(|(at, a)| format!("{at} {}", a["does"].as_str().unwrap_or_default()))
        .collect();
    list.sort();
    list
}

#[test]
fn the_workstation_with_ingresses_derives_one_composition_per_ingress_with_a_ui_export() {
    let (all, again) = derived_twice();
    let names: Vec<Option<&str>> = all.iter().map(|(n, _)| n.as_deref()).collect();
    assert_eq!(names, [Some("operator"), Some("user")]);
    for ((_, a), (_, b)) in all.iter().zip(&again) {
        assert_eq!(a.files, b.files, "deterministic");
        let stale = derive::differences(a);
        assert!(stale.is_empty(), "stale: {stale:?}; run task ui:derive");
    }
    let examples = root().join(EXAMPLES);
    let operator = examples.join("workstation-ingress.operator.ui.yaml");
    let user = examples.join("workstation-ingress.user.ui.yaml");

    // The user's actions are the two commands of its interface, where the derivation places them.
    assert_eq!(
        commands(&all[1].1, &user),
        [
            "part.operator agentic.channel.SendMessage",
            "part.owner/question_open_questions agentic.question.Answer",
        ]
    );
    // The operator's are the administration commands of the gateway, and no configuration.
    let op = commands(&all[0].1, &operator);
    let set: BTreeSet<&str> = op.iter().filter_map(|c| c.rsplit(' ').next()).collect();
    assert_eq!(
        set,
        BTreeSet::from([
            "agentic.gateway.DetachUpstream",
            "agentic.gateway.DrainGateway",
            "agentic.gateway.StopGateway",
        ]),
        "{op:?}"
    );

    // Each composition shows only the views of its ingress, carries only its feeds and says
    // which ingress it is.
    let c = contract("workstation-ingress");
    for (name, out) in [("operator", &operator), ("user", &user)] {
        let scope = ingress::scope(Some(&c), Some(name)).unwrap().unwrap();
        let d = &all
            .iter()
            .find(|(n, _)| n.as_deref() == Some(name))
            .unwrap()
            .1;
        let (_, doc) = document(d, out);
        assert_eq!(
            doc["title"].as_str(),
            Some(format!("workstation-ingress ({name})").as_str())
        );
        for (page, p) in doc["pages"].as_mapping().unwrap() {
            for s in p["sections"].as_sequence().into_iter().flatten() {
                let widget = s["component"]
                    .as_str()
                    .filter(|w| w.starts_with("agentic."))
                    .or_else(|| s["reads"]["view"].as_str())
                    .unwrap_or_default();
                if widget.starts_with("agentic.") && !widget.starts_with("agentic.architecture.") {
                    assert!(
                        scope.views.contains(widget),
                        "{name}: {page:?} shows {widget}, no view of the ingress"
                    );
                }
            }
        }
        for (_, ch) in doc["channels"].as_mapping().into_iter().flatten() {
            for e in ch["carries"]["events"].as_sequence().into_iter().flatten() {
                assert!(scope.feeds.contains(e.as_str().unwrap()), "{name}: {e:?}");
            }
        }
        let layout: Json = serde_json::from_str(
            &d.files
                .iter()
                .find(|(p, _)| p.to_string_lossy().ends_with(".layout.json"))
                .unwrap()
                .1,
        )
        .unwrap();
        assert_eq!(layout["ingress"], name);
    }
    // The user has no page for a part none of whose panels it may read.
    let (_, doc) = document(&all[1].1, &user);
    let pages = doc["pages"].as_mapping().unwrap();
    assert!(pages.get("part.gateway").is_none() && pages.get("part.builds").is_none());
    assert!(pages.get("part.operator").is_some());
}

#[test]
fn the_ui_of_an_ingress_is_its_ui_export_and_one_operator_ui_stays_unscoped() {
    let c = contract("workstation-ingress");
    let names: Vec<String> = ingress::ui_ingresses(&c)
        .into_iter()
        .map(|i| i.name)
        .collect();
    assert_eq!(names, ["operator", "user"]);
    assert!(ingress::per_ingress(&c));
    let user = ingress::scope(Some(&c), Some("user")).unwrap().unwrap();
    assert_eq!(
        user.commands,
        ["agentic.channel.SendMessage", "agentic.question.Answer"]
    );
    let err = ingress::scope(Some(&c), Some("metrics")).unwrap_err();
    assert_eq!(err.path, "contract/ingress");
    assert!(
        err.message.contains(
            "no ingress metrics with a ui export; the contract gives a UI to: operator, user"
        ),
        "{}",
        err.message
    );

    // The workstation's only ui export is the default operator ingress's: not scoped.
    let ws = contract("workstation");
    assert!(!ingress::per_ingress(&ws));
    assert_eq!(ingress::scope(Some(&ws), None).unwrap(), None);
    assert_eq!(ingress::scope(Some(&ws), Some("operator")).unwrap(), None);
    assert!(ingress::scope(Some(&ws), Some("user")).is_err());
    assert_eq!(ingress::scope(None, None).unwrap(), None);
    assert!(ingress::scope(None, Some("user")).is_err());
}
