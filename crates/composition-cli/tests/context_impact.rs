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

#[test]
fn context_command_returns_compact_summary() {
    let path = example("minimal-project");
    let output = run(&["context", path.to_str().unwrap()]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = json(&output);
    assert_eq!(value["ok"], true);
    assert_eq!(value["result"]["project"]["id"], "example.minimal");
    assert_eq!(value["result"]["counts"]["instances"], 2);
    assert_eq!(value["result"]["counts"]["modules"], 2);
}

#[test]
fn impact_command_supports_module_and_instance() {
    let path = example("minimal-project");
    let instance = run(&["impact", path.to_str().unwrap(), "--instance", "app"]);
    assert_eq!(instance.status.code(), Some(0));
    let value = json(&instance);
    assert_eq!(value["result"]["kind"], "instance");
    assert_eq!(
        value["result"]["instance"]["childBindings"][0]["child"],
        "hello"
    );

    let module = run(&[
        "impact",
        path.to_str().unwrap(),
        "--module",
        "leaf.widget@1.0.0",
    ]);
    assert_eq!(module.status.code(), Some(0));
    let value = json(&module);
    assert_eq!(value["result"]["kind"], "module");
    assert_eq!(
        value["result"]["module"]["instances"],
        serde_json::json!(["hello"])
    );
}

#[test]
fn impact_command_previews_transaction_without_writing() {
    let path = example("minimal-project");
    let composition = path.join("composition/main.toml");
    let before = fs::read_to_string(&composition).unwrap();

    let context = json(&run(&["context", path.to_str().unwrap()]));
    let digest = context["result"]["revision"]["source_digest"]
        .as_str()
        .unwrap()
        .to_owned();
    let transaction = serde_json::json!({
        "request_id": "cli-impact",
        "base_revision": { "seq": 0, "source_digest": digest },
        "commands": [{
            "type": "set_config",
            "instance": "app",
            "property": "title",
            "value": "CLI impact"
        }]
    });
    let transaction_path = std::env::temp_dir().join(format!(
        "composition-cli-impact-{}.json",
        std::process::id()
    ));
    fs::write(&transaction_path, serde_json::to_vec(&transaction).unwrap()).unwrap();

    let output = run(&[
        "impact",
        path.to_str().unwrap(),
        "--transaction",
        transaction_path.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0));
    let value = json(&output);
    assert_eq!(value["result"]["kind"], "transaction");
    assert_eq!(value["result"]["transaction"]["accepted"], true);
    assert_eq!(
        value["result"]["transaction"]["changedObjects"],
        serde_json::json!(["instance:app"])
    );
    assert_eq!(fs::read_to_string(&composition).unwrap(), before);
    let _ = fs::remove_file(&transaction_path);
}

#[test]
fn impact_command_rejects_ambiguous_targets() {
    let path = example("minimal-project");
    let output = run(&[
        "impact",
        path.to_str().unwrap(),
        "--instance",
        "app",
        "--module",
        "leaf.widget@1.0.0",
    ]);
    assert_eq!(output.status.code(), Some(2));
}
