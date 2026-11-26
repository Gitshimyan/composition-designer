# Module Not Found

- 状态：M0 样例
- 预期：无效：`ref-module-not-found`
- 覆盖：设计基线 §12.1 第 5 类（非法引用：模块不存在）

## 目的

Root 引用的模块 ID 在 `module_roots` 中不存在。

## 预期诊断

- `ref-module-not-found`

## 说明

`modules/` 目录存在但为空，因此不会额外触发 `file-module-root-missing`。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
