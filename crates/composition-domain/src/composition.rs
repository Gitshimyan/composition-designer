use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::{LocalId, ModuleId, Version};
use crate::schema::SchemaVersion;
use crate::source::SourceRef;
use crate::value::ConfigValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModuleRef {
    pub id: ModuleId,
    pub version: Version,
}

impl ModuleRef {
    pub fn new(id: ModuleId, version: Version) -> Self {
        Self { id, version }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Instance {
    pub id: LocalId,
    pub module: ModuleRef,
    pub config: BTreeMap<LocalId, ConfigValue>,
    pub effective_config: BTreeMap<LocalId, ConfigValue>,
    pub dependency_providers: BTreeMap<LocalId, LocalId>,
    pub source: Option<SourceRef>,
}

impl Instance {
    pub fn new(id: LocalId, module: ModuleRef) -> Self {
        Self {
            id,
            module,
            config: BTreeMap::new(),
            effective_config: BTreeMap::new(),
            dependency_providers: BTreeMap::new(),
            source: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Binding {
    pub id: LocalId,
    pub parent: LocalId,
    pub slot: LocalId,
    pub child: LocalId,
    pub source: Option<SourceRef>,
}

impl Binding {
    pub fn new(id: LocalId, parent: LocalId, slot: LocalId, child: LocalId) -> Self {
        Self {
            id,
            parent,
            slot,
            child,
            source: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRef {
    pub instance: LocalId,
    pub port: LocalId,
}

impl PortRef {
    pub fn new(instance: LocalId, port: LocalId) -> Self {
        Self { instance, port }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Connection {
    pub id: LocalId,
    pub from: PortRef,
    pub to: PortRef,
    pub source: Option<SourceRef>,
}

impl Connection {
    pub fn new(id: LocalId, from: PortRef, to: PortRef) -> Self {
        Self {
            id,
            from,
            to,
            source: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Composition {
    pub schema_version: SchemaVersion,
    pub id: ModuleId,
    pub root: LocalId,
    pub instances: Vec<Instance>,
    pub bindings: Vec<Binding>,
    pub connections: Vec<Connection>,
    pub source: Option<SourceRef>,
}

impl Composition {
    pub fn new(schema_version: SchemaVersion, id: ModuleId, root: LocalId) -> Self {
        Self {
            schema_version,
            id,
            root,
            instances: Vec::new(),
            bindings: Vec::new(),
            connections: Vec::new(),
            source: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local(value: &str) -> LocalId {
        LocalId::parse(value).unwrap()
    }

    fn module_ref(id: &str, version: &str) -> ModuleRef {
        ModuleRef::new(
            ModuleId::parse(id).unwrap(),
            Version::parse(version).unwrap(),
        )
    }

    #[test]
    fn new_instances_have_empty_maps() {
        let instance = Instance::new(local("app"), module_ref("shell.desktop", "1.0.0"));
        assert_eq!(instance.id, local("app"));
        assert!(instance.config.is_empty());
        assert!(instance.effective_config.is_empty());
        assert!(instance.dependency_providers.is_empty());
        assert!(instance.source.is_none());
    }

    #[test]
    fn compositions_start_empty_and_keep_identity() {
        let composition = Composition::new(
            SchemaVersion::V1,
            ModuleId::parse("example.minimal").unwrap(),
            local("app"),
        );
        assert_eq!(composition.schema_version, SchemaVersion::V1);
        assert_eq!(composition.root, local("app"));
        assert!(composition.instances.is_empty());
        assert!(composition.bindings.is_empty());
        assert!(composition.connections.is_empty());
    }

    #[test]
    fn connections_reference_ports() {
        let connection = Connection::new(
            local("editor-status"),
            PortRef::new(local("editor"), local("status")),
            PortRef::new(local("statusbar"), local("text")),
        );
        assert_eq!(connection.from.instance, local("editor"));
        assert_eq!(connection.to.port, local("text"));
    }
}
