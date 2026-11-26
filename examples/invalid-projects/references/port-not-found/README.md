# Port Not Found

- 状态：M0 样例
- 预期：无效：`ref-port-not-found`
- 覆盖：设计基线 §12.1 第 5 类（非法引用：Port 不存在）

## 目的

Connection 的 from 引用了 editor 未声明的 Port `missing-out`。

## 预期诊断

- `ref-port-not-found`

## 说明

端点缺失时跳过该条连接的方向/类型检查。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
