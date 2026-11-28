use std::fmt;

use serde::Serialize;

/// v0.1 的稳定错误代码。字符串形式见 `docs/spec/error-codes-v1.md`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    // L1 — files and syntax
    FileProjectMissing,
    FileCompositionMissing,
    FileModuleRootMissing,
    FileReadError,
    FilePathAbsolute,
    FilePathEscape,
    FilePathInvalid,
    TomlSyntaxError,
    TomlDuplicateKey,
    // L2 — schema
    SchemaMissingField,
    SchemaUnknownField,
    SchemaTypeMismatch,
    SchemaInvalidValue,
    SchemaInvalidIdentifier,
    SchemaInvalidVersion,
    SchemaUnsupportedVersion,
    SchemaUnsupportedFeature,
    SchemaPropertyTypeUnsupported,
    SchemaInconsistency,
    SchemaEmptyValue,
    SchemaDuplicateEntry,
    SchemaConstraintMismatch,
    SchemaVersionMismatch,
    // L3 — identity and references
    IdDuplicateModule,
    IdDuplicateInstance,
    IdDuplicateBinding,
    IdDuplicateConnection,
    IdDuplicateProperty,
    IdDuplicateSlot,
    IdDuplicatePort,
    IdDuplicateDependency,
    IdDuplicateInterface,
    RefModuleNotFound,
    RefModuleVersionNotFound,
    RefModuleAmbiguous,
    RefInstanceNotFound,
    RefSlotNotFound,
    RefPortNotFound,
    RefRootNotFound,
    RefInterfaceNotFound,
    // L4 — contracts and config
    ContractModuleInvalid,
    ContractPropertyUnknown,
    ContractPropertyRequiredMissing,
    ContractPropertyTypeMismatch,
    ContractPropertyValueInvalid,
    ContractSlotRejectsModule,
    ContractDependencyUnknown,
    ContractServiceModuleInvalid,
    // L5 — structure
    StructureRootNotComposite,
    StructureRootHasParent,
    StructureInstanceMultipleParents,
    StructureDuplicateBinding,
    StructureSlotCardinality,
    StructureCycle,
    StructureUnreachableInstance,
    StructureServiceInBinding,
    StructureRootScopeInvalid,
    // L6 — connections and dependencies
    ConnectionDirectionInvalid,
    ConnectionTypeMismatch,
    ConnectionProtocolMismatch,
    ConnectionDuplicate,
    ConnectionPortMultiViolation,
    DependencyUnresolved,
    DependencyOptionalUnresolved,
    DependencyAmbiguous,
    DependencyProviderInvalid,
    DependencySelf,
    DependencyInterfaceMismatch,
    ServiceUnused,
    // L7 — whole-project consistency
    CompositionProjectIdMismatch,
    CompositionInternalError,
    // reserved for M2/M3
    EditInvalidCommand,
    EditUnsupportedOperation,
    EditTransactionFailed,
    EditRevisionConflict,
    EditDuplicateRequest,
    WatchReloadFailed,
}

