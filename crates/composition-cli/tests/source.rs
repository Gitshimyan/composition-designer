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
fn cli_applies_replace_source_transaction() {
    let root = std::env::temp_dir().join(format!("composition-cli-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&workspace_root().join("examples/minimal-project"), &root);

    let before = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    let after = before.replace("title = \"Minimal Project\"", "title = \"CLI source\"");
    let digest = validate_project(&root).source_digest.expect("digest");
    let transaction = serde_json::json!({
        "request_id": "cli-source-1",
        "base_revision": { "seq": 0, "source_digest": digest.to_string() },
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
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["result"]["accepted"], true);
    assert_eq!(
        response["result"]["source_diff"]["after"].as_str().unwrap(),
        transaction["commands"][0]["source"].as_str().unwrap()
    );
    assert_eq!(
        fs::read_to_string(root.join("composition/main.toml")).unwrap(),
        transaction["commands"][0]["source"].as_str().unwrap()
    );
    let _ = fs::remove_dir_all(&root);
}
