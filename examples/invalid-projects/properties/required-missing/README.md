# Property Required Missing

- 状态：M0 样例
- 预期：无效：`contract-property-required-missing`
- 覆盖：设计基线 §12.1 第 6 类（属性错误：缺少必填属性）

## 目的

`title` 是必需属性且无默认值，实例没有提供配置。

## 预期诊断

- `contract-property-required-missing`

## 说明

必填判定在应用默认值之后：无默认值才报缺失。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
