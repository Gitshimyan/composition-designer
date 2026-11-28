# 043 — Revision/request 跨进程持久化

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：把 Editor 的 Revision 历史与 request 幂等记录持久化到项目 `.designer/state/`，重启/新进程后恢复 seq 与 replay；状态过期或损坏时丢弃。
- authorization：
  - required：modify、extend
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 明确授权开始 043）
- 来源：`working/plan.md` H1 043；UW-015
- 变更声明：`changes/043-revision-state.yaml`

## 背景

042 的 undo 与 010–011 的 Revision 历史只存在内存；重启后 `seq` 归零、重复 request 会再次写文件。
043 增加按项目/profile 的状态缓存，使 `base_revision` 与 request 幂等跨进程有效；不改变
Core/TOML/Schema。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | 持久化位置按项目/profile | implemented | `.designer/state/<profile>.json` |
| REQ-02 | 重启恢复 seq/history | implemented | editor persistence 测试 |
| REQ-03 | request 跨进程 replay 不二次写 | implemented | editor + CLI 测试 |
| REQ-04 | 磁盘 digest 不匹配丢弃状态 | implemented | `stale_state_is_discarded` |
| REQ-05 | 损坏状态不阻塞打开 | implemented | `corrupt_state_is_ignored` |
| REQ-06 | profile 状态隔离 | implemented | `profiles_keep_separate_state` |
| REQ-07 | 不持久化 undo 文本栈 | implemented | 重启后 canUndo=false |
| REQ-08 | 不修改 Core/TOML/Schema | implemented | 仅 editor/state + serde |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-019 | fsync 级崩溃恢复 | 可靠性 | 043 只做状态缓存 | Modify | 断电窗口仍存在 | H1 044 |
| 无 | 跨 profile/跨机同步状态 | 产品能力 | 状态本地、按 profile | 需设计 | 不共享 | 不登记为需求 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | Revision/request 持久化 | 只持久化 Revision/request，不含 undo 文本 | 避免重复实现 undo；043 聚焦幂等 | H1 拆分 |
| 2026-11-28 | `.designer/state` | 已忽略目录；best-effort 写入 | 状态是缓存，写失败不应破坏编辑 | 设计选择 |

## 清单

- [x] 状态文件/格式与原子写入
- [x] 打开时恢复 history/requests/source_requests
- [x] 每次提交/undo/redo 后保存
- [x] 过期/损坏/profile 隔离测试
- [x] CLI 跨进程 replay 测试
- [x] 文档/计划/UW 回写

## 验收

- 重启后 `seq`、history、Source 正确；相同 request 返回 replayed 且文件字节不变；
- 磁盘 digest 变化/状态损坏时忽略状态，不阻塞打开；
- profile 状态互不影响；
- 状态文件在 `.designer/`（已忽略）；不写入 `composition/`；
- 全量测试、clippy、fmt 通过。

## 结果 / 验证

- 实际执行：
  - `state.rs`：`SessionState {version, profile, base_revision, base_source_text, commits}`；
    `CommitState {request_id, transaction, revision, source_text, diff}`；
    `<root>/.designer/state/default.json` 或 `<profile>.json`，临时文件 + rename；
  - 打开时用提交后的 Source 文本重建模型，校验每步有效；不持久化完整模型；
  - 恢复 history、workspace requests、Source 请求；过期/损坏/profile 不匹配直接丢弃；
  - 每次成功 apply/undo/redo 后 best-effort 保存；保存失败不改变编辑结果；
  - `ModelDiff`/`ObjectId` 增加 Deserialize（状态文件需要）；无 Schema/协议变化。
- 验证：
  - `cargo test --workspace`：**247 passed**（editor +5、CLI +1）；
  - `tauri-test`：**17 passed**；
  - `npm run test:ui`：**36 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - change-gate block（工作树临时提交）：16 个文件、50 个 Manifest，**0 error / 0 warning**。
- 未执行：断电/fsync（044）；真实桌面重启人工验证（editor/CLI/Tauri 测试覆盖恢复路径）。
- 结果：Revision 与 request 幂等跨进程有效；状态可安全丢弃；不改变项目语义文件。
- 遗留 / 后续：044 fsync/崩溃恢复；045 外部合并；046 WebView2 smoke。
