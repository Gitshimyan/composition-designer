# Unknown Property

- 状态：M0 样例
- 预期：无效：`contract-property-unknown`
- 覆盖：设计基线 §12.1 第 6 类（属性错误：未知属性）

## 目的

实例配置包含模块契约未声明的属性 `subtitle`。

## 预期诊断

- `contract-property-unknown`

## 说明

未知属性不会被静默忽略，也不会进入有效配置。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
