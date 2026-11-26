#![forbid(unsafe_code)]

//! Composition Designer v0.1 Core：声明解码、引用/结构/依赖校验与规范化模型。
//!
//! 007 把 006 的 `LoadedWorkspace` 转成 `CompositionModel`；所有入口都只读，不写 TOML。

mod decode;
mod source_map;
mod validate;

pub use source_map::SourceMap;
pub use validate::{validate_loaded, validate_model, validate_project, ValidationResult};
