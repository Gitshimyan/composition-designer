use std::collections::BTreeSet;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use composition_domain::{
    Diagnostic, ErrorCode, ModuleId, ObjectId, Project, SchemaVersion, SourceDigest,
};
use toml_edit::{ImDocument, Item, Table};

use crate::digest::compute_source_digest;
use crate::loaded_file::{FileRole, LoadedFile};
use crate::paths::{is_nested_path, normalize_relative_path};

/// 一次项目加载的结果。`project` 为 `None` 表示项目入口/路径声明不可用；
/// 语义解析与引用校验由 007 负责。
pub struct LoadedWorkspace {
    pub root: PathBuf,
    pub project: Option<Project>,
    pub files: Vec<LoadedFile>,
    pub diagnostics: Vec<Diagnostic>,
    pub source_digest: Option<SourceDigest>,
}

impl LoadedWorkspace {
    pub fn is_valid(&self) -> bool {
        !self.diagnostics.iter().any(Diagnostic::is_error)
    }

    pub fn file(&self, relative_path: &str) -> Option<&LoadedFile> {
        self.files
            .iter()
            .find(|file| file.relative_path() == relative_path)
    }

    pub fn composition_file(&self) -> Option<&LoadedFile> {
        self.project.as_ref().and_then(|project| {
            self.files
                .iter()
                .find(|file| file.relative_path() == project.composition_entry)
        })
    }

    pub fn module_files(&self) -> impl Iterator<Item = &LoadedFile> {
        self.files
            .iter()
            .filter(|file| file.role() == FileRole::Module)
    }
}

/// 加载项目声明。不 panic；所有失败以 `Diagnostic` 返回。
pub fn load_project(root: impl AsRef<Path>) -> LoadedWorkspace {
    load_project_inner(root.as_ref(), None)
}

/// 使用给定的 Composition 文本替代磁盘文件，用于 Source 编辑的候选验证。
/// 其余项目文件仍从项目根读取；不会写入磁盘。
pub fn load_project_with_composition_text(
    root: impl AsRef<Path>,
    composition_text: &str,
) -> LoadedWorkspace {
    load_project_inner(root.as_ref(), Some(composition_text))
}

