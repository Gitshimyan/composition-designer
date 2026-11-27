# G03 — 历史未实现工作回溯

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：对 001–025 做一次未实现工作回溯，填充 `working/decisions/unresolved-work.md`，并给有未决项的卡补 `coverage_status` / `open_items`。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：`docs/reviews/2026-11-27-unimplemented-work-transparency.md` §10
- 变更声明：`changes/G03-unresolved-backfill.yaml`（已创建）

## 清单

- [x] 001–025 逐卡读取结果/遗留线索
- [x] 建立未实现工作回溯报告
- [x] 新增 UW-014–UW-027 并更新登记册
- [x] 001–025 补充 coverage_status / open_items / 回溯链接
- [x] 更新 plan 未决工作总览与计数
- [x] 创建 `changes/G03-unresolved-backfill.yaml`

## 范围

1. 逐张读取 001–025 的目标、范围、清单、验收、结果，提取：
   - 未实现、只读、暂缓、后续、需要授权、范围外发现的能力；
   - 目标提到但结果没有实现证据的能力；
   - 后续任务声称解决但没有实现证据的能力。
2. 为每个未决事项分配稳定 `UW-*` ID，写入
   `working/decisions/unresolved-work.md`：
   - 类型、状态、原因、所需授权、影响、最小方案、决策选项、首次/最近复核日期；
   - 已实现事项（如 025 Provider）保留关闭证据，不重复列为待决；
   - 无法证明历史意图的事项标 `historical_status_unknown`。
3. 对仍有未决项的卡补充：
   - `coverage_status`、`open_items`；
   - 精简的“需求覆盖与未实现项”表，不重写原范围。
4. 更新 `working/plan.md` 的未决工作总览与计数。

## 非目标

- 不实现任何未决能力；
- 不修改 G1–G3 的 Schema/领域模型；
- 不把“后续任务提到过”当作解决证据；
- 不伪造当时的用户授权或决策。

## 验收

- 001–025 每张卡的所有“遗留/后续/暂缓”线索都有去向：
  `implemented` / `pending_*` / `deferred` / `rejected` / `superseded` /
  `historical_status_unknown`；
- 每个未决事项可在登记册中检索，含影响、授权、后续处理；
- 已解决事项有实现任务与测试/验收证据链接；
- 计划页与登记册一致，未决数量、等待授权数量可核对；
- 历史卡原始范围与验收文本不被删除或改写。

## 范围调整记录

| 日期 | 原始要求 | 调整后范围 | 原因 | 决策依据 |
|---|---|---|---|---|
| 2026-11-27 | 每张卡补精简的“需求覆盖与未实现项”表 | 逐卡表统一放在回溯报告；卡内补 `coverage_status`/`open_items` 与回溯链接 | 避免 25 份重复表，保持报告与登记册为单一事实源 | G03 执行决定；不改变验收标准 |

## 结果 / 遗留问题

- 结果：
  - 新建 `docs/reviews/2026-11-27-unimplemented-work-backfill.md`，覆盖 001–025 的遗留线索去向；
  - 登记册扩展到 27 项：5 项已关闭、9 项等待决策、1 项 accepted（026）、12 项延期、0 项等待授权；
  - 新增 UW-014–UW-027：撤销/重做、历史持久化、array 编辑、外部 watcher/合并、多入口、fsync、
    MCP 高级能力、Skill 评测、模块库、Binding UX、窗口/安装包、Review 依赖影响、示例载荷、边界测试；
  - 001–025 任务卡补充 `coverage_status` / `open_items` / 回溯链接，不改原始范围与验收；
  - 更新 plan、未决登记、决策历史与评审索引。
- 验证：
  - 实际执行：全仓 Markdown 相对链接检查、`git diff --check`、确认 `crates/` 与 `apps/` 无改动；
  - 未执行：`cargo test` / `npm run test:ui`（仅文档/登记变更）；
  - 未执行机器 YAML 解析（环境无解析器；G03 Manifest 人工核对）。
- 遗留：
  - UW-002–005、UW-010、UW-016、UW-023、UW-025–027 仍需用户决策或排期；
  - 026 完整 Change Review 尚未开始。
- 下一步：026 — 完整 Change Review。
