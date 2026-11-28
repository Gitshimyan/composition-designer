# ADR-0008 — 045 外部修改检测与草稿策略

- 状态：**Accepted（2026-11-28，045）**
- 授权：用户 2026-11-28 明确授权 045，并要求不确定点研究后自行决策
- 关联：[`unresolved-work.md`](../../working/decisions/unresolved-work.md) UW-017 ·
  [`working/plan.md`](../../working/plan.md) H1 045
- 性质：产品/交互策略；不改变 Schema、协议或事务语义

## 背景

当前实现用 2s 轮询比较 session digest 与磁盘 digest；外部修改后阻止 apply，Reload 会重建
session，并清空 Review 与 Source 草稿。045 需要决定：

1. 是否引入原生文件 watcher；
2. 是否做自动三向合并；
3. 外部修改时如何保留用户草稿。

## 决策

### D1 — 保留 2s 轮询，不引入原生 watcher

研究后不引入 `notify`/OS 事件：

- 轮询已跨平台、无新依赖，并能覆盖保存、删除、替换、编辑器临时文件等变化；
- 2s 延迟对当前单项目桌面编辑可接受；
- 原生 watcher 需要事件去重、debounce、rename 语义、Tauri event 桥与 Linux/macOS 差异，
  但不解决“如何合并”问题；
- 轮询逻辑已经是 digest 比较，与 Core 的 Revision 判定一致，风险更低。

结论：保留轮询；把“原生 watcher”留在 UW-017。

### D2 — Source 草稿跨外部修改保留

- SourceView 在外部修改导致 `text` 更新时，如果本地 doc 是 dirty，则不覆盖草稿；
- 更新“磁盘基线”文本，Discard 回到新基线；
- 草稿继续通过 `replace_source` 提交，由 Core 重新验证；是否可应用由 Review 决定；
- 切换项目/关闭项目仍丢弃草稿（不跨项目保存）。

### D3 — 打开中的 Review 在 Reload 后语义重基

- 不自动丢弃 Review；
- Reload 后把事务的 `base_revision` 更新为新 Revision，重新 `prepare_transaction`；
- 显示新的 Diff/诊断；如果命令在新模型上不再合法，Review 显示拒绝，用户可取消；
- 这是“命令级重基”，不是文本三向合并；不会把外部改动静默合并进命令结果。

### D4 — 不做自动三向 TOML 合并

- 文本三向合并需要行/表/数组语义、格式保留、冲突标记与用户裁决；
- 现有 Core 的命令 + 全量校验 + Diff 已提供可审查的重基路径；
- 自动合并若实现不当会静默覆盖外部修改，风险高于收益。

结论：不做自动三向合并；UW-017 保留。

### D5 — Layout 合并不在 045 范围

`.designer/layout.toml` 是表现数据；外部修改布局不阻塞语义编辑，仍由现有加载/保存覆盖。
Layout 合并留在 UW-017。

### D6 — 冲突 UX

- 外部 dirty 时：顶部 banner 提示；Review 内显示“Files changed on disk”，提供
  “Reload and revalidate”；
- Reload 后草稿/Review 不丢；应用仍要求新的 Review 被接受；
- 没有“强制覆盖磁盘”的按钮；写入仍受 Revision/digest 保护。

## 备选方案

| 方案 | 优点 | 缺点 | 未采用原因 |
|---|---|---|---|
| 原生 watcher | 近实时 | 新依赖、事件语义/平台差异、仍需合并策略 | 轮询已足够；留给 UW-017 |
| 自动三向 TOML 合并 | 减少人工 | 格式/语义/冲突 UI 复杂，易静默覆盖 | 风险高于收益 |
| Reload 时清空草稿（现状） | 简单 | 丢用户输入 | 045 明确要保留 |
| 事务级重基 | 复用 Core 校验/Diff | 命令可能在新模型上失败 | 失败会显示拒绝，安全 |

## 后果

- 正面：外部修改不再丢 Source 草稿或打开的 Review；冲突路径可审查；无新依赖。
- 成本：外部检测仍有 ≤2s 延迟；Review 可能重基后被拒绝；不自动合并。
- 风险：Source 草稿是整文件文本，用户可能把外部改动“覆盖”为草稿内容；但必须经过
  Review 与 Core 校验，且 Diff 可见。
- 复审条件：需要亚秒级检测、多人/多进程编辑、或自动合并需求时重新评估 UW-017。

## 关联

- 实现：`apps/designer/src/source/SourceView.tsx`、`App.tsx`、`ChangeReview.tsx`；
- 测试：`source.spec.ts`（草稿保留）、`app.spec.ts`（Review 重基）；
- 未决：UW-017（原生 watcher、自动三向合并、layout 合并）。
