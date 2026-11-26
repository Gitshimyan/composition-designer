# crates/

v0.1 Core 的 crate 布局（设计基线 §10.2）。**不要求一次拆成 7 个 crate**：
M1 005 先建立 `composition-domain`，其余按任务与依赖方向逐步加入。

| crate | 状态 | 职责 | 允许依赖 |
|---|---|---|---|
| `composition-domain` | 已建立（005，58 项测试） | 领域类型与不变量 | 无（仅标准库） |
| `composition-toml` | 待 006 | TOML 读取、Source Map、保留格式修改 | domain |
| `composition-resolver` | 待 007 | 模块解析与规范化结果 | domain, toml |
| `composition-validator` | 待 007 | Schema、引用、契约校验 | domain, resolver |
| `composition-editor` | 待 M2 | 命令、事务、Revision、Diff、Undo | domain, resolver, validator, toml |
| `composition-protocol` | 待 M4 | 机器可读命令与结果 Schema | domain |
| `composition-app` | 待 M3/M4 | 应用服务编排 | 以上全部 |

规则：

- 依赖只能沿上表方向，不能反向；
- `composition-domain` 不依赖 Tauri、GUI、文件系统实现或协议细节；
- 解析、校验、事务规则只实现一次，GUI / CLI / Agent Adapter 复用同一组命令；
- 新增 crate 或改变依赖方向需要 Extend / Rearchitect 授权。
