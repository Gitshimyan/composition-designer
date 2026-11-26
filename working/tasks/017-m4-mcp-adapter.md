# 017 — M4 MCP Adapter

- 状态：done
- 目标：将 `composition-protocol` 暴露为 MCP stdio tools/resources；Agent 不直接读写 TOML。
- 授权：Extend（新 `composition-mcp` crate、MCP 消息实现）
- 变更声明：`changes/017-mcp-adapter.yaml`

## 背景

016 已有 JSONL 协议。017 增加标准 MCP stdio 适配器：

- `initialize` / `ping` / `tools/list` / `tools/call` / `resources/list` / `resources/read`；
- 6 个 tools：inspect/resolve/validate/source/prepare/apply；
- 4 个 resources：inspect/model/diagnostics/source；
- 单进程绑定一个项目目录，所有操作复用 ProtocolSession。

## 清单

- [x] 新 crate `composition-mcp` + binary
- [x] JSON-RPC 2.0 newline-delimited stdio
- [x] initialize capabilities / serverInfo / ping
- [x] tools/list 6 个工具与 inputSchema
- [x] tools/call 映射 protocol，content/structuredContent/isError
- [x] resources/list + resources/read 4 个 URI
- [x] JSON-RPC 错误码与通知无响应
- [x] `--project` 启动参数、stdout 只输出 JSON-RPC
- [x] 协议文档 `docs/spec/mcp-adapter-v1.md`
- [x] 单测：initialize/tools/resources/未知 method/通知
- [x] `cargo test` / clippy / fmt；更新 plan、README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**141 passed**
  （CLI 10 + core 8 + domain 58 + editor 5/18/7 + MCP 4 + protocol 4 + toml 12/15）；
- `cargo clippy` / `fmt` 通过；
- `composition-mcp --project examples/minimal-project` 可启动；
- initialize 返回 tools/resources capability；
- tools/list 6 项；validate tool 返回 `isError=false` 与 valid=true；
- resources/read `composition://project/model` 返回模型 JSON；
- 未知 method → JSON-RPC -32601；通知无输出；
- 工具失败使用 `isError=true`，不伪装成 JSON-RPC 错误。

## 结果 / 遗留问题

- 结果：
  - `composition-mcp` stdio adapter；
  - 6 tools / 4 resources；
  - 4 个单元测试；
  - 协议文档冻结。
- 关键决策：
  - 直接实现 MCP 基础 JSON-RPC 2.0，减少依赖、stdio 可单测；tool/resource 命名稳定，
    后续可替换官方 SDK；
  - 单进程单项目，`--project` 指定；
  - tools 输出完整 protocol response 文本 + 可选的 structuredContent；
  - 工具失败是 tool result `isError`，JSON-RPC 错误只用于传输/方法/参数。
- 遗留：
  - 无 MCP subscriptions/progress/cancellation/分页/认证；
  - initialize 回显客户端 protocolVersion，未做版本拒绝；
  - 不实现多项目切换；
  - 无 Agent 客户端配置/Skill（018）。
- 下一步：018 Agent Skill 与端到端。
