use std::collections::BTreeMap;
use std::path::PathBuf;

use composition_domain::{ConfigValue, ErrorCode, LocalId, ModuleId, ObjectId, Revision, Version};
use composition_editor::{ChangeKind, Command, EditorWorkspace, Transaction};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn example(relative: &str) -> String {
    workspace_root()
        .join("examples")
        .join(relative)
        .to_string_lossy()
        .into_owned()
}

fn open(relative: &str) -> EditorWorkspace {
    EditorWorkspace::open(example(relative)).expect("valid project")
}

fn transaction(workspace: &EditorWorkspace, commands: Vec<Command>) -> Transaction {
    Transaction::new("request-1", workspace.revision(), commands)
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

fn codes(outcome: &composition_editor::TransactionOutcome) -> Vec<ErrorCode> {
    outcome
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn set_config_is_atomic_and_updates_effective_config() {
    let mut workspace = open("minimal-project");
    let before = workspace.model().clone();
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Changed".into()),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let app = workspace
        .model()
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "app")
        .unwrap();
    assert_eq!(
        app.config.get(&local("title")),
        Some(&ConfigValue::String("Changed".into()))
    );
    assert_eq!(
        app.effective_config.get(&local("title")),
        Some(&ConfigValue::String("Changed".into()))
    );
    assert_eq!(workspace.revision().seq, 1);
    assert_ne!(before.revision, workspace.model().revision);
}

#[test]
fn invalid_config_rejects_without_mutating_workspace() {
    let mut workspace = open("minimal-project");
    let before = workspace.model().clone();
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::Integer(1),
        }],
    ));
    assert!(!outcome.is_accepted());
    assert!(codes(&outcome).contains(&ErrorCode::ContractPropertyTypeMismatch));
    assert_eq!(before, *workspace.model());
    assert_eq!(workspace.revision().seq, 0);
}

#[test]
fn deleting_a_required_binding_is_rejected() {
    let mut workspace = open("minimal-project");
    let before = workspace.model().clone();
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::DeleteBinding {
            id: local("app-content"),
        }],
    ));
    assert!(!outcome.is_accepted());
    assert!(codes(&outcome).contains(&ErrorCode::StructureSlotCardinality));
    assert_eq!(before, *workspace.model());
}

#[test]
fn rename_updates_bindings_and_revision() {
    let mut workspace = open("minimal-project");
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::RenameInstance {
            from: local("hello"),
            to: local("world"),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let binding = &workspace.model().composition.bindings[0];
    assert_eq!(binding.child, local("world"));
    assert!(workspace
        .model()
        .composition
        .instances
        .iter()
        .any(|instance| instance.id == local("world")));
    assert_eq!(workspace.revision().seq, 1);
}

#[test]
fn create_instance_and_binding_in_one_transaction() {
    let mut workspace = open("edge-valid/slot-many");
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![
            Command::CreateInstance {
                id: local("card-c"),
                module: module("item.card"),
                version: version(),
                config: BTreeMap::from([(local("title"), ConfigValue::String("C".into()))]),
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
    let model = workspace.model();
    assert_eq!(model.composition.instances.len(), 4);
    assert_eq!(model.composition.bindings.len(), 3);
    assert_eq!(workspace.revision().seq, 1);
}

#[test]
fn delete_instance_and_binding_in_one_transaction() {
    let mut workspace = open("edge-valid/slot-many");
    let outcome = workspace.apply(&transaction(
        &workspace,
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
    assert_eq!(workspace.model().composition.instances.len(), 2);
    assert_eq!(workspace.model().composition.bindings.len(), 1);
}

#[test]
fn deleting_a_referenced_instance_alone_is_rejected() {
    let mut workspace = open("minimal-project");
    let before = workspace.model().clone();
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::DeleteInstance { id: local("hello") }],
    ));
    assert!(!outcome.is_accepted());
    assert!(codes(&outcome).contains(&ErrorCode::StructureSlotCardinality));
    assert_eq!(before, *workspace.model());
}

#[test]
fn duplicate_instance_id_is_a_command_error() {
    let mut workspace = open("minimal-project");
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::CreateInstance {
            id: local("app"),
            module: module("leaf.widget"),
            version: version(),
            config: BTreeMap::new(),
        }],
    ));
    assert!(!outcome.is_accepted());
    assert_eq!(codes(&outcome), [ErrorCode::EditInvalidCommand]);
}

