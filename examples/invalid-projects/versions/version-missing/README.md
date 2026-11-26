# Version Missing

- 状态：M0 样例
- 预期：无效：`schema-missing-field`
- 覆盖：设计基线 §12.1 第 8 类（版本错误：模块版本缺失）

## 目的

实例缺少必需的 `version` 字段。

## 预期诊断

- `schema-missing-field`

## 说明

缺少必需字段属于 Schema 错误，不会继续做版本解析。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
