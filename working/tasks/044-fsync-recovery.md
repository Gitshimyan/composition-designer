# 044 — fsync / 崩溃恢复

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：让 Composition 写回在崩溃/断电窗口下可恢复：临时文件 fsync、备份、rename、目录 fsync（Unix）、校验后清理；打开时恢复 `.bak-*`。
- authorization：
  - required：modify
  - approval：
    - modify：default（用户 2026-11-28 明确授权开始 044）
- 来源：`working/plan.md` H1 044；UW-019
- 变更声明：`changes/044-fsync-recovery.yaml`

## 背景

011 已用临时文件 + `.bak-*` + rename，但未 fsync 文件或目录；断电时可能出现 0 字节/未落盘
文件。044 在不改变事务/TOML 语义的前提下补写屏障与恢复窗口测试。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | 临时文件写后 fsync | implemented | `commit_text` |
| REQ-02 | 文件 fsync 失败中止写回 | implemented | 临时文件删除、原文件不变 |
| REQ-03 | rename 后目录 fsync（Unix） | implemented | `sync_directory_best_effort` |
| REQ-04 | 备份恢复缺失入口 | implemented | `recover_composition_file` |
| REQ-05 | 已有入口时忽略残留备份 | implemented | crash test |
| REQ-06 | 残留 tmp 不阻塞打开 | implemented | crash test |
| REQ-07 | 成功提交清理 tmp/bak | implemented | crash test |
| REQ-08 | 不改 Schema/事务语义 | implemented | 仅 editor 写回 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-017 | 外部 watcher/三向合并/草稿保留 | 产品能力 | 044 只做写屏障 | Modify / Extend | 外部修改仍轮询/Reload | H1 045 |
| UW-012 | 真实 WebView2 smoke | 工程能力 | 044 不涉及 GUI | 待评估 | 无真实窗口自动化 | H1 046 |
| 无 | Windows 目录 fsync | 平台限制 | Windows 无 portable dir fsync | — | 目录项持久性 best-effort | 文档标注；文件 fsync + rename 为主 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | fsync/崩溃恢复 | 只覆盖 Composition 写回；状态缓存不强制 fsync | 044 聚焦语义文件；`.designer/state` 可重建 | H1 拆分 |

## 清单

- [x] 临时文件写 + fsync
- [x] 备份/rename/目录 fsync
- [x] 崩溃窗口恢复与残留清理
- [x] crash recovery 测试
- [x] 文档/计划/UW 回写

## 验收

- 写回顺序：写临时文件 → `sync_all` → 原文件改备份 → rename → 目录同步（Unix）→ 校验；
- 临时文件 fsync 失败：删除临时文件，原入口保持不变；
- 入口缺失 + `.bak-*`：打开时恢复最新备份；
- 入口存在 + 残留备份：使用入口，不消费备份；
- 残留 `.tmp-*`：忽略，不阻塞打开；
- 成功提交：无 `.tmp-*` / `.bak-*` 残留；v1/v2/v3 测试保持通过。

## 结果 / 验证

- 实际执行：
  - `commit_text` 使用 `OpenOptions` + `write_all` + `sync_all`；目录同步在 Unix 用
    `File::open(dir).sync_all()`，Windows no-op；
  - 备份恢复后目录同步；校验失败回滚后目录同步；
  - 新增 `crash_recovery.rs` 4 条：备份恢复、入口优先、孤立 tmp、提交清理。
- 验证：
  - `cargo test --workspace`：**251 passed**；
  - `tauri-test`：**17 passed**；
  - `npm run test:ui`：**36 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - change-gate block（工作树临时提交）：10 个文件、51 个 Manifest，**0 error / 0 warning**。
- 未执行：真实断电/VM 电源故障测试；Windows 目录 fsync（平台不提供）。
- 结果：写回具备文件级持久化屏障与可恢复备份；崩溃窗口从“可能空文件”缩小到
  “已 fsync 的新文件或可恢复备份”。
- 遗留 / 后续：045 外部 watcher/三向合并；046 WebView2 smoke。
