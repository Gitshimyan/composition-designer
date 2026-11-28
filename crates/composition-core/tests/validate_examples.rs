use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use composition_core::validate_project;
use composition_domain::{
    ConfigValue, DependencyStatus, ErrorCode, LocalId, ModuleId, ObjectId, Version,
};
use serde_json::Value;

fn examples_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn collect_datasets(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if path.join("expected.json").is_file() {
                out.push(path.clone());
            }
            collect_datasets(&path, out);
        }
    }
}

fn expected_codes(directory: &Path) -> BTreeSet<ErrorCode> {
    let text = fs::read_to_string(directory.join("expected.json")).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|code| ErrorCode::from_code(code.as_str().unwrap()).unwrap())
        .collect()
}

fn copy_dir_all(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn temp_copy(label: &str, relative: &str) -> PathBuf {
    let destination = std::env::temp_dir().join(format!(
        "composition-core-test-{}-{label}",
        std::process::id()
    ));
    if destination.exists() {
        fs::remove_dir_all(&destination).unwrap();
    }
    copy_dir_all(&examples_root().join(relative), &destination);
    destination
}

#[test]
fn every_dataset_matches_expected_codes() {
    let mut datasets = Vec::new();
    collect_datasets(&examples_root(), &mut datasets);
    datasets.sort();
    assert_eq!(datasets.len(), 40);

    for directory in datasets {
        let relative = directory
            .strip_prefix(examples_root())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let expected = expected_codes(&directory);
        let result = validate_project(&directory);
        let actual: BTreeSet<ErrorCode> = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect();
        assert_eq!(actual, expected, "{relative}");

        for pair in result.diagnostics.windows(2) {
            assert_ne!(
                pair[0].compare_order(&pair[1]),
                Ordering::Greater,
                "{relative}: diagnostics are not sorted"
            );
        }

        let model_expected = !matches!(
            relative.as_str(),
            "invalid-projects/files/entry-missing"
                | "invalid-projects/files/path-outside"
                | "invalid-projects/files/toml-syntax"
                | "invalid-projects/structure/multiple-roots"
                | "v3/invalid-reserved-profile"
        );
        assert_eq!(result.model.is_some(), model_expected, "{relative}");
    }
}

#[test]
fn minimal_project_builds_a_normalized_model() {
    let result = validate_project(examples_root().join("minimal-project"));
    assert!(result.is_valid());
    assert!(result.diagnostics.is_empty());
    let model = result.model.expect("model");
    assert_eq!(model.project.id.as_str(), "example.minimal");
    assert_eq!(
        model
            .modules
            .iter()
            .map(|module| module.id.as_str())
            .collect::<Vec<_>>(),
        ["leaf.widget", "shell.desktop"]
    );
    assert_eq!(
        model
            .composition
            .instances
            .iter()
            .map(|instance| instance.id.as_str())
            .collect::<Vec<_>>(),
        ["app", "hello"]
    );
    assert_eq!(model.composition.bindings.len(), 1);
    assert_eq!(model.composition.connections.len(), 0);
    assert_eq!(model.dependencies.len(), 0);
    let app = model
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "app")
        .expect("app");
    assert_eq!(
        app.effective_config.get(&LocalId::parse("title").unwrap()),
        Some(&ConfigValue::String("Minimal Project".into()))
    );
    assert!(model.is_valid());
    let revision = model.revision.expect("revision");
    assert_eq!(revision.seq, 0);
    assert_eq!(
        revision
            .source_digest
            .as_ref()
            .map(|digest| digest.as_str()),
        Some("sha256:10cef70e22f16ef050fd9d1ea3d0f4ba3502235641aaf4f88b91260daa34768e")
    );
}

