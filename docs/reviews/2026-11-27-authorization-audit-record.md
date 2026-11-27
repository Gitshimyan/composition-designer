# 001–025 历史授权审计记录

- 日期：2026-11-27（G02）
- 来源：[授权体系与任务卡审计](./2026-11-27-authorization-audit.md)
- 性质：**审计记录**，不是对历史授权的追溯批准
- 状态：G02 已完成；历史任务卡原始授权字段保持不变

## 1. 判定原则

1. 授权按实际改变的对象、语义和契约判断，不按文件位置或工作量判断；
2. 一个任务可以同时需要多个模式；不能用“最高一级”替代缺失权限；
3. 本记录只写审计建议与证据状态，不把建议改写成“当时已获批准”；
4. 旧 Manifest 保持原格式；新任务使用 G01 后的权限集合模板。

## 2. 总表

| 卡 | 原始授权 | 审计建议 | 证据 / 判定状态 | Manifest 备注 |
|---|---|---|---|---|
| 001 Bootstrap | Rearchitect | Rearchitect | 合理 | 无独立 Manifest |
| 002 Domain Model | Modify | **Rearchitect** | 等级不足 | 无独立 Manifest（M0 历史） |
| 003 TOML Contract Schema | Modify | **Extend + Rearchitect** | 等级不足 | 无独立 Manifest（M0 历史） |
| 004 Example Projects | Extend | **Extend + Compose** | 权限不完整 | 无独立 Manifest（M0 历史） |
| 005 Rust Workspace / Domain | Extend | Extend；workspace 基线部分 Rearchitect | 条件性遗漏 | `005-rust-domain.yaml` |
| 006 TOML Loader | Extend | Extend | 合理 | `006-toml-loader.yaml` |
| 007 Composition Core | Extend | Extend + Modify | 基本合理 | `007-composition-core.yaml` |
| 008 Composition CLI | Extend | Extend | 基本合理 | `008-composition-cli.yaml` |
| 009 Editor Transaction | Extend | **Extend + Rearchitect** | 等级不足 | `009-editor-transaction.yaml` |
| 010 Revision / Diff | Modify | **Modify + Rearchitect** | 等级不足 | `010-revision-diff.yaml`；`expected_command_changes=true` 与授权不一致 |
| 011 TOML Writeback | Modify | Modify | 合理 | `011-toml-writeback.yaml` |
| 012 Designer Shell | Extend | Extend；workspace/宿主工程边界部分 Rearchitect | 条件性遗漏 | `012-designer-shell.yaml` |
| 013 Canvas / Tree | Extend | Extend | 合理 | `013-canvas-tree.yaml` |
| 014 Inspector / Source | Extend | Extend + Modify；破坏性协议变化再评估 Rearchitect | 需明确契约边界 | `014-inspector-source.yaml` |
| 015 External Sync | Extend | Extend；改变共享事务协议再评估 Rearchitect | 需明确契约边界 | `015-external-sync.yaml` |
| 016 Protocol / CLI JSONL | Extend | **Extend + Rearchitect** | 等级不足 | `016-protocol-cli-jsonl.yaml` |
| 017 MCP Adapter | Extend | Extend | 合理 | `017-mcp-adapter.yaml` |
| 018 Agent Skill / E2E | Extend | Extend | 合理 | `018-agent-skill-e2e.yaml` |
| 019 Integrity / Acceptance | Modify | Modify | 合理 | `019-integrity-acceptance.yaml` |
| 020 Quality Gates | Extend | Extend；项目级强制门禁另需 Rearchitect | 条件性遗漏 | `020-quality-gates.yaml`；`expected_command_changes` 已修正为 false |
| 021 Glide Pilot | Compose + Modify | 类型合理；Compose 路径不足 | 路径定义不足 | `021-glide-pilot.yaml`；G01 已覆盖 `working/pilots/**` |
| 022 Instance Editing | Modify | Modify | 合理 | `022-instance-editing.yaml` |
| 023 Binding Editing | Modify | Modify | 合理 | `023-binding-editing.yaml` |
| 024 Relation Views | Modify | Modify | 合理 | `024-relation-views.yaml` |
| 025 Dependency Provider | Extend + Modify | Extend + Modify | 正确 | `025-dependency-provider.yaml`；已补 Protocol target |

