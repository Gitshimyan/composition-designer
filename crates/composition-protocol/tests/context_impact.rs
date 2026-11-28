use std::path::PathBuf;

use composition_protocol::{ProtocolSession, Request};
use serde_json::{json, Value};

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join(name)
}

fn pilot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../working/pilots/glide-task-terminal-v2")
}

fn dispatch(session: &mut ProtocolSession, op: &str, params: Value) -> Value {
    let response = session.dispatch(Request {
        id: "test".to_owned(),
        op: op.to_owned(),
        params,
    });
    serde_json::to_value(response).unwrap()
}

#[test]
fn context_summarizes_project_without_config_values() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    let response = dispatch(&mut session, "context", json!({}));
    assert_eq!(response["ok"], true, "{response}");
    let result = &response["result"];
    assert_eq!(result["project"]["id"], "example.minimal");
    assert_eq!(result["counts"]["modules"], 2);
    assert_eq!(result["counts"]["instances"], 2);
    assert_eq!(result["counts"]["structureInstances"], 2);
    assert_eq!(result["counts"]["services"], 0);
    assert_eq!(result["revision"]["seq"], 0);
    assert!(result["modules"]
        .as_array()
        .unwrap()
        .iter()
        .all(|module| module["interfaces"].as_array().unwrap().is_empty()));
}

#[test]
fn instance_impact_reports_bindings_and_dependents() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    let response = dispatch(&mut session, "impact", json!({ "instance": "hello" }));
    assert_eq!(response["ok"], true, "{response}");
    let result = &response["result"];
    assert_eq!(result["kind"], "instance");
    assert_eq!(result["instance"]["instance"]["id"], "hello");
    let parents = result["instance"]["parentBindings"].as_array().unwrap();
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0]["id"], "app-content");
    assert_eq!(parents[0]["parent"], "app");
}

#[test]
fn module_impact_reports_instances_and_dependents() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    let response = dispatch(
        &mut session,
        "impact",
        json!({ "module": { "id": "leaf.widget", "version": "1.0.0" } }),
    );
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["result"]["kind"], "module");
    assert_eq!(response["result"]["module"]["instances"], json!(["hello"]));

    let missing = dispatch(
        &mut session,
        "impact",
        json!({ "module": { "id": "leaf.widget", "version": "9.9.9" } }),
    );
    assert_eq!(missing["ok"], false);
    assert_eq!(missing["error"]["code"], "protocol-unknown-target");
}

#[test]
fn impact_requires_exactly_one_target() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    let none = dispatch(&mut session, "impact", json!({}));
    assert_eq!(none["ok"], false);
    assert_eq!(none["error"]["code"], "protocol-invalid-params");

    let both = dispatch(
        &mut session,
        "impact",
        json!({ "instance": "app", "module": { "id": "leaf.widget", "version": "1.0.0" } }),
    );
    assert_eq!(both["ok"], false);
    assert_eq!(both["error"]["code"], "protocol-invalid-params");
}

#[test]
fn transaction_impact_prepares_without_writing() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    let before = dispatch(&mut session, "source", json!({}));
    let source_before = before["result"]["text"].as_str().unwrap().to_owned();

    let context = dispatch(&mut session, "context", json!({}));
    let digest = context["result"]["revision"]["source_digest"]
        .as_str()
        .unwrap()
        .to_owned();
    let transaction = json!({
        "request_id": "impact-preview",
        "base_revision": { "seq": 0, "source_digest": digest },
        "commands": [{
            "type": "set_config",
            "instance": "app",
            "property": "title",
            "value": "Impact Preview"
        }]
    });

    let response = dispatch(
        &mut session,
        "impact",
        json!({ "transaction": transaction }),
    );
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["result"]["kind"], "transaction");
    assert_eq!(response["result"]["transaction"]["accepted"], true);
    assert_eq!(
        response["result"]["transaction"]["changedObjects"],
        json!(["instance:app"])
    );

    let after = dispatch(&mut session, "source", json!({}));
    assert_eq!(after["result"]["text"], Value::String(source_before));
}

#[test]
fn v2_module_impact_reports_interface_implementers() {
    let mut session = ProtocolSession::new(pilot());
    let response = dispatch(
        &mut session,
        "impact",
        json!({
            "module": {
                "id": "glide.platform.conpty-session",
                "version": "1.0.0"
            }
        }),
    );
    assert_eq!(response["ok"], true, "{response}");
    let implementers = response["result"]["module"]["interfaceImplementers"]
        .as_array()
        .unwrap();
    assert_eq!(implementers.len(), 1);
    assert_eq!(implementers[0]["interface"]["id"], "glide.terminal.session");
    assert_eq!(implementers[0]["instances"], json!(["conpty-session"]));
    assert!(response["result"]["module"]["dependents"]
        .as_array()
        .unwrap()
        .iter()
        .any(|dependent| dependent["instance"] == "task-runner"));
}
