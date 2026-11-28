# Composition Protocol v1（M4 016；M8 036 Extend）

- 状态：M4 016 冻结（2026-11-26）；M8 036 经用户授权新增只读
  `context` / `impact` 查询（协议语义未变）
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
| `context` | `{}` | 紧凑 Agent 上下文（见 §1.1） |
| `impact` | 三个目标之一（见 §1.2） | 模块/实例/事务影响分析 |
| `batch` | `{"requests":[Request,...]}` | 只读批量查询（见 §1.3） |

### 1.1 `context`

`context` 不复制 Config 值；需要完整值时调用 `resolve`。

```json
{
  "project": { "id": "...", "name": "...", "schemaVersion": 2, "root": "app" },
  "counts": {
    "modules": 2, "instances": 3, "structureInstances": 2, "services": 1,
    "bindings": 1, "connections": 0, "dependencies": 1, "errors": 0, "warnings": 0
  },
  "modules": [
    {
      "id": "example.app", "version": "1.0.0", "kind": "composite",
      "interfaces": [],
      "dependencies": [{ "id": "session", "target": { "kind": "interface", "id": "example.session.api", "version": "1.0.0" }, "optional": false }]
    }
  ],
  "instances": [
    {
      "id": "session", "module": { "id": "example.session", "version": "1.0.0" },
      "scope": "service", "interfaces": [{ "id": "example.session.api", "version": "1.0.0" }],
      "configKeys": [], "providerKeys": []
    }
  ],
  "diagnostics": [],
  "revision": { "seq": 0, "source_digest": "sha256:..." }
}
```

- 项目不可读时 `project=null`、计数为 0，`diagnostics` 仍返回。
- `scope` 仅 v2 实例出现；v1 省略（等价 structure）。

### 1.2 `impact`

`params` 必须恰好包含 `module`、`instance`、`transaction` 之一。

| 目标 | params | result 关键字段 |
|---|---|---|
| 模块 | `{"module":{"id":"...","version":"..."}}` | `kind=module`、`module.instances`、`interfaceImplementers`、`dependents` |
| 实例 | `{"instance":"app"}` | `kind=instance`、`parentBindings`、`childBindings`、`connections`、`requires`、`providesFor` |
| 事务 | `{"transaction": <Transaction>}` | `kind=transaction`、`accepted`、`replayed`、`diff`、`changedObjects`；只执行 prepare，不写文件 |

- 模块的 `interfaceImplementers` 按该模块声明的每个接口列出实现实例；
  `dependents` 列出以该模块（v1）或其接口（v2）为目标的依赖要求。
- 实例的 `connections` 标注 `role=from|to` 与对端端点；`providesFor` 同时包含显式映射与解析结果。
- 事务影响的 `changedObjects` 来自 `ModelDiff.changes[].object_id`；`diagnostics` 为 prepare 诊断。
- 目标不存在：`project-not-readable` 或 `protocol-unknown-target`。
- 影响分析只读；`transaction` 目标不会写入文件或递增 Revision。

### 1.3 `batch`

```json
{
  "requests": [
    { "id": "a", "op": "inspect" },
    { "id": "b", "op": "context" },
    { "id": "c", "op": "impact", "params": { "instance": "app" } }
  ]
}
```

- 1..16 条请求；只允许只读 op：`inspect`、`resolve`、`validate`、`source`、
  `context`、`impact`。写操作、`batch` 自身与未知 op 返回
  `protocol-operation-not-allowed`，整个 batch 不执行。
- 空数组或超过 16 条返回 `protocol-invalid-params`。
- `result` 为 `{count, results}`；每个 `results[i]` 是完整 `Response`
  （`id`、`ok`、`result`、`diagnostics`、`error`），按请求顺序排列。
- 单条失败不终止其它请求；顶层 `ok=true` 表示 batch 本身被接受。

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
  - `project-not-readable`（impact 需要可读模型）
  - `protocol-unknown-target`（impact 的模块/实例不存在）
  - `protocol-operation-not-allowed`（batch 中含写操作/未知 op）
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
composition-cli context examples/minimal-project
composition-cli impact examples/minimal-project --instance app
composition-cli impact examples/minimal-project --module leaf.widget@1.0.0
composition-cli impact examples/minimal-project --transaction transaction.json
composition-cli batch examples/minimal-project requests.json
composition-cli apply examples/minimal-project transaction.json --dry-run
composition-cli apply examples/minimal-project transaction.json
composition-cli jsonl examples/minimal-project
```

- `query` / `context` / `impact` / `batch` 输出 pretty JSON；`ok=false` 退出 1。
- `batch` 文件可以是请求数组，或 `{ "requests": [...] }` 对象；格式错误退出 2。
- `impact` 只接受一个目标；同时给多个或都不给退出 2。
- `apply` 读取 transaction JSON；`--dry-run` 调 `prepare_transaction`，否则 `apply_transaction`；
  `ok=false` 退出 1，IO/JSON 错误退出 2。
- 既有 `inspect` / `resolve` / `validate` 保持 M1 输出与退出码。

## 5. 一致性规则

- 所有查询/命令复用 `composition-core` / `composition-editor`。
- `context` / `impact` 只读派生自规范化模型，不维护缓存或第二套状态。
- 写操作必须带 `base_revision`；旧 Revision 或磁盘 digest 变化返回
  `edit-revision-conflict`，不覆盖。
- `source` 只读；直接 Source 编辑尚未进入协议。
- JSONL 会话与 CLI 单次命令不持久化 Revision 历史；跨进程幂等未实现。
- 协议不包含 GUI 布局、Canvas 坐标或 `.designer/**`。