#[test]
fn stale_base_revision_is_rejected() {
    let workspace = open("minimal-project");
    let transaction = Transaction::new(
        "request-1",
        Revision::new(99, None),
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Changed".into()),
        }],
    );
    let outcome = workspace.prepare(&transaction);
    assert!(!outcome.is_accepted());
    assert_eq!(codes(&outcome), [ErrorCode::EditRevisionConflict]);
}

#[test]
fn empty_request_id_is_rejected() {
    let workspace = open("minimal-project");
    let transaction = Transaction::new(
        "",
        workspace.revision(),
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Changed".into()),
        }],
    );
    let outcome = workspace.prepare(&transaction);
    assert!(!outcome.is_accepted());
    assert_eq!(codes(&outcome), [ErrorCode::EditInvalidCommand]);
}

#[test]
fn prepare_does_not_mutate_workspace() {
    let workspace = open("minimal-project");
    let before = workspace.model().clone();
    let history_before = workspace.history().len();
    let outcome = workspace.prepare(&transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Prepared".into()),
        }],
    ));
    assert!(outcome.is_accepted());
    assert_eq!(before, *workspace.model());
    assert_eq!(workspace.history().len(), history_before);
}

#[test]
fn diff_reports_modified_instance() {
    let mut workspace = open("minimal-project");
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Changed".into()),
        }],
    ));
    assert!(outcome.is_accepted());
    let diff = outcome.diff().expect("diff");
    assert_eq!(diff.summary.modified, 1);
    assert_eq!(diff.summary.added, 0);
    assert_eq!(diff.changes.len(), 1);
    let change = &diff.changes[0];
    assert_eq!(change.object_id, ObjectId::instance(&local("app")));
    assert_eq!(change.kind, ChangeKind::Modified);
    assert!(change.before.is_some());
    assert!(change.after.is_some());
}

#[test]
fn create_instance_diff_marks_both_objects_added() {
    let mut workspace = open("edge-valid/slot-many");
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![
            Command::CreateInstance {
                id: local("card-c"),
                module: module("item.card"),
                version: version(),
                config: BTreeMap::from([(local("title"), ConfigValue::String("C".into()))]),
            },
            Command::CreateBinding {
                id: local("board-cards-c"),
                parent: local("board"),
                slot: local("cards"),
                child: local("card-c"),
            },
        ],
    ));
    assert!(outcome.is_accepted());
    let diff = outcome.diff().expect("diff");
    assert_eq!(diff.summary.added, 2);
    assert!(diff
        .changes
        .iter()
        .any(|change| change.object_id == ObjectId::instance(&local("card-c"))));
    assert!(diff
        .changes
        .iter()
        .any(|change| change.object_id == ObjectId::binding(&local("board-cards-c"))));
}

#[test]
fn duplicate_request_replays_without_second_commit() {
    let mut workspace = open("minimal-project");
    let transaction = transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Once".into()),
        }],
    );
    let first = workspace.apply(&transaction);
    assert!(first.is_accepted());
    let first_diff = first.diff().expect("diff").clone();
    assert_eq!(workspace.revision().seq, 1);

    let second = workspace.apply(&transaction);
    assert!(second.is_replayed());
    assert_eq!(second.diff(), Some(&first_diff));
    assert_eq!(workspace.revision().seq, 1);
    assert_eq!(workspace.history().len(), 2);
}

#[test]
fn same_request_id_with_different_payload_is_rejected() {
    let mut workspace = open("minimal-project");
    workspace.apply(&transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("First".into()),
        }],
    ));
    let conflicting = Transaction::new(
        "request-1",
        workspace.revision(),
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Second".into()),
        }],
    );
    let outcome = workspace.prepare(&conflicting);
    assert!(!outcome.is_accepted());
    assert_eq!(codes(&outcome), [ErrorCode::EditDuplicateRequest]);
}

#[test]
fn stale_revision_after_a_commit_is_rejected() {
    let mut workspace = open("minimal-project");
    let initial = workspace.revision();
    workspace.apply(&transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Committed".into()),
        }],
    ));
    let stale = Transaction::new(
        "request-2",
        initial,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("Stale".into()),
        }],
    );
    let outcome = workspace.prepare(&stale);
    assert!(!outcome.is_accepted());
    assert_eq!(codes(&outcome), [ErrorCode::EditRevisionConflict]);
}

