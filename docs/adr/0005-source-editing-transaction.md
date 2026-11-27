# ADR-0005 — Source 编辑事务（M7）

- 状态：**Accepted（2026-11-27；029 已实现）**
- 授权：用户 2026-11-27 选择 A（独占 `replace_source`），批准 Extend + Rearchitect
- 关联：[`028-m7-source-design.md`](../../working/tasks/028-m7-source-design.md)、
  [`v0.1-design.md`](../design/v0.1-design.md) §7.4–§7.5、§8、
  [`command-schema-v1.md`](../spec/command-schema-v1.md)、`ProjectSession`

## 背景

Source 目前只读。v0.1 的所有写操作都经 `Transaction` + `ProjectSession`，但事务命令是
对象级（Instance/Binding/Connection/Config），没有“替换 Composition 文本”的路径。

Source 编辑不能：

- 让 GUI/Agent 直接写文件；
- 另建一套不经过 Revision/Diff/诊断的写入路径；
- 每次保存都重新格式化整个 TOML；
- 在非法文本上提交部分修改。

M7 需要决定：Source 编辑如何进入统一事务、如何审查、如何处理草稿/外部冲突。

## 选项

| 选项 | 做法 | 优点 | 代价 |
|---|---|---|---|
| A | 事务内新增独占的 `replace_source` 命令，携带完整文本 | 复用 Revision/幂等/Diff/Review；单一事务 | 需要 source-aware 候选/写回；协议扩展 |
| B | 独立 `SourceTransaction` 协议 | 语义清晰 | 复制 Revision/Review/幂等；两个写入路径 |
| C | 分区 patch 命令（按文本范围替换） | 精细 | 位置漂移、冲突、Diff 复杂；容易变成隐式编辑 |
| D | UI 直接写文件后 reload | 简单 | 绕过 Core/事务；不可接受 |

## 推荐：A（独占 `replace_source`）

### 命令

```json
{
  "type": "replace_source",
  "source": "full composition TOML text"
}
```

- 只允许编辑当前 `composition_entry`；
- 不提供任意路径；
- 一个事务只能包含这一条命令，不能与对象命令混合；
- 复用 `Transaction.request_id` / `base_revision`；
- 模块定义仍只读，不在 M7 首版开放。

### prepare

1. 用候选文本解析/加载 Composition；
2. 构建候选模型并执行 L1–L7 验证；
3. 解析失败或验证失败：整体拒绝，返回真实诊断，不写文件；
4. 成功：返回文本 Diff、对象级 ModelDiff、完整诊断、候选 Revision；
5. 不修改磁盘/当前工作区。

### apply

1. 校验 `base_revision`/当前 digest；磁盘外部修改则拒绝；
2. 将用户文本原样写入临时文件；
3. 备份原文件 → 替换 → 重新加载/验证；
4. 成功：刷新 `ProjectSession` 的文本/Document/模型/Revision/Digest；
5. 失败：恢复备份与工作区，保留诊断；
6. 不通过 `toml_edit` 重新格式化；用户文本就是权威格式。

### Diff 与 Review

- 文本 Diff：行级 before/after（新增协议/响应字段）；
- 对象 Diff：复用现有 `ModelDiff`；
- Review 显示两者 + 完整诊断；accepted 才能 Apply；
- 取消/失败不写入。

### 冲突与草稿

- 外部 digest 变化 → 阻止 prepare/apply，提示 Reload；
- Source 草稿保留在 UI 内存；Reload 丢弃草稿（首版明确提示）；
- 不做三向合并/自动保留；后续 UW-017 处理；
- 不自动覆盖外部文件。

### CLI / MCP

- `apply` 接受含 `replace_source` 的 transaction JSON；
- `source` 查询返回当前文本；
- Protocol/MCP 不新增直接文件写入口；
- 测试覆盖 prepare/apply/拒绝/冲突。

### 错误与限制

- 复用 `edit-invalid-command`、`edit-stale-revision` 与 L1–L7 诊断；
- 若需要新错误码，先更新错误码表；
- 首版只支持 Composition 入口，不支持模块文件；
- 文件大小/编码沿用 loader 规则，不新增隐式转换；
- 不做自动格式化、自动修复或部分提交。

## 影响

- Extend：新增 `replace_source` 命令、source diff 响应；
- Rearchitect：共享命令/事务协议增加文本命令；
- Editor：`ProjectSession` 需要 source-aware prepare/apply 分支；
- Protocol/CLI/MCP：transaction/source 查询；
- Designer：CodeMirror 可编辑、草稿、Review、冲突提示；
- 领域模型：不改变对象语义；Source 只是同一模型的文本入口；
- 测试：源文本保留、非法文本不写、外部冲突、重开一致、CLI/MCP。

## 被否决的替代方案

- **独立 SourceTransaction**：复制 Revision/Review/幂等，产生第二写入路径；
- **文本 patch 命令**：位置漂移和冲突难以可靠审查；
- **直接文件写入**：绕过 Core，违反单一权威模型；
- **只改 UI**：无法覆盖 CLI/MCP，也无法保证原子性。

## 待批准 / 已确认

用户 2026-11-27 选择 A 并批准继续：

- 接受独占 `replace_source` + 事务复用；
- 首版只编辑 Composition 入口；
- 外部冲突阻止、Reload 丢弃草稿、无三向合并。

029 已按此实现；030/031 仍待执行。
