# 045 — 外部修改、草稿保留与 Review 重基

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-017
- 目标：研究外部修改检测与合并策略；实现 Source 草稿跨外部修改保留、打开中的 Review 在 Reload 后语义重基与冲突 UX。
- authorization：
  - required：modify、extend
  - approval：
    - modify：default
    - extend：granted（用户 2026-11-28 明确授权 045，并要求研究后自行决策）
- 来源：`working/plan.md` H1 045；UW-017
- 变更声明：`changes/045-external-drafts.yaml`

## 背景

现状：2s 轮询检测 digest；外部 dirty 时阻止 apply；Reload 重建 session 并清空 Review 与
Source 草稿。045 需要决定 watcher/三向合并是否引入，并避免用户输入丢失。

## 研究结论（ADR-0008）

| 问题 | 决策 | 理由 |
|---|---|---|
| 原生 watcher | 不引入；保留 2s 轮询 | 轮询跨平台、无依赖、覆盖 rename/保存；watcher 不解决合并 |
| 自动三向 TOML 合并 | 不做 | 格式/语义/冲突 UI 复杂，易静默覆盖；命令 + 校验 + Diff 已可审查 |
| Source 草稿 | 跨外部 Reload 保留 | 草稿经 `replace_source` 重新验证，安全 |
| 打开中的 Review | Reload 后更新 base_revision 并重新 prepare | 命令级重基；失败显示拒绝 |
| Layout 合并 | 不做 | 表现数据；保留在 UW-017 |

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | 外部修改不丢 Source 草稿 | implemented | SourceView + Playwright |
| REQ-02 | Reload 后草稿可继续 Review | implemented | `keeps a source draft...` |
| REQ-03 | 打开中的 Review 不丢、按新 Revision 重基 | implemented | `rebases an open review...` |
| REQ-04 | Review 内可见冲突并提供 Reload | implemented | `ChangeReview` external panel |
| REQ-05 | 不自动合并/不强制覆盖 | implemented | 仅重新 prepare，失败拒绝 |
| REQ-06 | 决策记录 | implemented | ADR-0008、D-035 |
| REQ-07 | watcher/三向合并登记 | implemented | UW-017 继续 deferred |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| UW-017 | 原生 watcher、自动三向合并、layout 合并 | 产品能力 | 研究后收益/风险不匹配 | Modify / Extend | 检测 ≤2s；不自动合并 | 按真实反馈再评估 |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-28 | 外部 watcher/三向合并/草稿保留 | 保留轮询；草稿保留 + Review 重基；watcher/三向合并 deferred | 研究后决定 | ADR-0008 |

## 清单

- [x] 研究/ADR-0008
- [x] Source 草稿保留与 Discard 新基线
- [x] Review 冲突面板 + Reload 重基
- [x] Playwright 2 条新测试
- [x] 计划/UW/文档回写

## 验收

- 外部 dirty 时草稿不丢；Reload 后可继续 Review；
- 打开中的 Review 在 Reload 后保留，并按新 Revision 重新 prepare；
- Review 内可见冲突与 Reload 操作；未重基前 apply 禁用；
- 不自动合并、不强制覆盖；失败显示 Core 诊断；
- 全量测试与 UI 测试通过；变更范围与 Manifest 一致。

## 结果 / 验证

- 实际执行：
  - SourceView：文本更新时若本地 dirty 则保留草稿，只更新磁盘基线；Discard 回到新基线；
  - App：Reload 不再清空 pendingReview；用新 Revision 重基并重新 prepare；
  - ChangeReview：外部 dirty 面板 + “Reload and revalidate”，重基前禁用 apply；
  - ADR-0008：保留轮询、不做自动三向合并；UW-017 继续 deferred。
- 验证：
  - `npm run test:ui`：**38 passed**（新增草稿保留、Review 重基）；
  - `cargo test --workspace`：**251 passed**（045 无 Rust 变更，沿用 044 基线）；
  - `tauri-test`：**17 passed**（045 无 Tauri 变更）；
  - `npm run build`：通过；
  - `cargo clippy` / `fmt`：045 无 Rust 变更，沿用 044 通过状态；
  - change-gate block（工作树临时提交）：16 个文件、52 个 Manifest，**0 error / 0 warning**。
- 未执行：原生 watcher、自动三向合并、layout 合并（UW-017）；真实桌面人工外部修改（Playwright IPC fixture 覆盖）。
- 结果：外部修改不再丢失 Source 草稿或打开的 Review；冲突路径可审查且不静默覆盖。
- 遗留 / 后续：046 真实 WebView2 smoke；UW-017 剩余项按反馈评估。
