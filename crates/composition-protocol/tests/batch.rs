use std::path::PathBuf;

use composition_protocol::{ProtocolSession, Request};
use serde_json::{json, Value};

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join(name)
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
fn batch_runs_read_only_requests_in_order() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    let response = dispatch(
        &mut session,
        "batch",
        json!({
            "requests": [
                { "id": "a", "op": "inspect" },
                { "id": "b", "op": "context" },
                { "id": "c", "op": "impact", "params": { "instance": "app" } }
            ]
        }),
    );
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["result"]["count"], 3);
    let results = response["result"]["results"].as_array().unwrap();
    assert_eq!(results[0]["id"], "a");
    assert_eq!(results[0]["ok"], true);
    assert_eq!(results[0]["result"]["project"]["id"], "example.minimal");
    assert_eq!(results[1]["result"]["counts"]["instances"], 2);
    assert_eq!(results[2]["result"]["kind"], "instance");
}

#[test]
fn batch_rejects_write_operations_and_unknown_ops() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    for op in ["prepare_transaction", "apply_transaction", "batch"] {
        let response = dispatch(&mut session, "batch", json!({ "requests": [{ "op": op }] }));
        assert_eq!(response["ok"], false, "{op}: {response}");
        assert_eq!(
            response["error"]["code"], "protocol-operation-not-allowed",
            "{op}"
        );
    }
}

#[test]
fn batch_enforces_size_limits() {
    let mut session = ProtocolSession::new(example("minimal-project"));
    let empty = dispatch(&mut session, "batch", json!({ "requests": [] }));
    assert_eq!(empty["ok"], false);
    assert_eq!(empty["error"]["code"], "protocol-invalid-params");

    let requests: Vec<Value> = (0..17)
        .map(|index| json!({ "id": format!("r{index}"), "op": "inspect" }))
        .collect();
    let too_many = dispatch(&mut session, "batch", json!({ "requests": requests }));
    assert_eq!(too_many["ok"], false);
    assert_eq!(too_many["error"]["code"], "protocol-invalid-params");
}

#[test]
fn batch_keeps_per_item_failures_independent() {
    let mut session = ProtocolSession::new(example("invalid-projects/files/entry-missing"));
    let response = dispatch(
        &mut session,
        "batch",
        json!({
            "requests": [
                { "id": "inspect", "op": "inspect" },
                { "id": "source", "op": "source" }
            ]
        }),
    );
    assert_eq!(response["ok"], true, "{response}");
    let results = response["result"]["results"].as_array().unwrap();
    assert_eq!(results[0]["ok"], true);
    assert_eq!(results[1]["ok"], false);
    assert_eq!(results[1]["error"]["code"], "project-not-editable");
}
