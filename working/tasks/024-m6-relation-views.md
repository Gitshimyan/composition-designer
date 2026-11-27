# 024 — M6-005 Connection / Dependency 视图

- 状态：done
- 目标：为 Binding、Connection、Dependency 提供独立视图，共享同一 Composition Model；Connection 支持创建/修改/删除，Dependency 展示解析状态与 Provider；切换视图不丢失选择上下文。
- 授权：Modify（Designer UI、Tauri host 测试、设计与协作文档）
- 变更声明：`changes/024-relation-views.yaml`
- 依赖：022–023（操作经 Core 事务与 Review）

## 背景

022/023 已覆盖 Instance 与 Binding，但 Connection 与 Dependency 只有顶部计数和模型数据，
没有独立视图：用户无法创建/修改/删除 Port Connection，也看不到依赖解析的来源、状态与候选。

M6-005 要求三个投影共享同一模型：Structure（Instance/Slot/Binding）、Connection（Port/
方向/类型/协议/连接）、Dependency（要求/Provider/解析状态），切换视图保持选择上下文。

## 范围与关键决策

1. **三个投影，一个模型**：所有视图都从 `CompositionModel` 派生，不建第二套语义模型；
   React 不解析 TOML。
2. **Connection 可编辑**：复用冻结命令 `create_connection` / `update_connection` /
   `delete_connection`，经 prepare → Review → apply；端点按端口方向分组展示，
   type/protocol/multi 与重复端点由 Core 校验。
3. **Dependency 只读**：v0.1 冻结命令集没有“设置 Dependency Provider”的命令，
   `dependency_providers` 只能通过读取/重命名/删除间接变化。本卡实现只读投影：
   要求、required/optional、候选、显式/自动 Provider、解析状态；**Provider 编辑是
   明确的 Extend 缺口**，需要用户另行授权新增命令与命令 Schema，不能在本卡偷偷扩展。
4. **共享选择上下文**：`selectedId` 由 App 持有；三个视图都高亮并允许点击选中 Instance，
   切换视图不重置选择。
5. **不新增命令、Schema、错误码或设计基线**；Dependency 编辑缺口在卡内报告。

## 清单

- [x] 读设计基线 §8.3、command-schema、Connection/Dependency 模型
- [x] 任务卡与 Change Manifest
- [x] `relations/relationModel.ts`：端口端点、连接查找、连接 ID 建议、依赖详情/候选
- [x] ConnectionView：连接列表、端口投影、选中高亮、空状态
- [x] ConnectionDialog：from/to Instance+Port、方向分组、端点元数据、Create/Update
- [x] DependencyView：按 Instance 分组、状态、显式/自动 Provider、候选、只读说明
- [x] App：Structure/Connections/Dependencies 视图切换、选择保持、连接事务接线
- [x] Fixtures/mock：端口、连接、依赖数据与 connection 命令
- [x] Playwright：视图切换/选择保持、Connection 创建/修改/删除、Dependency 投影
- [x] Tauri host：Connection 创建/修改/删除真实 `ProjectSession` 写回
- [x] 更新 plan / README / architecture-entry，回写本卡

## 验收

本地已执行（2026-11-27）：

- `cd apps/designer && npm run build`：通过（tsc --noEmit + Vite，89 modules）；
- `cd apps/designer && npm run test:ui`：**23 passed**（原 18 条 + 新增 5 条关系视图链路）；
- `cargo test --manifest-path apps/designer/src-tauri/Cargo.toml`：**14 passed**
  （原 12 条 + Connection 创建/修改/删除与非法方向拒绝 2 条真实 `ProjectSession` 测试）；
- `cargo test --workspace`：**146 passed**；
- `cargo clippy --workspace --all-targets -- -D warnings`：通过；
- `cargo fmt --all --check`：通过。

逐项核对：

- Structure / Connections / Dependencies 可切换；`selectedId` 在切换前后保持，
  关系视图选中实例后切回 Structure 会聚焦该节点；
- Connection 列表显示 id、from/to、方向、type、protocol、multi 与连接数；
  Create/Update 经 prepare → Review → apply；Delete 经 Review；取消不写入；
- 端点按方向分组；type/protocol/multi/重复由 Core 校验，非法事务显示诊断且 Apply 禁用；
- Dependency 视图显示 target module/version、required/optional、候选、
  Provider（显式/自动）与四类 status；Provider 编辑未实现，已记录为 Extend 缺口；
- 未修改 `docs/spec/**`、`docs/design/**`、`composition/` 语义文件或模块定义；
- 变更范围与 `changes/024-relation-views.yaml` 一致。

## 结果 / 遗留问题

- 结果：
  - `src/relations/relationModel.ts`：端口端点、连接查询、连接 ID 建议、
    依赖详情与候选；
  - `src/relations/ConnectionView.tsx`：连接表 + 端口投影，选中高亮；
  - `src/relations/ConnectionDialog.tsx`：方向分组的 from/to 端点选择与 Create/Update；
  - `src/relations/DependencyView.tsx`：按实例分组的只读依赖投影；
  - `src/App.tsx`：三视图切换、连接事务接线、共享 `selectedId`；
  - `tests/ui/relations.spec.ts`：5 条 UI 链路；
  - `src-tauri/src/lib.rs`：Connection 创建/修改/删除与非法方向拒绝两条真实写回测试。
- 关键决策：
  - 三个视图都直接派生自 `CompositionModel`，不建第二套模型；
  - 端点按 Port 方向分组是交互辅助，合法性仍由 Core 判定；
  - Dependency 只读：冻结命令集没有 provider 命令，不在 UI 静默扩展。
- 遗留 / 明确的 Extend 缺口：
  - Dependency Provider 编辑需要新命令（输入/前置条件/结果/错误码/Schema）并同步
    Core、CLI、MCP、Designer；本卡未做，需用户明确授权；
  - Connection View 不是图形化 Port Graph，列表/端口投影优先；
  - 视图切换保留选择，但不保留 React Flow 的 pan/zoom；
  - 完整 Before/After 分类与影响摘要属于 025。
- 下一步：025 — 完整 Change Review。
