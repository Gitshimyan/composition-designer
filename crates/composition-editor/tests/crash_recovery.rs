use std::fs;
use std::path::{Path, PathBuf};

use composition_domain::{ConfigValue, LocalId};
use composition_editor::{Command, ProjectSession, Transaction};

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

fn copy_example(label: &str) -> PathBuf {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/minimal-project");
    let target =
        std::env::temp_dir().join(format!("composition-crash-{}-{label}", std::process::id()));
    let _ = fs::remove_dir_all(&target);
    copy_dir(&source, &target);
    target
}

fn set_title(session: &ProjectSession, title: &str) -> Transaction {
    Transaction::new(
        "crash-edit",
        session.workspace().revision(),
        vec![Command::SetConfig {
            instance: LocalId::parse("app").unwrap(),
            property: LocalId::parse("title").unwrap(),
            value: ConfigValue::String(title.to_owned()),
        }],
    )
}

#[test]
fn backup_is_restored_when_the_entry_is_missing() {
    let root = copy_example("restore-backup");
    let composition = root.join("composition/main.toml");
    let original = fs::read_to_string(&composition).unwrap();
    let backup = composition.with_file_name(".main.toml.bak-1");
    fs::rename(&composition, &backup).unwrap();
    assert!(!composition.exists());

    let session = ProjectSession::open(&root).unwrap();
    assert_eq!(session.source_text(), original.as_str());
    assert!(composition.exists());
    assert!(!backup.exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn an_existing_entry_wins_over_a_leftover_backup() {
    let root = copy_example("new-wins");
    let composition = root.join("composition/main.toml");
    let original = fs::read_to_string(&composition).unwrap();
    let backup = composition.with_file_name(".main.toml.bak-1");
    fs::write(&backup, &original).unwrap();

    let committed = original.replace("title = \"Minimal Project\"", "title = \"Committed\"");
    fs::write(&composition, &committed).unwrap();
    let session = ProjectSession::open(&root).unwrap();
    assert!(session.source_text().contains("Committed"));
    assert!(
        backup.exists(),
        "an existing entry must not consume the backup"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn orphan_temp_file_is_ignored() {
    let root = copy_example("orphan-temp");
    let composition = root.join("composition/main.toml");
    let temp = composition.with_file_name(".main.toml.tmp-999");
    fs::write(&temp, "this is not toml").unwrap();

    let session = ProjectSession::open(&root).unwrap();
    assert!(session.source_text().contains("Minimal Project"));
    assert!(temp.exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_successful_commit_removes_temp_and_backup_files() {
    let root = copy_example("clean-commit");
    let mut session = ProjectSession::open(&root).unwrap();
    let outcome = session.apply(&set_title(&session, "Committed"));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());

    let names: Vec<String> = fs::read_dir(root.join("composition"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !names
            .iter()
            .any(|name| name.contains(".tmp-") || name.contains(".bak-")),
        "{names:?}"
    );
    assert!(session.source_text().contains("Committed"));
    let _ = fs::remove_dir_all(&root);
}
