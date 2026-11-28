use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use composition_core::{validate_loaded, validate_project_with_profile, ValidationResult};
use composition_domain::{
    CompositionModel, ConfigValue, Diagnostic, ErrorCode, LocalId, ObjectId, PortRef, Revision,
};
use composition_toml::{load_project_with_profile, load_project_with_profile_and_composition_text};
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

use crate::command::{Command, Transaction};
use crate::diff::{ModelDiff, SourceDiff};
use crate::workspace::{EditorWorkspace, TransactionOutcome};

#[derive(Debug, Clone)]
struct SourceCommitted {
    transaction: Transaction,
    model: CompositionModel,
    diagnostics: Vec<Diagnostic>,
    diff: ModelDiff,
    source_diff: SourceDiff,
}

/// 011 的项目会话：在 010 的模型事务之上做保留格式 TOML 写回。
#[derive(Debug)]
pub struct ProjectSession {
    root: PathBuf,
    profile: Option<String>,
    composition_entry: String,
    current_text: String,
    document: DocumentMut,
    workspace: EditorWorkspace,
    source_requests: BTreeMap<String, SourceCommitted>,
}

impl ProjectSession {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, Vec<Diagnostic>> {
        Self::open_with_profile(root, None)
    }

    pub fn open_with_profile(
        root: impl AsRef<Path>,
        profile: Option<&str>,
    ) -> Result<Self, Vec<Diagnostic>> {
        let root = root.as_ref().to_path_buf();
        let mut loaded = load_project_with_profile(&root, profile);
        if loaded.composition_file().is_none() {
            if let Some(project) = loaded.project.as_ref() {
                let entry = project.composition_entry.clone();
                if recover_composition_file(&root, &entry) {
                    loaded = load_project_with_profile(&root, profile);
                }
            }
        }
        let workspace = EditorWorkspace::from_validation(validate_loaded(&loaded))?;
        let Some(project) = loaded.project.as_ref() else {
            return Err(loaded.diagnostics);
        };
        let composition_entry = project.composition_entry.clone();
        let Some(file) = loaded.file(&composition_entry) else {
            return Err(loaded.diagnostics);
        };
        let current_text = file.text().to_owned();
        let document = current_text.parse::<DocumentMut>().map_err(|error| {
            vec![transaction_failed(format!(
                "cannot parse {composition_entry} for editing: {error}"
            ))]
        })?;
        Ok(Self {
            root,
            profile: profile.map(str::to_owned),
            composition_entry,
            current_text,
            document,
            workspace,
            source_requests: BTreeMap::new(),
        })
    }

