use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use composition_core::{validate_project, ValidationResult};
use composition_domain::{CompositionModel, Diagnostic, ErrorCode, LocalId, ObjectId, Severity};
use composition_editor::{ModelDiff, ProjectSession, Transaction};
use serde::{Deserialize, Serialize};
use tauri::State;
use toml_edit::{value, DocumentMut, Item, Table};

type SessionState = Mutex<Option<ProjectSession>>;

/// Read-only/editable snapshot returned to the React shell. The model is the
/// canonical `CompositionModel` produced by the Rust Core; the UI never parses
/// project files itself.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub root: String,
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub source_digest: Option<String>,
    pub model: Option<CompositionModel>,
    pub diagnostics: Vec<Diagnostic>,
    pub can_edit: bool,
}

/// Presentation-only canvas position. `.designer/**` is never read by Core.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditResponse {
    pub accepted: bool,
    pub replayed: bool,
    pub diff: Option<ModelDiff>,
    pub diagnostics: Vec<Diagnostic>,
    pub snapshot: ProjectSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalChangeState {
    pub has_session: bool,
    pub dirty: bool,
    pub session_digest: Option<String>,
    pub disk_digest: Option<String>,
    pub disk_valid: bool,
    pub disk_error_count: usize,
}

#[derive(Serialize)]
pub struct SourcePayload {
    pub entry: String,
    pub text: String,
}

fn counts(diagnostics: &[Diagnostic]) -> (usize, usize) {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_error())
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Warning)
        .count();
    (errors, warnings)
}

fn snapshot_from_model(root: String, model: &CompositionModel, can_edit: bool) -> ProjectSnapshot {
    let diagnostics = model.diagnostics.clone();
    let (error_count, warning_count) = counts(&diagnostics);
    let valid = error_count == 0;
    let source_digest = model
        .revision
        .as_ref()
        .and_then(|revision| revision.source_digest.as_ref())
        .map(|digest| digest.to_string());
    ProjectSnapshot {
        root,
        valid,
        error_count,
        warning_count,
        source_digest,
        model: Some(model.clone()),
        diagnostics,
        can_edit,
    }
}

fn snapshot_from_validation(
    root: &str,
    validation: &ValidationResult,
    can_edit: bool,
) -> ProjectSnapshot {
    let (error_count, warning_count) = counts(&validation.diagnostics);
    ProjectSnapshot {
        root: root.to_owned(),
        valid: validation.is_valid(),
        error_count,
        warning_count,
        source_digest: validation
            .source_digest
            .as_ref()
            .map(|digest| digest.to_string()),
        model: validation.model.clone(),
        diagnostics: validation.diagnostics.clone(),
        can_edit,
    }
}

#[cfg(test)]
fn snapshot(path: &str) -> ProjectSnapshot {
    snapshot_from_validation(path, &validate_project(path), false)
}

fn snapshot_from_session(session: &ProjectSession) -> ProjectSnapshot {
    snapshot_from_model(
        session.root().to_string_lossy().into_owned(),
        session.workspace().model(),
        true,
    )
}

fn external_change_state(session: &ProjectSession) -> ExternalChangeState {
    let disk = validate_project(session.root());
    let session_digest = session.workspace().revision().source_digest;
    let disk_digest = disk.source_digest.clone();
    let dirty = session_digest != disk_digest;
    ExternalChangeState {
        has_session: true,
        dirty,
        session_digest: session_digest.map(|digest| digest.to_string()),
        disk_digest: disk_digest.map(|digest| digest.to_string()),
        disk_valid: disk.is_valid(),
        disk_error_count: disk.error_count(),
    }
}

fn external_dirty(session: &ProjectSession) -> bool {
    session.workspace().revision().source_digest != validate_project(session.root()).source_digest
}

fn conflict_response(session: &ProjectSession) -> EditResponse {
    EditResponse {
        accepted: false,
        replayed: false,
        diff: None,
        diagnostics: vec![Diagnostic::new(
            ErrorCode::EditRevisionConflict,
            "project changed on disk; reload before editing",
        )
        .with_object_id(ObjectId::composition())],
        snapshot: snapshot_from_session(session),
    }
}

fn edit_response(
    session: &ProjectSession,
    outcome: composition_editor::TransactionOutcome,
) -> EditResponse {
    let accepted = outcome.is_accepted();
    let replayed = outcome.is_replayed();
    let diff = outcome.diff().cloned();
    let diagnostics = outcome.diagnostics().to_vec();
    EditResponse {
        accepted,
        replayed,
        diff,
        diagnostics,
        snapshot: snapshot_from_session(session),
    }
}

