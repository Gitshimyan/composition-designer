use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

fn example(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join(path)
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
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
fn validate_and_inspect_select_profiles() {
    let project = example("v3/valid-profiles");
    let project = project.to_str().unwrap();

    let default = run(&["inspect", project]);
    assert_eq!(default.status.code(), Some(0), "{:?}", default);
    let value = json(&default);
    assert!(value["diagnostics"].as_array().unwrap().is_empty());
    assert_eq!(value["project"]["active_profile"], "default");
    assert_eq!(
        value["project"]["composition_entry"],
        "composition/main.toml"
    );
    assert_eq!(value["project"]["profiles"].as_array().unwrap().len(), 2);

    let desktop = run(&["inspect", project, "--profile", "desktop"]);
    assert_eq!(desktop.status.code(), Some(0));
    let value = json(&desktop);
    assert_eq!(value["project"]["active_profile"], "desktop");
    assert_eq!(
        value["project"]["composition_entry"],
        "composition/desktop.toml"
    );
    assert!(value["diagnostics"].as_array().unwrap().is_empty());

    let context = run(&["context", project, "--profile", "headless"]);
    assert_eq!(context.status.code(), Some(0));
    let value = json(&context);
    assert_eq!(value["result"]["project"]["activeProfile"], "headless");
    assert_eq!(value["result"]["counts"]["instances"], 1);
}

#[test]
fn apply_writes_only_the_selected_profile() {
    let root = std::env::temp_dir().join(format!("composition-cli-profile-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&example("v3/valid-profiles"), &root);
    let root_text = root.to_str().unwrap();

    let resolved = run(&["resolve", root_text, "--profile", "headless"]);
    assert_eq!(resolved.status.code(), Some(0));
    let value = json(&resolved);
    let revision = value["model"]["revision"].clone();
    let transaction = json!({
        "request_id": "profile-edit",
        "base_revision": revision,
        "commands": [{
            "type": "set_config",
            "instance": "app",
            "property": "title",
            "value": "Edited Headless"
        }]
    });
    let transaction_path = root.join("transaction.json");
    fs::write(
        &transaction_path,
        serde_json::to_string(&transaction).unwrap(),
    )
    .unwrap();

    let applied = run(&[
        "apply",
        root_text,
        transaction_path.to_str().unwrap(),
        "--profile",
        "headless",
    ]);
    assert_eq!(applied.status.code(), Some(0), "{:?}", applied);
    assert_eq!(json(&applied)["result"]["accepted"], true);

    let headless = fs::read_to_string(root.join("composition/headless.toml")).unwrap();
    assert!(headless.contains("Edited Headless"), "{headless}");
    let main = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert!(main.contains("V3 Main"), "{main}");
    let desktop = fs::read_to_string(root.join("composition/desktop.toml")).unwrap();
    assert!(desktop.contains("V3 Desktop"), "{desktop}");

    let validated = run(&["validate", root_text, "--profile", "headless", "--json"]);
    assert_eq!(validated.status.code(), Some(0));
    assert_eq!(json(&validated)["valid"], true);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn profile_errors_are_reported() {
    let project = example("v3/valid-profiles");
    let missing = run(&[
        "validate",
        project.to_str().unwrap(),
        "--profile",
        "missing",
        "--json",
    ]);
    assert_eq!(missing.status.code(), Some(1));
    let value = json(&missing);
    assert!(value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["code"] == "profile-not-found"));

    let v2 = example("v2/valid-service-interface");
    let unsupported = run(&[
        "validate",
        v2.to_str().unwrap(),
        "--profile",
        "desktop",
        "--json",
    ]);
    assert_eq!(unsupported.status.code(), Some(1));
    assert!(json(&unsupported)["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["code"] == "profile-unsupported"));
}

#[test]
fn profile_flag_requires_a_value() {
    let project = example("v3/valid-profiles");
    let output = run(&["validate", project.to_str().unwrap(), "--profile"]);
    assert_eq!(output.status.code(), Some(2));
}
