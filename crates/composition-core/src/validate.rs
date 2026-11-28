use std::collections::{BTreeMap, BTreeSet};

use composition_domain::{
    Binding, Composition, CompositionModel, DependencyResolution, DependencyStatus,
    DependencyTarget, DependencyTargetInfo, DependencyTargetKind, Diagnostic, ErrorCode, Instance,
    InstanceScope, InterfaceId, LocalId, ModuleDefinition, ModuleId, ModuleKind, ObjectId,
    PortContract, PortDirection, Project, Revision, SchemaVersion, SourceDigest, Version,
};
use composition_toml::{load_project, load_project_with_profile, LoadedWorkspace};

use crate::decode::decode;
use crate::source_map::SourceMap;

/// 007 的统一结果：可选的规范化模型 + 按稳定顺序排序的诊断。
#[derive(Debug)]
pub struct ValidationResult {
    pub model: Option<CompositionModel>,
    pub diagnostics: Vec<Diagnostic>,
    pub source_digest: Option<SourceDigest>,
    pub source_map: SourceMap,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        !self.diagnostics.iter().any(Diagnostic::is_error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.is_error()).count()
    }
}

pub fn validate_project(root: impl AsRef<std::path::Path>) -> ValidationResult {
    let loaded = load_project(root);
    validate_loaded(&loaded)
}

/// 按 profile 选择验证项目；`None` / `Some("default")` 使用 `workspace.composition`。
pub fn validate_project_with_profile(
    root: impl AsRef<std::path::Path>,
    profile: Option<&str>,
) -> ValidationResult {
    let loaded = load_project_with_profile(root, profile);
    validate_loaded(&loaded)
}

pub fn validate_loaded(loaded: &LoadedWorkspace) -> ValidationResult {
    let mut diagnostics = loaded.diagnostics.clone();
    let Some(project) = loaded.project.as_ref() else {
        return finish(None, diagnostics, loaded);
    };
    let Some(composition_file) = loaded.composition_file() else {
        return finish(None, diagnostics, loaded);
    };
    if composition_file.document().is_none() {
        return finish(None, diagnostics, loaded);
    }

    let decoded = decode(loaded);
    diagnostics.extend(decoded.diagnostics.iter().cloned());
    let Some(composition) = decoded.composition.clone() else {
        return finish(None, diagnostics, loaded);
    };
    validate_parts(
        project,
        composition,
        &decoded.modules,
        &decoded.invalid_instances,
        decoded.source_map,
        diagnostics,
        loaded.source_digest.clone(),
    )
}

/// 对内存中的候选模型重新执行 L2–L7 校验；供 M2 编辑事务使用。
pub fn validate_model(
    project: &Project,
    composition: &Composition,
    modules: &[ModuleDefinition],
    source_map: SourceMap,
    source_digest: Option<SourceDigest>,
) -> ValidationResult {
    validate_parts(
        project,
        composition.clone(),
        modules,
        &BTreeSet::new(),
        source_map,
        Vec::new(),
        source_digest,
    )
}

