use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::error::{DomainError, DomainResult};
use crate::error_code::ErrorCode;
use crate::ids::{LocalId, ModuleId, TypeId, Version};
use crate::value::ConfigValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Boolean,
    Enum,
    Array,
}

impl PropertyType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::Enum => "enum",
            Self::Array => "array",
        }
    }

    pub fn parse(value: &str) -> DomainResult<Self> {
        match value {
            "string" => Ok(Self::String),
            "integer" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            "boolean" => Ok(Self::Boolean),
            "enum" => Ok(Self::Enum),
            "array" => Ok(Self::Array),
            "object" => Err(DomainError::new(
                ErrorCode::SchemaPropertyTypeUnsupported,
                "object properties are not supported in v0.1",
            )),
            other => Err(DomainError::new(
                ErrorCode::SchemaInvalidValue,
                format!("unknown property type {other:?}"),
            )),
        }
    }
}

impl FromStr for PropertyType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotCardinality {
    One,
    OptionalOne,
    Many,
}

impl SlotCardinality {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::OptionalOne => "optional_one",
            Self::Many => "many",
        }
    }

    pub fn parse(value: &str) -> DomainResult<Self> {
        match value {
            "one" => Ok(Self::One),
            "optional_one" => Ok(Self::OptionalOne),
            "many" => Ok(Self::Many),
            other => Err(DomainError::new(
                ErrorCode::SchemaInvalidValue,
                format!("unknown slot cardinality {other:?}"),
            )),
        }
    }
}

impl FromStr for SlotCardinality {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
}

impl PortDirection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }

    pub fn parse(value: &str) -> DomainResult<Self> {
        match value {
            "input" => Ok(Self::Input),
            "output" => Ok(Self::Output),
            other => Err(DomainError::new(
                ErrorCode::SchemaInvalidValue,
                format!("unknown port direction {other:?}"),
            )),
        }
    }
}

impl FromStr for PortDirection {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// 单个属性的契约。字段可在解析阶段直接构造，`check` 负责自洽校验。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PropertyContract {
    #[serde(rename = "type")]
    pub property_type: PropertyType,
    pub required: bool,
    pub default: Option<ConfigValue>,
    pub description: Option<String>,
    #[serde(rename = "values")]
    pub enum_values: Vec<String>,
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub min: Option<ConfigValue>,
    pub max: Option<ConfigValue>,
    pub min_items: Option<u64>,
    pub max_items: Option<u64>,
    pub items: Option<Box<PropertyContract>>,
}

impl PropertyContract {
    pub fn new(property_type: PropertyType, required: bool) -> Self {
        Self {
            property_type,
            required,
            default: None,
            description: None,
            enum_values: Vec::new(),
            min_length: None,
            max_length: None,
            min: None,
            max: None,
            min_items: None,
            max_items: None,
            items: None,
        }
    }

