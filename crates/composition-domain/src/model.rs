use std::fmt;

use serde::Serialize;

use crate::composition::Composition;
use crate::diagnostic::Diagnostic;
use crate::ids::{LocalId, ModuleId};
use crate::module::ModuleDefinition;
use crate::revision::Revision;
use crate::schema::SchemaVersion;
use crate::source::SourceRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Project {
    pub schema_version: SchemaVersion,
    pub id: ModuleId,
    pub name: String,
    pub composition_entry: String,
    pub module_roots: Vec<String>,
    pub source: Option<SourceRef>,
}

impl Project {
    pub fn new(
        schema_version: SchemaVersion,
        id: ModuleId,
        name: impl Into<String>,
        composition_entry: impl Into<String>,
        module_roots: Vec<String>,
    ) -> Self {
        Self {
            schema_version,
            id,
            name: name.into(),
            composition_entry: composition_entry.into(),
            module_roots,
            source: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyStatus {
    Resolved,
    Unresolved,
    OptionalUnresolved,
    Ambiguous,
}

impl DependencyStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Unresolved => "unresolved",
            Self::OptionalUnresolved => "optional_unresolved",
            Self::Ambiguous => "ambiguous",
        }
    }
}

impl fmt::Display for DependencyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyResolution {
    pub requirement_id: LocalId,
    pub from_instance: LocalId,
    pub provider_instance: Option<LocalId>,
    pub optional: bool,
    pub status: DependencyStatus,
    pub source: Option<SourceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompositionModel {
    pub project: Project,
    pub composition: Composition,
    pub modules: Vec<ModuleDefinition>,
    pub dependencies: Vec<DependencyResolution>,
    pub diagnostics: Vec<Diagnostic>,
    pub revision: Option<Revision>,
}

impl CompositionModel {
    pub fn new(project: Project, composition: Composition) -> Self {
        Self {
            project,
            composition,
            modules: Vec::new(),
            dependencies: Vec::new(),
            diagnostics: Vec::new(),
            revision: None,
        }
    }

    /// schema §8.2 的稳定顺序；只排序，不修复非法数据。
    pub fn normalize(&mut self) {
        self.project.module_roots.sort();
        self.modules.sort_by(|left, right| {
            left.id
                .cmp(&right.id)
                .then_with(|| left.version.cmp(&right.version))
        });
        for module in &mut self.modules {
            module.normalize();
        }
        self.composition
            .instances
            .sort_by(|left, right| left.id.cmp(&right.id));
        self.composition.bindings.sort_by(|left, right| {
            left.parent
                .cmp(&right.parent)
                .then_with(|| left.slot.cmp(&right.slot))
                .then_with(|| left.child.cmp(&right.child))
                .then_with(|| left.id.cmp(&right.id))
        });
        self.composition.connections.sort_by(|left, right| {
            left.from
                .instance
                .cmp(&right.from.instance)
                .then_with(|| left.from.port.cmp(&right.from.port))
                .then_with(|| left.to.instance.cmp(&right.to.instance))
                .then_with(|| left.to.port.cmp(&right.to.port))
                .then_with(|| left.id.cmp(&right.id))
        });
        self.dependencies.sort_by(|left, right| {
            left.from_instance
                .cmp(&right.from_instance)
                .then_with(|| left.requirement_id.cmp(&right.requirement_id))
        });
        self.diagnostics
            .sort_by(|left, right| left.compare_order(right));
    }