fn validate_parts(
    project: &Project,
    mut composition: Composition,
    modules: &[ModuleDefinition],
    invalid_instances: &BTreeSet<LocalId>,
    source_map: SourceMap,
    mut diagnostics: Vec<Diagnostic>,
    source_digest: Option<SourceDigest>,
) -> ValidationResult {
    let mut registry = BTreeMap::new();
    let mut duplicate_keys = BTreeSet::new();
    for module in modules {
        let key = (module.id.clone(), module.version);
        match registry.entry(key.clone()) {
            std::collections::btree_map::Entry::Occupied(_) => {
                duplicate_keys.insert(key);
                push_at(
                    &source_map,
                    &mut diagnostics,
                    ErrorCode::IdDuplicateModule,
                    format!(
                        "module {}@{} is defined more than once",
                        module.id, module.version
                    ),
                    &ObjectId::module(&module.id, &module.version),
                );
            }
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(module.clone());
            }
        }
    }

    check_versions(
        project,
        &composition,
        &registry,
        &source_map,
        &mut diagnostics,
    );

    let instances_by_id: BTreeMap<LocalId, &Instance> = composition
        .instances
        .iter()
        .map(|instance| (instance.id.clone(), instance))
        .collect();

    let mut resolved: BTreeMap<LocalId, &ModuleDefinition> = BTreeMap::new();
    for instance in &composition.instances {
        let key = (instance.module.id.clone(), instance.module.version);
        let object = ObjectId::instance(&instance.id);
        if duplicate_keys.contains(&key) {
            push_at(
                &source_map,
                &mut diagnostics,
                ErrorCode::RefModuleAmbiguous,
                format!(
                    "instance {} references an ambiguous module {}@{}",
                    instance.id, instance.module.id, instance.module.version
                ),
                &object,
            );
            continue;
        }
        if let Some(module) = registry.get(&key) {
            resolved.insert(instance.id.clone(), module);
        } else if registry.keys().any(|(id, _)| id == &instance.module.id) {
            push_at(
                &source_map,
                &mut diagnostics,
                ErrorCode::RefModuleVersionNotFound,
                format!(
                    "module {}@{} is not defined",
                    instance.module.id, instance.module.version
                ),
                &object,
            );
        } else {
            push_at(
                &source_map,
                &mut diagnostics,
                ErrorCode::RefModuleNotFound,
                format!("module {} is not defined", instance.module.id),
                &object,
            );
        }
    }

    check_module_contracts(&registry, &source_map, &mut diagnostics);
    check_instance_configs(
        &composition.instances,
        &resolved,
        &source_map,
        &mut diagnostics,
    );
    check_service_modules(
        &composition.instances,
        &resolved,
        &source_map,
        &mut diagnostics,
    );
    let valid_bindings = resolve_bindings(
        &composition,
        &instances_by_id,
        &resolved,
        invalid_instances,
        &source_map,
        &mut diagnostics,
    );

    let root_module = resolved.get(&composition.root).copied();
    if !instances_by_id.contains_key(&composition.root)
        && !invalid_instances.contains(&composition.root)
    {
        push_at(
            &source_map,
            &mut diagnostics,
            ErrorCode::RefRootNotFound,
            format!(
                "composition root {:?} is not an instance",
                composition.root.as_str()
            ),
            &ObjectId::composition(),
        );
    }

    check_structure(
        &composition,
        &instances_by_id,
        &resolved,
        root_module,
        &valid_bindings,
        &source_map,
        &mut diagnostics,
    );

    let valid_connections = resolve_connections(
        &composition,
        &instances_by_id,
        &resolved,
        invalid_instances,
        &source_map,
        &mut diagnostics,
    );
    check_connections(&valid_connections, &source_map, &mut diagnostics);

    let dependencies = check_dependencies(
        &composition,
        &resolved,
        &registry,
        invalid_instances,
        project.schema_version.is_v2(),
        &source_map,
        &mut diagnostics,
    );
    check_service_usage(&composition, &dependencies, &source_map, &mut diagnostics);

    if composition.id != project.id {
        push_at(
            &source_map,
            &mut diagnostics,
            ErrorCode::CompositionProjectIdMismatch,
            format!(
                "composition id {} does not match project id {}",
                composition.id, project.id
            ),
            &ObjectId::composition(),
        );
    }

    for instance in &mut composition.instances {
        let key = (instance.module.id.clone(), instance.module.version);
        if let Some(module) = registry.get(&key) {
            let mut effective = BTreeMap::new();
            for (id, property) in module.contract.properties() {
                if let Some(default) = &property.default {
                    effective.insert(id.clone(), default.clone());
                }
            }
            for (id, value) in &instance.config {
                if module
                    .contract
                    .property(id)
                    .is_some_and(|property| property.check_value(value).is_ok())
                {
                    effective.insert(id.clone(), value.clone());
                }
            }
            instance.effective_config = effective;
        }
    }

    diagnostics.sort_by(Diagnostic::compare_order);

    let mut model = CompositionModel::new(project.clone(), composition);
    model.modules = registry.into_values().collect();
    if project.schema_version.supports_interfaces() {
        for module in &mut model.modules {
            module.contract.enable_v2_output();
        }
    }
    model.dependencies = dependencies;
    model.diagnostics = diagnostics.clone();
    model.revision = Some(Revision::initial(source_digest.clone()));
    model.normalize();

    ValidationResult {
        model: Some(model),
        diagnostics,
        source_digest: source_digest.clone(),
        source_map: source_map.clone(),
    }
}

fn finish(
    model: Option<CompositionModel>,
    mut diagnostics: Vec<Diagnostic>,
    loaded: &LoadedWorkspace,
) -> ValidationResult {
    diagnostics.sort_by(Diagnostic::compare_order);
    ValidationResult {
        model,
        diagnostics,
        source_digest: loaded.source_digest.clone(),
        source_map: SourceMap::new(),
    }
}

