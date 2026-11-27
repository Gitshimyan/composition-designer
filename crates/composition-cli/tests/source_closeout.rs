use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use composition_core::validate_project;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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
fn glide_pilot_source_edit_via_cli() {
    let root = std::env::temp_dir().join(format!(
        "composition-source-closeout-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    copy_dir(
        &workspace_root().join("working/pilots/glide-task-terminal"),
        &root,
    );

    let initial = validate_project(&root);
    assert!(initial.is_valid(), "{:?}", initial.diagnostics);
    let initial_digest = initial.source_digest.clone().expect("digest");
    let before = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    let after = before.replace("title = \"Glide\"", "title = \"Glide Source\"");
    let transaction = serde_json::json!({
        "request_id": "source-closeout-1",
        "base_revision": {
            "seq": 0,
            "source_digest": initial_digest.to_string()
        },
        "commands": [{ "type": "replace_source", "source": after }]
    });
    let transaction_path = root.join("source-transaction.json");
    fs::write(
        &transaction_path,
        serde_json::to_string(&transaction).unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_composition-cli"))
        .args([
            "apply",
            &root.to_string_lossy(),
            &transaction_path.to_string_lossy(),
        ])
        .output()
        .expect("run composition-cli");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let reopened = validate_project(&root);
    assert!(reopened.is_valid(), "{:?}", reopened.diagnostics);
    assert!(reopened.source_digest.is_some());
    assert_ne!(reopened.source_digest, Some(initial_digest));
    let model = reopened.model.as_ref().expect("model");
    assert_eq!(model.composition.instances.len(), 7);
    assert_eq!(model.composition.bindings.len(), 6);
    assert_eq!(model.composition.connections.len(), 6);
    let text = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert!(text.contains("title = \"Glide Source\""));
    assert!(!text.contains("title = \"Glide\"\n"));
    let _ = fs::remove_dir_all(&root);
}