impl ErrorCode {
    pub const ALL: [Self; 77] = [
        Self::FileProjectMissing,
        Self::FileCompositionMissing,
        Self::FileModuleRootMissing,
        Self::FileReadError,
        Self::FilePathAbsolute,
        Self::FilePathEscape,
        Self::FilePathInvalid,
        Self::TomlSyntaxError,
        Self::TomlDuplicateKey,
        Self::SchemaMissingField,
        Self::SchemaUnknownField,
        Self::SchemaTypeMismatch,
        Self::SchemaInvalidValue,
        Self::SchemaInvalidIdentifier,
        Self::SchemaInvalidVersion,
        Self::SchemaUnsupportedVersion,
        Self::SchemaUnsupportedFeature,
        Self::SchemaPropertyTypeUnsupported,
        Self::SchemaInconsistency,
        Self::SchemaEmptyValue,
        Self::SchemaDuplicateEntry,
        Self::SchemaConstraintMismatch,
        Self::SchemaVersionMismatch,
        Self::IdDuplicateModule,
        Self::IdDuplicateInstance,
        Self::IdDuplicateBinding,
        Self::IdDuplicateConnection,
        Self::IdDuplicateProperty,
        Self::IdDuplicateSlot,
        Self::IdDuplicatePort,
        Self::IdDuplicateDependency,
        Self::IdDuplicateInterface,
        Self::RefModuleNotFound,
        Self::RefModuleVersionNotFound,
        Self::RefModuleAmbiguous,
        Self::RefInstanceNotFound,
        Self::RefSlotNotFound,
        Self::RefPortNotFound,
        Self::RefRootNotFound,
        Self::RefInterfaceNotFound,
        Self::ContractModuleInvalid,
        Self::ContractPropertyUnknown,
        Self::ContractPropertyRequiredMissing,
        Self::ContractPropertyTypeMismatch,
        Self::ContractPropertyValueInvalid,
        Self::ContractSlotRejectsModule,
        Self::ContractDependencyUnknown,
        Self::ContractServiceModuleInvalid,
        Self::StructureRootNotComposite,
        Self::StructureRootHasParent,
        Self::StructureInstanceMultipleParents,
        Self::StructureDuplicateBinding,
        Self::StructureSlotCardinality,
        Self::StructureCycle,
        Self::StructureUnreachableInstance,
        Self::StructureServiceInBinding,
        Self::StructureRootScopeInvalid,
        Self::ConnectionDirectionInvalid,
        Self::ConnectionTypeMismatch,
        Self::ConnectionProtocolMismatch,
        Self::ConnectionDuplicate,
        Self::ConnectionPortMultiViolation,
        Self::DependencyUnresolved,
        Self::DependencyOptionalUnresolved,
        Self::DependencyAmbiguous,
        Self::DependencyProviderInvalid,
        Self::DependencySelf,
        Self::DependencyInterfaceMismatch,
        Self::ServiceUnused,
        Self::CompositionProjectIdMismatch,
        Self::CompositionInternalError,
        Self::EditInvalidCommand,
        Self::EditUnsupportedOperation,
        Self::EditTransactionFailed,
        Self::EditRevisionConflict,
        Self::EditDuplicateRequest,
        Self::WatchReloadFailed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileProjectMissing => "file-project-missing",
            Self::FileCompositionMissing => "file-composition-missing",
            Self::FileModuleRootMissing => "file-module-root-missing",
            Self::FileReadError => "file-read-error",
            Self::FilePathAbsolute => "file-path-absolute",
            Self::FilePathEscape => "file-path-escape",
            Self::FilePathInvalid => "file-path-invalid",
            Self::TomlSyntaxError => "toml-syntax-error",
            Self::TomlDuplicateKey => "toml-duplicate-key",
            Self::SchemaMissingField => "schema-missing-field",
            Self::SchemaUnknownField => "schema-unknown-field",
            Self::SchemaTypeMismatch => "schema-type-mismatch",
            Self::SchemaInvalidValue => "schema-invalid-value",
            Self::SchemaInvalidIdentifier => "schema-invalid-identifier",
            Self::SchemaInvalidVersion => "schema-invalid-version",
            Self::SchemaUnsupportedVersion => "schema-unsupported-version",
            Self::SchemaUnsupportedFeature => "schema-unsupported-feature",
            Self::SchemaPropertyTypeUnsupported => "schema-property-type-unsupported",
            Self::SchemaInconsistency => "schema-inconsistency",
            Self::SchemaEmptyValue => "schema-empty-value",
            Self::SchemaDuplicateEntry => "schema-duplicate-entry",
            Self::SchemaConstraintMismatch => "schema-constraint-mismatch",
            Self::SchemaVersionMismatch => "schema-version-mismatch",
            Self::IdDuplicateModule => "id-duplicate-module",
            Self::IdDuplicateInstance => "id-duplicate-instance",
            Self::IdDuplicateBinding => "id-duplicate-binding",
            Self::IdDuplicateConnection => "id-duplicate-connection",
            Self::IdDuplicateProperty => "id-duplicate-property",
            Self::IdDuplicateSlot => "id-duplicate-slot",
            Self::IdDuplicatePort => "id-duplicate-port",
            Self::IdDuplicateDependency => "id-duplicate-dependency",
            Self::IdDuplicateInterface => "id-duplicate-interface",
            Self::RefModuleNotFound => "ref-module-not-found",
            Self::RefModuleVersionNotFound => "ref-module-version-not-found",
            Self::RefModuleAmbiguous => "ref-module-ambiguous",
            Self::RefInstanceNotFound => "ref-instance-not-found",
            Self::RefSlotNotFound => "ref-slot-not-found",
            Self::RefPortNotFound => "ref-port-not-found",
            Self::RefRootNotFound => "ref-root-not-found",
            Self::RefInterfaceNotFound => "ref-interface-not-found",
            Self::ContractModuleInvalid => "contract-module-invalid",
            Self::ContractPropertyUnknown => "contract-property-unknown",
            Self::ContractPropertyRequiredMissing => "contract-property-required-missing",
            Self::ContractPropertyTypeMismatch => "contract-property-type-mismatch",
            Self::ContractPropertyValueInvalid => "contract-property-value-invalid",
            Self::ContractSlotRejectsModule => "contract-slot-rejects-module",
            Self::ContractDependencyUnknown => "contract-dependency-unknown",
            Self::ContractServiceModuleInvalid => "contract-service-module-invalid",
            Self::StructureRootNotComposite => "structure-root-not-composite",
            Self::StructureRootHasParent => "structure-root-has-parent",
            Self::StructureInstanceMultipleParents => "structure-instance-multiple-parents",
            Self::StructureDuplicateBinding => "structure-duplicate-binding",
            Self::StructureSlotCardinality => "structure-slot-cardinality",
            Self::StructureCycle => "structure-cycle",
            Self::StructureUnreachableInstance => "structure-unreachable-instance",
            Self::StructureServiceInBinding => "structure-service-in-binding",
            Self::StructureRootScopeInvalid => "structure-root-scope-invalid",
            Self::ConnectionDirectionInvalid => "connection-direction-invalid",
            Self::ConnectionTypeMismatch => "connection-type-mismatch",
            Self::ConnectionProtocolMismatch => "connection-protocol-mismatch",
            Self::ConnectionDuplicate => "connection-duplicate",
            Self::ConnectionPortMultiViolation => "connection-port-multi-violation",
            Self::DependencyUnresolved => "dependency-unresolved",
            Self::DependencyOptionalUnresolved => "dependency-optional-unresolved",
            Self::DependencyAmbiguous => "dependency-ambiguous",
            Self::DependencyProviderInvalid => "dependency-provider-invalid",
            Self::DependencySelf => "dependency-self",
            Self::DependencyInterfaceMismatch => "dependency-interface-mismatch",
            Self::ServiceUnused => "service-unused",
            Self::CompositionProjectIdMismatch => "composition-project-id-mismatch",
            Self::CompositionInternalError => "composition-internal-error",
            Self::EditInvalidCommand => "edit-invalid-command",
            Self::EditUnsupportedOperation => "edit-unsupported-operation",
            Self::EditTransactionFailed => "edit-transaction-failed",
            Self::EditRevisionConflict => "edit-revision-conflict",
            Self::EditDuplicateRequest => "edit-duplicate-request",
            Self::WatchReloadFailed => "watch-reload-failed",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }

