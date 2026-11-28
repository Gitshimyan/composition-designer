# MCP Adapter v1（M4 017；M8 036 Extend）

- 状态：M4 017 冻结（2026-11-26）；M8 036 新增 `composition_context` /
  `composition_impact` 工具与 context 资源（协议语义未变）
- 传输：MCP stdio（每行一个 JSON-RPC 2.0 消息）
- 实现：`composition-mcp`，复用 `composition-protocol`
- 启动：`composition-mcp --project <project-dir>`（缺省当前目录）

## 1. 生命周期

客户端发送：

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"agent","version":"1"}}}
```

服务端返回：

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": { "listChanged": false },
      "resources": { "subscribe": false, "listChanged": false }
    },
    "serverInfo": { "name": "composition-designer", "version": "0.1.0" }
  }
}
```

随后客户端发 `notifications/initialized`；通知无响应。服务端支持 `ping`。

## 2. Tools

`tools/list` 返回 8 个工具：

| Tool | 参数 | 语义 |
|---|---|---|
| `composition_inspect` | `{}` | 项目、模块、计数、digest、canEdit |
| `composition_resolve` | `{}` | 规范化 CompositionModel |
| `composition_validate` | `{}` | valid + L1–L7 诊断 |
| `composition_source` | `{}` | 只读 composition TOML |
| `composition_context` | `{}` | 紧凑 Agent 上下文（project/counts/modules/instances/diagnostics/revision） |
| `composition_impact` | `{module}` 或 `{instance}` 或 `{transaction}` | 模块/实例/事务影响分析（只读） |
| `composition_prepare_transaction` | `{transaction}` | 校验并返回 Diff，不写 |
| `composition_apply_transaction` | `{transaction}` | 原子应用并返回 Diff |

`tools/call`：

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"composition_validate","arguments":{}}}
```

结果：

```json
{
  "content": [{ "type": "text", "text": "<protocol response JSON>" }],
  "structuredContent": { "valid": true, "errorCount": 0, "...": "..." },
  "isError": false
}
```

- `isError` 为 `true` 表示工具业务失败（项目不可编辑、事务拒绝等）。
- 工具失败仍是 MCP tool result，不是 JSON-RPC error。
- `structuredContent` 只在 protocol result 是对象时提供；文本始终是完整 protocol response。

## 3. Resources

`resources/list`：

| URI | 内容 |
|---|---|
| `composition://project/inspect` | inspect protocol response |
| `composition://project/model` | resolve protocol response |
| `composition://project/diagnostics` | validate protocol response |
| `composition://project/source` | source protocol response |
| `composition://project/context` | context protocol response |

`resources/read` 返回 `contents:[{uri,mimeType:"application/json",text:"<protocol response>"}]`。
未知 URI 为 JSON-RPC `-32602`。

## 4. JSON-RPC 错误

| code | 场景 |
|---:|---|
| -32700 | 行不是 JSON |
| -32600 | `jsonrpc` 不是 `2.0` |
| -32601 | 未知 method |
| -32602 | tools/resources 参数错误、未知 tool/resource |
| -32603 | 内部序列化错误 |

## 5. 一致性

- 所有 tools/resources 都经 `composition-protocol` → `ProjectSession`/Core。
- 事务仍带 `base_revision`；外部 digest 变化返回 `edit-revision-conflict`。
- Adapter 不直接读写 TOML，不维护第二套模型。
- 不暴露 `.designer/**`、画布坐标或 GUI 状态。
- stdio 的 stdout 只输出 JSON-RPC；日志只能走 stderr。

## 6. 范围与限制

- 单进程单项目；不做多项目切换。
- 不实现 subscriptions、progress、cancellation、分页与认证（M8 037 评估后仍未启用）。
- `initialize` 回显客户端 protocolVersion；服务端只实现当前 MCP 基础消息集。
- 017 不包含 MCP 客户端配置/Agent Skill；018 负责端到端。

## 7. 决策

017 直接实现 JSON-RPC 2.0/MCP 基础消息集，而不是引入 MCP SDK 运行时：
依赖更少、stdio 行为可单测、与 016 协议边界清晰。若后续需要完整 MCP 特性
（subscriptions、progress、auth），再替换为官方 SDK，tool/resource 名称与协议保持不变。
