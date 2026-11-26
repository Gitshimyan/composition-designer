# Property Type Mismatch

- 状态：M0 样例
- 预期：无效：`contract-property-type-mismatch`
- 覆盖：设计基线 §12.1 第 6 类（属性错误：类型错误）

## 目的

`width` 声明为 integer，配置却给了字符串 `"wide"`。

## 预期诊断

- `contract-property-type-mismatch`

## 说明

TOML integer 与 string 严格区分，不做隐式转换。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
