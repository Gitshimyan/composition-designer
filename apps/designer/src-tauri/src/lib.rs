use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use composition_core::{validate_project, ValidationResult};
use composition_domain::{CompositionModel, Diagnostic, LocalId, Severity};
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
}
