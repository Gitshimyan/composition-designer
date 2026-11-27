# G07 — 领域缺口设计研究（G1/G2/G3）

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-002、UW-003、UW-004
- 目标：完成接口身份、服务容器、项目 Profile 的设计研究，给出 ADR 提案与 M7/M9 重排建议；不实现 Schema 变更。
- authorization：
  - required：modify
  - approval：
    - modify：default（设计研究/文档）
    - rearchitect：pending（Schema v2 实施需单独明确授权）
- 来源：UW-002/003/004；021 Glide 试点；用户 2026-11-27「先做设计研究，开始 G7」
- 变更声明：`changes/G07-domain-gaps.yaml`

## 研究范围

1. 复核 v0.1 Domain/Schema 中 Dependency、Binding、Profile 的当前语义；
2. 复核 Glide 试点 G1/G2/G3 证据；
3. 比较接口身份、service 容器、Profile 的候选方案；
4. 给出 v2.0 / v2.1 分阶段建议、迁移原则与 M7/M9 影响；
5. 不修改 Schema、领域模型、设计基线或产品代码。

## 非目标

- 不实现 Schema v2；
- 不提前选择或冻结接口/Profile 字段；
- 不修改 021 试点；
- 不把设计研究当作 Rearchitect 实施授权。

## 验收

- 三个缺口都有当前行为、真实证据、候选方案、推荐与影响；
- 明确 v1 兼容/迁移原则；
- 明确 M7/M9 的顺序影响；
- 决策项与所需授权可执行；
- 无 Schema/领域模型/产品代码变更。

## 结果 / 遗留问题

- 结果：
  - 新增 [`2026-11-27-g7-domain-gaps.md`](../../docs/reviews/2026-11-27-g7-domain-gaps.md)；
  - 新增 [`ADR-0004`](../../docs/adr/0004-schema-v2-direction.md)：
    - v2.0 = G1 模块契约接口标记 + G2 实例 scope；
    - v2.1 = G3 Profile-scoped Composition 入口（依赖多入口）；
    - v1 继续可读，不自动迁移；
    - M7 不阻塞；S1 v2.0 在 M7 后、M8 前；M9 被 S1/S2 解锁；
  - 用户 2026-11-27 明确确认 ADR-0004 推荐方案；ADR 状态更新为 Accepted。
- 未决：
  - UW-002/003/004 方向已接受，但实现仍待 S1/S2 逐卡 Rearchitect 授权；
  - `create_instance` scope 实施需要 Extend。
- 验证：
  - 设计文档/链接检查；
  - 未运行产品测试（无代码变更）。
- 下一步：用户决策；然后重排 M7/M9，创建 S1/S2 设计与实施卡。