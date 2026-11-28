//! The rules of a panel, on the small system: one widget per view named as the view, its key,
//! columns, scope, parameters and live updates, and the gallery that places it.

use serde_json::{Value, json};

use crate::{generated, mini_blocks, model, parsed};

fn document() -> (Value, Vec<(String, String)>) {
    let (ir, blocks) = model(&mini_blocks());
    let files = generated(&ir, &blocks);
    (parsed(&files, "blocks.ui.yaml"), files)
}

fn column<'a>(widget: &'a Value, field: &str) -> &'a Value {
    widget["body"][0]["columns"]
        .as_array()
        .expect("columns")
        .iter()
        .find(|c| c == &&json!(field) || c["field"] == field)
        .unwrap_or_else(|| panic!("a column {field}"))
}

#[test]
fn one_widget_per_view_named_as_the_view() {
    let (doc, _) = document();
    let widgets: Vec<&String> = doc["widgets"]
        .as_object()
        .expect("widgets")
        .keys()
        .collect();
    assert_eq!(widgets, ["mini.thing.NotesOfThing", "mini.thing.Things"]);
    assert_eq!(doc["model"], "mini");
    let pages: Vec<&String> = doc["pages"].as_object().expect("pages").keys().collect();
    assert_eq!(
        pages,
        ["thing"],
        "a block without views gets no gallery page"
    );
}

#[test]
fn a_panel_reads_its_view_keyed_by_the_first_field_with_a_column_per_field() {
    let (doc, _) = document();
    let things = &doc["widgets"]["mini.thing.Things"];
    let rows = &things["body"][0];
    assert_eq!(rows["component"], "collection");
    assert_eq!(
        rows["reads"],
        json!({"view": "mini.thing.Things", "key": "thing_id"})
    );
    let fields: Vec<String> = rows["columns"]
        .as_array()
        .expect("columns")
        .iter()
        .map(|c| {
            c.as_str().map_or_else(
                || c["field"].as_str().unwrap_or("").to_string(),
                str::to_string,
            )
        })
        .collect();
    assert_eq!(
        fields,
        [
            "thing_id", "kind", "label", "made_at", "weight", "tags", "state"
        ]
    );
    assert_eq!(
        column(things, "thing_id"),
        &json!("thing_id"),
        "an identifier is plain text"
    );
    assert_eq!(column(things, "state")["as"], "badge");
    assert_eq!(column(things, "kind")["as"], "badge");
    assert_eq!(column(things, "made_at")["as"], "timestamp");
    assert_eq!(column(things, "weight")["as"], "number");
    assert_eq!(column(things, "tags")["as"], "list");
}

#[test]
fn every_panel_takes_the_scope_and_nothing_binds_it_to_a_read() {
    let (doc, _) = document();
    for (name, w) in doc["widgets"].as_object().expect("widgets") {
        for p in ["source", "architecture", "part", "instance"] {
            assert!(w["params"][p].is_object(), "{name} takes {p}");
            assert!(
                w["params"][p]["required"].is_null(),
                "{name}: {p} is optional"
            );
        }
        let reads = w["body"][0]["reads"].to_string();
        assert!(
            !reads.contains("args.architecture") && !reads.contains("args.part"),
            "{name}: {reads}"
        );
    }
}

#[test]
fn live_refetches_unless_every_event_carries_every_field() {
    let (doc, _) = document();
    let things = &doc["widgets"]["mini.thing.Things"]["body"][0]["live"];
    assert_eq!(things["channel"], "args.channel");
    assert_eq!(
        things["on"],
        json!(["mini.thing.ThingMade", "mini.thing.ThingRetired"])
    );
    assert_eq!(things["effect"], "refetch");
    assert_eq!(things["coalesce"], "1s");

    let notes = &doc["widgets"]["mini.thing.NotesOfThing"]["body"][0]["live"];
    assert_eq!(notes["on"], json!(["mini.thing.NoteWritten"]));
    assert_eq!(notes["effect"], "insert_or_patch");
    assert_eq!(notes["match"], "note_id");
    assert_eq!(notes["only_if"], "matches(params)");

    let channel = &doc["channels"]["thing"];
    assert_eq!(
        channel["carries"]["events"],
        json!([
            "mini.thing.NoteWritten",
            "mini.thing.ThingMade",
            "mini.thing.ThingRetired"
        ])
    );
    assert_eq!(channel["resume"], "from_last_seen");
}

#[test]
fn a_required_view_parameter_is_a_required_panel_parameter_the_gallery_fills_from_the_fixture() {
    let (doc, files) = document();
    let notes = &doc["widgets"]["mini.thing.NotesOfThing"];
    assert_eq!(notes["params"]["thing_id"]["required"], true);
    assert_eq!(notes["params"]["thing_id"]["type"], "ThingId");
    assert_eq!(
        notes["body"][0]["reads"]["params"],
        json!({"thing_id": "args.thing_id"})
    );

    let fixture = parsed(&files, "fixtures/mini.thing.NotesOfThing.json");
    let rows = fixture["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 3);
    let first = rows[0]["thing_id"].as_str().expect("thing_id");
    assert!(
        rows.iter().all(|r| r["thing_id"] == first),
        "every row answers the gallery's parameter"
    );

    let things = parsed(&files, "fixtures/mini.thing.Things.json");
    assert_eq!(
        things["rows"][0]["thing_id"], first,
        "row 0 names the same thing in both views"
    );

    let section = doc["pages"]["thing"]["sections"]
        .as_array()
        .expect("sections")
        .iter()
        .find(|s| s["component"] == "mini.thing.NotesOfThing")
        .expect("the gallery places the panel");
    assert_eq!(section["args"]["thing_id"], format!("'{first}'"));
    assert_eq!(section["args"]["channel"], "thing");
}

#[test]
fn the_gallery_titles_a_section_by_the_display_name_of_its_view() {
    let (doc, _) = document();
    let sections = doc["pages"]["thing"]["sections"]
        .as_array()
        .expect("sections");
    let names: Vec<(&str, &str)> = sections
        .iter()
        .map(|s| {
            (
                s["name"].as_str().unwrap_or(""),
                s["title"].as_str().unwrap_or(""),
            )
        })
        .collect();
    assert_eq!(
        names,
        [("notes_of_thing", "NotesOfThing"), ("things", "All things")]
    );
    assert_eq!(doc["navigation"]["sections"][0]["label"], "Layer 1: things");
}

#[test]
fn a_view_parameter_named_like_the_scope_is_refused() {
    let (mut ir, blocks) = model(&mini_blocks());
    ir["views"]["mini.thing.NotesOfThing"]["params"][0]["name"] = json!("part");
    let err = agenticspec_views::panels::generate(&ir, &blocks).expect_err("a clash is refused");
    assert!(
        err.contains("mini.thing.NotesOfThing") && err.contains("`part`"),
        "{err}"
    );
}

#[test]
fn the_same_model_gives_the_same_bytes() {
    let (ir, blocks) = model(&mini_blocks());
    assert_eq!(generated(&ir, &blocks), generated(&ir, &blocks));
}
