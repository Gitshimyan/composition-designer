# Format Preserved

- 状态：M0 样例
- 预期：合法（无错误诊断）
- 覆盖：设计基线 §12.1 第 12 类（格式保留）

## 目的

合法项目，但字段顺序被打乱、含注释与空行；用于 M2 验证局部编辑只改目标节点且保留格式。

## 说明

`project.toml`、Composition 与 `module.toml` 都含注释、空行和非惯用字段顺序；M2 的往返测试应保持未编辑字节不变。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
