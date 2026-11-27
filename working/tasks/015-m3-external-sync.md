# 015 — M3 外部修改同步

- 状态：done
- 目标：检测外部 TOML 修改，阻止基于旧模型写回，并提供安全 Reload；完成 M3 收口。
- 授权：Extend（新 Tauri 查询/重载命令、前端轮询与冲突横幅）
- 变更声明：`changes/015-external-sync.yaml`

> 授权审计（2026-11-27）：审计建议 **Extend；若改变 Core/Protocol 的 prepare/apply 接受规则再评估 Rearchitect**；原始授权字段保持不变。见 [审计记录](../../docs/reviews/2026-11-27-authorization-audit-record.md)。

- coverage_status：complete_with_deferred_items
- open_items：UW-017
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

014 的 Designer 保存 `ProjectSession`。如果用户在外部编辑器修改 composition/project/module，
内存 Revision 与磁盘 `source_digest` 会分叉；继续 Apply 会覆盖外部修改。015 增加：

- `check_external_changes`：比较 session digest 与磁盘 digest；
- 编辑前守卫：外部 dirty 时 `prepare` / `apply` 返回 `edit-revision-conflict`；
- Reload：重新 `ProjectSession::open`，替换会话并返回新快照；
- UI：2s 轮询、冲突横幅、禁用配置编辑、Reload 按钮。

## 清单

- [x] `ExternalChangeState`：hasSession/dirty/两个 digest/disk valid/error count
- [x] `check_external_changes` Tauri 命令
- [x] `reload_project` Tauri 命令
- [x] `apply_transaction` / `prepare_transaction` 外部 dirty 守卫
- [x] 前端轮询与 external dirty 状态
- [x] 冲突横幅：Reload、禁用编辑、状态栏提示
- [x] Reload 后刷新快照/Source/选择/布局
- [x] Tauri 测试：外部追加注释 → dirty → 重新打开清除
- [x] `cargo test` / `npm run build` / `desktop` / Tauri test；更新 plan、README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**131 passed**；
- Tauri `cargo test`：**5 passed**（新增 external change 测试）；
- `npm run build`、`npm run desktop` 通过；
- 打开 minimal 后在外部追加注释：2s 内出现 “Files changed on disk”，配置 Review 禁用；
- 此时 Apply/Prepare 返回 `edit-revision-conflict`，不写 TOML；
- 点击 Reload：重新读取磁盘、canEdit 恢复、Source/选择刷新；
- 外部改为非法 TOML：Reload 后显示 invalid/诊断，编辑不可用；
- `.designer/layout.toml` 外部变化不触发项目 dirty（不在 source_digest 中）。

## 结果 / 遗留问题

- 结果：
  - `check_external_changes` / `reload_project`；
  - 写操作前磁盘 digest 守卫；
  - React 2s 轮询 + 冲突横幅 + Reload；
  - Tauri 外部修改测试通过；
  - M3 四张卡完成。
- 关键决策：
  - 用整个项目 `source_digest` 判断外部变化；注释/格式变化也算，保守但安全；
  - 外部 dirty 时只允许 Reload，不允许覆盖；
  - Reload 重建 `ProjectSession`，丢弃未提交 Review，重置到 root/自动布局；
  - 磁盘非法时 Reload 进入只读 invalid 状态；
  - 轮询而非文件系统 watcher，避免跨平台 watcher 依赖。
- 遗留：
  - 2s 轮询有延迟；大项目每 2s 重新 validate 有成本；
  - 无三向合并/局部保留；Reload 丢弃内存中的 Review；
  - Reload 后没有重新合并 `.designer/layout.toml`（节点会回到自动布局，文件仍保留）；
  - 外部修改检测只对已打开的可编辑会话生效；
  - 无前端自动化 UI 测试；Tauri 状态测试覆盖核心判定。
- 下一步：M4 016 领域协议与 CLI/JSONL。
