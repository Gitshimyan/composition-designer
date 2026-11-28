# Agent Skill Evaluation

本文件记录 `composition-designer` Skill 的可重复评测：用真实 MCP stdio 进程执行固定工作流，
断言协议结果。它不是 LLM 质量评分，也不访问网络。

## 1. 场景

| # | 场景 | 断言 |
|---|---|---|
| S1 | `initialize` | serverInfo = composition-designer |
| S2 | `tools/list` | 9 个工具，含 context/impact/batch/prepare/apply |
| S3 | `composition_context` | minimal 项目 2 实例、project id 正确 |
| S4 | `composition_impact` instance | `kind=instance`，返回 parent/child 引用 |
| S5 | `composition_batch` | 3 条只读请求按序返回、逐项 ok |
| S6 | `composition_resolve` | 读取 `revision.seq` / `source_digest` |
| S7 | `composition_prepare_transaction` | set_config 被接受，Diff 存在 |
| S8 | `composition_apply_transaction` | 写回成功 |
| S9 | `composition_validate` / `composition_source` | valid 且 Source 含新值 |
| S10 | Glide v2 试点 context/impact | 7 实例 / 3 service / 接口实现者与 requires 正确 |

## 2. 运行

```powershell
cargo test -p composition-mcp --test skill_eval
```

测试在临时目录复制 `examples/minimal-project` 与
`working/pilots/glide-task-terminal-v2`，不修改原目录。

## 3. 结果（2026-11-28）

```text
skill_eval_minimal_project_workflow ... ok
skill_eval_v2_pilot_context_and_impact ... ok
2 passed
```

## 4. 跨平台配置

```powershell
node agent/configure.mjs --project examples/minimal-project
node agent/configure.mjs --project examples/minimal-project --dest agent/mcp.local.json
```

`configure.mjs` 在 Windows/Linux/macOS 生成同一结构的 `mcpServers` 配置；
`configure.ps1` 仍保留给 PowerShell 用户。

## 5. 边界

- 评测只覆盖协议与工具链行为，不评价 Agent 的自然语言质量；
- 不覆盖 progress/cancel（UW-020）；
- 不覆盖真实 GUI/Canvas；这些由 Designer Playwright 与 Tauri host tests 负责。
