# Slot Required Missing

- 状态：M0 样例
- 预期：无效：`structure-slot-cardinality`
- 覆盖：设计基线 §12.1 第 4 类（Slot 基数边界：必需 Slot 缺失）

## 目的

`content` 是 `one` + required，但没有任何 Binding 接入。

## 预期诊断

- `structure-slot-cardinality`

## 说明

唯一预期错误是 Slot 基数为 0；Root 与模块解析均正常。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
