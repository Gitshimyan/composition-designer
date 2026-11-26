# Command Schema v1（M2 009）

- 状态：M2 009 冻结（2026-11-26）
- 范围：`composition-editor` 的编辑命令与原子事务
- 关联：[`composition-schema-v1.md`](./composition-schema-v1.md)、[`domain-model-v1.md`](./domain-model-v1.md)

本文件定义 v0.1 编辑命令的 JSON 形状、前置条件与事务规则。009 只操作内存模型；
TOML 落盘、`source_digest` 刷新、结构化 Diff 与 request ID 幂等分别在 010 / 011 完成。

## 1. 事务

```json
{
  "request_id": "request-1",
  "base_revision": {
    "seq": 0,
    "source_digest": "sha256:…"
  },
  "commands": [
    { "type": "set_config", "instance": "app", "property": "title", "value": "New title" }
  ]
}
```

- `request_id`：非空字符串；010 起用于幂等。
- `base_revision`：读取工作区时的 Revision；必须与当前一致。
- `commands`：非空、按顺序应用。
- 命令对象用 `type` 区分，字段为 snake_case。

## 2. 命令

| type | 字段 | 语义 |
|---|---|---|
| `create_instance` | `id`, `module`, `version`, `config` | 新增实例；ID 不得重复 |
| `delete_instance` | `id` | 删除实例；Root 拒绝；级联移除相关 Binding、Connection、依赖 provider |
| `rename_instance` | `from`, `to` | 重命名并更新 root、Binding、Connection、依赖 provider |
| `update_instance_module` | `id`, `module`, `version` | 更换模块/版本；保留实例配置，候选模型重新校验 |
| `set_config` | `instance`, `property`, `value` | 设置实例配置项（可新增未知属性，由候选校验拒绝） |
| `unset_config` | `instance`, `property` | 删除已有配置项；不存在则命令错误 |
| `create_binding` | `id`, `parent`, `slot`, `child` | 新增结构 Binding；ID 不得重复 |
| `update_binding` | `id`, `parent`, `slot`, `child` | 替换 Binding 的 parent/slot/child |
| `delete_binding` | `id` | 删除 Binding |
| `create_connection` | `id`, `from`, `to` | 新增 Port Connection；`from`/`to` 为 `{instance, port}` |
| `update_connection` | `id`, `from`, `to` | 替换连接端点 |
| `delete_connection` | `id` | 删除 Connection |

`config.value` 是原生 JSON 值（string / integer / float / boolean / array），与
`composition-schema-v1.md` 的 v0.1 配置类型一致；不支持 object。

模块定义（`modules/**/module.toml`）在 v0.1 是只读的，不在命令范围内。

## 3. 事务规则

1. `request_id` 非空、`commands` 非空，否则 `edit-invalid-command`。
2. `base_revision` 必须与工作区当前 Revision 完全一致，否则 `edit-revision-conflict`。
3. 基于当前模型生成候选模型；逐条应用命令。
   - 命令级错误（ID 重复、目标不存在、删除 Root）→ `edit-invalid-command`，整体拒绝。
4. 对候选模型执行完整 L2–L7 校验（`composition_core::validate_model`）。
   - 任意 error → 整体拒绝，诊断保留真实代码（如 `structure-slot-cardinality`）。
5. 校验通过后一次性替换工作区模型，`seq + 1`；009 不写 TOML，因此
   `source_digest` 保持读取时的值，011 落盘后重算。
6. 失败事务不改变工作区模型、Revision 或来源映射；`prepare` 是纯规划，`apply` 才提交。

删除/重命名的级联只做结构清理；是否仍然有效由候选模型的全量校验决定。
例如删除被依赖的实例可能产生 `dependency-unresolved`，删除必需 Slot 的 Binding 可能产生
`structure-slot-cardinality`。

## 4. 诊断与错误代码

| 情况 | 代码 |
|---|---|
| 命令字段错误、ID 重复、目标不存在、删除 Root | `edit-invalid-command` |
| `base_revision` 不匹配 | `edit-revision-conflict` |
| 候选模型违反 Schema/引用/契约/结构/连接/依赖 | 对应 L2–L7 代码 |
| 请求重复（010） | `edit-duplicate-request`（预留） |
| 落盘事务失败（011） | `edit-transaction-failed`（预留） |

## 5. 未包含（后续卡）

- 010：Revision 持久化、并发冲突的完整语义、结构化 Diff、request ID 幂等与撤销。
- 011：`toml_edit` 局部写回、保留注释/排版、`source_digest` 刷新。
- 新对象（Create）暂时没有来源 `SourceRef`；候选诊断只带对象 ID，011 落盘后重新解析会补上。
- 模块定义编辑、Port 协议协商、事务历史持久化不在 v0.1。
