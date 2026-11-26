use std::collections::BTreeMap;

use composition_core::{validate_model, validate_project, SourceMap, ValidationResult};
use composition_domain::{
    Binding, CompositionModel, Diagnostic, ErrorCode, Instance, ModuleRef, ObjectId, PortRef,
    Revision,
};
use serde::Serialize;

use crate::command::{Command, Transaction};
use crate::diff::ModelDiff;

/// 已提交 Revision 的历史记录。010 仅保存在工作区内存中；011 可随 TOML 一起持久化。
#[derive(Debug, Clone, Serialize)]
pub struct RevisionRecord {
    pub revision: Revision,
    pub request_id: String,
    pub diff: ModelDiff,
}

#[derive(Debug, Clone)]
struct CommittedRequest {
    transaction: Transaction,
    model: CompositionModel,
    diagnostics: Vec<Diagnostic>,
    diff: ModelDiff,
}

/// 编辑器工作区：已验证模型 + 来源映射 + Revision 历史/request 记录。
#[derive(Debug)]
pub struct EditorWorkspace {
    model: CompositionModel,
    source_map: SourceMap,
    history: Vec<RevisionRecord>,
    requests: BTreeMap<String, CommittedRequest>,
}

impl EditorWorkspace {
    pub fn from_validation(result: ValidationResult) -> Result<Self, Vec<Diagnostic>> {
        if result.model.is_none() || result.diagnostics.iter().any(Diagnostic::is_error) {
            return Err(result.diagnostics);
        }
        let mut model = result.model.expect("checked above");
        let revision = model
            .revision
            .clone()
            .unwrap_or_else(|| Revision::new(0, None));
        model.revision = Some(revision.clone());
        Ok(Self {
            model,
            source_map: result.source_map,
            history: vec![RevisionRecord {
                revision,
                request_id: "initial".to_owned(),
                diff: ModelDiff::empty(),
            }],
            requests: BTreeMap::new(),
        })
    }

    pub fn open(root: impl AsRef<std::path::Path>) -> Result<Self, Vec<Diagnostic>> {
        Self::from_validation(validate_project(root))
    }

    pub fn model(&self) -> &CompositionModel {
        &self.model
    }

    pub fn revision(&self) -> Revision {
        self.model
            .revision
            .clone()
            .unwrap_or_else(|| Revision::new(0, None))
    }

    pub fn history(&self) -> &[RevisionRecord] {
        &self.history
    }

    /// 原子应用事务：失败时工作区完全不变；成功时更新模型、`seq` 与历史。
    ///
    /// 010 不写 TOML，因此 `source_digest` 保持读取时的值；011 在落盘后重算。
    pub fn apply(&mut self, transaction: &Transaction) -> TransactionOutcome {
        let outcome = self.evaluate(transaction);
        if let TransactionOutcome::Accepted {
            model,
            diagnostics,
            diff,
        } = &outcome
        {
            self.model = model.as_ref().clone();
            let revision = self.revision();
            self.history.push(RevisionRecord {
                revision,
                request_id: transaction.request_id.clone(),
                diff: diff.clone(),
            });
            self.requests.insert(
                transaction.request_id.clone(),
                CommittedRequest {
                    transaction: transaction.clone(),
                    model: self.model.clone(),
                    diagnostics: diagnostics.clone(),
                    diff: diff.clone(),
                },
            );
        }
        outcome
    }

    /// 纯规划：不改变工作区，返回候选模型、Diff 或拒绝诊断。
    pub fn prepare(&self, transaction: &Transaction) -> TransactionOutcome {
        self.evaluate(transaction)
    }

    fn evaluate(&self, transaction: &Transaction) -> TransactionOutcome {
        if transaction.request_id.trim().is_empty() {
            return rejected(invalid_command(
                "transaction request_id must not be empty",
                ObjectId::composition(),
            ));
        }
        if transaction.commands.is_empty() {
            return rejected(invalid_command(
                "transaction must contain at least one command",
                ObjectId::composition(),
            ));
        }

        if let Some(record) = self.requests.get(&transaction.request_id) {
            if &record.transaction == transaction {
                return TransactionOutcome::Replayed {
                    model: Box::new(record.model.clone()),
                    diagnostics: record.diagnostics.clone(),
                    diff: record.diff.clone(),
                };
            }
            return rejected(Diagnostic::new(
                ErrorCode::EditDuplicateRequest,
                format!(
                    "request_id {:?} was already used with a different payload",
                    transaction.request_id
                ),
            ));
        }

        let current = self.revision();
        let known_base = self
            .history
            .iter()
            .any(|record| record.revision == transaction.base_revision);
        if transaction.base_revision != current || !known_base {
            return rejected(Diagnostic::new(
                ErrorCode::EditRevisionConflict,
                format!(
                    "base revision seq {} does not match current seq {}",
                    transaction.base_revision.seq, current.seq
                ),
            ));
        }

        let mut candidate = self.model.clone();
        for command in &transaction.commands {
            if let Err(diagnostic) = apply_command(&mut candidate, command) {
                return rejected(diagnostic);
            }
        }

        let result = validate_model(
            &candidate.project,
            &candidate.composition,
            &candidate.modules,
            self.source_map.clone(),
            current.source_digest.clone(),
        );
        if !result.is_valid() {
            return rejected_many(result.diagnostics);
        }
        let mut model = result.model.expect("validated model");
        let diff = ModelDiff::between(&self.model, &model);
        model.revision = Some(Revision::new(
            current.seq + 1,
            current.source_digest.clone(),
        ));
        model.diagnostics = result.diagnostics.clone();
        TransactionOutcome::Accepted {
            model: Box::new(model),
            diagnostics: result.diagnostics,
            diff,
        }
    }
}