    /// 契约自洽检查（L2/L4 的契约部分）。
    pub fn check(&self) -> Vec<DomainError> {
        let mut errors = Vec::new();
        if self.required && self.default.is_some() {
            errors.push(DomainError::new(
                ErrorCode::SchemaInconsistency,
                "required property must not declare a default",
            ));
        }

        match self.property_type {
            PropertyType::String => {
                if self.has_non_string_constraints() {
                    errors.push(constraint_mismatch(
                        "string property has non-string constraints",
                    ));
                }
                if exceeds(self.min_length, self.max_length) {
                    errors.push(length_bounds_inconsistency());
                }
            }
            PropertyType::Integer | PropertyType::Float => {
                if !self.enum_values.is_empty()
                    || self.min_length.is_some()
                    || self.max_length.is_some()
                    || self.items.is_some()
                    || self.min_items.is_some()
                    || self.max_items.is_some()
                {
                    errors.push(constraint_mismatch(
                        "numeric property has constraints of another type",
                    ));
                }
                self.check_numeric_bounds(&mut errors);
            }
            PropertyType::Boolean => {
                if self.has_any_constraint() {
                    errors.push(constraint_mismatch(
                        "boolean property must not declare constraints",
                    ));
                }
            }
            PropertyType::Enum => {
                if self.min.is_some()
                    || self.max.is_some()
                    || self.min_length.is_some()
                    || self.max_length.is_some()
                    || self.items.is_some()
                    || self.min_items.is_some()
                    || self.max_items.is_some()
                {
                    errors.push(constraint_mismatch(
                        "enum property has non-enum constraints",
                    ));
                }
                self.check_enum_values(&mut errors);
            }
            PropertyType::Array => {
                if !self.enum_values.is_empty()
                    || self.min.is_some()
                    || self.max.is_some()
                    || self.min_length.is_some()
                    || self.max_length.is_some()
                {
                    errors.push(constraint_mismatch("array property has scalar constraints"));
                }
                if exceeds(self.min_items, self.max_items) {
                    errors.push(size_bounds_inconsistency());
                }
                self.check_array_items(&mut errors);
            }
        }

        if let Some(default) = &self.default {
            if let Err(error) = self.check_value_with(
                default,
                ErrorCode::SchemaInvalidValue,
                ErrorCode::SchemaInvalidValue,
            ) {
                errors.push(error);
            }
        }
        errors
    }

    /// 实例配置值检查（L4）。
    pub fn check_value(&self, value: &ConfigValue) -> DomainResult<()> {
        self.check_value_with(
            value,
            ErrorCode::ContractPropertyTypeMismatch,
            ErrorCode::ContractPropertyValueInvalid,
        )
    }

    fn check_value_with(
        &self,
        value: &ConfigValue,
        type_code: ErrorCode,
        invalid_code: ErrorCode,
    ) -> DomainResult<()> {
        match self.property_type {
            PropertyType::String => match value {
                ConfigValue::String(text) => self.check_string(text, invalid_code),
                other => Err(type_mismatch("string", other, type_code)),
            },
            PropertyType::Integer => match value {
                ConfigValue::Integer(number) => self.check_integer(*number, invalid_code),
                other => Err(type_mismatch("integer", other, type_code)),
            },
            PropertyType::Float => match value {
                ConfigValue::Float(number) => self.check_float(*number, invalid_code),
                other => Err(type_mismatch("float", other, type_code)),
            },
            PropertyType::Boolean => match value {
                ConfigValue::Boolean(_) => Ok(()),
                other => Err(type_mismatch("boolean", other, type_code)),
            },
            PropertyType::Enum => match value {
                ConfigValue::String(text) => {
                    if self.enum_values.iter().any(|candidate| candidate == text) {
                        Ok(())
                    } else {
                        Err(invalid_value(
                            invalid_code,
                            format!("{text:?} is not an enum value"),
                        ))
                    }
                }
                other => Err(type_mismatch("enum string", other, type_code)),
            },
            PropertyType::Array => match value {
                ConfigValue::Array(values) => {
                    let count = values.len() as u64;
                    if self.min_items.is_some_and(|min| count < min)
                        || self.max_items.is_some_and(|max| count > max)
                    {
                        return Err(invalid_value(
                            invalid_code,
                            format!("array length {count} violates item bounds"),
                        ));
                    }
                    let items = self.items.as_ref().ok_or_else(|| {
                        invalid_value(invalid_code, "array property has no item contract")
                    })?;
                    for value in values {
                        items.check_value_with(value, type_code, invalid_code)?;
                    }
                    Ok(())
                }
                other => Err(type_mismatch("array", other, type_code)),
            },
        }
    }

    fn check_string(&self, text: &str, code: ErrorCode) -> DomainResult<()> {
        let length = text.chars().count() as u64;
        if self.min_length.is_some_and(|min| length < min)
            || self.max_length.is_some_and(|max| length > max)
        {
            return Err(invalid_value(
                code,
                format!("string length {length} violates length bounds"),
            ));
        }
        Ok(())
    }

