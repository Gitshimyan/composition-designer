# Command Schema v1（M2 009）

- 状态：M2 009–011 冻结（2026-11-26）；v0.2 025 经用户明确授权 **Extend** 增加
  Dependency Provider 命令；v0.3 029 经 ADR-0005 增加 Source 文本命令
- 范围：`composition-editor` 的编辑命令、原子事务、Revision 历史/幂等、结构化 Diff 与 TOML 写回
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
| `set_dependency_provider` | `instance`, `requirement`, `provider` | 设置显式依赖 Provider 映射；候选模型重新校验 |
| `unset_dependency_provider` | `instance`, `requirement` | 删除显式映射并回退自动解析；不存在则命令错误 |
| `create_binding` | `id`, `parent`, `slot`, `child` | 新增结构 Binding；ID 不得重复 |
| `update_binding` | `id`, `parent`, `slot`, `child` | 替换 Binding 的 parent/slot/child |
| `delete_binding` | `id` | 删除 Binding |
| `create_connection` | `id`, `from`, `to` | 新增 Port Connection；`from`/`to` 为 `{instance, port}` |
| `update_connection` | `id`, `from`, `to` | 替换连接端点 |
| `delete_connection` | `id` | 删除 Connection |
| `replace_source` | `source` | 替换当前 Composition 入口的完整 TOML 文本；必须独占事务 |

`config.value` 是原生 JSON 值（string / integer / float / boolean / array），与
`composition-schema-v1.md` 的 v0.1 配置类型一致；不支持 object。

Dependency Provider 命令（v0.2 025 Extend）：

- `set_dependency_provider` / `unset_dependency_provider` 只改实例的显式映射；
- 命令级错误：实例不存在、Unset 时没有显式映射 → `edit-invalid-command`；
- requirement 不在模块契约、provider 实例不存在/不满足目标模块与版本、provider 指向自身
  → 候选模型 L2–L7 诊断（`contract-dependency-unknown`、`dependency-provider-invalid`、
  `dependency-self` 等）；
- Unset 后按现有自动解析规则重新计算；是否歧义/无法解析由候选校验给出。

Source 命令（029 / ADR-0005）：

- `replace_source` 的 `source` 是完整 Composition TOML 文本；
- 必须独占事务，不能与对象命令混合；混合 → `edit-invalid-command`；
- `prepare` 用候选文本重新加载/验证，返回文本 `source_diff`、对象 `ModelDiff` 与完整诊断；
- `apply` 校验 base Revision/磁盘 digest，原样写入并备份/替换/重验，失败回滚；
- 只编辑 `workspace.composition` 指向的入口；模块定义仍只读；
- 不做自动格式化、隐式修复、三向合并。

### v2.0 注记（S1 032 冻结，033/034 实现）

- `create_instance` 增加可选 `scope`：`structure`（默认）或 `service`；
- v1 项目显式传 `scope`（即使值为 `structure`）→ `edit-invalid-command`；
- v2 项目的 `service` 实例必须是 `primitive` 模块，由候选校验报
  `contract-service-module-invalid`；
- 其余命令形态不变；接口依赖的 provider 有效性由候选模型判定；
- 完整语义见 [`composition-schema-v2.md`](./composition-schema-v2.md) §8。

模块定义（`modules/**/module.toml`）在 v0.1 是只读的，不在命令范围内。

## 3. 事务规则

1. `request_id` 非空、`commands` 非空，否则 `edit-invalid-command`。
2. 先检查 request 幂等：
   - 同一 `request_id` + 同一事务（request/base/commands 完全一致）→ `Replayed`，
     返回原模型/诊断/Diff，**不**递增 `seq`、不新增历史；
   - 同一 `request_id` + 不同载荷 → `edit-duplicate-request`。
3. `base_revision` 必须存在于工作区历史中，且与当前 Revision 完全一致，否则
   `edit-revision-conflict`。
4. 基于当前模型生成候选模型；逐条应用命令。
   - 命令级错误（ID 重复、目标不存在、删除 Root）→ `edit-invalid-command`，整体拒绝。
5. 对候选模型执行完整 L2–L7 校验（`composition_core::validate_model`）。
   - 任意 error → 整体拒绝，诊断保留真实代码（如 `structure-slot-cardinality`）。
