# 013 — M3 Canvas 与 Tree

- 状态：done
- 目标：在 012 的只读外壳上接入 `@xyflow/react`，用 Composition Model 渲染实例节点与结构
  Binding 边；增加结构 Tree、选择/聚焦、拖动节点与 `.designer/layout.toml` 持久化。
- 授权：Extend（新前端依赖、新布局文件与 Tauri 命令）
- 变更声明：`changes/013-canvas-tree.yaml`

## 背景

012 已有只读 `ProjectSnapshot` 与 Explorer/Composition/Inspector/Diagnostics。013 把中间的
只读卡片预览升级为真正的 Canvas，并让 Explorer 提供可点击的结构树。边界：

- Canvas/Tree 只消费 `ProjectSnapshot`，不解析 TOML、不修改 Composition；
- 节点坐标只写 `.designer/layout.toml`（表现层，Core 不读取）；
- 013 不做属性编辑/命令提交（014）；Connection 在 Canvas 中暂不单独绘制；
- 选择与聚焦是只读交互；Inspector 展示选中实例的只读信息。

## 清单

- [x] 前端加入 `@xyflow/react`
- [x] Tauri 命令 `load_layout` / `save_layout`（`.designer/layout.toml`）
- [x] 自动布局：按 Binding 层级生成节点位置
- [x] 自定义实例节点：root/composite/primitive、模块版本、配置/依赖计数
- [x] Binding 边：parent → child，slot 标签，只读
- [x] Explorer 结构 Tree：层级、slot、选择、点击聚焦
- [x] Canvas：拖动节点、选择、fit view、重置布局、错误/空状态
- [x] Inspector 展示选中实例的只读详情
- [x] 布局加载/保存；拖动后写 `.designer/layout.toml`；重新打开恢复
- [x] 布局写入不影响 Composition/Revision
- [x] Rust 布局 round-trip 测试；`npm run build`、Tauri build/test、真机验证
- [x] 更新 plan、README、docs/ai、任务卡

## 验收

- `npm run build` 通过；`npm run desktop` 通过；`cargo test`（Tauri）4 passed；
- 打开 `examples/minimal-project`：2 个节点、1 条 Binding 边、root 标记；
- 打开 `examples/edge-valid/slot-many`：root + 2 子节点、2 条边（实现按 Binding 生成）；
- Tree 点击会选中并触发 Canvas `fitView` 聚焦；Canvas 点击会选中节点并更新 Inspector；
- 拖动节点后调用 `save_layout` 写 `.designer/layout.toml`；重新打开时合并保存位置；
- `.designer/layout.toml` 只含 `x/y`，不改变 Composition、诊断或 digest；
- 非法项目仍显示模型（若可构建）与诊断；无模型时显示空/错误态；
- Canvas/Tree 不提交编辑命令，不写 `composition/**`。

## 结果 / 遗留问题

- 结果：
  - `@xyflow/react` 12：自定义实例节点、Binding 边、Fit/Reset、Controls；
  - Explorer 结构 Tree：层级、slot、选择与聚焦；
  - Inspector 只读展示选中实例的模块、契约计数、配置/依赖计数；
  - Tauri `load_layout` / `save_layout`，格式见 `docs/spec/designer-layout-v1.md`；
  - 4 个 Tauri 测试（含布局 round-trip）；前端构建通过；真机验证 minimal Canvas/Tree。
- 关键决策：
  - Canvas 只画结构 Binding；Port Connection 暂不画（014 再定）；
  - 自动布局按 Binding 深度分行，保存的位置按实例 ID 合并；
  - `.designer/layout.toml` 是表现层，不进入 Core、Revision 或 `source_digest`；
  - 布局保存失败只显示状态，不阻塞项目读取；
  - `--topmost` 仅用于桌面验收截图，不影响正常启动。
- 遗留：
  - 拖动/选择/聚焦有代码与布局 round-trip 测试，但没有前端自动化 UI 测试；
  - Connection 尚未在 Canvas 单独绘制；Inspector 仍只读；
  - 布局保存是直接写文件，没有 fsync/崩溃恢复；
  - 构建时为避开 esbuild 临时目录 ACL，将 TEMP 指向 `apps/designer/.tmp`；
  - 窗口仍未按显示器工作区自适应。
- 下一步：014 Inspector / Source / Diagnostics 编辑与 CodeMirror。
