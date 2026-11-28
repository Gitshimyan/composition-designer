#![forbid(unsafe_code)]

//! Composition Designer v0.1 领域模型。
//!
//! 本 crate 只包含领域类型、稳定错误代码与不变量检查；不读取文件、不解析 TOML、
//! 不依赖 GUI 或协议实现。规范见 `docs/spec/domain-model-v1.md` 与
//! `docs/spec/composition-schema-v1.md`。

pub mod composition;
pub mod contract;
pub mod diagnostic;
pub mod error;
pub mod error_code;
pub mod ids;
pub mod model;
pub mod module;
pub mod revision;
pub mod schema;
pub mod source;
pub mod value;

pub use composition::{
    Binding, Composition, Connection, Instance, InstanceScope, ModuleRef, PortRef,
};
pub use contract::{
    Contract, DependencyRequirement, DependencyTarget, InterfaceDeclaration, PortContract,
    PortDirection, PropertyContract, PropertyType, SlotCardinality, SlotContract,
};
pub use diagnostic::{Diagnostic, ObjectId};
pub use error::{DomainError, DomainResult};
pub use error_code::{ErrorCode, ErrorLayer, Severity};
pub use ids::{InterfaceId, LocalId, ModuleId, TypeId, Version};
pub use model::{
    CompositionModel, DependencyResolution, DependencyStatus, DependencyTargetInfo,
    DependencyTargetKind, Project,
};
pub use module::{ModuleDefinition, ModuleKind};
pub use revision::{Revision, SourceDigest};
pub use schema::SchemaVersion;
pub use source::{Position, SourceRange, SourceRef};
pub use value::ConfigValue;
