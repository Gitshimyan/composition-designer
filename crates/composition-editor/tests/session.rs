use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use composition_core::validate_project;
use composition_domain::{ConfigValue, ErrorCode, LocalId, ModuleId, PortRef, Version};
use composition_editor::{Command, ProjectSession, Transaction};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    path: PathBuf,
}

impl TempProject {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_example(relative: &str) -> TempProject {
    let source = workspace_root().join("examples").join(relative);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("composition-011-{}-{id}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    copy_dir(&source, &path);
    TempProject { path }
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

fn prepend_comment(project: &Path) {
    let path = project.join("composition/main.toml");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("# keep this comment\n{text}")).unwrap();
}

fn composition_text(project: &Path) -> String {
    fs::read_to_string(project.join("composition/main.toml")).unwrap()
}

fn transaction(session: &ProjectSession, commands: Vec<Command>) -> Transaction {
    Transaction::new("request-1", session.workspace().revision(), commands)
}

fn local(value: &str) -> LocalId {
    LocalId::parse(value).unwrap()
}

fn module(value: &str) -> ModuleId {
    ModuleId::parse(value).unwrap()
}

fn version() -> Version {
    Version::new(1, 0, 0)
}

fn assert_no_temporary_files(project: &Path) {
    let directory = project.join("composition");
    for entry in fs::read_dir(directory).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        assert!(
            !name.contains(".tmp-") && !name.contains(".bak-"),
            "temporary file left behind: {name}"
        );
    }
}

#[test]
fn set_config_preserves_comments_and_refreshes_digest() {
    let project = copy_example("minimal-project");
    prepend_comment(project.path());
    let mut session = ProjectSession::open(project.path()).unwrap();
    let before_digest = session.workspace().revision().source_digest.clone();
    let outcome = session.apply(&transaction(
        &session,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Edited".into()),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());

    let text = composition_text(project.path());
    assert!(text.contains("# keep this comment"));
    assert!(text.contains("title = \"Edited\""));
    assert!(text.contains("label = \"Hello\""));
    assert_no_temporary_files(project.path());

    let validation = validate_project(project.path());
    assert!(validation.is_valid());
    assert_ne!(session.workspace().revision().source_digest, before_digest);
    assert_eq!(
        session.workspace().revision().source_digest,
        validation.source_digest
    );
    assert_eq!(session.workspace().revision().seq, 1);
    assert_eq!(session.workspace().history().len(), 2);
}

#[test]
fn invalid_config_leaves_the_file_untouched() {
    let project = copy_example("minimal-project");
    let mut session = ProjectSession::open(project.path()).unwrap();
    let before = composition_text(project.path());
    let before_revision = session.workspace().revision();
    let outcome = session.apply(&transaction(
        &session,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::Integer(1),
        }],
    ));
    assert!(!outcome.is_accepted());
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == ErrorCode::ContractPropertyTypeMismatch));
    assert_eq!(composition_text(project.path()), before);
    assert_eq!(session.workspace().revision(), before_revision);
    assert_no_temporary_files(project.path());
}

#[test]
fn prepare_does_not_write_the_file() {
    let project = copy_example("minimal-project");
    let session = ProjectSession::open(project.path()).unwrap();
    let before = composition_text(project.path());
    let outcome = session.prepare(&transaction(
        &session,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Prepared".into()),
        }],
    ));
    assert!(outcome.is_accepted());
    assert_eq!(composition_text(project.path()), before);
}

#[test]
fn interrupted_backup_is_recovered_on_open() {
    let project = copy_example("minimal-project");
    let composition = project.path().join("composition/main.toml");
    let backup = composition.with_file_name(".main.toml.bak-999");
    fs::rename(&composition, &backup).unwrap();
    assert!(!composition.exists());

    let session = ProjectSession::open(project.path()).expect("recovered session");
    assert!(composition.exists());
    assert!(session.source_text().contains("[composition]"));
    assert_no_temporary_files(project.path());
}