    pub fn workspace(&self) -> &EditorWorkspace {
        &self.workspace
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    pub fn composition_entry(&self) -> &str {
        &self.composition_entry
    }

    pub fn source_text(&self) -> &str {
        &self.current_text
    }

    pub fn prepare(&self, transaction: &Transaction) -> TransactionOutcome {
        if has_replace_source(transaction) {
            self.prepare_source(transaction)
        } else {
            self.workspace.prepare(transaction)
        }
    }

    /// 先在模型上原子提交，再写回保留格式的 TOML；写回或重新校验失败时回滚文件与工作区。
    pub fn apply(&mut self, transaction: &Transaction) -> TransactionOutcome {
        if has_replace_source(transaction) {
            return self.apply_source(transaction);
        }
        self.apply_object(transaction)
    }

    fn apply_object(&mut self, transaction: &Transaction) -> TransactionOutcome {
        let snapshot_workspace = self.workspace.clone();
        let snapshot_document = self.document.clone();
        let snapshot_text = self.current_text.clone();

        let outcome = self.workspace.apply(transaction);
        if !outcome.is_accepted() || outcome.is_replayed() {
            return outcome;
        }
        let diff = outcome.diff().cloned().unwrap_or_else(ModelDiff::empty);

        let mut document = self.document.clone();
        for command in &transaction.commands {
            if let Err(message) = edit_document(&mut document, command) {
                self.workspace = snapshot_workspace;
                return TransactionOutcome::Rejected {
                    diagnostics: vec![transaction_failed(message)],
                };
            }
        }

        let new_text = document.to_string();
        match self.commit_text(&new_text) {
            Ok(validation) => {
                self.document = document;
                self.current_text = new_text;
                self.workspace.refresh_source(validation);
                let model = self.workspace.model().clone();
                let diagnostics = model.diagnostics.clone();
                TransactionOutcome::Accepted {
                    model: Box::new(model),
                    diagnostics,
                    diff,
                    source_diff: None,
                }
            }
            Err(diagnostics) => {
                self.workspace = snapshot_workspace;
                self.document = snapshot_document;
                self.current_text = snapshot_text;
                TransactionOutcome::Rejected { diagnostics }
            }
        }
    }

    fn prepare_source(&self, transaction: &Transaction) -> TransactionOutcome {
        if let Some(outcome) = self.replay_source(transaction) {
            return outcome;
        }
        if transaction.request_id.trim().is_empty() {
            return rejected_source("transaction request_id must not be empty");
        }
        if transaction.commands.is_empty() {
            return rejected_source("transaction must contain at least one command");
        }
        let source = match source_text(transaction) {
            Ok(source) => source,
            Err(diagnostic) => {
                return TransactionOutcome::Rejected {
                    diagnostics: vec![diagnostic],
                }
            }
        };
        if transaction.commands.len() != 1 {
            return rejected_source("replace_source must be the only command in a transaction");
        }
        if self.workspace.has_request(&transaction.request_id) {
            return rejected_duplicate(&transaction.request_id);
        }

        let current = self.workspace.revision();
        let known_base = self
            .workspace
            .history()
            .iter()
            .any(|record| record.revision == transaction.base_revision);
        if transaction.base_revision != current || !known_base {
            return TransactionOutcome::Rejected {
                diagnostics: vec![Diagnostic::new(
                    ErrorCode::EditRevisionConflict,
                    format!(
                        "base revision seq {} does not match current seq {}",
                        transaction.base_revision.seq, current.seq
                    ),
                )
                .with_object_id(ObjectId::composition())],
            };
        }

        let loaded = load_project_with_profile_and_composition_text(
            &self.root,
            self.profile.as_deref(),
            source,
        );
        let validation = validate_loaded(&loaded);
        if !validation.is_valid() {
            return TransactionOutcome::Rejected {
                diagnostics: validation.diagnostics,
            };
        }
        let mut model = validation
            .model
            .expect("valid source candidate has a model");
        let diff = ModelDiff::between(self.workspace.model(), &model);
        let source_diff = SourceDiff {
            before: self.current_text.clone(),
            after: source.to_owned(),
        };
        model.revision = Some(Revision::new(
            current.seq + 1,
            current.source_digest.clone(),
        ));
        model.diagnostics = validation.diagnostics.clone();
        TransactionOutcome::Accepted {
            model: Box::new(model),
            diagnostics: validation.diagnostics,
            diff,
            source_diff: Some(source_diff),
        }
    }

    fn apply_source(&mut self, transaction: &Transaction) -> TransactionOutcome {
        if let Some(outcome) = self.replay_source(transaction) {
            return outcome;
        }
        let prepared = self.prepare_source(transaction);
        if !prepared.is_accepted() {
            return prepared;
        }
        let Ok(source) = source_text(transaction) else {
            return rejected_source("replace_source must be the only command in a transaction");
        };

        let disk_digest =
            validate_project_with_profile(&self.root, self.profile.as_deref()).source_digest;
        if disk_digest != self.workspace.revision().source_digest {
            return TransactionOutcome::Rejected {
                diagnostics: vec![Diagnostic::new(
                    ErrorCode::EditRevisionConflict,
                    "project changed on disk; reload the session before applying",
                )
                .with_object_id(ObjectId::composition())],
            };
        }

        let document = match source.parse::<DocumentMut>() {
            Ok(document) => document,
            Err(error) => {
                return rejected_source(format!("cannot parse source: {error}"));
            }
        };
        let validation = match self.commit_text(source) {
            Ok(validation) => validation,
            Err(diagnostics) => {
                return TransactionOutcome::Rejected { diagnostics };
            }
        };
        let diff = prepared.diff().cloned().unwrap_or_else(ModelDiff::empty);
        let source_diff = SourceDiff {
            before: self.current_text.clone(),
            after: source.to_owned(),
        };
        self.workspace.record_source_commit(
            validation,
            transaction.request_id.clone(),
            diff.clone(),
        );
        self.document = document;
        self.current_text = source.to_owned();

        let model = self.workspace.model().clone();
        let diagnostics = model.diagnostics.clone();
        self.source_requests.insert(
            transaction.request_id.clone(),
            SourceCommitted {
                transaction: transaction.clone(),
                model: model.clone(),
                diagnostics: diagnostics.clone(),
                diff: diff.clone(),
                source_diff: source_diff.clone(),
            },
        );
        TransactionOutcome::Accepted {
            model: Box::new(model),
            diagnostics,
            diff,
            source_diff: Some(source_diff),
        }
    }

    fn replay_source(&self, transaction: &Transaction) -> Option<TransactionOutcome> {
        if let Some(record) = self.source_requests.get(&transaction.request_id) {
            if record.transaction == *transaction {
                return Some(TransactionOutcome::Replayed {
                    model: Box::new(record.model.clone()),
                    diagnostics: record.diagnostics.clone(),
                    diff: record.diff.clone(),
                    source_diff: Some(record.source_diff.clone()),
                });
            }
            return Some(rejected_duplicate(&transaction.request_id));
        }
        if self.workspace.has_request(&transaction.request_id) {
            return Some(rejected_duplicate(&transaction.request_id));
        }
        None
    }

    fn commit_text(&self, new_text: &str) -> Result<ValidationResult, Vec<Diagnostic>> {
        let path = self.root.join(&self.composition_entry);
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "composition.toml".to_owned());
        let suffix = std::process::id();
        let temp = path.with_file_name(format!(".{file_name}.tmp-{suffix}"));
        let backup = path.with_file_name(format!(".{file_name}.bak-{suffix}"));
        let _ = fs::remove_file(&temp);
        let _ = fs::remove_file(&backup);

        fs::write(&temp, new_text).map_err(|error| {
            vec![transaction_failed(format!(
                "cannot write temporary {}: {error}",
                temp.display()
            ))]
        })?;
        if let Err(error) = fs::rename(&path, &backup) {
            let _ = fs::remove_file(&temp);
            return Err(vec![transaction_failed(format!(
                "cannot back up {}: {error}",
                path.display()
            ))]);
        }
        if let Err(error) = fs::rename(&temp, &path) {
            let _ = fs::rename(&backup, &path);
            let _ = fs::remove_file(&temp);
            return Err(vec![transaction_failed(format!(
                "cannot replace {}: {error}",
                path.display()
            ))]);
        }

        let validation = validate_project_with_profile(&self.root, self.profile.as_deref());
        if !validation.is_valid() {
            let _ = fs::remove_file(&path);
            if let Err(error) = fs::rename(&backup, &path) {
                return Err(vec![transaction_failed(format!(
                    "edited TOML is invalid and rollback failed: {error}"
                ))]);
            }
            let mut diagnostics = validation.diagnostics;
            if diagnostics.is_empty() {
                diagnostics.push(transaction_failed(
                    "edited TOML is invalid without diagnostics",
                ));
            }
            return Err(diagnostics);
        }
        let _ = fs::remove_file(&backup);
        Ok(validation)
    }
}

