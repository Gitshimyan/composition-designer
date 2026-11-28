# 决策与授权历史

> 记录用户明确决策、授权与重要计划变更。只记录已发生的事实；提案写入任务卡/评审响应，
> 不在这里伪装成已批准决定。

| ID | 日期 | 决策 / 授权 | 依据 | 影响 |
|---|---|---|---|---|
| D-001 | 2026-11-27 | 明确授权补齐 Dependency Provider 编辑（Extend） | 用户对话 | 025 新增 set/unset 命令、命令 Schema 与 UI；不改领域模型/TOML 字段 |
| D-002 | 2026-11-27 | 明确要求推送远端进行评审 | 用户对话 | `e32b871` 推送；远端 CI run 45984567517 通过 |
| D-003 | 2026-11-27 | 接收授权审计与未实现披露两份评审，要求整理并编排计划 | 用户对话 | 评审归档到 `docs/reviews/`；提出 G01–G04 整改队列，等待范围批准 |
| D-004 | 2026-11-27 | 修正 025 Manifest 的 Protocol 测试 target、020 的 `expected_command_changes` | 授权审计事实性发现 | 只修正声明覆盖，不追溯授权或改产品代码 |
| D-005 | 2026-11-27 | 治理优先于 026；G1–G3 不随评审自动展开 | 计划提案；G01 已获授权启动 | 计划队列 G01 → G02 → G03 → 026 → G04 |
| D-006 | 2026-11-27 | 明确确认 G01 的 Rearchitect 范围并指示开始 | 用户对话 | `AGENTS.md`、任务卡/Manifest 模板、未决登记与汇报规则生效；产品代码/Schema 未改 |
| D-007 | 2026-11-27 | 完成 G02 历史授权审计记录与 Manifest 事实性修正 | G01 后默认 Modify；审计记录 | 001–025 分类可追溯；不追溯改写历史授权；025/020 事实性修正 |
| D-008 | 2026-11-27 | 完成 G03 历史未实现工作回溯 | G01 后默认 Modify；回溯报告 | 001–025 遗留线索有去向；登记册扩展至 27 项；不改历史范围/授权 |
| D-009 | 2026-11-27 | 完成 026 完整 Change Review | M6-006；Modify 默认授权 | 分类/Before-After/命令/依赖影响/完整诊断；28 UI + 154 workspace + 15 Tauri；M6 功能卡完成，027 收口待执行 |
| D-010 | 2026-11-27 | 完成 027 M6 收口验收 | M6 总验收；Modify | Glide 全链路 CLI/Protocol 通过；155 workspace；PR 分支保护 404，UW-029 待配置；M6 功能验收完成 |
| D-011 | 2026-11-27 | 完成 G04 机器门禁与 UW-029 | 用户明确授权 Extend + Rearchitect + 远端设置 | change-gate report 接入 CI；main 分支保护启用，失败探针 PR #1 被 BLOCKED；M6 完成；block 模式转 UW-030 |
| D-012 | 2026-11-27 | 完成 UW-030 / G05 block 模式 | 用户明确要求 UW-030；Modify + Rearchitect | CI 切换 block；重叠 Manifest 任一授权；声明违规阻止合并；UW-030 关闭 |
| D-013 | 2026-11-27 | 完成 G06 Post-M6 计划重排 | 用户明确要求重排；Modify | 归档 v0.2 计划；v0.3 R1→M7→M8→H1→M9；下一卡 028 Source 设计 |
| D-014 | 2026-11-27 | 完成 G7 领域缺口设计研究 | 用户要求先研究 G7；Modify | ADR-0004 Proposed：v2.0 G1+G2，v2.1 G3；S1/S2 插入路线图；未改 Schema |
| D-015 | 2026-11-27 | 确认 ADR-0004 推荐方案 | 用户明确确认 ADR-0004 | v2.0/v2.1、接口标记、实例 scope、Profile 入口；S1 在 M7 后、M8 前；实施逐卡授权 |
| D-016 | 2026-11-27 | 完成 028 M7 Source 设计 | 用户确认 ADR-0004 后继续；Modify（研究） | ADR-0005 Proposed：独占 replace_source、文本+对象 Diff、冲突阻止；029 待 Extend + Rearchitect |
| D-017 | 2026-11-27 | 完成 029 Source 后端 | 用户选择 A 并批准；Extend + Rearchitect | replace_source、source prepare/apply、文本+对象 Diff、冲突守卫；172 tests；030 待做 |
| D-018 | 2026-11-27 | 完成 030 Source UI | M7；Modify | CodeMirror 编辑、draft、Review/文本 Diff、Discard、冲突只读；30 UI |
| D-019 | 2026-11-27 | 完成 031 M7 Source 验收 | M7；Modify | Glide 试点 Source 编辑；173 tests/16 Tauri/30 UI；M7 完成 |
| D-020 | 2026-11-27 | 关闭 UW-011，登记 UW-031 | M7 完成后的覆盖审计 | Composition Source 已实现；模块文件编辑仍只读，转 UW-031 deferred |
| D-021 | 2026-11-28 | 明确授权：先修正文档漂移（G09），再执行 032 Schema v2.0 ADR 批准与接口/服务语义冻结（Rearchitect + Extend） | 用户对话 | G09 文档一致性；032 可执行；033–035 仍需逐卡授权 |
| D-022 | 2026-11-28 | 完成 032 Schema v2.0 语义冻结 | 用户授权（D-021）；032 卡 | ADR-0006、v2.0 规范与 8 个新代码；接口/scope/命令/兼容矩阵冻结；UW-002/003 in_progress；新增 UW-032；无代码变更 |
| D-023 | 2026-11-28 | 完成 033 Domain/Loader/Validation | 用户明确授权开始 033；Rearchitect + Modify | 接口/接口依赖/scope 类型、解码、版本矩阵、8 个诊断；v1 输出不变；workspace 189 测试；无命令/UI/示例改动 |
| D-024 | 2026-11-28 | 完成 034 Commands/UI/CLI/MCP | 用户明确授权进行 034；Extend + Modify | `create_instance.scope` 与写回、Services 视图/创建、接口展示；CLI/MCP v2；196 workspace / 34 UI / 17 Tauri；无 Schema/基线改动 |
| D-025 | 2026-11-28 | 完成 035 v2 验收，S1 收口 | 用户要求开始 035；Modify + Compose | 11 个 v2 数据集、迁移拒绝、Glide v2 试点 CLI 验收；UW-002/UW-003 关闭；S1 完成 |
| D-026 | 2026-11-28 | 完成 036 Agent context/impact | 用户明确授权 M8（036–038）；Extend + Rearchitect（协议文档） | protocol/CLI/MCP context/impact；只读；协议/MCP/Skill 更新；210 workspace 测试 |
| D-027 | 2026-11-28 | 完成 037 只读 batch | 用户授权 M8；Extend + Rearchitect（协议文档） | batch 1..16、只读白名单、逐项结果；CLI/MCP；218 workspace 测试；progress/cancel 仍 deferred |
| D-028 | 2026-11-28 | 完成 038 Skill 评测与 M8 收口 | 用户授权 M8；Modify + Extend | skill_eval（minimal + Glide v2）、configure.mjs、EVALUATION；220 workspace 测试；M8 完成 |
| D-029 | 2026-11-28 | 完成 039 Schema v2.1 语义冻结 | 用户明确授权开始 S2（039–041）；Rearchitect + Extend | format 3、`[profiles]`、选择/Revision/Digest/3 个诊断；ADR-0007；UW-004/018 in_progress；无代码变更 |
| D-030 | 2026-11-28 | 完成 040 Core/Session profile 实现 | 用户授权 S2；Rearchitect + Modify | v3、Loader/Core 选择、Revision/Digest、Session/Source；233 workspace；v1/v2.0 不变 |