    fn check_integer(&self, number: i64, code: ErrorCode) -> DomainResult<()> {
        let too_small = match self.min {
            Some(ConfigValue::Integer(min)) => number < min,
            _ => false,
        };
        let too_large = match self.max {
            Some(ConfigValue::Integer(max)) => number > max,
            _ => false,
        };
        if too_small || too_large {
            return Err(invalid_value(
                code,
                format!("integer {number} violates numeric bounds"),
            ));
        }
        Ok(())
    }

    fn check_float(&self, number: f64, code: ErrorCode) -> DomainResult<()> {
        if !number.is_finite() {
            return Err(invalid_value(code, "float value must be finite"));
        }
        let too_small = match self.min {
            Some(ConfigValue::Float(min)) => number < min,
            _ => false,
        };
        let too_large = match self.max {
            Some(ConfigValue::Float(max)) => number > max,
            _ => false,
        };
        if too_small || too_large {
            return Err(invalid_value(
                code,
                format!("float {number} violates numeric bounds"),
            ));
        }
        Ok(())
    }

    fn has_any_constraint(&self) -> bool {
        !self.enum_values.is_empty()
            || self.min_length.is_some()
            || self.max_length.is_some()
            || self.min.is_some()
            || self.max.is_some()
            || self.min_items.is_some()
            || self.max_items.is_some()
            || self.items.is_some()
    }

    fn has_non_string_constraints(&self) -> bool {
        !self.enum_values.is_empty()
            || self.min.is_some()
            || self.max.is_some()
            || self.min_items.is_some()
            || self.max_items.is_some()
            || self.items.is_some()
    }

    fn check_numeric_bounds(&self, errors: &mut Vec<DomainError>) {
        let expected_float = self.property_type == PropertyType::Float;
        for bound in [&self.min, &self.max].into_iter().flatten() {
            let matches = matches!(
                (expected_float, bound),
                (false, ConfigValue::Integer(_)) | (true, ConfigValue::Float(_))
            );
            if !matches {
                errors.push(constraint_mismatch(
                    "numeric bound does not match the property type",
                ));
            } else if let ConfigValue::Float(value) = bound {
                if !value.is_finite() {
                    errors.push(DomainError::new(
                        ErrorCode::SchemaInvalidValue,
                        "float bounds must be finite",
                    ));
                }
            }
        }
        let invalid = match (self.property_type, &self.min, &self.max) {
            (
                PropertyType::Integer,
                Some(ConfigValue::Integer(min)),
                Some(ConfigValue::Integer(max)),
            ) => min > max,
            (PropertyType::Float, Some(ConfigValue::Float(min)), Some(ConfigValue::Float(max))) => {
                min > max
            }
            _ => false,
        };
        if invalid {
            errors.push(DomainError::new(
                ErrorCode::SchemaInconsistency,
                "min must not be greater than max",
            ));
        }
    }

    fn check_enum_values(&self, errors: &mut Vec<DomainError>) {
        if self.enum_values.is_empty() {
            errors.push(DomainError::new(
                ErrorCode::SchemaEmptyValue,
                "enum property must declare at least one value",
            ));
            return;
        }
        if self.enum_values.iter().any(|value| value.trim().is_empty()) {
            errors.push(DomainError::new(
                ErrorCode::SchemaEmptyValue,
                "enum values must not be empty",
            ));
        }
        let mut seen = HashSet::new();
        let mut duplicate = false;
        for value in &self.enum_values {
            if !seen.insert(value.as_str()) {
                duplicate = true;
            }
        }
        if duplicate {
            errors.push(DomainError::new(
                ErrorCode::SchemaDuplicateEntry,
                "enum values must be unique",
            ));
        }
    }