    pub fn layer(self) -> ErrorLayer {
        match self {
            Self::FileProjectMissing
            | Self::FileCompositionMissing
            | Self::FileModuleRootMissing
            | Self::FileReadError
            | Self::FilePathAbsolute
            | Self::FilePathEscape
            | Self::FilePathInvalid
            | Self::TomlSyntaxError
            | Self::TomlDuplicateKey => ErrorLayer::L1,
            Self::SchemaMissingField
            | Self::SchemaUnknownField
            | Self::SchemaTypeMismatch
            | Self::SchemaInvalidValue
            | Self::SchemaInvalidIdentifier
            | Self::SchemaInvalidVersion
            | Self::SchemaUnsupportedVersion
            | Self::SchemaUnsupportedFeature
            | Self::SchemaPropertyTypeUnsupported
            | Self::SchemaInconsistency
            | Self::SchemaEmptyValue
            | Self::SchemaDuplicateEntry
            | Self::SchemaConstraintMismatch
            | Self::SchemaVersionMismatch => ErrorLayer::L2,
            Self::IdDuplicateModule
            | Self::IdDuplicateInstance
            | Self::IdDuplicateBinding
            | Self::IdDuplicateConnection
            | Self::IdDuplicateProperty
            | Self::IdDuplicateSlot
            | Self::IdDuplicatePort
            | Self::IdDuplicateDependency
            | Self::IdDuplicateInterface
            | Self::RefModuleNotFound
            | Self::RefModuleVersionNotFound
            | Self::RefModuleAmbiguous
            | Self::RefInstanceNotFound
            | Self::RefSlotNotFound
            | Self::RefPortNotFound
            | Self::RefRootNotFound
            | Self::RefInterfaceNotFound => ErrorLayer::L3,
            Self::ContractModuleInvalid
            | Self::ContractPropertyUnknown
            | Self::ContractPropertyRequiredMissing
            | Self::ContractPropertyTypeMismatch
            | Self::ContractPropertyValueInvalid
            | Self::ContractSlotRejectsModule
            | Self::ContractDependencyUnknown
            | Self::ContractServiceModuleInvalid => ErrorLayer::L4,
            Self::StructureRootNotComposite
            | Self::StructureRootHasParent
            | Self::StructureInstanceMultipleParents
            | Self::StructureDuplicateBinding
            | Self::StructureSlotCardinality
            | Self::StructureCycle
            | Self::StructureUnreachableInstance
            | Self::StructureServiceInBinding
            | Self::StructureRootScopeInvalid => ErrorLayer::L5,
            Self::ConnectionDirectionInvalid
            | Self::ConnectionTypeMismatch
            | Self::ConnectionProtocolMismatch
            | Self::ConnectionDuplicate
            | Self::ConnectionPortMultiViolation
            | Self::DependencyUnresolved
            | Self::DependencyOptionalUnresolved
            | Self::DependencyAmbiguous
            | Self::DependencyProviderInvalid
            | Self::DependencySelf
            | Self::DependencyInterfaceMismatch
            | Self::ServiceUnused => ErrorLayer::L6,
            Self::CompositionProjectIdMismatch | Self::CompositionInternalError => ErrorLayer::L7,
            Self::EditInvalidCommand
            | Self::EditUnsupportedOperation
            | Self::EditTransactionFailed
            | Self::EditRevisionConflict
            | Self::EditDuplicateRequest => ErrorLayer::Edit,
            Self::WatchReloadFailed => ErrorLayer::Watch,
        }
    }