#[derive(Debug)]
pub enum TransactionOutcome {
    Accepted {
        model: Box<CompositionModel>,
        diagnostics: Vec<Diagnostic>,
        diff: ModelDiff,
    },
    Replayed {
        model: Box<CompositionModel>,
        diagnostics: Vec<Diagnostic>,
        diff: ModelDiff,
    },
    Rejected {
        diagnostics: Vec<Diagnostic>,
    },
}

impl TransactionOutcome {
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted { .. } | Self::Replayed { .. })
    }

    pub fn is_replayed(&self) -> bool {
        matches!(self, Self::Replayed { .. })
    }

    pub fn model(&self) -> Option<&CompositionModel> {
        match self {
            Self::Accepted { model, .. } | Self::Replayed { model, .. } => Some(model.as_ref()),
            Self::Rejected { .. } => None,
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        match self {
            Self::Accepted { diagnostics, .. }
            | Self::Replayed { diagnostics, .. }
            | Self::Rejected { diagnostics } => diagnostics,
        }
    }

    pub fn diff(&self) -> Option<&ModelDiff> {
        match self {
            Self::Accepted { diff, .. } | Self::Replayed { diff, .. } => Some(diff),
            Self::Rejected { .. } => None,
        }
    }
}

fn rejected(diagnostic: Diagnostic) -> TransactionOutcome {
    TransactionOutcome::Rejected {
        diagnostics: vec![diagnostic],
    }
}

fn rejected_many(diagnostics: Vec<Diagnostic>) -> TransactionOutcome {
    TransactionOutcome::Rejected { diagnostics }
}

fn invalid_command(message: impl Into<String>, object: ObjectId) -> Diagnostic {
    Diagnostic::new(ErrorCode::EditInvalidCommand, message.into()).with_object_id(object)
}