fn load_project_inner(requested: &Path, composition_override: Option<&str>) -> LoadedWorkspace {
    let root = fs::canonicalize(requested).unwrap_or_else(|_| requested.to_path_buf());
    let mut workspace = LoadedWorkspace {
        root: root.clone(),
        project: None,
        files: Vec::new(),
        diagnostics: Vec::new(),
        source_digest: None,
    };

    let project_relative = "project.toml";
    let project_path = root.join(project_relative);
    if !project_path.exists() {
        push_path(
            &mut workspace.diagnostics,
            ErrorCode::FileProjectMissing,
            "project.toml not found at the project root",
            project_relative,
            Some(&ObjectId::project()),
        );
        return workspace;
    }
    let project_text = match read_text(&project_path) {
        Ok(text) => text,
        Err((code, message)) => {
            push_path(
                &mut workspace.diagnostics,
                code,
                message,
                project_relative,
                Some(&ObjectId::file(project_relative)),
            );
            return workspace;
        }
    };
    let (project_document, project_failure) = parse_toml(&project_text);
    let project_file = LoadedFile::new(
        project_relative,
        project_text,
        project_document,
        FileRole::Project,
    );
    if let Some(failure) = &project_failure {
        push_at(
            &mut workspace.diagnostics,
            failure.code,
            &failure.message,
            &project_file,
            failure.span.clone(),
            Some(&ObjectId::project()),
        );
    }
    workspace.files.push(project_file);
    if project_failure.is_some() {
        return workspace;
    }

    let project = match decode_project(&workspace.files[0], &mut workspace.diagnostics) {
        Some(project) => project,
        None => return workspace,
    };
    workspace.project = Some(project.clone());

    let composition_relative = project.composition_entry.clone();
    let composition_path = root.join(&composition_relative);
    if !composition_path.exists() {
        push_path(
            &mut workspace.diagnostics,
            ErrorCode::FileCompositionMissing,
            format!("composition entry {composition_relative:?} not found"),
            &composition_relative,
            Some(&ObjectId::file(&composition_relative)),
        );
        return workspace;
    }
    let composition_text = match composition_override {
        Some(text) => text.to_owned(),
        None => match read_text(&composition_path) {
            Ok(text) => text,
            Err((code, message)) => {
                push_path(
                    &mut workspace.diagnostics,
                    code,
                    message,
                    &composition_relative,
                    Some(&ObjectId::file(&composition_relative)),
                );
                return workspace;
            }
        },
    };
    let (composition_document, composition_failure) = parse_toml(&composition_text);
    let composition_file = LoadedFile::new(
        &composition_relative,
        composition_text,
        composition_document,
        FileRole::Composition,
    );
    if let Some(failure) = &composition_failure {
        push_at(
            &mut workspace.diagnostics,
            failure.code,
            &failure.message,
            &composition_file,
            failure.span.clone(),
            Some(&ObjectId::file(&composition_relative)),
        );
    }
    workspace.files.push(composition_file);
    workspace.source_digest = digest_files(&workspace.files);

    let mut digest_blocked = false;
    for module_root in &project.module_roots {
        let module_root_path = root.join(module_root);
        match fs::symlink_metadata(&module_root_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    push_path(
                        &mut workspace.diagnostics,
                        ErrorCode::FilePathEscape,
                        format!("module root {module_root:?} is a symbolic link; not followed"),
                        module_root,
                        Some(&ObjectId::file(module_root)),
                    );
                    continue;
                }
                if !metadata.is_dir() {
                    push_path(
                        &mut workspace.diagnostics,
                        ErrorCode::FileModuleRootMissing,
                        format!("module root {module_root:?} is not a directory"),
                        module_root,
                        Some(&ObjectId::file(module_root)),
                    );
                    continue;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                push_path(
                    &mut workspace.diagnostics,
                    ErrorCode::FileModuleRootMissing,
                    format!("module root {module_root:?} not found"),
                    module_root,
                    Some(&ObjectId::file(module_root)),
                );
                continue;
            }
            Err(error) => {
                push_path(
                    &mut workspace.diagnostics,
                    ErrorCode::FileReadError,
                    format!("cannot inspect module root {module_root:?}: {error}"),
                    module_root,
                    Some(&ObjectId::file(module_root)),
                );
                continue;
            }
        }

        let mut module_paths = Vec::new();
        discover_modules(
            &workspace.root,
            &module_root_path,
            &mut module_paths,
            &mut workspace.diagnostics,
        );
        module_paths.sort();
        for relative in module_paths {
            let module_path = root.join(&relative);
            let text = match read_text(&module_path) {
                Ok(text) => text,
                Err((code, message)) => {
                    digest_blocked = true;
                    push_path(
                        &mut workspace.diagnostics,
                        code,
                        message,
                        &relative,
                        Some(&ObjectId::file(&relative)),
                    );
                    continue;
                }
            };
            let (document, failure) = parse_toml(&text);
            let file = LoadedFile::new(&relative, text, document, FileRole::Module);
            if let Some(failure) = &failure {
                push_at(
                    &mut workspace.diagnostics,
                    failure.code,
                    &failure.message,
                    &file,
                    failure.span.clone(),
                    Some(&ObjectId::file(&relative)),
                );
            }
            workspace.files.push(file);
        }
    }
    workspace.source_digest = if digest_blocked {
        None
    } else {
        digest_files(&workspace.files)
    };
    workspace
}

