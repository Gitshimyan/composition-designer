# TOML Syntax Error

- 状态：M0 样例
- 预期：无效：`toml-syntax-error`
- 覆盖：设计基线 §12.1 第 9 类（文件错误：TOML 语法错误）

## 目的

Composition 文件的表头未闭合，TOML 1.0 解析失败。

## 预期诊断

- `toml-syntax-error`

## 说明

语法错误在 L1 终止，不再尝试构建模型。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