#[allow(clippy::result_large_err)]
fn apply_command(model: &mut CompositionModel, command: &Command) -> Result<(), Diagnostic> {
    match command {
        Command::CreateInstance {
            id,
            module,
            version,
            config,
        } => {
            if model
                .composition
                .instances
                .iter()
                .any(|instance| &instance.id == id)
            {
                return Err(invalid_command(
                    format!("instance {id} already exists"),
                    ObjectId::instance(id),
                ));
            }
            let mut instance = Instance::new(id.clone(), ModuleRef::new(module.clone(), *version));
            instance.config = config.clone();
            model.composition.instances.push(instance);
        }
        Command::DeleteInstance { id } => {
            if &model.composition.root == id {
                return Err(invalid_command(
                    format!("cannot delete root instance {id}"),
                    ObjectId::instance(id),
                ));
            }
            let Some(index) = model
                .composition
                .instances
                .iter()
                .position(|instance| &instance.id == id)
            else {
                return Err(invalid_command(
                    format!("instance {id} does not exist"),
                    ObjectId::instance(id),
                ));
            };
            model.composition.instances.remove(index);
            model
                .composition
                .bindings
                .retain(|binding| &binding.parent != id && &binding.child != id);
            model.composition.connections.retain(|connection| {
                &connection.from.instance != id && &connection.to.instance != id
            });
            for instance in &mut model.composition.instances {
                instance
                    .dependency_providers
                    .retain(|_, provider| provider != id);
            }
        }
        Command::RenameInstance { from, to } => {
            if from == to {
                return Err(invalid_command(
                    "rename source and target are identical",
                    ObjectId::instance(from),
                ));
            }
            if model
                .composition
                .instances
                .iter()
                .any(|instance| &instance.id == to)
            {
                return Err(invalid_command(
                    format!("instance {to} already exists"),
                    ObjectId::instance(to),
                ));
            }
            let Some(instance) = model
                .composition
                .instances
                .iter_mut()
                .find(|instance| &instance.id == from)
            else {
                return Err(invalid_command(
                    format!("instance {from} does not exist"),
                    ObjectId::instance(from),
                ));
            };
            instance.id = to.clone();
            if &model.composition.root == from {
                model.composition.root = to.clone();
            }
            for binding in &mut model.composition.bindings {
                if &binding.parent == from {
                    binding.parent = to.clone();
                }
                if &binding.child == from {
                    binding.child = to.clone();
                }
            }
            for connection in &mut model.composition.connections {
                if &connection.from.instance == from {
                    connection.from.instance = to.clone();
                }
                if &connection.to.instance == from {
                    connection.to.instance = to.clone();
                }
            }
            for instance in &mut model.composition.instances {
                for provider in instance.dependency_providers.values_mut() {
                    if provider == from {
                        *provider = to.clone();
                    }
                }
            }
        }
        Command::UpdateInstanceModule {
            id,
            module,
            version,
        } => {
            let Some(instance) = model
                .composition
                .instances
                .iter_mut()
                .find(|instance| &instance.id == id)
            else {
                return Err(invalid_command(
                    format!("instance {id} does not exist"),
                    ObjectId::instance(id),
                ));
            };
            instance.module = ModuleRef::new(module.clone(), *version);
        }
        Command::SetConfig {
            instance,
            property,
            value,
        } => {
            let Some(instance) = model
                .composition
                .instances
                .iter_mut()
                .find(|item| &item.id == instance)
            else {
                return Err(invalid_command(
                    format!("instance {instance} does not exist"),
                    ObjectId::instance(instance),
                ));
            };
            instance.config.insert(property.clone(), value.clone());
        }
        Command::UnsetConfig { instance, property } => {
            let instance_id = instance.clone();
            let Some(instance) = model
                .composition
                .instances
                .iter_mut()
                .find(|item| item.id == instance_id)
            else {
                return Err(invalid_command(
                    format!("instance {instance_id} does not exist"),
                    ObjectId::instance(&instance_id),
                ));
            };
            if instance.config.remove(property).is_none() {
                return Err(invalid_command(
                    format!("instance {instance_id} has no property {property}"),
                    ObjectId::instance(&instance_id),
                ));
            }
        }
        Command::CreateBinding {
            id,
            parent,
            slot,
            child,
        } => {
            if model
                .composition
                .bindings
                .iter()
                .any(|binding| &binding.id == id)
            {
                return Err(invalid_command(
                    format!("binding {id} already exists"),
                    ObjectId::binding(id),
                ));
            }
            model.composition.bindings.push(Binding::new(
                id.clone(),
                parent.clone(),
                slot.clone(),
                child.clone(),
            ));
        }
        Command::UpdateBinding {
            id,
            parent,
            slot,
            child,
        } => {
            let Some(binding) = model
                .composition
                .bindings
                .iter_mut()
                .find(|binding| &binding.id == id)
            else {
                return Err(invalid_command(
                    format!("binding {id} does not exist"),
                    ObjectId::binding(id),
                ));
            };
            binding.parent = parent.clone();
            binding.slot = slot.clone();
            binding.child = child.clone();
        }
        Command::DeleteBinding { id } => {
            let Some(index) = model
                .composition
                .bindings
                .iter()
                .position(|binding| &binding.id == id)
            else {
                return Err(invalid_command(
                    format!("binding {id} does not exist"),
                    ObjectId::binding(id),
                ));
            };
            model.composition.bindings.remove(index);
        }
        Command::CreateConnection { id, from, to } => {
            if model
                .composition
                .connections
                .iter()
                .any(|connection| &connection.id == id)
            {
                return Err(invalid_command(
                    format!("connection {id} already exists"),
                    ObjectId::connection(id),
                ));
            }
            model
                .composition
                .connections
                .push(composition_domain::Connection::new(
                    id.clone(),
                    PortRef::new(from.instance.clone(), from.port.clone()),
                    PortRef::new(to.instance.clone(), to.port.clone()),
                ));
        }
        Command::UpdateConnection { id, from, to } => {
            let Some(connection) = model
                .composition
                .connections
                .iter_mut()
                .find(|connection| &connection.id == id)
            else {
                return Err(invalid_command(
                    format!("connection {id} does not exist"),
                    ObjectId::connection(id),
                ));
            };
            connection.from = PortRef::new(from.instance.clone(), from.port.clone());
            connection.to = PortRef::new(to.instance.clone(), to.port.clone());
        }
        Command::DeleteConnection { id } => {
            let Some(index) = model
                .composition
                .connections
                .iter()
                .position(|connection| &connection.id == id)
            else {
                return Err(invalid_command(
                    format!("connection {id} does not exist"),
                    ObjectId::connection(id),
                ));
            };
            model.composition.connections.remove(index);
        }
    }
    Ok(())
}
