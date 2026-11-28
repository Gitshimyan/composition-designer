# 042 — 撤销 / 重做（Undo / Redo）

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：在 Editor 会话内提供 undo/redo，并在 Designer 暴露按钮与快捷键；每次撤销/重做生成新 Revision 并走同一 Source 验证/写回路径。
- authorization：
  - required：modify、extend
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 明确授权开始 042 undo）
- 来源：`working/plan.md` H1 042；UW-014
- 变更声明：`changes/042-undo-redo.yaml`

## 背景

编辑器已有内存 Revision 历史，但没有回退入口；误操作只能手工改 TOML。042 增加会话级
undo/redo：保存已提交文本快照，撤销/重做通过 `replace_source` 重新验证并写回，
因此不会绕过 Core 或复制一套模型。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | 提交后 undo 恢复上一文本 | implemented | editor undo tests |
| REQ-02 | redo 恢复被撤销文本 | implemented | editor/UI tests |
| REQ-03 | 新编辑清空 redo | implemented | `a_new_edit_clears_redo` |
| REQ-04 | 撤销/重做生成新 Revision | implemented | seq 断言 |
| REQ-05 | 磁盘文本与 session 一致 | implemented | roundtrip 测试 |
| REQ-06 | Designer 按钮 + Ctrl+Z/Ctrl+Shift+Z/Ctrl+Y | implemented | Playwright UI test |
| REQ-07 | 无可撤销时拒绝且不写文件 | implemented | `edit-invalid-command` |
| REQ-08 | 外部冲突不覆盖 | implemented | 走 `apply_source` 的 digest 检查 |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-015 | Revision/request 跨进程持久化 | 可靠性 | 042 仅会话内 | Modify / Extend | 重启后历史丢失 | H1 043 |
| 无 | 跨 profile/跨项目撤销 | 产品能力 | 会话绑定 profile | 需设计 | 切换后历史清空 | 不登记为需求 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 042 undo | 会话内文本快照；不持久化 | 043 单独负责持久化 | H1 拆分 |

## 清单

- [x] Editor undo/redo 栈与 Revision
- [x] Tauri undo/redo IPC + snapshot canUndo/canRedo
- [x] Designer 按钮/快捷键
- [x] Editor + Playwright 测试
- [x] 计划/UW/文档回写

## 验收

- 任意已提交事务可撤销，redo 可恢复；新编辑清空 redo；
- undo/redo 均写入磁盘、生成新 seq，并保持 TOML 格式；
- 空栈返回 `edit-invalid-command`，不写文件；
- 外部修改时拒绝撤销/重做；
- UI 按钮禁用状态与快捷键正确；全量测试通过。

## 结果 / 验证

- 实际执行：
  - `ProjectSession` 增加 `undo_stack`/`redo_stack`、`can_undo`/`can_redo`、`undo`/`redo`；
    每次成功 apply 记录 before 文本，undo/redo 通过 `replace_source` 走同一验证与写回；
  - Tauri 增加 `undo`/`redo`，Snapshot 增加 `canUndo`/`canRedo`；
  - Designer 顶栏 Undo/Redo 按钮，Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y（输入框内不拦截）；
  - 测试：editor 3 条、Playwright 1 条。
- 验证：
  - `cargo test --workspace`：**241 passed**；
  - `tauri-test`：**17 passed**；
  - `npm run test:ui`：**36 passed**；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - change-gate block（工作树临时提交）：17 个文件、49 个 Manifest，**0 error / 0 warning**。
- 未执行：跨进程历史（UW-015，043）；真实桌面窗口人工快捷键（Playwright + Tauri 测试覆盖 IPC/session）。
- 结果：误操作可在会话内回退；撤销/重做仍是普通 Revision，不破坏事务/外部冲突语义。
- 遗留 / 后续：043 Revision/request 持久化；H1 其余卡按反馈排序。