fn check_versions(
    project: &Project,
    composition: &Composition,
    registry: &BTreeMap<(ModuleId, Version), ModuleDefinition>,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let project_version = project.schema_version;
    match project_version {
        SchemaVersion::V1 => {
            if composition.schema_version != SchemaVersion::V1 {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::SchemaUnsupportedVersion,
                    "composition.schema_version must be 1 in a v1 project",
                    &ObjectId::composition(),
                );
            }
        }
        SchemaVersion::V2 => {
            if composition.schema_version != SchemaVersion::V2 {
                let (code, message) = if composition.schema_version == SchemaVersion::V1 {
                    (
                        ErrorCode::SchemaVersionMismatch,
                        "v2 project requires composition.schema_version = 2",
                    )
                } else {
                    (
                        ErrorCode::SchemaUnsupportedVersion,
                        "unsupported composition.schema_version",
                    )
                };
                push_at(
                    source_map,
                    diagnostics,
                    code,
                    message,
                    &ObjectId::composition(),
                );
            }
        }
        SchemaVersion::V3 => {
            if composition.schema_version != SchemaVersion::V3 {
                let (code, message) = if matches!(
                    composition.schema_version,
                    SchemaVersion::V1 | SchemaVersion::V2
                ) {
                    (
                        ErrorCode::SchemaVersionMismatch,
                        "v3 project requires composition.schema_version = 3",
                    )
                } else {
                    (
                        ErrorCode::SchemaUnsupportedVersion,
                        "unsupported composition.schema_version",
                    )
                };
                push_at(
                    source_map,
                    diagnostics,
                    code,
                    message,
                    &ObjectId::composition(),
                );
            }
        }
        other => {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::SchemaUnsupportedVersion,
                format!("unsupported project.schema_version {other}"),
                &ObjectId::project(),
            );
        }
    }

    for module in registry.values() {
        let module_version = module.contract.schema_version();
        let supported = match project_version {
            SchemaVersion::V1 => module_version == SchemaVersion::V1,
            SchemaVersion::V2 => matches!(module_version, SchemaVersion::V1 | SchemaVersion::V2),
            SchemaVersion::V3 => module_version.is_supported(),
            _ => false,
        };
        if !supported {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::SchemaUnsupportedVersion,
                format!(
                    "module {}@{} uses unsupported schema_version {}",
                    module.id, module.version, module_version
                ),
                &ObjectId::module(&module.id, &module.version),
            );
        }
    }
}

fn check_module_contracts(
    registry: &BTreeMap<(ModuleId, Version), ModuleDefinition>,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for module in registry.values() {
        if module.kind == ModuleKind::Primitive && !module.contract.slots().is_empty() {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::ContractModuleInvalid,
                "primitive module must not declare slots",
                &ObjectId::module(&module.id, &module.version),
            );
        }
        for (id, property) in module.contract.properties() {
            let object = ObjectId::property(&module.id, &module.version, id);
            for error in property.check() {
                push_at(
                    source_map,
                    diagnostics,
                    error.code(),
                    error.message(),
                    &object,
                );
            }
        }
        for (id, slot) in module.contract.slots() {
            let object = ObjectId::slot(&module.id, &module.version, id);
            for error in slot.check() {
                push_at(
                    source_map,
                    diagnostics,
                    error.code(),
                    error.message(),
                    &object,
                );
            }
        }
        for (id, port) in module.contract.ports() {
            let object = ObjectId::port(&module.id, &module.version, id);
            for error in port.check() {
                push_at(
                    source_map,
                    diagnostics,
                    error.code(),
                    error.message(),
                    &object,
                );
            }
        }
    }
}

fn check_instance_configs(
    instances: &[Instance],
    resolved: &BTreeMap<LocalId, &ModuleDefinition>,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for instance in instances {
        let Some(module) = resolved.get(&instance.id) else {
            continue;
        };
        let object = ObjectId::instance(&instance.id);
        for key in instance.dependency_providers.keys() {
            if module.contract.dependency(key).is_none() {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::ContractDependencyUnknown,
                    format!("instance {} maps undeclared dependency {key}", instance.id),
                    &object,
                );
            }
        }
        for key in instance.config.keys() {
            if module.contract.property(key).is_none() {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::ContractPropertyUnknown,
                    format!("instance {} configures unknown property {key}", instance.id),
                    &object,
                );
            }
        }
        for (id, property) in module.contract.properties() {
            let Some(value) = instance.config.get(id) else {
                if property.required && property.default.is_none() {
                    push_at(
                        source_map,
                        diagnostics,
                        ErrorCode::ContractPropertyRequiredMissing,
                        format!("instance {} is missing required property {id}", instance.id),
                        &object,
                    );
                }
                continue;
            };
            if let Err(error) = property.check_value(value) {
                push_at(
                    source_map,
                    diagnostics,
                    error.code(),
                    error.message(),
                    &object,
                );
            }
        }
    }
}

