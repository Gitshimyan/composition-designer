# Slot Not Found

- 状态：M0 样例
- 预期：无效：`ref-slot-not-found`
- 覆盖：设计基线 §12.1 第 5 类（非法引用：Slot 不存在）

## 目的

第二个 Binding 引用了父模块未声明的 Slot `body`。

## 预期诊断

- `ref-slot-not-found`

## 说明

`hello` 已由合法 Binding 接入；无效 Binding 不参与父级计数，因此没有多父错误。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