fn has_replace_source(transaction: &Transaction) -> bool {
    transaction
        .commands
        .iter()
        .any(|command| matches!(command, Command::ReplaceSource { .. }))
}

#[allow(clippy::result_large_err)]
fn source_text(transaction: &Transaction) -> Result<&str, Diagnostic> {
    match transaction.commands.as_slice() {
        [Command::ReplaceSource { source }] => Ok(source),
        _ => Err(Diagnostic::new(
            ErrorCode::EditInvalidCommand,
            "replace_source must be the only command in a transaction",
        )
        .with_object_id(ObjectId::composition())),
    }
}

fn rejected_source(message: impl Into<String>) -> TransactionOutcome {
    TransactionOutcome::Rejected {
        diagnostics: vec![transaction_failed(message)],
    }
}

fn rejected_duplicate(request_id: &str) -> TransactionOutcome {
    TransactionOutcome::Rejected {
        diagnostics: vec![Diagnostic::new(
            ErrorCode::EditDuplicateRequest,
            format!("request_id {request_id:?} was already used with a different payload"),
        )],
    }
}

/// Restore a missing composition entry from an interrupted `.bak-*` write./// Only the newest matching backup is restored; stale `.tmp-*` files are left alone.
fn recover_composition_file(root: &Path, entry: &str) -> bool {
    let path = root.join(entry);
    if path.exists() {
        return false;
    }
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let prefix = format!(".{file_name}.bak-");
    let Ok(entries) = fs::read_dir(parent) else {
        return false;
    };
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(&prefix) {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if newest
            .as_ref()
            .is_none_or(|(current, _)| modified > *current)
        {
            newest = Some((modified, entry.path()));
        }
    }
    let Some((_, backup)) = newest else {
        return false;
    };
    fs::rename(backup, &path).is_ok()
}

