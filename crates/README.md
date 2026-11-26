# crates/

v0.1 Core 的目标 crate 布局（设计基线 §10.2）。**首个实现不一定一次拆成 7 个 crate**：
如果早期拆分带来不必要的边界成本，可以先在一个 crate 内按模块隔离，再按依赖方向拆分
（拆分决策在 005 冻结）。

| crate | 职责 | 允许依赖 |
|---|---|---|
| `composition-domain` | 领域类型与不变量 | 无（仅标准库 / serde） |
| `composition-toml` | TOML 读取、Source Map、保留格式修改 | domain |
| `composition-resolver` | 模块解析与规范化结果 | domain, toml |
| `composition-validator` | Schema、引用、契约校验 | domain, resolver |
| `composition-editor` | 命令、事务、Revision、Diff、Undo | domain, resolver, validator, toml |
| `composition-protocol` | 机器可读命令与结果 Schema | domain |
| `composition-app` | 应用服务编排 | 以上全部 |

规则：

- 依赖只能沿上表方向，不能反向；
- `composition-domain` 不依赖 Tauri、GUI、文件系统实现或协议细节；
- 解析、校验、事务规则只实现一次，GUI / CLI / Agent Adapter 复用同一组命令；
- 新增 crate 或改变依赖方向需要 Extend / Rearchitect 授权。
