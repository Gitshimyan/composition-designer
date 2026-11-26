# Composition Protocol v1（M4 016）

- 状态：M4 016 冻结（2026-11-26）
- 传输：单行 JSON（JSONL）或 CLI 单次请求
- 实现：`composition-protocol` + `composition-cli`

本协议是 Agent/CLI 访问同一 Core 的机器可读入口；不引入第二套模型，不直接写 TOML。

## 1. 请求

```json
{
  "id": "req-1",
  "op": "inspect",
  "params": {}
}
```

- `id`：调用方请求标识；原样回显，可为空。
- `op`：操作名，见下表。
- `params`：操作参数；缺省为 `{}`。

### 查询

| op | params | result |
|---|---|---|
| `inspect` | `{}` | `{project, modules, valid, errorCount, warningCount, sourceDigest, canEdit}` |
| `resolve` | `{}` | `{model}`（可为 null） |
| `validate` | `{}` | `{valid, errorCount, warningCount, sourceDigest}` |
| `source` | `{}` | `{entry, text}`；仅可编辑会话 |

### 命令

| op | params | result |
|---|---|---|
| `prepare_transaction` | `{"transaction": <Transaction>}` | `{accepted, replayed, diff}`，不写文件 |
| `apply_transaction` | `{"transaction": <Transaction>}` | `{accepted, replayed, diff}`，成功后写回 |

`Transaction` 形状见 [`command-schema-v1.md`](./command-schema-v1.md)。

## 2. 响应

```json
{
  "id": "req-1",
  "ok": true,
  "result": { "valid": true },
  "diagnostics": []
}
```

- `ok`：
  - 查询成功为 `true`；项目无效仍返回 `ok=true`，由 `result.valid` 表达。
  - 命令被接受为 `true`；被拒绝为 `false`，`diagnostics` 给出原因。
  - 协议级错误为 `false`，`error` 给出 `code/message`。
- `diagnostics`：始终是数组；内容为 Core 诊断（L1–L7 或 `edit-*`）。
- `error`：仅协议级错误，例如：
  - `protocol-invalid-request`
  - `protocol-unknown-operation`
  - `protocol-invalid-params`
  - `project-not-editable`
- 命令诊断示例：`edit-revision-conflict`（外部修改/旧 Revision）、
  `structure-slot-cardinality`（候选模型非法）。

## 3. JSONL

`composition-cli jsonl <project-dir>`：

- stdin 每行一个 `Request`；空行忽略。
- stdout 每行一个 `Response`，发送后 flush。
- 会话在行间保持；`apply_transaction` 成功后后续请求看到新模型。
- 某行 JSON 非法：返回 `id=""`、`ok=false`、`error.code=protocol-invalid-request`，进程继续。
- EOF 后进程退出 0；启动/IO 错误退出 2。

## 4. CLI 单次命令

```powershell
composition-cli query examples/minimal-project validate
composition-cli apply examples/minimal-project transaction.json --dry-run
composition-cli apply examples/minimal-project transaction.json
composition-cli jsonl examples/minimal-project
```

- `query` 输出 pretty JSON；`ok=false` 退出 1。
- `apply` 读取 transaction JSON；`--dry-run` 调 `prepare_transaction`，否则 `apply_transaction`；
  `ok=false` 退出 1，IO/JSON 错误退出 2。
- 既有 `inspect` / `resolve` / `validate` 保持 M1 输出与退出码。

## 5. 一致性规则

- 所有查询/命令复用 `composition-core` / `composition-editor`。
- 写操作必须带 `base_revision`；旧 Revision 或磁盘 digest 变化返回
  `edit-revision-conflict`，不覆盖。
- `source` 只读；直接 Source 编辑尚未进入协议。
- JSONL 会话与 CLI 单次命令不持久化 Revision 历史；跨进程幂等未实现。
- 协议不包含 GUI 布局、Canvas 坐标或 `.designer/**`。
