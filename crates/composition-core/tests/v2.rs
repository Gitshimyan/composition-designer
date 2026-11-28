use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use composition_core::{validate_project, ValidationResult};
use composition_domain::{
    DependencyStatus, DependencyTargetKind, ErrorCode, InstanceScope, SchemaVersion,
};
use serde_json::Value;

const PROJECT_V2: &str = r#"[project]
schema_version = 2
id = "example.v2"
name = "V2 Example"

[workspace]
composition = "composition/main.toml"
module_roots = ["modules"]
"#;

fn app_module(dependencies: &str) -> String {
    format!(
        r#"[module]
schema_version = 2
id = "example.app"
version = "1.0.0"
kind = "composite"

[[contract.slots]]
id = "runtime"
cardinality = "optional_one"
required = false
accepts = ["example.session"]

{dependencies}
"#
    )
}

fn service_module(kind: &str, interfaces: &str) -> String {
    format!(
        r#"[module]
schema_version = 2
id = "example.session"
version = "1.0.0"
kind = "{kind}"

{interfaces}
"#
    )
}

const SERVICE_INTERFACE: &str = r#"[[contract.interfaces]]
id = "example.session.api"
version = "1.0.0"
"#;

const APP_DEPENDENCY: &str = r#"[[contract.dependencies]]
id = "session"
interface = "example.session.api"
version = "1.0.0"
optional = false
"#;

const BASIC_COMPOSITION: &str = r#"[composition]
schema_version = 2
id = "example.v2"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"

[[instances]]
id = "session"
module = "example.session"
version = "1.0.0"
scope = "service"
"#;

fn project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "composition-core-v2-{}-{label}",
        std::process::id()
    ))
}

fn write(project: &Path, relative: &str, text: &str) {
    let path = project.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text).unwrap();
}

fn setup(label: &str, project: &str, composition: &str, modules: &[(&str, &str)]) -> PathBuf {
    let root = project_root(label);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    write(&root, "project.toml", project);
    write(&root, "composition/main.toml", composition);
    for (relative, text) in modules {
        write(&root, relative, text);
    }
    root
}

fn codes(result: &ValidationResult) -> BTreeSet<ErrorCode> {
    result.diagnostics.iter().map(|d| d.code).collect()
}

fn json(result: &ValidationResult) -> Value {
    serde_json::to_value(result.model.as_ref().unwrap()).unwrap()
}

fn has_key(value: &Value, key: &str) -> bool {
    match value {
        Value::Object(map) => {
            map.contains_key(key) || map.values().any(|value| has_key(value, key))
        }
        Value::Array(items) => items.iter().any(|item| has_key(item, key)),
        _ => false,
    }
}

fn basic_project(label: &str, app_dependencies: &str, composition: &str) -> PathBuf {
    setup(
        label,
        PROJECT_V2,
        composition,
        &[
            ("modules/app/module.toml", &app_module(app_dependencies)),
            (
                "modules/session/module.toml",
                &service_module("primitive", SERVICE_INTERFACE),
            ),
        ],
    )
}

