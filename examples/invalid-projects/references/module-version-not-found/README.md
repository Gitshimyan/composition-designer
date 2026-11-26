# Module Version Not Found

- 状态：M0 样例
- 预期：无效：`ref-module-version-not-found`
- 覆盖：设计基线 §12.1 第 8 类（版本不可用）

## 目的

模块 ID 存在，但实例请求的精确版本不存在。

## 预期诊断

- `ref-module-version-not-found`

## 说明

不会退化为 `ref-module-not-found`，也不会自动选择 `1.0.0`。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
