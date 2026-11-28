# 034 — Commands/UI/CLI/MCP：scope 创建与 interface 展示

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-002、UW-003
- 目标：把 033 的 v2 模型接入编辑命令与入口：`create_instance.scope`、Services 视图、接口目标展示，并验证 CLI/MCP 事务。
- authorization：
  - required：modify、extend
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28）
- 来源：`working/plan.md` S1 034；032 冻结命令字段；033 Core 支持
- 变更声明：`changes/034-scope-ui.yaml`

## 背景

033 已能解析与校验 v2 项目，但命令层仍无法写 `scope`，UI 也没有 Services 入口或接口展示，
CLI/MCP 的 v2 事务未验证。034 补齐这些入口，不修改 TOML Schema 或设计基线。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | `create_instance.scope` 命令字段与 v1 拒绝 | implemented | `composition-editor` + v2 测试 |
| REQ-02 | TOML 写回 `scope` 并可重开 | implemented | session/editor/Tauri 测试 |
| REQ-03 | Services 视图（列表、选中、创建） | implemented | `ServicesView`、`CreateServiceDialog` |
| REQ-04 | Inspector 显示 scope 与 interface | implemented | `SelectionDetails` |
| REQ-05 | Dependency 视图/对话框显示接口目标 | implemented | `relationModel`、`DependencyView` |
| REQ-06 | CLI inspect/validate/apply v2 | implemented | `composition-cli/tests/v2.rs` |
| REQ-07 | MCP prepare/apply v2 | implemented | `composition-mcp/tests/e2e.rs` |
| REQ-08 | Playwright Services/接口链路 | implemented | `tests/ui/services.spec.ts` |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-002 | 接口身份的正式验收 | Rearchitect | 035 建 v2 数据集/试点 | Modify | 尚无正式 v2 样例 | 035 |
| UW-003 | service scope 的正式验收 | Rearchitect | 同上 | Modify | 尚无 Glide v2 验收 | 035 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| — | — | — | — | — |

## 清单

- [x] Command/写回：`create_instance.scope`、v1 拒绝
- [x] Tauri 宿主 scope 写回测试
- [x] Services 视图与创建对话框
- [x] Inspector scope / interface 展示
- [x] Dependency 接口目标与候选计算
- [x] CLI/MCP v2 事务测试
- [x] Playwright 新链路
- [x] 文档与计划回写

## 验收

- v2 项目可创建 service 并写回 `scope = "service"`，重开 valid；
- v1 项目显式 scope 被 `edit-invalid-command` 拒绝；
- Services 视图可列出/选中/创建 service；接口在 Inspector 与 Dependency 视图可见；
- CLI/MCP 的 prepare/apply 接受 scope，写回后 valid；
- UI/Tauri/workspace 测试通过；
- 未改 TOML Schema/设计基线/示例；
- 变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - 命令：`create_instance` 增加可选 `scope`；v1 项目显式 scope（含 `structure`）→
    `edit-invalid-command`；v2 未传时写 `structure`；
  - 写回：保留格式 TOML 增加 `scope`，重开 valid；
  - UI：Services 视图（列表/选中/创建）、`CreateServiceDialog`（仅 primitive 模块）、
    Inspector 显示 scope 与 interfaces、Dependency 目标显示接口并按其计算候选；
  - 入口：CLI inspect/validate/apply v2；MCP prepare/apply v2；Tauri 宿主写回 scope；
  - 夹具：新增 v2 snapshot/UI mock 模式与 Services 链路。
- 验证：
  - `cargo test --workspace`：**196 passed**（editor v2 3、CLI v2 2、MCP v2 1）；
  - Tauri 宿主：**17 passed**（含 service scope 写回）；
  - Playwright：**34 passed**（新增 Services/接口 4 条）；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - `npm run build`：通过；
  - change-gate block（工作树临时提交）：28 个文件、41 个 Manifest，**0 error / 0 warning**。
- 未执行：035 的正式 v2 数据集、迁移拒绝路径与 Glide 试点验收；桌面 smoke。
- 结果：v2 的 scope/接口从 Core 贯通到命令、写回、UI、CLI、MCP；v1 项目保持只读兼容。
  `coverage_status = complete_with_deferred_items`：入口完成，正式验收在 035。
- 遗留 / 后续：UW-002/003 待 035 验收后关闭；035 补 examples v2 与试点。
