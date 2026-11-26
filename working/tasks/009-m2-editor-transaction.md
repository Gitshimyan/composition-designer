# 009 — M2 命令 Schema 与事务

- 状态：done
- 目标：新建 `composition-editor`，定义 Instance / Config / Binding / Connection 编辑命令，
  在候选模型上原子应用并复用 Core 做全量 L2–L7 校验；失败不改变工作区。
- 授权：Extend（新编辑器 crate）
- 变更声明：`changes/009-editor-transaction.yaml`

## 背景

M1 的 Core 只读。M2 要让人和 Agent 通过统一命令修改项目，而不是各自改 TOML。009 先冻结
命令形状与事务语义：命令只作用于模型，候选模型整体校验，成功才替换工作区；TOML 局部写回
与 Revision 持久化拆到 010 / 011。

决策：

- 009 **不写 TOML**；成功事务把模型 `seq` 加一，`source_digest` 保持读取值，011 落盘后重算。
- 模块定义在 v0.1 只读，不在命令范围。
- 命令级错误用 `edit-invalid-command`，候选模型错误保留真实 L2–L7 代码。
- 重建 Core 的候选校验入口 `validate_model`，避免编辑器和 CLI 各写一套规则。

## 清单

- [x] 根 workspace 加入 `crates/composition-editor`
- [x] `Command` 枚举：12 个命令 + serde `type` 标签
- [x] `Transaction`：request_id / base_revision / commands
- [x] `EditorWorkspace`：open、from_validation、prepare、apply、revision
- [x] 命令应用：实例增删改名/换模块、配置 set/unset、Binding 增删改、Connection 增删改
- [x] 删除实例级联清理 Binding / Connection / 依赖 provider；Root 拒绝
- [x] 重命名更新 root / Binding / Connection / 依赖 provider
- [x] `composition-core::validate_model`：候选模型 L2–L7 校验
- [x] 原子性：命令失败或候选校验失败时模型与 Revision 不变
- [x] 命令 Schema 文档 `docs/spec/command-schema-v1.md`
- [x] 测试：配置、重命名、增删级联、非法候选、重复 ID、旧 Revision、空请求、prepare 不变
- [x] `cargo test` / `clippy` / `fmt` 全绿；更新 plan、crates/README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**116 passed / 0 failed**
  （domain 58 + toml 12/15 + core 8 + cli 8 + editor 3/12）；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check` 通过；
- `set_config` 成功后模型与 `effective_config` 更新，`seq` 从 0 变 1；
- 非法配置、删除必需 Binding、删除被引用实例都被拒绝，工作区与 Revision 完全不变；
- 创建实例 + Binding 在同一事务成功；删除实例 + Binding 在同一事务成功；
- 重命名同时更新 Binding 的 child；
- 重复实例 ID → `edit-invalid-command`；旧 `base_revision` → `edit-revision-conflict`；
- `prepare` 返回候选但不改变工作区；
- 命令 JSON 带稳定 `type` 标签，`COMMAND_KINDS` 为 12 项。

## 结果 / 遗留问题

- 结果：新增 `crates/composition-editor`：
  - `Command` 12 个命令 + `Transaction`（request_id / base_revision / commands）；
  - `EditorWorkspace`：`open` / `prepare` / `apply`，候选模型全量校验后原子替换；
  - `composition-core::validate_model` 复用 007 的 L2–L7 校验；
  - 命令 Schema 文档与 3 个单元 + 12 个集成测试。
- 关键决策：
  - 删除实例级联清理 Binding/Connection/依赖 provider，是否合法交给候选校验；
  - 重命名同步更新 root 与所有引用；
  - `apply` 成功 `seq + 1`，`source_digest` 暂不刷新（011 负责）；
  - 拒绝结果返回真实候选诊断；命令级错误用 `edit-invalid-command`。
- 遗留：
  - 009 不写 TOML，也没有结构化 Diff / request ID 幂等（010 / 011）；
  - 新建对象的 SourceRef 为空，候选诊断主要靠对象 ID，落盘后重新解析才补齐；
  - 模块定义只读；没有撤销/事务历史；
  - `apply_command` 的错误类型较大，已局部 allow clippy；后续可改为 boxed error。
- 下一步：010（Revision 冲突、结构化 Diff、request ID 幂等）。
