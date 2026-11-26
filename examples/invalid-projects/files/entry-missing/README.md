# Entry Missing

- 状态：M0 样例
- 预期：无效：`file-project-missing`
- 覆盖：设计基线 §12.1 第 9 类（文件错误：入口缺失）

## 目的

项目目录中没有 `project.toml`，无法定位项目身份、Composition 入口与模块来源。

## 预期诊断

- `file-project-missing`

## 说明

本目录故意不提供 `project.toml`；测试应直接指向该目录。

## 验证

M1 起用 `composition-cli validate` 校验；当前以 `expected.json` 与 M0 规范核对。