#[test]
fn history_records_initial_and_committed_revisions() {
    let mut workspace = open("minimal-project");
    assert_eq!(workspace.history().len(), 1);
    assert_eq!(workspace.history()[0].request_id, "initial");
    assert_eq!(workspace.history()[0].revision.seq, 0);

    workspace.apply(&transaction(
        &workspace,
        vec![Command::SetConfig {
            instance: local("app"),
            property: local("title"),
            value: ConfigValue::String("History".into()),
        }],
    ));
    assert_eq!(workspace.history().len(), 2);
    assert_eq!(workspace.history()[1].request_id, "request-1");
    assert_eq!(workspace.history()[1].revision.seq, 1);
    assert_eq!(workspace.history()[1].diff.summary.modified, 1);
}

#[test]
fn opening_an_invalid_project_returns_diagnostics() {
    let error = EditorWorkspace::open(example("invalid-projects/files/entry-missing"))
        .expect_err("invalid project");
    assert!(error
        .iter()
        .any(|d| d.code == ErrorCode::FileProjectMissing));
}

#[test]
fn set_dependency_provider_updates_resolution() {
    let mut workspace = open("edge-valid/dependency-provider");
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::SetDependencyProvider {
            instance: local("engine"),
            requirement: local("logger"),
            provider: local("logger-1"),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let engine = workspace
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
    let resolution = workspace
        .model()
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
    assert_eq!(workspace.revision().seq, 1);
    assert_eq!(outcome.diff().unwrap().summary.modified, 1);
}

#[test]
fn unset_dependency_provider_falls_back_to_auto_resolution() {
    let mut workspace = open("edge-valid/dependency-provider");
    let outcome = workspace.apply(&transaction(
        &workspace,
        vec![Command::UnsetDependencyProvider {
            instance: local("app"),
            requirement: local("logger"),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());
    let app = workspace
        .model()
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "app")
        .unwrap();
    assert!(app.dependency_providers.is_empty());
    let resolution = workspace
        .model()
        .dependencies
        .iter()
        .find(|dependency| {
            dependency.from_instance.as_str() == "app"
                && dependency.requirement_id.as_str() == "logger"
        })
        .unwrap();
    assert_eq!(
        resolution.provider_instance.as_ref().map(LocalId::as_str),
        Some("logger-1")
    );
}

#[test]
fn invalid_dependency_providers_are_rejected() {
    let workspace = open("edge-valid/dependency-provider");
    let cases = [
        (
            Command::SetDependencyProvider {
                instance: local("engine"),
                requirement: local("logger"),
                provider: local("app"),
            },
            ErrorCode::DependencyProviderInvalid,
        ),
        (
            Command::SetDependencyProvider {
                instance: local("engine"),
                requirement: local("logger"),
                provider: local("engine"),
            },
            ErrorCode::DependencySelf,
        ),
        (
            Command::SetDependencyProvider {
                instance: local("engine"),
                requirement: local("unknown"),
                provider: local("logger-1"),
            },
            ErrorCode::ContractDependencyUnknown,
        ),
        (
            Command::SetDependencyProvider {
                instance: local("engine"),
                requirement: local("logger"),
                provider: local("ghost"),
            },
            ErrorCode::RefInstanceNotFound,
        ),
    ];
    for (command, code) in cases {
        let outcome = workspace.prepare(&transaction(&workspace, vec![command]));
        assert!(!outcome.is_accepted(), "expected rejection for {code:?}");
        assert!(
            codes(&outcome).contains(&code),
            "expected {code:?}, got {:?}",
            codes(&outcome)
        );
    }
    assert_eq!(workspace.revision().seq, 0);
}

#[test]
fn dependency_provider_command_errors() {
    let workspace = open("edge-valid/dependency-provider");
    let missing_mapping = workspace.prepare(&transaction(
        &workspace,
        vec![Command::UnsetDependencyProvider {
            instance: local("engine"),
            requirement: local("logger"),
        }],
    ));
    assert_eq!(codes(&missing_mapping), [ErrorCode::EditInvalidCommand]);

    let missing_instance = workspace.prepare(&transaction(
        &workspace,
        vec![Command::SetDependencyProvider {
            instance: local("ghost"),
            requirement: local("logger"),
            provider: local("logger-1"),
        }],
    ));
    assert_eq!(codes(&missing_instance), [ErrorCode::EditInvalidCommand]);
}
