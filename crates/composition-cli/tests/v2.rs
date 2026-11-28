use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const PROJECT_V2: &str = r#"[project]
schema_version = 2
id = "example.v2"
name = "V2 CLI"

[workspace]
composition = "composition/main.toml"
module_roots = ["modules"]
"#;

const COMPOSITION_V2: &str = r#"[composition]
schema_version = 2
id = "example.v2"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"
"#;

const APP_MODULE: &str = r#"[module]
schema_version = 2
id = "example.app"
version = "1.0.0"
kind = "composite"
"#;

const SERVICE_MODULE: &str = r#"[module]
schema_version = 2
id = "example.session"
version = "1.0.0"
kind = "primitive"

[[contract.interfaces]]
id = "example.session.api"
version = "1.0.0"
"#;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_composition-cli"))
        .args(args)
        .output()
        .expect("run composition-cli")
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid JSON output")
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text).unwrap();
}

fn v2_project(label: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("composition-cli-v2-{}-{label}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    write(&root, "project.toml", PROJECT_V2);
    write(&root, "composition/main.toml", COMPOSITION_V2);
    write(&root, "modules/app/module.toml", APP_MODULE);
    write(&root, "modules/session/module.toml", SERVICE_MODULE);
    root
}

#[test]
fn validate_and_inspect_expose_v2_fields() {
    let root = v2_project("inspect");
    let root_arg = root.to_string_lossy().into_owned();

    let validated = run(&["validate", &root_arg, "--json"]);
    assert_eq!(validated.status.code(), Some(0));
    let validate = json(&validated);
    assert_eq!(validate["valid"], true);
    assert!(validate["source_digest"].as_str().is_some());

    let inspected = run(&["inspect", &root_arg, "--json"]);
    assert_eq!(inspected.status.code(), Some(0));
    let inspect = json(&inspected);
    let service = inspect["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|module| module["id"] == "example.session")
        .unwrap();
    assert_eq!(
        service["contract"]["interfaces"][0]["id"],
        "example.session.api"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn apply_create_instance_with_scope_writes_service() {
    let root = v2_project("apply");
    let root_arg = root.to_string_lossy().into_owned();
    let validate = json(&run(&["validate", &root_arg, "--json"]));
    let digest = validate["source_digest"].as_str().unwrap();

    let transaction = serde_json::json!({
        "request_id": "create-service",
        "base_revision": {
            "seq": 0,
            "source_digest": digest
        },
        "commands": [{
            "type": "create_instance",
            "id": "session",
            "module": "example.session",
            "version": "1.0.0",
            "config": {},
            "scope": "service"
        }]
    });
    let transaction_path = root.join("transaction.json");
    fs::write(&transaction_path, serde_json::to_vec(&transaction).unwrap()).unwrap();

    let applied = run(&["apply", &root_arg, transaction_path.to_str().unwrap()]);
    assert_eq!(
        applied.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    let response = json(&applied);
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["result"]["accepted"], true, "{response}");

    let text = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert!(text.contains("scope = \"service\""), "{text}");
    assert!(text.contains("id = \"session\""), "{text}");

    let revalidated = json(&run(&["validate", &root_arg, "--json"]));
    assert_eq!(revalidated["valid"], true);

    let _ = fs::remove_dir_all(&root);
}
