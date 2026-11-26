use std::fs;
use std::path::{Path, PathBuf};

use composition_domain::{ErrorCode, ErrorLayer};
use composition_toml::{load_project, LoadedWorkspace};

fn examples_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn load(relative: &str) -> LoadedWorkspace {
    load_project(examples_root().join(relative))
}

fn codes(workspace: &LoadedWorkspace) -> Vec<ErrorCode> {
    workspace
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn collect_projects(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if path.join("project.toml").is_file() {
                out.push(path.clone());
            }
            collect_projects(&path, out);
        }
    }
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

fn temp_copy(label: &str) -> PathBuf {
    let destination = std::env::temp_dir().join(format!(
        "composition-toml-test-{}-{label}",
        std::process::id()
    ));
    if destination.exists() {
        fs::remove_dir_all(&destination).unwrap();
    }
    copy_dir_all(&examples_root().join("minimal-project"), &destination);
    destination
}

#[test]
fn minimal_project_loads_files_project_and_digest() {
    let workspace = load("minimal-project");
    assert!(workspace.is_valid());
    let project = workspace.project.as_ref().expect("project");
    assert_eq!(project.id.as_str(), "example.minimal");
    assert_eq!(project.composition_entry, "composition/main.toml");
    assert_eq!(project.module_roots, ["modules"]);
    assert_eq!(workspace.files.len(), 4);
    assert_eq!(workspace.module_files().count(), 2);
    assert!(workspace.files.iter().all(|file| file.document().is_some()));
    assert_eq!(
        workspace
            .source_digest
            .as_ref()
            .map(|digest| digest.as_str()),
        Some("sha256:10cef70e22f16ef050fd9d1ea3d0f4ba3502235641aaf4f88b91260daa34768e")
    );
}

#[test]
fn nested_project_loads_all_module_files() {
    let workspace = load("nested-composition");
    assert!(workspace.is_valid());
    assert_eq!(workspace.files.len(), 6);
    assert_eq!(workspace.module_files().count(), 4);
    assert!(workspace
        .module_files()
        .all(|file| file.document().is_some()));
}

#[test]
fn source_spans_map_to_expected_lines() {
    let workspace = load("minimal-project");
    let composition = workspace
        .file("composition/main.toml")
        .expect("composition file");
    let table = composition.document().unwrap().as_table();
    let composition_item = table.get("composition").expect("composition item");
    let composition_table = composition_item.as_table().expect("composition table");
    assert_eq!(
        composition
            .range(composition_item.span().or_else(|| composition_table.span()))
            .unwrap()
            .start
            .line,
        1
    );

    let instances = table
        .get("instances")
        .and_then(|item| item.as_array_of_tables())
        .expect("instances");
    let first = instances.get(0).expect("first instance");
    assert_eq!(composition.range(first.span()).unwrap().start.line, 6);

    let module = workspace
        .file("modules/shell/module.toml")
        .expect("module file");
    let module_item = module
        .document()
        .unwrap()
        .as_table()
        .get("module")
        .expect("module item");
    let module_table = module_item.as_table().expect("module table");
    assert_eq!(
        module
            .range(module_item.span().or_else(|| module_table.span()))
            .unwrap()
            .start
            .line,
        1
    );
}

#[test]
fn format_preserved_text_and_spans_are_kept() {
    let workspace = load("edge-valid/format-preserved");
    assert!(workspace.is_valid());
    let project = workspace.file("project.toml").expect("project file");
    assert!(project.text().contains("# Format-preservation fixture"));
    let project_item = project
        .document()
        .unwrap()
        .as_table()
        .get("project")
        .expect("project item");
    let project_table = project_item.as_table().expect("project table");
    assert_eq!(
        project
            .range(project_item.span().or_else(|| project_table.span()))
            .unwrap()
            .start
            .line,
        3
    );
}

#[test]
fn syntax_error_keeps_file_but_drops_document() {
    let workspace = load("invalid-projects/files/toml-syntax");
    assert_eq!(codes(&workspace), [ErrorCode::TomlSyntaxError]);
    let composition = workspace
        .file("composition/main.toml")
        .expect("composition file");
    assert!(composition.document().is_none());
    assert!(workspace.source_digest.is_some());
}

#[test]
fn duplicate_root_key_is_classified_as_duplicate_key() {
    let workspace = load("invalid-projects/structure/multiple-roots");
    assert_eq!(codes(&workspace), [ErrorCode::TomlDuplicateKey]);
    assert!(workspace
        .file("composition/main.toml")
        .expect("composition file")
        .document()
        .is_none());
}

#[test]
fn path_escape_stops_before_composition() {
    let workspace = load("invalid-projects/files/path-outside");
    assert_eq!(codes(&workspace), [ErrorCode::FilePathEscape]);
    assert!(workspace.project.is_none());
    assert!(workspace.source_digest.is_none());
    assert!(workspace.composition_file().is_none());
}

#[test]
fn missing_entry_reports_project_missing() {
    let workspace = load("invalid-projects/files/entry-missing");
    assert_eq!(codes(&workspace), [ErrorCode::FileProjectMissing]);
    assert!(workspace.files.is_empty());
    assert!(workspace.source_digest.is_none());
}