fn decode_project(file: &LoadedFile, diagnostics: &mut Vec<Diagnostic>) -> Option<Project> {
    let document = file.document()?;
    let root_table = document.as_table();
    let object = ObjectId::project();
    check_unknown(
        root_table,
        &["project", "workspace"],
        file,
        Some(&object),
        diagnostics,
    );

    let project_table = required_table(
        root_table,
        "project",
        file,
        Some(&object),
        "project",
        diagnostics,
    )?;
    check_unknown(
        project_table,
        &["schema_version", "id", "name"],
        file,
        Some(&object),
        diagnostics,
    );
    let workspace_table = required_table(
        root_table,
        "workspace",
        file,
        Some(&object),
        "workspace",
        diagnostics,
    )?;
    check_unknown(
        workspace_table,
        &["composition", "module_roots"],
        file,
        Some(&object),
        diagnostics,
    );

    let version = required_integer(
        project_table,
        "schema_version",
        file,
        Some(&object),
        "project",
        diagnostics,
    )?;
    let schema_version = SchemaVersion::new(u16::try_from(version).unwrap_or(0));
    if !schema_version.is_supported() {
        push_at(
            diagnostics,
            ErrorCode::SchemaUnsupportedVersion,
            format!("unsupported schema_version {version}; v0.1 expects 1"),
            file,
            table_field_span(project_table, "schema_version"),
            Some(&object),
        );
    }

    let id_raw = required_string(
        project_table,
        "id",
        file,
        Some(&object),
        "project",
        diagnostics,
    )?;
    let id = match ModuleId::parse(&id_raw) {
        Ok(id) => id,
        Err(error) => {
            push_at(
                diagnostics,
                error.code(),
                error.message(),
                file,
                table_field_span(project_table, "id"),
                Some(&object),
            );
            return None;
        }
    };
    let name = required_string(
        project_table,
        "name",
        file,
        Some(&object),
        "project",
        diagnostics,
    )?;
    if name.trim().is_empty() {
        push_at(
            diagnostics,
            ErrorCode::SchemaEmptyValue,
            "project.name must not be empty",
            file,
            table_field_span(project_table, "name"),
            Some(&object),
        );
    }

    let composition_raw = required_string(
        workspace_table,
        "composition",
        file,
        Some(&object),
        "workspace",
        diagnostics,
    )?;
    let composition_entry = match normalize_relative_path(&composition_raw) {
        Ok(path) => path,
        Err(code) => {
            push_path_error(
                diagnostics,
                code,
                &composition_raw,
                file,
                table_field_span(workspace_table, "composition"),
                Some(&object),
            );
            return None;
        }
    };
    if !composition_entry.ends_with(".toml") {
        push_at(
            diagnostics,
            ErrorCode::FilePathInvalid,
            format!("composition path {composition_raw:?} must end with .toml"),
            file,
            table_field_span(workspace_table, "composition"),
            Some(&object),
        );
        return None;
    }

    let roots_raw = required_string_array(
        workspace_table,
        "module_roots",
        file,
        Some(&object),
        "workspace",
        diagnostics,
    )?;
    if roots_raw.is_empty() {
        push_at(
            diagnostics,
            ErrorCode::SchemaEmptyValue,
            "workspace.module_roots must not be empty",
            file,
            table_field_span(workspace_table, "module_roots"),
            Some(&object),
        );
        return None;
    }
    let mut module_roots = Vec::with_capacity(roots_raw.len());
    for raw in &roots_raw {
        match normalize_relative_path(raw) {
            Ok(path) => module_roots.push(path),
            Err(code) => {
                push_path_error(
                    diagnostics,
                    code,
                    raw,
                    file,
                    table_field_span(workspace_table, "module_roots"),
                    Some(&object),
                );
                return None;
            }
        }
    }
    let mut seen = BTreeSet::new();
    for root in &module_roots {
        if !seen.insert(root.clone()) {
            push_at(
                diagnostics,
                ErrorCode::SchemaDuplicateEntry,
                format!("duplicate module root {root:?}"),
                file,
                table_field_span(workspace_table, "module_roots"),
                Some(&object),
            );
            return None;
        }
    }
    for (index, outer) in module_roots.iter().enumerate() {
        for inner in module_roots.iter().skip(index + 1) {
            if is_nested_path(outer, inner) || is_nested_path(inner, outer) {
                push_at(
                    diagnostics,
                    ErrorCode::SchemaInconsistency,
                    format!("module roots {outer:?} and {inner:?} must not be nested"),
                    file,
                    table_field_span(workspace_table, "module_roots"),
                    Some(&object),
                );
                return None;
            }
        }
    }

    let mut project = Project::new(schema_version, id, name, composition_entry, module_roots);
    project.source = Some(
        file.source_ref(table_field_span(root_table, "project").or_else(|| project_table.span())),
    );
    Some(project)
}

