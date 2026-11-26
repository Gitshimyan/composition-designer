# Slot Single Duplicate

- 状态：M0 样例
- 预期：无效：`structure-slot-cardinality`
- 覆盖：设计基线 §12.1 第 4 类（单 Slot 重复接入）

## 目的

`content` 是 `one`，却接入了两个不同子实例。

## 预期诊断

- `structure-slot-cardinality`

## 说明

两个子实例各自只有一个父级；错误只在 Slot 基数超限。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