fn transaction_failed(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(ErrorCode::EditTransactionFailed, message.into())
        .with_object_id(ObjectId::composition())
}

fn edit_document(document: &mut DocumentMut, command: &Command) -> Result<(), String> {
    match command {
        Command::CreateInstance {
            id,
            module,
            version,
            config,
            scope,
        } => {
            let mut table = Table::new();
            set_str(&mut table, "id", id.as_str());
            set_str(&mut table, "module", module.as_str());
            set_str(&mut table, "version", &version.to_string());
            if let Some(scope) = scope {
                set_str(&mut table, "scope", scope.as_str());
            }
            if !config.is_empty() {
                let mut config_table = Table::new();
                for (key, value) in config {
                    config_table.insert(key.as_str(), config_item(value));
                }
                table.insert("config", Item::Table(config_table));
            }
            instances_mut(document)?.push(table);
        }
        Command::DeleteInstance { id } => {
            let array = instances_mut(document)?;
            let index = find_index(array, id)
                .ok_or_else(|| format!("instance {id} is not present in {}", "composition"))?;
            array.remove(index);
            remove_bindings_referencing(document, id);
            remove_connections_referencing(document, id);
            remove_dependency_providers(document, id);
        }
        Command::RenameInstance { from, to } => {
            let array = instances_mut(document)?;
            let index =
                find_index(array, from).ok_or_else(|| format!("instance {from} is not present"))?;
            let table = array
                .get_mut(index)
                .ok_or_else(|| format!("instance {from} is not present"))?;
            set_str(table, "id", to.as_str());
            if let Some(composition) = document.get_mut("composition").and_then(Item::as_table_mut)
            {
                if field_str(composition, "root") == Some(from.as_str()) {
                    set_str(composition, "root", to.as_str());
                }
            }
            rewrite_binding_references(document, from, to);
            rewrite_connection_references(document, from, to);
            rewrite_dependency_providers(document, from, to);
        }
        Command::UpdateInstanceModule {
            id,
            module,
            version,
        } => {
            let array = instances_mut(document)?;
            let index =
                find_index(array, id).ok_or_else(|| format!("instance {id} is not present"))?;
            let table = array
                .get_mut(index)
                .ok_or_else(|| format!("instance {id} is not present"))?;
            set_str(table, "module", module.as_str());
            set_str(table, "version", &version.to_string());
        }
        Command::SetConfig {
            instance,
            property,
            value,
        } => {
            let table = instance_mut(document, instance)?;
            if table.get("config").is_none() {
                table.insert("config", Item::Table(Table::new()));
            }
            let config = table
                .get_mut("config")
                .and_then(Item::as_table_mut)
                .ok_or_else(|| format!("instance {instance} config is not a table"))?;
            config.insert(property.as_str(), config_item(value));
        }
        Command::UnsetConfig { instance, property } => {
            let table = instance_mut(document, instance)?;
            let remove_config =
                if let Some(config) = table.get_mut("config").and_then(Item::as_table_mut) {
                    config.remove(property.as_str());
                    config.is_empty()
                } else {
                    return Err(format!("instance {instance} has no config table"));
                };
            if remove_config {
                table.remove("config");
            }
        }
        Command::SetDependencyProvider {
            instance,
            requirement,
            provider,
        } => {
            let table = instance_mut(document, instance)?;
            if table.get("dependencies").is_none() {
                table.insert("dependencies", Item::Table(Table::new()));
            }
            let dependencies = table
                .get_mut("dependencies")
                .and_then(Item::as_table_mut)
                .ok_or_else(|| format!("instance {instance} dependencies is not a table"))?;
            dependencies.insert(
                requirement.as_str(),
                Item::Value(Value::from(provider.as_str())),
            );
        }
        Command::UnsetDependencyProvider {
            instance,
            requirement,
        } => {
            let table = instance_mut(document, instance)?;
            let remove_table = if let Some(dependencies) =
                table.get_mut("dependencies").and_then(Item::as_table_mut)
            {
                if dependencies.remove(requirement.as_str()).is_none() {
                    return Err(format!(
                        "instance {instance} has no explicit dependency provider {requirement}"
                    ));
                }
                dependencies.is_empty()
            } else {
                return Err(format!(
                    "instance {instance} has no dependency provider table"
                ));
            };
            if remove_table {
                table.remove("dependencies");
            }
        }
        Command::CreateBinding {
            id,
            parent,
            slot,
            child,
        } => {
            let mut table = Table::new();
            set_str(&mut table, "id", id.as_str());
            set_str(&mut table, "parent", parent.as_str());
            set_str(&mut table, "slot", slot.as_str());
            set_str(&mut table, "child", child.as_str());
            bindings_mut(document)?.push(table);
        }
        Command::UpdateBinding {
            id,
            parent,
            slot,
            child,
        } => {
            let table = binding_mut(document, id)?;
            set_str(table, "parent", parent.as_str());
            set_str(table, "slot", slot.as_str());
            set_str(table, "child", child.as_str());
        }
        Command::DeleteBinding { id } => {
            let array = bindings_mut(document)?;
            let index =
                find_index(array, id).ok_or_else(|| format!("binding {id} is not present"))?;
            array.remove(index);
        }
        Command::CreateConnection { id, from, to } => {
            let mut table = Table::new();
            set_str(&mut table, "id", id.as_str());
            table.insert("from", Item::Table(port_table(from)));
            table.insert("to", Item::Table(port_table(to)));
            connections_mut(document)?.push(table);
        }
        Command::UpdateConnection { id, from, to } => {
            let table = connection_mut(document, id)?;
            table.insert("from", Item::Table(port_table(from)));
            table.insert("to", Item::Table(port_table(to)));
        }
        Command::DeleteConnection { id } => {
            let array = connections_mut(document)?;
            let index =
                find_index(array, id).ok_or_else(|| format!("connection {id} is not present"))?;
            array.remove(index);
        }
        Command::ReplaceSource { .. } => {
            return Err(
                "replace_source must be handled by ProjectSession, not edit_document".to_owned(),
            );
        }
    }
    Ok(())
}

