//! The projections of an ingress's exports. Of an `http` export: the `ess-http/1` protocol binding
//! over exactly the interface's commands and views, the OpenAPI document narrowed to them, and the
//! pin of the compiled blocks. Of the other protocols: the stream list, the `ess-cli/2` binding,
//! the tool list, the agent card and the metric list, each with its refusal. The functions are
//! pure, over stubs of the compiled blocks and their OpenAPI document; `agenticspec ingress
//! project --check` (the gate's `check:projections`) compares the committed projections of the
//! worked example with a fresh run of `ess`, and the last test reads them.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, ProjectOptions};
use agenticspec_sim::contract::derive::{self, Contract};
use agenticspec_sim::contract::ingress::{self, Kpis};
use agenticspec_sim::contract::ports::PortsFile;
use agenticspec_sim::contract::projection::{
    self as pr, Pin, agent, cli as pcli, metrics, stream, tool,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn contract(arch: &str) -> Contract {
    let root = repo_root();
    let ports = PortsFile::load(&root.join("blocks/ports.json")).unwrap();
    let kpis = Kpis::load(&root.join("kpis/catalogue.yaml")).unwrap();
    let doc = agenticspec_sim::doc::load_yaml(&root.join("architectures").join(arch)).unwrap();
    let mut c = derive::derive(&doc, &ports).unwrap();
    let (resolved, refusals) = ingress::resolve(&doc, &c, &ports, &kpis);
    assert_eq!(refusals, Vec::new());
    c.ingress = resolved;
    c
}

fn pin() -> Pin {
    Pin {
        system: "agentic".into(),
        version: "v1".into(),
        source_digest: format!("sha256:{}", "0".repeat(64)),
    }
}

#[test]
fn the_pin_digests_the_compact_ir_with_its_keys_in_the_order_written() {
    let text = "{\n  \"system\": \"agentic\",\n  \"version\": 1,\n  \"summary\": \"a \\\"quoted\\\" text, with spaces\",\n  \"a\": [1, 2]\n}\n";
    let compact = pr::compact(text);
    assert_eq!(
        compact,
        "{\"system\":\"agentic\",\"version\":1,\"summary\":\"a \\\"quoted\\\" text, with spaces\",\"a\":[1,2]}"
    );
    let p = Pin::of_ir(text).unwrap();
    let digest: String = Sha256::digest(compact.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(p.source_digest, format!("sha256:{digest}"));
    assert_eq!((p.system.as_str(), p.version.as_str()), ("agentic", "v1"));
    assert!(Pin::of_ir("{\"version\": 1}").is_err());
}

#[test]
fn the_binding_routes_exactly_the_commands_and_views_of_the_interface_with_open_paths() {
    let c = contract("workstation-ingress.yaml");
    let user = c.ingress.iter().find(|i| i.name == "user").unwrap();
    let text = pr::http_binding(&c, user, &pin());
    let doc = agenticspec_sim::doc::parse_yaml(&text).unwrap();
    assert_eq!(doc["type"], "ess-http/1");
    assert_eq!(doc["specification"]["system"], "agentic");
    let routes = doc["routes"].as_array().unwrap();
    let carried = user
        .operations
        .iter()
        .filter(|o| matches!(o.kind, ingress::OpKind::Command | ingress::OpKind::View))
        .count();
    assert_eq!(routes.len(), carried);
    assert_eq!(
        routes[0],
        json!({"command": "agentic.channel.SendMessage", "method": "post"})
    );
    assert!(routes.iter().all(|r| r.get("path").is_none()));
    assert!(
        routes
            .iter()
            .any(|r| r == &json!({"view": "agentic.question.OpenQuestions", "method": "get"}))
    );
    assert_eq!(text, pr::http_binding(&c, user, &pin()));
    // The metrics ingress has no http export, so no projection.
    let names: Vec<&str> = pr::http_ingresses(&c)
        .iter()
        .map(|i| i.name.as_str())
        .collect();
    assert_eq!(names, ["operator", "user"]);
}

fn full() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {"title": "blocks", "version": "v1"},
        "tags": [{"name": "channel"}, {"name": "gateway"}, {"name": "question"}],
        "paths": {
            "/channel/commands/send-message": {"post": {
                "operationId": "agentic.channel.SendMessage", "tags": ["channel"],
                "requestBody": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Send.Input"}}}}
            }},
            "/gateway/commands/drain-gateway": {"post": {
                "operationId": "agentic.gateway.DrainGateway", "tags": ["gateway"],
                "requestBody": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Drain.Input"}}}}
            }},
            "/question/views/open-questions": {"get": {
                "operationId": "agentic.question.OpenQuestions", "tags": ["question"],
                "responses": {"200": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Open.Response"}}}}}
            }}
        },
        "components": {"schemas": {
            "Send.Input": {"properties": {"channel_id": {"$ref": "#/components/schemas/ChannelId"}}},
            "ChannelId": {"type": "string"},
            "Drain.Input": {"properties": {"gateway_id": {"$ref": "#/components/schemas/GatewayId"}}},
            "GatewayId": {"type": "string"},
            "Open.Response": {"type": "object"}
        }},
        "x-ess-entities": {"agentic.gateway.Gateway": {"type": "object"}}
    })
}