fn is_service(instances: &BTreeMap<LocalId, &Instance>, id: &LocalId) -> bool {
    instances
        .get(id)
        .is_some_and(|instance| instance.scope == Some(InstanceScope::Service))
}

fn check_service_modules(
    instances: &[Instance],
    resolved: &BTreeMap<LocalId, &ModuleDefinition>,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for instance in instances {
        if instance.scope != Some(InstanceScope::Service) {
            continue;
        }
        let Some(module) = resolved.get(&instance.id) else {
            continue;
        };
        if module.kind != ModuleKind::Primitive {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::ContractServiceModuleInvalid,
                format!(
                    "service {} must use a primitive module, but {}@{} is composite",
                    instance.id, module.id, module.version
                ),
                &ObjectId::instance(&instance.id),
            );
        }
    }
}

fn resolve_bindings<'a>(
    composition: &'a composition_domain::Composition,
    instances: &BTreeMap<LocalId, &'a Instance>,
    resolved: &BTreeMap<LocalId, &'a ModuleDefinition>,
    invalid_instances: &BTreeSet<LocalId>,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<&'a Binding> {
    let mut valid = Vec::new();
    for binding in &composition.bindings {
        let object = ObjectId::binding(&binding.id);
        let mut endpoints_ok = true;
        for id in [&binding.parent, &binding.child] {
            if !instances.contains_key(id) {
                if !invalid_instances.contains(id) {
                    push_at(
                        source_map,
                        diagnostics,
                        ErrorCode::RefInstanceNotFound,
                        format!("binding {} references missing instance {id}", binding.id),
                        &object,
                    );
                }
                endpoints_ok = false;
            }
        }
        if !endpoints_ok {
            continue;
        }
        if is_service(instances, &binding.parent) || is_service(instances, &binding.child) {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::StructureServiceInBinding,
                format!(
                    "binding {} must not reference a service instance",
                    binding.id
                ),
                &object,
            );
            continue;
        }
        let Some(parent_module) = resolved.get(&binding.parent) else {
            continue;
        };
        let Some(slot) = parent_module.contract.slot(&binding.slot) else {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::RefSlotNotFound,
                format!(
                    "binding {} references missing slot {} on {}",
                    binding.id, binding.slot, binding.parent
                ),
                &object,
            );
            continue;
        };
        let Some(child_module) = resolved.get(&binding.child) else {
            continue;
        };
        if !slot.accepts_module(&child_module.id) {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::ContractSlotRejectsModule,
                format!(
                    "slot {} does not accept module {}",
                    binding.slot, child_module.id
                ),
                &object,
            );
            continue;
        }
        valid.push(binding);
    }
    valid
}

