use std::collections::BTreeMap;

use composition_domain::{ConfigValue, LocalId, ModuleId, PortRef, Revision, Version};
use serde::{Deserialize, Serialize};

/// v0.1 支持的编辑命令。所有命令都通过事务应用，不直接写 TOML。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    CreateInstance {
        id: LocalId,
        module: ModuleId,
        version: Version,
        config: BTreeMap<LocalId, ConfigValue>,
    },
    DeleteInstance {
        id: LocalId,
    },
    RenameInstance {
        from: LocalId,
        to: LocalId,
    },
    UpdateInstanceModule {
        id: LocalId,
        module: ModuleId,
        version: Version,
    },
    SetConfig {
        instance: LocalId,
        property: LocalId,
        value: ConfigValue,
    },
    UnsetConfig {
        instance: LocalId,
        property: LocalId,
    },
    CreateBinding {
        id: LocalId,
        parent: LocalId,
        slot: LocalId,
        child: LocalId,
    },
    UpdateBinding {
        id: LocalId,
        parent: LocalId,
        slot: LocalId,
        child: LocalId,
    },
    DeleteBinding {
        id: LocalId,
    },
    CreateConnection {
        id: LocalId,
        from: PortRef,
        to: PortRef,
    },
    UpdateConnection {
        id: LocalId,
        from: PortRef,
        to: PortRef,
    },
    DeleteConnection {
        id: LocalId,
    },
}

impl Command {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::CreateInstance { .. } => "create_instance",
            Self::DeleteInstance { .. } => "delete_instance",
            Self::RenameInstance { .. } => "rename_instance",
            Self::UpdateInstanceModule { .. } => "update_instance_module",
            Self::SetConfig { .. } => "set_config",
            Self::UnsetConfig { .. } => "unset_config",
            Self::CreateBinding { .. } => "create_binding",
            Self::UpdateBinding { .. } => "update_binding",
            Self::DeleteBinding { .. } => "delete_binding",
            Self::CreateConnection { .. } => "create_connection",
            Self::UpdateConnection { .. } => "update_connection",
            Self::DeleteConnection { .. } => "delete_connection",
        }
    }
}

pub const COMMAND_KINDS: &[&str] = &[
    "create_instance",
    "delete_instance",
    "rename_instance",
    "update_instance_module",
    "set_config",
    "unset_config",
    "create_binding",
    "update_binding",
    "delete_binding",
    "create_connection",
    "update_connection",
    "delete_connection",
];

/// 一次原子事务：一组命令 + 读取时的 Revision。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub request_id: String,
    pub base_revision: Revision,
    pub commands: Vec<Command>,
}

impl Transaction {
    pub fn new(
        request_id: impl Into<String>,
        base_revision: Revision,
        commands: Vec<Command>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            base_revision,
            commands,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use composition_domain::SourceDigest;

    #[test]
    fn commands_serialize_with_type_tags() {
        let command = Command::SetConfig {
            instance: LocalId::parse("app").unwrap(),
            property: LocalId::parse("title").unwrap(),
            value: ConfigValue::String("x".into()),
        };
        let value = serde_json::to_value(&command).unwrap();
        assert_eq!(value["type"], "set_config");
        assert_eq!(value["instance"], "app");
        assert_eq!(value["property"], "title");
        assert_eq!(value["value"], "x");
    }

    #[test]
    fn command_kinds_are_unique() {
        let unique: std::collections::BTreeSet<&str> = COMMAND_KINDS.iter().copied().collect();
        assert_eq!(unique.len(), COMMAND_KINDS.len());
        assert_eq!(COMMAND_KINDS.len(), 12);
    }

    #[test]
    fn transaction_deserializes_from_wire() {
        let json = r#"{
            "request_id": "r-1",
            "base_revision": { "seq": 0, "source_digest": null },
            "commands": [
                {
                    "type": "set_config",
                    "instance": "app",
                    "property": "title",
                    "value": "Hello"
                }
            ]
        }"#;
        let transaction: Transaction = serde_json::from_str(json).unwrap();
        assert_eq!(transaction.request_id, "r-1");
        assert_eq!(transaction.commands.len(), 1);
        assert_eq!(
            transaction.commands[0],
            Command::SetConfig {
                instance: LocalId::parse("app").unwrap(),
                property: LocalId::parse("title").unwrap(),
                value: ConfigValue::String("Hello".into()),
            }
        );
    }

    #[test]
    fn transaction_keeps_request_and_revision() {
        let revision = Revision::new(0, SourceDigest::from_hex(&"0".repeat(64)));
        let transaction = Transaction::new("r-1", revision.clone(), Vec::new());
        assert_eq!(transaction.request_id, "r-1");
        assert_eq!(transaction.base_revision, revision);
        let value = serde_json::to_value(&transaction).unwrap();
        assert_eq!(value["request_id"], "r-1");
    }
}
