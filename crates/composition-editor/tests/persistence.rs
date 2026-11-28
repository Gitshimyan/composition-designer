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

fn copy_example(relative: &str, label: &str) -> PathBuf {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join(relative);
    let target =
        std::env::temp_dir().join(format!("composition-state-{}-{label}", std::process::id()));
    let _ = fs::remove_dir_all(&target);
    copy_dir(&source, &target);
    target
}

fn set_title(session: &ProjectSession, request_id: &str, title: &str) -> Transaction {
    Transaction::new(
        request_id,
        session.workspace().revision(),
        vec![Command::SetConfig {
            instance: LocalId::parse("app").unwrap(),
            property: LocalId::parse("title").unwrap(),
            value: ConfigValue::String(title.to_owned()),
        }],
    )
}

#[test]
fn revision_and_requests_survive_reopen() {
    let root = copy_example("minimal-project", "reopen");
    let mut session = ProjectSession::open(&root).unwrap();
    let edit1 = set_title(&session, "edit-1", "One");
    assert!(session.apply(&edit1).is_accepted());
    let edit2 = set_title(&session, "edit-2", "Two");
    assert!(session.apply(&edit2).is_accepted());
    assert_eq!(session.workspace().revision().seq, 2);
    drop(session);

    let state = root.join(".designer/state/default.json");
    assert!(state.exists(), "state file is missing");
    let text = fs::read_to_string(&state).unwrap();
    assert!(text.contains("\"version\":1"));

    let mut reopened = ProjectSession::open(&root).unwrap();
    assert_eq!(reopened.workspace().revision().seq, 2);
    assert_eq!(reopened.workspace().history().len(), 3);
    assert!(reopened.source_text().contains("Two"));

    let before = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    let replay = reopened.apply(&edit2);
    assert!(replay.is_replayed(), "{:?}", replay.diagnostics());
    let after = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert_eq!(before, after);

    assert!(reopened
        .apply(&set_title(&reopened, "edit-3", "Three"))
        .is_accepted());
    assert_eq!(reopened.workspace().revision().seq, 3);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn source_request_replay_survives_reopen() {
    let root = copy_example("minimal-project", "source-reopen");
    let mut session = ProjectSession::open(&root).unwrap();
    let before = session.source_text().to_owned();
    let after = before.replace("title = \"Minimal Project\"", "title = \"Source\"");
    let transaction = Transaction::new(
        "source-1",
        session.workspace().revision(),
        vec![Command::ReplaceSource {
            source: after.clone(),
        }],
    );
    let applied = session.apply(&transaction);
    assert!(applied.is_accepted(), "{:?}", applied.diagnostics());
    drop(session);

    let mut reopened = ProjectSession::open(&root).unwrap();
    let replay = reopened.apply(&transaction);
    assert!(replay.is_replayed(), "{:?}", replay.diagnostics());
    assert_eq!(
        replay.source_diff().map(|diff| diff.after.as_str()),
        Some(after.as_str())
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn stale_state_is_discarded() {
    let root = copy_example("minimal-project", "stale");
    let mut session = ProjectSession::open(&root).unwrap();
    assert!(session
        .apply(&set_title(&session, "edit-1", "One"))
        .is_accepted());
    drop(session);

    let path = root.join("composition/main.toml");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        text.replace("title = \"One\"", "title = \"External\""),
    )
    .unwrap();

    let reopened = ProjectSession::open(&root).unwrap();
    assert_eq!(reopened.workspace().revision().seq, 0);
    assert_eq!(reopened.workspace().history().len(), 1);
    assert!(reopened.source_text().contains("External"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn profiles_keep_separate_state() {
    let root = copy_example("v3/valid-profiles", "profiles");
    let mut desktop = ProjectSession::open_with_profile(&root, Some("desktop")).unwrap();
    assert!(desktop
        .apply(&set_title(&desktop, "desktop-1", "Desktop Edited"))
        .is_accepted());
    drop(desktop);
    assert!(root.join(".designer/state/desktop.json").exists());

    let default = ProjectSession::open(&root).unwrap();
    assert_eq!(default.workspace().revision().seq, 0);
    drop(default);

    let reopened = ProjectSession::open_with_profile(&root, Some("desktop")).unwrap();
    assert_eq!(reopened.workspace().revision().seq, 1);
    assert!(reopened.source_text().contains("Desktop Edited"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn corrupt_state_is_ignored() {
    let root = copy_example("minimal-project", "corrupt");
    let session = ProjectSession::open(&root).unwrap();
    drop(session);
    let state = root.join(".designer/state/default.json");
    fs::create_dir_all(state.parent().unwrap()).unwrap();
    fs::write(&state, "{not json").unwrap();
    let reopened = ProjectSession::open(&root).unwrap();
    assert_eq!(reopened.workspace().revision().seq, 0);
    let _ = fs::remove_dir_all(&root);
}