fn instances_mut(document: &mut DocumentMut) -> Result<&mut ArrayOfTables, String> {
    array_mut(document, "instances")
}

fn bindings_mut(document: &mut DocumentMut) -> Result<&mut ArrayOfTables, String> {
    array_mut(document, "bindings")
}

fn connections_mut(document: &mut DocumentMut) -> Result<&mut ArrayOfTables, String> {
    array_mut(document, "connections")
}

fn array_mut<'a>(
    document: &'a mut DocumentMut,
    key: &str,
) -> Result<&'a mut ArrayOfTables, String> {
    let table = document.as_table_mut();
    if !table.contains_key(key) {
        table.insert(key, Item::ArrayOfTables(ArrayOfTables::new()));
    }
    table
        .get_mut(key)
        .and_then(Item::as_array_of_tables_mut)
        .ok_or_else(|| format!("{key} is not an array of tables"))
}

fn instance_mut<'a>(document: &'a mut DocumentMut, id: &LocalId) -> Result<&'a mut Table, String> {
    let array = instances_mut(document)?;
    let index = find_index(array, id).ok_or_else(|| format!("instance {id} is not present"))?;
    array
        .get_mut(index)
        .ok_or_else(|| format!("instance {id} is not present"))
}

fn binding_mut<'a>(document: &'a mut DocumentMut, id: &LocalId) -> Result<&'a mut Table, String> {
    let array = bindings_mut(document)?;
    let index = find_index(array, id).ok_or_else(|| format!("binding {id} is not present"))?;
    array
        .get_mut(index)
        .ok_or_else(|| format!("binding {id} is not present"))
}

fn connection_mut<'a>(
    document: &'a mut DocumentMut,
    id: &LocalId,
) -> Result<&'a mut Table, String> {
    let array = connections_mut(document)?;
    let index = find_index(array, id).ok_or_else(|| format!("connection {id} is not present"))?;
    array
        .get_mut(index)
        .ok_or_else(|| format!("connection {id} is not present"))
}

fn find_index(array: &ArrayOfTables, id: &LocalId) -> Option<usize> {
    array
        .iter()
        .position(|table| field_str(table, "id") == Some(id.as_str()))
}

fn field_str<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    table.get(key).and_then(Item::as_str)
}

fn set_str(table: &mut Table, key: &str, value: &str) {
    table.insert(key, Item::Value(Value::from(value)));
}

fn config_item(value: &ConfigValue) -> Item {
    Item::Value(config_value(value))
}

fn config_value(value: &ConfigValue) -> Value {
    match value {
        ConfigValue::String(value) => Value::from(value.as_str()),
        ConfigValue::Integer(value) => Value::from(*value),
        ConfigValue::Float(value) => Value::from(*value),
        ConfigValue::Boolean(value) => Value::from(*value),
        ConfigValue::Array(values) => {
            let mut array = Array::new();
            for value in values {
                array.push(config_value(value));
            }
            Value::Array(array)
        }
    }
}

