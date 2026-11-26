# Path Outside

- 状态：M0 样例
- 预期：无效：`file-path-escape`
- 覆盖：设计基线 §12.1 第 9 类（文件错误：路径越界）

## 目的

`workspace.composition` 使用 `../` 指向项目根之外。

## 预期诊断

- `file-path-escape`

## 说明

路径归一化后越出项目根，拒绝继续解析。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