#[tauri::command]
fn open_project(path: String, state: State<'_, SessionState>) -> Result<ProjectSnapshot, String> {
    let validation = validate_project(&path);
    let session = ProjectSession::open(&path).ok();
    let can_edit = session.is_some();
    let response = match &session {
        Some(session) => snapshot_from_session(session),
        None => snapshot_from_validation(&path, &validation, can_edit),
    };
    let mut guard = state
        .lock()
        .map_err(|_| "project state lock poisoned".to_owned())?;
    *guard = session;
    Ok(response)
}

#[tauri::command]
fn apply_transaction(
    transaction: Transaction,
    state: State<'_, SessionState>,
) -> Result<EditResponse, String> {
    let mut guard = state
        .lock()
        .map_err(|_| "project state lock poisoned".to_owned())?;
    let Some(session) = guard.as_mut() else {
        return Err("project is not editable".to_owned());
    };
    if external_dirty(session) {
        return Ok(conflict_response(session));
    }
    let outcome = session.apply(&transaction);
    Ok(edit_response(session, outcome))
}

#[tauri::command]
fn prepare_transaction(
    transaction: Transaction,
    state: State<'_, SessionState>,
) -> Result<EditResponse, String> {
    let guard = state
        .lock()
        .map_err(|_| "project state lock poisoned".to_owned())?;
    let Some(session) = guard.as_ref() else {
        return Err("project is not editable".to_owned());
    };
    if external_dirty(session) {
        return Ok(conflict_response(session));
    }
    let outcome = session.prepare(&transaction);
    Ok(edit_response(session, outcome))
}

#[tauri::command]
fn read_source(state: State<'_, SessionState>) -> Result<SourcePayload, String> {
    let guard = state
        .lock()
        .map_err(|_| "project state lock poisoned".to_owned())?;
    let Some(session) = guard.as_ref() else {
        return Err("project is not editable".to_owned());
    };
    Ok(SourcePayload {
        entry: session.composition_entry().to_owned(),
        text: session.source_text().to_owned(),
    })
}

#[tauri::command]
fn check_external_changes(state: State<'_, SessionState>) -> Result<ExternalChangeState, String> {
    let guard = state
        .lock()
        .map_err(|_| "project state lock poisoned".to_owned())?;
    let Some(session) = guard.as_ref() else {
        return Ok(ExternalChangeState {
            has_session: false,
            dirty: false,
            session_digest: None,
            disk_digest: None,
            disk_valid: false,
            disk_error_count: 0,
        });
    };
    Ok(external_change_state(session))
}

#[tauri::command]
fn reload_project(state: State<'_, SessionState>) -> Result<ProjectSnapshot, String> {
    let mut guard = state
        .lock()
        .map_err(|_| "project state lock poisoned".to_owned())?;
    let Some(session) = guard.as_ref() else {
        return Err("no open project to reload".to_owned());
    };
    let root = session.root().to_path_buf();
    let validation = validate_project(&root);
    let new_session = ProjectSession::open(&root).ok();
    let can_edit = new_session.is_some();
    let response = match &new_session {
        Some(session) => snapshot_from_session(session),
        None => snapshot_from_validation(&root.to_string_lossy(), &validation, can_edit),
    };
    *guard = new_session;
    Ok(response)
}

/// Optional startup project for automation:
/// `composition-designer --project <path>` or `--project=<path>`.
#[tauri::command]
fn initial_project() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--project" {
            return args.next();
        }
        if let Some(path) = arg.strip_prefix("--project=") {
            return Some(path.to_owned());
        }
    }
    None
}

fn layout_path(root: &str) -> PathBuf {
    Path::new(root).join(".designer").join("layout.toml")
}

fn number(item: Option<&Item>) -> Option<f64> {
    item.and_then(|item| {
        item.as_float()
            .or_else(|| item.as_integer().map(|value| value as f64))
    })
}

#[tauri::command]
fn load_layout(root: String) -> Result<BTreeMap<String, NodePosition>, String> {
    let path = layout_path(&root);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let document = text
        .parse::<DocumentMut>()
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
    let Some(nodes) = document.get("nodes").and_then(Item::as_table) else {
        return Ok(BTreeMap::new());
    };
    let mut positions = BTreeMap::new();
    for (id, item) in nodes.iter() {
        if LocalId::parse(id).is_err() {
            continue;
        }
        let Some(table) = item.as_table() else {
            continue;
        };
        let Some(x) = number(table.get("x")) else {
            continue;
        };
        let Some(y) = number(table.get("y")) else {
            continue;
        };
        positions.insert(id.to_owned(), NodePosition { x, y });
    }
    Ok(positions)
}