## 3. 重点说明

### 002 / 003 / 004（M0 历史）

- 002 冻结了领域对象、关系语义与不变量，审计建议 Rearchitect；原卡仍写 Modify（仅 docs）。
- 003 同时建立 TOML 字段/契约并确定 Schema 结构与版本策略，审计建议 Extend + Rearchitect。
- 004 新增示例类型并编写真实组合声明，审计建议 Extend + Compose。
- 三张卡没有独立 Change Manifest；作为 M0 初始化历史保留，不补造当时文件。

### 005 / 012 / 020（条件性 Rearchitect）

- 005：新建 crate 属 Extend；首次建立 Cargo workspace、resolver、edition 等全局基线，
  该部分属 Rearchitect。
- 012：新 Tauri/React 应用属 Extend；将 app 排除出根 workspace、独立 lock/build 边界
  属 Rearchitect。
- 020：workflow、Playwright、fixture 属 Extend；若定义全项目强制合并门禁策略，
  该部分属 Rearchitect。当前实现是 push 触发的 CI，不声称已建立合并门禁。
- 不因这三张卡存在条件性边界就重写其历史授权；后续同类变更按 G01 规则判断。

### 009 / 010 / 016（共享协议）

- 009 首次定义 Command / Transaction / base_revision / 原子提交语义，属命令协议。
- 010 增加 Revision 历史、幂等、重放、冲突与 Diff 契约；其 Manifest
  `authorization: modify` 与 `expected_command_changes: true` 不一致。
- 016 建立 Agent/CLI Request / Response / Operation / JSONL 通信契约。
- 审计建议均包含 Rearchitect；本记录保留原授权，不追溯改写。

### 014 / 015（契约边界）

- 014 新 UI/IPC 属 Extend，修改既有实现/测试属 Modify；新增兼容字段不必然 Rearchitect，
  改变字段语义/协议版本才升级。
- 015 外部检测/Reload 若只在 Host 复用既有冲突契约，属 Extend；若改变 Core/Protocol 的
  prepare/apply 接受规则，需 Rearchitect。

### 025（Provider Extend）

- 新增 `set_dependency_provider` / `unset_dependency_provider` 属 Extend；
  UI、写回、测试属 Modify；
- 未改领域模型、TOML 字段或设计基线；
- 审计指出 Manifest 漏列 `crates/composition-protocol` 测试，已补 target 与
  `protocol_transaction_test`（事实性修正，不改产品代码）。

## 4. Manifest 修正清单

| Manifest | 修正 | 状态 |
|---|---|---|
| `025-dependency-provider.yaml` | 补 `crates/composition-protocol`、`protocol_transaction_test` | 已修正 |
| `020-quality-gates.yaml` | `expected_command_changes: false`，注明生产 IPC 未变 | 已修正 |
| `010-revision-diff.yaml` | 记录 `authorization` 与 `expected_command_changes` 不一致；不追溯改授权 | 已记录 |
| `021-glide-pilot.yaml` | 记录 Compose 路径缺口；`working/pilots/**` 已由 G01 覆盖 | 已记录 / 规则已修 |
| `002`–`004` | 无独立 Manifest | 作为 M0 历史保留 |

## 5. 结论

- 002、003、009、010、016 是最明确的授权分类遗漏；
- 005、012、020 为条件性边界，不一律升级；
- 004 应补 Compose；021 的试点路径已纳入 G01 的 Compose 范围；
- 025 的 Extend 决策正确，Manifest 事实性遗漏已修正；
- 本记录不改变旧任务卡的授权字段，也不声称当时已获得 Rearchitect 授权。

## 6. 后续

- G03：对 001–025 做未实现工作回溯，把仍开放的能力登记为 `UW-*`；
- 026：完整 Change Review；
- G04：机器授权/覆盖门禁。

相关入口：[整改计划](./2026-11-27-audit-response.md) ·
[未决登记](../../working/decisions/unresolved-work.md) ·
[计划](../../working/plan.md)。