fn check_structure(
    composition: &composition_domain::Composition,
    instances: &BTreeMap<LocalId, &Instance>,
    resolved: &BTreeMap<LocalId, &ModuleDefinition>,
    root_module: Option<&ModuleDefinition>,
    valid_bindings: &[&Binding],
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if instances
        .get(&composition.root)
        .is_some_and(|instance| instance.scope == Some(InstanceScope::Service))
    {
        push_at(
            source_map,
            diagnostics,
            ErrorCode::StructureRootScopeInvalid,
            format!("root {} must be a structure instance", composition.root),
            &ObjectId::instance(&composition.root),
        );
    }

    let Some(root_module) = root_module else {
        return;
    };
    if root_module.kind != ModuleKind::Composite {
        push_at(
            source_map,
            diagnostics,
            ErrorCode::StructureRootNotComposite,
            format!("root {} is not a composite module", composition.root),
            &ObjectId::instance(&composition.root),
        );
        return;
    }

    let mut kept: Vec<&Binding> = Vec::new();
    let mut seen_tuples = BTreeSet::new();
    for binding in valid_bindings {
        let binding = *binding;
        if binding.child == composition.root {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::StructureRootHasParent,
                format!("root {} must not be a binding child", composition.root),
                &ObjectId::binding(&binding.id),
            );
            continue;
        }
        let tuple = (
            binding.parent.clone(),
            binding.slot.clone(),
            binding.child.clone(),
        );
        if !seen_tuples.insert(tuple) {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::StructureDuplicateBinding,
                format!(
                    "duplicate binding ({}, {}, {})",
                    binding.parent, binding.slot, binding.child
                ),
                &ObjectId::binding(&binding.id),
            );
            continue;
        }
        kept.push(binding);
    }

    let mut parents: BTreeMap<LocalId, Vec<&Binding>> = BTreeMap::new();
    for binding in &kept {
        parents
            .entry(binding.child.clone())
            .or_default()
            .push(*binding);
    }
    for (child, bindings) in &parents {
        if child != &composition.root && bindings.len() > 1 {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::StructureInstanceMultipleParents,
                format!("instance {child} has {} parents", bindings.len()),
                &ObjectId::instance(child),
            );
        }
    }

    for (instance_id, module) in resolved {
        if is_service(instances, instance_id) {
            continue;
        }
        for (slot_id, slot) in module.contract.slots() {
            let count = kept
                .iter()
                .filter(|binding| &binding.parent == instance_id && &binding.slot == slot_id)
                .count();
            if !slot.binding_count_ok(count) {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::StructureSlotCardinality,
                    format!(
                        "slot {slot_id} on {instance_id} has {count} bindings; expected cardinality {}",
                        slot.cardinality.as_str()
                    ),
                    &ObjectId::slot(&module.id, &module.version, slot_id),
                );
            }
        }
    }

    let cycles = find_cycles(instances, &kept);
    if let Some(first) = cycles.iter().next() {
        push_at(
            source_map,
            diagnostics,
            ErrorCode::StructureCycle,
            format!("binding cycle includes instance {first}"),
            &ObjectId::instance(first),
        );
    }

    let mut reachable = BTreeSet::new();
    reachable.insert(composition.root.clone());
    let mut stack = vec![composition.root.clone()];
    while let Some(current) = stack.pop() {
        for binding in &kept {
            if binding.parent == current && reachable.insert(binding.child.clone()) {
                stack.push(binding.child.clone());
            }
        }
    }
    for instance in instances.keys() {
        if instance == &composition.root
            || cycles.contains(instance)
            || reachable.contains(instance)
            || is_service(instances, instance)
        {
            continue;
        }
        push_at(
            source_map,
            diagnostics,
            ErrorCode::StructureUnreachableInstance,
            format!(
                "instance {instance} is not reachable from root {}",
                composition.root
            ),
            &ObjectId::instance(instance),
        );
    }
}

fn find_cycles(
    instances: &BTreeMap<LocalId, &Instance>,
    bindings: &[&Binding],
) -> BTreeSet<LocalId> {
    let mut graph: BTreeMap<LocalId, Vec<LocalId>> = BTreeMap::new();
    for binding in bindings {
        graph
            .entry(binding.parent.clone())
            .or_default()
            .push(binding.child.clone());
    }
    let mut state: BTreeMap<LocalId, u8> = BTreeMap::new();
    let mut path = Vec::new();
    let mut cycles = BTreeSet::new();
    for instance in instances.keys() {
        if state.get(instance).copied().unwrap_or(0) == 0 {
            dfs_cycles(instance, &graph, &mut state, &mut path, &mut cycles);
        }
    }
    cycles
}

fn dfs_cycles(
    node: &LocalId,
    graph: &BTreeMap<LocalId, Vec<LocalId>>,
    state: &mut BTreeMap<LocalId, u8>,
    path: &mut Vec<LocalId>,
    cycles: &mut BTreeSet<LocalId>,
) {
    state.insert(node.clone(), 1);
    path.push(node.clone());
    if let Some(children) = graph.get(node) {
        for child in children {
            match state.get(child).copied().unwrap_or(0) {
                0 => dfs_cycles(child, graph, state, path, cycles),
                1 => {
                    if let Some(index) = path.iter().position(|item| item == child) {
                        cycles.extend(path[index..].iter().cloned());
                    }
                }
                _ => {}
            }
        }
    }
    path.pop();
    state.insert(node.clone(), 2);
}

struct ResolvedConnection<'a> {
    connection: &'a composition_domain::Connection,
    from_module: &'a ModuleDefinition,
    to_module: &'a ModuleDefinition,
    from_port: &'a PortContract,
    to_port: &'a PortContract,
}

