# Slot Many

- 状态：M0 样例
- 预期：合法（无错误诊断）
- 覆盖：设计基线 §12.1 第 3 类（多实例复用）与第 4 类（Slot 基数边界：多 Slot 合法接入）

## 目的

`many` + `required = true` 的合法多值 Slot；两个 `item.card` 实例复用同一模块定义。

## 说明

`cards` 允许任意数量；本例放 2 个，满足 required。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
