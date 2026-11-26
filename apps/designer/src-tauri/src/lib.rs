use composition_core::validate_project;
use composition_domain::{CompositionModel, Diagnostic, Severity};
use serde::Serialize;

/// Read-only snapshot returned to the React shell. The model is the same
/// canonical `CompositionModel` produced by the Rust Core; the UI never
/// parses project files itself.
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
}

fn snapshot(path: &str) -> ProjectSnapshot {
    let validation = validate_project(path);
    let error_count = validation.error_count();
    let warning_count = validation
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Warning)
        .count();
    ProjectSnapshot {
        root: path.to_owned(),
        valid: validation.is_valid(),
        error_count,
        warning_count,
        source_digest: validation.source_digest.map(|digest| digest.to_string()),
        model: validation.model,
        diagnostics: validation.diagnostics,
    }
}

#[tauri::command]
fn open_project(path: String) -> ProjectSnapshot {
    snapshot(&path)
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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_position(tauri::PhysicalPosition::new(24, 24));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![open_project, initial_project])
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
}
