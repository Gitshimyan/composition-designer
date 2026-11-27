# NNN — <标题>

<!--
状态：todo | in_progress | blocked | done
coverage_status：complete | complete_with_deferred_items | partial | blocked_by_authorization
authorization.required：实际需要的全部模式（inspect/modify/compose/extend/rearchitect）
approval：granted（用户明确批准）/ default（默认允许）/ pending（等待授权）
-->

- 状态：todo
- coverage_status：partial
- open_items：无
- 目标：<一句话结果，可验证>
- authorization：
  - required：modify
  - approval：
    - modify：default
- 变更声明：`changes/<file>.yaml`

## 背景

<为什么现在做这张卡；只写与决策相关的事实>

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | <具体能力> | implemented / partial / blocked / deferred | <文件、测试或原因> |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| 无 | — | — | — | — | — | — |

> 所有非“无”的事项必须登记到
> [`working/decisions/unresolved-work.md`](../decisions/unresolved-work.md)。

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| — | — | — | — | — |

## 清单

- [ ] <可勾选的具体动作>
- [ ] <实现 / 样例 / 测试 / 文档>

## 验收

- <可机器验证或可逐项核对的条件>
- <变更范围与 Change Manifest 一致>
- <未实现项已登记，coverage_status 与 open_items 如实>

## 结果 / 验证

- 实际执行：
- 未执行：
- 结果：
- 遗留 / 后续：
