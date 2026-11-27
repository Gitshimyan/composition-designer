# 机器授权/覆盖门禁与 PR 强制门禁（2026-11-27）

- 任务：G04 + UW-029 + G05（block 模式）
- 状态：`done`
- coverage_status：`complete`
- open_items：无
- 关联：[`working/plan.md`](../../working/plan.md)、[`working/tasks/G04-machine-gate.md`](../../working/tasks/G04-machine-gate.md)、[`working/tasks/G05-gate-block.md`](../../working/tasks/G05-gate-block.md)

## 1. 交付

### `composition-change-gate`

位置：[`tools/change-gate`](../../tools/change-gate/README.md)。

- 读取 `changes/*.yaml` 与 `git diff`；
- 校验 targets、required/approved 授权、operation、敏感路径、
  Compose/Extend/forbidden、`coverage_status` 与 `unresolved_items`；
- 支持 `--mode report`（永远 0）与 `--mode block`（错误时 1）；
- 7 个单元测试 + 1 个真实仓库 report 集成测试。

CI：

- checkout 改为 `fetch-depth: 0`；
- 新增 `Change gate (block)` 步骤；声明/授权/覆盖不一致会使 required check 失败；
- 重叠 Manifest：一个文件被多个 `targets` 覆盖时，任一覆盖 Manifest 满足约束即通过
  （G05；修复 README 等宽 target 的误报）。

### PR 强制门禁（UW-029）

`main` 分支保护已启用：

- required status check：`Windows build and UI tests`（strict）；
- `enforce_admins=true`；
- 禁止 force push / 删除；
- PR 需要 review 流程（0 approvals，但要求 PR）。

验证探针：

- 分支 `ci-gate-probe` 引入一个故意失败的测试；
- PR #1 CI 结果：**fail**；
- PR `mergeStateStatus`：**BLOCKED**；
- 已关闭 PR 并删除探针分支。

集成 PR：

- PR #2（`gov/m6-g04`）CI 通过（run 45997141339），已合并到 `main`；
- 合并后 main push CI 通过（run 45997795133）；
- 远端仅保留 `main` 分支。

## 2. 报告结果

对当前分支全量 diff（`65ac30c..HEAD`）运行 report：

- 72 个 changed files；
- 0 error、0 warning（G04 结卡后）；
- block 模式在已知 coverage mismatch 上退出 1，证明阻断逻辑有效。

## 3. 限制

- 机器门禁只检查声明与 diff 的一致性；不执行 Core 语义校验，不自动授予权限；
- 新 crate/command/Adapter 的识别基于路径启发式，不是 AST 分析；
- legacy Manifest 没有 `coverage_status` 时不做 coverage 检查；touch 时再补；
- block 模式已启用；声明违规会让 CI 失败并阻止合并。

## 4. 结论

- UW-010（机器门禁）关闭；
- UW-029（PR 分支保护/失败合并阻止）关闭；
- UW-030（CI block 模式）由 G05 关闭；
- M6 完成标准三项全部通过。