    fn check_array_items(&self, errors: &mut Vec<DomainError>) {
        let Some(items) = &self.items else {
            errors.push(DomainError::new(
                ErrorCode::SchemaMissingField,
                "array property must declare items",
            ));
            return;
        };
        if items.property_type == PropertyType::Array {
            errors.push(constraint_mismatch(
                "nested array properties are not supported",
            ));
            return;
        }
        if items.required {
            errors.push(constraint_mismatch("array items must not be required"));
        }
        if items.default.is_some() {
            errors.push(constraint_mismatch(
                "array items must not declare a default",
            ));
        }
        if items.items.is_some() || items.min_items.is_some() || items.max_items.is_some() {
            errors.push(constraint_mismatch("array items are scalar-only"));
        }
        for error in items.check() {
            errors.push(DomainError::new(
                error.code(),
                format!("array items: {}", error.message()),
            ));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SlotContract {
    pub cardinality: SlotCardinality,
    pub required: bool,
    pub accepts: Vec<ModuleId>,
}

impl SlotContract {
    pub fn new(cardinality: SlotCardinality, required: bool, accepts: Vec<ModuleId>) -> Self {
        Self {
            cardinality,
            required,
            accepts,
        }
    }

    pub fn check(&self) -> Vec<DomainError> {
        let mut errors = Vec::new();
        if self.accepts.is_empty() {
            errors.push(DomainError::new(
                ErrorCode::SchemaEmptyValue,
                "slot must accept at least one module",
            ));
        }
        let mut seen = HashSet::new();
        if self
            .accepts
            .iter()
            .any(|module| !seen.insert(module.as_str()))
        {
            errors.push(DomainError::new(
                ErrorCode::SchemaDuplicateEntry,
                "slot accepts entries must be unique",
            ));
        }
        let invalid_pair = matches!(
            (self.cardinality, self.required),
            (SlotCardinality::One, false) | (SlotCardinality::OptionalOne, true)
        );
        if invalid_pair {
            errors.push(DomainError::new(
                ErrorCode::SchemaInconsistency,
                "slot cardinality and required are inconsistent",
            ));
        }
        errors
    }

    pub fn accepts_module(&self, module: &ModuleId) -> bool {
        self.accepts.iter().any(|candidate| candidate == module)
    }

    pub fn binding_count_ok(&self, count: usize) -> bool {
        match self.cardinality {
            SlotCardinality::One => count == 1,
            SlotCardinality::OptionalOne => count <= 1,
            SlotCardinality::Many => !self.required || count >= 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PortContract {
    pub direction: PortDirection,
    #[serde(rename = "type")]
    pub type_id: TypeId,
    pub multi: bool,
    pub protocol: Option<String>,
}

impl PortContract {
    pub fn new(direction: PortDirection, type_id: TypeId) -> Self {
        Self {
            direction,
            type_id,
            multi: false,
            protocol: None,
        }
    }

    pub fn check(&self) -> Vec<DomainError> {
        let mut errors = Vec::new();
        if self
            .protocol
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(DomainError::new(
                ErrorCode::SchemaEmptyValue,
                "port protocol must not be empty",
            ));
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyRequirement {
    #[serde(rename = "module")]
    pub target: ModuleId,
    pub version: Option<Version>,
    pub optional: bool,
}

impl DependencyRequirement {
    pub fn new(target: ModuleId, version: Option<Version>, optional: bool) -> Self {
        Self {
            target,
            version,
            optional,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Contract {
    properties: BTreeMap<LocalId, PropertyContract>,
    slots: BTreeMap<LocalId, SlotContract>,
    ports: BTreeMap<LocalId, PortContract>,
    dependencies: BTreeMap<LocalId, DependencyRequirement>,
}

#[derive(Serialize)]
struct PropertyEntry<'a> {
    id: &'a LocalId,
    #[serde(flatten)]
    contract: &'a PropertyContract,
}

#[derive(Serialize)]
struct SlotEntry<'a> {
    id: &'a LocalId,
    #[serde(flatten)]
    contract: &'a SlotContract,
}

#[derive(Serialize)]
struct PortEntry<'a> {
    id: &'a LocalId,
    #[serde(flatten)]
    contract: &'a PortContract,
}

#[derive(Serialize)]
struct DependencyEntry<'a> {
    id: &'a LocalId,
    #[serde(flatten)]
    contract: &'a DependencyRequirement,
}

impl Serialize for Contract {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let properties: Vec<PropertyEntry<'_>> = self
            .properties
            .iter()
            .map(|(id, contract)| PropertyEntry { id, contract })
            .collect();
        let slots: Vec<SlotEntry<'_>> = self
            .slots
            .iter()
            .map(|(id, contract)| SlotEntry { id, contract })
            .collect();
        let ports: Vec<PortEntry<'_>> = self
            .ports
            .iter()
            .map(|(id, contract)| PortEntry { id, contract })
            .collect();
        let dependencies: Vec<DependencyEntry<'_>> = self
            .dependencies
            .iter()
            .map(|(id, contract)| DependencyEntry { id, contract })
            .collect();
        let mut state = serializer.serialize_struct("Contract", 4)?;
        state.serialize_field("properties", &properties)?;
        state.serialize_field("slots", &slots)?;
        state.serialize_field("ports", &ports)?;
        state.serialize_field("dependencies", &dependencies)?;
        state.end()
    }
}

impl Contract {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_property(&mut self, id: LocalId, property: PropertyContract) -> DomainResult<()> {
        if self.properties.contains_key(&id) {
            return Err(duplicate_id(
                ErrorCode::IdDuplicateProperty,
                "property",
                &id,
            ));
        }
        self.properties.insert(id, property);
        Ok(())
    }

    pub fn insert_slot(&mut self, id: LocalId, slot: SlotContract) -> DomainResult<()> {
        if self.slots.contains_key(&id) {
            return Err(duplicate_id(ErrorCode::IdDuplicateSlot, "slot", &id));
        }
        self.slots.insert(id, slot);
        Ok(())
    }

    pub fn insert_port(&mut self, id: LocalId, port: PortContract) -> DomainResult<()> {
        if self.ports.contains_key(&id) {
            return Err(duplicate_id(ErrorCode::IdDuplicatePort, "port", &id));
        }
        self.ports.insert(id, port);
        Ok(())
    }

    pub fn insert_dependency(
        &mut self,
        id: LocalId,
        dependency: DependencyRequirement,
    ) -> DomainResult<()> {
        if self.dependencies.contains_key(&id) {
            return Err(duplicate_id(
                ErrorCode::IdDuplicateDependency,
                "dependency",
                &id,
            ));
        }
        self.dependencies.insert(id, dependency);
        Ok(())
    }

    pub fn properties(&self) -> &BTreeMap<LocalId, PropertyContract> {
        &self.properties
    }

    pub fn slots(&self) -> &BTreeMap<LocalId, SlotContract> {
        &self.slots
    }

    pub fn ports(&self) -> &BTreeMap<LocalId, PortContract> {
        &self.ports
    }

    pub fn dependencies(&self) -> &BTreeMap<LocalId, DependencyRequirement> {
        &self.dependencies
    }

    pub fn property(&self, id: &LocalId) -> Option<&PropertyContract> {
        self.properties.get(id)
    }

    pub fn slot(&self, id: &LocalId) -> Option<&SlotContract> {
        self.slots.get(id)
    }

    pub fn port(&self, id: &LocalId) -> Option<&PortContract> {
        self.ports.get(id)
    }

    pub fn dependency(&self, id: &LocalId) -> Option<&DependencyRequirement> {
        self.dependencies.get(id)
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
            && self.slots.is_empty()
            && self.ports.is_empty()
            && self.dependencies.is_empty()
    }

    pub fn check(&self) -> Vec<DomainError> {
        let mut errors = Vec::new();
        for (id, property) in &self.properties {
            for error in property.check() {
                errors.push(with_context("property", id, error));
            }
        }
        for (id, slot) in &self.slots {
            for error in slot.check() {
                errors.push(with_context("slot", id, error));
            }
        }
        for (id, port) in &self.ports {
            for error in port.check() {
                errors.push(with_context("port", id, error));
            }
        }
        errors
    }

    /// 规范化集合顺序；不负责去重或修复非法数据。
    pub fn normalize(&mut self) {
        for property in self.properties.values_mut() {
            property.normalize();
        }
        for slot in self.slots.values_mut() {
            slot.accepts.sort();
        }
    }
}

impl PropertyContract {
    pub fn normalize(&mut self) {
        self.enum_values.sort();
        if let Some(items) = &mut self.items {
            items.normalize();
        }
    }
}

fn duplicate_id(code: ErrorCode, kind: &str, id: &LocalId) -> DomainError {
    DomainError::new(code, format!("duplicate {kind} id {id}"))
}

fn with_context(kind: &str, id: &LocalId, error: DomainError) -> DomainError {
    DomainError::new(error.code(), format!("{kind} {id}: {}", error.message()))
}

fn constraint_mismatch(message: &str) -> DomainError {
    DomainError::new(ErrorCode::SchemaConstraintMismatch, message)
}

fn length_bounds_inconsistency() -> DomainError {
    DomainError::new(
        ErrorCode::SchemaInconsistency,
        "min_length must not be greater than max_length",
    )
}

fn size_bounds_inconsistency() -> DomainError {
    DomainError::new(
        ErrorCode::SchemaInconsistency,
        "min_items must not be greater than max_items",
    )
}

fn exceeds(min: Option<u64>, max: Option<u64>) -> bool {
    matches!((min, max), (Some(min), Some(max)) if min > max)
}

fn type_mismatch(expected: &str, actual: &ConfigValue, code: ErrorCode) -> DomainError {
    DomainError::new(
        code,
        format!("expected {expected}, found {}", actual.type_name()),
    )
}

fn invalid_value(code: ErrorCode, message: impl Into<String>) -> DomainError {
    DomainError::new(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::ModuleId;

    fn local(value: &str) -> LocalId {
        LocalId::parse(value).unwrap()
    }

    fn module(value: &str) -> ModuleId {
        ModuleId::parse(value).unwrap()
    }

    fn codes(errors: &[DomainError]) -> Vec<ErrorCode> {
        errors.iter().map(DomainError::code).collect()
    }

    #[test]
    fn slot_matrix_is_enforced() {
        let accepts = vec![module("leaf.widget")];
        assert!(
            SlotContract::new(SlotCardinality::One, true, accepts.clone())
                .check()
                .is_empty()
        );
        assert_eq!(
            codes(&SlotContract::new(SlotCardinality::One, false, accepts.clone()).check()),
            [ErrorCode::SchemaInconsistency]
        );
        assert_eq!(
            codes(&SlotContract::new(SlotCardinality::OptionalOne, true, accepts.clone()).check()),
            [ErrorCode::SchemaInconsistency]
        );
        assert!(
            SlotContract::new(SlotCardinality::OptionalOne, false, accepts.clone())
                .check()
                .is_empty()
        );
        assert!(SlotContract::new(SlotCardinality::Many, true, accepts)
            .check()
            .is_empty());
    }

    #[test]
    fn slot_rejects_empty_and_duplicate_accepts() {
        assert_eq!(
            codes(&SlotContract::new(SlotCardinality::Many, false, vec![]).check()),
            [ErrorCode::SchemaEmptyValue]
        );
        let duplicate = vec![module("leaf.widget"), module("leaf.widget")];
        assert_eq!(
            codes(&SlotContract::new(SlotCardinality::Many, false, duplicate).check()),
            [ErrorCode::SchemaDuplicateEntry]
        );
    }

    #[test]
    fn slot_binding_counts_follow_cardinality() {
        let one = SlotContract::new(SlotCardinality::One, true, vec![module("a")]);
        assert!(one.binding_count_ok(1));
        assert!(!one.binding_count_ok(0));
        assert!(!one.binding_count_ok(2));

        let optional = SlotContract::new(SlotCardinality::OptionalOne, false, vec![module("a")]);
        assert!(optional.binding_count_ok(0));
        assert!(optional.binding_count_ok(1));
        assert!(!optional.binding_count_ok(2));

        let many_required = SlotContract::new(SlotCardinality::Many, true, vec![module("a")]);
        assert!(!many_required.binding_count_ok(0));
        assert!(many_required.binding_count_ok(1));
        assert!(many_required.binding_count_ok(9));
    }

    #[test]
    fn required_property_cannot_have_default() {
        let mut property = PropertyContract::new(PropertyType::String, true);
        property.default = Some(ConfigValue::String("x".into()));
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaInconsistency]);
    }

    #[test]
    fn string_bounds_must_be_ordered() {
        let mut property = PropertyContract::new(PropertyType::String, false);
        property.min_length = Some(5);
        property.max_length = Some(2);
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaInconsistency]);
    }

    #[test]
    fn boolean_property_rejects_scalar_constraints() {
        let mut property = PropertyContract::new(PropertyType::Boolean, false);
        property.min = Some(ConfigValue::Integer(1));
        assert_eq!(
            codes(&property.check()),
            [ErrorCode::SchemaConstraintMismatch]
        );
    }

    #[test]
    fn enum_requires_unique_non_empty_values() {
        let mut property = PropertyContract::new(PropertyType::Enum, false);
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaEmptyValue]);
        property.enum_values = vec!["light".into(), "light".into()];
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaDuplicateEntry]);
        property.enum_values = vec!["light".into(), "".into()];
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaEmptyValue]);
    }

    #[test]
    fn enum_default_must_be_a_declared_value() {
        let mut property = PropertyContract::new(PropertyType::Enum, false);
        property.enum_values = vec!["light".into(), "dark".into()];
        property.default = Some(ConfigValue::String("sepia".into()));
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaInvalidValue]);
        property.default = Some(ConfigValue::String("dark".into()));
        assert!(property.check().is_empty());
    }

    #[test]
    fn array_requires_items_and_rejects_nested_arrays() {
        let property = PropertyContract::new(PropertyType::Array, false);
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaMissingField]);

        let mut nested = PropertyContract::new(PropertyType::Array, false);
        nested.items = Some(Box::new(PropertyContract::new(PropertyType::Array, false)));
        assert_eq!(
            codes(&nested.check()),
            [ErrorCode::SchemaConstraintMismatch]
        );
    }

    #[test]
    fn array_items_cannot_be_required_or_defaulted() {
        let mut property = PropertyContract::new(PropertyType::Array, false);
        let mut items = PropertyContract::new(PropertyType::String, true);
        items.default = Some(ConfigValue::String("x".into()));
        property.items = Some(Box::new(items));
        let errors = codes(&property.check());
        assert!(errors.contains(&ErrorCode::SchemaConstraintMismatch));
        assert!(errors.contains(&ErrorCode::SchemaInconsistency));
    }

    #[test]
    fn config_values_are_checked_against_the_contract() {
        let mut width = PropertyContract::new(PropertyType::Integer, true);
        width.min = Some(ConfigValue::Integer(1));
        width.max = Some(ConfigValue::Integer(10));
        assert!(width.check_value(&ConfigValue::Integer(5)).is_ok());
        assert_eq!(
            width
                .check_value(&ConfigValue::Integer(0))
                .err()
                .map(|error| error.code()),
            Some(ErrorCode::ContractPropertyValueInvalid)
        );
        assert_eq!(
            width
                .check_value(&ConfigValue::String("5".into()))
                .err()
                .map(|error| error.code()),
            Some(ErrorCode::ContractPropertyTypeMismatch)
        );
    }

    #[test]
    fn arrays_check_each_item_type() {
        let mut tags = PropertyContract::new(PropertyType::Array, false);
        tags.items = Some(Box::new(PropertyContract::new(PropertyType::String, false)));
        assert!(tags
            .check_value(&ConfigValue::Array(vec![
                ConfigValue::String("a".into()),
                ConfigValue::String("b".into()),
            ]))
            .is_ok());
        assert_eq!(
            tags.check_value(&ConfigValue::Array(vec![ConfigValue::Integer(1)]))
                .err()
                .map(|error| error.code()),
            Some(ErrorCode::ContractPropertyTypeMismatch)
        );
    }

    #[test]
    fn contract_rejects_duplicate_contract_ids() {
        let mut contract = Contract::new();
        contract
            .insert_property(
                local("title"),
                PropertyContract::new(PropertyType::String, true),
            )
            .unwrap();
        let error = contract
            .insert_property(
                local("title"),
                PropertyContract::new(PropertyType::String, true),
            )
            .expect_err("duplicate property");
        assert_eq!(error.code(), ErrorCode::IdDuplicateProperty);
    }

    #[test]
    fn contract_normalizes_sets() {
        let mut contract = Contract::new();
        contract
            .insert_slot(
                local("content"),
                SlotContract::new(
                    SlotCardinality::Many,
                    false,
                    vec![module("z.widget"), module("a.widget")],
                ),
            )
            .unwrap();
        let mut property = PropertyContract::new(PropertyType::Enum, false);
        property.enum_values = vec!["z".into(), "a".into()];
        contract.insert_property(local("theme"), property).unwrap();
        contract.normalize();
        let slot = contract.slot(&local("content")).unwrap();
        let accepts: Vec<&str> = slot.accepts.iter().map(ModuleId::as_str).collect();
        assert_eq!(accepts, ["a.widget", "z.widget"]);
        let property = contract.property(&local("theme")).unwrap();
        assert_eq!(property.enum_values, ["a", "z"]);
    }

    #[test]
    fn enum_parsers_round_trip_and_reject_unknown_values() {
        for value in ["string", "integer", "float", "boolean", "enum", "array"] {
            let property_type = PropertyType::parse(value).unwrap();
            assert_eq!(property_type.as_str(), value);
        }
        assert_eq!(
            PropertyType::parse("object").unwrap_err().code(),
            ErrorCode::SchemaPropertyTypeUnsupported
        );
        assert_eq!(
            PropertyType::parse("map").unwrap_err().code(),
            ErrorCode::SchemaInvalidValue
        );
        for value in ["one", "optional_one", "many"] {
            assert_eq!(SlotCardinality::parse(value).unwrap().as_str(), value);
        }
        for value in ["input", "output"] {
            assert_eq!(PortDirection::parse(value).unwrap().as_str(), value);
        }
        assert_eq!(
            SlotCardinality::parse("optional").unwrap_err().code(),
            ErrorCode::SchemaInvalidValue
        );
    }

    #[test]
    fn float_bounds_must_be_finite() {
        let mut property = PropertyContract::new(PropertyType::Float, false);
        property.min = Some(ConfigValue::Float(f64::NAN));
        assert_eq!(codes(&property.check()), [ErrorCode::SchemaInvalidValue]);
    }

    #[test]
    fn port_protocol_must_not_be_empty() {
        let mut port = PortContract::new(PortDirection::Output, TypeId::parse("string").unwrap());
        assert!(port.check().is_empty());
        assert!(!port.multi);
        port.protocol = Some("".into());
        assert_eq!(codes(&port.check()), [ErrorCode::SchemaEmptyValue]);
    }
}
