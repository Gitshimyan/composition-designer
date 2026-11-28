# 035 — v2 数据集、迁移拒绝与 Glide 试点验收

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：补齐 v2 正反例数据集与迁移拒绝路径，在 Glide 试点 v2 副本上完成验收，并关闭 UW-002/UW-003、结束 S1。
- authorization：
  - required：modify、compose
  - approval：
    - modify：default
    - compose：default（examples/pilots 组合声明）
- 来源：`working/plan.md` S1 035；032–034
- 变更声明：`changes/035-v2-acceptance.yaml`

## 背景

033/034 已实现 v2 Core 与入口，但正式数据集仍只有 v1；迁移拒绝路径没有机器可读证据，
Glide 试点仍停在 v1。035 用真实声明数据与试点副本完成 S1 验收，不改规范或代码语义。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | v2 合法/非法数据集与 expected.json | implemented | `examples/v2/**`（11 个） |
| REQ-02 | 每个数据集的诊断集合稳定 | implemented | core `every_dataset_matches_expected_codes`（37 个） |
| REQ-03 | v1/v2 迁移拒绝路径 | implemented | `invalid-version-mismatch`、`invalid-v1-project-v2-module` |
| REQ-04 | Glide v2 试点（接口 + service） | implemented | `working/pilots/glide-task-terminal-v2` |
| REQ-05 | 试点 CLI 编辑与重开验收 | implemented | `crates/composition-cli/tests/v2_closeout.rs` |
| REQ-06 | 关闭 UW-002/UW-003，S1 收口 | implemented | 登记册、plan、D-025 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-032 | 接口方法/Port 形状、service 内部组合 | Rearchitect | v2.0 明确排除 | Rearchitect | 复杂接口契约与组合服务不可表达 | 出现真实需求时评估 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 035 验收 | 新增 11 个 v2 数据集 + v2 试点副本 | 需要可重复证据 | 032 §13 |

## 清单

- [x] v2 数据集与 README/expected.json
- [x] core 数据集计数与回归
- [x] 迁移拒绝 CLI 测试
- [x] Glide v2 试点与 CLI 编辑验收
- [x] UW-002/003 关闭与统计更新
- [x] plan/README/架构入口回写

## 验收

- 37 个数据集的诊断集合与 expected.json 一致；
- v1 拒绝 v2 模块、v2 项目拒绝 v1 Composition；
- Glide v2 试点 validate 通过：7 实例 / 3 Binding / 6 Connection / 3 接口依赖；
- 试点副本上新增 service + 显式 provider 后重开 valid；
- `cargo test --workspace` / clippy / fmt 通过；
- 变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - `examples/v2/**` 新增 11 个数据集：合法 service/interface、service-unused warning、
    未知/重复接口、双目标、service Binding、Root service、composite service、
    Composition 版本不匹配、provider 不匹配、v1 项目加载 v2 模块；
  - core 数据集总数 26 → 37，诊断集合逐目录校验；
  - Glide v2 试点：4 structure + 3 service、3 接口依赖、6 Connection、3 Binding；
  - CLI 验收：试点 validate/resolve；副本新增 service + 显式 provider 后重开 valid；
  - 迁移拒绝：CLI 校验 v1/v2 两个拒绝路径；
  - UW-002/UW-003 关闭，登记册 13 关闭 / 0 进行中。
- 验证：
  - `cargo test --workspace`：**199 passed**（新增 CLI v2_closeout 3 条）；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - `composition-cli validate working/pilots/glide-task-terminal-v2 --json`：valid；
  - change-gate block（工作树临时提交）：84 个文件、42 个 Manifest，**0 error / 0 warning**。
- 未执行：Playwright/Tauri（无 app 改动）；桌面 smoke；推送。
- 结果：S1 完成；v2 有正式数据、迁移拒绝与真实试点验收；v1 样例与试点保持原样。
- 遗留 / 后续：UW-032（接口形状/service 组合）仍 deferred；下一阶段 M8（036）需 Extend 授权。
