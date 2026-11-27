# G06 — Post-M6 计划重排

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：在 M6 与 G01–G05 完成后，重新编排 v0.3 路线图、下一张卡与决策门，避免继续沿用已完成的 M6 队列。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：用户 2026-11-27「开始重新编排计划」
- 变更声明：`changes/G06-plan-rebaseline.yaml`

## 范围

1. 归档 v0.2 M6 计划，保留历史；
2. 建立 v0.3 路线图：R1 Source 设计 → M7 Source Authoring → M8 Agent → H1 可靠性 → M9 生态；
3. 明确 G1–G3、ConfigValue wire、array、Binding UX 等决策门，不自动实现；
4. 指定下一张卡：**028 — M7 Source 设计决议**；
5. 不创建 M7 实现，不修改设计基线、Schema、命令协议或产品代码。

## 非目标

- 不实现 Source 编辑；
- 不提前承诺 M9；
- 不把未决 decision 转成已批准；
- 不修改产品代码/测试。

## 验收

- `working/plan.md` 反映 Post-M6 队列与当前状态；
- 旧计划已归档；
- 下一张卡与授权路径清晰；
- 未决事项数量与登记册一致；
- 变更范围与 Manifest 一致。

## 结果 / 遗留问题

- 结果：
  - 归档 [`working/archive/plan-v0.2-m6.md`](../archive/plan-v0.2-m6.md)；
  - 新计划 [`working/plan.md`](../plan.md)：R1 → M7 → M8 → H1 → M9，G7 决策门；
  - 下一卡：028 M7 Source 设计决议；
  - 未决登记保持 30 项（10 关闭、8 待决策、12 延期）。
- 验证：只读/文档变更；无代码测试需要运行；Markdown 链接检查与 `git diff --check` 执行。
- 下一步：028 — M7 Source 设计决议（需先读设计基线与命令协议，产出 ADR）。