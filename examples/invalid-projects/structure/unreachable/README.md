# Unreachable Instance

- 状态：M0 样例
- 预期：无效：`structure-unreachable-instance`
- 覆盖：设计基线 §12.1 第 7 类（结构错误：孤立实例）

## 目的

`orphan` 实例没有父级 Binding，无法从 Root 到达。

## 预期诊断

- `structure-unreachable-instance`

## 说明

Root 是 composite 但无 Slot；除不可达外没有其他错误。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