#[test]
fn the_openapi_document_keeps_the_interface_operations_and_what_they_reference() {
    let mut c = contract("workstation-ingress.yaml");
    let user = c.ingress.iter_mut().find(|i| i.name == "user").unwrap();
    user.operations.retain(|o| {
        o.name == "agentic.channel.SendMessage" || o.name == "agentic.question.OpenQuestions"
    });
    let user = user.clone();
    let doc = pr::openapi(&full(), &c, &user).unwrap();
    let paths: Vec<&String> = doc["paths"].as_object().unwrap().keys().collect();
    assert_eq!(
        paths,
        [
            "/channel/commands/send-message",
            "/question/views/open-questions"
        ]
    );
    let schemas: Vec<&String> = doc["components"]["schemas"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(schemas, ["ChannelId", "Open.Response", "Send.Input"]);
    assert_eq!(
        doc["tags"],
        json!([{"name": "channel"}, {"name": "question"}])
    );
    assert!(doc.get("x-ess-entities").is_none());
    assert_eq!(doc["info"]["title"], "workstation-ingress ingress user");
    assert_eq!(doc["info"]["x-agenticspec-ingress"]["audience"], "owner");
    let text = pr::openapi_text(&c, &user, &doc).unwrap();
    assert_eq!(text, pr::openapi_text(&c, &user, &doc).unwrap());

    // An operation of the interface the document lacks is refused by name.
    let mut more = user.clone();
    more.operations.push(ingress::Operation {
        kind: ingress::OpKind::View,
        name: "agentic.question.Questions".into(),
    });
    let err = pr::openapi(&full(), &c, &more).unwrap_err();
    assert!(err.contains("agentic.question.Questions"), "{err}");
}

#[test]
fn ingress_project_needs_one_of_out_and_check_and_refuses_a_refused_document() {
    let root = repo_root();
    let dir = scratch("project-options");
    let arch = root.join("architectures/workstation.yaml");
    let err = cli::ingress_project(&ProjectOptions {
        architecture: &arch,
        out: None,
        check: None,
        blocks: None,
        ports: None,
        kpis: None,
    })
    .unwrap_err();
    assert_eq!(err.code, "projection_refused");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases/ingress/ingress_reserved_part.yaml");
    let (ports, kpis) = (
        root.join("blocks/ports.json"),
        root.join("kpis/catalogue.yaml"),
    );
    let err = cli::ingress_project(&ProjectOptions {
        architecture: &fixture,
        out: Some(&dir),
        check: None,
        blocks: Some(&root.join("blocks")),
        ports: Some(&ports),
        kpis: Some(&kpis),
    })
    .unwrap_err();
    assert_eq!(err.code, "architecture_refused");
    assert!(std::fs::read_dir(&dir).unwrap().next().is_none());
}

/// The ingress `user` of the worked example, its operations cut to those named and `extra`
/// added.
fn user_with(
    c: &Contract,
    keep: &[&str],
    extra: &[(ingress::OpKind, &str)],
) -> ingress::ContractIngress {
    let mut user = c.ingress.iter().find(|i| i.name == "user").unwrap().clone();
    user.operations.retain(|o| keep.contains(&o.name.as_str()));
    for (kind, name) in extra {
        user.operations.push(ingress::Operation {
            kind: *kind,
            name: (*name).into(),
        });
    }
    user.operations.sort();
    user
}

/// A stub of the compiled blocks: two commands, two views (one with a parameter) and two events.
fn ir() -> Value {
    json!({
        "system": "agentic",
        "version": 1,
        "commands": {
            "agentic.channel.SendMessage": {
                "name": "agentic.channel.SendMessage",
                "naming": {"wire": "send-message", "display": "Send a message", "summary": "Send one message."},
                "input": [{"name": "channel_id"}, {"name": "body_text"}],
                "outcomes": [
                    {"name": "sent", "error": null},
                    {"name": "closed", "error": "agentic.channel.ChannelIsClosed"},
                    {"name": "unknown", "error": "agentic.channel.UnknownChannel"},
                    {"name": "closed-again", "error": "agentic.channel.ChannelIsClosed"}
                ]
            },
            "agentic.gateway.DrainGateway": {
                "name": "agentic.gateway.DrainGateway",
                "naming": {"wire": "drain-gateway", "display": "Drain a gateway"},
                "input": [{"name": "gateway_id"}],
                "outcomes": [{"name": "draining", "error": null}]
            }
        },
        "views": {
            "agentic.question.OpenQuestions": {
                "name": "agentic.question.OpenQuestions",
                "naming": {"wire": "open-questions", "display": "Open questions"},
                "params": [{"name": "channel_id"}]
            },
            "agentic.channel.Channels": {"name": "agentic.channel.Channels", "naming": {}}
        },
        "events": {
            "agentic.channel.MessageSent": {"name": "agentic.channel.MessageSent", "naming": {}},
            "agentic.question.QuestionAsked": {"name": "agentic.question.QuestionAsked", "naming": {}}
        }
    })
}

/// [`full`] with the view `OpenQuestions` taking a parameter.
fn full_with_parameter() -> Value {
    let mut full = full();
    full["paths"]["/question/views/open-questions"]["get"]["parameters"] = json!([{
        "name": "channel_id", "in": "query", "required": true,
        "description": "Read by the view's filter.",
        "schema": {"$ref": "#/components/schemas/ChannelId"}
    }]);
    full
}

#[test]
fn the_stream_list_carries_every_feed_of_one_level_resumable_from_a_cursor() {
    let c = contract("workstation-ingress.yaml");
    let user = user_with(
        &c,
        &[],
        &[
            (ingress::OpKind::Feed, "agentic.channel.MessageSent"),
            (ingress::OpKind::Feed, "agentic.question.QuestionAsked"),
        ],
    );
    let text = stream::streams(&c, &user, &pin(), &ir()).unwrap();
    assert_eq!(text, stream::streams(&c, &user, &pin(), &ir()).unwrap());
    let doc = agenticspec_sim::doc::parse_yaml(&text).unwrap();
    assert_eq!(doc["format"], "agenticspec-streams/1");
    assert_eq!(doc["specification"]["system"], "agentic");
    assert_eq!(doc["ingress"]["audience"], "owner");
    let streams = doc["streams"].as_array().unwrap();
    assert_eq!(streams.len(), 1);
    let s = &streams[0];
    assert_eq!(s["name"], "workstation-ingress");
    assert_eq!(
        (&s["resume"], &s["expired"], &s["delivery"], &s["scope"]),
        (
            &json!("from_last_seen"),
            &json!("CursorExpired"),
            &json!("every_event"),
            &json!("param")
        )
    );
    assert_eq!(
        s["carries"],
        json!([
            "agentic.channel.MessageSent",
            "agentic.question.QuestionAsked"
        ])
    );

    // An event the compiled blocks lack is refused by name.
    let mut lacking = ir();
    lacking["events"]
        .as_object_mut()
        .unwrap()
        .remove("agentic.question.QuestionAsked");
    let err = stream::streams(&c, &user, &pin(), &lacking).unwrap_err();
    assert!(
        err.contains("no event agentic.question.QuestionAsked"),
        "{err}"
    );
}

#[test]
fn the_cli_binding_forwards_each_command_and_view_with_an_option_per_field() {
    let c = contract("workstation-ingress.yaml");
    let user = user_with(
        &c,
        &[
            "agentic.channel.SendMessage",
            "agentic.question.OpenQuestions",
        ],
        &[(ingress::OpKind::View, "agentic.channel.Channels")],
    );
    let text = pcli::cli_binding(&c, &user, &ir(), "blocks").unwrap();
    assert_eq!(text, pcli::cli_binding(&c, &user, &ir(), "blocks").unwrap());
    let doc = agenticspec_sim::doc::parse_yaml(&text).unwrap();
    assert_eq!(doc["format"], "ess-cli/2");
    assert_eq!(doc["binary"], "workstation-ingress-user");
    assert_eq!(doc["globals"], json!({"state": "state-dir"}));
    let send = &doc["callables"]["channel_send_message"];
    assert_eq!(
        send["target"],
        json!({"kind": "service_forward", "owner": "blocks", "operation": "agentic.channel.SendMessage"})
    );
    assert_eq!(send["input"], "agentic.channel.SendMessage.Input");
    assert_eq!(send["result"], "agentic.channel.SendMessage.Outcome");
    let open = &doc["callables"]["question_open_questions"];
    assert_eq!(open["input"], "agentic.question.OpenQuestions.Params");
    assert_eq!(open["result"], "List<agentic.question.OpenQuestions.Row>");
    // A view without parameters takes no input; without a wire name its path is its short name.
    assert_eq!(doc["callables"]["channel_channels"]["input"], Value::Null);
    let commands = doc["commands"].as_array().unwrap();
    let paths: Vec<&Value> = commands.iter().map(|c| &c["path"]).collect();
    assert_eq!(
        paths,
        [
            &json!(["channel", "send-message"]),
            &json!(["channel", "channels"]),
            &json!(["question", "open-questions"])
        ]
    );
    assert_eq!(commands[0]["about"], "Send one message.");
    assert_eq!(
        commands[0]["arguments"],
        json!([
            {"field": "channel_id", "source": {"kind": "option", "long": "channel-id"}},
            {"field": "body_text", "source": {"kind": "option", "long": "body-text"}}
        ])
    );

    // An operation the compiled blocks lack, and two operations on one path, are refused.
    let more = user_with(
        &c,
        &["agentic.channel.SendMessage"],
        &[(ingress::OpKind::View, "agentic.channel.Missing")],
    );
    let err = pcli::cli_binding(&c, &more, &ir(), "blocks").unwrap_err();
    assert!(
        err.contains("no operation agentic.channel.Missing"),
        "{err}"
    );
    let mut clash = ir();
    clash["views"]["agentic.channel.Channels"]["naming"]["wire"] = json!("send-message");
    let err = pcli::cli_binding(&c, &user, &clash, "blocks").unwrap_err();
    assert!(err.contains("command path `channel send-message`"), "{err}");
}

#[test]
fn the_tool_list_names_each_tool_by_its_wire_name_with_an_input_schema_that_stands_alone() {
    let c = contract("workstation-ingress.yaml");
    let user = user_with(
        &c,
        &[
            "agentic.channel.SendMessage",
            "agentic.question.OpenQuestions",
        ],
        &[],
    );
    let full = full_with_parameter();
    let text = tool::tools(&c, &user, &pin(), &ir(), &full).unwrap();
    assert_eq!(text, tool::tools(&c, &user, &pin(), &ir(), &full).unwrap());
    let doc = agenticspec_sim::doc::parse_yaml(&text).unwrap();
    assert_eq!(doc["format"], "agenticspec-tools/1");
    let tools = doc["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    let send = &tools[0];
    assert_eq!(send["name"], "send-message");
    assert_eq!(send["description"], "Send one message.");
    assert_eq!(send["operation"], "agentic.channel.SendMessage");
    assert_eq!(
        send["errors"],
        json!([
            "agentic.channel.ChannelIsClosed",
            "agentic.channel.UnknownChannel"
        ])
    );
    assert_eq!(send["annotations"]["readOnlyHint"], false);
    assert_eq!(
        send["inputSchema"]["properties"]["channel_id"]["$ref"],
        "#/$defs/ChannelId"
    );
    assert_eq!(
        send["inputSchema"]["$defs"],
        json!({"ChannelId": {"type": "string"}})
    );
    let open = &tools[1];
    assert_eq!(open["name"], "open-questions");
    assert_eq!(open["description"], "Open questions");
    assert!(open.get("errors").is_none());
    assert_eq!(open["annotations"]["readOnlyHint"], true);
    assert_eq!(open["inputSchema"]["required"], json!(["channel_id"]));
    assert_eq!(
        open["inputSchema"]["properties"]["channel_id"]["$ref"],
        "#/$defs/ChannelId"
    );
    assert_eq!(open["inputSchema"]["additionalProperties"], false);

    // An operation the OpenAPI document lacks is refused by name; so are two tools of one name.
    let more = user_with(
        &c,
        &["agentic.channel.SendMessage"],
        &[(ingress::OpKind::View, "agentic.question.Questions")],
    );
    let err = tool::tools(&c, &more, &pin(), &ir(), &full).unwrap_err();
    assert!(
        err.contains("no operation agentic.question.Questions"),
        "{err}"
    );
    let mut clash = full.clone();
    clash["paths"]["/gateway/views/send-message"] =
        json!({"get": {"operationId": "agentic.gateway.Gateways"}});
    let two = user_with(
        &c,
        &["agentic.channel.SendMessage"],
        &[(ingress::OpKind::View, "agentic.gateway.Gateways")],
    );
    let err = tool::tools(&c, &two, &pin(), &ir(), &clash).unwrap_err();
    assert!(
        err.contains("tool name send-message is the wire name of both"),
        "{err}"
    );
}

#[test]
fn the_agent_card_has_a_skill_per_command_and_its_feeds_as_updates() {
    let c = contract("workstation-ingress.yaml");
    let user = user_with(
        &c,
        &[
            "agentic.channel.SendMessage",
            "agentic.question.OpenQuestions",
        ],
        &[(ingress::OpKind::Feed, "agentic.channel.MessageSent")],
    );
    let full = full_with_parameter();
    let text = agent::agent_card(&c, &user, &pin(), &ir(), &full).unwrap();
    assert_eq!(
        text,
        agent::agent_card(&c, &user, &pin(), &ir(), &full).unwrap()
    );
    let doc = agenticspec_sim::doc::parse_yaml(&text).unwrap();
    assert_eq!(doc["format"], "agenticspec-agent-card/1");
    assert_eq!(doc["name"], "workstation-ingress");
    assert_eq!(doc["capabilities"]["streaming"], true);
    assert!(doc.get("url").is_none());
    let skills = doc["skills"].as_array().unwrap();
    assert_eq!(skills.len(), 1, "a view is no skill");
    assert_eq!(skills[0]["id"], "send-message");
    assert_eq!(skills[0]["name"], "Send a message");
    assert_eq!(
        skills[0]["inputSchema"]["properties"]["channel_id"]["$ref"],
        "#/$defs/ChannelId"
    );
    assert_eq!(doc["updates"], json!(["agentic.channel.MessageSent"]));

    // A feed whose event the compiled blocks lack is refused by name.
    let more = user_with(
        &c,
        &["agentic.channel.SendMessage"],
        &[(ingress::OpKind::Feed, "agentic.channel.Missing")],
    );
    let err = agent::agent_card(&c, &more, &pin(), &ir(), &full).unwrap_err();
    assert!(err.contains("no event agentic.channel.Missing"), "{err}");
}

#[test]
fn the_metric_list_holds_a_gauge_per_kpi_with_its_unit_and_the_placement_labels() {
    let c = contract("workstation-ingress.yaml");
    let kpis = Kpis::load(&repo_root().join("kpis/catalogue.yaml")).unwrap();
    let m = c.ingress.iter().find(|i| i.name == "metrics").unwrap();
    let text = metrics::metrics(&c, m, &kpis).unwrap();
    assert_eq!(text, metrics::metrics(&c, m, &kpis).unwrap());
    let doc = agenticspec_sim::doc::parse_yaml(&text).unwrap();
    assert_eq!(doc["format"], "agenticspec-metrics/1");
    assert_eq!(
        doc["labels"],
        json!(["architecture", "part_path", "instance"])
    );
    let list = doc["metrics"].as_array().unwrap();
    assert_eq!(list.len(), 5);
    for metric in list {
        let k = &kpis.0[metric["name"].as_str().unwrap()];
        assert_eq!(metric["type"], "gauge");
        assert_eq!(metric["unit"], k.unit.as_str());
        assert_eq!(metric["help"], k.summary.as_str());
    }

    // A KPI the catalogue lacks, or gives no unit, is refused by name.
    let mut fewer = kpis.clone();
    fewer.0.remove("cost_usd");
    fewer.0.get_mut("pool_utilization").unwrap().unit.clear();
    let err = metrics::metrics(&c, m, &fewer).unwrap_err();
    assert!(err.contains("cost_usd (not in the catalogue)"), "{err}");
    assert!(
        err.contains("pool_utilization (no unit or no summary)"),
        "{err}"
    );
}

#[test]
fn the_committed_projections_of_the_worked_example_hold_what_each_export_promises() {
    let c = contract("workstation-ingress.yaml");
    let dir = repo_root().join("architectures/projections/workstation-ingress");
    let mut expected: Vec<String> = pr::projected(&c)
        .into_iter()
        .flat_map(|(i, p)| pr::files_of(&i.name, p))
        .collect();
    expected.sort();
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|n| pr::is_projection(n))
        .collect();
    on_disk.sort();
    assert_eq!(on_disk, expected);
    assert!(pr::files_of("user", "ui").is_empty());
    let read = |name: &str| {
        agenticspec_sim::doc::load_yaml(&dir.join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
    };
    let count = |i: &ingress::ContractIngress, kind: ingress::OpKind| {
        i.operations.iter().filter(|o| o.kind == kind).count()
    };

    // The user's OpenAPI: exactly 2 command operations and one GET per resolved view.
    let user = c.ingress.iter().find(|i| i.name == "user").unwrap();
    let openapi = read("user.openapi.yaml");
    let (mut posts, mut gets) = (0, 0);
    for item in openapi["paths"].as_object().unwrap().values() {
        posts += usize::from(item.get("post").is_some());
        gets += usize::from(item.get("get").is_some());
    }
    assert_eq!((posts, gets), (2, count(user, ingress::OpKind::View)));

    // The operator's tool list: 3 command tools and one read tool per view, each with a schema.
    let operator = c.ingress.iter().find(|i| i.name == "operator").unwrap();
    let tools = read("operator.tools.yaml");
    let tools = tools["tools"].as_array().unwrap();
    let commands: Vec<&str> = tools
        .iter()
        .filter(|t| t["annotations"]["readOnlyHint"] == false)
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        commands,
        ["detach-upstream", "drain-gateway", "stop-gateway"]
    );
    assert_eq!(tools.len(), 3 + count(operator, ingress::OpKind::View));
    assert!(tools.iter().all(|t| t["inputSchema"]["type"] == "object"));

    // The metrics ingress: 5 gauges with the catalogue's units and the placement labels.
    let kpis = Kpis::load(&repo_root().join("kpis/catalogue.yaml")).unwrap();
    let list = read("metrics.metrics.yaml");
    assert_eq!(
        list["labels"],
        json!(["architecture", "part_path", "instance"])
    );
    let gauges = list["metrics"].as_array().unwrap();
    assert_eq!(gauges.len(), 5);
    for g in gauges {
        assert_eq!(g["type"], "gauge");
        assert_eq!(g["unit"], kpis.0[g["name"].as_str().unwrap()].unit.as_str());
    }

    // Each stream list carries every feed of its interface.
    for i in [user, operator] {
        let s = read(&format!("{}.streams.yaml", i.name));
        assert_eq!(
            s["streams"][0]["carries"].as_array().unwrap().len(),
            count(i, ingress::OpKind::Feed)
        );
    }
}