    pub fn is_valid(&self) -> bool {
        !self.diagnostics.iter().any(Diagnostic::is_error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.is_error()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::{Binding, Composition, Connection, Instance, ModuleRef, PortRef};
    use crate::diagnostic::ObjectId;
    use crate::error_code::ErrorCode;
    use crate::ids::{LocalId, ModuleId, Version};

    fn local(value: &str) -> LocalId {
        LocalId::parse(value).unwrap()
    }

    fn model() -> CompositionModel {
        let project = Project::new(
            SchemaVersion::V1,
            ModuleId::parse("example.minimal").unwrap(),
            "Minimal",
            "composition/main.toml",
            vec!["modules".into(), "modules".into()],
        );
        let composition = Composition::new(
            SchemaVersion::V1,
            ModuleId::parse("example.minimal").unwrap(),
            local("app"),
        );
        CompositionModel::new(project, composition)
    }

    #[test]
    fn normalize_sorts_instances_bindings_and_connections() {
        let mut model = model();
        let module = ModuleRef::new(
            ModuleId::parse("leaf.widget").unwrap(),
            Version::parse("1.0.0").unwrap(),
        );
        model.composition.instances = vec![
            Instance::new(local("z"), module.clone()),
            Instance::new(local("a"), module.clone()),
        ];
        model.composition.bindings = vec![
            Binding::new(local("b"), local("app"), local("content"), local("z")),
            Binding::new(local("a"), local("app"), local("content"), local("a")),
        ];
        model.composition.connections = vec![
            Connection::new(
                local("z"),
                PortRef::new(local("b"), local("out")),
                PortRef::new(local("b"), local("in")),
            ),
            Connection::new(
                local("a"),
                PortRef::new(local("a"), local("out")),
                PortRef::new(local("a"), local("in")),
            ),
        ];
        model.normalize();
        assert_eq!(
            model
                .composition
                .instances
                .iter()
                .map(|instance| instance.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "z"]
        );
        assert_eq!(
            model
                .composition
                .bindings
                .iter()
                .map(|binding| binding.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(
            model
                .composition
                .connections
                .iter()
                .map(|connection| connection.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "z"]
        );
    }

    #[test]
    fn normalize_sorts_module_roots_and_diagnostics() {
        let mut model = model();
        model.project.module_roots = vec!["z".into(), "a".into()];
        model.diagnostics = vec![
            Diagnostic::new(ErrorCode::TomlSyntaxError, "b").with_source("b.toml", None),
            Diagnostic::new(ErrorCode::TomlSyntaxError, "a").with_source("a.toml", None),
        ];
        model.normalize();
        assert_eq!(model.project.module_roots, ["a", "z"]);
        assert_eq!(
            model
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.file.as_deref().unwrap())
                .collect::<Vec<_>>(),
            ["a.toml", "b.toml"]
        );
    }

    #[test]
    fn validity_follows_error_severity() {
        let mut model = model();
        assert!(model.is_valid());
        model.diagnostics.push(Diagnostic::new(
            ErrorCode::DependencyOptionalUnresolved,
            "optional missing",
        ));
        assert!(model.is_valid());
        assert_eq!(model.error_count(), 0);
        model.diagnostics.push(Diagnostic::new(
            ErrorCode::RefModuleNotFound,
            "missing module",
        ));
        assert!(!model.is_valid());
        assert_eq!(model.error_count(), 1);
    }

    #[test]
    fn normalize_sorts_modules_by_id_then_version() {
        use crate::contract::Contract;
        use crate::module::ModuleKind;

        let mut model = model();
        let build = |id: &str, version: &str| {
            ModuleDefinition::new(
                ModuleId::parse(id).unwrap(),
                Version::parse(version).unwrap(),
                ModuleKind::Composite,
                Contract::new(),
            )
        };
        model.modules = vec![
            build("z.shell", "1.0.0"),
            build("a.shell", "2.0.0"),
            build("a.shell", "1.0.0"),
        ];
        model.normalize();
        let sorted: Vec<(String, String)> = model
            .modules
            .iter()
            .map(|module| (module.id.to_string(), module.version.to_string()))
            .collect();
        assert_eq!(
            sorted,
            [
                ("a.shell".to_string(), "1.0.0".to_string()),
                ("a.shell".to_string(), "2.0.0".to_string()),
                ("z.shell".to_string(), "1.0.0".to_string()),
            ]
        );
    }

    #[test]
    fn dependency_status_strings_match_the_schema() {
        assert_eq!(DependencyStatus::Resolved.as_str(), "resolved");
        assert_eq!(DependencyStatus::Unresolved.as_str(), "unresolved");
        assert_eq!(
            DependencyStatus::OptionalUnresolved.as_str(),
            "optional_unresolved"
        );
        assert_eq!(DependencyStatus::Ambiguous.as_str(), "ambiguous");
    }

    #[test]
    fn object_id_is_usable_from_diagnostics() {
        let diagnostic =
            Diagnostic::new(ErrorCode::RefModuleNotFound, "x").with_object_id(ObjectId::project());
        assert_eq!(diagnostic.object_id, Some(ObjectId::project()));
    }
}
