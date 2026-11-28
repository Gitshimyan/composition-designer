# G09 — M7 后文档一致性修正

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：修正 M7 完成后遗留的文档漂移，使 README、Designer README 与计划对 Source 能力、测试基线和 ADR 状态的描述与实现一致。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：2026-11-28 会话审查；用户明确要求先更新文档
- 变更声明：`changes/G09-docs-consistency.yaml`

## 背景

M7（029–031）把 Source 从只读升级为可编辑，但三处文档仍停留在 M3/M6 状态：

1. `working/plan.md` 第 39、70 行仍写“ADR-0004 为 Proposed”，与 ADR-0004 的
   `Accepted（2026-11-27）` 冲突；
2. `README.md` 与 `apps/designer/README.md` 仍把 TOML Source 描述为只读，
   且 UI/宿主测试数量停留在 28 UI + 15 Tauri（实际 30 UI + 16 Tauri）；
3. `plan.md` 与 README 的“当前 GUI”摘要未收录 Source 编辑、草稿与冲突只读。

这些是描述性文档，不改变产品行为；修正属 Modify，不涉及 Schema/命令/领域模型。

## 原始目标与验收覆盖

| 需求 ID | 需求 / 验收点 | 状态 | 证据 / 说明 |
|---|---|---|---|
| REQ-01 | README 的 Source 描述与 M7 一致 | implemented | `README.md` §2 表格与 M7 行 |
| REQ-02 | Designer README 的 Source/测试数量与实现一致 | implemented | `apps/designer/README.md` |
| REQ-03 | plan 的 ADR-0004 状态与 ADR 文件一致 | implemented | `working/plan.md` §2/§4 |
| REQ-04 | 授权与决策可追溯 | implemented | `decision-history.md` D-021 |
| REQ-05 | 不修改产品代码/Schema/命令 | implemented | Manifest 范围；无 `crates/`、`apps/designer/src` diff |

## 未实现 / 被搁置事项

| UW ID | 未实现能力 | 类型 | 原因 | 所需授权 | 影响 | 后续处理 |
|---|---|---|---|---|---|---|
| 无 | — | — | — | — | — | — |

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| — | — | — | — | — |

## 清单

- [x] 修正 `plan.md` 的 ADR-0004 状态与“只读 Source”摘要
- [x] 修正 `plan.md` 的 R1 遗留句（029 已完成、030/031 已完成）
- [x] 修正 `README.md` §2 状态表与 M7 行
- [x] 修正 `apps/designer/README.md` 的 Source、测试数量与链路列表
- [x] 记录 D-021 授权
- [x] 回写本卡结果与 plan 日志

## 验收

- 三处文档不再声称 Source 只读或 ADR-0004 Proposed；
- UI/宿主测试数量与实际用例数一致（30 / 16）；
- Manifest 的 targets 覆盖全部实际改动；
- 无产品代码、Schema 或命令变更；
- `git diff --check` 通过。

## 结果 / 验证

- 实际执行：
  - `README.md`：§2 日期、editor 测试/命令数（44 / 15）、Source 行、M7/G06–G08 行、
    M0–M7 收口说明与模块文件边界；
  - `apps/designer/README.md`：摘要、Source 行/要点、UI 自动化（30 链路、16 host tests、
    12 个真实 ProjectSession，含 Source 文本写回）；
  - `working/plan.md`：ADR-0004 Accepted（§2、G7）、可编辑 Source 摘要、R1 遗留句、
    G09 日志行；
  - `decision-history.md`：D-021 授权记录。
- 未执行：完整构建/测试（本卡只改描述性文档，不触代码）。
- 门禁：`composition-change-gate --mode block`（工作树临时提交 diff）0 error / 0 warning，
  Manifest 与 diff 覆盖一致。
- 用例计数核对：`tests/ui/*.spec.ts` 的 `test(` = 30；`src-tauri/src/lib.rs` 的
  `#[test]` = 16（其中 12 个使用 `ProjectSession::open`）。
- 结果：三处文档与 M7 实现一致；无 `crates/`、`apps/designer/src`、Schema 或命令改动。
- 遗留 / 后续：032 Schema v2.0（已获 Rearchitect + Extend 授权）。
