# 038 — Agent Skill 评测与跨平台配置

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：为 Agent Skill 提供确定性评测与跨平台 MCP 配置生成；完成 M8 收口。
- authorization：
  - required：modify、extend
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 授权 M8）
- 来源：`working/plan.md` M8 038
- 变更声明：`changes/038-agent-skill-eval.yaml`

## 背景

036/037 已提供 context/impact/batch；038 需要把 Skill 的固定工作流变成可重复评测，
并提供不依赖 PowerShell 的配置入口，避免 Agent 接入只在 Windows 可用。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | Skill 工作流评测（minimal） | implemented | `skill_eval.rs` S1–S9 |
| REQ-02 | v2 试点 context/impact 评测 | implemented | S10 |
| REQ-03 | 跨平台配置生成 | implemented | `agent/configure.mjs` |
| REQ-04 | 评测文档与结果 | implemented | `agent/EVALUATION.md` |
| REQ-05 | 不修改产品代码/Schema | implemented | 只加 Agent 测试与脚本 |
| REQ-06 | M8 收口 | implemented | plan/README/架构入口/D-028 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| 无 | — | — | — | — | — | — |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 038 configure.sh | 改为 Node `configure.mjs` | Node 在仓库工具链中跨平台可用且可执行验证；PowerShell 保留 | plan M8 |

## 清单

- [x] skill_eval（minimal + v2 pilot）
- [x] configure.mjs 与文档
- [x] EVALUATION 记录
- [x] 计划与决策回写

## 验收

- `cargo test -p composition-mcp --test skill_eval` 通过；
- 评测覆盖 context/impact/batch/prepare/apply/validate/source；
- configure.mjs 在 Windows 生成绝对路径配置（Linux/macOS 走同一逻辑）；
- 不修改 `examples/`/`working/pilots/` 原目录；临时副本清理；
- 全量测试通过；变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - `crates/composition-mcp/tests/skill_eval.rs`：
    - minimal：initialize、9 tools、context、impact、batch、resolve、prepare/apply、validate/source；
    - Glide v2 试点：7 实例 / 3 service、接口实现者、service instance 的 requires；
  - `agent/configure.mjs`：跨平台生成 `mcpServers` 绝对路径配置（Windows 已实测；
    Linux/macOS 走同一 `process.platform` 分支）；
  - `agent/EVALUATION.md` 记录场景、命令与结果；agent README/SKILL 更新。
- 验证：
  - `cargo test -p composition-mcp --test skill_eval`：**2 passed**；
  - `cargo test --workspace`：**220 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - configure.mjs 冒烟：生成配置 command=composition-mcp.exe、args=--project + 绝对路径；
  - change-gate block（工作树临时提交）：11 个文件、45 个 Manifest，**0 error / 0 warning**。
- 未执行：Linux/macOS 实机运行（Rust 测试与 Node 脚本按平台分支；Windows 已实测）；
  Playwright/Tauri（无 app 改动）。
- 结果：Skill 的固定工作流可重复评测；跨平台配置不再依赖 PowerShell；M8 收口。
- 遗留 / 后续：S2/H1 由用户决定；UW-020 仍 deferred。
