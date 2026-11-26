# Duplicate Binding

- 状态：M0 样例
- 预期：无效：`structure-duplicate-binding`
- 覆盖：设计基线 §12.1 第 7 类（结构错误：重复 Binding）

## 目的

相同 `(parent, slot, child)` 三元组声明了两次，只有 Binding ID 不同。

## 预期诊断

- `structure-duplicate-binding`

## 说明

保留首个 Binding；重复项不参与 Slot 基数与父级计数，因此不附加其他错误。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
