# ADR-0003 — Dependency Provider 编辑命令（v0.2 Extend）

- 状态：Accepted（2026-11-27）
- 授权：用户明确授权补齐 024 记录的 Extend 缺口
- 关联：[`command-schema-v1.md`](../spec/command-schema-v1.md)、024 / 025 任务卡

## 背景

024 的 Dependency 视图只能读取解析结果：v0.1 冻结的 12 条编辑命令没有设置或清除
`dependency_providers` 的能力。显式 Provider 映射只能通过 TOML 文本或实例重命名/删除
间接变化，GUI 无法完成“选择候选 Provider”的完整编辑链路。

用户明确授权补齐该能力。按 AGENTS §1，新命令属于 Extend；本次不修改领域对象、TOML 字段、
`schema_version` 或设计基线。

## 决策

在 `composition-editor` 增加两条命令，并扩展 `docs/spec/command-schema-v1.md`：

```json
{ "type": "set_dependency_provider", "instance": "engine", "requirement": "logger", "provider": "logger-1" }
{ "type": "unset_dependency_provider", "instance": "engine", "requirement": "logger" }
```

规则：

1. `set` 只写实例的显式映射；`unset` 删除显式映射并回退自动解析。
2. 实例不存在、`unset` 时没有显式映射 → `edit-invalid-command`。
3. requirement 是否在模块契约、provider 是否存在/满足目标模块与版本、是否自引用
   → 候选模型 L2–L7 诊断，不在命令层复制规则。
4. TOML 写回 `[instances.dependencies]`：Set 写键，Unset 删除键并在表为空时移除表；
   其余格式由 `toml_edit` 保留。
5. Designer 的候选列表只显示目标模块/版本匹配的实例；最终合法性仍由 Core 判定。

## 影响

- `COMMAND_KINDS` 12 → 14；CLI/MCP/Protocol 复用同一 `Transaction`，无 Adapter 语义变化。
- Command Schema 增加命令表与错误说明；领域模型、TOML Schema、错误码集合不变。
- 新增 Editor/Session/Protocol/Tauri/UI 测试覆盖 Set、Unset、非法 provider 与写回。

## 被否决的替代方案

- **只改 UI / 直接写 TOML**：绕过 Core 事务，违反单一写入路径。
- **新增 TOML 字段**：现有 `[instances.dependencies]` 已能表达显式映射，无字段缺口。
- **按 Rearchitect 改领域模型**：Provider 指回 Instance ID 的语义已经存在，不需要模型变更。
