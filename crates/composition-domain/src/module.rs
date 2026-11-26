use std::str::FromStr;

use serde::Serialize;

use crate::contract::Contract;
use crate::error::{DomainError, DomainResult};
use crate::error_code::ErrorCode;
use crate::ids::{ModuleId, Version};
use crate::source::SourceRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    Composite,
    Primitive,
}

impl ModuleKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Composite => "composite",
            Self::Primitive => "primitive",
        }
    }

    pub fn parse(value: &str) -> DomainResult<Self> {
        match value {
            "composite" => Ok(Self::Composite),
            "primitive" => Ok(Self::Primitive),
            other => Err(DomainError::new(
                ErrorCode::SchemaInvalidValue,
                format!("unknown module kind {other:?}"),
            )),
        }
    }
}

impl FromStr for ModuleKind {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModuleDefinition {
    pub id: ModuleId,
    pub version: Version,
    pub kind: ModuleKind,
    pub description: Option<String>,
    pub contract: Contract,
    pub source: Option<SourceRef>,
}

impl ModuleDefinition {
    pub fn new(id: ModuleId, version: Version, kind: ModuleKind, contract: Contract) -> Self {
        Self {
            id,
            version,
            kind,
            description: None,
            contract,
            source: None,
        }
    }

    pub fn check(&self) -> Vec<DomainError> {
        let mut errors = Vec::new();
        if self
            .description
            .as_deref()
            .is_some_and(|summary| summary.trim().is_empty())
        {
            errors.push(DomainError::new(
                ErrorCode::SchemaEmptyValue,
                "module summary must not be empty",
            ));
        }
        if self.kind == ModuleKind::Primitive && !self.contract.slots().is_empty() {
            errors.push(DomainError::new(
                ErrorCode::ContractModuleInvalid,
                "primitive module must not declare slots",
            ));
        }
        errors.extend(self.contract.check());
        errors
    }

    pub fn normalize(&mut self) {
        self.contract.normalize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{PropertyContract, PropertyType, SlotCardinality, SlotContract};
    use crate::ids::{LocalId, ModuleId};

    fn module(id: &str, version: &str, kind: ModuleKind) -> ModuleDefinition {
        ModuleDefinition::new(
            ModuleId::parse(id).unwrap(),
            Version::parse(version).unwrap(),
            kind,
            Contract::new(),
        )
    }

    #[test]
    fn primitive_must_not_declare_slots() {
        let mut definition = module("leaf.widget", "1.0.0", ModuleKind::Primitive);
        definition
            .contract
            .insert_slot(
                LocalId::parse("content").unwrap(),
                SlotContract::new(SlotCardinality::Many, false, vec![]),
            )
            .unwrap();
        let codes: Vec<ErrorCode> = definition.check().iter().map(DomainError::code).collect();
        assert!(codes.contains(&ErrorCode::ContractModuleInvalid));
    }

    #[test]
    fn composite_may_declare_slots() {
        let mut definition = module("shell.desktop", "1.0.0", ModuleKind::Composite);
        definition
            .contract
            .insert_slot(
                LocalId::parse("content").unwrap(),
                SlotContract::new(
                    SlotCardinality::One,
                    true,
                    vec![ModuleId::parse("leaf.widget").unwrap()],
                ),
            )
            .unwrap();
        assert!(definition.check().is_empty());
    }

    #[test]
    fn empty_summary_is_rejected() {
        let mut definition = module("shell.desktop", "1.0.0", ModuleKind::Composite);
        definition.description = Some("  ".into());
        let codes: Vec<ErrorCode> = definition.check().iter().map(DomainError::code).collect();
        assert_eq!(codes, [ErrorCode::SchemaEmptyValue]);
    }

    #[test]
    fn contract_errors_propagate() {
        let mut definition = module("shell.desktop", "1.0.0", ModuleKind::Composite);
        definition
            .contract
            .insert_property(
                LocalId::parse("title").unwrap(),
                PropertyContract::new(PropertyType::String, true),
            )
            .unwrap();
        // No default, so this contract is valid; adding an invalid constraint triggers it.
        let mut property = PropertyContract::new(PropertyType::Boolean, false);
        property.min = Some(crate::value::ConfigValue::Integer(1));
        definition
            .contract
            .insert_property(LocalId::parse("flag").unwrap(), property)
            .unwrap();
        let codes: Vec<ErrorCode> = definition.check().iter().map(DomainError::code).collect();
        assert_eq!(codes, [ErrorCode::SchemaConstraintMismatch]);
    }
}
