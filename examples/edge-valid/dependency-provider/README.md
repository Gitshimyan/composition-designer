# Dependency Provider

- 状态：M0 样例
- 预期：合法（无错误诊断）
- 覆盖：设计基线 §5.4 / §12.1 第 5 类补充（依赖显式解析）

## 目的

一个依赖用实例映射显式指定提供者，另一个依赖靠项目内唯一匹配自动解析；依赖不是 Binding。

## 说明

`app` 的 `logger` 依赖显式映射到 `logger-1`；`engine` 未映射，但全项目只有一个 `logging.console` 实例，故唯一解析成功。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
