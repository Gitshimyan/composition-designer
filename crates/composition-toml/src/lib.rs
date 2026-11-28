#![forbid(unsafe_code)]

//! 项目 TOML 的加载、源码定位与 `source_digest`。
//!
//! 本 crate 只负责“文件 → 可定位的 TOML 文档”；Composition / Module 的语义解码与
//! L3–L7 校验属于 `composition-resolver` / `composition-validator`（007）。

mod digest;
mod loaded_file;
mod loader;
mod paths;

pub use digest::compute_source_digest;
pub use loaded_file::{FileRole, LoadedFile};
pub use loader::{
    load_project, load_project_with_composition_text, load_project_with_profile,
    load_project_with_profile_and_composition_text, LoadedWorkspace,
};
pub use paths::normalize_relative_path;