#[test]
fn create_instance_and_binding_writes_valid_toml() {
    let project = copy_example("edge-valid/slot-many");
    prepend_comment(project.path());
    let mut session = ProjectSession::open(project.path()).unwrap();
    let outcome = session.apply(&transaction(
        &session,
        vec![
            Command::CreateInstance {
                id: local("card-c"),
                module: module("item.card"),
                version: version(),
                config: [(local("title"), ConfigValue::String("C".into()))]
                    .into_iter()
                    .collect(),
            },
            Command::CreateBinding {
                id: local("board-cards-c"),
                parent: local("board"),
                slot: local("cards"),
                child: local("card-c"),
            },
        ],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let text = composition_text(project.path());
    assert!(text.contains("# keep this comment"));
    assert!(text.contains("id = \"card-c\""));
    assert!(text.contains("id = \"board-cards-c\""));
    assert_no_temporary_files(project.path());
    let validation = validate_project(project.path());
    assert!(validation.is_valid());
    let model = validation.model.unwrap();
    assert_eq!(model.composition.instances.len(), 4);
    assert_eq!(model.composition.bindings.len(), 3);
}

#[test]
fn delete_instance_and_binding_removes_tables() {
    let project = copy_example("edge-valid/slot-many");
    prepend_comment(project.path());
    let mut session = ProjectSession::open(project.path()).unwrap();
    let outcome = session.apply(&transaction(
        &session,
        vec![
            Command::DeleteBinding {
                id: local("board-cards-a"),
            },
            Command::DeleteInstance {
                id: local("card-a"),
            },
        ],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let text = composition_text(project.path());
    assert!(!text.contains("id = \"card-a\""));
    assert!(!text.contains("id = \"board-cards-a\""));
    assert!(text.contains("id = \"card-b\""));
    assert_eq!(text.matches("[[instances]]").count(), 2);
    assert_eq!(text.matches("[[bindings]]").count(), 1);
    assert_no_temporary_files(project.path());
    assert!(validate_project(project.path()).is_valid());
}

#[test]
fn rename_updates_bindings_in_the_file() {
    let project = copy_example("minimal-project");
    let mut session = ProjectSession::open(project.path()).unwrap();
    let outcome = session.apply(&transaction(
        &session,
        vec![Command::RenameInstance {
            from: local("hello"),
            to: local("world"),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let text = composition_text(project.path());
    assert!(text.contains("id = \"world\""));
    assert!(text.contains("child = \"world\""));
    assert!(!text.contains("id = \"hello\""));
    assert_no_temporary_files(project.path());
    assert!(validate_project(project.path()).is_valid());
}

#[test]
fn create_connection_writes_nested_endpoint_tables() {
    let project = copy_example("invalid-projects/connections/errors");
    fs::write(
        project.path().join("composition/main.toml"),
        r#"[composition]
schema_version = 1
id = "example.invalid.connections"
root = "app"

[[instances]]
id = "app"
module = "shell.desktop"
version = "1.0.0"

[[instances]]
id = "editor"
module = "ui.editor"
version = "1.0.0"

[[instances]]
id = "statusbar"
module = "ui.statusbar"
version = "1.0.0"

[[instances]]
id = "logger"
module = "ui.logger"
version = "1.0.0"

[[bindings]]
id = "app-editor"
parent = "app"
slot = "content"
child = "editor"

[[bindings]]
id = "app-statusbar"
parent = "app"
slot = "content"
child = "statusbar"

[[bindings]]
id = "app-logger"
parent = "app"
slot = "content"
child = "logger"

[[connections]]
id = "c1"
[connections.from]
instance = "editor"
port = "text"
[connections.to]
instance = "statusbar"
port = "text"
"#,
    )
    .unwrap();

    let mut session = ProjectSession::open(project.path()).unwrap();
    let outcome = session.apply(&transaction(
        &session,
        vec![Command::CreateConnection {
            id: local("c2"),
            from: PortRef::new(local("statusbar"), local("ack")),
            to: PortRef::new(local("editor"), local("ack")),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let text = composition_text(project.path());
    assert!(text.contains("id = \"c2\""));
    assert!(text.contains("[connections.from]"));
    assert_no_temporary_files(project.path());
    let validation = validate_project(project.path());
    assert!(validation.is_valid(), "{:?}", validation.diagnostics);
    assert_eq!(validation.model.unwrap().composition.connections.len(), 2);
}

#[test]
fn dependency_provider_set_and_unset_write_back() {
    let project = copy_example("edge-valid/dependency-provider");
    prepend_comment(project.path());
    let mut session = ProjectSession::open(project.path()).unwrap();

    let outcome = session.apply(&Transaction::new(
        "set-engine-provider",
        session.workspace().revision(),
        vec![Command::SetDependencyProvider {
            instance: local("engine"),
            requirement: local("logger"),
            provider: local("logger-1"),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let text = composition_text(project.path());
    assert!(text.contains("# keep this comment"));
    assert_eq!(text.matches("[instances.dependencies]").count(), 2);
    let engine = session
        .workspace()
        .model()
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "engine")
        .unwrap();
    assert_eq!(
        engine.dependency_providers.get(&local("logger")),
        Some(&local("logger-1"))
    );

    let outcome = session.apply(&Transaction::new(
        "clear-app-provider",
        session.workspace().revision(),
        vec![Command::UnsetDependencyProvider {
            instance: local("app"),
            requirement: local("logger"),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let text = composition_text(project.path());
    assert_eq!(text.matches("[instances.dependencies]").count(), 1);

    let outcome = session.apply(&Transaction::new(
        "clear-engine-provider",
        session.workspace().revision(),
        vec![Command::UnsetDependencyProvider {
            instance: local("engine"),
            requirement: local("logger"),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let text = composition_text(project.path());
    assert!(!text.contains("[instances.dependencies]"));

    let validation = validate_project(project.path());
    assert!(validation.is_valid(), "{:?}", validation.diagnostics);
    let model = validation.model.unwrap();
    assert!(model
        .composition
        .instances
        .iter()
        .all(|instance| instance.dependency_providers.is_empty()));
    let resolution = model
        .dependencies
        .iter()
        .find(|dependency| {
            dependency.from_instance.as_str() == "engine"
                && dependency.requirement_id.as_str() == "logger"
        })
        .unwrap();
    assert_eq!(
        resolution.provider_instance.as_ref().map(LocalId::as_str),
        Some("logger-1")
    );
    assert_no_temporary_files(project.path());
}

#[test]
fn invalid_dependency_provider_leaves_file_untouched() {
    let project = copy_example("edge-valid/dependency-provider");
    let mut session = ProjectSession::open(project.path()).unwrap();
    let before = composition_text(project.path());
    let outcome = session.apply(&Transaction::new(
        "invalid-engine-provider",
        session.workspace().revision(),
        vec![Command::SetDependencyProvider {
            instance: local("engine"),
            requirement: local("logger"),
            provider: local("app"),
        }],
    ));
    assert!(!outcome.is_accepted());
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == ErrorCode::DependencyProviderInvalid));
    assert_eq!(composition_text(project.path()), before);
    assert_eq!(session.workspace().revision().seq, 0);
    assert_no_temporary_files(project.path());
}
