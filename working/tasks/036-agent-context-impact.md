# 036 — Agent context/impact 查询

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：为 Agent 提供只读 `context` 与 `impact` 查询（模块/实例/事务 dry-run），贯通 protocol、CLI、MCP，并更新协议文档与 Skill。
- authorization：
  - required：modify、extend、rearchitect
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 授权 M8）
    - rearchitect：granted（用户 2026-11-28 授权 M8；036 更新冻结协议文档）
- 来源：`working/plan.md` M8 036；S1 完成
- 变更声明：`changes/036-agent-context-impact.yaml`

## 背景

M8 要让 Agent 在编辑前理解上下文与影响，而不是先拉全量模型再猜。036 新增两个只读查询：
`context`（项目/模块/实例索引/诊断/Revision）与 `impact`（模块、实例、transaction prepare），
复用同一 CompositionModel；不新增写命令、不改事务语义。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | protocol `context` 紧凑上下文 | implemented | `composition-protocol` |
| REQ-02 | protocol `impact` 三目标与错误码 | implemented | module/instance/transaction |
| REQ-03 | CLI `context` / `impact` | implemented | `composition-cli` |
| REQ-04 | MCP 2 tools + context resource | implemented | `composition-mcp` |
| REQ-05 | 协议与 MCP 文档、Skill 更新 | implemented | `docs/spec`、`agent/` |
| REQ-06 | 只读与一致性 | implemented | transaction impact prepare 不写 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| 无 | — | — | — | — | — | — |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 036 context/impact | 只做只读查询；不做 progress/cancel | M8 非目标；UW-020 仍 deferred | plan M8 |

## 清单

- [x] protocol context/impact
- [x] CLI context/impact
- [x] MCP tools/resource
- [x] protocol/MCP 文档与 Skill
- [x] 测试：protocol/CLI/MCP
- [x] 计划与决策回写

## 验收

- context 输出 project/counts/modules/instances/diagnostics/revision，不复制 Config；不可读项目仍有诊断；
- impact 恰好一个目标；module/instance/transaction 字段稳定；transaction 只 prepare 不写；
- impact 错误码：`protocol-unknown-target`、`project-not-readable`；
- CLI 与 MCP 复用 protocol，无第二套模型；
- 全量测试通过；变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - protocol：新增 `context`（project/counts/modules/instances/diagnostics/revision，不复制 Config）
    与 `impact`（module / instance / transaction prepare）；错误码 `protocol-unknown-target`、
    `project-not-readable`；
  - CLI：`context <project>`、`impact <project> --module|--instance|--transaction`；
  - MCP：`composition_context`、`composition_impact` 与
    `composition://project/context` 资源（tools 6 → 8）；
  - 文档：protocol-v1、mcp-adapter-v1、agent SKILL/README；
  - 测试：protocol 6 条、CLI 4 条、MCP 1 条。
- 验证：
  - `cargo test --workspace`：**210 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - change-gate block（工作树临时提交）：17 个文件、43 个 Manifest，**0 error / 0 warning**。
- 未执行：Playwright/Tauri（无 app 改动）；MCP progress/cancel（M8 非目标，UW-020 仍 deferred）。
- 结果：Agent 可在编辑前后查询紧凑上下文与引用影响；transaction impact 只 prepare，不写文件。
- 遗留 / 后续：037 batch；038 Skill 评测与跨平台配置。