#[tauri::command]
fn save_layout(root: String, nodes: BTreeMap<String, NodePosition>) -> Result<(), String> {
    if !Path::new(&root).is_dir() {
        return Err(format!("{root} is not a project directory"));
    }
    let path = layout_path(&root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let mut table = Table::new();
    for (id, position) in &nodes {
        if LocalId::parse(id).is_err() {
            continue;
        }
        let mut node = Table::new();
        node.insert("x", value(position.x));
        node.insert("y", value(position.y));
        table.insert(id, Item::Table(node));
    }
    let mut document = DocumentMut::new();
    document.insert("nodes", Item::Table(table));
    fs::write(&path, document.to_string())
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

pub fn run() {
    tauri::Builder::default()
        .manage(SessionState::default())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_position(tauri::PhysicalPosition::new(24, 24));
                if std::env::args().any(|arg| arg == "--topmost") {
                    let _ = window.set_always_on_top(true);
                }
                let _ = window.set_focus();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_project,
            initial_project,
            apply_transaction,
            prepare_transaction,
            read_source,
            check_external_changes,
            reload_project,
            load_layout,
            save_layout
        ])
        .run(tauri::generate_context!())
        .expect("error while running composition designer");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn example(name: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../examples")
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn minimal_project_snapshot_is_valid() {
        let snapshot = snapshot(&example("minimal-project"));
        assert!(snapshot.valid);
        assert_eq!(snapshot.error_count, 0);
        let model = snapshot.model.expect("model");
        assert_eq!(model.project.id.as_str(), "example.minimal");
        assert_eq!(model.composition.instances.len(), 2);
        assert_eq!(model.modules.len(), 2);
    }

    #[test]
    fn invalid_project_snapshot_keeps_diagnostics() {
        let snapshot = snapshot(&example("invalid-projects/files/entry-missing"));
        assert!(!snapshot.valid);
        assert!(snapshot.error_count > 0);
        assert!(snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.code.as_str() == "file-project-missing"
                || diagnostic.code.to_string() == "file-project-missing"
        }));
    }

    #[test]
    fn missing_layout_is_empty() {
        let root =
            std::env::temp_dir().join(format!("composition-layout-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let loaded = load_layout(root.to_string_lossy().into_owned()).unwrap();
        assert!(loaded.is_empty());
        let _ = fs::remove_dir_all(&root);
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
    fn external_change_is_detected_and_reloaded() {
        let source = PathBuf::from(example("minimal-project"));
        let root =
            std::env::temp_dir().join(format!("composition-external-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_dir(&source, &root);
        let session = ProjectSession::open(&root).unwrap();
        let clean = external_change_state(&session);
        assert!(!clean.dirty);

        let composition = root.join("composition/main.toml");
        let mut text = fs::read_to_string(&composition).unwrap();
        text.insert_str(0, "# changed outside the designer\n");
        fs::write(&composition, text).unwrap();
        let changed = external_change_state(&session);
        assert!(changed.dirty);
        assert!(changed.disk_valid);

        let reloaded = ProjectSession::open(&root).unwrap();
        let clean_again = external_change_state(&reloaded);
        assert!(!clean_again.dirty);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn layout_round_trips() {
        let root =
            std::env::temp_dir().join(format!("composition-layout-round-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut nodes = BTreeMap::new();
        nodes.insert("app".to_owned(), NodePosition { x: 12.5, y: -40.0 });
        nodes.insert("hello".to_owned(), NodePosition { x: 240.0, y: 180.0 });
        let root_string = root.to_string_lossy().into_owned();
        save_layout(root_string.clone(), nodes.clone()).unwrap();
        assert!(root.join(".designer/layout.toml").exists());
        assert_eq!(load_layout(root_string).unwrap(), nodes);
        let _ = fs::remove_dir_all(&root);
    }

    use composition_domain::{ConfigValue, ModuleId, Version};
    use composition_editor::Command;

    fn local(value: &str) -> LocalId {
        LocalId::parse(value).unwrap()
    }

    fn module(value: &str) -> ModuleId {
        ModuleId::parse(value).unwrap()
    }

    fn version(value: &str) -> Version {
        Version::parse(value).unwrap()
    }

    fn example_copy(name: &str, prefix: &str) -> PathBuf {
        let source = PathBuf::from(example(name));
        let root = std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        copy_dir(&source, &root);
        root
    }

    #[test]
    fn instance_create_transaction_writes_instance_and_binding() {
        let root = example_copy("nested-composition", "composition-instance-create");
        let mut session = ProjectSession::open(&root).unwrap();
        let mut config = BTreeMap::new();
        config.insert(local("label"), ConfigValue::String("Status 2".into()));
        let transaction = Transaction::new(
            "create-status-2",
            session.workspace().revision(),
            vec![
                Command::CreateInstance {
                    id: local("status-2"),
                    module: module("leaf.widget"),
                    version: version("1.0.0"),
                    config,
                },
                Command::CreateBinding {
                    id: local("panel-1-status-2"),
                    parent: local("panel-1"),
                    slot: local("body"),
                    child: local("status-2"),
                },
            ],
        );
        let outcome = session.apply(&transaction);
        assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());

        let validation = validate_project(&root);
        assert!(validation.is_valid(), "{:?}", validation.diagnostics);
        let model = validation.model.expect("model");
        assert_eq!(model.composition.instances.len(), 9);
        assert_eq!(model.composition.bindings.len(), 8);

        let reopened = ProjectSession::open(&root).unwrap();
        let instance = reopened
            .workspace()
            .model()
            .composition
            .instances
            .iter()
            .find(|instance| instance.id.as_str() == "status-2")
            .expect("persisted instance");
        assert_eq!(
            instance.config.get(&local("label")),
            Some(&ConfigValue::String("Status 2".into()))
        );
        let text = fs::read_to_string(root.join("composition/main.toml")).unwrap();
        assert!(text.contains("id = \"status-2\""));
        assert!(text.contains("child = \"status-2\""));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn instance_delete_transaction_cascades_relations() {
        let root = example_copy("nested-composition", "composition-instance-delete");
        let mut session = ProjectSession::open(&root).unwrap();
        let transaction = Transaction::new(
            "delete-hello",
            session.workspace().revision(),
            vec![Command::DeleteInstance {
                id: local("hello"),
            }],
        );
        let outcome = session.apply(&transaction);
        assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());

        let validation = validate_project(&root);
        assert!(validation.is_valid(), "{:?}", validation.diagnostics);
        let model = validation.model.expect("model");
        assert_eq!(model.composition.instances.len(), 7);
        assert_eq!(model.composition.bindings.len(), 6);
        assert!(model
            .composition
            .instances
            .iter()
            .all(|instance| instance.id.as_str() != "hello"));
        assert!(model
            .composition
            .bindings
            .iter()
            .all(|binding| binding.id.as_str() != "panel-1-hello"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn instance_delete_rejection_leaves_files_untouched() {
        let root = example_copy("minimal-project", "composition-instance-reject");
        let mut session = ProjectSession::open(&root).unwrap();
        let path = root.join("composition/main.toml");
        let before = fs::read_to_string(&path).unwrap();
        let transaction = Transaction::new(
            "delete-required-child",
            session.workspace().revision(),
            vec![Command::DeleteInstance {
                id: local("hello"),
            }],
        );
        let outcome = session.apply(&transaction);
        assert!(!outcome.is_accepted());
        assert!(outcome
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "structure-slot-cardinality"));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert_eq!(session.workspace().model().composition.instances.len(), 2);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn instance_rename_rewrites_root_and_references() {
        let root = example_copy("minimal-project", "composition-instance-rename");
        let mut session = ProjectSession::open(&root).unwrap();
        let transaction = Transaction::new(
            "rename-root",
            session.workspace().revision(),
            vec![Command::RenameInstance {
                from: local("app"),
                to: local("shell"),
            }],
        );
        let outcome = session.apply(&transaction);
        assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());

        let validation = validate_project(&root);
        assert!(validation.is_valid(), "{:?}", validation.diagnostics);
        let model = validation.model.expect("model");
        assert_eq!(model.composition.root.as_str(), "shell");
        assert_eq!(model.composition.bindings[0].parent.as_str(), "shell");
        let text = fs::read_to_string(root.join("composition/main.toml")).unwrap();
        assert!(text.contains("root = \"shell\""));
        assert!(text.contains("parent = \"shell\""));
        let _ = fs::remove_dir_all(&root);
    }
}
