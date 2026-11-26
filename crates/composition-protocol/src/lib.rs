use std::path::{Path, PathBuf};

use composition_core::validate_project;
use composition_domain::{
    CompositionModel, Diagnostic, ErrorCode, ModuleDefinition, ObjectId, Project, Severity,
};
use composition_editor::{ModelDiff, ProjectSession, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// One protocol request. `op` selects a query/command; `params` is op-specific.
#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub id: String,
    pub op: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
}

#[derive(Debug, Serialize)]
pub struct Inspection {
    pub project: Option<Project>,
    pub modules: Vec<ModuleDefinition>,
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub source_digest: Option<String>,
    pub can_edit: bool,
}

#[derive(Debug, Serialize)]
pub struct ValidationView {
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub source_digest: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransactionView {
    pub accepted: bool,
    pub replayed: bool,
    pub diff: Option<ModelDiff>,
}

#[derive(Debug, Deserialize)]
struct TransactionParams {
    transaction: Transaction,
}

pub struct ProtocolSession {
    root: PathBuf,
    session: Option<ProjectSession>,
}

impl ProtocolSession {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let session = ProjectSession::open(&root).ok();
        Self { root, session }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn can_edit(&self) -> bool {
        self.session.is_some()
    }

    pub fn dispatch(&mut self, request: Request) -> Response {
        match request.op.as_str() {
            "inspect" => self.inspect(request.id),
            "resolve" => self.resolve(request.id),
            "validate" => self.validate(request.id),
            "source" => self.source(request.id),
            "prepare_transaction" => self.transaction(request.id, request.params, false),
            "apply_transaction" => self.transaction(request.id, request.params, true),
            other => error(
                request.id,
                "protocol-unknown-operation",
                format!("unknown operation {other:?}"),
            ),
        }
    }

    fn current(
        &self,
    ) -> (
        Option<CompositionModel>,
        Vec<Diagnostic>,
        Option<String>,
        bool,
    ) {
        if let Some(session) = &self.session {
            let model = session.workspace().model();
            let diagnostics = model.diagnostics.clone();
            let digest = model
                .revision
                .as_ref()
                .and_then(|revision| revision.source_digest.as_ref())
                .map(|digest| digest.to_string());
            return (Some(model.clone()), diagnostics, digest, true);
        }
        let validation = validate_project(&self.root);
        let digest = validation
            .source_digest
            .as_ref()
            .map(|digest| digest.to_string());
        (
            validation.model,
            validation.diagnostics,
            digest,
            self.session.is_some(),
        )
    }

    fn inspect(&self, id: String) -> Response {
        let (model, diagnostics, digest, can_edit) = self.current();
        let inspection = Inspection {
            project: model.as_ref().map(|model| model.project.clone()),
            modules: model
                .as_ref()
                .map(|model| model.modules.clone())
                .unwrap_or_default(),
            valid: !diagnostics.iter().any(Diagnostic::is_error),
            error_count: diagnostics.iter().filter(|d| d.is_error()).count(),
            warning_count: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count(),
            source_digest: digest,
            can_edit,
        };
        success(id, serde_json::to_value(inspection).unwrap(), diagnostics)
    }

    fn resolve(&self, id: String) -> Response {
        let (model, diagnostics, _, _) = self.current();
        success(id, json!({ "model": model }), diagnostics)
    }

    fn validate(&self, id: String) -> Response {
        let (_, diagnostics, digest, _) = self.current();
        let view = ValidationView {
            valid: !diagnostics.iter().any(Diagnostic::is_error),
            error_count: diagnostics.iter().filter(|d| d.is_error()).count(),
            warning_count: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count(),
            source_digest: digest,
        };
        success(id, serde_json::to_value(view).unwrap(), diagnostics)
    }

    fn source(&self, id: String) -> Response {
        let Some(session) = &self.session else {
            return error(
                id,
                "project-not-editable",
                "source is only available for a valid project session",
            );
        };
        success(
            id,
            json!({
                "entry": session.composition_entry(),
                "text": session.source_text(),
            }),
            Vec::new(),
        )
    }

