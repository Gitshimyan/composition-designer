# Reviews

外部评审与路线图归档。评审建议是输入，不代表已自动采纳或已排期；正式范围以
[`working/plan.md`](../../working/plan.md)、任务卡与
[`working/decisions/unresolved-work.md`](../../working/decisions/unresolved-work.md) 为准。

| 日期 | 评审 | 结论 |
|---|---|---|
| 2026-11-27 | [项目评审与后续发展方向](./2026-11-27-project-review-and-roadmap.md) | v0.1 保持冻结；下一阶段建议 v0.2 / M6 Complete Composition Authoring |
| 2026-11-27 | [授权体系与任务卡审计](./2026-11-27-authorization-audit.md) | 五种模式应改为权限集合；002/003/009/010/016 等历史分类需记录；025 Manifest 漏列 Protocol target |
| 2026-11-27 | [001–025 历史授权审计记录](./2026-11-27-authorization-audit-record.md) | 逐卡审计分类与 Manifest 差异；不追溯改写历史授权 |
| 2026-11-27 | [001–025 未实现工作回溯](./2026-11-27-unimplemented-work-backfill.md) | 遗留线索去向、27 项 UW、任务卡 coverage/open_items |
| 2026-11-27 | [未实现功能显性化与授权阻塞治理](./2026-11-27-unimplemented-work-transparency.md) | 未实现事项必须有稳定 ID、影响、授权和汇报章节；`done` 与需求覆盖状态分离 |
| 2026-11-27 | [评审响应与整改计划](./2026-11-27-audit-response.md) | 已采纳；G01 完成，G02 下一步；治理优先于 026；G1–G3 不自动展开 |

## 当前高优先级缺口

M6 020–025 已完成：CI/UI 测试、Glide 试点、Instance/Binding 编辑、Connection/Dependency
视图、Dependency Provider Extend。当前评审关注点转为：

1. 授权规则从单值等级改为可组合权限集合（G01，done）；
2. 任务卡/Manifest 模板与未实现披露制度（G01，done）；
3. 001–025 历史授权审计记录与 Manifest 事实性修正（G02，done）；
4. 历史未实现工作回溯与登记（G03，done）；
5. 026 完整 Change Review（done）；
6. 机器授权/覆盖门禁（G04，待授权）。

未决事项见 [`working/decisions/unresolved-work.md`](../../working/decisions/unresolved-work.md)。
