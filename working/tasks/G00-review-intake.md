# G00 — 评审归档与整改计划

- 状态：done
- 目标：归档 2026-11-27 的两份评审，形成整改计划、未决工作登记与任务卡队列；不执行授权规则变更。
- 授权：Modify（`docs/reviews/`、`working/`、`docs/ai/`、Manifest 事实性修正）
- 变更声明：`changes/G00-review-intake.yaml`
- 来源：根目录两份评审文档（已移动归档）

## 范围

- 归档：授权体系审计、未实现功能显性化；
- 响应：接受多模式权限、未实现披露、历史审计与机器门禁方向；治理优先于 026；
- 计划：G01–G04 队列、026 顺延、依赖与授权状态；
- 登记：`working/decisions/unresolved-work.md` 首批 UW；
- 事实性修正：025 Manifest 补 Protocol 测试 target；020 `expected_command_changes=false`；
- 不修改 `AGENTS.md`、任务卡模板、Manifest 模板、领域模型、TOML Schema 或产品代码。

## 清单

- [x] 两份根目录评审移入 `docs/reviews/` 并更新索引
- [x] 新增评审响应与整改计划
- [x] 新增 G01–G04 任务卡（G01 等待 Rearchitect 授权）
- [x] 建立未决工作登记册与首批 UW
- [x] 建立决策历史
- [x] 更新 `working/plan.md` 的整改队列、未决总览与下一步
- [x] 修正 025/020 Manifest 的事实性声明
- [x] 更新 architecture-entry 当前阶段

## 验收

- 根目录无散落评审文件；
- 每份评审可从 `docs/reviews/README.md` 进入；
- `working/plan.md`、`unresolved-work.md`、G 任务卡互相链接且状态一致；
- G01 标记为等待 Rearchitect 授权，未开始；
- 没有修改 `AGENTS.md`、任务卡模板、设计基线、Schema 或产品代码；
- 实际改动文件与 `changes/G00-review-intake.yaml` 一致。

## 结果

- 评审归档与整改计划完成；下一步等待用户决定是否批准 G01 的 Rearchitect 范围。
- 未决事项 13 项：1 已关闭、5 等待决策、2 等待授权、2 已接受待排期、3 延期。
