# v0.1 验收报告（M5 019）

- 状态：完成（2026-11-26）
- 设计基线：[v0.1-design.md](./design/v0.1-design.md) §12
- 测试：**146 Core/Protocol/MCP/CLI + 5 Tauri**
- 验证命令：`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`、
  `cargo fmt --all --check`、`cargo test --manifest-path apps/designer/src-tauri/Cargo.toml`、
  `npm run build` / `npm run desktop`

## A01–A18

| 编号 | 验收项 | 结果 | 证据 |
|---|---|---|---|
| A01 | 项目加载 | 通过 | `composition-toml` 项目/入口/模块来源测试；CLI/Protocol inspect |
| A02 | TOML 解析 | 通过 | 语法/位置测试、L1 数据集、`file-syntax-*` 样例 |
| A03 | Schema 验证 | 通过 | 必填/类型/未知字段测试、63 个验证错误码与 26 个数据集 |
| A04 | 模块解析 | 通过 | 模块 registry、版本/重复/缺失诊断；26/26 expected 代码集合 |
| A05 | 引用完整性 | 通过 | invalid references 数据集、Binding/Connection 解析测试 |
| A06 | 组合合法性 | 通过 | Root/父子/Slot/cardinality 测试与 `edge-valid` 数据集 |
| A07 | 配置验证 | 通过 | 属性契约/类型/默认值测试；effective_config 测试 |
| A08 | 依赖检查 | 通过 | 显式 provider、唯一自动 provider、未解析/歧义/optional 测试 |
| A09 | 确定性 | 通过 | `validation_is_deterministic`、digest 稳定性、golden JSON |
| A10 | 编辑一致性 | 通过（Source 只读） | Tauri/Protocol/MCP 共用 `ProjectSession`/Core；无第二模型 |
| A11 | 事务安全 | 通过 | editor 18 集成 + session 原子测试；失败工作区/文件不变 |
| A12 | Revision 冲突 | 通过 | stale base revision、request 幂等、外部 digest 守卫测试 |
| A13 | TOML 保留 | 通过 | `set_config_preserves_comments_and_refreshes_digest` |
| A14 | 外部变更 | 通过 | Tauri `external_change_is_detected_and_reloaded`；dirty 时拒绝写 |
| A15 | Canvas 隔离 | 通过 | `.designer/layout.toml` round-trip；布局不进入 source_digest/Revision |
| A16 | Agent 闭环 | 通过 | MCP e2e：inspect→resolve→prepare→apply→validate→source |
| A17 | 错误可操作 | 通过 | 稳定 code/severity/object/file/range；诊断排序与 source map 测试 |
| A18 | 无隐式兼容 | 通过 | 未定义字段/类型/版本均报错；单入口不静默合并 |

## 019 新增边界验证

| 项 | 结果 |
|---|---|
| 崩溃备份恢复 | 通过：`interrupted_backup_is_recovered_on_open` 恢复 `.bak-*` |
| 性能 | 通过：80 实例 + 80 Binding 的合成项目 <5s（本机测试约 0.03s） |
| 单入口边界 | 通过：`composition/parts/**` 不合并、digest 不变 |
| 版本迁移 | 通过：`schema_version = 2` → `schema-unsupported-version`，不静默迁移 |

## 已知非目标 / 限制

- v0.1 只支持单 composition 入口；多文件/import 是显式不支持项。
- Source 编辑器只读；直接文本编辑未进入事务协议。
- 崩溃恢复只处理 composition 写回 `.bak-*`；没有 fsync 级断电保证。
- Revision 历史/request 幂等只在进程内；跨进程持久化未实现。
- MCP 无 subscriptions/progress/cancel/auth；单进程单项目。
- 没有前端自动化 UI 测试；M3 视觉与交互以构建 + 真实验收窗口验证。

## 结论

v0.1 的 A01–A18 在本仓库的实现范围内全部通过。未实现项与设计基线 §12.1/§14 的
显式不支持项一致，没有被静默兼容或伪装为已支持。
