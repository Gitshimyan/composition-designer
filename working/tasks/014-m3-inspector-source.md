# 014 — M3 Inspector、Source 与 Diagnostics

- 状态：done
- 目标：把 013 的只读外壳接上编辑事务：Inspector 配置编辑、Change Review、只读 CodeMirror
  TOML Source、诊断定位；所有写操作经 Core/`ProjectSession`。
- 授权：Extend（新前端依赖、Tauri 编辑命令、Source 面板）
- 变更声明：`changes/014-inspector-source.yaml`

## 背景

013 已有 Canvas/Tree 与只读 Inspector。014 增加：

- Tauri 会话状态：打开有效项目时保留 `ProjectSession`；
- `prepare_transaction` / `apply_transaction` / `read_source`；
- Inspector 属性编辑：Set/Unset Config，先预览 Diff/诊断，再原子写回；
- Change Review：显示命令数、Diff summary、受影响对象、诊断；
- 底部 Diagnostics / TOML Source 双 Tab；CodeMirror 6 只读展示 composition TOML；
- 点击诊断跳转到 Source 对应行；默认选中 root。

## 清单

- [x] Tauri `SessionState` + `apply_transaction` / `prepare_transaction` / `read_source`
- [x] `ProjectSnapshot.canEdit`
- [x] `Command` / `Transaction` / `ConfigValue` / ID / Version / Revision 反序列化
- [x] 契约 JSON 元素显式带 `id`（Inspector 属性定位）
- [x] Inspector 配置编辑器：string/integer/float/boolean/enum；array 只读
- [x] Change Review：Diff、诊断、Apply/Cancel
- [x] CodeMirror 6 只读 TOML Source（StreamLanguage TOML）
- [x] Diagnostics 行点击跳转 Source 行
- [x] 默认选中 root；编辑后保留布局与选择
- [x] `npm run build`、`npm run desktop`、Tauri/root 测试、真机验证
- [x] 更新 spec、plan、README、docs/ai、任务卡

## 验收

- `npm run build`：80 modules，通过；`npm run desktop` 通过；
- `cargo test --workspace`：**131 passed**；Tauri `cargo test`：**4 passed**；
- 打开 `examples/minimal-project`：默认选中 `app`，Inspector 显示配置输入；
- 配置编辑生成事务，`prepare_transaction` 返回 Diff/诊断；Apply 后 TOML 写回、
  `source_digest` 刷新、Source 更新；
- 非法值在 Review 中显示诊断且 Apply 禁用；
- Source Tab 加载 CodeMirror、显示 TOML、正文只读；
- 诊断点击可切到 Source 并定位到文件行；
- 编辑不改变 Canvas 布局；无第二套模型。

## 结果 / 遗留问题

- 结果：
  - Tauri 持有 `Mutex<Option<ProjectSession>>`，打开有效项目即可编辑；
  - `prepare_transaction` / `apply_transaction` 返回 `EditResponse`（Diff/诊断/快照）；
  - Inspector 配置编辑器 + Change Review + 默认 root 选中；
  - CodeMirror 6 只读 Source 与诊断行跳转；
  - 契约 JSON 增加显式 `id`，CLI inspect/resolve golden 已同步；
  - root 131 tests / Tauri 4 tests / 前端构建通过。
- 关键决策：
  - TOML Source 014 只读；直接文本编辑需要新的 source transaction，留后续；
  - Inspector 只实现 Set/Unset Config；Binding/Connection 编辑仍走 Core 命令，UI 后续；
  - 编辑前 `prepare` 预览，Apply 后重新读取 Source；
  - array 属性只读；enum 用 select；
  - 默认选中 root，编辑后按 `revision.seq` 重置局部输入。
- 遗留：
  - Source 只读；模块文件诊断不会打开模块 Source；
  - 没有前端自动化 UI 测试；配置界面已人工验证，Review/Source 以构建与代码路径验证；
  - 无多命令批量 Review；无 Binding/Connect 编辑 UI；
  - bundle 约 841KB（CodeMirror + xyflow），未做 code splitting；
  - 编辑历史/幂等仍只在进程内。
- 下一步：015 外部修改同步。
