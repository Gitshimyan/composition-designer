use std::fs;
use std::path::{Path, PathBuf};

use composition_core::validate_project;
use composition_domain::{InstanceScope, LocalId, ModuleId, SchemaVersion};
use composition_editor::{Command, EditorWorkspace, ProjectSession, Transaction};

const PROJECT_V2: &str = r#"[project]
schema_version = 2
id = "example.v2"
name = "V2 Example"

[workspace]
composition = "composition/main.toml"
module_roots = ["modules"]
"#;

const COMPOSITION_V2: &str = r#"[composition]
schema_version = 2
id = "example.v2"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"
"#;

const APP_MODULE: &str = r#"[module]
schema_version = 2
id = "example.app"
version = "1.0.0"
kind = "composite"

[[contract.slots]]
id = "runtime"
cardinality = "optional_one"
required = false
accepts = ["example.session"]
"#;

const SERVICE_MODULE: &str = r#"[module]
schema_version = 2
id = "example.session"
version = "1.0.0"
kind = "primitive"
"#;

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text).unwrap();
}

fn project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "composition-editor-v2-{}-{label}",
        std::process::id()
    ))
}

fn v2_project(label: &str) -> PathBuf {
    let root = project_root(label);
    let _ = fs::remove_dir_all(&root);
    write(&root, "project.toml", PROJECT_V2);
    write(&root, "composition/main.toml", COMPOSITION_V2);
    write(&root, "modules/app/module.toml", APP_MODULE);
    write(&root, "modules/session/module.toml", SERVICE_MODULE);
    root
}

fn local(value: &str) -> LocalId {
    LocalId::parse(value).unwrap()
}

fn module(value: &str) -> ModuleId {
    ModuleId::parse(value).unwrap()
}

fn create_service(scope: Option<InstanceScope>) -> Command {
    Command::CreateInstance {
        id: local("session"),
        module: module("example.session"),
        version: composition_domain::Version::parse("1.0.0").unwrap(),
        config: Default::default(),
        scope,
    }
}

#[test]
fn v2_create_service_writes_scope_and_remains_valid() {
    let root = v2_project("service-writeback");
    let mut session = ProjectSession::open(&root).unwrap();
    assert_eq!(
        session.workspace().model().project.schema_version,
        SchemaVersion::V2
    );
    let revision = session.workspace().revision();
    let outcome = session.apply(&Transaction::new(
        "create-service",
        revision,
        vec![create_service(Some(InstanceScope::Service))],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());

    let model = session.workspace().model();
    let service = model
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "session")
        .unwrap();
    assert_eq!(service.scope, Some(InstanceScope::Service));

    let text = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert!(text.contains("scope = \"service\""), "{text}");
    assert!(validate_project(&root).is_valid());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_create_without_scope_defaults_to_structure() {
    let root = v2_project("default-scope");
    let mut workspace = EditorWorkspace::open(&root).unwrap();
    let revision = workspace.revision();
    let outcome = workspace.apply(&Transaction::new(
        "create-structure",
        revision,
        vec![
            create_service(None),
            Command::CreateBinding {
                id: local("app-runtime"),
                parent: local("app"),
                slot: local("runtime"),
                child: local("session"),
            },
        ],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let service = workspace
        .model()
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "session")
        .unwrap();
    assert_eq!(service.scope, Some(InstanceScope::Structure));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v1_project_rejects_explicit_scope() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/minimal-project");
    let root = project_root("v1-scope");
    let _ = fs::remove_dir_all(&root);
    copy_dir(&examples, &root);

    let mut workspace = EditorWorkspace::open(&root).unwrap();
    let revision = workspace.revision();
    let outcome = workspace.apply(&Transaction::new(
        "v1-scope",
        revision,
        vec![Command::CreateInstance {
            id: local("scope-probe"),
            module: module("leaf.widget"),
            version: composition_domain::Version::parse("1.0.0").unwrap(),
            config: Default::default(),
            scope: Some(InstanceScope::Structure),
        }],
    ));
    let codes: Vec<_> = outcome.diagnostics().iter().map(|d| d.code).collect();
    assert!(!outcome.is_accepted());
    assert!(
        codes.contains(&composition_domain::ErrorCode::EditInvalidCommand),
        "{codes:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