fn resolve_connections<'a>(
    composition: &'a composition_domain::Composition,
    instances: &BTreeMap<LocalId, &'a Instance>,
    resolved: &BTreeMap<LocalId, &'a ModuleDefinition>,
    invalid_instances: &BTreeSet<LocalId>,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ResolvedConnection<'a>> {
    let mut valid = Vec::new();
    for connection in &composition.connections {
        let object = ObjectId::connection(&connection.id);
        let mut endpoints_ok = true;
        for id in [&connection.from.instance, &connection.to.instance] {
            if !instances.contains_key(id) {
                if !invalid_instances.contains(id) {
                    push_at(
                        source_map,
                        diagnostics,
                        ErrorCode::RefInstanceNotFound,
                        format!(
                            "connection {} references missing instance {id}",
                            connection.id
                        ),
                        &object,
                    );
                }
                endpoints_ok = false;
            }
        }
        if !endpoints_ok {
            continue;
        }
        let Some(from_module) = resolved.get(&connection.from.instance) else {
            continue;
        };
        let Some(to_module) = resolved.get(&connection.to.instance) else {
            continue;
        };
        let Some(from_port) = from_module.contract.port(&connection.from.port) else {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::RefPortNotFound,
                format!(
                    "connection {} references missing port {} on {}",
                    connection.id, connection.from.port, connection.from.instance
                ),
                &object,
            );
            continue;
        };
        let Some(to_port) = to_module.contract.port(&connection.to.port) else {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::RefPortNotFound,
                format!(
                    "connection {} references missing port {} on {}",
                    connection.id, connection.to.port, connection.to.instance
                ),
                &object,
            );
            continue;
        };
        valid.push(ResolvedConnection {
            connection,
            from_module,
            to_module,
            from_port,
            to_port,
        });
    }
    valid
}

fn check_connections(
    valid: &[ResolvedConnection<'_>],
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut checked = Vec::new();
    for connection in valid {
        let object = ObjectId::connection(&connection.connection.id);
        if connection.from_port.direction != PortDirection::Output
            || connection.to_port.direction != PortDirection::Input
        {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::ConnectionDirectionInvalid,
                format!(
                    "connection {} must go from an output port to an input port",
                    connection.connection.id
                ),
                &object,
            );
            continue;
        }
        if connection.from_port.type_id != connection.to_port.type_id {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::ConnectionTypeMismatch,
                format!(
                    "connection {} connects {} to {}",
                    connection.connection.id,
                    connection.from_port.type_id,
                    connection.to_port.type_id
                ),
                &object,
            );
            continue;
        }
        if let (Some(from), Some(to)) =
            (&connection.from_port.protocol, &connection.to_port.protocol)
        {
            if from != to {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::ConnectionProtocolMismatch,
                    format!(
                        "connection {} uses protocols {from:?} and {to:?}",
                        connection.connection.id
                    ),
                    &object,
                );
                continue;
            }
        }
        checked.push(connection);
    }

    let mut seen_pairs = BTreeSet::new();
    for connection in &checked {
        let pair = (
            connection.connection.from.instance.clone(),
            connection.connection.from.port.clone(),
            connection.connection.to.instance.clone(),
            connection.connection.to.port.clone(),
        );
        if !seen_pairs.insert(pair) {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::ConnectionDuplicate,
                format!(
                    "connection {} duplicates an existing endpoint pair",
                    connection.connection.id
                ),
                &ObjectId::connection(&connection.connection.id),
            );
        }
    }

    let mut out_degree: BTreeMap<(LocalId, LocalId), usize> = BTreeMap::new();
    let mut in_degree: BTreeMap<(LocalId, LocalId), usize> = BTreeMap::new();
    for connection in &checked {
        *out_degree
            .entry((
                connection.connection.from.instance.clone(),
                connection.connection.from.port.clone(),
            ))
            .or_default() += 1;
        *in_degree
            .entry((
                connection.connection.to.instance.clone(),
                connection.connection.to.port.clone(),
            ))
            .or_default() += 1;
    }
    let mut reported_ports = BTreeSet::new();
    for connection in &checked {
        let from_key = (
            connection.connection.from.instance.clone(),
            connection.connection.from.port.clone(),
        );
        let to_key = (
            connection.connection.to.instance.clone(),
            connection.connection.to.port.clone(),
        );
        let entries = [
            (
                connection.from_module,
                &connection.connection.from.port,
                connection.from_port,
                &from_key,
                out_degree.get(&from_key).copied().unwrap_or(0),
            ),
            (
                connection.to_module,
                &connection.connection.to.port,
                connection.to_port,
                &to_key,
                in_degree.get(&to_key).copied().unwrap_or(0),
            ),
        ];
        for (module, port_id, port, key, degree) in entries {
            if port.multi || degree <= 1 {
                continue;
            }
            let object = ObjectId::port(&module.id, &module.version, port_id);
            if reported_ports.insert(object.clone()) {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::ConnectionPortMultiViolation,
                    format!(
                        "port {} on {} is not multi but has {degree} connections",
                        port_id, key.0
                    ),
                    &object,
                );
            }
        }
    }
}

