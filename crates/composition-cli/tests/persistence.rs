use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

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
fn request_replay_survives_a_new_cli_process() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/minimal-project");
    let root = std::env::temp_dir().join(format!("composition-cli-state-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&source, &root);
    let root_text = root.to_str().unwrap();

    let resolved = run(&["resolve", root_text]);
    assert_eq!(resolved.status.code(), Some(0));
    let revision = json(&resolved)["model"]["revision"].clone();
    let transaction = json!({
        "request_id": "persisted-request",
        "base_revision": revision,
        "commands": [{
            "type": "set_config",
            "instance": "app",
            "property": "title",
            "value": "Persisted"
        }]
    });
    let path = root.join("transaction.json");
    fs::write(&path, serde_json::to_string(&transaction).unwrap()).unwrap();

    let first = run(&["apply", root_text, path.to_str().unwrap()]);
    assert_eq!(first.status.code(), Some(0), "{:?}", first);
    let first = json(&first);
    assert_eq!(first["result"]["accepted"], true);
    assert_eq!(first["result"]["replayed"], false);
    let after_first = fs::read_to_string(root.join("composition/main.toml")).unwrap();

    let second = run(&["apply", root_text, path.to_str().unwrap()]);
    assert_eq!(second.status.code(), Some(0), "{:?}", second);
    let second = json(&second);
    assert_eq!(second["result"]["accepted"], true);
    assert_eq!(second["result"]["replayed"], true);
    let after_second = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert_eq!(after_first, after_second);
    assert!(root.join(".designer/state/default.json").exists());
    let _ = fs::remove_dir_all(&root);
}
