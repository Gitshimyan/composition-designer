# 040 — Profile 的 Domain/Loader/Core/Session 实现

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-004、UW-018
- 目标：按 039 冻结的 format 3 实现 Project profiles、按 profile 加载/校验、Revision/Digest 作用域与 Session/Source 选择；不实现 CLI/MCP UI。
- authorization：
  - required：modify、rearchitect
  - approval：
    - modify：default
    - rearchitect：granted（用户 2026-11-28 授权 S2 039–041）
- 来源：`working/plan.md` S2 040；ADR-0007
- 变更声明：`changes/040-profile-core.yaml`

## 背景

039 已冻结 v2.1（format 3）；040 负责把 Project/Loader/Core/Session 从单入口扩展为
按 profile 选择，并保持 v1/v2.0 输出与行为不变。CLI/MCP/Designer 选择属 041。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | `SchemaVersion` v3、模块 1/2/3 矩阵 | implemented | domain/core |
| REQ-02 | Project profiles/default/active 输出 | implemented | domain + v3 tests |
| REQ-03 | Loader `[profiles]`/默认/显式选择 | implemented | `load_project_with_profile` |
| REQ-04 | 3 个新诊断与选择失败级联 | implemented | error codes + v3 tests |
| REQ-05 | Revision/Digest 按所选 profile | implemented | v3 digest tests |
| REQ-06 | Session/Source 只写所选入口 | implemented | editor v3 tests |
| REQ-07 | v1/v2.0 输出/行为不变 | implemented | 既有回归 + v2 输出测试 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-004 | Profile 的 CLI/MCP/Designer 入口与验收 | Rearchitect | 041 实现 | Extend + Modify | 用户尚不能选择 profile | 041 |
| UW-018 | 多入口的入口层与验收 | Rearchitect | 041 实现 | Extend + Modify | 仍需手改 TOML/直接调用 API | 041 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| — | 040 Core/Session | 不做 CLI/MCP/UI | plan 分工 | S2 |

## 清单

- [x] Domain：v3、profiles、3 个错误码
- [x] Loader：profile 解析/选择/错误
- [x] Core：版本矩阵、按 profile 校验、输出
- [x] Session：open_with_profile、Source/digest 作用域
- [x] 测试：core v3（11）、editor v3（2）
- [x] 计划/文档回写

## 验收

- v3 默认/显式 profile 选择正确；其他 profile 文件不影响当前 Digest；
- 未知/保留/不支持/非法名/路径/入口缺失均产生稳定诊断；
- v3 允许模块 1/2/3；v1/v2.0 行为与输出不变；
- 事务/Source 只写所选入口，另一 profile 字节不变；
- 全量测试通过；变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - Domain：`SchemaVersion::V3`、`supports_interfaces`；`Project.profiles/default_composition/
    active_profile`；`ProfileDefinition`；3 个新错误码（ALL 80 / 验证 74）；
  - Loader：`load_project_with_profile` + Source 候选变体；`[profiles]` 解析、
    reserved/default/未知名/路径/版本；所选入口加载；
  - Core：v3 版本矩阵、模块 1/2/3、`validate_project_with_profile`、v3 输出；
  - Editor：`ProjectSession::open_with_profile`；Source prepare/digest/外部冲突按 profile；
  - 测试：core v3 11 条、editor v3 2 条。
- 验证：
  - `cargo test --workspace`：**233 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - change-gate block（工作树临时提交）：21 个文件、47 个 Manifest，**0 error / 0 warning**。
- 未执行：CLI/MCP/Designer profile 入口（041）；Tauri 宿主（无 app 改动，但将在最终回归确认）；
  v3 fixtures（041）。
- 结果：Core/Session 可按 profile 选择并保持 v1/v2.0 兼容；其他 profile 文件不影响当前
  Digest；事务/Source 只写所选入口。
- 遗留 / 后续：041 入口与验收；UW-004/018 在 041 验收后关闭。
