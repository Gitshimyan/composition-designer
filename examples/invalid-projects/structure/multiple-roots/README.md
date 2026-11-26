# Multiple Roots

- 状态：M0 样例
- 预期：无效：`toml-duplicate-key`
- 覆盖：设计基线 §12.1 第 7 类（结构错误：多个 Root）

## 目的

Composition 中重复声明 `root`，试图表达多个根。

## 预期诊断

- `toml-duplicate-key`

## 说明

v0.1 的 `root` 是单值字段，多个 Root 在模型上不可表达；重复键在 TOML 语法层被拒绝。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
