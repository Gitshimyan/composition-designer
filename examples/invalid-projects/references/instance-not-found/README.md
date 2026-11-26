# Instance Not Found

- 状态：M0 样例
- 预期：无效：`ref-instance-not-found`
- 覆盖：设计基线 §12.1 第 5 类（非法引用：Instance 不存在）

## 目的

Binding 的 child 指向不存在的实例 `ghost`。

## 预期诊断

- `ref-instance-not-found`

## 说明

Slot 是可选单值，因此不会再触发基数错误；该 Binding 被跳过。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
