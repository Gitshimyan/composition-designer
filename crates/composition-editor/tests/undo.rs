use std::fs;
use std::path::{Path, PathBuf};

use composition_domain::{ConfigValue, ErrorCode, LocalId};
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
        std::env::temp_dir().join(format!("composition-undo-{}-{label}", std::process::id()));
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
fn undo_and_redo_restore_text_and_revisions() {
    let root = copy_example("roundtrip");
    let mut session = ProjectSession::open(&root).unwrap();
    let initial = session.source_text().to_owned();
    assert!(!session.can_undo());
    assert!(!session.can_redo());

    let first = session.apply(&set_title(&session, "edit-1", "One"));
    assert!(first.is_accepted(), "{:?}", first.diagnostics());
    let one = session.source_text().to_owned();
    assert_ne!(one, initial);
    assert!(session.can_undo());

    let second = session.apply(&set_title(&session, "edit-2", "Two"));
    assert!(second.is_accepted(), "{:?}", second.diagnostics());
    let two = session.source_text().to_owned();
    assert_ne!(two, one);
    let revision_before_undo = session.workspace().revision().seq;

    let undo = session.undo();
    assert!(undo.is_accepted(), "{:?}", undo.diagnostics());
    assert_eq!(session.source_text(), one.as_str());
    assert_eq!(session.workspace().revision().seq, revision_before_undo + 1);
    assert!(session.can_redo());

    let undo = session.undo();
    assert!(undo.is_accepted(), "{:?}", undo.diagnostics());
    assert_eq!(session.source_text(), initial.as_str());
    assert!(!session.can_undo());
    assert!(session.can_redo());

    let redo = session.redo();
    assert!(redo.is_accepted(), "{:?}", redo.diagnostics());
    assert_eq!(session.source_text(), one.as_str());

    let redo = session.redo();
    assert!(redo.is_accepted(), "{:?}", redo.diagnostics());
    assert_eq!(session.source_text(), two.as_str());
    assert!(!session.can_redo());

    let disk = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert_eq!(disk.as_str(), session.source_text());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_new_edit_clears_redo() {
    let root = copy_example("clear-redo");
    let mut session = ProjectSession::open(&root).unwrap();
    assert!(session
        .apply(&set_title(&session, "edit-1", "One"))
        .is_accepted());
    assert!(session.undo().is_accepted());
    assert!(session.can_redo());

    let edit = session.apply(&set_title(&session, "edit-3", "Three"));
    assert!(edit.is_accepted(), "{:?}", edit.diagnostics());
    assert!(!session.can_redo());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn undo_without_history_is_rejected() {
    let root = copy_example("empty");
    let mut session = ProjectSession::open(&root).unwrap();
    let outcome = session.undo();
    assert!(!outcome.is_accepted());
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == ErrorCode::EditInvalidCommand));
    let _ = fs::remove_dir_all(&root);
}
