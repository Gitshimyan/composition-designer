# Multiple Parents

- 状态：M0 样例
- 预期：无效：`structure-instance-multiple-parents`
- 覆盖：设计基线 §12.1 第 7 类（结构错误：多父节点）

## 目的

叶子实例 `leaf` 同时被 `branch-a` 与 `branch-b` 的 `item` Slot 接入。

## 预期诊断

- `structure-instance-multiple-parents`

## 说明

单父规则：非 Root 实例只能有一个父级 Binding。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