fn has_instance(composition: &composition_domain::Composition, id: &LocalId) -> bool {
    composition
        .instances
        .iter()
        .any(|instance| &instance.id == id)
}

fn check_dependencies(
    composition: &composition_domain::Composition,
    resolved: &BTreeMap<LocalId, &ModuleDefinition>,
    registry: &BTreeMap<(ModuleId, Version), ModuleDefinition>,
    invalid_instances: &BTreeSet<LocalId>,
    v2_output: bool,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<DependencyResolution> {
    let mut results = Vec::new();
    for instance in &composition.instances {
        let Some(module) = resolved.get(&instance.id) else {
            continue;
        };
        let object = ObjectId::instance(&instance.id);
        for (requirement_id, requirement) in module.contract.dependencies() {
            let mut status = DependencyStatus::Unresolved;
            let mut provider = None;
            match &requirement.target {
                DependencyTarget::Module(target) => {
                    if let Some(explicit) = instance.dependency_providers.get(requirement_id) {
                        if !has_instance(composition, explicit) {
                            if !invalid_instances.contains(explicit) {
                                push_at(
                                    source_map,
                                    diagnostics,
                                    ErrorCode::RefInstanceNotFound,
                                    format!(
                                        "instance {} maps dependency {requirement_id} to missing instance {explicit}",
                                        instance.id
                                    ),
                                    &object,
                                );
                            }
                        } else if explicit == &instance.id {
                            push_at(
                                source_map,
                                diagnostics,
                                ErrorCode::DependencySelf,
                                format!(
                                    "instance {} cannot provide its own dependency {requirement_id}",
                                    instance.id
                                ),
                                &object,
                            );
                        } else if let Some(provider_module) = resolved.get(explicit) {
                            if provider_module.id == *target
                                && version_matches(provider_module, requirement.version.as_ref())
                            {
                                status = DependencyStatus::Resolved;
                                provider = Some(explicit.clone());
                            } else {
                                push_at(
                                    source_map,
                                    diagnostics,
                                    ErrorCode::DependencyProviderInvalid,
                                    format!(
                                        "instance {} dependency {requirement_id} provider {explicit} does not satisfy {}",
                                        instance.id, target
                                    ),
                                    &object,
                                );
                            }
                        }
                    } else {
                        let candidates: Vec<LocalId> = resolved
                            .iter()
                            .filter(|(id, candidate)| {
                                *id != &instance.id
                                    && candidate.id == *target
                                    && version_matches(candidate, requirement.version.as_ref())
                            })
                            .map(|(id, _)| (*id).clone())
                            .collect();
                        (status, provider) = resolve_auto(
                            instance,
                            requirement_id,
                            requirement.optional,
                            candidates,
                            &object,
                            source_map,
                            diagnostics,
                        );
                    }
                }
                DependencyTarget::Interface(interface) => {
                    let declared = registry.values().any(|candidate| {
                        implements_interface(candidate, interface, requirement.version.as_ref())
                    });
                    if !declared {
                        let requirement_object =
                            ObjectId::dependency(&module.id, &module.version, requirement_id);
                        push_at(
                            source_map,
                            diagnostics,
                            ErrorCode::RefInterfaceNotFound,
                            format!(
                                "instance {} dependency {requirement_id} requires interface {interface} that no module declares",
                                instance.id
                            ),
                            &requirement_object,
                        );
                    } else if let Some(explicit) = instance.dependency_providers.get(requirement_id)
                    {
                        if !has_instance(composition, explicit) {
                            if !invalid_instances.contains(explicit) {
                                push_at(
                                    source_map,
                                    diagnostics,
                                    ErrorCode::RefInstanceNotFound,
                                    format!(
                                        "instance {} maps dependency {requirement_id} to missing instance {explicit}",
                                        instance.id
                                    ),
                                    &object,
                                );
                            }
                        } else if explicit == &instance.id {
                            push_at(
                                source_map,
                                diagnostics,
                                ErrorCode::DependencySelf,
                                format!(
                                    "instance {} cannot provide its own dependency {requirement_id}",
                                    instance.id
                                ),
                                &object,
                            );
                        } else if let Some(provider_module) = resolved.get(explicit) {
                            if implements_interface(
                                provider_module,
                                interface,
                                requirement.version.as_ref(),
                            ) {
                                status = DependencyStatus::Resolved;
                                provider = Some(explicit.clone());
                            } else {
                                push_at(
                                    source_map,
                                    diagnostics,
                                    ErrorCode::DependencyInterfaceMismatch,
                                    format!(
                                        "instance {} dependency {requirement_id} provider {explicit} does not implement interface {interface}",
                                        instance.id
                                    ),
                                    &object,
                                );
                            }
                        }
                    } else {
                        let candidates: Vec<LocalId> = resolved
                            .iter()
                            .filter(|(id, candidate)| {
                                *id != &instance.id
                                    && implements_interface(
                                        candidate,
                                        interface,
                                        requirement.version.as_ref(),
                                    )
                            })
                            .map(|(id, _)| (*id).clone())
                            .collect();
                        (status, provider) = resolve_auto(
                            instance,
                            requirement_id,
                            requirement.optional,
                            candidates,
                            &object,
                            source_map,
                            diagnostics,
                        );
                    }
                }
            }
            results.push(DependencyResolution {
                requirement_id: requirement_id.clone(),
                from_instance: instance.id.clone(),
                target: if v2_output {
                    Some(dependency_target_info(requirement))
                } else {
                    None
                },
                provider_instance: provider,
                optional: requirement.optional,
                status,
                source: instance.source.clone(),
            });
        }
    }
    results
}

fn version_matches(module: &ModuleDefinition, version: Option<&Version>) -> bool {
    version.is_none_or(|version| &module.version == version)
}

fn implements_interface(
    module: &ModuleDefinition,
    interface: &InterfaceId,
    version: Option<&Version>,
) -> bool {
    module.contract.interfaces().iter().any(|declaration| {
        &declaration.id == interface && version.is_none_or(|v| &declaration.version == v)
    })
}

fn dependency_target_info(
    requirement: &composition_domain::DependencyRequirement,
) -> DependencyTargetInfo {
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

fn resolve_auto(
    instance: &Instance,
    requirement_id: &LocalId,
    optional: bool,
    candidates: Vec<LocalId>,
    object: &ObjectId,
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) -> (DependencyStatus, Option<LocalId>) {
    match candidates.len() {
        0 => {
            if optional {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::DependencyOptionalUnresolved,
                    format!(
                        "instance {} optional dependency {requirement_id} has no provider",
                        instance.id
                    ),
                    object,
                );
                (DependencyStatus::OptionalUnresolved, None)
            } else {
                push_at(
                    source_map,
                    diagnostics,
                    ErrorCode::DependencyUnresolved,
                    format!(
                        "instance {} dependency {requirement_id} has no provider",
                        instance.id
                    ),
                    object,
                );
                (DependencyStatus::Unresolved, None)
            }
        }
        1 => (DependencyStatus::Resolved, Some(candidates[0].clone())),
        _ => {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::DependencyAmbiguous,
                format!(
                    "instance {} dependency {requirement_id} has multiple providers",
                    instance.id
                ),
                object,
            );
            (DependencyStatus::Ambiguous, None)
        }
    }
}

