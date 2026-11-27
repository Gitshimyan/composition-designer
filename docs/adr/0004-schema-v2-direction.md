# ADR-0004 — Schema v2 方向：接口身份、服务 scope 与项目 Profile

- 状态：**Accepted（2026-11-27）**
- 授权：用户 2026-11-27 明确确认推荐方案（v2.0/v2.1、接口标记、实例 scope、Profile 入口、S1 在 M7 后）
- 关联：[`2026-11-27-g7-domain-gaps.md`](../reviews/2026-11-27-g7-domain-gaps.md)、
  [`composition-schema-v1.md`](../spec/composition-schema-v1.md)、
  [`domain-model-v1.md`](../spec/domain-model-v1.md)、021 / G07 任务卡

## 背景

Glide Task + Terminal 试点证明 v0.1 能表达拓扑，但不能表达：

1. **接口/契约身份**：Dependency 目标只能是模块 ID，`ITerminalSessionFactory` 等端口
   无法与实现解耦；
2. **非结构 runtime/service 容器**：单根 Binding 树要求所有实例从 root 可达，
   服务被塞进人工 `runtime` Slot，形成假结构；
3. **项目级 Profile**：`glide.toml` 的 project profile/环境/task 差异无法用单份
   Composition + 静态实例配置表达。

这三项都属于 Rearchitect，不是 UI 或局部字段问题。

## 决策（提案）

分阶段引入 Schema v2：

- **v2.0**：G1 接口身份 + G2 service scope；
- **v2.1**：G3 项目 Profile + 多 Composition 入口；
- v1 项目继续可读，不自动迁移；v2 通过 `schema_version` 显式启用。

### G1：模块契约内的接口标记

- 模块可声明 `[[contract.interfaces]]`（id + 精确版本）；
- 依赖可用 `interface` 替代 `module`，二选一；
- provider 模块必须声明对应 interface；显式 provider 映射不变；
- 自动解析在实现该接口的实例中唯一匹配，歧义仍报错；
- 接口先是身份标记，不承诺完整方法/端口 shape。

### G2：实例 scope

- `[[instances]]` 增加 `scope = "structure" | "service"`，默认 structure；
- service 不参与 Binding 树，不得有父/子 Binding；root 必须 structure；
- service 仍可有 config、dependency、Port/Connection，进入独立 Services 视图；
- `create_instance` 命令增加可选 scope（Extend）。

### G3：Profile 入口

- `[profiles.<name>] composition = ...`，每个 profile 是完整 Composition；
- 默认 profile 保持 `workspace.composition`；
- CLI/MCP/Designer 显式选择 profile；Source/Revision/Digest 针对当前 profile；
- 首版不做 overlay/继承，避免隐式合并；
- 依赖 UW-018 多入口，排入 v2.1。

## 影响

- 领域模型：InterfaceDeclaration、Instance scope、Profile entry；
- Schema：新字段、v2 诊断、SourceMap；
- Loader/Core：接口解析、service 可达性、profile 选择；
- Editor/Commands：`create_instance` scope、provider 目标 interface；
- Designer：Dependency interface 展示、Services 视图、Profile 选择；
- CLI/MCP：profile 参数、v2 模型输出；
- 测试：v1 回归 + v2 fixtures + 迁移/拒绝路径。

## 被否决的替代方案

- **继续用模块名模拟接口**：无法替换实现，真实项目映射失真；
- **继续人工 runtime Slot**：把服务伪装成结构子节点，污染 Binding 树；
- **单 Composition + Profile overlay**：合并、验证、Diff 语义复杂，容易变成隐式兼容；
- **自动迁移 v1 → v2**：违反“不静默修复/升级”原则。

## 未决 / 已确认

以下方向已由用户 2026-11-27 确认：

- 接受 v2.0/v2.1 分阶段方案；
- 接口采用模块契约内的接口标记；
- service 采用实例 `scope`；
- Profile 采用完整 Composition 入口；
- S1 放在 M7 之后、M8 之前。

实施仍需逐卡授权；在 S1/S2 实现前，不修改 Schema、领域模型或设计基线。
