# examples/

v0.1 的项目样例，同时作为 M0 的契约测试数据（设计基线 §12.1）。由任务卡 004 建立。

| 目录 | 内容 |
|---|---|
| `minimal-project/` | 最小合法项目：一个 Root + 一个叶子模块 |
| `nested-composition/` | 多层 Composite 与 Primitive，多实例复用 |
| `invalid-projects/` | 按错误类别分组的非法项目：引用、属性、结构、版本、文件 |

约定：

- 每个样例是完整可打开的项目目录，入口固定为 `project.toml`；
- 非法样例按 `invalid-projects/<category>/<case>/` 组织，README 写明预期诊断代码与触发层次；
- 合法样例在 M1 的 `validate` 上应全部通过；非法样例应给出稳定错误代码；
- 样例不写 `.designer/` 表现数据，除非该样例专门验证 A15（Canvas 隔离）。

覆盖清单与验收见 [`working/tasks/004-m0-example-projects.md`](../working/tasks/004-m0-example-projects.md)。
