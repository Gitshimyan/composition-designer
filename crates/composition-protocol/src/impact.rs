use composition_domain::{
    CompositionModel, DependencyRequirement, DependencyStatus, DependencyTarget,
    DependencyTargetInfo, Diagnostic, InstanceScope, InterfaceDeclaration, LocalId, ModuleId,
    ModuleKind, ModuleRef, Version,
};
use composition_editor::ModelDiff;
use serde::Serialize;

use crate::context::{find_module, target_of};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImpactView {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<ModuleImpact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<InstanceImpact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<TransactionImpact>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModuleImpact {
    pub module: ModuleHead,
    pub instances: Vec<LocalId>,
    pub interface_implementers: Vec<InterfaceImplementers>,
    pub dependents: Vec<Dependent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModuleHead {
    pub id: ModuleId,
    pub version: Version,
    pub kind: ModuleKind,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InterfaceImplementers {
    pub interface: InterfaceDeclaration,
    pub instances: Vec<LocalId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Dependent {
    pub instance: LocalId,
    pub requirement: LocalId,
    pub target: DependencyTargetInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstanceImpact {
    pub instance: InstanceHead,
    pub parent_bindings: Vec<BindingView>,
    pub child_bindings: Vec<BindingView>,
    pub connections: Vec<ConnectionView>,
    pub requires: Vec<RequiresView>,
    pub provides_for: Vec<ProvidesView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstanceHead {
    pub id: LocalId,
    pub module: ModuleRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<InstanceScope>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BindingView {
    pub id: LocalId,
    pub parent: LocalId,
    pub slot: LocalId,
    pub child: LocalId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectionView {
    pub id: LocalId,
    pub role: String,
    pub port: LocalId,
    pub peer_instance: LocalId,
    pub peer_port: LocalId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequiresView {
    pub requirement: LocalId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<DependencyTargetInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<LocalId>,
    pub optional: bool,
    pub status: DependencyStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProvidesView {
    pub instance: LocalId,
    pub requirement: LocalId,
    pub explicit: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionImpact {
    pub accepted: bool,
    pub replayed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<ModelDiff>,
    pub changed_objects: Vec<String>,
}

impl ImpactView {
    pub(crate) fn for_module(
        model: &CompositionModel,
        id: &ModuleId,
        version: &Version,
    ) -> Option<Self> {
        let module = model
            .modules
            .iter()
            .find(|module| &module.id == id && &module.version == version)?;
        let instances: Vec<LocalId> = model
            .composition
            .instances
            .iter()
            .filter(|instance| &instance.module.id == id && &instance.module.version == version)
            .map(|instance| instance.id.clone())
            .collect();

        let interface_implementers = module
            .contract
            .interfaces()
            .iter()
            .map(|interface| InterfaceImplementers {
                interface: interface.clone(),
                instances: model
                    .composition
                    .instances
                    .iter()
                    .filter(|instance| {
                        find_module(model, &instance.module).is_some_and(|candidate| {
                            candidate.contract.interfaces().iter().any(|declaration| {
                                declaration.id == interface.id
                                    && declaration.version == interface.version
                            })
                        })
                    })
                    .map(|instance| instance.id.clone())
                    .collect(),
            })
            .collect();

        let mut dependents = Vec::new();
        for instance in &model.composition.instances {
            let Some(candidate) = find_module(model, &instance.module) else {
                continue;
            };
            for (requirement_id, requirement) in candidate.contract.dependencies() {
                if requirement_matches_module(requirement, module) {
                    dependents.push(Dependent {
                        instance: instance.id.clone(),
                        requirement: requirement_id.clone(),
                        target: target_of(requirement),
                    });
                }
            }
        }

        let prefixes = module_prefixes(module);
        let diagnostics = matching_diagnostics(model, &prefixes, &[]);
        Some(Self {
            kind: "module".to_owned(),
            module: Some(ModuleImpact {
                module: ModuleHead {
                    id: module.id.clone(),
                    version: module.version,
                    kind: module.kind,
                },
                instances,
                interface_implementers,
                dependents,
            }),
            instance: None,
            transaction: None,
            diagnostics,
        })
    }

    pub(crate) fn for_instance(model: &CompositionModel, id: &LocalId) -> Option<Self> {
        let instance = model
            .composition
            .instances
            .iter()
            .find(|candidate| &candidate.id == id)?;
        let module = find_module(model, &instance.module);

        let mut parent_bindings = Vec::new();
        let mut child_bindings = Vec::new();
        let mut related_bindings = Vec::new();
        for binding in &model.composition.bindings {
            let view = BindingView {
                id: binding.id.clone(),
                parent: binding.parent.clone(),
                slot: binding.slot.clone(),
                child: binding.child.clone(),
            };
            let mut related = false;
            if &binding.child == id {
                parent_bindings.push(BindingView {
                    id: view.id.clone(),
                    parent: view.parent.clone(),
                    slot: view.slot.clone(),
                    child: view.child.clone(),
                });
                related = true;
            }
            if &binding.parent == id {
                child_bindings.push(view);
                related = true;
            }
            if related {
                related_bindings.push(binding.id.clone());
            }
        }

        let mut connections = Vec::new();
        let mut related_connections = Vec::new();
        for connection in &model.composition.connections {
            if &connection.from.instance == id {
                connections.push(ConnectionView {
                    id: connection.id.clone(),
                    role: "from".to_owned(),
                    port: connection.from.port.clone(),
                    peer_instance: connection.to.instance.clone(),
                    peer_port: connection.to.port.clone(),
                });
                related_connections.push(connection.id.clone());
            }
            if &connection.to.instance == id {
                connections.push(ConnectionView {
                    id: connection.id.clone(),
                    role: "to".to_owned(),
                    port: connection.to.port.clone(),
                    peer_instance: connection.from.instance.clone(),
                    peer_port: connection.from.port.clone(),
                });
                related_connections.push(connection.id.clone());
            }
        }

        let mut requires = Vec::new();
        for resolution in &model.dependencies {
            if &resolution.from_instance != id {
                continue;
            }
            let target = module
                .and_then(|module| module.contract.dependency(&resolution.requirement_id))
                .map(target_of);
            requires.push(RequiresView {
                requirement: resolution.requirement_id.clone(),
                target,
                provider: resolution.provider_instance.clone(),
                optional: resolution.optional,
                status: resolution.status,
            });
        }

        let mut provides_for: Vec<ProvidesView> = model
            .dependencies
            .iter()
            .filter(|resolution| resolution.provider_instance.as_ref() == Some(id))
            .filter(|resolution| {
                !model
                    .composition
                    .instances
                    .iter()
                    .find(|candidate| candidate.id == resolution.from_instance)
                    .is_some_and(|candidate| {
                        candidate
                            .dependency_providers
                            .get(&resolution.requirement_id)
                            == Some(id)
                    })
            })
            .map(|resolution| ProvidesView {
                instance: resolution.from_instance.clone(),
                requirement: resolution.requirement_id.clone(),
                explicit: false,
            })
            .collect();
        for consumer in &model.composition.instances {
            for (requirement, provider) in &consumer.dependency_providers {
                if provider == id {
                    provides_for.push(ProvidesView {
                        instance: consumer.id.clone(),
                        requirement: requirement.clone(),
                        explicit: true,
                    });
                }
            }
        }
        provides_for.sort_by(|left, right| {
            left.instance
                .cmp(&right.instance)
                .then_with(|| left.requirement.cmp(&right.requirement))
        });
        provides_for.dedup_by(|left, right| {
            left.instance == right.instance && left.requirement == right.requirement
        });

        let object_prefix = format!("instance:{id}");
        let mut prefixes = vec![object_prefix.clone()];
        for binding in related_bindings {
            prefixes.push(format!("binding:{binding}"));
        }
        for connection in related_connections {
            prefixes.push(format!("connection:{connection}"));
        }
        let diagnostics = matching_diagnostics(model, &prefixes, &[&object_prefix]);

        Some(Self {
            kind: "instance".to_owned(),
            module: None,
            instance: Some(InstanceImpact {
                instance: InstanceHead {
                    id: instance.id.clone(),
                    module: instance.module.clone(),
                    scope: instance.scope,
                },
                parent_bindings,
                child_bindings,
                connections,
                requires,
                provides_for,
            }),
            transaction: None,
            diagnostics,
        })
    }

    pub(crate) fn for_transaction(
        accepted: bool,
        replayed: bool,
        diff: Option<ModelDiff>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        let changed_objects = diff
            .as_ref()
            .map(|diff| {
                diff.changes
                    .iter()
                    .map(|change| change.object_id.as_str().to_owned())
                    .collect()
            })
            .unwrap_or_default();
        Self {
            kind: "transaction".to_owned(),
            module: None,
            instance: None,
            transaction: Some(TransactionImpact {
                accepted,
                replayed,
                diff,
                changed_objects,
            }),
            diagnostics,
        }
    }
}

fn requirement_matches_module(
    requirement: &DependencyRequirement,
    module: &composition_domain::ModuleDefinition,
) -> bool {
    match &requirement.target {
        DependencyTarget::Module(id) => {
            id == &module.id
                && requirement
                    .version
                    .is_none_or(|version| version == module.version)
        }
        DependencyTarget::Interface(id) => module.contract.interfaces().iter().any(|declaration| {
            &declaration.id == id
                && requirement
                    .version
                    .is_none_or(|version| version == declaration.version)
        }),
    }
}

fn module_prefixes(module: &composition_domain::ModuleDefinition) -> Vec<String> {
    let base = format!("module:{}@{}", module.id, module.version);
    vec![
        base.clone(),
        format!("property:{}@{}#", module.id, module.version),
        format!("slot:{}@{}#", module.id, module.version),
        format!("port:{}@{}#", module.id, module.version),
        format!("dependency:{}@{}#", module.id, module.version),
    ]
}

fn matching_diagnostics(
    model: &CompositionModel,
    prefixes: &[String],
    related: &[&str],
) -> Vec<Diagnostic> {
    model
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.object_id.as_ref().is_some_and(|object| {
                prefixes
                    .iter()
                    .any(|prefix| object.as_str().starts_with(prefix))
            }) || diagnostic
                .related
                .iter()
                .any(|object| related.iter().any(|item| object.as_str() == *item))
        })
        .cloned()
        .collect()
}