    pub fn severity(self) -> Severity {
        match self {
            Self::DependencyOptionalUnresolved | Self::ServiceUnused => Severity::Warning,
            _ => Severity::Error,
        }
    }

    pub fn is_reserved(self) -> bool {
        matches!(
            self,
            Self::EditInvalidCommand
                | Self::EditUnsupportedOperation
                | Self::EditTransactionFailed
                | Self::EditRevisionConflict
                | Self::EditDuplicateRequest
                | Self::WatchReloadFailed
        )
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorLayer {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    Edit,
    Watch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_codes_are_listed_once() {
        assert_eq!(ErrorCode::ALL.len(), 77);
        let unique: HashSet<&str> = ErrorCode::ALL.iter().map(|code| code.as_str()).collect();
        assert_eq!(unique.len(), 77);
    }

    #[test]
    fn validation_and_reserved_counts_match_spec() {
        let validation = ErrorCode::ALL
            .iter()
            .filter(|code| !code.is_reserved())
            .count();
        let reserved = ErrorCode::ALL
            .iter()
            .filter(|code| code.is_reserved())
            .count();
        assert_eq!(validation, 71);
        assert_eq!(reserved, 6);
    }

    #[test]
    fn string_round_trip_works_for_every_code() {
        for code in ErrorCode::ALL {
            assert_eq!(ErrorCode::from_code(code.as_str()), Some(code));
        }
        assert_eq!(ErrorCode::from_code("not-a-code"), None);
    }

    #[test]
    fn layers_follow_the_spec() {
        assert_eq!(ErrorCode::TomlSyntaxError.layer(), ErrorLayer::L1);
        assert_eq!(ErrorCode::SchemaInconsistency.layer(), ErrorLayer::L2);
        assert_eq!(ErrorCode::SchemaVersionMismatch.layer(), ErrorLayer::L2);
        assert_eq!(ErrorCode::RefModuleNotFound.layer(), ErrorLayer::L3);
        assert_eq!(ErrorCode::RefInterfaceNotFound.layer(), ErrorLayer::L3);
        assert_eq!(ErrorCode::ContractPropertyUnknown.layer(), ErrorLayer::L4);
        assert_eq!(
            ErrorCode::ContractServiceModuleInvalid.layer(),
            ErrorLayer::L4
        );
        assert_eq!(ErrorCode::StructureCycle.layer(), ErrorLayer::L5);
        assert_eq!(ErrorCode::StructureServiceInBinding.layer(), ErrorLayer::L5);
        assert_eq!(ErrorCode::ConnectionTypeMismatch.layer(), ErrorLayer::L6);
        assert_eq!(
            ErrorCode::DependencyInterfaceMismatch.layer(),
            ErrorLayer::L6
        );
        assert_eq!(
            ErrorCode::CompositionProjectIdMismatch.layer(),
            ErrorLayer::L7
        );
        assert_eq!(ErrorCode::EditRevisionConflict.layer(), ErrorLayer::Edit);
        assert_eq!(ErrorCode::WatchReloadFailed.layer(), ErrorLayer::Watch);
    }

    #[test]
    fn optional_dependency_and_unused_service_are_warnings() {
        for code in ErrorCode::ALL {
            let expected = if matches!(
                code,
                ErrorCode::DependencyOptionalUnresolved | ErrorCode::ServiceUnused
            ) {
                Severity::Warning
            } else {
                Severity::Error
            };
            assert_eq!(code.severity(), expected, "{code}");
        }
    }

    #[test]
    fn reserved_codes_are_exactly_m2_m3_codes() {
        let reserved: Vec<&str> = ErrorCode::ALL
            .iter()
            .filter(|code| code.is_reserved())
            .map(|code| code.as_str())
            .collect();
        assert_eq!(
            reserved,
            [
                "edit-invalid-command",
                "edit-unsupported-operation",
                "edit-transaction-failed",
                "edit-revision-conflict",
                "edit-duplicate-request",
                "watch-reload-failed",
            ]
        );
    }
}
