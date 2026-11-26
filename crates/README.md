# crates/

v0.1 Core 的 crate 布局（设计基线 §10.2）。M1/M2 已完成只读 Core、编辑事务与 TOML 写回；
M3 012–015 建立 `apps/designer`（Tauri 2 + React + xyflow + CodeMirror）；M4 016 建立
`composition-protocol`。桌面 host 不在本表中。

| crate | 状态 | 职责 | 允许依赖 |
|---|---|---|---|
| `composition-domain` | 已建立（005，58 项测试） | 领域类型、不变量、serde JSON | serde |
| `composition-toml` | 已建立（006，27 项测试） | TOML 加载、Source Map、source_digest | domain, toml_edit, sha2 |
| `composition-core` | 已建立（007，8 项测试） | 解码、模块注册、L1–L7 校验、CompositionModel | domain, toml, toml_edit |
| `composition-cli` | 已建立（008/016，10 项测试） | inspect / resolve / validate、query / apply / jsonl | core, domain, protocol, serde, serde_json |
| `composition-editor` | 已建立（009–011，30 项测试） | 命令、事务、Revision/幂等、结构化 Diff、保留格式 TOML 写回 | core, domain, toml, toml_edit, serde, serde_json |
| `composition-protocol` | 已建立（016，4 项测试） | Query/Command 协议、JSONL、外部冲突守卫 | core, domain, editor, serde, serde_json |
| `composition-resolver` | 暂并入 core | 模块解析与规范化结果 | domain, toml |
| `composition-validator` | 暂并入 core | Schema、引用、契约校验 | domain, resolver |
| `composition-app` | 待 M3/M4 | 应用服务编排 | core |

规则：

- 依赖只能沿上表方向，不能反向；
- `composition-domain` 不依赖 Tauri、GUI、文件系统实现或协议细节；
- 解析、校验、事务规则只实现一次，GUI / CLI / Agent Adapter 复用同一组命令；
- 先以 `composition-core` 合并 resolver/validator，等出现独立复用需求再拆分；
- M2 已完成：editor 上已有 Revision/幂等/Diff 与 `ProjectSession` TOML 写回；
- M3 将在 `apps/designer` 与 `composition-app` 上加入 Designer 外壳；
- 新增 crate 或改变依赖方向需要 Extend / Rearchitect 授权。