fn required_table<'a>(
    table: &'a Table,
    key: &str,
    file: &LoadedFile,
    object: Option<&ObjectId>,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<&'a Table> {
    match table.get(key) {
        None => {
            push_at(
                diagnostics,
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                file,
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_table() {
            Some(table) => Some(table),
            None => {
                push_at(
                    diagnostics,
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a table"),
                    file,
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn required_string(
    table: &Table,
    key: &str,
    file: &LoadedFile,
    object: Option<&ObjectId>,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    match table.get(key) {
        None => {
            push_at(
                diagnostics,
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                file,
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_str() {
            Some(value) => Some(value.to_owned()),
            None => {
                push_at(
                    diagnostics,
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be a string"),
                    file,
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn required_integer(
    table: &Table,
    key: &str,
    file: &LoadedFile,
    object: Option<&ObjectId>,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match table.get(key) {
        None => {
            push_at(
                diagnostics,
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                file,
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_integer() {
            Some(value) => Some(value),
            None => {
                push_at(
                    diagnostics,
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be an integer"),
                    file,
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn required_string_array(
    table: &Table,
    key: &str,
    file: &LoadedFile,
    object: Option<&ObjectId>,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Vec<String>> {
    match table.get(key) {
        None => {
            push_at(
                diagnostics,
                ErrorCode::SchemaMissingField,
                format!("{context}.{key} is required"),
                file,
                table.span(),
                object,
            );
            None
        }
        Some(item) => match item.as_array() {
            Some(array) => {
                let mut values = Vec::new();
                let mut valid = true;
                for value in array.iter() {
                    match value.as_str() {
                        Some(text) => values.push(text.to_owned()),
                        None => {
                            push_at(
                                diagnostics,
                                ErrorCode::SchemaTypeMismatch,
                                format!("{context}.{key} entries must be strings"),
                                file,
                                value.span(),
                                object,
                            );
                            valid = false;
                        }
                    }
                }
                if valid {
                    Some(values)
                } else {
                    None
                }
            }
            None => {
                push_at(
                    diagnostics,
                    ErrorCode::SchemaTypeMismatch,
                    format!("{context}.{key} must be an array"),
                    file,
                    item.span(),
                    object,
                );
                None
            }
        },
    }
}

fn check_unknown(
    table: &Table,
    allowed: &[&str],
    file: &LoadedFile,
    object: Option<&ObjectId>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (key, item) in table.iter() {
        if !allowed.contains(&key) {
            push_at(
                diagnostics,
                ErrorCode::SchemaUnknownField,
                format!("unknown field {key:?}"),
                file,
                item.span(),
                object,
            );
        }
    }
}

fn table_field_span(table: &Table, key: &str) -> Option<Range<usize>> {
    table.get(key).and_then(Item::span)
}

fn push_path_error(
    diagnostics: &mut Vec<Diagnostic>,
    code: ErrorCode,
    value: &str,
    file: &LoadedFile,
    span: Option<Range<usize>>,
    object: Option<&ObjectId>,
) {
    push_at(
        diagnostics,
        code,
        format!("invalid path {value:?}"),
        file,
        span,
        object,
    );
}

fn push_at(
    diagnostics: &mut Vec<Diagnostic>,
    code: ErrorCode,
    message: impl AsRef<str>,
    file: &LoadedFile,
    span: Option<Range<usize>>,
    object: Option<&ObjectId>,
) {
    let mut diagnostic =
        Diagnostic::new(code, message.as_ref()).with_source(file.relative_path(), file.range(span));
    if let Some(object) = object {
        diagnostic = diagnostic.with_object_id(object.clone());
    }
    diagnostics.push(diagnostic);
}

fn push_path(
    diagnostics: &mut Vec<Diagnostic>,
    code: ErrorCode,
    message: impl AsRef<str>,
    file: &str,
    object: Option<&ObjectId>,
) {
    let mut diagnostic = Diagnostic::new(code, message.as_ref()).with_source(file, None);
    if let Some(object) = object {
        diagnostic = diagnostic.with_object_id(object.clone());
    }
    diagnostics.push(diagnostic);
}

struct ParseFailure {
    code: ErrorCode,
    message: String,
    span: Option<Range<usize>>,
}

fn parse_toml(text: &str) -> (Option<ImDocument<String>>, Option<ParseFailure>) {
    match ImDocument::parse(text.to_owned()) {
        Ok(document) => (Some(document), None),
        Err(error) => {
            let message = error.to_string();
            let code = if message.to_ascii_lowercase().contains("duplicate") {
                ErrorCode::TomlDuplicateKey
            } else {
                ErrorCode::TomlSyntaxError
            };
            (
                None,
                Some(ParseFailure {
                    code,
                    message,
                    span: error.span(),
                }),
            )
        }
    }
}

fn read_text(path: &Path) -> Result<String, (ErrorCode, String)> {
    let bytes = fs::read(path).map_err(|error| {
        (
            ErrorCode::FileReadError,
            format!("failed to read file: {error}"),
        )
    })?;
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err((ErrorCode::FileReadError, "file has a UTF-8 BOM".to_owned()));
    }
    String::from_utf8(bytes).map_err(|error| {
        (
            ErrorCode::FileReadError,
            format!("file is not valid UTF-8: {error}"),
        )
    })
}

fn digest_files(files: &[LoadedFile]) -> Option<SourceDigest> {
    compute_source_digest(
        files
            .iter()
            .map(|file| (file.relative_path(), file.text().as_bytes())),
    )
}

fn discover_modules(
    project_root: &Path,
    module_root: &Path,
    found: &mut Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let entries = match fs::read_dir(module_root) {
        Ok(entries) => entries,
        Err(error) => {
            let relative = relative_path(project_root, module_root)
                .unwrap_or_else(|| module_root.display().to_string());
            push_path(
                diagnostics,
                ErrorCode::FileReadError,
                format!("cannot read directory {relative}: {error}"),
                &relative,
                Some(&ObjectId::file(&relative)),
            );
            return;
        }
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            discover_modules(project_root, &path, found, diagnostics);
        } else if file_type.is_file() && entry.file_name() == std::ffi::OsStr::new("module.toml") {
            if let Some(relative) = relative_path(project_root, &path) {
                found.push(relative);
            }
        }
    }
}

fn relative_path(project_root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(project_root).ok()?;
    let mut parts = Vec::new();
    for component in relative.components() {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    Some(parts.join("/"))
}
