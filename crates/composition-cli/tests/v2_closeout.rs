use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pilot() -> PathBuf {
    workspace_root().join("working/pilots/glide-task-terminal-v2")
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

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target_path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target_path);
        } else {
            fs::copy(entry.path(), target_path).unwrap();
        }
    }
}

#[test]
fn glide_v2_pilot_validates_with_services_and_interfaces() {
    let root = pilot();
    let root_arg = root.to_string_lossy().into_owned();

    let validated = run(&["validate", &root_arg, "--json"]);
    assert_eq!(validated.status.code(), Some(0));
    let validate = json(&validated);
    assert_eq!(validate["valid"], true);
    assert_eq!(validate["warning_count"], 0);
    assert_eq!(validate["error_count"], 0);

    let resolved = run(&["resolve", &root_arg, "--json"]);
    assert_eq!(resolved.status.code(), Some(0));
    let resolve = json(&resolved);
    let model = &resolve["model"];
    assert_eq!(
        model["composition"]["instances"].as_array().unwrap().len(),
        7
    );
    assert_eq!(
        model["composition"]["bindings"].as_array().unwrap().len(),
        3
    );
    assert_eq!(
        model["composition"]["connections"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(model["dependencies"].as_array().unwrap().len(), 3);
    assert_eq!(model["modules"].as_array().unwrap().len(), 7);
    assert!(model["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .all(|dependency| dependency["status"] == "resolved"));

    let services: Vec<&str> = model["composition"]["instances"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|instance| instance["scope"] == "service")
        .map(|instance| instance["id"].as_str().unwrap())
        .collect();
    assert_eq!(services, ["conpty-session", "task-runner", "terminal-web"]);
}

#[test]
fn glide_v2_pilot_accepts_a_second_service_with_explicit_provider() {
    let root = std::env::temp_dir().join(format!("composition-v2-closeout-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&pilot(), &root);
    let root_arg = root.to_string_lossy().into_owned();

    let validate = json(&run(&["validate", &root_arg, "--json"]));
    let digest = validate["source_digest"].as_str().unwrap();
    let transaction = serde_json::json!({
        "request_id": "v2-closeout-service",
        "base_revision": { "seq": 0, "source_digest": digest },
        "commands": [
            {
                "type": "create_instance",
                "id": "session-2",
                "module": "glide.platform.conpty-session",
                "version": "1.0.0",
                "config": {},
                "scope": "service"
            },
            {
                "type": "set_dependency_provider",
                "instance": "task-runner",
                "requirement": "terminal-session",
                "provider": "session-2"
            }
        ]
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
    assert!(text.contains("id = \"session-2\""), "{text}");
    assert!(text.contains("scope = \"service\""), "{text}");
    assert!(text.contains("terminal-session = \"session-2\""), "{text}");

    let revalidated = json(&run(&["validate", &root_arg, "--json"]));
    assert_eq!(revalidated["valid"], true);
    let resolved = json(&run(&["resolve", &root_arg, "--json"]));
    assert_eq!(
        resolved["model"]["composition"]["instances"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert!(resolved["model"]["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .all(|dependency| dependency["status"] == "resolved"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cli_rejects_v1_v2_migration_paths() {
    let examples = workspace_root().join("examples/v2");
    for (name, code) in [
        ("invalid-version-mismatch", "schema-version-mismatch"),
        ("invalid-v1-project-v2-module", "schema-unsupported-version"),
    ] {
        let path = examples.join(name);
        let output = run(&["validate", &path.to_string_lossy(), "--json"]);
        assert_eq!(output.status.code(), Some(1), "{name}");
        let value = json(&output);
        assert_eq!(value["valid"], false, "{name}");
        let codes: Vec<&str> = value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|diagnostic| diagnostic["code"].as_str().unwrap())
            .collect();
        assert!(codes.contains(&code), "{name}: {codes:?}");
    }
}