    fn transaction(&mut self, id: String, params: Value, apply: bool) -> Response {
        let Some(session) = self.session.as_mut() else {
            return error(
                id,
                "project-not-editable",
                "project could not be opened as an editable session",
            );
        };
        let params: TransactionParams = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(parse_error) => {
                return error(
                    id,
                    "protocol-invalid-params",
                    format!("invalid transaction params: {parse_error}"),
                )
            }
        };
        if external_dirty(session) {
            let diagnostic = Diagnostic::new(
                ErrorCode::EditRevisionConflict,
                "project changed on disk; reload the session before applying",
            )
            .with_object_id(ObjectId::composition());
            return Response {
                id,
                ok: false,
                result: None,
                diagnostics: vec![diagnostic],
                error: None,
            };
        }
        let outcome = if apply {
            session.apply(&params.transaction)
        } else {
            session.prepare(&params.transaction)
        };
        let accepted = outcome.is_accepted();
        let view = TransactionView {
            accepted,
            replayed: outcome.is_replayed(),
            diff: outcome.diff().cloned(),
        };
        if accepted {
            success(
                id,
                serde_json::to_value(view).unwrap(),
                outcome.diagnostics().to_vec(),
            )
        } else {
            Response {
                id,
                ok: false,
                result: Some(serde_json::to_value(view).unwrap()),
                diagnostics: outcome.diagnostics().to_vec(),
                error: None,
            }
        }
    }
}

fn external_dirty(session: &ProjectSession) -> bool {
    session.workspace().revision().source_digest != validate_project(session.root()).source_digest
}

fn success(id: String, result: Value, diagnostics: Vec<Diagnostic>) -> Response {
    Response {
        id,
        ok: true,
        result: Some(result),
        diagnostics,
        error: None,
    }
}

fn error(id: String, code: &str, message: impl Into<String>) -> Response {
    Response {
        id,
        ok: false,
        result: None,
        diagnostics: Vec::new(),
        error: Some(ProtocolError {
            code: code.to_owned(),
            message: message.into(),
        }),
    }
}

/// Build a protocol-level error response for malformed input before dispatch.
pub fn error_response(id: impl Into<String>, code: &str, message: impl Into<String>) -> Response {
    error(id.into(), code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn example(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join(name)
    }

    fn request(op: &str, params: Value) -> Request {
        Request {
            id: "test".into(),
            op: op.into(),
            params,
        }
    }

    #[test]
    fn inspect_and_validate_queries_work() {
        let mut session = ProtocolSession::new(example("minimal-project"));
        let inspected = session.dispatch(request("inspect", json!({})));
        assert!(inspected.ok);
        assert_eq!(
            inspected.result.unwrap()["project"]["id"],
            "example.minimal"
        );

        let validated = session.dispatch(request("validate", json!({})));
        assert!(validated.ok);
        assert_eq!(validated.result.unwrap()["valid"], true);
    }

    #[test]
    fn invalid_project_is_still_queryable() {
        let mut session = ProtocolSession::new(example("invalid-projects/files/entry-missing"));
        let response = session.dispatch(request("validate", json!({})));
        assert!(response.ok);
        assert_eq!(response.result.unwrap()["valid"], false);
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == ErrorCode::FileProjectMissing));
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

    fn temp_project(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "composition-protocol-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        copy_dir(&example("minimal-project"), &root);
        root
    }

    #[test]
    fn apply_transaction_writes_and_external_change_is_rejected() {
        let root = temp_project("apply");
        let mut session = ProtocolSession::new(&root);
        let (_, _, digest, _) = session.current();
        let digest = digest.expect("digest");
        let transaction = json!({
            "request_id": "protocol-1",
            "base_revision": { "seq": 0, "source_digest": digest },
            "commands": [{
                "type": "set_config",
                "instance": "app",
                "property": "title",
                "value": "Protocol"
            }]
        });
        let applied = session.dispatch(request(
            "apply_transaction",
            json!({ "transaction": transaction }),
        ));
        assert!(applied.ok, "{:?}", applied.diagnostics);
        let composition = fs::read_to_string(root.join("composition/main.toml")).unwrap();
        assert!(composition.contains("title = \"Protocol\""));

        let mut text = composition;
        text.insert_str(0, "# external\n");
        fs::write(root.join("composition/main.toml"), text).unwrap();
        let (_, _, digest, _) = session.current();
        let seq = session.session.as_ref().unwrap().workspace().revision().seq;
        let stale = json!({
            "request_id": "protocol-2",
            "base_revision": { "seq": seq, "source_digest": digest },
            "commands": [{
                "type": "set_config",
                "instance": "app",
                "property": "title",
                "value": "Overwrite"
            }]
        });
        let rejected = session.dispatch(request(
            "apply_transaction",
            json!({ "transaction": stale }),
        ));
        assert!(!rejected.ok);
        assert!(rejected
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == ErrorCode::EditRevisionConflict));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unknown_operation_is_a_protocol_error() {
        let mut session = ProtocolSession::new(example("minimal-project"));
        let response = session.dispatch(request("nope", json!({})));
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "protocol-unknown-operation");
    }
}
