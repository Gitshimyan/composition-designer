# Agent Interface

M4 的 Agent 入口：

- [SKILL.md](./SKILL.md)：Agent Skill 指令（MCP tools/resources、事务流程、错误处理）。
- [mcp.example.json](./mcp.example.json)：MCP client 配置模板。
- [configure.ps1](./configure.ps1)：按当前仓库路径生成 `mcp.local.json`。

## 1. 构建 MCP server

```powershell
cargo build -p composition-mcp
```

产物：`target\debug\composition-mcp.exe`。

## 2. 生成 MCP 配置

```powershell
powershell -NoProfile -File agent\configure.ps1 -Project "E:\path\to\project"
```

生成 `agent\mcp.local.json`（已忽略，不提交）。把它加入支持 MCP 的 Agent/客户端配置。
`mcp.example.json` 使用 `${COMPOSITION_DESIGNER_ROOT}` / `${COMPOSITION_PROJECT}` 占位符；
如果客户端不做变量展开，请使用 `configure.ps1` 生成绝对路径。

## 3. 可用能力

Tools：

- `composition_inspect`
- `composition_resolve`
- `composition_validate`
- `composition_source`
- `composition_context`
- `composition_impact`
- `composition_prepare_transaction`
- `composition_apply_transaction`

Resources：

- `composition://project/inspect`
- `composition://project/model`
- `composition://project/diagnostics`
- `composition://project/source`
- `composition://project/context`

协议细节见 [protocol-v1.md](../docs/spec/protocol-v1.md)、
[mcp-adapter-v1.md](../docs/spec/mcp-adapter-v1.md) 与
[command-schema-v1.md](../docs/spec/command-schema-v1.md)。

## 4. 端到端验证

```powershell
cargo test -p composition-mcp --test e2e -- --nocapture
```

测试启动真实 MCP stdio 进程，对临时项目执行：

1. initialize / inspect / resolve；
2. prepare `set_config` transaction，检查 Diff；
3. apply transaction；
4. validate；
5. read source 并确认 TOML 中写入新标题。

不会修改 `examples/`。

## 5. 边界

- Agent 不得直接编辑 TOML；所有写操作走 `composition_apply_transaction`。
- `composition_source` 只读。
- `composition_context` / `composition_impact` 只读；impact 的 transaction 目标只做 prepare 预览，
  不写文件、不递增 Revision。
- 单 MCP server 进程绑定一个项目；切换项目需重新启动或重新配置。
- External changes / stale Revision 返回 `edit-revision-conflict`，应重新 resolve 后再提交。
- Agent 必须报告 prepare/apply 的 Diff 与最终 validation，而不是只说“已修改”。