#[test]
fn nested_and_multiple_instance_counts_are_preserved() {
    let nested = validate_project(examples_root().join("nested-composition"));
    let model = nested.model.expect("nested model");
    assert_eq!(model.modules.len(), 4);
    assert_eq!(model.composition.instances.len(), 8);
    assert_eq!(model.composition.bindings.len(), 7);

    let slot_many = validate_project(examples_root().join("edge-valid/slot-many"));
    let model = slot_many.model.expect("slot-many model");
    assert_eq!(model.modules.len(), 2);
    assert_eq!(model.composition.instances.len(), 3);
    assert_eq!(model.composition.bindings.len(), 2);
}

#[test]
fn dependency_provider_resolves_both_explicit_and_unique_auto() {
    let result = validate_project(examples_root().join("edge-valid/dependency-provider"));
    assert!(result.is_valid());
    let model = result.model.expect("model");
    assert_eq!(model.dependencies.len(), 2);
    assert!(model
        .dependencies
        .iter()
        .all(|dependency| dependency.status == DependencyStatus::Resolved));
    assert!(model.dependencies.iter().all(|dependency| dependency
        .provider_instance
        .as_ref()
        .is_some_and(|id| id.as_str() == "logger-1")));
}

#[test]
fn effective_config_applies_module_defaults() {
    let directory = temp_copy("defaults", "nested-composition");
    let composition = directory.join("composition/main.toml");
    let text = fs::read_to_string(&composition).unwrap();
    fs::write(&composition, text.replace("heading = \"Left\"\n", "")).unwrap();

    let result = validate_project(&directory);
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    let model = result.model.expect("model");
    let panel = model
        .composition
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "panel-1")
        .expect("panel-1");
    let heading = LocalId::parse("heading").unwrap();
    assert_eq!(panel.config.get(&heading), None);
    assert_eq!(
        panel.effective_config.get(&heading),
        Some(&ConfigValue::String("Panel".into()))
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn unknown_dependency_mapping_key_is_reported() {
    let directory = temp_copy("dependency-unknown", "edge-valid/dependency-provider");
    let composition = directory.join("composition/main.toml");
    let text = fs::read_to_string(&composition).unwrap();
    fs::write(
        &composition,
        text.replace(
            "logger = \"logger-1\"",
            "logger = \"logger-1\"\nunknown = \"logger-1\"",
        ),
    )
    .unwrap();

    let result = validate_project(&directory);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == ErrorCode::ContractDependencyUnknown));
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn source_map_contains_contract_and_instance_locations() {
    let result = validate_project(examples_root().join("minimal-project"));
    let module = ModuleId::parse("shell.desktop").unwrap();
    let version = Version::parse("1.0.0").unwrap();
    let title = LocalId::parse("title").unwrap();
    let property = ObjectId::property(&module, &version, &title);
    let source = result.source_map.get(&property).expect("property source");
    assert_eq!(source.file, "modules/shell/module.toml");
    assert!(source.range.is_some());

    let instance = ObjectId::instance(&LocalId::parse("app").unwrap());
    let source = result.source_map.get(&instance).expect("instance source");
    assert_eq!(source.file, "composition/main.toml");
    assert!(source.range.is_some());
}

#[test]
fn validation_is_deterministic() {
    let first = validate_project(examples_root().join("minimal-project"));
    let second = validate_project(examples_root().join("minimal-project"));
    assert_eq!(first.diagnostics.len(), second.diagnostics.len());
    for (left, right) in first.diagnostics.iter().zip(second.diagnostics.iter()) {
        assert_eq!(left.code, right.code);
        assert_eq!(left.object_id, right.object_id);
        assert_eq!(left.message, right.message);
        assert_eq!(left.range, right.range);
    }

    let first = first.model.expect("first model");
    let second = second.model.expect("second model");
    assert_eq!(first.project.id, second.project.id);
    assert_eq!(first.composition.id, second.composition.id);
    assert_eq!(first.revision, second.revision);
    assert_eq!(
        first
            .composition
            .instances
            .iter()
            .map(|instance| instance.id.clone())
            .collect::<Vec<_>>(),
        second
            .composition
            .instances
            .iter()
            .map(|instance| instance.id.clone())
            .collect::<Vec<_>>()
    );
}
