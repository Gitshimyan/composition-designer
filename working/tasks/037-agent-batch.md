# 037 — Agent read-only batch

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-020
- 目标：为 Agent 提供限流的只读批量查询（protocol/CLI/MCP），逐项返回结果；不实现 progress/cancel。
- authorization：
  - required：modify、extend、rearchitect
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 授权 M8）
    - rearchitect：granted（用户 2026-11-28 授权 M8；更新冻结协议文档）
- 来源：`working/plan.md` M8 037
- 变更声明：`changes/037-agent-batch.yaml`

## 背景

Agent 常需一次读取多个对象（inspect + context + impact）。逐次 round trip 成本高；037 增加
只读 batch，限制 16 条、拒绝写操作，并保持每条请求独立的 Response。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | protocol `batch` 1..16、只读白名单 | implemented | `composition-protocol` |
| REQ-02 | 逐项 Response 与独立失败 | implemented | batch 测试 |
| REQ-03 | CLI `batch` 文件（数组/对象） | implemented | `composition-cli` |
| REQ-04 | MCP `composition_batch` | implemented | tools 8 → 9 |
| REQ-05 | 协议/MCP/Skill 文档 | implemented | `docs/spec`、`agent/` |
| REQ-06 | 不实现 progress/cancel | implemented | M8 非目标；UW-020 deferred |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-020 | MCP progress/cancel/subscription/auth | 协议能力 | 037 评估后仍不需要 | Extend | 长任务无进度反馈 | 出现真实长任务需求时评估 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 037 批处理/流式/长任务 | 只做只读 batch；不做 progress/cancel | 无长任务证据；避免协议过度扩展 | plan M8 |

## 清单

- [x] protocol batch
- [x] CLI batch
- [x] MCP tool
- [x] 文档与 Skill
- [x] 测试 protocol/CLI/MCP
- [x] 计划与决策回写

## 验收

- batch 1..16；写操作/未知 op 返回 `protocol-operation-not-allowed` 且不执行；
- 空/超限返回 `protocol-invalid-params`；
- 每条结果按顺序、含独立 `ok/error/diagnostics`；单条失败不终止；
- CLI 接受数组或 `{requests}` 对象；MCP 工具与协议一致；
- 全量测试通过；变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - protocol `batch`：1..16 条、只读白名单（inspect/resolve/validate/source/context/impact）；
    写操作/未知 op → `protocol-operation-not-allowed`（整批不执行）；空/超限 →
    `protocol-invalid-params`；结果 `{count, results:[Response]}`，逐项独立失败；
  - CLI `batch <project> <requests.json>`：接受数组或 `{requests}` 对象；
  - MCP `composition_batch`（tools 8 → 9）；
  - 文档：protocol-v1、mcp-adapter-v1、agent SKILL/README。
- 验证：
  - `cargo test --workspace`：**218 passed**（protocol +4、CLI +3、MCP +1）；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - change-gate block（工作树临时提交）：16 个文件、44 个 Manifest，**0 error / 0 warning**。
- 未执行：Playwright/Tauri（无 app 改动）；MCP progress/cancel（M8 非目标）。
- 结果：Agent 可一次读取多个对象并逐项处理失败；写操作仍只能走 prepare/apply 事务。
- 遗留 / 后续：038 Skill 评测与跨平台配置；UW-020 继续 deferred。
