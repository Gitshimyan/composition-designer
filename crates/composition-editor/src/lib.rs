#![forbid(unsafe_code)]

//! Composition Designer v0.1 编辑命令与原子事务。
//!
//! 009 只操作内存 `CompositionModel`：命令应用到候选模型，经 `composition-core`
//! 全量 L2–L7 校验后原子替换工作区；TOML 落盘与 `source_digest` 刷新属于 011。

mod command;
mod diff;
mod workspace;

pub use command::{Command, Transaction, COMMAND_KINDS};
pub use diff::{ChangeKind, DiffSummary, ModelDiff, ObjectChange};
pub use workspace::{EditorWorkspace, RevisionRecord, TransactionOutcome};
