use std::fs;
use std::path::{Path, PathBuf};

use composition_core::{validate_project, validate_project_with_profile};
use composition_domain::ErrorCode;

const PROJECT: &str = r#"[project]
schema_version = 3
id = "example.v3"
name = "V3 Profiles"

[workspace]
composition = "composition/main.toml"
module_roots = ["modules"]

[profiles.desktop]
composition = "composition/desktop.toml"

[profiles.headless]
composition = "composition/headless.toml"
"#;

const APP_MODULE: &str = r#"[module]
schema_version = 3
id = "example.app"
version = "1.0.0"
kind = "composite"

[contract.properties.variant]
type = "string"
required = false
"#;

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text).unwrap();
}

fn composition(entry: &str) -> String {
    format!(
        r#"[composition]
schema_version = 3
id = "example.v3"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"

[instances.config]
variant = "{entry}"
"#
    )
}

fn setup(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "composition-core-v3-{}-{label}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    write(&root, "project.toml", PROJECT);
    write(&root, "composition/main.toml", &composition("main"));
    write(&root, "composition/desktop.toml", &composition("desktop"));
    write(&root, "composition/headless.toml", &composition("headless"));
    write(&root, "modules/app/module.toml", APP_MODULE);
    root
}

fn codes(result: &composition_core::ValidationResult) -> Vec<ErrorCode> {
    result.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn v3_default_profile_uses_workspace_entry() {
    let root = setup("default");
    let result = validate_project(&root);
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    let model = result.model.expect("model");
    assert_eq!(model.project.schema_version.get(), 3);
    assert_eq!(model.project.active_profile.as_deref(), Some("default"));
    assert_eq!(
        model.project.default_composition.as_deref(),
        Some("composition/main.toml")
    );
    assert_eq!(model.project.composition_entry, "composition/main.toml");
    assert_eq!(
        model
            .project
            .profiles
            .iter()
            .map(|profile| profile.name.as_str())
            .collect::<Vec<_>>(),
        ["desktop", "headless"]
    );
    assert_eq!(model.composition.schema_version.get(), 3);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_explicit_profile_selects_entry_and_digest() {
    let root = setup("select");
    let default = validate_project(&root);
    let default_digest = default.source_digest.clone();

    let desktop = validate_project_with_profile(&root, Some("desktop"));
    assert!(desktop.is_valid(), "{:?}", desktop.diagnostics);
    let model = desktop.model.expect("model");
    assert_eq!(model.project.active_profile.as_deref(), Some("desktop"));
    assert_eq!(model.project.composition_entry, "composition/desktop.toml");
    assert_eq!(
        model.project.default_composition.as_deref(),
        Some("composition/main.toml")
    );

    let validation = validate_project(&root);
    let other = validate_project_with_profile(&root, Some("headless"));
    assert_ne!(default_digest, desktop.source_digest);
    assert_ne!(validation.source_digest, other.source_digest);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_other_profile_files_do_not_change_digest() {
    let root = setup("digest-scope");
    let before = validate_project(&root).source_digest;
    fs::write(
        root.join("composition/headless.toml"),
        composition("headless-edited"),
    )
    .unwrap();
    let after = validate_project(&root).source_digest;
    assert_eq!(before, after);
    let changed = validate_project_with_profile(&root, Some("headless")).source_digest;
    assert_ne!(after, changed);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_profile_errors_are_reported() {
    let root = setup("errors");
    let missing = validate_project_with_profile(&root, Some("missing"));
    assert!(!missing.is_valid());
    assert!(codes(&missing).contains(&ErrorCode::ProfileNotFound));
    assert!(missing.model.is_none());

    let reserved =
        format!("{PROJECT}\n[profiles.default]\ncomposition = \"composition/main.toml\"\n");
    write(&root, "project.toml", &reserved);
    let invalid = validate_project(&root);
    assert!(codes(&invalid).contains(&ErrorCode::ProfileReservedName));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_profile_selection_is_unsupported_on_v2_projects() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let result =
        validate_project_with_profile(examples.join("v2/valid-service-interface"), Some("desktop"));
    assert!(!result.is_valid());
    assert!(codes(&result).contains(&ErrorCode::ProfileUnsupported));
}

#[test]
fn v3_composition_version_mismatch() {
    let root = setup("version-mismatch");
    let text = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    write(
        &root,
        "composition/main.toml",
        &text.replacen("schema_version = 3", "schema_version = 2", 1),
    );
    let result = validate_project(&root);
    assert!(codes(&result).contains(&ErrorCode::SchemaVersionMismatch));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_project_accepts_v1_v2_v3_modules() {
    let root = setup("modules");
    write(
        &root,
        "modules/legacy-v1/module.toml",
        "[module]\nschema_version = 1\nid = \"example.legacy-v1\"\nversion = \"1.0.0\"\nkind = \"primitive\"\n",
    );
    write(
        &root,
        "modules/legacy-v2/module.toml",
        "[module]\nschema_version = 2\nid = \"example.legacy-v2\"\nversion = \"1.0.0\"\nkind = \"primitive\"\n",
    );
    let result = validate_project(&root);
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    let model = result.model.expect("model");
    assert_eq!(model.modules.len(), 3);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_profile_path_and_name_errors() {
    let root = setup("profile-shape");
    let text = fs::read_to_string(root.join("project.toml")).unwrap();
    let bad_name =
        format!("{text}\n[profiles.\"Bad Name\"]\ncomposition = \"composition/main.toml\"\n");
    write(&root, "project.toml", &bad_name);
    let result = validate_project(&root);
    assert!(
        codes(&result).contains(&ErrorCode::SchemaInvalidIdentifier),
        "{:?}",
        result.diagnostics
    );

    let escape = format!("{text}\n[profiles.escape]\ncomposition = \"../outside.toml\"\n");
    write(&root, "project.toml", &escape);
    let result = validate_project(&root);
    assert!(
        codes(&result).contains(&ErrorCode::FilePathEscape),
        "{:?}",
        result.diagnostics
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_missing_selected_entry_is_reported() {
    let root = setup("missing-entry");
    fs::remove_file(root.join("composition/desktop.toml")).unwrap();
    let result = validate_project_with_profile(&root, Some("desktop"));
    assert!(!result.is_valid());
    assert!(codes(&result).contains(&ErrorCode::FileCompositionMissing));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v3_default_profile_can_be_selected_explicitly() {
    let root = setup("explicit-default");
    let result = validate_project_with_profile(&root, Some("default"));
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    let model = result.model.expect("model");
    assert_eq!(model.project.active_profile.as_deref(), Some("default"));
    assert_eq!(model.project.composition_entry, "composition/main.toml");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn v2_output_has_no_profile_fields() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let result = validate_project(examples.join("v2/valid-service-interface"));
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    let value = serde_json::to_value(result.model.unwrap()).unwrap();
    assert!(value["project"].get("profiles").is_none());
    assert!(value["project"].get("active_profile").is_none());
    assert!(value["project"].get("default_composition").is_none());
}
