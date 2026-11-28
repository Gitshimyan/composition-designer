use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join(name)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_composition-cli"))
        .args(args)
        .output()
        .expect("run composition-cli")
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid JSON output")
}

fn write_requests(label: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "composition-cli-batch-{}-{label}.json",
        std::process::id()
    ));
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn batch_command_runs_array_and_object_payloads() {
    let project = example("minimal-project");
    let project = project.to_str().unwrap();

    let array = write_requests(
        "array",
        r#"[{"id":"a","op":"inspect"},{"id":"b","op":"context"}]"#,
    );
    let output = run(&["batch", project, array.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0));
    let value = json(&output);
    assert_eq!(value["result"]["count"], 2);
    assert_eq!(value["result"]["results"][0]["id"], "a");
    assert_eq!(
        value["result"]["results"][1]["result"]["counts"]["instances"],
        2
    );

    let object = write_requests("object", r#"{"requests":[{"op":"validate"}]}"#);
    let output = run(&["batch", project, object.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        json(&output)["result"]["results"][0]["result"]["valid"],
        true
    );

    let _ = fs::remove_file(array);
    let _ = fs::remove_file(object);
}

#[test]
fn batch_command_rejects_write_ops_and_limits() {
    let project = example("minimal-project");
    let project_path = project.to_str().unwrap();

    let write = write_requests(
        "write",
        r#"[{"op":"prepare_transaction","params":{"transaction":{}}}]"#,
    );
    let output = run(&["batch", project_path, write.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        json(&output)["error"]["code"],
        "protocol-operation-not-allowed"
    );

    let requests: Vec<String> = (0..17)
        .map(|index| format!(r#"{{"id":"r{index}","op":"inspect"}}"#))
        .collect();
    let too_many = write_requests("too-many", &format!("[{}]", requests.join(",")));
    let output = run(&["batch", project_path, too_many.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json(&output)["error"]["code"], "protocol-invalid-params");

    let _ = fs::remove_file(write);
    let _ = fs::remove_file(too_many);
}

#[test]
fn batch_command_rejects_malformed_files() {
    let project = example("minimal-project");
    let project_path = project.to_str().unwrap();

    let malformed = write_requests("malformed", "{not json");
    let output = run(&["batch", project_path, malformed.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));

    let missing = write_requests("missing-requests", r#"{"items":[]}"#);
    let output = run(&["batch", project_path, missing.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));

    let _ = fs::remove_file(malformed);
    let _ = fs::remove_file(missing);
}
