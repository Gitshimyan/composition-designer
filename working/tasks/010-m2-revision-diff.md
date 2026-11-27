# 010 — M2 Revision、幂等与结构化 Diff

- 状态：done
- 目标：在 009 的候选事务上补齐 Revision 历史、`base_revision` 冲突语义、request_id 幂等
  与结构化 Diff/影响分析；仍不写 TOML。
- 授权：Modify（editor/core 文档与测试）
- 变更声明：`changes/010-revision-diff.yaml`

> 授权审计（2026-11-27）：审计建议 **Modify + Rearchitect**（事务历史/幂等/Diff 契约）；Manifest 的 `expected_command_changes=true` 与授权记录不一致，仅记录不追溯改写。见 [审计记录](../../docs/reviews/2026-11-27-authorization-audit-record.md)。

## 背景

009 已有命令和内存原子事务，但 Revision 只递增 `seq`、没有历史；同一 request_id 未处理；
调用方看不到修改了哪些对象。010 补齐 M2 的并发与可解释性：

- Revision 历史在工作区内保存，供冲突判断与调用方查询；
- 同一 request_id + 同一事务 → 返回原结果，不重复执行；
- 同一 request_id + 不同载荷 → `edit-duplicate-request`；
- 成功事务返回结构化 Diff（实例/Binding/Connection 的 added/removed/modified）。

磁盘持久化与 TOML 写回仍由 011 完成；`source_digest` 在 010 中保持读取值。

## 清单

- [x] `RevisionRecord`：revision / request_id / diff
- [x] `EditorWorkspace` 保存 history 与已提交 request 记录
- [x] `base_revision` 必须存在于历史且等于当前 Revision
- [x] 同一 request + 同一事务 → `Replayed`，不再递增 seq
- [x] 同一 request + 不同事务 → `edit-duplicate-request`
- [x] `ModelDiff`：实例/Binding/Connection 的规范化 JSON before/after 与 summary
- [x] `TransactionOutcome` 暴露 diff；`prepare` 也返回 diff 但不改工作区
- [x] 测试：配置 Diff、创建对象 Diff、重放、重复 request 不同载荷、提交后旧 Revision、历史记录
- [x] `cargo test` / `clippy` / `fmt` 全绿；更新 command schema、plan、README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**123 passed / 0 failed**
  （domain 58 + toml 12/15 + core 8 + cli 8 + editor 4/18）；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check` 通过；
- `set_config` 的 Diff：1 个 modified 实例，before/after 都有值；
- 创建实例 + Binding：summary.added = 2，Diff 含对应 object_id；
- 同一 request 重放：`is_replayed()`，seq 不再增加，历史长度不变；
- 同一 request 不同载荷：`edit-duplicate-request`；
- 提交后使用旧 base revision：`edit-revision-conflict`；
- 历史包含 initial（seq 0）与每次成功提交的 Revision/request/diff；
- `prepare` 返回 Diff 但不改变模型或历史。

## 结果 / 遗留问题

- 结果：
  - `RevisionRecord` 历史（initial + 每次提交）；
  - request_id → 已提交事务/模型/诊断/Diff 映射；
  - 同事务重放返回 `TransactionOutcome::Replayed`，不递增 seq；
  - 不同载荷复用 request_id → `edit-duplicate-request`；
  - `ModelDiff::between` 输出实例/Binding/Connection 的 added/removed/modified 与 summary；
  - 4 个 editor 单元 + 18 个集成测试，workspace 123 项测试。
- 关键决策：
  - 幂等只记录成功事务；失败事务可用同一 request 重试；
  - 重放返回原结果（可能是旧 Revision），不覆盖当前工作区；
  - Diff 用对象规范化 JSON 的 before/after，重命名表现为 remove + add（对象 ID 变化），
    引用对象（Binding）表现为 modified；
  - 历史与 request 记录只在内存；011 可随 TOML 一起持久化。
- 遗留：
  - 历史尚未写盘，进程重启丢失；request_id 幂等只跨同一 workspace 实例；
  - Diff 未包含依赖解析结果和模块定义（模块只读）；
  - `source_digest` 仍未随模型变化刷新（011）；
  - 没有撤销/重做栈。
- 下一步：011（保留格式 TOML 编辑、落盘、`source_digest` 刷新）。