#[test]
fn v2_interface_and_service_are_valid_and_normalized() {
    let root = basic_project("valid", APP_DEPENDENCY, BASIC_COMPOSITION);
    let result = validate_project(&root);
    assert!(result.is_valid(), "{:?}", result.diagnostics);

    let model = result.model.as_ref().unwrap();
    let app = model
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "app")
        .unwrap();
    let session = model
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "session")
        .unwrap();
    assert_eq!(app.scope, Some(InstanceScope::Structure));
    assert_eq!(session.scope, Some(InstanceScope::Service));

    let dependency = &model.dependencies[0];
    assert_eq!(dependency.status, DependencyStatus::Resolved);
    assert_eq!(
        dependency.provider_instance.as_ref().map(|id| id.as_str()),
        Some("session")
    );
    let target = dependency.target.as_ref().unwrap();
    assert_eq!(target.kind, DependencyTargetKind::Interface);
    assert_eq!(target.id, "example.session.api");
    assert_eq!(
        target.version.map(|v| v.to_string()),
        Some("1.0.0".to_string())
    );

    let value = json(&result);
    let session_scope = value["composition"]["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|instance| instance["id"] == "session")
        .unwrap()["scope"]
        .clone();
    assert_eq!(session_scope, Value::String("service".into()));
    let modules = value["modules"].as_array().unwrap();
    let app = modules
        .iter()
        .find(|module| module["id"] == "example.app")
        .unwrap();
    assert_eq!(app["contract"]["interfaces"], Value::Array(vec![]));
    let service = modules
        .iter()
        .find(|module| module["id"] == "example.session")
        .unwrap();
    assert_eq!(
        service["contract"]["interfaces"][0]["id"],
        "example.session.api"
    );
    assert_eq!(
        value["dependencies"][0]["target"]["kind"],
        Value::String("interface".into())
    );
    let again = json(&validate_project(&root));
    assert_eq!(value, again, "v2 output must be deterministic");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_unknown_interface() {
    let dependency = APP_DEPENDENCY.replace("example.session.api", "example.missing.api");
    let root = basic_project("unknown-interface", &dependency, BASIC_COMPOSITION);
    let result = validate_project(&root);
    let codes = codes(&result);
    assert!(
        codes.contains(&ErrorCode::RefInterfaceNotFound),
        "{codes:?}"
    );
    assert!(!codes.contains(&ErrorCode::DependencyUnresolved));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_duplicate_interface() {
    let interfaces = format!("{SERVICE_INTERFACE}\n{SERVICE_INTERFACE}");
    let root = setup(
        "duplicate-interface",
        PROJECT_V2,
        BASIC_COMPOSITION,
        &[
            ("modules/app/module.toml", &app_module(APP_DEPENDENCY)),
            (
                "modules/session/module.toml",
                &service_module("primitive", &interfaces),
            ),
        ],
    );
    let codes = codes(&validate_project(&root));
    assert!(
        codes.contains(&ErrorCode::IdDuplicateInterface),
        "{codes:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_both_dependency_targets() {
    let dependency = format!("{APP_DEPENDENCY}\nmodule = \"example.other\"\n");
    let root = basic_project("both-targets", &dependency, BASIC_COMPOSITION);
    let codes = codes(&validate_project(&root));
    assert!(codes.contains(&ErrorCode::SchemaInconsistency), "{codes:?}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_service_in_binding() {
    let composition = format!("{BASIC_COMPOSITION}\n[[bindings]]\nid = \"app-runtime\"\nparent = \"app\"\nslot = \"runtime\"\nchild = \"session\"\n");
    let root = basic_project("service-binding", APP_DEPENDENCY, &composition);
    let codes = codes(&validate_project(&root));
    assert!(
        codes.contains(&ErrorCode::StructureServiceInBinding),
        "{codes:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_root_service() {
    let composition = r#"[composition]
schema_version = 2
id = "example.v2"
root = "session"

[[instances]]
id = "session"
module = "example.session"
version = "1.0.0"
scope = "service"
"#;
    let root = basic_project("root-service", APP_DEPENDENCY, composition);
    let codes = codes(&validate_project(&root));
    assert!(
        codes.contains(&ErrorCode::StructureRootScopeInvalid),
        "{codes:?}"
    );
    assert!(
        codes.contains(&ErrorCode::StructureRootNotComposite),
        "{codes:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_composite_service() {
    let root = setup(
        "composite-service",
        PROJECT_V2,
        BASIC_COMPOSITION,
        &[
            ("modules/app/module.toml", &app_module(APP_DEPENDENCY)),
            (
                "modules/session/module.toml",
                &service_module("composite", SERVICE_INTERFACE),
            ),
        ],
    );
    let codes = codes(&validate_project(&root));
    assert!(
        codes.contains(&ErrorCode::ContractServiceModuleInvalid),
        "{codes:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_composition_version_mismatch() {
    let composition = BASIC_COMPOSITION.replacen("schema_version = 2", "schema_version = 1", 1);
    let root = basic_project("version-mismatch", APP_DEPENDENCY, &composition);
    let result = validate_project(&root);
    let codes = codes(&result);
    assert!(
        codes.contains(&ErrorCode::SchemaVersionMismatch),
        "{codes:?}"
    );
    assert!(codes.contains(&ErrorCode::SchemaUnknownField), "{codes:?}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_reports_interface_provider_mismatch() {
    let composition = r#"[composition]
schema_version = 2
id = "example.v2"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"

[instances.dependencies]
session = "other"

[[instances]]
id = "session"
module = "example.session"
version = "1.0.0"
scope = "service"

[[instances]]
id = "other"
module = "example.other"
version = "1.0.0"
scope = "service"
"#;
    let other_module = r#"[module]
schema_version = 2
id = "example.other"
version = "1.0.0"
kind = "primitive"
"#;
    let root = setup(
        "provider-mismatch",
        PROJECT_V2,
        composition,
        &[
            ("modules/app/module.toml", &app_module(APP_DEPENDENCY)),
            (
                "modules/session/module.toml",
                &service_module("primitive", SERVICE_INTERFACE),
            ),
            ("modules/other/module.toml", other_module),
        ],
    );
    let codes = codes(&validate_project(&root));
    assert!(
        codes.contains(&ErrorCode::DependencyInterfaceMismatch),
        "{codes:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_warns_unused_service_without_invalidating_project() {
    let root = basic_project("unused-service", "", BASIC_COMPOSITION);
    let result = validate_project(&root);
    let codes = codes(&result);
    assert!(codes.contains(&ErrorCode::ServiceUnused), "{codes:?}");
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    assert!(!codes.contains(&ErrorCode::StructureUnreachableInstance));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_interface_without_version_is_ambiguous_with_two_implementations() {
    let dependency = r#"[[contract.dependencies]]
id = "session"
interface = "example.session.api"
optional = false
"#;
    let composition = r#"[composition]
schema_version = 2
id = "example.v2"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"

[[instances]]
id = "session-a"
module = "example.session.a"
version = "1.0.0"
scope = "service"

[[instances]]
id = "session-b"
module = "example.session.b"
version = "1.0.0"
scope = "service"
"#;
    let service = |id: &str| {
        format!(
            r#"[module]
schema_version = 2
id = "{id}"
version = "1.0.0"
kind = "primitive"

[[contract.interfaces]]
id = "example.session.api"
version = "1.0.0"
"#
        )
    };
    let root = setup(
        "ambiguous-interface",
        PROJECT_V2,
        composition,
        &[
            ("modules/app/module.toml", &app_module(dependency)),
            ("modules/a/module.toml", &service("example.session.a")),
            ("modules/b/module.toml", &service("example.session.b")),
        ],
    );
    let result = validate_project(&root);
    assert!(
        codes(&result).contains(&ErrorCode::DependencyAmbiguous),
        "{:?}",
        result.diagnostics
    );
    let model = result.model.as_ref().unwrap();
    assert_eq!(model.dependencies[0].status, DependencyStatus::Ambiguous);
    let value = json(&result);
    assert_eq!(value["dependencies"][0]["target"]["version"], Value::Null);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_project_accepts_v1_modules() {
    let app_v1 = r#"[module]
schema_version = 1
id = "example.app"
version = "1.0.0"
kind = "composite"
"#;
    let composition = r#"[composition]
schema_version = 2
id = "example.v2"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"
"#;
    let root = setup(
        "v1-module",
        PROJECT_V2,
        composition,
        &[("modules/app/module.toml", app_v1)],
    );
    let result = validate_project(&root);
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    let value = json(&result);
    assert_eq!(
        value["modules"][0]["contract"]["interfaces"],
        Value::Array(vec![]),
        "v1 modules in a v2 project still emit the v2 interfaces array"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v1_project_rejects_v2_modules() {
    let project_v1 = PROJECT_V2.replacen("schema_version = 2", "schema_version = 1", 1);
    let root = basic_project("v1-project", APP_DEPENDENCY, BASIC_COMPOSITION);
    let path = root.join("project.toml");
    fs::write(&path, project_v1).unwrap();
    let codes = codes(&validate_project(&root));
    assert!(
        codes.contains(&ErrorCode::SchemaUnsupportedVersion),
        "{codes:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v1_model_output_has_no_v2_fields() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let result = validate_project(examples.join("nested-composition"));
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    let value = json(&result);
    assert!(!has_key(&value, "scope"));
    assert!(!has_key(&value, "interfaces"));
    assert!(!has_key(&value, "target"));
    assert_eq!(
        value["composition"]["schema_version"],
        Value::from(1),
        "v1 output keeps schema_version 1"
    );
    let model = result.model.as_ref().unwrap();
    assert_eq!(model.project.schema_version, SchemaVersion::V1);
}