#[test]
fn all_complete_projects_emit_only_expected_l1_codes() {
    let mut projects = Vec::new();
    collect_projects(&examples_root(), &mut projects);
    projects.sort();
    assert_eq!(projects.len(), 25);

    for project in projects {
        let relative = project
            .strip_prefix(examples_root())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let workspace = load_project(&project);
        let actual = codes(&workspace);
        let expected: &[ErrorCode] = match relative.as_str() {
            "invalid-projects/files/path-outside" => &[ErrorCode::FilePathEscape],
            "invalid-projects/files/toml-syntax" => &[ErrorCode::TomlSyntaxError],
            "invalid-projects/structure/multiple-roots" => &[ErrorCode::TomlDuplicateKey],
            _ => &[],
        };
        assert_eq!(actual, expected, "{relative}");
        for code in actual {
            assert!(
                matches!(code.layer(), ErrorLayer::L1 | ErrorLayer::L2),
                "{relative}: 006 must not emit {code}"
            );
        }
    }
}

#[test]
fn duplicate_module_roots_are_rejected() {
    let directory = temp_copy("duplicate-roots");
    let project_file = directory.join("project.toml");
    let text = fs::read_to_string(&project_file).unwrap();
    fs::write(
        &project_file,
        text.replace(
            "module_roots = [\"modules\"]",
            "module_roots = [\"modules\", \"modules/../modules\"]",
        ),
    )
    .unwrap();
    let workspace = load_project(&directory);
    assert!(codes(&workspace).contains(&ErrorCode::SchemaDuplicateEntry));
    assert!(workspace.project.is_none());
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn nested_module_roots_are_rejected() {
    let directory = temp_copy("nested-roots");
    let project_file = directory.join("project.toml");
    let text = fs::read_to_string(&project_file).unwrap();
    fs::write(
        &project_file,
        text.replace(
            "module_roots = [\"modules\"]",
            "module_roots = [\"modules\", \"modules/shell\"]",
        ),
    )
    .unwrap();
    let workspace = load_project(&directory);
    assert!(codes(&workspace).contains(&ErrorCode::SchemaInconsistency));
    assert!(workspace.project.is_none());
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn unknown_project_field_is_l2() {
    let directory = temp_copy("unknown-field");
    let project_file = directory.join("project.toml");
    let text = fs::read_to_string(&project_file).unwrap();
    fs::write(
        &project_file,
        text.replace(
            "name = \"Minimal Project\"",
            "name = \"Minimal Project\"\nextra = 1",
        ),
    )
    .unwrap();
    let workspace = load_project(&directory);
    assert!(codes(&workspace).contains(&ErrorCode::SchemaUnknownField));
    for code in codes(&workspace) {
        assert!(matches!(code.layer(), ErrorLayer::L1 | ErrorLayer::L2));
    }
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn missing_module_roots_is_l2() {
    let directory = temp_copy("missing-roots");
    let project_file = directory.join("project.toml");
    let text = fs::read_to_string(&project_file).unwrap();
    fs::write(
        &project_file,
        text.replace("module_roots = [\"modules\"]\n", ""),
    )
    .unwrap();
    let workspace = load_project(&directory);
    assert!(codes(&workspace).contains(&ErrorCode::SchemaMissingField));
    assert!(workspace.project.is_none());
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn digest_is_stable_and_changes_with_comments() {
    let directory = temp_copy("digest-comment");
    let first = load_project(&directory);
    let second = load_project(&directory);
    assert_eq!(first.source_digest, second.source_digest);
    assert_eq!(first.files.len(), second.files.len());

    let project_file = directory.join("project.toml");
    let mut text = fs::read_to_string(&project_file).unwrap();
    text.push_str("\n# comment only\n");
    fs::write(&project_file, text).unwrap();

    let changed = load_project(&directory);
    assert_ne!(first.source_digest, changed.source_digest);
    assert!(changed.is_valid());
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn hidden_directories_are_not_scanned_for_modules() {
    let directory = temp_copy("hidden-modules");
    let hidden = directory.join("modules/.hidden");
    fs::create_dir_all(&hidden).unwrap();
    fs::copy(
        directory.join("modules/leaf/module.toml"),
        hidden.join("module.toml"),
    )
    .unwrap();
    let workspace = load_project(&directory);
    assert!(workspace.is_valid());
    assert_eq!(workspace.module_files().count(), 2);
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn composition_parts_are_not_merged_in_v01() {
    let directory = temp_copy("single-entry");
    let before = load_project(&directory);
    let parts = directory.join("composition/parts");
    fs::create_dir_all(&parts).unwrap();
    fs::write(
        parts.join("extra.toml"),
        "[composition]\nschema_version = 1\nid = \"ignored.extra\"\nroot = \"ghost\"\n",
    )
    .unwrap();
    let after = load_project(&directory);
    assert!(after.is_valid());
    assert_eq!(before.source_digest, after.source_digest);
    assert!(after.file("composition/parts/extra.toml").is_none());
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn unsupported_schema_version_is_rejected_without_migration() {
    let directory = temp_copy("schema-version-2");
    let project_file = directory.join("project.toml");
    let text = fs::read_to_string(&project_file).unwrap();
    fs::write(
        &project_file,
        text.replacen("schema_version = 1", "schema_version = 2", 1),
    )
    .unwrap();
    let workspace = load_project(&directory);
    assert!(!workspace.is_valid());
    assert!(codes(&workspace).contains(&ErrorCode::SchemaUnsupportedVersion));
    let _ = fs::remove_dir_all(&directory);
}
