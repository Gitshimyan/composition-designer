# Property Enum Invalid

- 状态：M0 样例
- 预期：无效：`contract-property-value-invalid`
- 覆盖：设计基线 §12.1 第 6 类（属性错误：枚举值错误）

## 目的

`theme` 的枚举值是 `light` / `dark`，配置给了 `sepia`。

## 预期诊断

- `contract-property-value-invalid`

## 说明

枚举只接受声明的字符串值。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