fn port_table(port: &PortRef) -> Table {
    let mut table = Table::new();
    set_str(&mut table, "instance", port.instance.as_str());
    set_str(&mut table, "port", port.port.as_str());
    table
}

fn remove_bindings_referencing(document: &mut DocumentMut, id: &LocalId) {
    if let Some(bindings) = document
        .get_mut("bindings")
        .and_then(Item::as_array_of_tables_mut)
    {
        let mut index = 0;
        while index < bindings.len() {
            let remove = bindings.get(index).is_some_and(|table| {
                field_str(table, "parent") == Some(id.as_str())
                    || field_str(table, "child") == Some(id.as_str())
            });
            if remove {
                bindings.remove(index);
            } else {
                index += 1;
            }
        }
    }
}

fn remove_connections_referencing(document: &mut DocumentMut, id: &LocalId) {
    if let Some(connections) = document
        .get_mut("connections")
        .and_then(Item::as_array_of_tables_mut)
    {
        let mut index = 0;
        while index < connections.len() {
            let remove = connections.get(index).is_some_and(|table| {
                endpoint_instance(table, "from") == Some(id.as_str())
                    || endpoint_instance(table, "to") == Some(id.as_str())
            });
            if remove {
                connections.remove(index);
            } else {
                index += 1;
            }
        }
    }
}

fn remove_dependency_providers(document: &mut DocumentMut, id: &LocalId) {
    if let Some(instances) = document
        .get_mut("instances")
        .and_then(Item::as_array_of_tables_mut)
    {
        for index in 0..instances.len() {
            if let Some(table) = instances.get_mut(index) {
                if let Some(deps) = table.get_mut("dependencies").and_then(Item::as_table_mut) {
                    let keys: Vec<String> = deps
                        .iter()
                        .filter(|(_, value)| value.as_str() == Some(id.as_str()))
                        .map(|(key, _)| key.to_owned())
                        .collect();
                    for key in keys {
                        deps.remove(&key);
                    }
                }
            }
        }
    }
}

fn rewrite_binding_references(document: &mut DocumentMut, from: &LocalId, to: &LocalId) {
    if let Some(bindings) = document
        .get_mut("bindings")
        .and_then(Item::as_array_of_tables_mut)
    {
        for index in 0..bindings.len() {
            if let Some(table) = bindings.get_mut(index) {
                if field_str(table, "parent") == Some(from.as_str()) {
                    set_str(table, "parent", to.as_str());
                }
                if field_str(table, "child") == Some(from.as_str()) {
                    set_str(table, "child", to.as_str());
                }
            }
        }
    }
}

fn rewrite_connection_references(document: &mut DocumentMut, from: &LocalId, to: &LocalId) {
    if let Some(connections) = document
        .get_mut("connections")
        .and_then(Item::as_array_of_tables_mut)
    {
        for index in 0..connections.len() {
            if let Some(table) = connections.get_mut(index) {
                rewrite_endpoint(table, "from", from, to);
                rewrite_endpoint(table, "to", from, to);
            }
        }
    }
}

fn rewrite_endpoint(table: &mut Table, key: &str, from: &LocalId, to: &LocalId) {
    if let Some(endpoint) = table.get_mut(key).and_then(Item::as_table_mut) {
        if field_str(endpoint, "instance") == Some(from.as_str()) {
            set_str(endpoint, "instance", to.as_str());
        }
    }
}

fn rewrite_dependency_providers(document: &mut DocumentMut, from: &LocalId, to: &LocalId) {
    if let Some(instances) = document
        .get_mut("instances")
        .and_then(Item::as_array_of_tables_mut)
    {
        for index in 0..instances.len() {
            if let Some(table) = instances.get_mut(index) {
                if let Some(deps) = table.get_mut("dependencies").and_then(Item::as_table_mut) {
                    let keys: Vec<String> = deps
                        .iter()
                        .filter(|(_, value)| value.as_str() == Some(from.as_str()))
                        .map(|(key, _)| key.to_owned())
                        .collect();
                    for key in keys {
                        deps.insert(&key, Item::Value(Value::from(to.as_str())));
                    }
                }
            }
        }
    }
}

fn endpoint_instance<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    table
        .get(key)
        .and_then(Item::as_table)
        .and_then(|endpoint| field_str(endpoint, "instance"))
}
