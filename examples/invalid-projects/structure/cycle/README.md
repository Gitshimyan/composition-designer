# Binding Cycle

- 状态：M0 样例
- 预期：无效：`structure-cycle`
- 覆盖：设计基线 §12.1 第 7 类（结构错误：环）

## 目的

`node-a` 与 `node-b` 互相作为对方的子节点，形成 Binding 环。

## 预期诊断

- `structure-cycle`

## 说明

环内实例不再重复报不可达；Root 的 content Slot 是可选且未使用。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
