# 011 — M2 保留格式 TOML 写回

- 状态：done
- 目标：把 009–010 的模型事务落到 composition TOML：用 `toml_edit` 做局部编辑，保留未编辑
  部分的注释/排版，写回后重新校验并刷新 `source_digest`；失败回滚文件与工作区。
- 授权：Modify（editor 依赖与写回实现）
- 变更声明：`changes/011-toml-writeback.yaml`

- coverage_status：complete_with_deferred_items
- open_items：UW-014、UW-015、UW-017、UW-018、UW-019、UW-027
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

010 结束时，事务只更新内存模型，TOML 仍是旧内容，Revision 的 `source_digest` 也不一致。
011 增加 `ProjectSession`：

- 打开项目时同时保留已验证模型与 composition `DocumentMut`；
- 命令先经 010 事务校验，再把变更局部写入 TOML；
- 写回使用临时文件 + 备份 + 重新解析；失败恢复原文件与工作区；
- 成功后刷新 SourceMap/digest，更新当前 Revision 与历史记录。

v0.1 只编辑单个 composition 入口；模块定义、`.designer/**` 与 project.toml 不在写回范围。

## 清单

- [x] editor 接入 `composition-toml` 与 `toml_edit`
- [x] `ProjectSession::open`：加载项目、保留 DocumentMut、保留工作区
- [x] `ProjectSession::prepare`：只读规划，不写文件
- [x] `ProjectSession::apply`：模型事务 + TOML 局部编辑 + 写回
- [x] 12 个命令的 TOML 编辑：实例/配置/Binding/Connection
- [x] 删除实例级联清理 Binding/Connection/dependencies；重命名同步引用
- [x] 临时文件 + 备份 + 重新校验；失败恢复文件与工作区
- [x] 成功后刷新 SourceMap/digest、Revision 与历史
- [x] 测试：注释保留、digest 刷新、非法事务文件不变、prepare 不写、创建/删除/重命名/连接、无临时文件残留
- [x] `cargo test` / `clippy` / `fmt` 全绿；更新 command schema、plan、README、docs/ai、任务卡

## 验收

- `cargo test --workspace`：**130 passed / 0 failed**
  （domain 58 + toml 12/15 + core 8 + cli 8 + editor 4/18 + session 7）；
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all --check` 通过；
- 修改 `title` 后文件保留 `# keep this comment` 与未编辑的 `label = "Hello"`；
- 写回后 `validate_project` 有效，`source_digest` 与工作区 Revision 一致且不同于修改前；
- 非法配置：文件逐字节不变、Revision 不变、无 `.tmp-*` / `.bak-*`；
- `prepare` 不写文件；
- 创建实例 + Binding、删除实例 + Binding、重命名、创建 Connection 写回后都能重新解析为有效模型；
- 文件格式仍是合法 TOML，未编辑部分保持原排版。

## 结果 / 遗留问题

- 结果：
  - 新增 `ProjectSession`：`open` / `prepare` / `apply`；
  - `toml_edit` 局部编辑 12 个命令，删除/重命名同步引用与 dependencies；
  - 写回策略：临时文件 → 备份原文件 → 替换 → `validate_project` → 失败恢复；
  - 成功后 `refresh_source` 更新模型 Revision、历史记录与 request 记录中的 digest；
  - 7 个 session 集成测试，workspace 130 项测试。
- 关键决策：
  - 只编辑 composition 入口；不改 project.toml/module.toml；
  - 未编辑标量/表保留原 decor；新建表采用 toml_edit 默认排版；
  - 命令先在模型上全量校验，再写文件；写回后再次全量校验；
  - 写回失败使用 `edit-transaction-failed`，候选错误保留原 L2–L7 代码。
- 遗留：
  - Revision 历史与 request 幂等记录仍只在内存，跨进程不恢复；
  - 写回是“备份 + 替换 + 回滚”，没有 fsync/崩溃恢复；
  - 只支持单 composition 入口；模块编辑、撤销/重做、外部修改同步未实现；
  - `UpdateBinding` / `UpdateConnection` / `UpdateInstanceModule` / `UnsetConfig` 已实现，
    但 session 层只有部分命令的直接测试；
  - 新建表的字段顺序与空行由 toml_edit 决定，不保证与手写风格完全一致。
- 下一步：M2 收口；M3 012 工程打开与 Designer 外壳（或先补跨进程 Revision 持久化）。
