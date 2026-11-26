# Minimal Project

- 状态：M0 样例
- 预期：合法（无错误诊断）
- 覆盖：设计基线 §12.1 第 1 类（最小合法项目）

## 目的

一个 Root（composite）+ 一个叶子模块（primitive）的最小合法项目，含一个必需 Slot 和一个必需属性。

## 说明

Root `app` 是 composite，`hello` 通过 `content` Slot 接入。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
