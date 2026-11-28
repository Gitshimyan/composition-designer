use std::fs;
use std::path::{Path, PathBuf};

use composition_domain::{ConfigValue, ErrorCode, LocalId};
use composition_editor::{Command, ProjectSession, Transaction};

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
        "composition-editor-v3-{}-{label}",
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

#[test]
fn session_writes_only_the_selected_profile_entry() {
    let root = setup("write");
    let mut session = ProjectSession::open_with_profile(&root, Some("desktop")).unwrap();
    assert_eq!(session.composition_entry(), "composition/desktop.toml");
    assert!(session.source_text().contains("variant = \"desktop\""));

    let outcome = session.apply(&Transaction::new(
        "profile-edit",
        session.workspace().revision(),
        vec![Command::SetConfig {
            instance: LocalId::parse("app").unwrap(),
            property: LocalId::parse("variant").unwrap(),
            value: ConfigValue::String("edited".to_owned()),
        }],
    ));
    assert!(outcome.is_accepted(), "{:?}", outcome.diagnostics());

    let desktop = fs::read_to_string(root.join("composition/desktop.toml")).unwrap();
    assert!(desktop.contains("variant = \"edited\""), "{desktop}");
    let main = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert!(main.contains("variant = \"main\""), "{main}");
    let headless = fs::read_to_string(root.join("composition/headless.toml")).unwrap();
    assert!(headless.contains("variant = \"headless\""), "{headless}");

    let reopened = ProjectSession::open_with_profile(&root, Some("desktop")).unwrap();
    assert!(reopened
        .workspace()
        .model()
        .composition
        .instances
        .iter()
        .any(
            |instance| instance.config.get(&LocalId::parse("variant").unwrap())
                == Some(&ConfigValue::String("edited".to_owned()))
        ));
    let default = ProjectSession::open(&root).unwrap();
    assert_eq!(default.composition_entry(), "composition/main.toml");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn session_profile_errors_are_reported() {
    let root = setup("errors");
    let missing = ProjectSession::open_with_profile(&root, Some("missing")).unwrap_err();
    assert!(missing
        .iter()
        .any(|diagnostic| diagnostic.code == ErrorCode::ProfileNotFound));

    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let unsupported = ProjectSession::open_with_profile(
        examples.join("v2/valid-service-interface"),
        Some("desktop"),
    )
    .unwrap_err();
    assert!(unsupported
        .iter()
        .any(|diagnostic| diagnostic.code == ErrorCode::ProfileUnsupported));
    let _ = fs::remove_dir_all(&root);
}
