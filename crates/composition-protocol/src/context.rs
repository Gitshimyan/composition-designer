use composition_domain::{
    CompositionModel, DependencyRequirement, DependencyTarget, DependencyTargetInfo,
    DependencyTargetKind, Diagnostic, InstanceScope, InterfaceDeclaration, LocalId, ModuleId,
    ModuleKind, ModuleRef, Revision, SchemaVersion, Version,
};
use serde::Serialize;

/// Compact agent context: project identity, module/instance indexes, diagnostics and Revision.
/// Config values stay in `resolve`; this view is intentionally summary-only.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextView {
    pub project: Option<ContextProject>,
    pub counts: ContextCounts,
    pub modules: Vec<ContextModule>,
    pub instances: Vec<ContextInstance>,
    pub diagnostics: Vec<Diagnostic>,
    pub revision: Option<Revision>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextProject {
    pub id: ModuleId,
    pub name: String,
    pub schema_version: SchemaVersion,
    pub root: LocalId,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextCounts {
    pub modules: usize,
    pub instances: usize,
    pub structure_instances: usize,
    pub services: usize,
    pub bindings: usize,
    pub connections: usize,
    pub dependencies: usize,
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextModule {
    pub id: ModuleId,
    pub version: Version,
    pub kind: ModuleKind,
    pub interfaces: Vec<InterfaceDeclaration>,
    pub dependencies: Vec<ContextDependency>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextDependency {
    pub id: LocalId,
    pub target: DependencyTargetInfo,
    pub optional: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextInstance {
    pub id: LocalId,
    pub module: ModuleRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<InstanceScope>,
    pub interfaces: Vec<InterfaceDeclaration>,
    pub config_keys: Vec<String>,
    pub provider_keys: Vec<String>,
}

impl ContextView {
    pub(crate) fn build(model: Option<&CompositionModel>, diagnostics: &[Diagnostic]) -> Self {
        let Some(model) = model else {
            return Self {
                project: None,
                counts: ContextCounts::default(),
                modules: Vec::new(),
                instances: Vec::new(),
                diagnostics: diagnostics.to_vec(),
                revision: None,
            };
        };

        let composition = &model.composition;
        let mut counts = ContextCounts {
            modules: model.modules.len(),
            instances: composition.instances.len(),
            bindings: composition.bindings.len(),
            connections: composition.connections.len(),
            dependencies: model.dependencies.len(),
            errors: diagnostics.iter().filter(|d| d.is_error()).count(),
            warnings: diagnostics
                .iter()
                .filter(|d| d.severity == composition_domain::Severity::Warning)
                .count(),
            ..ContextCounts::default()
        };
        for instance in &composition.instances {
            if instance.scope == Some(InstanceScope::Service) {
                counts.services += 1;
            } else {
                counts.structure_instances += 1;
            }
        }

        let modules = model
            .modules
            .iter()
            .map(|module| ContextModule {
                id: module.id.clone(),
                version: module.version,
                kind: module.kind,
                interfaces: module.contract.interfaces().to_vec(),
                dependencies: module
                    .contract
                    .dependencies()
                    .iter()
                    .map(|(id, requirement)| ContextDependency {
                        id: id.clone(),
                        target: target_of(requirement),
                        optional: requirement.optional,
                    })
                    .collect(),
            })
            .collect();

        let instances = composition
            .instances
            .iter()
            .map(|instance| {
                let interfaces = find_module(model, &instance.module)
                    .map(|module| module.contract.interfaces().to_vec())
                    .unwrap_or_default();
                ContextInstance {
                    id: instance.id.clone(),
                    module: instance.module.clone(),
                    scope: instance.scope,
                    interfaces,
                    config_keys: instance
                        .config
                        .keys()
                        .map(|id| id.as_str().to_owned())
                        .collect(),
                    provider_keys: instance
                        .dependency_providers
                        .keys()
                        .map(|id| id.as_str().to_owned())
                        .collect(),
                }
            })
            .collect();

        Self {
            project: Some(ContextProject {
                id: model.project.id.clone(),
                name: model.project.name.clone(),
                schema_version: model.project.schema_version,
                root: composition.root.clone(),
            }),
            counts,
            modules,
            instances,
            diagnostics: diagnostics.to_vec(),
            revision: model.revision.clone(),
        }
    }
}

pub(crate) fn find_module<'a>(
    model: &'a CompositionModel,
    reference: &ModuleRef,
) -> Option<&'a composition_domain::ModuleDefinition> {
    model
        .modules
        .iter()
        .find(|module| module.id == reference.id && module.version == reference.version)
}

pub(crate) fn target_of(requirement: &DependencyRequirement) -> DependencyTargetInfo {
    match &requirement.target {
        DependencyTarget::Module(id) => DependencyTargetInfo {
            kind: DependencyTargetKind::Module,
            id: id.to_string(),
            version: requirement.version,
        },
        DependencyTarget::Interface(id) => DependencyTargetInfo {
            kind: DependencyTargetKind::Interface,
            id: id.to_string(),
            version: requirement.version,
        },
    }
}
