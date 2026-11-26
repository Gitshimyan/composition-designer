# Nested Composition

- 状态：M0 样例
- 预期：合法（无错误诊断）
- 覆盖：设计基线 §12.1 第 2 类（嵌套组合）与第 3 类（多实例复用）

## 目的

四层嵌套（shell → page → panel → widget），并让 `ui.page`、`ui.panel`、`leaf.widget` 各被多个实例复用。

## 说明

`content` 与 `body` 使用 `many`，`main` 使用 `one`；同一模块定义被多个实例引用。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
