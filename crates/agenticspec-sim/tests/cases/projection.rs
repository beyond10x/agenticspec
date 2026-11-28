//! The projections of an `http` export: the `ess-http/1` protocol binding over exactly the
//! interface's commands and views, the OpenAPI document narrowed to them, and the pin of the
//! compiled blocks. The functions are pure; `agenticspec ingress project --check` (the gate's
//! `check:projections`) compares the committed projections of the worked example with a fresh
//! run of `ess`.

use super::common::{repo_root, scratch};
use agenticspec_sim::cli::{self, ProjectOptions};
use agenticspec_sim::contract::derive::{self, Contract};
use agenticspec_sim::contract::ingress::{self, Kpis};
use agenticspec_sim::contract::ports::PortsFile;
use agenticspec_sim::contract::projection::{self as pr, Pin};
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