6. 校验通过后计算结构化 Diff，一次性替换工作区模型：`seq + 1`，写入历史记录。
   - `EditorWorkspace::apply` 只更新内存；`ProjectSession::apply` 在 TOML 写回并重新解析后
     用新的 `source_digest` 刷新 Revision。
7. 失败事务不改变工作区模型、Revision、历史或 request 记录；`prepare` 是纯规划，
   `apply` 才提交。失败事务不记录 request，可用同一 `request_id` 重试。

删除/重命名的级联只做结构清理；是否仍然有效由候选模型的全量校验决定。
例如删除被依赖的实例可能产生 `dependency-unresolved`，删除必需 Slot 的 Binding 可能产生
`structure-slot-cardinality`。

## 4. 诊断与错误代码

| 情况 | 代码 |
|---|---|
| 命令字段错误、ID 重复、目标不存在、删除 Root | `edit-invalid-command` |
| `base_revision` 不匹配或不在历史中 | `edit-revision-conflict` |
| 同一 request_id 复用于不同载荷 | `edit-duplicate-request` |
| 候选模型违反 Schema/引用/契约/结构/连接/依赖 | 对应 L2–L7 代码 |
| 落盘/重新解析失败 | `edit-transaction-failed` |
| 未实现操作（后续） | `edit-unsupported-operation`（预留） |

## 5. 结构化 Diff

成功事务（含重放）返回 `ModelDiff`：

```json
{
  "summary": { "added": 0, "removed": 0, "modified": 1 },
  "changes": [
    {
      "object_id": "instance:app",
      "kind": "modified",
      "before": { "id": "app", "config": { "title": "Old" }, "effective_config": { "title": "Old" } },
      "after": { "id": "app", "config": { "title": "New" }, "effective_config": { "title": "New" } }
    }
  ]
}
```

- 覆盖实例、Binding、Connection；对象值是该对象的规范化 JSON。
- `replace_source` 额外返回 `source_diff: { before, after }`（完整文本；行级展示由 UI 负责）。
- `kind` 为 `added` / `removed` / `modified`；未变化的字段不单独列出，修改时给完整 before/after。
- 重命名实例表现为旧 ID `removed` + 新 ID `added`；引用它的 Binding/Connection 是 `modified`。
- Diff 即本次事务的**影响集合**；调用方可用它做预览或影响分析。
- 模块定义只读，不在 Diff 中；依赖解析结果暂不单列（可由实例/端口 Diff 推导）。

## 6. Revision 历史与幂等范围

- 工作区历史包含 `initial`（`seq = 0`）与每次成功提交的
  `{ revision, request_id, diff }`。
- 重放返回原请求的结果（可能是旧 Revision/模型），不覆盖当前工作区。
- 010/011 历史与 request 记录只在工作区内存；跨进程持久化尚未实现。
- 幂等只覆盖已成功提交的 request；失败事务不记录。

## 7. TOML 写回

`ProjectSession` 在 `EditorWorkspace` 之上把事务落到 composition 入口：

1. 命令先在候选模型上完成 L2–L7 校验；
2. 用 `toml_edit` 对 `DocumentMut` 做局部编辑，只改命令涉及的对象/字段；
3. 写临时文件 → 备份原文件 → 替换 → 重新 `validate_project`；
4. 重新校验失败时恢复备份，并把 `edit-transaction-failed` 或原诊断返回；
5. 成功后刷新 SourceMap/`source_digest`，当前 Revision 的 digest 与历史记录同步更新。

范围：只写 `workspace.composition` 指向的单个 composition 文件；`project.toml`、模块定义与
`.designer/**` 不写。未编辑部分的注释与排版由 `toml_edit` 保留；新建表使用默认排版。
`replace_source` 直接写入用户文本，不经过 `toml_edit` 重新格式化；写回是备份/替换/回滚，
不提供 fsync 级崩溃恢复。

## 8. 未包含（后续卡）

- 跨进程 Revision/request 历史持久化、fsync 崩溃恢复。
- 新对象在 `EditorWorkspace` 内存候选中没有 `SourceRef`；`ProjectSession` 写回后重新解析会补齐，
  写回前诊断只带对象 ID。
- 撤销/重做、模块定义编辑、Port 协议协商、外部修改同步。
