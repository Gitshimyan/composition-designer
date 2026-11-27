use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use composition_core::validate_project;
use composition_domain::{ConfigValue, LocalId};
use serde_json::{json, Value};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pilot() -> PathBuf {
    workspace_root().join("working/pilots/glide-task-terminal")
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

fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_composition-cli"))
        .args(args)
        .output()
        .expect("run composition-cli")
}

fn json(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid JSON output")
}

#[test]
fn glide_pilot_full_chain_via_cli_and_core() {
    let root = std::env::temp_dir().join(format!("composition-m6-closeout-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&pilot(), &root);

    let initial = validate_project(&root);
    assert!(initial.is_valid(), "{:?}", initial.diagnostics);
    let digest = initial.source_digest.expect("digest").to_string();
    let transaction = json!({
        "request_id": "m6-closeout-1",
        "base_revision": { "seq": 0, "source_digest": digest },
        "commands": [
            {
                "type": "create_instance",
                "id": "terminal-web-2",
                "module": "glide.terminal.web",
                "version": "1.0.0",
                "config": { "protocol-version": "2" }
            },
            {
                "type": "create_instance",
                "id": "terminal-web-3",
                "module": "glide.terminal.web",
                "version": "1.0.0",
                "config": {}
            },
            {
                "type": "create_binding",
                "id": "app-runtime-terminal-web-2",
                "parent": "app",
                "slot": "runtime",
                "child": "terminal-web-2"
            },
            {
                "type": "create_binding",
                "id": "app-runtime-terminal-web-3",
                "parent": "app",
                "slot": "runtime",
                "child": "terminal-web-3"
            },
            {
                "type": "create_connection",
                "id": "terminal-web-2-to-terminal-web-3",
                "from": { "instance": "terminal-web-2", "port": "to-panel" },
                "to": { "instance": "terminal-web-3", "port": "from-panel" }
            },
            {
                "type": "set_dependency_provider",
                "instance": "task-runner",
                "requirement": "terminal-session",
                "provider": "conpty-session"
            },
            {
                "type": "set_dependency_provider",
                "instance": "terminal-panel",
                "requirement": "terminal-web",
                "provider": "terminal-web"
            }
        ]
    });
    let transaction_path = root.join("m6-closeout-transaction.json");
    let mut file = fs::File::create(&transaction_path).unwrap();
    writeln!(file, "{transaction}").unwrap();

    let applied = cli(&[
        "apply",
        &root.to_string_lossy(),
        &transaction_path.to_string_lossy(),
    ]);
    assert_eq!(
        applied.status.code(),
        Some(0),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&applied.stdout),
        String::from_utf8_lossy(&applied.stderr)
    );
    let applied_json = json(&applied);
    assert_eq!(applied_json["ok"], true);
    assert_eq!(applied_json["result"]["accepted"], true);

    let validation = validate_project(&root);
    assert!(validation.is_valid(), "{:?}", validation.diagnostics);
    let model = validation.model.expect("model");
    assert_eq!(model.composition.instances.len(), 9);
    assert_eq!(model.composition.bindings.len(), 8);
    assert_eq!(model.composition.connections.len(), 7);
    let terminal_web = model
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "terminal-web-2")
        .expect("created instance");
    assert_eq!(
        terminal_web
            .config
            .get(&LocalId::parse("protocol-version").unwrap()),
        Some(&ConfigValue::String("2".into()))
    );
    let task_runner = model
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "task-runner")
        .expect("task runner");
    assert_eq!(
        task_runner
            .dependency_providers
            .get(&LocalId::parse("terminal-session").unwrap()),
        Some(&LocalId::parse("conpty-session").unwrap())
    );

    let reopened = cli(&["validate", &root.to_string_lossy(), "--json"]);
    assert_eq!(
        reopened.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&reopened.stderr)
    );
    let reopened_json = json(&reopened);
    assert_eq!(reopened_json["valid"], true);
    assert_eq!(reopened_json["error_count"], 0);
    assert_eq!(
        reopened_json["source_digest"],
        validation.source_digest.expect("digest").to_string()
    );

    let composition = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert!(composition.contains("id = \"terminal-web-2\""));
    assert!(composition.contains("id = \"app-runtime-terminal-web-3\""));
    assert!(composition.contains("id = \"terminal-web-2-to-terminal-web-3\""));
    assert!(composition.contains("terminal-session = \"conpty-session\""));

    let _ = fs::remove_dir_all(&root);
}
