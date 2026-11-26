# Connection Errors

- 状态：M0 样例
- 预期：无效：`connection-direction-invalid`、`connection-type-mismatch`、`connection-duplicate`
- 覆盖：L6 Connection 校验（不属于 §12.1 的 12 类，但属于 M0 错误码覆盖）

## 目的

一个项目集中演示三种独立的 Connection 错误：方向、类型与重复端点。

## 预期诊断

- `connection-direction-invalid`
- `connection-type-mismatch`
- `connection-duplicate`

## 说明

所有 Port 都是 `multi=true`，因此重复连接不会额外触发度数错误；`c-direction` 的类型恰好相同，只报方向。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
