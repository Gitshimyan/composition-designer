# 018 — M4 Agent Skill 与端到端

- 状态：done
- 目标：提供 Agent Skill/MCP 配置，并用真实 MCP stdio 进程完成一次
  inspect → resolve → prepare → apply → validate 的可复现修改。
- 授权：Extend（Agent Skill、MCP 配置模板、端到端测试）
- 变更声明：`changes/018-agent-skill-e2e.yaml`

- coverage_status：complete_with_deferred_items
- open_items：UW-011、UW-020、UW-021
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

016 有协议，017 有 MCP Adapter。018 补齐 Agent 使用方式与真实验证：

- `agent/SKILL.md`：工具选择、事务流程、Revision/错误处理；
- `agent/mcp.example.json` / `agent/configure.ps1`：MCP 客户端配置；
- `crates/composition-mcp/tests/e2e.rs`：启动真实 `composition-mcp` 进程，对临时项目执行
  prepare/apply/validate/source；
- 不修改 `examples/`。

## 清单

- [x] `agent/SKILL.md`（frontmatter + 工具/流程/规则）
- [x] `agent/README.md`
- [x] `agent/mcp.example.json`
- [x] `agent/configure.ps1`（生成 `mcp.local.json`）
- [x] `.gitignore` 忽略 `agent/mcp.local.json`
- [x] MCP e2e 测试：真实 stdio 进程 + 临时项目
- [x] 演示 set_config 事务、Diff、写回、validate、source
- [x] `cargo test` / clippy / fmt；更新 plan、README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**142 passed**
  （CLI 10 + core 8 + domain 58 + editor 5/18/7 + MCP 4/1 + protocol 4 + toml 12/15）；
- `cargo clippy` / `fmt` 通过；
- `cargo test -p composition-mcp --test e2e -- --nocapture` 通过；
- e2e 使用真实 `composition-mcp --project <temp>`，完成 initialize、inspect、resolve、
  prepare、apply、validate、resources/read；
- 临时项目 TOML 写入 `title = "Agent E2E"`，`valid=true`；
- `agent/SKILL.md` 明确禁止直接改 TOML，要求报告 Diff 与最终诊断。

## 结果 / 遗留问题

- 结果：
  - Agent Skill + MCP 配置模板/生成脚本；
  - 真实 MCP stdio e2e 测试；
  - 修复 protocol inspect/validate 结果为 camelCase（与协议文档一致）；
  - M4 收口。
- 关键决策：
  - e2e 不依赖 LLM，只验证 Agent 实际会调用的 MCP 边界；
  - 测试使用临时复制项目，不污染 examples；
  - MCP 配置按绝对路径生成，避免客户端变量展开差异；
  - Skill 把 prepare/review/apply/validate 作为固定流程。
- 遗留：
  - 无自动化“LLM 按 Skill 执行”的评测；e2e 覆盖工具链；
  - 配置生成仅 Windows PowerShell；
  - MCP 单进程单项目；
  - external changes 仍依赖 MCP 调用时 digest 守卫，没有主动通知；
  - Source 只读。
- 下一步：M5 019 完整性与验收（多文件/性能/崩溃恢复/迁移）。