fn check_service_usage(
    composition: &composition_domain::Composition,
    dependencies: &[DependencyResolution],
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut referenced = BTreeSet::new();
    for instance in &composition.instances {
        for provider in instance.dependency_providers.values() {
            referenced.insert(provider.clone());
        }
    }
    for dependency in dependencies {
        if let Some(provider) = &dependency.provider_instance {
            referenced.insert(provider.clone());
        }
    }
    for connection in &composition.connections {
        referenced.insert(connection.from.instance.clone());
        referenced.insert(connection.to.instance.clone());
    }
    for instance in &composition.instances {
        if instance.scope == Some(InstanceScope::Service) && !referenced.contains(&instance.id) {
            push_at(
                source_map,
                diagnostics,
                ErrorCode::ServiceUnused,
                format!(
                    "service {} is not used by any dependency or connection",
                    instance.id
                ),
                &ObjectId::instance(&instance.id),
            );
        }
    }
}

fn push_at(
    source_map: &SourceMap,
    diagnostics: &mut Vec<Diagnostic>,
    code: ErrorCode,
    message: impl Into<String>,
    object: &ObjectId,
) {
    let mut diagnostic = Diagnostic::new(code, message.into());
    if let Some(source) = source_map.get(object) {
        diagnostic = diagnostic.with_source(source.file.clone(), source.range);
    }
    diagnostic = diagnostic.with_object_id(object.clone());
    diagnostics.push(diagnostic);
}
