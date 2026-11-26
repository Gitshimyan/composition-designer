use std::fs;
use std::path::{Path, PathBuf};

use composition_core::{validate_loaded, validate_project, ValidationResult};
use composition_domain::{ConfigValue, Diagnostic, ErrorCode, LocalId, ObjectId, PortRef};
use composition_toml::load_project;
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

use crate::command::{Command, Transaction};
use crate::diff::ModelDiff;
use crate::workspace::{EditorWorkspace, TransactionOutcome};

/// 011 的项目会话：在 010 的模型事务之上做保留格式 TOML 写回。
#[derive(Debug)]
pub struct ProjectSession {
    root: PathBuf,
    composition_entry: String,
    current_text: String,
    document: DocumentMut,
    workspace: EditorWorkspace,
}

impl ProjectSession {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, Vec<Diagnostic>> {
        let root = root.as_ref().to_path_buf();
        let loaded = load_project(&root);
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
            composition_entry,
            current_text,
            document,
            workspace,
        })
    }

    pub fn workspace(&self) -> &EditorWorkspace {
        &self.workspace
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn composition_entry(&self) -> &str {
        &self.composition_entry
    }

    pub fn source_text(&self) -> &str {
        &self.current_text
    }

    pub fn prepare(&self, transaction: &Transaction) -> TransactionOutcome {
        self.workspace.prepare(transaction)
    }

    /// 先在模型上原子提交，再写回保留格式的 TOML；写回或重新校验失败时回滚文件与工作区。
    pub fn apply(&mut self, transaction: &Transaction) -> TransactionOutcome {
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
        match self.commit_document(&new_text) {
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

    fn commit_document(&self, new_text: &str) -> Result<ValidationResult, Vec<Diagnostic>> {
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

        let validation = validate_project(&self.root);
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
        } => {
            let mut table = Table::new();
            set_str(&mut table, "id", id.as_str());
            set_str(&mut table, "module", module.as_str());
            set_str(&mut table, "version", &version.to_string());
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
